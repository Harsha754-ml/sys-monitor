use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::Local;

/// Manages historical data for a specific metric using a sliding window.
#[derive(Debug)]
pub struct HistoryBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> HistoryBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.data.iter()
    }
    
    pub fn latest(&self) -> Option<&T> {
        self.data.back()
    }
}

/// Handles session logging to a file.
pub struct SessionLogger {
    file: Option<File>,
}

impl SessionLogger {
    pub fn new(filename: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .ok();
        Self { file }
    }

    pub fn log(&mut self, message: &str) {
        if let Some(file) = &mut self.file {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
        }
    }
}