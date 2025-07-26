use color_eyre::Result;
use ratatui::DefaultTerminal;

mod app;
mod ui;
mod utils;
use crate::{
    app::App,
    ui::{draw_on_clear, render},
};


fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = run(terminal, &mut app);

    app.on_exit();

    // Restore the terminal and return the result from run()
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    app.setup()?;

    // Main application loop
    while app.running {
        // Timer for displaying notifications
        app.on_tick();

        // If the user typed
        if app.typed {
            app.update_id_field();
            app.update_lines();
            app.typed = false;
        }

        // Clear the entire area
        if app.needs_clear { 
            terminal.draw(|frame| draw_on_clear(frame))?;
            app.needs_clear = false;
            app.needs_redraw = true;
        }

        // Draw/Redraw the ui
        if app.needs_redraw {
            terminal.draw(|frame| render(frame, app))?;
            app.needs_redraw = false;
        }

        // Read terminal events
        app.handle_crossterm_events()?;
    }

    Ok(())
}