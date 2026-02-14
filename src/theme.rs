//! Theme configuration for terminal colors.

use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Theme {
    #[serde(default)]
    pub(crate) reset: Color,
    #[serde(default)]
    pub(crate) fg_reset: Color,
    #[serde(default)]
    pub(crate) dir: Color,
    #[serde(default)]
    pub(crate) file: Color,
    #[serde(default)]
    pub(crate) error: Color,
    #[serde(default)]
    pub(crate) highlight_start: Color,
    #[serde(default)]
    pub(crate) highlight_end: Color,
}

macro_rules! define_preset_colors {
    (
        $(
            $Name:ident => {
                ansi: $ansi:expr,
                aliases: [$($alias:expr),+ $(,)?]
            }
        ),+ $(,)?
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub(crate) enum PresetColor {
            $($Name),+
        }

        impl PresetColor {
            pub(crate) fn to_ansi(self) -> &'static str {
                match self {
                    $(PresetColor::$Name => $ansi),+
                }
            }

            pub(crate) fn parse(name: &str) -> anyhow::Result<Self> {
                let name = name.to_ascii_lowercase();
                match name.as_str() {
                    $(
                        $($alias)|+ => Ok(PresetColor::$Name),
                    )+
                    _ => anyhow::bail!("Unknown preset color: {}", name),
                }
            }
        }
    };
}

define_preset_colors! {
    Reset => {
        ansi: "\x1b[0m",
        aliases: ["reset", "default"]
    },
    FgReset => {
        ansi: "\x1b[39m",
        aliases: ["fg_reset", "fgreset", "default_fg"]
    },
    Invert => {
        ansi: "\x1b[7m",
        aliases: ["invert", "reverse"]
    },
    Red => {
        ansi: "\x1b[31m",
        aliases: ["red"]
    },
    Yellow => {
        ansi: "\x1b[33m",
        aliases: ["yellow", "yel"]
    },
    Blue => {
        ansi: "\x1b[34m",
        aliases: ["blue"]
    },
    Green => {
        ansi: "\x1b[32m",
        aliases: ["green"]
    },
    Cyan => {
        ansi: "\x1b[36m",
        aliases: ["cyan"]
    },
    Magenta => {
        ansi: "\x1b[35m",
        aliases: ["magenta", "purple"]
    },
    White => {
        ansi: "\x1b[37m",
        aliases: ["white"]
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Color {
    Preset { name: String },
    RGB { r: u8, g: u8, b: u8 },
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            reset: Color::Preset {
                name: "reset".to_string(),
            },
            fg_reset: Color::Preset {
                name: "fg_reset".to_string(),
            },
            dir: Color::Preset {
                name: "blue".to_string(),
            },
            file: Color::Preset {
                name: "white".to_string(),
            },
            error: Color::Preset {
                name: "red".to_string(),
            },
            highlight_start: Color::Preset {
                name: "invert".to_string(),
            },
            highlight_end: Color::Preset {
                name: "reset".to_string(),
            },
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Preset {
            name: "reset".to_string(),
        }
    }
}

impl Theme {
    pub(crate) fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&text)?;
        theme.validate()?;
        Ok(theme)
    }

    fn validate(&self) -> anyhow::Result<()> {
        self.reset.validate()?;
        self.fg_reset.validate()?;
        self.dir.validate()?;
        self.file.validate()?;
        self.error.validate()?;
        self.highlight_start.validate()?;
        self.highlight_end.validate()?;
        Ok(())
    }
}

impl Color {
    pub(crate) fn to_ansi(&self) -> anyhow::Result<String> {
        match self {
            Color::Preset { name } => {
                let preset = PresetColor::parse(name)?;
                Ok(preset.to_ansi().to_string())
            }
            Color::RGB { r, g, b } => Ok(format!("\x1b[38;2;{};{};{}m", r, g, b)),
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        if let Color::Preset { name } = self {
            PresetColor::parse(name)?;
        }
        Ok(())
    }
}

pub(crate) fn load_theme_from_env_or_default() -> Theme {
    if let Ok(path) = env::var("FSWHY_THEME") {
        if let Ok(theme) = Theme::load_from_file(Path::new(&path)) {
            return theme;
        }
    }

    if let Ok(theme) = Theme::load_from_file(Path::new("theme.toml")) {
        return theme;
    }

    Theme::default()
}
