use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Sparkline, Table
    },
    Frame,
};
use sysinfo::System;

use crate::app::{App, SortMode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(12), // Charts
            Constraint::Min(0),    // Processes
        ])
        .split(f.size());

    draw_header(f, app, chunks[0]);
    draw_charts(f, app, chunks[1]);
    draw_process_table(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, _app: &App, area: Rect) {
    let name = System::name().unwrap_or("Unknown".to_string());
    let os_ver = System::os_version().unwrap_or("Unknown".to_string());
    let host = System::host_name().unwrap_or("Unknown".to_string());
    let uptime = System::uptime();

    let text = vec![
        Line::from(vec![
            Span::styled("SYSMON", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(format!("{} {}", name, os_ver), Style::default().fg(Color::White)),
            Span::raw(" | Host: "),
            Span::styled(host, Style::default().fg(Color::Green)),
            Span::raw(" | Uptime: "),
            Span::styled(format_uptime(uptime), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let block = Block::default().borders(Borders::ALL).title("System Info");
    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn format_uptime(mut seconds: u64) -> String {
    let days = seconds / 86400;
    seconds %= 86400;
    let hours = seconds / 3600;
    seconds %= 3600;
    let minutes = seconds / 60;
    seconds %= 60;
    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}

fn draw_charts(f: &mut Frame, app: &App, area: Rect) {
    let constraints = if area.width > 100 {
        [Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)]
    } else {
        [Constraint::Percentage(50), Constraint::Percentage(50), Constraint::Percentage(0)]
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // CPU Chart
    let cpu_data: Vec<u64> = app.cpu_history.iter().map(|&v| v as u64).collect();
    let cpu_sparkline = Sparkline::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .data(&cpu_data)
        .style(Style::default().fg(get_usage_color(app.cpu_history.back().copied().unwrap_or(0.0))));
    f.render_widget(cpu_sparkline, chunks[0]);

    // Memory Chart
    let mem_data: Vec<u64> = app.mem_history.iter().map(|&v| v as u64).collect();
    let mem_sparkline = Sparkline::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .data(&mem_data)
        .style(Style::default().fg(get_usage_color(app.mem_history.back().copied().unwrap_or(0.0))));
    f.render_widget(mem_sparkline, chunks[1]);

    // Network Chart (only if width allows)
    if chunks.len() > 2 {
        // Just showing RX for sparkline for simplicity, or sum? Let's do Max(RX, TX) scaling or just RX
        // Ideally we'd have two sparklines or a proper Chart, but sparkline is easier for "dashboard" look
        // Let's visualize RX speed scaled roughly.
        let net_data: Vec<u64> = app.net_history.iter().map(|(rx, _)| *rx / 1024).collect(); // KB/s roughly
        let net_sparkline = Sparkline::default()
            .block(Block::default().title("Network RX (KB/s)").borders(Borders::ALL))
            .data(&net_data)
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(net_sparkline, chunks[2]);
    }
}

fn get_usage_color(usage: f64) -> Color {
    if usage > 80.0 {
        Color::Red
    } else if usage > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn draw_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_cells = ["PID", "Name", "CPU %", "Memory"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1)
        .bottom_margin(1);

    let rows = app.processes.iter().map(|p| {
        let cells = vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}", p.cpu)),
            Cell::from(format!("{:.1} MB", p.memory as f64 / 1024.0 / 1024.0)),
        ];
        Row::new(cells).height(1)
    });

    let sort_label = match app.sort_mode {
        SortMode::Cpu => "Sort: CPU (s)",
        SortMode::Memory => "Sort: Mem (s)",
        SortMode::Pid => "Sort: PID (s)",
    };

    let title = format!(" Processes [Total: {}] | {} | Arrows to navigate ", app.processes.len(), sort_label);

    let t = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, area, &mut app.process_table_state);
}
