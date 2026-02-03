use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem},
    Terminal,
};
use sysinfo::System;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut system = System::new_all();

    loop {
        system.refresh_all();

        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                ])
                .split(size);

            // CPU gauge
            let cpu = system.global_cpu_info().cpu_usage() / 100.0;
            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Red))
                .ratio(cpu as f64);
            f.render_widget(cpu_gauge, chunks[0]);

            // RAM gauge
            let mem_used = system.used_memory() as f64;
            let mem_total = system.total_memory() as f64;
            let mem_ratio = mem_used / mem_total;

            let mem_gauge = Gauge::default()
                .block(Block::default().title("Memory Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Blue))
                .ratio(mem_ratio);
            f.render_widget(mem_gauge, chunks[1]);

            // Process list
            let processes: Vec<ListItem> = system
                .processes()
                .iter()
                .take(8)
                .map(|(pid, p)| {
                    ListItem::new(format!(
                        "[{}] {} | CPU: {:.2}%",
                        pid,
                        p.name(),
                        p.cpu_usage()
                    ))
                })
                .collect();

            let process_list = List::new(processes)
                .block(Block::default().title("Top Processes").borders(Borders::ALL));

            f.render_widget(process_list, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
