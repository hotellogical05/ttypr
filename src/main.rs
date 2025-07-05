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

        // Timer for notifications diplay
        app.on_tick();

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
                            if app.words.len() == 0 {}
                            else {
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

                                        // Remove the length of the first line of words from the front, 
                                        // and push the new one to the back.
                                        app.lines_len.pop_front();
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

    // What happens on key presses
    fn on_key_event(&mut self, key: KeyEvent) {
        // What mode is currently selected Menu or Typing
        match self.current_mode {
            CurrentMode::Menu => {
                match key.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('m') => { 
                        self.needs_clear = true;
                        self.show_option_notification();
                        match self.current_typing_mode {
                            // If switched to Words typing option - clear charset, input_chars
                            // and ids. Afterward - generate new words charset.
                            CurrentTypingMode::Ascii => {
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                
                                if self.words.len() == 0 {}
                                else {
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
                                self.lines_len.clear();
                                
                                for _ in 0..self.line_len*3 {
                                    self.charset.push_back(gen_random_ascii_char());
                                    self.ids.push_back(0);
                                }
                                
                                // Switch the typing option to Ascii
                                self.current_typing_mode = CurrentTypingMode::Ascii 
                            },
                        }
                    }
                    KeyCode::Char('i') => { 
                        self.current_mode = CurrentMode::Typing;
                        self.show_mode_notification();
                        self.needs_redraw = true;
                    },
                    // If Enter pressed in the Words typing option, with no words file provided - create the default one.
                    KeyCode::Enter => { 
                        match self.current_typing_mode {
                            CurrentTypingMode::Words => {
                                if self.words.len() == 0 {
                                    let default_words = vec!["the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now", "look", "only", "come", "over", "think", "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any", "these", "give", "day", "most", "us", "thing", "man", "find", "part", "eye", "place", "week", "case", "point", "government", "company", "number", "group", "problem", "fact", "leave", "while", "mean", "keep", "student", "great", "seem", "same", "tell", "begin", "help", "talk", "where", "turn", "start", "might", "show", "hear", "play", "run", "move", "live", "believe", "hold", "bring", "happen", "must", "write", "provide", "sit", "stand", "lose", "pay", "meet", "include", "continue", "set", "learn", "change", "lead", "understand", "watch", "follow", "stop", "create", "speak", "read", "allow", "add", "spend", "grow", "open", "walk", "win", "offer", "remember", "love", "consider", "appear", "buy", "wait", "serve", "die", "send", "expect", "build", "stay", "fall", "cut", "reach", "kill", "remain", "suggest", "raise", "pass", "sell", "require", "report", "decide", "pull", "return", "explain", "hope", "develop", "carry", "break", "receive", "agree", "support", "hit", "produce", "eat", "cover", "catch", "draw", "choose", "cause", "listen", "maybe", "until", "without", "probably", "around", "small", "green", "special", "difficult", "available", "likely", "short", "single", "medical", "current", "wrong", "private", "past", "foreign", "fine", "common", "poor", "natural", "significant", "similar", "hot", "dead", "central", "happy", "serious", "ready", "simple", "left", "physical", "general", "environmental", "financial", "blue", "democratic", "dark", "various", "entire", "close", "legal", "religious", "cold", "final", "main", "huge", "popular", "traditional", "cultural", "choice", "high", "big", "large", "particular", "tiny", "enormous"];
                                    let default_words: Vec<String> = default_words
                                        .iter()
                                        .map(|w| w.to_string())
                                        .collect();
                                    self.words = default_words;
                                    self.needs_redraw = true;
                                }
                                else {}
                            }
                            _ => {}
                        }
                        // Generate three lines of words, charset[_, 100+] and lines_len[_, _, _]
                        for _ in 0..3 {
                            let one_line = gen_one_line_of_words(self.line_len, &self.words);
                            let characters: Vec<char> = one_line.chars().collect();
                            self.lines_len.push_back(characters.len());
                            for char in characters {
                                self.charset.push_back(char.to_string());
                                self.ids.push_back(0);
                            }
                        }
                    }
                    _ => {}
                }
            },
            // If Typing mode is selected, take actions depending on typing option selected (ASCII, Words)
            CurrentMode::Typing => {
                match self.current_typing_mode {
                    CurrentTypingMode::Ascii => {
                        match key.code {
                            KeyCode::Esc => { // Switch to Menu mode if ESC pressed
                                self.current_mode = CurrentMode::Menu; 
                                self.show_mode_notification();
                                self.needs_redraw = true;
                            },
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
                            KeyCode::Esc => { // Switch to Menu mode if ESC pressed
                                self.current_mode = CurrentMode::Menu;
                                self.show_mode_notification();
                                self.needs_redraw = true;
                            },
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