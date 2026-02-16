//! 主题配置模块

use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// 主题配置
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
    #[serde(default)]
    pub(crate) dir_gradient_start: Color,
    #[serde(default)]
    pub(crate) dir_gradient_end: Color,
    #[serde(default)]
    pub(crate) file_gradient_start: Color,
    #[serde(default)]
    pub(crate) file_gradient_end: Color,
}

/// 预设颜色宏
macro_rules! define_preset_colors {
    (
        $(
            $Name:ident => {
                ansi: $ansi:expr,
                rgb: ($r:expr, $g:expr, $b:expr),
                aliases: [$($alias:expr),+ $(,)?]
            }
        ),+ $(,)?
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub(crate) enum PresetColor {
            $($Name),+
        }

        impl PresetColor {
            /// 转ANSI转义码
            pub(crate) fn to_ansi(self) -> &'static str {
                match self {
                    $(PresetColor::$Name => $ansi),+
                }
            }

            /// 转RGB值
            pub(crate) fn to_rgb(self) -> (u8, u8, u8) {
                match self {
                    $(PresetColor::$Name => ($r, $g, $b)),+
                }
            }

            /// 解析颜色名
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
        rgb: (255, 255, 255),
        aliases: ["reset", "default"]
    },
    FgReset => {
        ansi: "\x1b[39m",
        rgb: (255, 255, 255),
        aliases: ["fg_reset", "fgreset", "default_fg"]
    },
    Invert => {
        ansi: "\x1b[7m",
        rgb: (255, 255, 255),
        aliases: ["invert", "reverse"]
    },
    Red => {
        ansi: "\x1b[31m",
        rgb: (205, 49, 49),
        aliases: ["red"]
    },
    Yellow => {
        ansi: "\x1b[33m",
        rgb: (229, 229, 16),
        aliases: ["yellow", "yel"]
    },
    Blue => {
        ansi: "\x1b[34m",
        rgb: (36, 114, 200),
        aliases: ["blue"]
    },
    Green => {
        ansi: "\x1b[32m",
        rgb: (13, 188, 121),
        aliases: ["green"]
    },
    Cyan => {
        ansi: "\x1b[36m",
        rgb: (17, 168, 205),
        aliases: ["cyan"]
    },
    Magenta => {
        ansi: "\x1b[35m",
        rgb: (188, 63, 188),
        aliases: ["magenta", "purple"]
    },
    White => {
        ansi: "\x1b[37m",
        rgb: (229, 229, 229),
        aliases: ["white"]
    },
}

/// 颜色配置（预设或RGB）
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
            dir_gradient_start: Color::RGB {
                r: 58,
                g: 123,
                b: 213,
            },
            dir_gradient_end: Color::RGB {
                r: 0,
                g: 210,
                b: 255,
            },
            file_gradient_start: Color::RGB {
                r: 180,
                g: 180,
                b: 180,
            },
            file_gradient_end: Color::RGB {
                r: 255,
                g: 200,
                b: 120,
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

impl Color {
    /// 转ANSI转义码
    pub(crate) fn to_ansi(&self) -> anyhow::Result<String> {
        match self {
            Color::Preset { name } => {
                let preset = PresetColor::parse(name)?;
                Ok(preset.to_ansi().to_string())
            }
            Color::RGB { r, g, b } => Ok(format!("\x1b[38;2;{};{};{}m", r, g, b)),
        }
    }

    /// 转RGB值
    pub(crate) fn to_rgb(&self) -> anyhow::Result<(u8, u8, u8)> {
        match self {
            Color::Preset { name } => {
                let preset = PresetColor::parse(name)?;
                Ok(preset.to_rgb())
            }
            Color::RGB { r, g, b } => Ok((*r, *g, *b)),
        }
    }

    /// 校验颜色配置
    fn validate(&self) -> anyhow::Result<()> {
        if let Color::Preset { name } = self {
            PresetColor::parse(name)?;
        }
        Ok(())
    }
}

impl Theme {
    /// 从文件加载主题
    pub(crate) fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&text)?;
        theme.validate()?;
        Ok(theme)
    }

    /// 校验主题配置
    fn validate(&self) -> anyhow::Result<()> {
        self.reset.validate()?;
        self.fg_reset.validate()?;
        self.dir.validate()?;
        self.file.validate()?;
        self.error.validate()?;
        self.highlight_start.validate()?;
        self.highlight_end.validate()?;
        self.dir_gradient_start.validate()?;
        self.dir_gradient_end.validate()?;
        self.file_gradient_start.validate()?;
        self.file_gradient_end.validate()?;
        Ok(())
    }
}

/// 从环境变量或默认路径加载主题
pub(crate) fn load_theme_from_env_or_default() -> Theme {
    // 优先从环境变量加载
    if let Ok(path) = env::var("FSWHY_THEME") {
        if let Ok(theme) = Theme::load_from_file(Path::new(&path)) {
            return theme;
        }
    }
    // 其次从当前目录加载
    if let Ok(theme) = Theme::load_from_file(Path::new("theme.toml")) {
        return theme;
    }
    // 最后使用默认主题
    Theme::default()
}
