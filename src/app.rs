use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;
use sysinfo::{Networks, System};

/// Application state.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// System object to gather stats.
    pub system: System,
    /// Networks object to gather stats.
    pub networks: Networks,
    /// History of CPU usage (global).
    pub cpu_history: VecDeque<f64>,
    /// History of Memory usage.
    pub mem_history: VecDeque<f64>,
    /// History of Swap usage.
    pub swap_history: VecDeque<f64>,
    /// History of Network IO (RX, TX).
    pub net_history: VecDeque<(u64, u64)>,
    /// State for the process table.
    pub process_table_state: TableState,
    /// Current sort mode for processes.
    pub sort_mode: SortMode,
    /// Sliding window size for graphs.
    pub window_size: usize,
    /// Cache for sorted processes to avoid sorting every render frame if not needed.
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Cpu,
    Memory,
    Pid,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();

        Self {
            running: true,
            system,
            networks,
            cpu_history: VecDeque::from(vec![0.0; 100]),
            mem_history: VecDeque::from(vec![0.0; 100]),
            swap_history: VecDeque::from(vec![0.0; 100]),
            net_history: VecDeque::from(vec![(0, 0); 100]),
            process_table_state: TableState::default(),
            sort_mode: SortMode::Cpu,
            window_size: 100,
            processes: Vec::new(),
        }
    }

    /// Handles the tick event (updates system stats).
    pub fn on_tick(&mut self) {
        // Refresh system stats
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.networks.refresh();
        self.system.refresh_processes();

        // Update CPU History
        let cpu_usage = self.system.global_cpu_info().cpu_usage() as f64;
        Self::push_history(&mut self.cpu_history, cpu_usage, self.window_size);

        // Update Memory History
        let total_mem = self.system.total_memory() as f64;
        let used_mem = self.system.used_memory() as f64;
        let mem_usage = if total_mem > 0.0 { (used_mem / total_mem) * 100.0 } else { 0.0 };
        Self::push_history(&mut self.mem_history, mem_usage, self.window_size);

        // Update Swap History
        let total_swap = self.system.total_swap() as f64;
        let used_swap = self.system.used_swap() as f64;
        let swap_usage = if total_swap > 0.0 { (used_swap / total_swap) * 100.0 } else { 0.0 };
        Self::push_history(&mut self.swap_history, swap_usage, self.window_size);

        // Update Network History
        let mut total_rx = 0;
        let mut total_tx = 0;
        for (_name, data) in &self.networks {
            total_rx += data.received();
            total_tx += data.transmitted();
        }
        if self.net_history.len() >= self.window_size {
            self.net_history.pop_front();
        }
        self.net_history.push_back((total_rx, total_tx));

        // Update Process List
        self.update_process_list();
    }

    fn push_history(history: &mut VecDeque<f64>, value: f64, window_size: usize) {
        if history.len() >= window_size {
            history.pop_front();
        }
        history.push_back(value);
    }

    fn update_process_list(&mut self) {
        let mut procs: Vec<ProcessInfo> = self.system.processes().iter().map(|(pid, proc)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: proc.name().to_string(),
                cpu: proc.cpu_usage(),
                memory: proc.memory(),
            }
        }).collect();

        match self.sort_mode {
            SortMode::Cpu => procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)),
            SortMode::Memory => procs.sort_by(|a, b| b.memory.cmp(&a.memory)),
            SortMode::Pid => procs.sort_by(|a, b| a.pid.cmp(&b.pid)),
        }

        self.processes = procs;
    }

    /// Handles key events.
    pub fn on_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.running = false;
            }
            (KeyCode::Char('s'), _) => {
                self.sort_mode = match self.sort_mode {
                    SortMode::Cpu => SortMode::Memory,
                    SortMode::Memory => SortMode::Pid,
                    SortMode::Pid => SortMode::Cpu,
                };
                self.update_process_list();
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.next_row();
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                self.previous_row();
            }
            _ => {}
        }
    }

    fn next_row(&mut self) {
        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i >= self.processes.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
    }

    fn previous_row(&mut self) {
        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.processes.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
    }
}
