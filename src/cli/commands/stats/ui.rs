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

use crate::commands::stats::app::App;
use crate::commands::stats::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::stats::node_charts::render_node_charts;
use crate::commands::stats::pipeline_viewer;

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size());

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title("Logstash")
        .title(app.title);

    f.render_widget(main_block, chunks[0]);

    let title_chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .margin(1)
        .split(chunks[0]);

    let tab_titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )
        .select(app.tabs.index);

    f.render_widget(tabs, title_chunks[0]);

    if let Some(node_info) = &app.state.node_info {
        let status_text = vec![Spans::from(vec![
            Span::styled("Connected", Style::default().fg(Color::Green)),
            Span::styled(" @ ", Style::default().fg(Color::Gray)),
            Span::from(node_info.node.http_address.to_string()),
            Span::styled(
                format!(" | Sampling every {}ms", app.refresh_interval.as_millis()),
                Style::default().fg(Color::Gray),
            ),
        ])];

        let w = Paragraph::new(status_text)
            .alignment(Alignment::Right)
            .wrap(Wrap { trim: true });

        f.render_widget(w, title_chunks[1]);
    }

    match app.tabs.index {
        0 => draw_pipelines_tab(f, app, chunks[1]),
        1 => draw_node_tab(f, app, chunks[1]),
        _ => {}
    };
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

        let overview_text: Vec<Spans>;
        if let Some(node_stats) = &app.state.node_stats {
            overview_text = vec![Spans::from(vec![
                // Span::styled("Transport: ", Style::default().fg(Color::DarkGray)),
                // Span::from(node_stats.http_address.as_str()),
                // Span::styled(" | ", Style::default().fg(Color::Yellow)),
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
            ])];
        } else {
            overview_text = vec![Spans::from(vec![Span::styled(
                "-",
                Style::default().fg(Color::DarkGray),
            )])];
        }

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
        // Pipeline info
        let events_block = Block::default()
            .title("Pipeline events")
            .borders(Borders::ALL);

        let events_text: Vec<Spans>;
        if let Some(selected_pipeline) = &app.pipelines.selected_item() {
            if let Some(node_stats) = &app.state.node_stats {
                let selected_pipeline_stats =
                    node_stats.pipelines.get(&selected_pipeline.name).unwrap();

                events_text = vec![Spans::from(vec![
                    Span::styled("In: ", Style::default().fg(Color::DarkGray)),
                    Span::from(selected_pipeline_stats.events.r#in.format_number()),
                    Span::styled(" | ", Style::default().fg(Color::Yellow)),
                    Span::styled("Filtered: ", Style::default().fg(Color::DarkGray)),
                    Span::from(selected_pipeline_stats.events.filtered.format_number()),
                    Span::styled(" | ", Style::default().fg(Color::Yellow)),
                    Span::styled("Out: ", Style::default().fg(Color::DarkGray)),
                    Span::from(selected_pipeline_stats.events.out.format_number()),
                    Span::styled(" | ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        "Queue push duration (ms/e): ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::from(
                        selected_pipeline_stats
                            .events
                            .queue_push_duration_in_millis
                            .format_duration_per_event(selected_pipeline_stats.events.r#in as u64),
                    ),
                    Span::styled(" | ", Style::default().fg(Color::Yellow)),
                    Span::styled("Duration (ms/e): ", Style::default().fg(Color::DarkGray)),
                    Span::from(
                        selected_pipeline_stats
                            .events
                            .duration_in_millis
                            .format_duration_per_event(selected_pipeline_stats.events.out as u64),
                    ),
                ])];
            } else {
                events_text = vec![];
            }
        } else {
            events_text = vec![Spans::from(vec![Span::styled(
                "Select a pipeline",
                Style::default().fg(Color::DarkGray),
            )])];
        }

        let info_paragraph = Paragraph::new(events_text)
            .block(events_block)
            .wrap(Wrap { trim: true });
        f.render_widget(info_paragraph, chunks[0]);

        // Pipeline
        pipeline_viewer::render_pipeline_viewer(f, app, chunks[1]);
    }
}
