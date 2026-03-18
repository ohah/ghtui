use ghtui_core::AppState;
use ghtui_widgets::EditorView;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, title: &str, hint: &str) {
    let theme = &state.theme;

    // Full-width layout: hint bar + editor
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // hint
            Constraint::Min(0),    // editor content (includes its own status bar)
        ])
        .split(area);

    // Hint bar
    let hint_line = Line::from(vec![
        Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {}", hint), Style::default().fg(theme.fg_dim)),
    ]);
    let hint_bar = Paragraph::new(hint_line).style(Style::default().bg(theme.bg_subtle));
    frame.render_widget(hint_bar, chunks[0]);

    // Editor content using EditorView widget
    let editor_theme = ghtui_widgets::EditorTheme {
        text: theme.fg,
        line_number: theme.fg_muted,
        line_number_active: theme.fg,
        cursor: theme.accent,
        separator: theme.border,
        bg: theme.bg,
        border: theme.border,
        status_bg: theme.bg_subtle,
        status_fg: theme.fg_dim,
        selection_bg: theme.selection_bg,
        selection_fg: theme.tab_active_fg,
    };

    let editor_view = EditorView::new(&state.modal_editor, title)
        .theme(editor_theme)
        .status_hint("Ctrl+S:Submit  Esc:Cancel  Ctrl+Z:Undo  Ctrl+Y:Redo");
    frame.render_widget(editor_view, chunks[1]);
}
