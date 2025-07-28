use crate::app::{App, CurrentMode, CurrentTypingOption};
use crate::utils::{default_text, default_words};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::collections::HashMap;

/// Reads the terminal events.
pub fn handle_events(app: &mut App) -> Result<()> {
    // Only wait for keyboard events for 50ms - otherwise continue the loop iteration
    if event::poll(std::time::Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key), // Handle keyboard input
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {
                app.needs_redraw = true;
            } // Re-render if terminal window resized
            _ => {}
        }
    }
    Ok(())
}

/// Handles keyboard input.
fn on_key_event(app: &mut App, key: KeyEvent) {
    // First boot page input (if toggled takes all input)
    // If Enter key is pressed sets first_boot to false in the config file
    if app.config.first_boot {
        match key.code {
            KeyCode::Enter => {
                app.config.first_boot = false;
                if let Ok(config_dir) = crate::utils::get_config_dir() {
                    crate::utils::save_config(&app.config, &config_dir).unwrap_or_else(|err| {
                        eprintln!("Failed to save config: {}", err);
                    });
                }
                app.needs_clear = true;
                app.needs_redraw = true;
            }
            _ => {}
        }
        return;
    }

    // Help page input (if toggled takes all input)
    if app.show_help {
        match key.code {
            KeyCode::Enter | KeyCode::Char('h') => {
                app.show_help = false;
                app.needs_clear = true;
                app.needs_redraw = true;
            }
            _ => {}
        }
        return; // Stop here
    }

    // Most mistyped page input (if toggled takes all input)
    if app.show_mistyped {
        match key.code {
            KeyCode::Enter | KeyCode::Char('w') => {
                app.show_mistyped = false;
                app.needs_clear = true;
                app.needs_redraw = true;
            }
            _ => {}
        }
        return;
    }

    match app.current_mode {
        // Menu mode input
        CurrentMode::Menu => {
            match key.code {
                // Exit the application
                KeyCode::Char('q') => app.quit(),

                // Reset mistyped characters count
                KeyCode::Char('r') => {
                    app.config.mistyped_chars = HashMap::new();
                    app.notifications.show_clear_mistyped();
                    app.needs_redraw = true;
                }

                // Show most mistyped page
                KeyCode::Char('w') => {
                    app.show_mistyped = true;
                    app.needs_clear = true;
                    app.needs_redraw = true;
                }

                // Toggle counting mistyped characters
                KeyCode::Char('c') => {
                    app.config.save_mistyped = !app.config.save_mistyped;
                    app.notifications.show_mistyped();
                    app.needs_clear = true;
                    app.needs_redraw = true;
                }

                // Toggle displaying notifications
                KeyCode::Char('n') => {
                    app.config.show_notifications = !app.config.show_notifications;
                    app.notifications.show_toggle();
                    app.needs_clear = true;
                    app.needs_redraw = true;
                }

                // Show help page
                KeyCode::Char('h') => {
                    app.show_help = true;
                    app.needs_clear = true;
                    app.needs_redraw = true;
                }

                // Typing option switch (ASCII, Words, Text)
                KeyCode::Char('o') => app.switch_typing_option(),

                // Switch to Typing mode
                KeyCode::Char('i') => {
                    // Check for whether the words/text has anything
                    // to prevent being able to switch to Typing mode
                    // in info page if no words/text file was provided
                    match app.current_typing_option {
                        CurrentTypingOption::Words => {
                            if app.words.len() == 0 {
                                return;
                            }
                        }
                        CurrentTypingOption::Text => {
                            if app.text.len() == 0 {
                                return;
                            }
                        }
                        _ => {}
                    }

                    app.current_mode = CurrentMode::Typing;
                    app.notifications.show_mode();
                    app.needs_redraw = true;
                }

                // If Enter is pressed in the Words/Text typing options,
                // with no words/text file provided - use the default set.
                KeyCode::Enter => {
                    match app.current_typing_option {
                        CurrentTypingOption::Words => {
                            if app.words.is_empty() {
                                // Get the default words set
                                app.words = default_words();

                                // Generate three lines worth of words (characters) and ids.
                                // Keep track of the length of those lines in characters.
                                for _ in 0..3 {
                                    let one_line = app.gen_one_line_of_words();
                                    app.populate_charset_from_line(one_line);
                                }

                                // Remember to use the default word set
                                app.config.use_default_word_set = true;

                                app.needs_redraw = true;
                            }
                        }
                        CurrentTypingOption::Text => {
                            // Only generate the lines if the text file was provided or the default text was chosen
                            if app.text.is_empty() {
                                // Get the default sentences
                                app.text = default_text();

                                // Generate three lines worth of words (characters) and ids.
                                // Keep track of the length of those lines in characters.
                                for _ in 0..3 {
                                    let one_line = app.get_one_line_of_text();

                                    // Count for how many "words" there were on the first three lines
                                    // to keep position on option switch and exit.
                                    // Otherwise would always skip 3 lines down.
                                    let first_text_gen_len: Vec<String> = one_line
                                        .split_whitespace()
                                        .map(String::from)
                                        .collect();
                                    app.first_text_gen_len += first_text_gen_len.len();

                                    app.populate_charset_from_line(one_line);
                                }

                                // Remember to use the default text set
                                app.config.use_default_text_set = true;

                                app.needs_redraw = true;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Typing mode input
        CurrentMode::Typing => {
            match key.code {
                KeyCode::Esc => {
                    // Switch to Menu mode if ESC pressed
                    app.current_mode = CurrentMode::Menu;
                    app.notifications.show_mode();
                    app.needs_redraw = true;
                }
                KeyCode::Char(c) => {
                    // Add to input characters
                    app.input_chars.push_back(c.to_string());
                    app.needs_redraw = true;
                    app.typed = true;
                    app.wpm.on_key_press();
                }
                KeyCode::Backspace => {
                    // Remove from input characters
                    let position = app.input_chars.len();
                    if position > 0 {
                        // If there are no input characters - don't do anything
                        app.input_chars.pop_back();
                        app.ids[position - 1] = 0;
                        app.needs_redraw = true;
                    }
                }
                _ => {}
            }
        }
    }
}
