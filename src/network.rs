use sysinfo::Networks;

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub rx_speed: u64, // bytes per tick
    pub tx_speed: u64, // bytes per tick
}

pub struct NetworkMonitor {
    networks: Networks,
    last_rx: u64,
    last_tx: u64,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        let (last_rx, last_tx) = Self::calculate_totals(&networks);
        
        Self {
            networks,
            last_rx,
            last_tx,
        }
    }

    pub fn refresh(&mut self) -> NetworkStats {
        self.networks.refresh();
        let (_current_rx, _current_tx) = Self::calculate_totals(&self.networks);
        
        let mut rx_speed = 0;
        let mut tx_speed = 0;
        
        for (_name, data) in &self.networks {
            rx_speed += data.received();
            tx_speed += data.transmitted();
        }

        self.last_rx += rx_speed;
        self.last_tx += tx_speed;

        NetworkStats {
            rx_speed,
            tx_speed,
        }
    }
    
    pub fn total_rx(&self) -> u64 {
        self.last_rx
    }
    
    pub fn total_tx(&self) -> u64 {
        self.last_tx
    }

    fn calculate_totals(networks: &Networks) -> (u64, u64) {
        let mut rx = 0;
        let mut tx = 0;
        for (_name, data) in networks {
            rx += data.total_received();
            tx += data.total_transmitted();
        }
        (rx, tx)
    }
}