use std::cmp::max;
use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span, Style, Stylize, Text};

use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::commands::tui::app::App;
use crate::commands::tui::threads::state::{ThreadTableItem, ThreadsState};
use crate::commands::tui::widgets::{
    TABLE_HEADER_CELL_STYLE, TABLE_HEADER_ROW_STYLE, TABLE_SELECTED_ROW_STYLE,
    TABLE_SELECTED_ROW_SYMBOL,
};

pub(crate) fn draw_threads_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_threads_widgets(f, app, chunks[0]);
}

pub(crate) fn threads_tab_shortcuts_help(_: &App) -> HashMap<String, String> {
    let mut keys = HashMap::with_capacity(1);
    keys.insert("[↵]".to_string(), "view thread traces".to_string());
    keys
}

fn draw_threads_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Min(0), Constraint::Length(2)])
        .direction(Direction::Vertical)
        .split(area);

    let tables_constraints = if app.threads_state.show_selected_thread {
        vec![Constraint::Percentage(40), Constraint::Percentage(60)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let tables_chunks = Layout::default()
        .constraints(tables_constraints)
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    draw_threads_table(f, app, tables_chunks[0]);

    if app.threads_state.show_selected_thread {
        draw_selected_thread_traces(f, app, tables_chunks[1]);
    }

    draw_labels_panel(f, chunks[1]);
}

fn draw_labels_panel(f: &mut Frame, area: Rect) {
    let states_labels = ["NEW", "RUNNABLE", "BLOCKED", "WAITING", "TIMED_WAITING"];

    let mut spans = Vec::with_capacity(states_labels.len() * 2);
    for state in states_labels {
        spans.push(Span::styled(
            " ■",
            Style::default().fg(thread_state_color(state)),
        ));
        spans.push(Span::styled(
            format!(" {}", state),
            Style::default().fg(Color::DarkGray),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_selected_thread_traces(f: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app
        .threads_state
        .selected_thread_traces
        .items
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let mut row_value = value.to_string();
            if let Some(selected) = app.threads_state.selected_thread_traces.state.selected() {
                if selected == i
                    && app.threads_state.selected_thread_trace_value_offset < value.len()
                {
                    let full_value = value.to_string();
                    let strip_value =
                        &full_value[app.threads_state.selected_thread_trace_value_offset..];
                    row_value = strip_value.to_string();
                }
            }

            Row::new(vec![Cell::from(Text::from(row_value.to_string()))])
        })
        .collect();

    let traces_block_title = if let Some(thread) = app.threads_state.threads_table.selected_item() {
        thread.name.to_string()
    } else {
        "Traces".to_string()
    };

    let widths: Vec<Constraint> = vec![Constraint::Ratio(rows.len() as u32, 1); rows.len()];
    let traces = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::from(traces_block_title).bold()),
        )
        .column_spacing(2)
        .highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL)
        .widths([
            Constraint::Percentage(100), // Name
        ]);

    f.render_stateful_widget(
        traces,
        area,
        &mut app.threads_state.selected_thread_traces.state,
    );
}

fn draw_threads_table(f: &mut Frame, app: &mut App, area: Rect) {
    let states_cell_size_percentage: u16 = 44;
    let mut thread_skipped_states: usize = 0;

    let rows: Vec<Row> = app
        .threads_state
        .threads_table
        .items
        .iter()
        .map(|i| {
            let (skipped_states, state_line) = create_thread_states_line(
                &app.threads_state,
                i,
                &states_cell_size_percentage,
                &area.width,
            );
            if skipped_states != 0 {
                thread_skipped_states = skipped_states;
            }

            let cells = vec![
                Cell::from(Text::from(i.id.to_string())),
                Cell::from(Line::from(vec![
                    Span::styled("■ ", Style::default().fg(thread_state_color(&i.state))),
                    Span::raw(i.name.to_string()),
                ])),
                Cell::from(Text::from(format!("{} %", i.percent_of_cpu_time))),
                Cell::from(state_line),
            ];

            Row::new(cells)
        })
        .collect();

    let headers = ["ID", "Name", "CPU Usage"];

    let mut header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE))
        .collect();

    header_cells.push(Cell::from(create_thread_states_title(
        &app.threads_state,
        &states_cell_size_percentage,
        &area.width,
        thread_skipped_states,
    )));

    let header = Row::new(header_cells)
        .style(TABLE_HEADER_ROW_STYLE)
        .height(1);

    let widths: Vec<Constraint> = vec![
        Constraint::Percentage(4),                           // Thread ID
        Constraint::Percentage(45),                          // Name
        Constraint::Percentage(7),                           // CPU Usage
        Constraint::Percentage(states_cell_size_percentage), // States
    ];

    let busiest_threads = if let Some(hot_threads) = app.data.hot_threads() {
        hot_threads.hot_threads.busiest_threads
    } else {
        app.threads_state.threads_table.items.len() as u64
    };

    let threads = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Busiest {} threads", busiest_threads)),
        )
        .column_spacing(2)
        .highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL);

    f.render_stateful_widget(threads, area, &mut app.threads_state.threads_table.state);
}

