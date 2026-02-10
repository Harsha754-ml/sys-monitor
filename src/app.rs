use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

use crate::history::{HistoryBuffer, SessionLogger};
use crate::network::NetworkMonitor;
use crate::process::ProcessManager;
use crate::system::SystemMonitor;
use crate::disk::DiskMonitor;
use crate::export::export_history;

#[derive(Debug, PartialEq)]
pub enum ActivePanel {
    Charts,
    Logs,
    History,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub active_panel: ActivePanel,
    pub terminal_size: (u16, u16),
    
    // Modules
    pub system: SystemMonitor,
    pub network: NetworkMonitor,
    pub disk: DiskMonitor,
    pub process: ProcessManager,
    pub logger: SessionLogger,
    
    // UI State
    pub process_table_state: TableState,
    
    // History Data
    pub cpu_history: HistoryBuffer<f64>,
    pub mem_history: HistoryBuffer<f64>,
    pub net_rx_history: HistoryBuffer<u64>,
    pub net_tx_history: HistoryBuffer<u64>,
    
    // Core history: Map of Core Index -> History
    pub core_history: Vec<HistoryBuffer<f32>>,
    
    // Alerts/Logs
    pub logs: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let system = SystemMonitor::new();
        let core_count = system.inner().cpus().len();
        
        let mut core_history = Vec::with_capacity(core_count);
        for _ in 0..core_count {
            core_history.push(HistoryBuffer::new(100));
        }

        Self {
            running: true,
            input_mode: InputMode::Normal,
            active_panel: ActivePanel::Charts,
            terminal_size: (0, 0),
            system,
            network: NetworkMonitor::new(),
            disk: DiskMonitor::new(),
            process: ProcessManager::new(),
            logger: SessionLogger::new("sysmon_session.log"),
            process_table_state: TableState::default(),
            cpu_history: HistoryBuffer::new(100),
            mem_history: HistoryBuffer::new(100),
            net_rx_history: HistoryBuffer::new(100),
            net_tx_history: HistoryBuffer::new(100),
            core_history,
            logs: Vec::new(),
        }
    }

    pub fn on_tick(&mut self) {
        // Refresh Data
        self.system.refresh();
        let net_stats = self.network.refresh();
        self.disk.refresh();

        // Update History
        let cpu = self.system.global_cpu();
        self.cpu_history.push(cpu as f64);
        
        let (used_mem, total_mem) = self.system.memory_usage();
        let mem_pct = if total_mem > 0 { (used_mem as f64 / total_mem as f64) * 100.0 } else { 0.0 };
        self.mem_history.push(mem_pct);
        
        // Cores
        let cores = self.system.cores_cpu();
        for (i, usage) in cores.iter().enumerate() {
            if let Some(hist) = self.core_history.get_mut(i) {
                hist.push(*usage);
            }
        }

        // Network
        self.net_rx_history.push(net_stats.rx_speed);
        self.net_tx_history.push(net_stats.tx_speed);
        
        // Anomaly Detection
        if cpu > 90.0 {
            self.add_log(format!("CRITICAL: High CPU Usage: {:.1}%", cpu));
        }
        if mem_pct > 90.0 {
            self.add_log(format!("CRITICAL: High Memory Usage: {:.1}%", mem_pct));
        }
    }
    
    pub fn on_resize(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);
    }
    
    pub fn add_log(&mut self, msg: String) {
        // Avoid spamming logs
        if self.logs.last() != Some(&msg) {
            self.logger.log(&msg);
            if self.logs.len() > 50 {
                self.logs.remove(0);
            }
            self.logs.push(msg);
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.running = false,
                KeyCode::Char('l') | KeyCode::Char('L') => {
                     self.active_panel = match self.active_panel {
                         ActivePanel::Logs => ActivePanel::Charts,
                         _ => ActivePanel::Logs,
                     };
                },
                KeyCode::Char('h') | KeyCode::Char('H') => {
                     self.active_panel = match self.active_panel {
                         ActivePanel::History => ActivePanel::Charts,
                         _ => ActivePanel::History,
                     };
                },
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    if self.process.kill_process(self.system.inner()) {
                        self.add_log(format!("Killed process PID {:?}", self.process.selected_pid));
                    } else {
                        self.add_log("Failed to kill process".to_string());
                    }
                },
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.process.toggle_sort();
                },
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    match export_history(self) {
                        Ok(fname) => self.add_log(format!("Exported data to {}", fname)),
                        Err(err) => self.add_log(format!("Export failed: {}", err)),
                    }
                },
                KeyCode::Down => self.next_process(),
                KeyCode::Up => self.previous_process(),
                _ => {}
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    let mut current = self.process.filter.clone();
                    current.push(c);
                    self.process.set_filter(current);
                }
                KeyCode::Backspace => {
                    let mut current = self.process.filter.clone();
                    current.pop();
                    self.process.set_filter(current);
                }
                _ => {}
            }
        }
    }
    
    fn next_process(&mut self) {
        let procs_len = self.system.inner().processes().len();
        if procs_len == 0 { return; }
        
        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i >= procs_len - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
        self.update_selected_pid(i);
    }

    fn previous_process(&mut self) {
        let procs_len = self.system.inner().processes().len();
        if procs_len == 0 { return; }

        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i == 0 { procs_len - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
        self.update_selected_pid(i);
    }
    
    fn update_selected_pid(&mut self, index: usize) {
        let sorted = self.process.get_sorted_processes(self.system.inner());
        if let Some(p) = sorted.get(index) {
            self.process.selected_pid = Some(p.pid);
        }
    }
}
