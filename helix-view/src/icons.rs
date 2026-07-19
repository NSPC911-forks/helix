use crate::{
    graphics::{Modifier, UnderlineStyle},
    theme::{Color, Style},
    Theme,
};
use arc_swap::ArcSwap;
use hashbrown::HashMap;
use helix_core::unicode::width::UnicodeWidthStr;
use helix_stdx::string::StackString;
use once_cell::sync::Lazy;
use serde::{de::value::MapAccessDeserializer, Deserialize};
use smartstring::LazyCompact;
use std::{
    fmt::{Display, Write},
    path::Path,
    str::FromStr,
    sync::LazyLock,
};

type SmartString = smartstring::SmartString<LazyCompact>;

/// Centralized location for icons that can be used throughout the UI.
pub static ICONS: Lazy<ArcSwap<Icons>> = Lazy::new(ArcSwap::default);

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Icons {
    fs: Fs,
    kind: Kind,
    diagnostic: Diagnostic,
    vcs: Vcs,
    dap: Dap,
    ui: Ui,
}

impl Icons {
    #[inline]
    #[must_use]
    pub const fn fs(&self) -> &Fs {
        &self.fs
    }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }

    #[inline]
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    #[inline]
    #[must_use]
    pub const fn vcs(&self) -> &Vcs {
        &self.vcs
    }

    #[inline]
    #[must_use]
    pub const fn dap(&self) -> &Dap {
        &self.dap
    }

    #[inline]
    #[must_use]
    pub const fn ui(&self) -> &Ui {
        &self.ui
    }
}

macro_rules! icon {
    ( glyph: $glyph:expr, fg: $fg:expr, padding: [$p_left:expr, $p_right:expr] ) => {
        Icon {
            glyph: Some(StackString::from($glyph)),
            style: Some(Style::from(Color::from_hex($fg).unwrap())),
            padding: Some(Padding {
                left: $p_left,
                right: $p_right,
            }),
            is_user_overridden: false,
        }
    };
    ( glyph: $glyph:expr, fg: $fg:expr ) => {
        Icon {
            glyph: Some(StackString::from($glyph)),
            style: Some(Color::from_hex($fg).unwrap().into()),
            padding: None,
            is_user_overridden: false,
        }
    };
    ( glyph: $glyph:expr, padding: [$p_left:expr, $p_right:expr] ) => {
        Icon {
            glyph: Some(StackString::from($glyph)),
            style: None,
            padding: Some(Padding {
                left: $p_left,
                right: $p_right,
            }),
            is_user_overridden: false,
        }
    };
    ( glyph: $glyph:expr ) => {
        Icon {
            glyph: Some(StackString::from($glyph)),
            style: None,
            padding: None,
            is_user_overridden: false,
        }
    };
    ( $glyph:expr ) => {
        Icon {
            glyph: Some(StackString::from($glyph)),
            style: None,
            padding: None,
            is_user_overridden: false,
        }
    };
}

macro_rules! icons {
    ( $( $key:literal => { $($body:tt)* } ),* $(,)? ) => {{
        HashMap::from_iter(
            vec![
                $(
                    (SmartString::from($key), icon!( $($body)* )),
                )*
            ]
        )
    }};
}

#[derive(Debug, Default, Eq, Clone, Copy)]
pub struct Icon {
    glyph: Option<StackString>,
    style: Option<Style>,
    padding: Option<Padding>,
    is_user_overridden: bool,
}

impl Icon {
    #[inline]
    #[must_use]
    pub fn glyph(&self) -> StackString {
        // WARN:
        // This is done so that the padding can be applied to the glyph.
        let mut glyph = StackString::new();
        write!(&mut glyph, "{self}").unwrap();
        glyph
    }

    #[inline]
    pub fn set_glyph(&mut self, glyph: &'static str) {
        self.glyph = Some(StackString::from(glyph));
    }

    #[inline]
    #[must_use]
    pub fn style(&self) -> Style {
        self.style.unwrap_or_default()
    }

    #[inline]
    pub fn set_style(&mut self, style: Style) {
        self.style = Some(style);
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        if let Some(Padding { left, right }) = self.padding {
            left as u16 + self.glyph.unwrap_or_default().width() as u16 + right as u16
        } else {
            self.glyph().width() as u16
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.glyph.unwrap_or_default().is_empty()
    }

    #[inline]
    #[must_use]
    pub fn from(glyph: &'static str) -> Self {
        Self {
            glyph: Some(StackString::from(glyph)),
            style: None,
            padding: None,
            is_user_overridden: false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_padding(mut self, left: u8, right: u8) -> Self {
        self.padding = Some(Padding { left, right });
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_no_padding(self) -> Self {
        self.with_padding(0, 0)
    }

    #[inline]
    #[must_use]
    pub const fn is_user_overridden(&self) -> bool {
        self.is_user_overridden
    }

    #[inline]
    #[must_use]
    fn patch_from_user_override(mut self, other: Self) -> Self {
        self.glyph = other.glyph.or(self.glyph);
        if let Some(other) = other.style {
            if let Some(style) = self.style {
                self.style = Some(style.patch(other));
            } else {
                self.style = Some(other);
            }
        }
        self.padding = other.padding.or(self.padding);
        self
    }
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let padding = self.padding.unwrap_or_default();

        for _ in 0..padding.left {
            write!(f, " ")?;
        }

        write!(f, "{}", self.glyph.unwrap_or_default())?;

        for _ in 0..padding.right {
            write!(f, " ")?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
struct Padding {
    left: u8,
    right: u8,
}

impl PartialEq<str> for Icon {
    fn eq(&self, other: &str) -> bool {
        self.glyph().as_str() == other
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            glyph,
            style,
            padding,
            is_user_overridden: _,
        } = self;

        *padding == other.padding && *style == other.style && *glyph == other.glyph
    }
}

impl<'de> Deserialize<'de> for Icon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(IconVisitor)
    }
}

struct IconVisitor;

impl<'de> serde::de::Visitor<'de> for IconVisitor {
    type Value = Icon;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string glyph or a map with 'glyph, 'fg', 'bg', 'padding', 'underline', or 'modifiers'"
        )
    }

    fn visit_str<E>(self, glyph: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Icon {
            glyph: Some(StackString::try_from(glyph).map_err(|err| serde::de::Error::custom(err))?),
            style: None,
            padding: None,
            is_user_overridden: true,
        })
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        #[derive(Deserialize)]
        struct UserIcon {
            glyph: Option<StackString>,
            fg: Option<StackString>,
            bg: Option<StackString>,
            modifiers: Option<Vec<String>>,
            padding: Option<Padding>,
            underline: Option<Underline>,
        }

        #[derive(serde::Deserialize)]
        struct Underline {
            style: StackString,
            color: StackString,
        }

        let icon = UserIcon::deserialize(MapAccessDeserializer::new(map))?;

        let mut style: Option<Style> = None;

        if let Some(fg) = icon
            .fg
            .map(|hex| Color::from_hex(&hex))
            .transpose()
            .map_err(|err| serde::de::Error::custom(err.to_string()))?
        {
            if let Some(s) = style {
                style = Some(s.fg(fg));
            } else {
                style = Some(Style::default().fg(fg));
            }
        }

        if let Some(bg) = icon
            .bg
            .map(|hex| Color::from_hex(&hex))
            .transpose()
            .map_err(|err| serde::de::Error::custom(err.to_string()))?
        {
            if let Some(s) = style {
                style = Some(s.bg(bg));
            } else {
                style = Some(Style::default().bg(bg));
            }
        }

        if let Some(options) = icon.modifiers {
            for option in options {
                let modifier = Modifier::from_str(&option)
                    .map_err(|err| serde::de::Error::custom(err.to_string()))?;

                if let Some(s) = style {
                    style = Some(s.add_modifier(modifier));
                } else {
                    style = Some(Style::default().add_modifier(modifier));
                }
            }
        }

        if let Some(underline) = icon.underline {
            let color = Color::from_hex(underline.color.as_str())
                .map_err(|err| serde::de::Error::custom(err.to_string()))?;

            if let Some(s) = style {
                style = Some(s.underline_color(color));
            } else {
                style = Some(Style::default().underline_color(color));
            }

            style = style.map(|style| style.underline_color(color));

            let underline_style = UnderlineStyle::from_str(&underline.style)
                .map_err(|err| serde::de::Error::custom(err.to_string()))?;

            if let Some(s) = style {
                style = Some(s.underline_style(underline_style));
            } else {
                style = Some(Style::default().underline_style(underline_style));
            }
        }

        Ok(Icon {
            glyph: icon.glyph,
            style,
            padding: icon.padding,
            is_user_overridden: true,
        })
    }
}

