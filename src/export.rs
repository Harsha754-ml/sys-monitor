use anyhow::Result;
use chrono::Local;
use std::fs::File;
use std::io::Write;
use crate::app::App;

pub fn export_history(app: &App) -> Result<String> {
    let filename = format!("sysmon_export_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
    let mut file = File::create(&filename)?;
    
    writeln!(file, "Index,CPU_Usage,Memory_Usage,Net_RX,Net_TX")?;
    
    // All history buffers should be synchronized by the tick loop
    let len = app.cpu_history.iter().len();
    
    for i in 0..len {
        let cpu = app.cpu_history.iter().nth(i).copied().unwrap_or(0.0);
        let mem = app.mem_history.iter().nth(i).copied().unwrap_or(0.0);
        let rx = app.net_rx_history.iter().nth(i).copied().unwrap_or(0);
        let tx = app.net_tx_history.iter().nth(i).copied().unwrap_or(0);
        
        writeln!(file, "{},{:.2},{:.2},{},{}", i, cpu, mem, rx, tx)?;
    }
    
    Ok(filename)
}
