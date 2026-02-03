use std::{collections::VecDeque, io, time::{Duration, Instant}};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Paragraph},
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
    let mut cpu_hist: VecDeque<(f64, f64)> = VecDeque::with_capacity(120);
    let mut mem_hist: VecDeque<(f64, f64)> = VecDeque::with_capacity(120);
    let start = Instant::now();
    let mut tick = 0.0;

    loop {
        system.refresh_all();
        tick += 1.0;

        let cpu = system.global_cpu_info().cpu_usage() as f64;
        let mem = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

        if cpu_hist.len() == 120 { cpu_hist.pop_front(); mem_hist.pop_front(); }
        cpu_hist.push_back((tick, cpu));
        mem_hist.push_back((tick, mem));

        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35),
                    Constraint::Percentage(25),
                    Constraint::Percentage(40),
                ])
                .split(f.size());

            // Top charts
            let charts = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[0]);

            let cpu_data: Vec<(f64, f64)> = cpu_hist.iter().copied().collect();
            let mem_data: Vec<(f64, f64)> = mem_hist.iter().copied().collect();

            let cpu_chart = Chart::new(vec![Dataset::default()
                .name("CPU")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red))
                .data(&cpu_data)])
                .block(Block::default().title("CPU History").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([tick-120.0, tick]))
                .y_axis(Axis::default().bounds([0.0,100.0]));

            let mem_chart = Chart::new(vec![Dataset::default()
                .name("RAM")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&mem_data)])
                .block(Block::default().title("Memory History").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([tick-120.0, tick]))
                .y_axis(Axis::default().bounds([0.0,100.0]));

            f.render_widget(cpu_chart, charts[0]);
            f.render_widget(mem_chart, charts[1]);

            // Middle system info
            let info = Paragraph::new(format!(
                "RAM: {} / {} MB\nProcesses: {}\nUptime: {} sec",
                system.used_memory()/1024,
                system.total_memory()/1024,
                system.processes().len(),
                start.elapsed().as_secs()
            ))
            .block(Block::default().title("System Info").borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));

            f.render_widget(info, main_chunks[1]);

            // Bottom process list
            let mut procs: Vec<_> = system.processes().iter().collect();
            procs.sort_by(|a,b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());

            let items: Vec<ListItem> = procs.iter().take(12).map(|(pid,p)| {
                ListItem::new(format!(
                    "[{}] {} | CPU: {:.1}% | RAM: {} MB",
                    pid, p.name(), p.cpu_usage(), p.memory()/1024
                ))
            }).collect();

            let list = List::new(items)
                .block(Block::default().title("Top Processes").borders(Borders::ALL));

            f.render_widget(list, main_chunks[2]);
        })?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(k) = event::read()? {
                if k.code == KeyCode::Char('q') { break; }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