static KIND: LazyLock<Kind> = LazyLock::new(|| Kind {
    enable: false,
    icons: icons! {
        "file" => { glyph: "", padding: [1, 1] },
        "folder" => { glyph: "󰉋", padding: [1, 1] },
        "module" => { glyph: "", padding: [1, 1] },
        "namespace" => { glyph: "", padding: [1, 1] },
        "package" => { glyph: "", padding: [1, 1] },
        "class" => { glyph: "", padding: [1, 1] },
        "method" => { glyph: "", padding: [1, 1] },
        "property" => { glyph: "", padding: [1, 1] },
        "field" => { glyph: "", padding: [1, 1] },
        "constructor" => { glyph: "", padding: [1, 1] },
        "enum" => { glyph: "", padding: [1, 1] },
        "interface" => { glyph: "", padding: [1, 1] },
        "function" => { glyph: "", padding: [1, 1] },
        "variable" => { glyph: "", padding: [1, 1] },
        "constant" => { glyph: "", padding: [1, 1] },
        "string" => { glyph: "", padding: [1, 1] },
        "number" => { glyph: "", padding: [1, 1] },
        "boolean" => { glyph: "", padding: [1, 1] },
        "array" => { glyph: "", padding: [1, 1] },
        "object" => { glyph: "", padding: [1, 1] },
        "key" => { glyph: "", padding: [1, 1] },
        "null" => { glyph: "󰟢", padding: [1, 1] },
        "enum_member" => { glyph: "", padding: [1, 1] },
        "struct" => { glyph: "", padding: [1, 1] },
        "event" => { glyph: "", padding: [1, 1] },
        "operator" => { glyph: "", padding: [1, 1] },
        "type_param" => { glyph: "", padding: [1, 1] },
        "keyword" => { glyph: "", padding: [1, 1] },
        "color" => { glyph: "■", padding: [0, 0] },
        "value" => { glyph: "󰎠", padding: [1, 1] },
        "snippet" => { glyph: "", padding: [1, 1] },
        "reference" => { glyph: "", padding: [1, 1] },
        "text" => { glyph: "", padding: [1, 1] },
        "unit" => { glyph: "", padding: [1, 1] },
        "word" => { glyph: "", padding: [1, 1] },
        "spellcheck" => { glyph: "󰓆", padding: [1, 1] },
        "link" => { glyph: "", padding: [1, 1] },
    },
});

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct Kind {
    enable: bool,
    #[serde(flatten)]
    icons: HashMap<SmartString, Icon>,
}

