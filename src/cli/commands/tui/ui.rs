use std::collections::HashMap;
use std::vec;

use ratatui::layout::{Alignment, Flex};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::commands::tui::app::App;
use crate::commands::tui::flows::ui::{draw_flows_tab, flows_tab_shortcuts_help};
use crate::commands::tui::node::ui::draw_node_tab;
use crate::commands::tui::pipelines::ui::{draw_pipelines_tab, pipelines_tab_shortcuts_help};
use crate::commands::tui::threads::ui::{draw_threads_tab, threads_tab_shortcuts_help};

pub(crate) fn draw(f: &mut Frame, app: &mut App) {
    let last_error_message = app.data.read().unwrap().last_error_message().clone();
    let constraints = if app.show_help || last_error_message.is_some() {
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
        .split(f.area());

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(app.title.as_str());

    f.render_widget(header_block, chunks[0]);

    let title_chunks = Layout::default()
        .flex(Flex::Legacy)
        .constraints(
            [
                Constraint::Length(37),
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
                Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
            ),
            Span::styled("ipelines", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                "F",
                Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
            ),
            Span::styled("lows", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                "T",
                Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
            ),
            Span::styled("hreads", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                "N",
                Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
            ),
            Span::styled("ode", Style::default().add_modifier(Modifier::BOLD)),
        ]),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.tabs.index);

    f.render_widget(tabs, title_chunks[0]);

    // Help text
    let help_text = Line::from(vec![
        Span::styled(" [H] ", Style::default().fg(Color::Yellow)),
        Span::styled("for help", Style::default().fg(Color::DarkGray)),
    ]);

    let w = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(w, title_chunks[1]);

    // Connection status
    let errored = app.data.read().unwrap().errored();
    let conn_status_span: Span = if errored {
        Span::styled("Disconnected", Style::default().fg(Color::Red))
    } else {
        Span::styled("Connected", Style::default().fg(Color::Green))
    };

    let mut status_text_spans = vec![
        conn_status_span,
        Span::styled(" @ ", Style::default().fg(Color::Gray)),
        Span::from(app.host.as_str()),
    ];

    if let Some(interval) = app.sampling_interval {
        status_text_spans.push(Span::styled(
            format!(" | Sampling every {}s", interval.as_secs()),
            Style::default().fg(Color::Gray),
        ));
    }

    let w = Paragraph::new(Line::from(status_text_spans)).alignment(Alignment::Right);

    f.render_widget(w, title_chunks[2]);

    if !errored {
        match app.tabs.index {
            App::TAB_PIPELINES => draw_pipelines_tab(f, app, chunks[1]),
            App::TAB_NODE => draw_node_tab(f, app, chunks[1]),
            App::TAB_FLOWS => draw_flows_tab(f, app, chunks[1]),
            App::TAB_THREADS => draw_threads_tab(f, app, chunks[1]),
            _ => {}
        };
    }

    if last_error_message.is_some() {
        draw_error_panel(f, app, chunks[2]);
    } else if app.show_help {
        let (defaults, shortcuts) = match app.tabs.index {
            App::TAB_PIPELINES => (true, pipelines_tab_shortcuts_help(app)),
            App::TAB_FLOWS => (true, flows_tab_shortcuts_help(app)),
            App::TAB_THREADS => (true, threads_tab_shortcuts_help(app)),
            _ => (true, Default::default()),
        };

        draw_help_panel(f, defaults, shortcuts, chunks[2]);
    }
}

fn draw_error_panel(f: &mut Frame, app: &App, area: Rect) {
    let last_error_message = app.data.read().unwrap().last_error_message().clone();
    if let Some(error) = &last_error_message {
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

fn draw_help_panel(f: &mut Frame, defaults: bool, shortcuts: HashMap<String, String>, area: Rect) {
    let footer_block = Block::default().borders(Borders::ALL).title("Help");
    f.render_widget(footer_block, area);

    let footer_chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .margin(1)
        .split(area);

    f.render_widget(
        default_help_paragraph(defaults, shortcuts),
        footer_chunks[0],
    );
}

fn default_help_paragraph<'a>(defaults: bool, shortcuts: HashMap<String, String>) -> Paragraph<'a> {
    let separator_span = Span::styled(" |", Style::default());

    let mut content = vec![Span::styled("Shortcuts:", Style::default().fg(Color::Gray))];
    let mut shortcuts_vec: Vec<(&String, &String)> = shortcuts.iter().collect();
    shortcuts_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (key, desc) in shortcuts_vec {
        content.push(separator_span.clone());
        content.push(Span::styled(
            format!("{} ", key),
            Style::default().fg(Color::Yellow),
        ));
        content.push(Span::styled(
            desc.to_string(),
            Style::default().fg(Color::Gray),
        ));
    }

    if defaults {
        content.extend(vec![
            separator_span.clone(),
            Span::styled("[P][F][N] ", Style::default().fg(Color::Yellow)),
            Span::styled("switch tabs", Style::default().fg(Color::Gray)),
            separator_span.clone(),
            Span::styled("[▲][▼][◀][▶][Tab] ", Style::default().fg(Color::Yellow)),
            Span::styled("navigate", Style::default().fg(Color::Gray)),
            separator_span.clone(),
            Span::styled("[Esc][Q] ", Style::default().fg(Color::Yellow)),
            Span::styled("exit", Style::default().fg(Color::Gray)),
        ]);
    }

    Paragraph::new(Line::from(content))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}
