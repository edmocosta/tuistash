use tui::{backend::Backend, Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Span, Spans}, widgets::{Block, Borders, Tabs}};
use tui::text::Text;
use tui::widgets::{Cell, Paragraph, Row, Sparkline, Table, Wrap};

use crate::commands::stats::app::App;
use crate::commands::stats::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::stats::graph::{PipelineGraph, VertexEdge};
use crate::commands::stats::pipeline_viewer;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED))
        .select(app.tabs.index);

    f.render_widget(tabs, chunks[0]);

    match app.tabs.index {
        0 => draw_pipelines_tab(f, app, chunks[1]),
        // 1 => draw_second_tab(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_pipelines_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(80), Constraint::Min(8), Constraint::Length(7)].as_ref())
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
        .map(|i| {
            Row::new(
                vec![
                    Cell::from(Text::from(i.name.to_string())),
                ])
        })
        .collect();

    let headers = vec![
        "Name",
    ];

    let header_style: Style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
    let row_style: Style = Style::default().bg(Color::DarkGray);
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(header_style));

    let header = Row::new(header_cells)
        .style(row_style)
        .height(1);

    let pipelines = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .column_spacing(2)
        .highlight_style(selected_style)
        .widths(
            &[
                Constraint::Percentage(100), // Name
            ]
        );

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

        // f.render_widget(events_block, chunks[0]);

        // let events_chunks = Layout::default()
        //     .constraints(vec![Constraint::Percentage(15), Constraint::Length(3)])
        //     .direction(Direction::Horizontal)
        //     .margin(1)
        //     .split(chunks[0]);
        //
        // let events_in_block = Block::default()
        //     .title(Span::styled("In: ", Style::default().fg(Color::Gray)))
        //     .borders( Borders::RIGHT);
        // //
        // // let mut data:Vec<u64> = vec![];
        // // if let Some(selected_pipeline) = app.pipelines.selected_item() {
        // //     data = app.state.node_stats.pipeline_events_in
        // //         .get_mut(&selected_pipeline.name)
        // //         .unwrap()
        // //         .to_vec();
        // // }
        // //
        // // let events_in_sparkline = Sparkline::default()
        // //     .block(
        // //         events_in_block,
        // //     )
        // //     .bar_set(symbols::bar::NINE_LEVELS)
        // //     .data(&data)
        // //     .style(Style::default().fg(Color::Yellow));
        // //
        // // f.render_widget(events_in_sparkline, events_chunks[0]);

        let events_text: Vec<Spans>;
        if let Some(selected_pipeline) = &app.pipelines.selected_item() {
            if let Some(node_stats) = &app.state.node_stats {
                let selected_pipeline_stats = node_stats.pipelines.get(&selected_pipeline.name)
                    .unwrap();

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
                    Span::styled("Queue push duration: ", Style::default().fg(Color::DarkGray)),
                    Span::from(selected_pipeline_stats.events.queue_push_duration_in_millis.format_duration()),
                    Span::styled(" | ", Style::default().fg(Color::Yellow)),
                    Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                    Span::from(selected_pipeline_stats.events.duration_in_millis.format_duration()),
                ])];
            } else {
                events_text = vec![];
            }
        } else {
            events_text = vec![
                Spans::from(vec![
                    Span::styled("Select a pipeline", Style::default().fg(Color::DarkGray))
                ])
            ];
        }

        let info_paragraph = Paragraph::new(events_text).block(events_block).wrap(Wrap { trim: true });
        f.render_widget(info_paragraph, chunks[0]);


        // Pipeline viewer
        pipeline_viewer::render_pipeline_viewer(f, app, chunks[1]);
        // let rowss: Vec<Row> = app
        //     .plugins
        //     .items
        //     .iter()
        //     .map(|i| {
        //         let kind = i.kind_description.clone().unwrap_or(i.kind.to_string());
        //         let duration_cell_value: String;
        //         if let Some(queue_push_duration_in_millis) = i.queue_push_duration_in_millis {
        //             if i.duration_in_millis.unwrap_or(0) > 0 {
        //                 duration_cell_value = format!("{} / {} (Enqueuing)",
        //                                               i.duration_in_millis.unwrap_or(0).format_duration(),
        //                                               queue_push_duration_in_millis.format_duration()
        //                 );
        //             } else {
        //                 duration_cell_value = format!("{} (Enqueuing)", queue_push_duration_in_millis.format_duration());
        //             }
        //         } else {
        //             duration_cell_value = i.duration_in_millis.unwrap_or(0).format_duration();
        //         }
        //
        //         let mut cells = vec![
        //             Cell::from(Text::from(i.name.to_string())),
        //             Cell::from(Text::from(kind)),
        //             Cell::from(Text::from(i.events_in.unwrap_or(0).format_number())),
        //             Cell::from(Text::from(i.events_out.unwrap_or(0).format_number())),
        //             Cell::from(Text::from(duration_cell_value)),
        //             Cell::from(Text::from(i.id.to_string())),
        //         ];
        //
        //
        //         Row::new(cells)
        //     })
        //     .collect();
    }
}

