use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal};
use ttypr::{gen_random_ascii_char, read_words_from_file, gen_one_line_of_words};

mod app;
mod ui;
use crate::{
    app::{App, CurrentMode, CurrentTypingMode},
    ui::{render, draw_on_clear},
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

// Run the application's main loop
fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    
    // (For the ASCII option) - Generate initial random charset and all ids set to 0
    // (This for block is here because the default typing option is Ascii)
    for _ in 0..app.line_len*3 {
        app.charset.push_back(gen_random_ascii_char());
        app.ids.push_back(0);
    }

    // (For the Words option) - Read the words from .config/ttypr/words
    app.words = match read_words_from_file() {
        Ok(words) => words,
        Err(_) => { vec![] }
    };
    
    while app.running {
        // Clears the entire area when switching typing modes to draw switched mode ui on top of
        if app.needs_clear { 
            terminal.draw(|frame| draw_on_clear(frame))?;
            app.needs_clear = false;
            app.needs_redraw = true;
        }
        if app.needs_redraw {
            // Which current typing mode the app is currently in?
            match app.current_mode {
                CurrentMode::Menu => {},
                CurrentMode::Typing => {
                    // Which typing option is the app currently in? Run logic for it accordingly
                    match app.current_typing_mode {
                        CurrentTypingMode::Ascii => {
                            if app.typed {
                                // Number of characters the user typed, to compare with the charset
                                let pos = app.input_chars.len() - 1;

                                // If the input character matches the characters in the
                                // charset replace the 0 in ids with 1 (correct), 2 (incorrect)
                                if app.input_chars[pos] == app.charset[pos] {
                                    app.ids[pos] = 1;
                                } else {
                                    app.ids[pos] = 2;
                                }
                            
                                // If reached the end of the second line, remove line_len
                                // (the first line) characters from the character set, the user
                                // inputted characters, and ids. Then push the same amount of
                                // new random characters to charset, and that amount of 0's to ids
                                if app.input_chars.len() == app.line_len*2 {
                                    for _ in 0..app.line_len {
                                        app.charset.pop_front();
                                        app.input_chars.pop_front();
                                        app.ids.pop_front();
                                    
                                        app.charset.push_back(gen_random_ascii_char());
                                        app.ids.push_back(0);
                                    }
                                }
                                app.typed = false;
                            }
                        }
                        CurrentTypingMode::Words => {
                            if app.typed {
                                // Number of characters the user typed, to compare with the charset
                                let pos = app.input_chars.len() - 1;

                                // If the input character matches the characters in the
                                // charset replace the 0 in ids with 1 (correct), 2 (incorrect)
                                if app.input_chars[pos] == app.charset[pos] {
                                    app.ids[pos] = 1;
                                } else {
                                    app.ids[pos] = 2;
                                }

                                // If reached the end of the second line, remove first line amount
                                // of characters (words) from the character set, the user
                                // inputted characters, and ids. Then push new line amount of 
                                // characters (words) to charset, and that amount of 0's to ids
                                if app.input_chars.len() == app.lines_len[0] + app.lines_len[1] {
                                    for _ in 0..app.lines_len[0] {
                                        app.charset.pop_front();
                                        app.input_chars.pop_front();
                                        app.ids.pop_front();
                                    }
                                    let one_line = gen_one_line_of_words(app.line_len, &app.words);
                                    let characters: Vec<char> = one_line.chars().collect();
                                    app.lines_len.push_back(characters.len());
                                    for char in characters {
                                        app.charset.push_back(char.to_string());
                                        app.ids.push_back(0);
                                    }
                                }
                                app.typed = false;
                            }
                        }
                    }
                }
            }
            terminal.draw(|frame| render(frame, app))?; // Draw the ui
            app.needs_redraw = false;
        }
        app.handle_crossterm_events()?; // Read terminal events
    }
    Ok(())
}

// Keyboard input
impl App {
    // Reads the terminal events
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key), // Handle keyboard input
            Event::Mouse(_) => {}
            Event::Resize(_, _) => { self.needs_redraw = true; } // Re-render if terminal window resized
            _ => {}
        }
        Ok(())
    }

    // What happens on key presses
    fn on_key_event(&mut self, key: KeyEvent) {
        // What mode is currently selected Menu or Typing
        match self.current_mode {
            CurrentMode::Menu => {
                match key.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('m') => { 
                        self.needs_clear = true;
                        match self.current_typing_mode {
                            // If switched to Words typing option - clear charset, input_chars
                            // and ids. Afterward - generate new words charset.
                            CurrentTypingMode::Ascii => {
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                

                                // * need similar logic to ascii in the main loop, and similar display logic in ui.rs
                                // generate three lines of words, charaset[_, 100+] & lines_len[_, _, _] long
                                for _ in 0..3 {
                                    let one_line = gen_one_line_of_words(self.line_len, &self.words);
                                    let characters: Vec<char> = one_line.chars().collect();
                                    self.lines_len.push_back(characters.len());
                                    for char in characters {
                                        self.charset.push_back(char.to_string());
                                        self.ids.push_back(0);
                                    }
                                }

                                // Switch the typing option to Ascii
                                self.current_typing_mode = CurrentTypingMode::Words 
                            },
                            CurrentTypingMode::Words => { 
                                // If switched to Words typing option - clear charset, input_chars
                                // and ids. Afterward - generate new words charset.
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                
                                for _ in 0..self.line_len*3 {
                                    self.charset.push_back(gen_random_ascii_char());
                                    self.ids.push_back(0);
                                }
                                
                                // Switch the typing option to Ascii
                                self.current_typing_mode = CurrentTypingMode::Ascii 
                            },
                        }
                    }
                    KeyCode::Char('i') => self.current_mode = CurrentMode::Typing,
                    _ => {}
                }
            },
            // If Typing mode is selected, take actions depending on typing option selected (ASCII, Words)
            CurrentMode::Typing => {
                match self.current_typing_mode {
                    CurrentTypingMode::Ascii => {
                        match key.code {
                            KeyCode::Esc => self.current_mode = CurrentMode::Menu, // Switch to Menu mode if ESC pressed
                            KeyCode::Char(c) => {
                                self.input_chars.push_back(c.to_string()); // Add to input characters
                                self.needs_redraw = true;
                                self.typed = true;
                            }
                            KeyCode::Backspace => {
                                let position = self.input_chars.len();
                                if position > 0 { // If there are no input characters - don't do anything
                                    self.input_chars.pop_back();
                                    self.ids[position-1] = 0;
                                    self.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    },
                    CurrentTypingMode::Words => {
                        match key.code {
                            KeyCode::Esc => self.current_mode = CurrentMode::Menu, // Switch to Menu mode if ESC pressed
                            KeyCode::Char(c) => {
                                self.input_chars.push_back(c.to_string()); // Add to input characters
                                self.needs_redraw = true;
                                self.typed = true;
                            }
                            KeyCode::Backspace => {
                                let position = self.input_chars.len();
                                if position > 0 { // If there are no input characters - don't do anything
                                    self.input_chars.pop_back();
                                    self.ids[position-1] = 0;
                                    self.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    },
                }
            }
        }
    }
}