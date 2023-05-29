use std::vec;
use tui::layout::Alignment;
use tui::text::Text;
use tui::widgets::{Cell, Paragraph, Row, Table, Wrap};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::commands::view::app::App;
use crate::commands::view::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::view::node_charts::render_node_charts;
use crate::commands::view::pipeline_view;

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let constraints = if app.show_help {
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

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title("Logstash")
        .title(app.title);

    f.render_widget(header_block, chunks[0]);

    let title_chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(20),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .direction(Direction::Horizontal)
        .margin(1)
        .split(chunks[0]);

    let tab_titles = vec![
        Spans::from(vec![
            Span::styled(
                "P",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("ipelines", Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::styled(
                "N",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("ode", Style::default().fg(Color::Green)),
        ]),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.tabs.index);

    f.render_widget(tabs, title_chunks[0]);

    // Help text
    let help_text = vec![Spans::from(vec![
        Span::styled("Press", Style::default().fg(Color::DarkGray)),
        Span::styled(" <H> ", Style::default().fg(Color::Gray)),
        Span::styled("for help", Style::default().fg(Color::DarkGray)),
    ])];

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

    let status_text = vec![Spans::from(vec![
        conn_status_span,
        Span::styled(" @ ", Style::default().fg(Color::Gray)),
        Span::from(app.host),
        Span::styled(
            format!(" | Sampling every {}ms", app.refresh_interval.as_millis()),
            Style::default().fg(Color::Gray),
        ),
    ])];

    let w = Paragraph::new(status_text)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });

    f.render_widget(w, title_chunks[2]);

    if app.connected {
        match app.tabs.index {
            0 => draw_pipelines_tab(f, app, chunks[1]),
            1 => draw_node_tab(f, app, chunks[1]),
            _ => {}
        };
    }

    if app.show_help {
        draw_help_panel(f, chunks[2]);
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

    let w = Paragraph::new(vec![Spans::from(vec![
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
        Span::styled("<F> ", Style::default().fg(Color::Yellow)),
        Span::styled("show pipeline flow", Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Enter> ", Style::default().fg(Color::Yellow)),
        Span::styled("show component details", Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Esc> <Q> ", Style::default().fg(Color::Yellow)),
        Span::styled("exit", Style::default().fg(Color::Gray)),
    ])])
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });

    f.render_widget(w, footer_chunks[0]);
}

fn draw_node_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_node_widgets(f, app, chunks[0]);
}

fn draw_node_widgets<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(82)])
        .direction(Direction::Vertical)
        .split(area);
    {
        // Node overview
        let events_block = Block::default().title("Overview").borders(Borders::ALL);
        let overview_text: Vec<Spans> = if let Some(node_stats) = &app.state.node_stats {
            vec![Spans::from(vec![
                Span::styled("Events in: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.events.r#in.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Events out: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.events.out.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Reloads: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.reloads.successes.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Pipeline workers: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.pipeline.workers.to_string()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Pipeline batch size: ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::from(node_stats.pipeline.batch_size.to_string()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.version.as_str()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.jvm.uptime_in_millis.format_duration()),
            ])]
        } else {
            vec![Spans::from(vec![Span::styled(
                "-",
                Style::default().fg(Color::DarkGray),
            )])]
        };

        let info_paragraph = Paragraph::new(overview_text)
            .block(events_block)
            .wrap(Wrap { trim: true });
        f.render_widget(info_paragraph.clone(), chunks[0]);

        render_node_charts(f, app, chunks[1]);
    }
}

fn draw_pipelines_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_pipelines_widgets(f, app, chunks[0]);
}

fn draw_pipelines_widgets<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(18), Constraint::Percentage(82)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        draw_pipelines_table(f, app, chunks[0]);
        draw_selected_pipeline_section(f, app, chunks[1]);
    }
}

fn draw_pipelines_table<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let rows: Vec<Row> = app
        .pipelines
        .items
        .iter()
        .map(|i| Row::new(vec![Cell::from(Text::from(i.name.to_string()))]))
        .collect();

    let headers = vec!["Name"];
    let header_style: Style = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::BOLD);

    let row_style: Style = Style::default().bg(Color::DarkGray);
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = headers.iter().map(|h| Cell::from(*h).style(header_style));
    let header = Row::new(header_cells).style(row_style).height(1);

    let pipelines = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .column_spacing(2)
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Percentage(100), // Name
        ]);

    f.render_stateful_widget(pipelines, area, &mut app.pipelines.state);
}

fn draw_selected_pipeline_section<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(82)])
        .direction(Direction::Vertical)
        .split(area);
    {
        pipeline_view::render_pipeline_events_block(f, app, chunks[0]);
        pipeline_view::render_pipeline_vertices(f, app, chunks[1]);
    }
}
