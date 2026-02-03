use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Sparkline, Table
    },
    Frame,
};

use crate::app::{App, ActivePanel};
use crate::process::SortBy;
use crate::system::SystemMonitor;


pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Percentage(40), // Top Panel (Charts/Logs)
            Constraint::Percentage(60), // Bottom Panel (Processes)
        ])
        .split(f.size());

    draw_header(f, app, chunks[0]);
    
    match app.active_panel {
        ActivePanel::Charts => draw_dashboard(f, app, chunks[1]),
        ActivePanel::Logs => draw_logs(f, app, chunks[1]),
        ActivePanel::History => draw_history_panel(f, app, chunks[1]),
    }
    
    draw_process_table(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = " SYSMON ULTIMATE v2.0 ";
    let help = " Q:Quit | S:Sort | K:Kill | L:Logs | H:History ";
    
    let sys_name = SystemMonitor::name();
    let host = SystemMonitor::host_name();
    let os_ver = SystemMonitor::os_version();
    let uptime = SystemMonitor::uptime();
    
    // Format uptime
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;
    let uptime_str = format!("{}d {}h {}m", days, hours, mins);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));
        
    let text = vec![
        Line::from(vec![
            Span::styled(title, Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
            Span::raw(format!("| {} {} | {} | ", sys_name, os_ver, host)),
            Span::styled(format!("Up: {} ", uptime_str), Style::default().fg(Color::Green)),
            Span::raw(format!("| Size: {}x{}", app.terminal_size.0, app.terminal_size.1)),
        ]),
        Line::from(vec![Span::raw(help)])
    ];
    
    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
        
    f.render_widget(p, area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // CPU & Cores
            Constraint::Percentage(33), // Mem & Swap
            Constraint::Percentage(33), // Net & Disk
        ])
        .split(area);
        
    draw_cpu_panel(f, app, chunks[0]);
    draw_mem_panel(f, app, chunks[1]);
    draw_net_panel(f, app, chunks[2]);
}

fn draw_cpu_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title(" CPU Usage ").borders(Borders::ALL);
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);
        
    // Global Gauge
    let cpu_val = app.cpu_history.latest().copied().unwrap_or(0.0);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(if cpu_val > 80.0 { Color::Red } else { Color::Green }))
        .ratio(cpu_val / 100.0)
        .label(format!("{:.1}%", cpu_val));
    f.render_widget(gauge, chunks[0]);
    
    // Core Sparklines
    let cores: Vec<u64> = app.core_history.iter()
        .map(|h| h.latest().copied().unwrap_or(0.0) as u64)
        .collect();
        
    if !cores.is_empty() {
        let sparkline = Sparkline::default()
            .block(Block::default().title("Cores"))
            .style(Style::default().fg(Color::Green))
            .data(&cores);
        f.render_widget(sparkline, chunks[1]);
    }
}

fn draw_mem_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title(" Memory & Swap ").borders(Borders::ALL);
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(inner_area);

    let (used, total) = app.system.memory_usage();
    let ratio = if total > 0 { used as f64 / total as f64 } else { 0.0 };
    
    let mem_gauge = Gauge::default()
        .block(Block::default().title("RAM"))
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(ratio)
        .label(format!("{:.1} GB / {:.1} GB", used as f64 / 1024.0 / 1024.0 / 1024.0, total as f64 / 1024.0 / 1024.0 / 1024.0));
    f.render_widget(mem_gauge, chunks[0]);
    
    let (used_swap, total_swap) = app.system.swap_usage();
    let swap_ratio = if total_swap > 0 { used_swap as f64 / total_swap as f64 } else { 0.0 };

    let swap_gauge = Gauge::default()
        .block(Block::default().title("Swap"))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(swap_ratio)
        .label(format!("{:.1} GB", used_swap as f64 / 1024.0 / 1024.0 / 1024.0));
    f.render_widget(swap_gauge, chunks.get(1).copied().unwrap_or(chunks[0]));
}

fn draw_net_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title(" Network & Disk ").borders(Borders::ALL);
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_area);
        
    // Network Sparkline & Total Stats
    let rx_data: Vec<u64> = app.net_rx_history.iter().copied().collect();
    
    let title = format!(
        " Net | RX: {:.1} MB | TX: {:.1} MB ", 
        app.network.total_rx() as f64 / 1024.0 / 1024.0,
        app.network.total_tx() as f64 / 1024.0 / 1024.0
    );
    
    let rx_spark = Sparkline::default()
        .block(Block::default().title(title))
        .style(Style::default().fg(Color::Blue))
        .data(&rx_data);
    f.render_widget(rx_spark, chunks[0]);
    
    // Disk Usage Text
    let disks = app.disk.get_disks();
    let disk_info: Vec<ListItem> = disks.iter().take(4).map(|d| {
        let usage = d.total_space() - d.available_space();
        ListItem::new(format!("{:?}: {:.1}/{:.1} GB", d.mount_point(), usage as f64/1e9, d.total_space() as f64/1e9))
    }).collect();
    
    let disk_list = List::new(disk_info).block(Block::default().title("Disks"));
    f.render_widget(disk_list, chunks[1]);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.logs.iter().rev()
        .map(|msg| ListItem::new(msg.clone()).style(Style::default().fg(Color::Red)))
        .collect();
        
    let list = List::new(items)
        .block(Block::default().title(" System Alerts ").borders(Borders::ALL));
    f.render_widget(list, area);
}

fn draw_history_panel(f: &mut Frame, _app: &App, area: Rect) {
    let p = Paragraph::new("Detailed history charts would go here.\n(Placeholder for expanded feature)")
        .block(Block::default().title(" History ").borders(Borders::ALL));
    f.render_widget(p, area);
}

fn draw_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    let procs = app.process.get_sorted_processes(app.system.inner());
    
    let header_cells = ["PID", "Name", "CPU %", "Mem (MB)", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows = procs.iter().map(|p| {
        let cells = vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}", p.cpu)),
            Cell::from(format!("{:.1}", p.memory as f64 / 1024.0 / 1024.0)),
            Cell::from(p.status.clone()),
        ];
        Row::new(cells).height(1)
    });
    
    let sort_str = match app.process.sort_mode {
        SortBy::Pid => "PID",
        SortBy::Name => "Name",
        SortBy::Cpu => "CPU",
        SortBy::Memory => "Memory",
    };

    let title = format!(" Processes | Sort: {} | Total: {} ", sort_str, procs.len());

    let t = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">>");

    f.render_stateful_widget(t, area, &mut app.process_table_state);
}