fn create_thread_states_title<'a>(
    threads_state: &ThreadsState,
    line_constraint_percentage: &u16,
    area_width: &u16,
    thread_skipped_states: usize,
) -> Line<'a> {
    let format = format_description::parse("[hour]:[minute]:[second]").unwrap();
    let oldest = threads_state
        .threads_table_states_times
        .front()
        .map(|p| p.format(&format).unwrap_or("-".to_string()))
        .unwrap_or("-".to_string());

    let oldest_span = Span::styled(oldest, TABLE_HEADER_CELL_STYLE);
    if thread_skipped_states == 0 {
        return Line::from(vec![oldest_span]);
    }

    let max_line_width = max(
        ((*line_constraint_percentage as f64 / 100.0) * (*area_width as f64)) as i64,
        1,
    );

    let cells_size = max(max_line_width / 3, 1);
    let middle = threads_state
        .threads_table_states_times
        .get(threads_state.threads_table_states_times.len() / 2)
        .map(|p| p.format(&format).unwrap_or("-".to_string()))
        .unwrap_or("-".to_string());

    let newest = threads_state
        .threads_table_states_times
        .back()
        .map(|p| p.format(&format).unwrap_or("-".to_string()))
        .unwrap_or("-".to_string());

    let middle_span = Span::styled(middle, TABLE_HEADER_CELL_STYLE);
    let newest_span = Span::styled(newest, TABLE_HEADER_CELL_STYLE);

    let oldest_span_width = oldest_span.width() as i64;
    let middle_span_width = middle_span.width() as i64;
    let newest_span_width = newest_span.width() as i64;

    let spans: Vec<Span> = vec![
        oldest_span,
        Span::raw(" ".repeat(max(cells_size - oldest_span_width, 1) as usize)),
        Span::raw(" ".repeat(max((cells_size - middle_span_width) / 2, 0) as usize)),
        middle_span,
        Span::raw(" ".repeat(max(cells_size - middle_span_width, 1) as usize)),
        Span::raw(" ".repeat(max(((cells_size - 1) - newest_span_width) / 2, 0) as usize)),
        newest_span,
    ];

    Line::from(spans)
}

fn create_thread_states_line<'a>(
    threads_state: &ThreadsState,
    thread_table_item: &ThreadTableItem,
    line_constraint_percentage: &u16,
    area_width: &u16,
) -> (usize, Line<'a>) {
    let bar = "▆";
    let bar_width = Span::raw(bar).width();

    if let Some(states) = threads_state
        .threads_table_states
        .get(&thread_table_item.id)
    {
        let max_line_width = (((*line_constraint_percentage as f64 / 100.0) * (*area_width as f64))
            as usize)
            - bar_width;
        let skip_states = if states.len() * bar_width >= max_line_width {
            let max_bars = max_line_width / bar_width;
            states.len() - max_bars
        } else {
            0
        };

        let mut spans: Vec<Span> = Vec::with_capacity(states.len() - skip_states);
        for state in states.iter().skip(skip_states) {
            spans.push(Span::styled(
                bar,
                Style::default().fg(thread_state_color(state)),
            ));
        }

        (skip_states, Line::from(spans))
    } else {
        (
            0,
            Line::from(Span::styled(
                bar,
                Style::default().fg(thread_state_color(&thread_table_item.state)),
            )),
        )
    }
}

fn thread_state_color(state: &str) -> Color {
    match state.to_uppercase().as_str() {
        "NEW" => Color::Blue,
        "RUNNABLE" => Color::Green,
        "BLOCKED" => Color::Red,
        "WAITING" => Color::LightYellow,
        "TIMED_WAITING" => Color::Indexed(208),
        "TERMINATED" => Color::Gray,
        _ => Color::Reset,
    }
}
