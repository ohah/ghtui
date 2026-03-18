use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let Some(ref ks) = state.keymap_settings else {
        return;
    };

    let width = 64u16.min(area.width.saturating_sub(4));
    let height = 30u16.min(area.height.saturating_sub(4));
    let popup_area = super::components::centered_rect(width, height, area);

    frame.render_widget(Clear, popup_area);

    let theme = &state.theme;

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Title hint
    let title_hint = if ks.capturing {
        " Press any key to assign... (Esc to cancel) "
    } else {
        " Enter: Edit  R: Reset All  Esc: Close "
    };

    // Build rows with category headers
    let mut last_category = String::new();

    // Calculate visible window: we want the selected row visible in the scrollable area
    // Reserve 1 line for potential first category header
    let max_visible = (height as usize).saturating_sub(3); // borders + title

    // First, build all display rows to know total count
    let mut all_rows: Vec<(bool, String)> = Vec::new(); // (is_header, text)
    let mut binding_to_row: Vec<usize> = Vec::new(); // maps binding index -> row index

    for (i, (cat, name, key, default)) in ks.bindings.iter().enumerate() {
        if cat != &last_category {
            all_rows.push((true, cat.clone()));
            last_category = cat.clone();
        }
        binding_to_row.push(all_rows.len());
        let modified = key != default;
        let display = if modified {
            format!("{:<22} {:<14} (default: {})", name, key, default)
        } else {
            format!("{:<22} {}", name, key)
        };
        all_rows.push((false, display));
        let _ = i; // used for indexing
    }

    // Find the row index of the selected binding
    let selected_row = binding_to_row.get(ks.selected).copied().unwrap_or(0);

    // Calculate scroll offset
    let scroll_offset = if selected_row >= max_visible {
        selected_row - max_visible + 2
    } else {
        0
    };

    // Render visible rows using all_rows with scroll
    let end = all_rows.len().min(scroll_offset + max_visible);

    // We need to map back from row to binding index
    let mut row_to_binding: Vec<Option<usize>> = vec![None; all_rows.len()];
    for (bi, &ri) in binding_to_row.iter().enumerate() {
        row_to_binding[ri] = Some(bi);
    }

    for (vi, row_idx) in (scroll_offset..end).enumerate() {
        let (is_header, text) = &all_rows[row_idx];
        let _ = vi;

        if *is_header {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", text),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            let bi = row_to_binding[row_idx].unwrap_or(0);
            let is_selected = bi == ks.selected;
            let is_capturing = is_selected && ks.capturing;

            let (cat, _name, key, default) = &ks.bindings[bi];
            let _ = (cat, key, default);

            let style = if is_capturing {
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.warning)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme.tab_active_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let prefix = if is_selected { ">" } else { " " };

            let display = if is_capturing {
                format!("{} {:<22} [press a key...]", prefix, _name)
            } else {
                format!("{} {}", prefix, text)
            };

            lines.push(Line::from(vec![Span::styled(display, style)]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title_hint)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.bg)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
