//! CLI 主题和样式定义
//!
//! 提供统一的颜色方案、图标和样式配置

use colored::Color;

/// UI 主题配置
#[derive(Debug, Clone)]
pub struct Theme {
    // 图标
    pub user_icon: &'static str,
    pub assistant_icon: &'static str,
    pub system_icon: &'static str,
    pub error_icon: &'static str,
    pub success_icon: &'static str,
    pub warning_icon: &'static str,
    pub loading_icon: &'static str,

    // Server 状态图标
    pub server_running: &'static str,
    pub server_stopped: &'static str,
    pub server_draining: &'static str,

    // 文件夹/文件
    pub folder_icon: &'static str,
    pub file_icon: &'static str,
    pub config_icon: &'static str,

    // 操作
    pub rocket_icon: &'static str,
    pub chat_icon: &'static str,
    pub tools_icon: &'static str,

    // 颜色
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub success_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // 使用 Unicode emoji 图标
            user_icon: "👤",
            assistant_icon: "🤖",
            system_icon: "ℹ️",
            error_icon: "❌",
            success_icon: "✅",
            warning_icon: "⚠️",
            loading_icon: "⏳",

            server_running: "🟢",
            server_stopped: "⏸️",
            server_draining: "🔄",

            folder_icon: "📁",
            file_icon: "📄",
            config_icon: "⚙️",

            rocket_icon: "🚀",
            chat_icon: "💬",
            tools_icon: "🔧",

            // 颜色方案
            primary_color: Color::Cyan,
            secondary_color: Color::Blue,
            accent_color: Color::Magenta,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            success_color: Color::Green,
        }
    }
}

impl Theme {
    /// 创建深色主题
    pub fn dark() -> Self {
        Self {
            primary_color: Color::BrightCyan,
            secondary_color: Color::BrightBlue,
            accent_color: Color::BrightMagenta,
            ..Default::default()
        }
    }

    /// 创建单色主题（无颜色）
    pub fn monochrome() -> Self {
        Self {
            primary_color: Color::White,
            secondary_color: Color::White,
            accent_color: Color::White,
            error_color: Color::White,
            warning_color: Color::White,
            success_color: Color::White,
            ..Default::default()
        }
    }
}

/// 主题类型
#[derive(Debug, Clone, Copy)]
pub enum ThemeType {
    Default,
    Dark,
    Monochrome,
}

impl ThemeType {
    pub fn to_theme(self) -> Theme {
        match self {
            ThemeType::Default => Theme::default(),
            ThemeType::Dark => Theme::dark(),
            ThemeType::Monochrome => Theme::monochrome(),
        }
    }
}

/// 图标模块 - 提供常用图标常量
pub mod icons {
    pub const USER: &str = "👤";
    pub const ASSISTANT: &str = "🤖";
    pub const SYSTEM: &str = "ℹ️";
    pub const ERROR: &str = "❌";
    pub const SUCCESS: &str = "✅";
    pub const WARNING: &str = "⚠️";
    pub const LOADING: &str = "⏳";
    pub const LIGHT_BULB: &str = "💡";

    pub const SERVER_RUNNING: &str = "🟢";
    pub const SERVER_STOPPED: &str = "⏸️";
    pub const SERVER_DRAINING: &str = "🔄";
    pub const SERVER_ERROR: &str = "🔴";

    pub const FOLDER: &str = "📁";
    pub const FILE: &str = "📄";
    pub const CONFIG: &str = "⚙️";

    pub const ROCKET: &str = "🚀";
    pub const CHAT: &str = "💬";
    pub const TOOLS: &str = "🔧";

    /// 生成数字徽章
    pub fn badge(count: usize) -> String {
        if count == 0 {
            "⭕".to_string()
        } else if count < 10 {
            format!("{}️⃣", count)
        } else {
            format!("[{}]", count)
        }
    }
}