// fn draw_selected_pipeline_section<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
//     where
//         B: Backend,
// {
//     let chunks = Layout::default()
//         .constraints(vec![Constraint::Length(3), Constraint::Percentage(82)])
//         .direction(Direction::Vertical)
//         .split(area);
//     {
//         // Pipeline info
//         let events_block = Block::default()
//             .title("Pipeline events")
//             .borders(Borders::ALL);
//
//         // f.render_widget(events_block, chunks[0]);
//         //
//         // let events_chunks = Layout::default()
//         //     .constraints(vec![Constraint::Percentage(15), Constraint::Length(3)])
//         //     .direction(Direction::Horizontal)
//         //     .margin(1)
//         //     .split(chunks[0]);
//         //
//         // let events_in_block = Block::default()
//         //     .title(Span::styled("In: ", Style::default().fg(Color::Gray)))
//         //     .borders( Borders::RIGHT);
//         //
//         // let mut data:Vec<u64> = vec![];
//         // if let Some(selected_pipeline) = app.pipelines.selected_item() {
//         //     data = app.state.pipeline_events_in
//         //         .get_mut(&selected_pipeline.name)
//         //         .unwrap()
//         //         .to_vec();
//         // }
//         //
//         // let events_in_sparkline = Sparkline::default()
//         //     .block(
//         //         events_in_block,
//         //     )
//         //     .bar_set(symbols::bar::NINE_LEVELS)
//         //     .data(&data)
//         //     .style(Style::default().fg(Color::Yellow));
//         //
//         // f.render_widget(events_in_sparkline, events_chunks[0]);
//
//         // let events_text: Vec<Spans>;
//         // if let Some(selected_pipeline) = &app.pipelines.selected_item() {
//         //     events_text = vec![Spans::from(vec![
//         //         Span::styled("In: ", Style::default().fg(Color::DarkGray)),
//         //         Span::from(selected_pipeline.pipeline.events.r#in.format_number()),
//         //         Span::styled(" | ", Style::default().fg(Color::Yellow)),
//         //         Span::styled("Filtered: ", Style::default().fg(Color::DarkGray)),
//         //         Span::from(selected_pipeline.pipeline.events.filtered.format_number()),
//         //         Span::styled(" | ", Style::default().fg(Color::Yellow)),
//         //         Span::styled("Out: ", Style::default().fg(Color::DarkGray)),
//         //         Span::from(selected_pipeline.pipeline.events.out.format_number()),
//         //         Span::styled(" | ", Style::default().fg(Color::Yellow)),
//         //         Span::styled("Queue push duration: ", Style::default().fg(Color::DarkGray)),
//         //         Span::from(selected_pipeline.pipeline.events.queue_push_duration_in_millis.format_duration()),
//         //         Span::styled(" | ", Style::default().fg(Color::Yellow)),
//         //         Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
//         //         Span::from(selected_pipeline.pipeline.events.duration_in_millis.format_duration()),
//         //     ])];
//         // } else {
//         //     events_text = vec![
//         //         Spans::from(vec![
//         //             Span::styled("Select a pipeline", Style::default().fg(Color::DarkGray))
//         //         ])
//         //     ];
//         // }
//         //
//         // let info_paragraph = Paragraph::new(events_text).block(events_block).wrap(Wrap { trim: true });
//         // f.render_widget(info_paragraph, chunks[0]);
//
//
//         // Plugins table
//         let rows: Vec<Row> = app
//             .plugins
//             .items
//             .iter()
//             .map(|i| {
//                 let kind = i.kind_description.clone().unwrap_or(i.kind.to_string());
//                 let duration_cell_value: String;
//                 if let Some(queue_push_duration_in_millis) = i.queue_push_duration_in_millis {
//                     if i.duration_in_millis.unwrap_or(0) > 0 {
//                         duration_cell_value = format!("{} / {} (Enqueuing)",
//                                                       i.duration_in_millis.unwrap_or(0).format_duration(),
//                                                       queue_push_duration_in_millis.format_duration()
//                         );
//                     } else {
//                         duration_cell_value = format!("{} (Enqueuing)", queue_push_duration_in_millis.format_duration());
//                     }
//                 } else {
//                     duration_cell_value = i.duration_in_millis.unwrap_or(0).format_duration();
//                 }
//
//                 let mut cells = vec![
//                     Cell::from(Text::from(i.name.to_string())),
//                     Cell::from(Text::from(kind)),
//                     Cell::from(Text::from(i.events_in.unwrap_or(0).format_number())),
//                     Cell::from(Text::from(i.events_out.unwrap_or(0).format_number())),
//                     Cell::from(Text::from(duration_cell_value)),
//                     Cell::from(Text::from(i.id.to_string())),
//                 ];
//
//
//                 Row::new(cells)
//             })
//             .collect();
//
//         let headers = vec![
//             "Name",
//             "Kind",
//             "Events in",
//             "Events out",
//             "Duration",
//             "ID",
//         ];
//
//         let header_style: Style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
//         let row_style: Style = Style::default().bg(Color::DarkGray);
//         let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);
//
//         let header_cells = headers
//             .iter()
//             .map(|h| Cell::from(*h).style(header_style));
//
//         let header = Row::new(header_cells)
//             .style(row_style)
//             .height(1);
//
//         let plugins = Table::new(rows)
//             .header(header)
//             .block(Block::default().borders(Borders::ALL).title("Plugins"))
//             .column_spacing(2)
//             .highlight_style(selected_style)
//             .widths(
//                 &[
//                     Constraint::Percentage(20), // Name
//                     Constraint::Percentage(10), // Kind
//                     Constraint::Percentage(10), // In
//                     Constraint::Percentage(10), // Out
//                     Constraint::Percentage(25), // Duration
//                     Constraint::Percentage(20), // ID
//                 ]
//             );
//
//         f.render_stateful_widget(plugins, chunks[1], &mut app.plugins.state);
//     }
// }