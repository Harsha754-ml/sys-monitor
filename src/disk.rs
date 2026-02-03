use sysinfo::Disks;

pub struct DiskMonitor {
    disks: Disks,
}

impl DiskMonitor {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.disks.refresh();
    }

    pub fn get_disks(&self) -> &Disks {
        &self.disks
    }
}