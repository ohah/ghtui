use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

pub fn pr_state_badge(state: &ghtui_core::types::PrState) -> Span<'static> {
    match state {
        ghtui_core::types::PrState::Open => Span::styled(
            " OPEN ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        ghtui_core::types::PrState::Closed => Span::styled(
            " CLOSED ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        ghtui_core::types::PrState::Merged => Span::styled(
            " MERGED ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    }
}

pub fn issue_state_badge(state: &ghtui_core::types::IssueState) -> Span<'static> {
    match state {
        ghtui_core::types::IssueState::Open => Span::styled(
            " OPEN ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        ghtui_core::types::IssueState::Closed => Span::styled(
            " CLOSED ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    }
}

pub fn run_status_badge(
    status: Option<&ghtui_core::types::RunStatus>,
    conclusion: Option<&ghtui_core::types::RunConclusion>,
) -> Span<'static> {
    if let Some(conclusion) = conclusion {
        match conclusion {
            ghtui_core::types::RunConclusion::Success => Span::styled(
                " ✓ SUCCESS ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            ghtui_core::types::RunConclusion::Failure => Span::styled(
                " ✗ FAILURE ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            ghtui_core::types::RunConclusion::Cancelled => Span::styled(
                " ◌ CANCELLED ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            _ => Span::styled(
                format!(" {} ", conclusion),
                Style::default().fg(Color::Yellow),
            ),
        }
    } else if let Some(status) = status {
        match status {
            ghtui_core::types::RunStatus::InProgress => Span::styled(
                " ● RUNNING ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            ghtui_core::types::RunStatus::Queued => {
                Span::styled(" ○ QUEUED ", Style::default().fg(Color::Yellow))
            }
            _ => Span::styled(
                format!(" {} ", status),
                Style::default().fg(Color::DarkGray),
            ),
        }
    } else {
        Span::styled(" UNKNOWN ", Style::default().fg(Color::DarkGray))
    }
}
