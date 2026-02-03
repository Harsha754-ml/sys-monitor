use sysinfo::{System, Pid};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Pid,
    Name,
    Cpu,
    Memory,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
    pub status: String,
}

pub struct ProcessManager {
    pub sort_mode: SortBy,
    pub selected_pid: Option<Pid>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sort_mode: SortBy::Cpu,
            selected_pid: None,
        }
    }

    pub fn get_sorted_processes(&self, system: &System) -> Vec<ProcessInfo> {
        let mut procs: Vec<ProcessInfo> = system.processes().iter().map(|(pid, proc)| {
            ProcessInfo {
                pid: *pid,
                name: proc.name().to_string(),
                cpu: proc.cpu_usage(),
                memory: proc.memory(),
                status: format!("{:?}", proc.status()),
            }
        }).collect();

        match self.sort_mode {
            SortBy::Pid => procs.sort_by(|a, b| a.pid.cmp(&b.pid)),
            SortBy::Name => procs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            SortBy::Cpu => procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Ordering::Equal)),
            SortBy::Memory => procs.sort_by(|a, b| b.memory.cmp(&a.memory)),
        }

        procs
    }

    pub fn kill_process(&self, system: &System) -> bool {
        if let Some(pid) = self.selected_pid {
            if let Some(proc) = system.process(pid) {
                return proc.kill();
            }
        }
        false
    }
    
    pub fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortBy::Cpu => SortBy::Memory,
            SortBy::Memory => SortBy::Pid,
            SortBy::Pid => SortBy::Name,
            SortBy::Name => SortBy::Cpu,
        };
    }
}