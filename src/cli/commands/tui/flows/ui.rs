use std::vec;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Tabs};
use ratatui::Frame;

use crate::commands::tui::app::App;

pub(crate) fn draw_flows_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_flow_widgets(f, app, chunks[0]);
}

fn draw_flow_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(2), Constraint::Min(0)])
        .direction(Direction::Vertical)
        .split(area);

    draw_analysis_window_options(f, app, chunks[0]);
    draw_pipelines_table(f, app, chunks[1]);
}

fn draw_analysis_window_options(f: &mut Frame, app: &mut App, area: Rect) {
    let options: Vec<&str> = vec![
        "Analysis window:",
        "~10s",
        "~1m",
        "~5m",
        "~15m",
        "~1h",
        "~24h",
        "Lifetime",
    ];

    let tab_options: Vec<Line> = options
        .iter()
        .map(|value| {
            Line::from(vec![Span::styled(
                value.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            )])
        })
        .collect();

    let tabs = Tabs::new(tab_options)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.flows_state.analysis_window_tabs.index);

    f.render_widget(tabs, area);
}

fn draw_pipelines_table(f: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app
        .pipelines_state
        .pipelines_table
        .items
        .iter()
        .map(|i| Row::new(vec![Cell::from(Text::from(i.name.to_string()))]))
        .collect();

    let headers = [
        "Name",
        "Input",
        "Filter",
        "Queue Backpressure",
        "Worker Concurrency",
    ];

    let header_style: Style = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::BOLD);

    let row_style: Style = Style::default().bg(Color::DarkGray);
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = headers.iter().map(|h| Cell::from(*h).style(header_style));
    let header = Row::new(header_cells).style(row_style).height(1);

    let widths: Vec<Constraint> = vec![Constraint::Ratio(rows.len() as u32, 1); rows.len()];
    let pipelines = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .column_spacing(2)
        .highlight_style(selected_style)
        .widths([
            Constraint::Percentage(20), // Name
            Constraint::Percentage(20), // Input
            Constraint::Percentage(20), // Filter
            Constraint::Percentage(20), // Queue Backpressure
            Constraint::Percentage(20), // Worker Concurrency
        ]);

    f.render_stateful_widget(
        pipelines,
        area,
        &mut app.pipelines_state.pipelines_table.state,
    );
}
