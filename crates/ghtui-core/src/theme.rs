use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Theme based on GitHub's Primer design system colors
#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,

    // Canvas (backgrounds)
    pub bg: Color,
    pub bg_subtle: Color,
    pub bg_overlay: Color,

    // Foreground
    pub fg: Color,
    pub fg_dim: Color,   // secondary text
    pub fg_muted: Color, // placeholder, disabled

    // Accent (links, interactive)
    pub accent: Color, // GitHub blue
    pub accent_emphasis: Color,

    // Status colors (matching GitHub exactly)
    pub success: Color, // green - open PR/issue
    pub danger: Color,  // red - closed
    pub warning: Color, // yellow
    pub info: Color,
    pub done: Color,     // purple - merged
    pub sponsors: Color, // pink

    // UI chrome
    pub border: Color,
    pub border_muted: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,

    // Tab bar
    pub tab_active_fg: Color,
    pub tab_active_border: Color,
    pub tab_inactive_fg: Color,
    pub tab_counter_bg: Color,

    // Status bar / header
    pub header_bg: Color,
    pub header_fg: Color,
    pub footer_bg: Color,

    // Diff
    pub diff_add_fg: Color,
    pub diff_add_bg: Color,
    pub diff_remove_fg: Color,
    pub diff_remove_bg: Color,
    pub diff_hunk: Color,

    // State labels (matching GitHub badge colors)
    pub state_open_fg: Color,
    pub state_open_bg: Color,
    pub state_closed_fg: Color,
    pub state_closed_bg: Color,
    pub state_merged_fg: Color,
    pub state_merged_bg: Color,
    pub state_draft_fg: Color,
    pub state_draft_bg: Color,
}

impl Theme {
    /// GitHub Dark Default theme
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            // Canvas
            bg: Color::Rgb(13, 17, 23),         // #0d1117
            bg_subtle: Color::Rgb(22, 27, 34),  // #161b22
            bg_overlay: Color::Rgb(30, 35, 44), // #1e232c

            // Foreground
            fg: Color::Rgb(230, 237, 243),       // #e6edf3
            fg_dim: Color::Rgb(125, 133, 144),   // #7d8590
            fg_muted: Color::Rgb(110, 118, 129), // #6e7681

            // Accent
            accent: Color::Rgb(88, 166, 255),          // #58a6ff
            accent_emphasis: Color::Rgb(31, 111, 235), // #1f6feb

            // Status
            success: Color::Rgb(63, 185, 80),   // #3fb950
            danger: Color::Rgb(248, 81, 73),    // #f85149
            warning: Color::Rgb(210, 153, 34),  // #d29922
            info: Color::Rgb(88, 166, 255),     // #58a6ff
            done: Color::Rgb(163, 113, 247),    // #a371f7
            sponsors: Color::Rgb(219, 97, 162), // #db61a2

            // UI chrome
            border: Color::Rgb(48, 54, 61),       // #30363d
            border_muted: Color::Rgb(33, 38, 45), // #21262d
            selection_bg: Color::Rgb(23, 54, 93), // #17365d
            selection_fg: Color::Rgb(230, 237, 243),

            // Tabs (GitHub style: orange underline for active)
            tab_active_fg: Color::Rgb(230, 237, 243),
            tab_active_border: Color::Rgb(246, 124, 43), // #f67c2b (primer orange)
            tab_inactive_fg: Color::Rgb(125, 133, 144),
            tab_counter_bg: Color::Rgb(48, 54, 61),

            // Header
            header_bg: Color::Rgb(22, 27, 34), // #161b22
            header_fg: Color::Rgb(230, 237, 243),
            footer_bg: Color::Rgb(13, 17, 23),

            // Diff
            diff_add_fg: Color::Rgb(63, 185, 80),
            diff_add_bg: Color::Rgb(18, 56, 25), // #12381a
            diff_remove_fg: Color::Rgb(248, 81, 73),
            diff_remove_bg: Color::Rgb(67, 20, 23), // #431417
            diff_hunk: Color::Rgb(163, 113, 247),

