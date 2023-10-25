use ratatui::layout::Alignment;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};
use std::vec;

use crate::commands::view::app::App;
use crate::commands::view::ui_node_tab::draw_node_tab;
use crate::commands::view::ui_pipelines_tab::draw_pipelines_tab;

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let constraints = if app.show_help || app.show_error.is_some() {
        vec![
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]
    } else {
        vec![Constraint::Length(3), Constraint::Min(0)]
    };

    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(f.size());

    let header_block = Block::default().borders(Borders::ALL).title(app.title);

    f.render_widget(header_block, chunks[0]);

    let title_chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(28),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .direction(Direction::Horizontal)
        .margin(1)
        .split(chunks[0]);

    let tab_titles = vec![
        Line::from(vec![
            Span::styled(
                "P",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("ipelines", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(
                "F",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("low", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(
                "N",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("ode", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.tabs.index);

    f.render_widget(tabs, title_chunks[0]);

    // Help text
    let help_text = Line::from(vec![
        Span::styled("Press", Style::default().fg(Color::DarkGray)),
        Span::styled(" <H> ", Style::default().fg(Color::Gray)),
        Span::styled("for help", Style::default().fg(Color::DarkGray)),
    ]);

    let w = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(w, title_chunks[1]);

    // Connection status
    let conn_status_span: Span = if app.connected {
        Span::styled("Connected", Style::default().fg(Color::Green))
    } else {
        Span::styled("Disconnected", Style::default().fg(Color::Red))
    };

    let status_text = Line::from(vec![
        conn_status_span,
        Span::styled(" @ ", Style::default().fg(Color::Gray)),
        Span::from(app.host),
        Span::styled(
            format!(" | Sampling every {}s", app.refresh_interval.as_secs()),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let w = Paragraph::new(status_text)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });

    f.render_widget(w, title_chunks[2]);

    if app.connected {
        match app.tabs.index {
            App::TAB_PIPELINES => draw_pipelines_tab(f, app, chunks[1]),
            App::TAB_NODE => draw_node_tab(f, app, chunks[1]),
            App::TAB_FLOW => {}
            _ => {}
        };
    }

    if app.show_error.is_some() {
        draw_error_panel(f, app, chunks[2]);
    } else if app.show_help {
        draw_help_panel(f, chunks[2]);
    }
}

fn draw_error_panel<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    if let Some(error) = &app.show_error {
        f.render_widget(Block::default().borders(Borders::ALL), area);

        let footer_chunks = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .direction(Direction::Horizontal)
            .margin(1)
            .split(area);

        let w = Paragraph::new(vec![Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red)),
            Span::styled(error, Style::default().fg(Color::DarkGray)),
        ])])
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

        f.render_widget(w, footer_chunks[0]);
    }
}

fn draw_help_panel<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let footer_block = Block::default().borders(Borders::ALL).title("Help");
    f.render_widget(footer_block, area);

    let footer_chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .margin(1)
        .split(area);

    let w = Paragraph::new(Line::from(vec![
        Span::styled("Shortcuts: ", Style::default().fg(Color::Gray)),
        Span::styled("<P> <N> ", Style::default().fg(Color::Yellow)),
        Span::styled("switch views", Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "<Up> <Down> <Left> <Right> ",
            Style::default().fg(Color::Yellow),
        ),
        Span::styled("navigate", Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Enter> ", Style::default().fg(Color::Yellow)),
        Span::styled(
            "show pipeline flow/component details",
            Style::default().fg(Color::Gray),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Esc> <Q> ", Style::default().fg(Color::Yellow)),
        Span::styled("exit", Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });

    f.render_widget(w, footer_chunks[0]);
}