impl Kind {
    #[inline]
    #[must_use]
    pub fn get(&self, kind: &str) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.icons
            .get(kind)
            .or_else(|| KIND.icons.get(kind))
            .copied()
    }

    #[inline]
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn color(&self) -> Icon {
        self.icons
            .get("color")
            .or_else(|| KIND.icons.get("color"))
            .copied()
            .expect("`color` should be populated in the Lazy impl")
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Diagnostic {
    hint: Option<Icon>,
    info: Option<Icon>,
    warning: Option<Icon>,
    error: Option<Icon>,
}

impl Diagnostic {
    #[inline]
    #[must_use]
    pub fn hint(&self) -> Icon {
        self.hint
            .unwrap_or_else(|| icon!(glyph: "○", padding: [1, 1]))
    }

    #[inline]
    #[must_use]
    pub fn info(&self) -> Icon {
        self.info
            .unwrap_or_else(|| icon!(glyph: "●", padding: [1, 1]))
    }

    #[inline]
    #[must_use]
    pub fn warning(&self) -> Icon {
        self.warning
            .unwrap_or_else(|| icon!(glyph: "▲", padding: [1, 1]))
    }

    #[inline]
    #[must_use]
    pub fn error(&self) -> Icon {
        self.error
            .unwrap_or_else(|| icon!(glyph: "■", padding: [1, 1]))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Vcs {
    enable: bool,
    branch: Option<Icon>,
    added: Option<Icon>,
    removed: Option<Icon>,
    ignored: Option<Icon>,
    modified: Option<Icon>,
    renamed: Option<Icon>,
    conflict: Option<Icon>,
}

impl Vcs {
    #[inline]
    #[must_use]
    pub fn branch(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.branch
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn added(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.added
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn removed(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.removed
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn ignored(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.ignored
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn modified(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.modified
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn renamed(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.renamed
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }

    #[inline]
    #[must_use]
    pub fn conflict(&self) -> Option<Icon> {
        if !self.enable {
            return None;
        }

        self.conflict
            .or_else(|| Some(icon!(glyph: "", padding: [1, 1])))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Fs {
    enable: bool,
    #[serde(default)]
    directory: Directory,
    #[serde(default)]
    file: File,
}

impl Fs {
    #[inline]
    #[must_use]
    pub const fn directory(&self) -> Option<&Directory> {
        if !self.enable {
            return None;
        }
        Some(&self.directory)
    }

    #[inline]
    #[must_use]
    pub const fn file(&self) -> Option<&File> {
        if !self.enable {
            return None;
        }
        Some(&self.file)
    }
}

static FILE: LazyLock<File> = LazyLock::new(|| File {
    filename: icons! {
        ".dockerignore" => {glyph: "󰡨", fg: "#0096e6", padding: [1, 1] },
        ".editorconfig" => { glyph: "", padding: [1, 1] },
        ".env" => { glyph: "", padding: [1, 1] },
        ".envrc" => { glyph: "", padding: [1, 1] },
        ".git-blame-ignore-revs" => { glyph: "", fg: "#f15233", padding: [1, 1] },
        ".gitattributes" => { glyph: "", fg: "#f15233", padding: [1, 1] },
        ".gitignore" => { glyph: "", fg: "#f15233", padding: [1, 1] },
        ".gitmodules" => { glyph: "", fg: "#f15233", padding: [1, 1] },
        ".ignore" => {glyph: "󰈉", padding: [1, 1] },
        ".mailmap" => { glyph: "", padding: [1, 1] },
        ".prettierignore" => {glyph: "", padding: [1, 1] },
        ".prettierrc" => {glyph: "", padding: [1, 1] },
        ".vimrc" => { glyph: "", fg: "#007f00", padding: [1, 1] },
        "CHANGELOG.md" => { glyph: "", fg: "#7bab43", padding: [1, 1] },
        "CODE_OF_CONDUCT.md" => { glyph: "", fg: "#f7769d", padding: [1, 1] },
        "Dockerfile" => {glyph: "󰡨", fg: "#0096e6", padding: [1, 1] },
        "Justfile" => { glyph : "󰖷", fg: "#888888", padding: [1, 1] },
        "LICENSE" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "LICENSE-AGPL" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "LICENSE-APACHE" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "LICENSE-GPL" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "LICENSE-MIT" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "LICENSE-WHATWG" => { glyph: "", fg: "#D0BF41", padding: [1, 1] },
        "Makefile" => {glyph: "", padding: [1, 1] },
        "README" => { glyph: "", fg: "#C8C8C8", padding: [1, 1] },
        "README.md" => { glyph: "", fg: "#C8C8C8", padding: [1, 1] },
        "SECURITY.md" => { glyph: "󰒃", fg: "#BEC4C9", padding: [1, 1] },
        "compose.yaml" => {glyph: "󰡨", fg: "#0096e6", padding: [1, 1] },
        "docker-compose.yaml" => {glyph: "󰡨", fg: "#0096e6", padding: [1, 1] },
        "justfile" => { glyph : "󰖷", fg: "#888888", padding: [1, 1] },
        "zig.build.zon" => { glyph : "", fg: "#F69A1B", padding: [1, 1] },
    },
    extension: icons! {
        "3gp" => { glyph:  "", fg: "#FD971F", padding: [1, 1] },
        "3mf" => { glyph: "󰆧", fg: "#888888", padding: [1, 1] },
        "7z" => { glyph: "", fg: "#ECA517", padding: [1, 1] },
        "Dockerfile" => { glyph: "󰡨", fg : "#458EE6", padding: [1, 1] },
        "R" => { glyph: "󰟔", fg: "#2266BA", padding: [1, 1] },
        "a" => { glyph: "", fg: "#C8C8C8", padding: [1, 1] },
        "aac" => { glyph: "", fg: "#00AFFF", padding: [1, 1] },
        "ada" => { glyph: "", fg: "#599EFF", padding: [1, 1] },
        "adb" => { glyph: "", fg: "#599EFF", padding: [1, 1] },
        "ads" => { glyph: "", fg: "#A074C4", padding: [1, 1] },
        "ai" => { glyph: "", fg: "#CBCB41", padding: [1, 1] },
        "aif" => { glyph: "", fg: "#00AFFF", padding: [1, 1] },
        "aiff" => { glyph: "", fg: "#00AFFF", padding: [1, 1] },
        "android" => { glyph: "", fg: "#34A853", padding: [1, 1] },
        "ape" => { glyph: "", fg: "#00AFFF", padding: [1, 1] },
        "apk" => { glyph: "", fg: "#34A853", padding: [1, 1] },
        "apl" => { glyph: "", fg: "#24A148", padding: [1, 1] },
        "app" => { glyph: "", fg: "#9F0500", padding: [1, 1] },
        "applescript" => { glyph: "", fg: "#6D8085", padding: [1, 1] },
        "asc" => { glyph: "󰦝", fg: "#576D7F", padding: [1, 1] },
        "asm" => { glyph: "", fg: "#0091BD", padding: [1, 1] },
        "ass" => { glyph: "󰨖", fg: "#FFB713", padding: [1, 1] },
        "astro" => { glyph: "", fg: "#E23F67", padding: [1, 1] },
        "avif" => { glyph: "", fg: "#A074C4", padding: [1, 1] },
        "awk" => { glyph: "", fg: "#4D5A5E", padding: [1, 1] },
        "azcli" => { glyph: "", fg: "#0078D4", padding: [1, 1] },
        "bak" => { glyph: "󰁯", fg: "#6D8086", padding: [1, 1] },
        "bash" => { glyph: "", fg: "#89E051", padding: [1, 1] },
        "bat" => { glyph: "", fg: "#C1F12E", padding: [1, 1] },
        "bazel" => { glyph: "", fg: "#89E051", padding: [1, 1] },
        "bib" => { glyph: "󱉟", fg: "#CBCB41", padding: [1, 1] },
        "bicep" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "bicepparam" => { glyph: "", fg: "#9F74B3", padding: [1, 1] },
        "bin" => { glyph: "", fg: "#9F0500", padding: [1, 1] },
        "blade.php" => { glyph: "", fg: "#F05340", padding: [1, 1] },
        "blend" => { glyph: "󰂫", fg: "#EA7600", padding: [1, 1] },
        "blp" => { glyph: "󰺾", fg: "#5796E2", padding: [1, 1] },
        "bmp" => { glyph: "", fg: "#A074C4", padding: [1, 1] },
        "bqn" => { glyph: "", fg: "#24A148", padding: [1, 1] },
        "brep" => { glyph: "󰻫", fg: "#839463", padding: [1, 1] },
        "bz" => { glyph: "", fg: "#ECA517", padding: [1, 1] },
        "bz2" => { glyph: "", fg: "#ECA517", padding: [1, 1] },
        "bz3" => { glyph: "", fg: "#ECA517", padding: [1, 1] },
        "bzl" => { glyph: "", fg: "#89E051", padding: [1, 1] },
        "c" => { glyph: "", fg: "#599EFF", padding: [1, 1] },
        "c++" => { glyph: "", fg: "#F34B7D", padding: [1, 1] },
        "cache" => { glyph: "", fg: "#DDDDDD", padding: [1, 1] },
        "cast" => { glyph: "", fg: "#FD971F", padding: [1, 1] },
        "cbl" => { glyph: "", fg: "#005CA5", padding: [1, 1] },
        "cc" => { glyph: "", fg: "#F34B7D", padding: [1, 1] },
        "ccm" => { glyph: "", fg: "#F34B7D", padding: [1, 1] },
        "cfc" => { glyph: "", fg: "#01A4BA", padding: [1, 1] },
        "cfg" => { glyph: "", fg: "#6D8086", padding: [1, 1] },
        "cfm" => { glyph: "", fg: "#01A4BA", padding: [1, 1] },
        "cjs" => { glyph: "", fg: "#CBCB41", padding: [1, 1] },
        "clj" => { glyph: "", fg: "#8DC149", padding: [1, 1] },
        "cljc" => { glyph: "", fg: "#8DC149", padding: [1, 1] },
        "cljd" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cljs" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cmake" => { glyph: "", fg: "#DCE3EB", padding: [1, 1] },
        "cob" => { glyph: "", fg: "#005CA5", padding: [1, 1] },
        "cobol" => { glyph: "", fg: "#005CA5", padding: [1, 1] },
        "coffee" => { glyph: "", fg: "#CBCB41", padding: [1, 1] },
        "conda" => { glyph: "", fg: "#43B02A", padding: [1, 1] },
        "conf" => { glyph: "", fg: "#6D8086", padding: [1, 1] },
        "config.ru" => { glyph: "", fg: "#701516", padding: [1, 1] },
        "cow" => { glyph: "󰆚", fg: "#965824", padding: [1, 1] },
        "cp" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cpp" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cppm" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cpy" => { glyph: "", fg: "#005CA5", padding: [1, 1] },
        "cr" => { glyph: "", fg: "#C8C8C8", padding: [1, 1] },
        "crdownload" => { glyph: "", fg: "#44CDA8", padding: [1, 1] },
        "cs" => { glyph: "󰌛", fg: "#596706", padding: [1, 1] },
        "csh" => { glyph: "", fg: "#4D5A5E", padding: [1, 1] },
        "cshtml" => { glyph: "󱦗", fg: "#512BD4", padding: [1, 1] },
        "cson" => { glyph: "", fg: "#CBCB41", padding: [1, 1] },
        "csproj" => { glyph: "󰪮", fg: "#512BD4", padding: [1, 1] },
        "css" => { glyph: "", fg: "#663399", padding: [1, 1] },
        "csv" => { glyph: "", fg: "#89E051", padding: [1, 1] },
        "cts" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cu" => { glyph: "", fg: "#89E051", padding: [1, 1] },
        "cue" => { glyph: "󰲹", fg: "#ED95AE", padding: [1, 1] },
        "cuh" => { glyph: "", fg: "#A074C4", padding: [1, 1] },
        "cxx" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "cxxm" => { glyph: "", fg: "#519ABA", padding: [1, 1] },
        "d" => { glyph: "", fg: "#B03931", padding: [1, 1] },
        "d.ts" => { glyph: "", fg: "#D59855", padding: [1, 1] },
        "dart" => { glyph: "", fg: "#03589C", padding: [1, 1] },
        "db" => { glyph: "", fg: "#DAD8D8", padding: [1, 1] },
        "dconf" => { glyph: "", fg: "#DDDDDD", padding: [1, 1] },
        "desktop" => { glyph: "", fg: "#563D7C", padding: [1, 1] },
        "diff" => { glyph: "", fg: "#41535B", padding: [1, 1] },
        "dll" => { glyph: "", fg: "#4D2C0B", padding: [1, 1] },
        "doc" => { glyph: "󰈬", fg: "#185ABD", padding: [1, 1] },
        "dockerignore" => { glyph : "󰡨", fg: "#458EE6", padding: [1, 1] },
        "docx" => { glyph : "󰈬", fg: "#185ABD", padding: [1, 1] },
        "dot" => { glyph : "󱁉", fg: "#30638E", padding: [1, 1] },
        "download" => { glyph : "", fg: "#44CDA8", padding: [1, 1] },
        "drl" => { glyph : "", fg: "#FFAFAF", padding: [1, 1] },
        "dropbox" => { glyph : "", fg: "#0061FE", padding: [1, 1] },
        "dump" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "dwg" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "dxf" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "ebook" => { glyph : "", fg: "#EAB16D", padding: [1, 1] },
        "ebuild" => { glyph : "", fg: "#4C416E", padding: [1, 1] },
        "edn" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "eex" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "ejs" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "el" => { glyph : "", fg: "#8172BE", padding: [1, 1] },
        "elc" => { glyph : "", fg: "#8172BE", padding: [1, 1] },
        "elf" => { glyph : "", fg: "#9F0500", padding: [1, 1] },
        "elm" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "eln" => { glyph : "", fg: "#8172BE", padding: [1, 1] },
        "env" => { glyph : "", fg: "#FAF743", padding: [1, 1] },
        "eot" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "epp" => { glyph : "", fg: "#FFA61A", padding: [1, 1] },
        "epub" => { glyph : "", fg: "#EAB16D", padding: [1, 1] },
        "erb" => { glyph : "", fg: "#701516", padding: [1, 1] },
        "erl" => { glyph : "", fg: "#B83998", padding: [1, 1] },
        "ex" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "exe" => { glyph : "", fg: "#9F0500", padding: [1, 1] },
        "exs" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "f#" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "f3d" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "f90" => { glyph : "󱈚", fg: "#734F96", padding: [1, 1] },
        "fbx" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "fcbak" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcmacro" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcmat" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcparam" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcscript" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcstd" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fcstd1" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fctb" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fctl" => { glyph : "", fg: "#CB333B", padding: [1, 1] },
        "fdmdownload" => { glyph : "", fg: "#44CDA8", padding: [1, 1] },
        "feature" => { glyph : "", fg: "#00A818", padding: [1, 1] },
        "fish" => { glyph : "", fg: "#4D5A5E", padding: [1, 1] },
        "flac" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "flc" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "flf" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "fnl" => { glyph : "", fg: "#E0D6BD", padding: [1, 1] },
        "fodg" => { glyph : "", fg: "#FFFB57", padding: [1, 1] },
        "fodp" => { glyph : "", fg: "#FE9C45", padding: [1, 1] },
        "fods" => { glyph : "", fg: "#78FC4E", padding: [1, 1] },
        "fodt" => { glyph : "", fg: "#2DCBFD", padding: [1, 1] },
        "frag" => { glyph : "", fg: "#5586A6", padding: [1, 1] },
        "fs" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "fsi" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "fsscript" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "fsx" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "gcode" => { glyph : "󰐫", fg: "#1471AD", padding: [1, 1] },
        "gd" => { glyph : "", fg: "#4b8cc0", padding: [1, 1] },
        "gemspec" => { glyph : "", fg: "#701516", padding: [1, 1] },
        "geom" => { glyph : "", fg: "#5586A6", padding: [1, 1] },
        "gif" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "git" => { glyph : "", fg: "#F14C28", padding: [1, 1] },
        "glb" => { glyph : "", fg: "#FFB13B", padding: [1, 1] },
        "gleam" => { glyph : "", fg: "#FFAFF3", padding: [1, 1] },
        "glsl" => { glyph : "", fg: "#5586A6", padding: [1, 1] },
        "gnumakefile" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "go" => { glyph : "", fg: "#00ADD8", padding: [1, 1] },
        "godot" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "gpr" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "gql" => { glyph : "", fg: "#E535AB", padding: [1, 1] },
        "gradle" => { glyph : "", fg: "#005F87", padding: [1, 1] },
        "graphql" => { glyph : "", fg: "#E535AB", padding: [1, 1] },
        "gresource" => { glyph : "", fg: "#DDDDDD", padding: [1, 1] },
        "gv" => { glyph : "󱁉", fg: "#30638E", padding: [1, 1] },
        "gz" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "h" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "haml" => { glyph : "", fg: "#DFDFDF", padding: [1, 1] },
        "hbs" => { glyph : "", fg: "#F0772B", padding: [1, 1] },
        "heex" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "hex" => { glyph : "", fg: "#2E63FF", padding: [1, 1] },
        "hh" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "hpp" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "hrl" => { glyph : "", fg: "#B83998", padding: [1, 1] },
        "hs" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "htm" => { glyph : "", fg: "#E34C26", padding: [1, 1] },
        "html" => { glyph : "", fg: "#E44D26", padding: [1, 1] },
        "http" => { glyph : "", fg: "#008EC7", padding: [1, 1] },
        "huff" => { glyph : "󰡘", fg: "#4242C7", padding: [1, 1] },
        "hurl" => { glyph : "", fg: "#FF0288", padding: [1, 1] },
        "hx" => { glyph : "", fg: "#EA8220", padding: [1, 1] },
        "hxx" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "ical" => { glyph : "", fg: "#2B2E83", padding: [1, 1] },
        "icalendar" => { glyph : "", fg: "#2B2E83", padding: [1, 1] },
        "ico" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "ics" => { glyph : "", fg: "#2B2E83", padding: [1, 1] },
        "ifb" => { glyph : "", fg: "#2B2E83", padding: [1, 1] },
        "ifc" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "ige" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "iges" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "igs" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "image" => { glyph : "", fg: "#D0BEC8", padding: [1, 1] },
        "img" => { glyph : "", fg: "#D0BEC8", padding: [1, 1] },
        "import" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "info" => { glyph : "", fg: "#BBBBBB", padding: [1, 1] },
        "ini" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "ino" => { glyph : "", fg: "#56B6C2", padding: [1, 1] },
        "ipynb" => { glyph : "", fg: "#F57D01", padding: [1, 1] },
        "iso" => { glyph : "", fg: "#D0BEC8", padding: [1, 1] },
        "ixx" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "jar" => { glyph : "", fg: "#ffaf67", padding: [1, 1] },
        "java" => { glyph : "", fg: "#CC3E44", padding: [1, 1] },
        "jl" => { glyph : "", fg: "#A270BA", padding: [1, 1] },
        "jpeg" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "jpg" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "js" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "json" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "json5" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "jsonc" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "jsx" => { glyph : "", fg: "#20C2E3", padding: [1, 1] },
        "Justfile" => { glyph : "󰖷", fg: "#888888", padding: [1, 1] },
        "justfile" => { glyph : "󰖷", fg: "#888888", padding: [1, 1] },
        "jwmrc" => { glyph : "", fg: "#0078CD", padding: [1, 1] },
        "jxl" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "kbx" => { glyph : "󰯄", fg: "#737672", padding: [1, 1] },
        "kdb" => { glyph : "", fg: "#529B34", padding: [1, 1] },
        "kdbx" => { glyph : "", fg: "#529B34", padding: [1, 1] },
        "kdenlive" => { glyph : "", fg: "#83B8F2", padding: [1, 1] },
        "kdenlivetitle" => { glyph : "", fg: "#83B8F2", padding: [1, 1] },
        "kicad_dru" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_mod" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_pcb" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_prl" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_pro" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_sch" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_sym" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "kicad_wks" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "ko" => { glyph : "", fg: "#DCDDD6", padding: [1, 1] },
        "kpp" => { glyph : "", fg: "#F245FB", padding: [1, 1] },
        "kra" => { glyph : "", fg: "#F245FB", padding: [1, 1] },
        "krz" => { glyph : "", fg: "#F245FB", padding: [1, 1] },
        "ksh" => { glyph : "", fg: "#4D5A5E", padding: [1, 1] },
        "kt" => { glyph : "", fg: "#7F52FF", padding: [1, 1] },
        "kts" => { glyph : "", fg: "#7F52FF", padding: [1, 1] },
        "lck" => { glyph : "", fg: "#BBBBBB", padding: [1, 1] },
        "leex" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "less" => { glyph : "", fg: "#563D7C", padding: [1, 1] },
        "lff" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "lhs" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "lib" => { glyph : "", fg: "#4D2C0B", padding: [1, 1] },
        "LICENSE" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "license" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "liquid" => { glyph : "", fg: "#95BF47", padding: [1, 1] },
        "lock" => { glyph : "", fg: "#BBBBBB", padding: [1, 1] },
        "log" => { glyph : "󰌱", fg: "#DDDDDD", padding: [1, 1] },
        "lrc" => { glyph : "󰨖", fg: "#FFB713", padding: [1, 1] },
        "lua" => { glyph : "", fg: "#51A0CF", padding: [1, 1] },
        "luac" => { glyph : "", fg: "#51A0CF", padding: [1, 1] },
        "luau" => { glyph : "", fg: "#00A2FF", padding: [1, 1] },
        "m" => { glyph : "", fg: "#599EFF", padding: [1, 1] },
        "m3u" => { glyph : "󰲹", fg: "#ED95AE", padding: [1, 1] },
        "m3u8" => { glyph : "󰲹", fg: "#ED95AE", padding: [1, 1] },
        "m4a" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "m4v" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "magnet" => { glyph : "", fg: "#A51B16", padding: [1, 1] },
        "makefile" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "markdown" => { glyph : "", fg: "#DDDDDD", padding: [1, 1] },
        "material" => { glyph : "", fg: "#B83998", padding: [1, 1] },
        "md" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "md5" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "mdx" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "mint" => { glyph : "󰌪", fg: "#87C095", padding: [1, 1] },
        "mjs" => { glyph : "", fg: "#F1E05A", padding: [1, 1] },
        "mk" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "mkv" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "ml" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "mli" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "mm" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "mo" => { glyph : "", fg: "#9772FB", padding: [1, 1] },
        "mobi" => { glyph : "", fg: "#EAB16D", padding: [1, 1] },
        "mojo" => { glyph : "", fg: "#FF4C1F", padding: [1, 1] },
        "mov" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "mp3" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "mp4" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "mpp" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "msf" => { glyph : "", fg: "#137BE1", padding: [1, 1] },
        "mts" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "mustache" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "nfo" => { glyph : "", fg: "#FFFFCD", padding: [1, 1] },
        "nim" => { glyph : "", fg: "#F3D400", padding: [1, 1] },
        "nix" => { glyph : "", fg: "#7EBAE4", padding: [1, 1] },
        "norg" => { glyph : "", fg: "#4878BE", padding: [1, 1] },
        "nswag" => { glyph : "", fg: "#85EA2D", padding: [1, 1] },
        "nu" => { glyph : "", fg: "#3AA675", padding: [1, 1] },
        "o" => { glyph : "", fg: "#9F0500", padding: [1, 1] },
        "obj" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "odf" => { glyph : "", fg: "#FF5A96", padding: [1, 1] },
        "odg" => { glyph : "", fg: "#FFFB57", padding: [1, 1] },
        "odin" => { glyph : "󰟢", fg: "#3882D2", padding: [1, 1] },
        "odp" => { glyph : "", fg: "#FE9C45", padding: [1, 1] },
        "ods" => { glyph : "", fg: "#78FC4E", padding: [1, 1] },
        "odt" => { glyph : "", fg: "#2DCBFD", padding: [1, 1] },
        "oga" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "ogg" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "ogv" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "ogx" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "opus" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "org" => { glyph : "", fg: "#77AA99", padding: [1, 1] },
        "otf" => { glyph : "", fg: "#C8C8C8", padding: [1, 1] },
        "out" => { glyph : "", fg: "#9F0500", padding: [1, 1] },
        "part" => { glyph : "", fg: "#44CDA8", padding: [1, 1] },
        "patch" => { glyph : "", fg: "#41535B", padding: [1, 1] },
        "pck" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "pcm" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "pdf" => { glyph : "", fg: "#B30B00", padding: [1, 1] },
        "php" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "pl" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "pls" => { glyph : "󰲹", fg: "#ED95AE", padding: [1, 1] },
        "ply" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "pm" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "png" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "po" => { glyph : "", fg: "#2596BE", padding: [1, 1] },
        "pot" => { glyph : "", fg: "#2596BE", padding: [1, 1] },
        "pp" => { glyph : "", fg: "#FFA61A", padding: [1, 1] },
        "ppt" => { glyph : "󰈧", fg: "#CB4A32", padding: [1, 1] },
        "pptx" => { glyph : "󰈧", fg: "#CB4A32", padding: [1, 1] },
        "prisma" => { glyph : "", fg: "#f7fafc", padding: [1, 1] },
        "pro" => { glyph : "", fg: "#E4B854", padding: [1, 1] },
        "ps1" => { glyph : "󰨊", fg: "#4273CA", padding: [1, 1] },
        "psb" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "psd" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "psd1" => { glyph : "󰨊", fg: "#6975C4", padding: [1, 1] },
        "psm1" => { glyph : "󰨊", fg: "#6975C4", padding: [1, 1] },
        "pub" => { glyph : "󰷖", fg: "#E3C58E", padding: [1, 1] },
        "pxd" => { glyph : "", fg: "#5AA7E4", padding: [1, 1] },
        "pxi" => { glyph : "", fg: "#5AA7E4", padding: [1, 1] },
        "py" => { glyph : "", fg: "#FFBC03", padding: [1, 1] },
        "pyc" => { glyph : "", fg: "#FFE291", padding: [1, 1] },
        "pyd" => { glyph : "", fg: "#FFE291", padding: [1, 1] },
        "pyi" => { glyph : "", fg: "#FFBC03", padding: [1, 1] },
        "pyo" => { glyph : "", fg: "#FFE291", padding: [1, 1] },
        "pyw" => { glyph : "", fg: "#5AA7E4", padding: [1, 1] },
        "pyx" => { glyph : "", fg: "#5AA7E4", padding: [1, 1] },
        "qm" => { glyph : "", fg: "#2596BE", padding: [1, 1] },
        "qml" => { glyph : "", fg: "#40CD52", padding: [1, 1] },
        "qrc" => { glyph : "", fg: "#40CD52", padding: [1, 1] },
        "qss" => { glyph : "", fg: "#40CD52", padding: [1, 1] },
        "query" => { glyph : "", fg: "#90A850", padding: [1, 1] },
        "r" => { glyph : "󰟔", fg: "#2266BA", padding: [1, 1] },
        "rake" => { glyph : "", fg: "#701516", padding: [1, 1] },
        "rar" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "rasi" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "razor" => { glyph : "󱦘", fg: "#512BD4", padding: [1, 1] },
        "rb" => { glyph : "", fg: "#701516", padding: [1, 1] },
        "res" => { glyph : "", fg: "#CC3E44", padding: [1, 1] },
        "resi" => { glyph : "", fg: "#F55385", padding: [1, 1] },
        "rkt" => { glyph : "󰘧", fg: "#9F1D20", padding: [1, 1] },
        "rlib" => { glyph : "", fg: "#DEA584", padding: [1, 1] },
        "rmd" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "rproj" => { glyph : "󰗆", fg: "#358A5B", padding: [1, 1] },
        "rs" => { glyph : "", fg: "#DEA584", padding: [1, 1] },
        "rss" => { glyph : "", fg: "#FB9D3B", padding: [1, 1] },
        "s" => { glyph : "", fg: "#0071C5", padding: [1, 1] },
        "sass" => { glyph : "", fg: "#F55385", padding: [1, 1] },
        "sbt" => { glyph : "", fg: "#CC3E44", padding: [1, 1] },
        "sc" => { glyph : "", fg: "#CC3E44", padding: [1, 1] },
        "scad" => { glyph : "", fg: "#F9D72C", padding: [1, 1] },
        "scala" => { glyph : "", fg: "#CC3E44", padding: [1, 1] },
        "scm" => { glyph : "󰘧", fg: "#9F1D20", padding: [1, 1] },
        "scss" => { glyph : "", fg: "#F55385", padding: [1, 1] },
        "sh" => { glyph : "", fg: "#4D5A5E", padding: [1, 1] },
        "sha1" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "sha224" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "sha256" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "sha384" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "sha512" => { glyph : "󰕥", fg: "#8C86AF", padding: [1, 1] },
        "sig" => { glyph : "󰘧", fg: "#E37933", padding: [1, 1] },
        "signature" => { glyph : "󰘧", fg: "#E37933", padding: [1, 1] },
        "skp" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "sldasm" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "sldprt" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "slim" => { glyph : "", fg: "#E34C26", padding: [1, 1] },
        "sln" => { glyph : "", fg: "#854CC7", padding: [1, 1] },
        "slnx" => { glyph : "", fg: "#854CC7", padding: [1, 1] },
        "slvs" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "sml" => { glyph : "󰘧", fg: "#E37933", padding: [1, 1] },
        "so" => { glyph : "", fg: "#DCDDD6", padding: [1, 1] },
        "sol" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "spec.js" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "spec.jsx" => { glyph : "", fg: "#20C2E3", padding: [1, 1] },
        "spec.ts" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "spec.tsx" => { glyph : "", fg: "#1354BF", padding: [1, 1] },
        "spx" => { glyph : "", fg: "#0075AA", padding: [1, 1] },
        "sql" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "sqlite" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "sqlite3" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "srt" => { glyph : "󰨖", fg: "#FFB713", padding: [1, 1] },
        "ssa" => { glyph : "󰨖", fg: "#FFB713", padding: [1, 1] },
        "ste" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "step" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "stl" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "stories.js" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.jsx" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.mjs" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.svelte" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.ts" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.tsx" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stories.vue" => { glyph : "", fg: "#FF4785", padding: [1, 1] },
        "stp" => { glyph : "󰻫", fg: "#839463", padding: [1, 1] },
        "strings" => { glyph : "", fg: "#2596BE", padding: [1, 1] },
        "styl" => { glyph : "", fg: "#8DC149", padding: [1, 1] },
        "sub" => { glyph : "󰨖", fg: "#FFB713", padding: [1, 1] },
        "sublime" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "suo" => { glyph : "", fg: "#854CC7", padding: [1, 1] },
        "sv" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "svelte" => { glyph : "", fg: "#FF3E00", padding: [1, 1] },
        "svg" => { glyph : "󰜡", fg: "#FFB13B", padding: [1, 1] },
        "svgz" => { glyph : "󰜡", fg: "#FFB13B", padding: [1, 1] },
        "svh" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "svx" => { glyph : "", fg: "#FF0042", padding: [1, 1] },
        "swift" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "t" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "tbc" => { glyph : "󰛓", fg: "#1E5CB3", padding: [1, 1] },
        "tcl" => { glyph : "󰛓", fg: "#1E5CB3", padding: [1, 1] },
        "templ" => { glyph : "", fg: "#DBBD30", padding: [1, 1] },
        "terminal" => { glyph : "", fg: "#31B53E", padding: [1, 1] },
        "test.js" => { glyph : "", fg: "#CBCB41", padding: [1, 1] },
        "test.jsx" => { glyph : "", fg: "#20C2E3", padding: [1, 1] },
        "test.ts" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "test.tsx" => { glyph : "", fg: "#1354BF", padding: [1, 1] },
        "tex" => { glyph : "", fg: "#3D6117", padding: [1, 1] },
        "tf" => { glyph : "", fg: "#5F43E9", padding: [1, 1] },
        "tfvars" => { glyph : "", fg: "#5F43E9", padding: [1, 1] },
        "tgz" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "tmpl" => { glyph : "", fg: "#DBBD30", padding: [1, 1] },
        "tmux" => { glyph : "", fg: "#14BA19", padding: [1, 1] },
        "toml" => { glyph : "", fg: "#9C4221", padding: [1, 1] },
        "torrent" => { glyph : "", fg: "#44CDA8", padding: [1, 1] },
        "tres" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "ts" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "tscn" => { glyph : "", fg: "#6D8086", padding: [1, 1] },
        "tsconfig" => { glyph : "", fg: "#FF8700", padding: [1, 1] },
        "tsx" => { glyph : "", fg: "#1354BF", padding: [1, 1] },
        "ttf" => { glyph : "", fg: "#ECECEC", padding: [1, 1] },
        "twig" => { glyph : "", fg: "#8DC149", padding: [1, 1] },
        "txt" => { glyph : "󰈙", fg: "#89E051", padding: [1, 1] },
        "txz" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "typ" => { glyph : "", fg: "#0DBCC0", padding: [1, 1] },
        "typoscript" => { glyph : "", fg: "#FF8700", padding: [1, 1] },
        "ui" => { glyph : "", fg: "#015BF0", padding: [1, 1] },
        "v" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "vala" => { glyph : "", fg: "#7B3DB9", padding: [1, 1] },
        "vert" => { glyph : "", fg: "#5586A6", padding: [1, 1] },
        "vh" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "vhd" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "vhdl" => { glyph : "󰍛", fg: "#019833", padding: [1, 1] },
        "vi" => { glyph : "", fg: "#FEC60A", padding: [1, 1] },
        "vim" => { glyph : "", fg: "#019833", padding: [1, 1] },
        "vsh" => { glyph : "", fg: "#5D87BF", padding: [1, 1] },
        "vsix" => { glyph : "", fg: "#854CC7", padding: [1, 1] },
        "vue" => { glyph : "", fg: "#8DC149", padding: [1, 1] },
        "wasm" => { glyph : "", fg: "#5C4CDB", padding: [1, 1] },
        "wav" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "webm" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "webmanifest" => { glyph : "", fg: "#F1E05A", padding: [1, 1] },
        "webp" => { glyph : "", fg: "#A074C4", padding: [1, 1] },
        "webpack" => { glyph : "󰜫", fg: "#519ABA", padding: [1, 1] },
        "wma" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "wmv" => { glyph : "", fg: "#FD971F", padding: [1, 1] },
        "woff" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "woff2" => { glyph : "", fg: "#DAD8D8", padding: [1, 1] },
        "wrl" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "wrz" => { glyph : "󰆧", fg: "#888888", padding: [1, 1] },
        "wv" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "wvc" => { glyph : "", fg: "#00AFFF", padding: [1, 1] },
        "x" => { glyph : "", fg: "#599EFF", padding: [1, 1] },
        "xaml" => { glyph : "󰙳", fg: "#512BD4", padding: [1, 1] },
        "xcf" => { glyph : "", fg: "#635B46", padding: [1, 1] },
        "xcplayground" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "xcstrings" => { glyph : "", fg: "#2596BE", padding: [1, 1] },
        "xls" => { glyph : "󰈛", fg: "#207245", padding: [1, 1] },
        "xlsx" => { glyph : "󰈛", fg: "#207245", padding: [1, 1] },
        "xm" => { glyph : "", fg: "#519ABA", padding: [1, 1] },
        "xml" => { glyph : "󰗀", fg: "#E37933", padding: [1, 1] },
        "xpi" => { glyph : "", fg: "#FF1B01", padding: [1, 1] },
        "xslt" => { glyph : "󰗀", fg: "#33A9DC", padding: [1, 1] },
        "xul" => { glyph : "", fg: "#E37933", padding: [1, 1] },
        "xz" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "yaml" => { glyph : "", fg: "#D70000", padding: [1, 1] },
        "yml" => { glyph : "", fg: "#D70000", padding: [1, 1] },
        "zig" => { glyph : "", fg: "#F69A1B", padding: [1, 1] },
        "zip" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "zsh" => { glyph : "", fg: "#89E051", padding: [1, 1] },
        "zst" => { glyph : "", fg: "#ECA517", padding: [1, 1] },
        "🔥" => { glyph : "", fg: "#FF4C1F", padding: [1, 1] },
    },
});

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct File {
    #[serde(default)]
    filename: HashMap<SmartString, Icon>,
    #[serde(default)]
    extension: HashMap<SmartString, Icon>,
}

impl File {
    #[inline]
    #[must_use]
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<Icon> {
        self.get_by_filename(&path)
            .or_else(|| self.get_by_extension(&path))
    }

    #[inline]
    #[must_use]
    pub fn get_by_filename<P: AsRef<Path>>(&self, path: P) -> Option<Icon> {
        let path = path.as_ref();
        let name = path.file_name()?.to_string_lossy();

        // All icons in `filename` are user overrides.
        match self.filename.get(name.as_ref()) {
            Some(icon) => {
                // If there is no existing `base` icon, then just
                // return whatever the user provided.
                FILE.filename
                    .get(name.as_ref())
                    .map_or(Some(*icon), |base| {
                        Some(base.patch_from_user_override(*icon))
                    })
            }
            None => FILE.filename.get(name.as_ref()).copied(),
        }
    }

    #[inline]
    #[must_use]
    pub fn get_by_extension<P: AsRef<Path>>(&self, path: P) -> Option<Icon> {
        let path = path.as_ref();
        let ext = path.extension()?.to_string_lossy();

        // All icons in `extension` are user overrides.
        match self.extension.get(ext.as_ref()) {
            Some(icon) => {
                // If there is no existing `base` icon, then just
                // return whatever the user provided.
                FILE.extension.get(ext.as_ref()).map_or_else(
                    || {
                        FILE.extension
                            .get(ext.as_ref())
                            .copied()
                            .map(|base| base.patch_from_user_override(*icon))
                    },
                    |base| Some(base.patch_from_user_override(*icon)),
                )
            }
            None => FILE.extension.get(ext.as_ref()).copied(),
        }
    }

    #[inline]
    #[must_use]
    pub fn get_or_default<P: AsRef<Path>>(&self, path: P) -> Icon {
        self.get(path)
            .unwrap_or_else(|| icon!(glyph: "", padding: [1, 1]))
    }

    #[inline]
    #[must_use]
    pub fn get_with_style_or_default<P: AsRef<Path>>(&self, path: P, theme: &Theme) -> Icon {
        let path = path.as_ref();

        let name = path.file_name().map(|name| name.display());
        let ext = path.extension().map(|ext| ext.display());

        let mut icon = self.get_or_default(path);

        let style = if icon.is_user_overridden() {
            icon.style()
        } else {
            // WARN: Order matters, with `name` needing to
            // be first to match exact filenames before extensions.
            [name.as_ref(), ext.as_ref()]
                .into_iter()
                .flatten()
                .find_map(|scope| self.get_style_from_theme(theme, scope))
                .map_or_else(|| icon.style(), |style| icon.style().patch(style))
        };

        icon.set_style(style);

        icon
    }

    #[inline]
    #[must_use]
    pub fn get_with_active_style_or_default<P: AsRef<Path>>(
        &self,
        path: P,
        theme: &Theme,
        base: Style,
    ) -> Icon {
        let path = path.as_ref();

        let name = path.file_name().map(|name| name.display());
        let ext = path.extension().map(|ext| ext.display());

        let mut icon = self.get_or_default(path);

        let style = if icon.is_user_overridden() {
            base.patch(icon.style())
        } else {
            // WARN: Order matters, with `name` needing to be
            // first to match exact filenames before extensions.
            [name, ext]
                .into_iter()
                .flatten()
                .find_map(|scope| self.get_active_style_from_theme(theme, scope))
                .map_or_else(
                    || base.patch(icon.style()),
                    |style| base.patch(icon.style()).patch(style),
                )
        };

        icon.set_style(style);

        icon
    }

    #[inline]
    #[must_use]
    fn get_style_from_theme<D: Display>(&self, theme: &Theme, scope: D) -> Option<Style> {
        theme.try_get_exact(&format!("icons.file.{scope}"))
    }

    #[inline]
    #[must_use]
    fn get_active_style_from_theme<D: Display>(&self, theme: &Theme, scope: D) -> Option<Style> {
        if let Some(style) = theme.try_get_exact(&format!("icons.file.{scope}.active")) {
            return Some(style);
        }
        self.get_style_from_theme(theme, scope)
    }
}

static DIRECTORY: LazyLock<Directory> = LazyLock::new(|| Directory {
    icons: icons! {
        // Special names.
        "default" => { glyph: "󰉋", padding: [1, 1] },
        "open" => { glyph: "󰝰", padding: [1, 1] },

        ".config" => { glyph: "", padding: [1, 1] },
        ".exercism" => { glyph: "", padding: [1, 1] },
        ".git" => { glyph: "", padding: [1, 1] },
        ".jj" => { glyph: "", padding: [1, 1] },
        ".github" => { glyph: "", padding: [1, 1] },
        ".npm" => { glyph: "", padding: [1, 1] },
        ".opam" => { glyph: "", padding: [1, 1] },
        ".ssh" => { glyph: "󰢬", padding: [1, 1] },
        "Contacts" => { glyph: "󰉌", padding: [1, 1] },
        "Documents" => { glyph: "󰲂", padding: [1, 1] },
        "Downloads" => { glyph: "󰉍", padding: [1, 1] },
        "Favorites" => { glyph: "󰚝", padding: [1, 1] },
        "Mail" => { glyph: "󰇰", padding: [1, 1] },
        "Movies" => { glyph: "󰿎", padding: [1, 1] },
        "Music" => { glyph: "󱍙", padding: [1, 1] },
        "Pictures" => { glyph: "󰉏", padding: [1, 1] },
        "Videos" => { glyph: "", padding: [1, 1] },
        "build" => { glyph: "󱧼", padding: [1, 1] },
        "cabal" => { glyph: "", fg: "#A074C4", padding: [1, 1] },
        "config" => { glyph: "", padding: [1, 1] },
        "home" => { glyph: "󱂵", padding: [1, 1] },
        "include" => { glyph: "", padding: [1, 1] },
        "node_modules" => { glyph: "", padding: [1, 1] },
        "npm_cache" => { glyph: "", padding: [1, 1] },
        "src" => { glyph: "󰣞", padding: [1, 1] },
        "ssh" => { glyph: "󰢬", padding: [1, 1] },
    },
});

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct Directory {
    #[serde(flatten)]
    icons: HashMap<SmartString, Icon>,
}

impl Directory {
    #[inline]
    #[must_use]
    pub fn get(&self, name: &str, is_open: bool) -> Icon {
        if is_open {
            return match self.icons.get("open") {
                Some(icon) => DIRECTORY
                    .icons
                    .get("open")
                    .map_or_else(
                        || DIRECTORY.icons.get("open").copied(),
                        |base| Some(base.patch_from_user_override(*icon)),
                    )
                    .expect("this should default to `open` which is always provided"),
                None => DIRECTORY
                    .icons
                    .get("open")
                    .copied()
                    .expect("this should default to `open` which is always provided"),
            };
        }

        // All icons in `self.icons` are user overrides.
        match self.icons.get(name) {
            Some(icon) => {
                // If there is no existing `base` icon, then just
                // return whatever the user provided.
                DIRECTORY
                    .icons
                    .get(name)
                    .map_or_else(
                        || {
                            DIRECTORY
                                .icons
                                .get(name)
                                .or_else(|| DIRECTORY.icons.get(name))
                                .or_else(|| DIRECTORY.icons.get("default"))
                                .copied()
                                .map(|base| base.patch_from_user_override(*icon))
                        },
                        |base| Some(base.patch_from_user_override(*icon)),
                    )
                    .expect("this should default to `default` which is always provided")
            }
            None => DIRECTORY
                .icons
                .get(name)
                .or_else(|| DIRECTORY.icons.get("default"))
                .copied()
                .expect("this should default to `default` which is always provided"),
        }
    }

    #[inline]
    #[must_use]
    pub fn get_with_style_or_default(
        &self,
        name: &str,
        is_open: bool,
        theme: &Theme,
        base: Style,
    ) -> Icon {
        let mut icon = self.get(name, is_open);

        let style = if icon.is_user_overridden() {
            base.patch(icon.style())
        } else {
            self.get_style_from_theme(theme, name).map_or_else(
                || base.patch(icon.style()),
                |style| base.patch(icon.style()).patch(style),
            )
        };

        icon.set_style(style);

        icon
    }

    #[inline]
    #[must_use]
    pub fn get_style_from_theme<D: Display>(&self, theme: &Theme, scope: D) -> Option<Style> {
        theme.try_get_exact(&format!("icons.directory.{scope}"))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Dap {
    verified: Option<Icon>,
    unverified: Option<Icon>,
    play: Option<Icon>,
}

impl Dap {
    #[inline]
    #[must_use]
    pub fn verified(&self) -> Icon {
        self.verified.unwrap_or_else(|| icon!("●"))
    }

    #[inline]
    #[must_use]
    pub fn unverified(&self) -> Icon {
        self.unverified.unwrap_or_else(|| icon!("◯"))
    }

    #[inline]
    #[must_use]
    pub fn play(&self) -> Icon {
        self.play.unwrap_or_else(|| icon!("▶"))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Ui {
    workspace: Option<Icon>,
    gutter: Gutter,
    #[serde(rename = "virtual")]
    r#virtual: Virtual,
    statusline: Statusline,
    indicator: Indicator,
}

impl Ui {
    /// Returns a workspace diagnostic icon.
    ///
    /// If no icon is set in the config, it will return `W` by default.
    #[inline]
    #[must_use]
    pub fn workspace(&self) -> Icon {
        self.workspace.unwrap_or_else(|| icon!("W"))
    }

    #[inline]
    #[must_use]
    pub const fn gutter(&self) -> &Gutter {
        &self.gutter
    }

    #[inline]
    #[must_use]
    pub const fn r#virtual(&self) -> &Virtual {
        &self.r#virtual
    }

    #[inline]
    #[must_use]
    pub const fn statusline(&self) -> &Statusline {
        &self.statusline
    }

    #[inline]
    #[must_use]
    pub const fn indicator(&self) -> &Indicator {
        &self.indicator
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Gutter {
    added: Option<Icon>,
    modified: Option<Icon>,
    removed: Option<Icon>,
}

impl Gutter {
    #[inline]
    #[must_use]
    pub fn added(&self) -> Icon {
        self.added.unwrap_or_else(|| icon!("▍"))
    }

    #[inline]
    #[must_use]
    pub fn modified(&self) -> Icon {
        self.modified.unwrap_or_else(|| icon!("▍"))
    }

    #[inline]
    #[must_use]
    pub fn removed(&self) -> Icon {
        self.removed.unwrap_or_else(|| icon!("▔"))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Virtual {
    // Whitespace
    space: Option<Icon>,
    nbsp: Option<Icon>,
    nnbsp: Option<Icon>,
    tab: Option<Icon>,
    newline: Option<Icon>,
    tabpad: Option<Icon>,

    // Soft-wrap
    wrap: Option<Icon>,

    // Indentation guide
    indentation: Option<Icon>,

    // Ruler
    ruler: Option<Icon>,
}

impl Virtual {
    #[inline]
    #[must_use]
    pub fn space(&self) -> Icon {
        // Default: U+00B7
        self.space.unwrap_or_else(|| icon!("·"))
    }

    #[inline]
    #[must_use]
    pub fn nbsp(&self) -> Icon {
        // Default: U+237D
        self.nbsp.unwrap_or_else(|| icon!("⍽"))
    }

    #[inline]
    #[must_use]
    pub fn nnbsp(&self) -> Icon {
        // Default: U+2423
        self.nnbsp.unwrap_or_else(|| icon!("␣"))
    }

    #[inline]
    #[must_use]
    pub fn tab(&self) -> Icon {
        // Default: U+2192
        self.tab.unwrap_or_else(|| icon!("→"))
    }

    #[inline]
    #[must_use]
    pub fn newline(&self) -> Icon {
        // Default: U+23CE
        self.newline.unwrap_or_else(|| icon!("⏎"))
    }

    #[inline]
    #[must_use]
    pub fn tabpad(&self) -> Icon {
        // Default: U+23CE
        self.tabpad.unwrap_or_else(|| icon!(" "))
    }

    #[inline]
    #[must_use]
    pub fn wrap(&self) -> Icon {
        // Default: U+21AA
        self.wrap.unwrap_or_else(|| icon!("↪"))
    }

    #[inline]
    #[must_use]
    pub fn indentation(&self) -> Icon {
        // Default: U+254E
        self.indentation.unwrap_or_else(|| icon!("╎"))
    }

    #[inline]
    #[must_use]
    pub fn ruler(&self) -> Icon {
        // TODO: Default: ┊
        self.ruler.unwrap_or_else(|| icon!(" "))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Statusline {
    separator: Option<Icon>,
}

impl Statusline {
    #[inline]
    #[must_use]
    pub fn separator(&self) -> Icon {
        self.separator.unwrap_or_else(|| icon!("│"))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Indicator {
    readonly: Option<Icon>,
    modified: Option<Icon>,
    insecure: Option<Icon>,
}

impl Indicator {
    #[inline]
    #[must_use]
    pub fn readonly(&self) -> Icon {
        self.readonly.unwrap_or_else(|| icon!("[readonly]"))
    }

    #[inline]
    #[must_use]
    pub fn modified(&self) -> Icon {
        // TODO: ●?
        self.modified.unwrap_or_else(|| icon!("[+]"))
    }

    #[inline]
    #[must_use]
    pub fn insecure(&self) -> Icon {
        self.insecure.unwrap_or_else(|| icon!("[⚠]"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn kind_should_always_contain_color_icon() {
        let icons = Icons::default();
        let icon = icons.kind().color();
        assert_eq!(icon.glyph(), "■");
    }
}
