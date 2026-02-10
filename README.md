# Sysmon Ultimate

A high-performance, real-time system monitor with a Terminal User Interface (TUI), built in Rust.

![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## Features

- **Real-time Monitoring**: Track CPU, Memory, Disk I/O, and Network traffic.
- **Hardware Sensors**: Monitor CPU and system temperatures.
- **Advanced Process Manager**:
    - Sort by CPU, Memory, PID, or Name.
    - **Live Filtering**: Press `/` to search and filter processes instantly.
    - **Process Control**: Select and kill processes directly from the UI.
- **Data Visualization**: Sparklines for individual CPU cores and network traffic.
- **Data Export**: Export your session's history data to CSV for external analysis.
- **Alert System**: Automatic logging of critical system events (e.g., high resource usage).

## Controls

| Key | Action |
|-----|--------|
| `Q` | Quit application |
| `S` | Toggle process sort mode (CPU, Memory, PID, Name) |
| `/` | Enter search mode to filter processes |
| `K` | Kill the selected process |
| `E` | Export current history data to CSV |
| `L` | Toggle the System Alerts Log panel |
| `H` | Toggle the History panel |
| `↑`/`↓` | Navigate the process table |
| `ESC` | Exit search mode / Return to Normal mode |

## Installation

Ensure you have Rust and Cargo installed, then:

```bash
cargo build --release
./target/release/sysmon
```

## Dependencies

- `ratatui`: Terminal UI rendering.
- `sysinfo`: System data collection.
- `crossterm`: Terminal event handling.
- `chrono`: Timestamping for logs and exports.
- `csv`: Data export functionality.
