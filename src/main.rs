use std::{collections::VecDeque, io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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
    let mut cpu_history: VecDeque<(f64, f64)> = VecDeque::with_capacity(60);
    let mut mem_history: VecDeque<(f64, f64)> = VecDeque::with_capacity(60);
    let mut tick: f64 = 0.0;

    loop {
        system.refresh_all();
        tick += 1.0;

        let cpu = system.global_cpu_info().cpu_usage() as f64;
        let mem = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

        if cpu_history.len() == 60 {
            cpu_history.pop_front();
            mem_history.pop_front();
        }
        cpu_history.push_back((tick, cpu));
        mem_history.push_back((tick, mem));

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ])
                .split(f.size());

            // CPU Chart
            let cpu_data: Vec<(f64, f64)> = cpu_history.iter().copied().collect();
            let datasets = vec![Dataset::default()
                .name("CPU %")
                .marker(ratatui::symbols::Marker::Dot)
                .style(Style::default().fg(Color::Red))
                .data(&cpu_data)];

            let chart = Chart::new(datasets)
                .block(Block::default().title("CPU History").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([tick - 60.0, tick]))
                .y_axis(Axis::default().bounds([0.0, 100.0]));

            f.render_widget(chart, chunks[0]);

            // Memory chart
            let mem_data: Vec<(f64, f64)> = mem_history.iter().copied().collect();
            let mem_dataset = vec![Dataset::default()
                .name("Memory %")
                .marker(ratatui::symbols::Marker::Dot)
                .style(Style::default().fg(Color::Blue))
                .data(&mem_data)];

            let mem_chart = Chart::new(mem_dataset)
                .block(Block::default().title("Memory Usage").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([tick - 60.0, tick]))
                .y_axis(Axis::default().bounds([0.0, 100.0]));

            f.render_widget(mem_chart, chunks[1]);

            // Process list sorted by CPU
            let mut processes: Vec<_> = system.processes().iter().collect();
            processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());

            let items: Vec<ListItem> = processes
                .iter()
                .take(8)
                .map(|(pid, p)| {
                    ListItem::new(format!(
                        "[{}] {} | CPU: {:.1}% | RAM: {} MB",
                        pid,
                        p.name(),
                        p.cpu_usage(),
                        p.memory() / 1024
                    ))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Top Processes").borders(Borders::ALL));

            f.render_widget(list, chunks[2]);
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
