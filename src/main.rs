use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal};
use std::{collections::HashMap};

mod app;
mod ui;
mod utils;
use crate::{
    app::{App, CurrentMode, CurrentTypingOption},
    ui::{draw_on_clear, render},
    utils::{default_text, default_words, save_config},
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

impl App {
    // Reads the terminal events
    fn handle_crossterm_events(&mut self) -> Result<()> {
        // Only wait for keyboard events for 50ms - otherwise continue the loop iteration
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key), // Handle keyboard input
                Event::Mouse(_) => {}
                Event::Resize(_, _) => { self.needs_redraw = true; } // Re-render if terminal window resized
                _ => {}
            }
        }
        Ok(())
    }

    // Keyboard input
    fn on_key_event(&mut self, key: KeyEvent) {
        // First boot page input (if toggled takes all input)
        // If Enter key is pressed sets first_boot to false in the config file
        if self.config.first_boot {
            match key.code {
                KeyCode::Enter => {
                    self.config.first_boot = false;
                    save_config(&self.config).unwrap_or_else(|err| {
                        eprintln!("Failed to save config: {}", err);
                    });
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
                _ => {}
            }
            return;
        }

        // Help page input (if toggled takes all input)
        if self.show_help {
            match key.code {
                KeyCode::Enter | KeyCode::Char('h') => {
                    self.show_help = false;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
                _ => {}
            }
            return; // Stop here
        }

        // Most mistyped page input (if toggled takes all input)
        if self.show_mistyped {
            match key.code {
                KeyCode::Enter | KeyCode::Char('w') => {
                    self.show_mistyped = false;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
                _ => {}
            }
            return;
        }

        match self.current_mode {
            // Menu mode input
            CurrentMode::Menu => {
                match key.code {
                    // Exit the application
                    KeyCode::Char('q') => self.quit(),

                    // Reset mistyped characters count
                    KeyCode::Char('r') => {
                        self.config.mistyped_chars = HashMap::new();
                        self.notifications.show_clear_mistyped();
                        self.needs_redraw = true;
                    }

                    // Show most mistyped page
                    KeyCode::Char('w') => {
                        self.show_mistyped = true;
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }

                    // Toggle counting mistyped characters
                    KeyCode::Char('c') => {
                        self.config.save_mistyped = !self.config.save_mistyped;
                        self.notifications.show_mistyped();
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }
                    
                    // Toggle displaying notifications
                    KeyCode::Char('n') => {
                        self.config.show_notifications = !self.config.show_notifications;
                        self.notifications.show_toggle();
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }

                    // Show help page
                    KeyCode::Char('h') => {
                        self.show_help = true; 
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }

                    // Typing option switch (ASCII, Words, Text)
                    KeyCode::Char('o') => self.switch_typing_option(),

                    // Switch to Typing mode
                    KeyCode::Char('i') => { 
                        // Check for whether the words/text has anything
                        // to prevent being able to switch to Typing mode
                        // in info page if no words/text file was provided
                        match self.current_typing_option {
                            CurrentTypingOption::Words => {
                                if self.words.len() == 0 {
                                    return
                                }
                            }
                            CurrentTypingOption::Text => {
                                if self.text.len() == 0 {
                                    return
                                }
                            }
                            _ => {}
                        }

                        self.current_mode = CurrentMode::Typing;
                        self.notifications.show_mode();
                        self.needs_redraw = true;
                    },

                    // If Enter is pressed in the Words/Text typing options, 
                    // with no words/text file provided - use the default set.
                    KeyCode::Enter => { 
                        match self.current_typing_option {
                            CurrentTypingOption::Words => {
                                if self.words.is_empty() {
                                    // Get the default words set
                                    self.words = default_words();

                                    // Generate three lines worth of words (characters) and ids.
                                    // Keep track of the length of those lines in characters.
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_words();
                                        self.populate_charset_from_line(one_line);
                                    }

                                    // Remember to use the default word set
                                    self.config.use_default_word_set = true;

                                    self.needs_redraw = true;
                                }
                            }
                            CurrentTypingOption::Text => {
                                // Only generate the lines if the text file was provided or the default text was chosen
                                if self.text.is_empty() {
                                    // Get the default sentences
                                    self.text = default_text();

                                    // Generate three lines worth of words (characters) and ids.
                                    // Keep track of the length of those lines in characters.
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_text();

                                        // Count for how many "words" there were on the first three lines
                                        // to keep position on option switch and exit.
                                        // Otherwise would always skip 3 lines down.
                                        let first_text_gen_len: Vec<String> = one_line.split_whitespace().map(String::from).collect();
                                        self.first_text_gen_len += first_text_gen_len.len();
                                    
                                        self.populate_charset_from_line(one_line);
                                    }

                                    // Remember to use the default text set
                                    self.config.use_default_text_set = true;

                                    self.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            },

            // Typing mode input
            CurrentMode::Typing => {
                match key.code {
                    KeyCode::Esc => { // Switch to Menu mode if ESC pressed
                        self.current_mode = CurrentMode::Menu;
                        self.notifications.show_mode();
                        self.needs_redraw = true;
                    },
                    KeyCode::Char(c) => { // Add to input characters
                        self.input_chars.push_back(c.to_string());
                        self.needs_redraw = true;
                        self.typed = true;
                    }
                    KeyCode::Backspace => { // Remove from input characters
                        let position = self.input_chars.len();
                        if position > 0 { // If there are no input characters - don't do anything
                            self.input_chars.pop_back();
                            self.ids[position-1] = 0;
                            self.needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Switches to the next typing option and generates the text.
    ///
    /// This function cycles through the available typing options (ASCII, Words, Text)
    /// and prepares the application state for the new option. It clears the
    /// existing content in the buffers, generates new content, and signals to update the UI.
    fn switch_typing_option(&mut self) {
        self.needs_clear = true;
        self.notifications.show_option();
        self.clear_typing_buffers();

        // Switches current typing option
        match self.current_typing_option {
            // If ASCII - switch to Words
            CurrentTypingOption::Ascii => {
                self.current_typing_option = CurrentTypingOption::Words;

                // Only generate the lines if the words file was provided or the default set was chosen
                if !self.words.is_empty() {
                    // Generate three lines of words
                    for _ in 0..3 {
                        let one_line = self.gen_one_line_of_words();
                        self.populate_charset_from_line(one_line);
                    }
                }
            }
            // If Words - switch to Text
            CurrentTypingOption::Words => {
                self.current_typing_option = CurrentTypingOption::Text;

                // Only generate the lines if the text file was provided or the default text was chosen
                if !self.text.is_empty() {
                    for _ in 0..3 {
                        let one_line = self.gen_one_line_of_text();
                        // Count for how many "words" there were on the first three lines
                        // to keep position on option switch and exit.
                        // Otherwise would always skip 3 lines down.
                        let first_text_gen_len: Vec<String> = one_line.split_whitespace().map(String::from).collect();
                        self.first_text_gen_len += first_text_gen_len.len();

                        self.populate_charset_from_line(one_line);
                    }
                }
            }
            // If Text - switch to ASCII
            CurrentTypingOption::Text => {
                // Subtract how many "words" there were on the first three lines
                if self.config.skip_len >= self.first_text_gen_len {
                    self.config.skip_len -= self.first_text_gen_len;
                } else {
                    self.config.skip_len = 0;
                }
                self.first_text_gen_len = 0;

                self.current_typing_option = CurrentTypingOption::Ascii;

                // Generate three lines worth of characters and ids
                for _ in 0..3 {
                    let one_line = self.gen_one_line_of_ascii();
                    self.populate_charset_from_line(one_line);
                }
            }
        }
    }

    /// Populates the character set and related fields from a single line of text.
    ///
    /// This helper function takes a string, splits it into characters, and updates
    /// the `charset`, `ids`, and `lines_len` fields of the `App` state. This is
    /// used to prepare the text that the user will be prompted to type.
    fn populate_charset_from_line(&mut self, one_line: String) {
        // Push a line of characters and ids
        let characters: Vec<char> = one_line.chars().collect();
        self.lines_len.push_back(characters.len());
        for char in characters {
            self.charset.push_back(char.to_string());
            self.ids.push_back(0);
        }
    }
}