            // State labels
            state_open_fg: Color::Rgb(230, 237, 243),
            state_open_bg: Color::Rgb(35, 134, 54), // #238636
            state_closed_fg: Color::Rgb(230, 237, 243),
            state_closed_bg: Color::Rgb(218, 54, 51), // #da3633
            state_merged_fg: Color::Rgb(230, 237, 243),
            state_merged_bg: Color::Rgb(130, 80, 223), // #8250df
            state_draft_fg: Color::Rgb(230, 237, 243),
            state_draft_bg: Color::Rgb(110, 118, 129), // #6e7681
        }
    }

    /// GitHub Light Default theme
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            // Canvas
            bg: Color::Rgb(255, 255, 255),        // #ffffff
            bg_subtle: Color::Rgb(246, 248, 250), // #f6f8fa
            bg_overlay: Color::Rgb(255, 255, 255),

            // Foreground
            fg: Color::Rgb(31, 35, 40),          // #1f2328
            fg_dim: Color::Rgb(101, 109, 118),   // #656d76
            fg_muted: Color::Rgb(139, 148, 158), // #8b949e

            // Accent
            accent: Color::Rgb(9, 105, 218), // #0969da
            accent_emphasis: Color::Rgb(9, 105, 218),

            // Status
            success: Color::Rgb(26, 127, 55), // #1a7f37
            danger: Color::Rgb(207, 34, 46),  // #cf222e
            warning: Color::Rgb(156, 110, 0), // #9c6e00
            info: Color::Rgb(9, 105, 218),
            done: Color::Rgb(130, 80, 223),     // #8250df
            sponsors: Color::Rgb(191, 57, 137), // #bf3989

            // UI chrome
            border: Color::Rgb(208, 215, 222),       // #d0d7de
            border_muted: Color::Rgb(216, 222, 228), // #d8dee4
            selection_bg: Color::Rgb(218, 230, 249), // #dae6f9
            selection_fg: Color::Rgb(31, 35, 40),

            // Tabs
            tab_active_fg: Color::Rgb(31, 35, 40),
            tab_active_border: Color::Rgb(246, 124, 43),
            tab_inactive_fg: Color::Rgb(101, 109, 118),
            tab_counter_bg: Color::Rgb(175, 184, 193),

            // Header
            header_bg: Color::Rgb(246, 248, 250),
            header_fg: Color::Rgb(31, 35, 40),
            footer_bg: Color::Rgb(246, 248, 250),

            // Diff
            diff_add_fg: Color::Rgb(26, 127, 55),
            diff_add_bg: Color::Rgb(218, 251, 225), // #dafbe1
            diff_remove_fg: Color::Rgb(207, 34, 46),
            diff_remove_bg: Color::Rgb(255, 235, 233), // #ffebe9
            diff_hunk: Color::Rgb(130, 80, 223),

            // State labels
            state_open_fg: Color::Rgb(255, 255, 255),
            state_open_bg: Color::Rgb(26, 127, 55),
            state_closed_fg: Color::Rgb(255, 255, 255),
            state_closed_bg: Color::Rgb(207, 34, 46),
            state_merged_fg: Color::Rgb(255, 255, 255),
            state_merged_bg: Color::Rgb(130, 80, 223),
            state_draft_fg: Color::Rgb(255, 255, 255),
            state_draft_bg: Color::Rgb(101, 109, 118),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    // Convenience style methods
    pub fn text(&self) -> Style {
        Style::default().fg(self.fg)
    }

    pub fn text_dim(&self) -> Style {
        Style::default().fg(self.fg_dim)
    }

    pub fn text_muted(&self) -> Style {
        Style::default().fg(self.fg_muted)
    }

    pub fn text_accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn text_bold(&self) -> Style {
        Style::default().fg(self.fg).add_modifier(Modifier::BOLD)
    }

    pub fn selected(&self) -> Style {
        Style::default().fg(self.selection_fg).bg(self.selection_bg)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
}
