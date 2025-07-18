use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal};
use std::{collections::HashMap, time::Instant};
use ttypr::{gen_random_ascii_char, load_config, read_text_from_file, read_words_from_file, save_config, Config};

mod app;
mod ui;
use crate::{
    app::{App, CurrentMode, CurrentTypingOption},
    ui::{draw_on_clear, render},
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = run(terminal, &mut app);

    // (If exited the application while being the Text option)
    // Subtract how many "words" there were on the first three lines 
    match app.current_typing_mode {
        CurrentTypingOption::Text => {
            if app.config.as_ref().unwrap().skip_len >= app.first_text_gen_len {
                app.config.as_mut().unwrap().skip_len -= app.first_text_gen_len;
            } else {
                app.config.as_mut().unwrap().skip_len = 0;
            }
        }
        _ => {}
    }

    // Save config (for mistyped characters) before exiting
    if let Some(config) = &app.config {
        save_config(config).unwrap_or_else(|err| {
            eprintln!("Failed to save config: {}", err);
        });
    }

    // Restore the terminal and return the result from run()
    ratatui::restore();
    result
}

// Run the application's main loop
fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    // Load config file or create it
    app.config = Some(load_config().unwrap_or_else(|_err| {
        Config::default()
    }));

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
    
    // (For the Text option) - Read the text from .config/ttypr/text
    app.text = match read_text_from_file() {
        Ok(text) => text,
        Err(_) => { vec![] }
    };

    // Keep in first boot screen until Enter is pressed
    while app.config.as_ref().unwrap().first_boot {
        terminal.draw(|frame| render(frame, app))?; // Draw the ui
        app.handle_crossterm_events()?; // Read terminal events
    }

    // Main application loop
    while app.running {
        // Timer for displaying notifications
        app.on_tick();

        // If the user typed
        if app.typed {
            match app.current_typing_mode {
                CurrentTypingOption::Ascii => {
                    app.update_id_field();
                    app.update_lines(); // Only does anything if reached the end of the second line
                    app.typed = false;
                }
                CurrentTypingOption::Words => {
                    if app.words.len() == 0 {}
                    else {
                        app.update_id_field();
                        app.update_lines();
                        app.typed = false;
                    }
                }
                CurrentTypingOption::Text => {
                    if app.text.len() == 0 {}
                    else {
                        app.update_id_field();
                        app.update_lines();
                        app.typed = false;
                    }
                }
            }
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
        if self.config.as_ref().unwrap().first_boot {
            match key.code {
                KeyCode::Enter => {
                    self.config.as_mut().unwrap().first_boot = false;
                    if let Some(config) = &self.config {
                        save_config(config).unwrap_or_else(|err| {
                            eprintln!("Failed to save config: {}", err);
                        });
                    }
                }
                _ => {}
            }
            return;
        }

        // Help page input (if toggled takes all input)
        if self.show_help {
            match key.code {
                KeyCode::Enter => {
                    self.show_help = false;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
                KeyCode::Char('h') => {
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
                KeyCode::Enter => {
                    self.show_mistyped = false;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
                KeyCode::Char('w') => {
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

                    // Reset mistyped charactes count
                    KeyCode::Char('r') => {
                        self.config.as_mut().unwrap().mistyped_chars = HashMap::new();
                        self.show_clear_mistyped_notification = true;
                        self.needs_redraw = true;
                        self.notification_time_count = Some(Instant::now());
                    }

                    // Show most mistyped page
                    KeyCode::Char('w') => {
                        self.show_mistyped = true;
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }

                    // Toggle counting mistyped characters
                    KeyCode::Char('c') => {
                        if self.config.as_ref().unwrap().save_mistyped {
                            self.config.as_mut().unwrap().save_mistyped = false;
                        } else {
                            self.config.as_mut().unwrap().save_mistyped = true;
                        }
                        self.show_mistyped_notification = true;
                        self.needs_clear = true;
                        self.needs_redraw = true;
                        self.notification_time_count = Some(Instant::now());
                    }
                    
                    // Toggle displaying notifications
                    KeyCode::Char('n') => {
                        if self.config.as_ref().unwrap().show_notifications {
                            self.config.as_mut().unwrap().show_notifications = false;
                        } else {
                            self.config.as_mut().unwrap().show_notifications = true;
                        }
                        self.show_notification_toggle = true;
                        self.needs_clear = true;
                        self.needs_redraw = true;
                        self.notification_time_count = Some(Instant::now());
                    }

                    // Show help page
                    KeyCode::Char('h') => {
                        self.show_help = true; 
                        self.needs_clear = true;
                        self.needs_redraw = true;
                    }

                    // Typing option switch (ASCII, Words, Text)
                    KeyCode::Char('o') => {

                        // Option switch notification
                        self.needs_clear = true;
                        self.show_option_notification = true;
                        self.notification_time_count = Some(Instant::now());

                        // Switches current typing option
                        match self.current_typing_mode {
                            
                            // If ASCII - switch to Words
                            CurrentTypingOption::Ascii => {
                                // Clear charset, input_chars and ids.
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                
                                // Only generate the lines if the words file was provided or the default set was chosen
                                if self.words.len() == 0 {}
                                else {
                                    // Generate three lines of words (charset)
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_words();

                                        // Push three lines worth of characters (from words) and ids
                                        let characters: Vec<char> = one_line.chars().collect();
                                        self.lines_len.push_back(characters.len());
                                        for char in characters {
                                            self.charset.push_back(char.to_string());
                                            self.ids.push_back(0);
                                        }
                                    }
                                }

                                // Switch the typing option to Words
                                self.current_typing_mode = CurrentTypingOption::Words 
                            },
                            
                            // If Words - switch to Text
                            CurrentTypingOption::Words => { 
                                // Clear charset, input_chars, ids and length of lines
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                self.lines_len.clear();

                                // Only generate the lines if the text file was provided or the default text was chosen
                                if self.text.len() == 0 {}
                                else {
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_text();

                                        // Count for how many "words" there were on the first three lines
                                        // to keep position on option switch and exit.
                                        // Otherwise would always skip 3 lines down.
                                        let first_text_gen_len: Vec<String> = one_line.split_whitespace().map(String::from).collect();
                                        self.first_text_gen_len += first_text_gen_len.len();

                                        // Push a line of characters (from text) and ids
                                        let characters: Vec<char> = one_line.chars().collect();
                                        self.lines_len.push_back(characters.len());
                                        for char in characters {
                                            self.charset.push_back(char.to_string());
                                            self.ids.push_back(0);
                                        }
                                    }
                                }

                                // Switch the typing option to Text
                                self.current_typing_mode = CurrentTypingOption::Text
                            },

                            // If Text - switch to ASCII
                            CurrentTypingOption::Text => {
                                // Clear charset, input_chars, ids and length of lines
                                self.charset.clear();
                                self.input_chars.clear();
                                self.ids.clear();
                                self.lines_len.clear();

                                // Subtract how many "words" there were on the first three lines
                                if self.config.as_ref().unwrap().skip_len >= self.first_text_gen_len {
                                    self.config.as_mut().unwrap().skip_len -= self.first_text_gen_len;
                                } else {
                                    self.config.as_mut().unwrap().skip_len = 0;
                                }
                                self.first_text_gen_len = 0;

                                // Generate three lines worth of characters and ids
                                for _ in 0..self.line_len*3 {
                                    self.charset.push_back(gen_random_ascii_char());
                                    self.ids.push_back(0);
                                }
                                
                                // Switch the typing option to Ascii
                                self.current_typing_mode = CurrentTypingOption::Ascii 
                            }
                        }
                    }

                    // Switch to Typing mode
                    KeyCode::Char('i') => { 
                        self.current_mode = CurrentMode::Typing;
                        self.show_mode_notification = true;
                        self.notification_time_count = Some(Instant::now());
                        self.needs_redraw = true;
                    },

                    // If Enter is pressed in the Words/Text typing options, 
                    // with no words/text file provided - use the default set.
                    KeyCode::Enter => { 
                        match self.current_typing_mode {
                            CurrentTypingOption::Words => {
                                if self.words.len() == 0 {
                                    // Set the Words option to use the default set
                                    let default_words = vec!["the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now", "look", "only", "come", "over", "think", "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any", "these", "give", "day", "most", "us", "thing", "man", "find", "part", "eye", "place", "week", "case", "point", "government", "company", "number", "group", "problem", "fact", "leave", "while", "mean", "keep", "student", "great", "seem", "same", "tell", "begin", "help", "talk", "where", "turn", "start", "might", "show", "hear", "play", "run", "move", "live", "believe", "hold", "bring", "happen", "must", "write", "provide", "sit", "stand", "lose", "pay", "meet", "include", "continue", "set", "learn", "change", "lead", "understand", "watch", "follow", "stop", "create", "speak", "read", "allow", "add", "spend", "grow", "open", "walk", "win", "offer", "remember", "love", "consider", "appear", "buy", "wait", "serve", "die", "send", "expect", "build", "stay", "fall", "cut", "reach", "kill", "remain", "suggest", "raise", "pass", "sell", "require", "report", "decide", "pull", "return", "explain", "hope", "develop", "carry", "break", "receive", "agree", "support", "hit", "produce", "eat", "cover", "catch", "draw", "choose", "cause", "listen", "maybe", "until", "without", "probably", "around", "small", "green", "special", "difficult", "available", "likely", "short", "single", "medical", "current", "wrong", "private", "past", "foreign", "fine", "common", "poor", "natural", "significant", "similar", "hot", "dead", "central", "happy", "serious", "ready", "simple", "left", "physical", "general", "environmental", "financial", "blue", "democratic", "dark", "various", "entire", "close", "legal", "religious", "cold", "final", "main", "huge", "popular", "traditional", "cultural", "choice", "high", "big", "large", "particular", "tiny", "enormous"];
                                    let default_words: Vec<String> = default_words
                                        .iter()
                                        .map(|w| w.to_string())
                                        .collect();
                                    self.words = default_words;

                                    // Generate three lines worth of words (characters) and ids.
                                    // Keep track of the length of those lines in characters.
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_words();
                                        let characters: Vec<char> = one_line.chars().collect();
                                        self.lines_len.push_back(characters.len());
                                        for char in characters {
                                            self.charset.push_back(char.to_string());
                                            self.ids.push_back(0);
                                        }
                                    }

                                    self.needs_redraw = true;
                                }
                            }
                            CurrentTypingOption::Text => {
                                if self.text.len() == 0 {
                                    // Set the Text option to use the default set
                                    let default_text = vec!["The", "shimmering", "dragonfly", "hovered", "over", "the", "tranquil", "pond.", "Ancient", "mountains", "guard", "secrets", "of", "a", "time", "long", "forgotten.", "A", "melancholic", "melody", "drifted", "from", "the", "old,", "forgotten", "gramophone.", "The", "bustling", "city", "market", "was", "a", "kaleidoscope", "of", "colors,", "sounds,", "and", "smells.", "Through", "the", "fog,", "a", "lone", "lighthouse", "cast", "a", "guiding", "beam", "for", "lost", "sailors.", "The", "philosopher", "pondered", "the", "intricate", "dance", "between", "fate", "and", "free", "will.", "A", "child's", "laughter", "echoed", "in", "the", "empty", "playground,", "a", "ghost", "of", "happier", "times.", "The", "weathered", "fisherman", "mended", "his", "nets,", "his", "face", "a", "map", "of", "the", "sea.", "Cryptic", "symbols", "adorned", "the", "walls", "of", "the", "newly", "discovered", "tomb.", "The", "scent", "of", "rain", "on", "dry", "earth", "filled", "the", "air,", "a", "promise", "of", "renewal.", "A", "weary", "traveler", "sought", "refuge", "from", "the", "relentless", "storm", "in", "a", "deserted", "cabin.", "The", "artist's", "canvas", "held", "a", "chaotic", "explosion", "of", "emotions,", "rendered", "in", "oil", "and", "acrylic.", "Stars,", "like", "scattered", "diamonds,", "adorned", "the", "velvet", "canvas", "of", "the", "night", "sky.", "The", "old", "librarian", "cherished", "the", "silent", "companionship", "of", "his", "leather-bound", "books.", "A", "forgotten", "diary", "revealed", "the", "secret", "love", "story", "of", "a", "bygone", "era.", "The", "chef", "meticulously", "arranged", "the", "dish,", "transforming", "food", "into", "a", "work", "of", "art.", "In", "the", "heart", "of", "the", "forest,", "a", "hidden", "waterfall", "cascaded", "into", "a", "crystal-clear", "pool.", "The", "politician's", "speech", "was", "a", "carefully", "constructed", "fortress", "of", "half-truths", "and", "promises.", "A", "sudden", "gust", "of", "wind", "scattered", "the", "autumn", "leaves", "like", "a", "flurry", "of", "colorful", "confetti.", "The", "detective", "followed", "a", "labyrinthine", "trail", "of", "clues,", "each", "one", "more", "perplexing", "than", "the", "last.", "The", "scent", "of", "jasmine", "hung", "heavy", "in", "the", "humid", "evening", "air.", "Time", "seemed", "to", "slow", "down", "in", "the", "sleepy,", "sun-drenched", "village.", "The", "blacksmith's", "hammer", "rang", "out", "a", "rhythmic", "chorus", "against", "the", "glowing", "steel.", "A", "lone", "wolf", "howled", "at", "the", "full", "moon,", "its", "call", "a", "lament", "for", "its", "lost", "pack.", "The", "mathematician", "found", "elegance", "and", "beauty", "in", "the", "complex", "simplicity", "of", "equations.", "From", "the", "ashes", "of", "defeat,", "a", "spark", "of", "resilience", "began", "to", "glow.", "The", "antique", "clock", "ticked", "with", "a", "solemn,", "unhurried", "rhythm,", "marking", "the", "passage", "of", "time.", "A", "hummingbird,", "a", "jeweled", "marvel", "of", "nature,", "darted", "from", "flower", "to", "flower.", "The", "decrepit", "mansion", "on", "the", "hill", "was", "rumored", "to", "be", "haunted", "by", "a", "benevolent", "spirit.", "Sunlight", "streamed", "through", "the", "stained-glass", "windows,", "painting", "the", "cathedral", "floor", "in", "vibrant", "hues.", "The", "aroma", "of", "freshly", "baked", "bread", "wafted", "from", "the", "cozy", "little", "bakery.", "A", "complex", "network", "of", "roots", "anchored", "the", "ancient", "oak", "tree", "to", "the", "earth.", "The", "programmer", "stared", "at", "the", "screen,", "searching", "for", "the", "single,", "elusive", "bug", "in", "a", "million", "lines", "of", "code.", "The", "waves", "crashed", "against", "the", "rocky", "shore", "in", "a", "timeless,", "powerful", "rhythm.", "A", "flock", "of", "geese", "flew", "south", "in", "a", "perfect", "V-formation,", "a", "testament", "to", "their", "instinctual", "harmony.", "The", "historian", "pieced", "together", "the", "fragments", "of", "the", "past", "to", "tell", "a", "coherent", "story.", "In", "the", "quiet", "solitude", "of", "the", "desert,", "one", "could", "hear", "the", "whisper", "of", "the", "wind.", "The", "gardener", "tended", "to", "her", "roses", "with", "a", "gentle,", "nurturing", "touch.", "A", "crackling", "fireplace", "provided", "a", "warm", "and", "inviting", "centerpiece", "to", "the", "rustic", "living", "room.", "The", "mountaineer", "stood", "at", "the", "summit,", "humbled", "by", "the", "breathtaking", "vista", "below.", "A", "single,", "perfect", "snowflake", "landed", "on", "the", "child's", "outstretched", "mitten."];
                                    let default_text: Vec<String> = default_text
                                        .iter()
                                        .map(|w| w.to_string())
                                        .collect();
                                    self.text = default_text;

                                    // Generate three lines worth of words (characters) and ids.
                                    // Keep track of the length of those lines in characters.
                                    for _ in 0..3 {
                                        let one_line = self.gen_one_line_of_text();

                                        // Count for how many "words" there were on the first three lines
                                        // to keep position on option switch and exit.
                                        // Otherwise would always skip 3 lines down.
                                        let first_text_gen_len: Vec<String> = one_line.split_whitespace().map(String::from).collect();
                                        self.first_text_gen_len += first_text_gen_len.len();

                                        // Push a line of characters (from text) and ids
                                        let characters: Vec<char> = one_line.chars().collect();
                                        self.lines_len.push_back(characters.len());
                                        for char in characters {
                                            self.charset.push_back(char.to_string());
                                            self.ids.push_back(0);
                                        }
                                    }
                                    
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
                        self.show_mode_notification = true;
                        self.notification_time_count = Some(Instant::now());
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
}