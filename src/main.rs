mod app;
mod disk;
mod event;
mod history;
mod network;
mod process;
mod system;
mod tui;
mod ui;
mod export;

use anyhow::Result;
use app::App;
use event::{Event, EventHandler};
use tui::Tui;

fn main() -> Result<()> {
    // Setup terminal
    let mut terminal = tui::init()?;

    // Create app state
    let mut app = App::new();

    // Create event handler (tick rate 250ms)
    let events = EventHandler::new(250);

    // Run the main loop
    let res = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    tui::restore()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Tui, app: &mut App, events: &EventHandler) -> Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        match events.next()? {
            Event::Tick => app.on_tick(),
            Event::Key(key) => app.on_key(key),
            Event::Resize(w, h) => app.on_resize(w, h),
        }
    }
    Ok(())
}