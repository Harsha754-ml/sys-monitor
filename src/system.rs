use sysinfo::System;

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.sys.refresh_processes();
    }
    
    pub fn global_cpu(&self) -> f32 {
        self.sys.global_cpu_info().cpu_usage()
    }
    
    pub fn cores_cpu(&self) -> Vec<f32> {
        self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }

    pub fn memory_usage(&self) -> (u64, u64) {
        (self.sys.used_memory(), self.sys.total_memory())
    }

    pub fn swap_usage(&self) -> (u64, u64) {
        (self.sys.used_swap(), self.sys.total_swap())
    }
    
    // Wrappers for static methods to allow mocking or consistent access via struct if needed,
    // but here we just call static methods. To make them "used", we must call them in UI.
    pub fn uptime() -> u64 {
        System::uptime()
    }
    
    pub fn name() -> String {
        System::name().unwrap_or_default()
    }
    
    pub fn os_version() -> String {
        System::os_version().unwrap_or_default()
    }
    
    pub fn host_name() -> String {
        System::host_name().unwrap_or_default()
    }
    
    pub fn inner(&self) -> &System {
        &self.sys
    }
}