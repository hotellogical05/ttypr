use std::collections::VecDeque;
use std::time::{Duration, Instant};
use rand::Rng;
use ttypr::{Config, gen_random_ascii_char};

pub struct App {
    pub running: bool,
    pub needs_redraw: bool,
    pub needs_clear: bool,
    pub typed: bool,
    pub charset: VecDeque<String>, // The random ASCII/Words character set (both are set of characters: ["a", "b", "c"])
    pub input_chars: VecDeque<String>, // The characters user typed
    pub ids: VecDeque<u8>, // Identifiers to display colored characters (0 - untyped, 1 - correct, 2 - incorrect)
    pub line_len: usize,
    pub lines_len: VecDeque<usize>, // Current length of lines in characters for the Words option
    pub current_mode: CurrentMode,
    pub current_typing_option: CurrentTypingOption,
    pub words: Vec<String>,
    pub show_mode_notification: bool,
    pub show_option_notification: bool,
    pub show_notification_toggle: bool,
    pub show_mistyped_notification: bool,
    pub show_clear_mistyped_notification: bool,
    pub notification_time_count: Option<Instant>,
    pub config: Config,
    pub show_help: bool,
    pub show_mistyped: bool,
    pub text: Vec<String>,
    pub first_text_gen_len: usize,
}

pub enum CurrentMode {
    Menu,
    Typing,
}

pub enum CurrentTypingOption {
    Ascii,
    Words,
    Text,
}

impl App {
   // Construct a new instance of App
    pub fn new() -> App {
        App { 
            running: true, 
            needs_redraw: true,
            needs_clear: false,
            typed: false,
            charset: VecDeque::new(),
            input_chars: VecDeque::new(),
            ids: VecDeque::new(),
            line_len: 50,
            lines_len: VecDeque::new(),
            current_mode: CurrentMode::Menu,
            current_typing_option: CurrentTypingOption::Ascii,
            words: vec![],
            show_mode_notification: false,
            show_option_notification: false,
            show_notification_toggle: false,
            show_mistyped_notification: false,
            show_clear_mistyped_notification: false,
            notification_time_count: None,
            config: Config::default(),
            show_help: false,
            show_mistyped: false,
            text: vec![],
            first_text_gen_len: 0,
        }
    }

    // Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    // Timer for notifications display
    pub fn on_tick(&mut self) {
        // If one of the notifications was triggered -
        // start counting
        if self.show_option_notification || 
           self.show_mode_notification || 
           self.show_notification_toggle || 
           self.show_mistyped_notification || 
           self.show_clear_mistyped_notification {

            // Pressing a key that triggers a notification sets
            // notification_time_count to Some()
            // So the logic below runs
            if let Some(shown_at) = self.notification_time_count {
                // If two seconds have passed since a notification was triggered
                if shown_at.elapsed() > Duration::from_secs(2) {
                    // Set displaying all notifications to false
                    self.show_option_notification = false;
                    self.show_mode_notification = false;
                    self.show_notification_toggle = false;
                    self.show_mistyped_notification = false;
                    self.show_clear_mistyped_notification = false;

                    // Stop the timer, clear and redraw the area
                    self.notification_time_count = None;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
            }
        }
    }

    pub fn gen_one_line_of_words(&mut self) -> String {
        let mut line_of_words = vec![];
        loop {
            let index = rand::rng().random_range(0..self.words.len());
            let word = self.words[index].clone();
            line_of_words.push(word);

            let current_line_len = line_of_words.join(" ").chars().count();

            if current_line_len > self.line_len {
                line_of_words.pop();
                let current_line = line_of_words.join(" ");
                return current_line; 
            };
        };
    }

    pub fn gen_one_line_of_text(&mut self) -> String {
        let mut line_of_text = vec![];
        loop {
            // If reached the end of the text - set position to 0
            if self.config.skip_len == self.text.len() { self.config.skip_len = 0 }

            line_of_text.push(self.text[self.config.skip_len].clone());
            let current_line_len = line_of_text.join(" ").chars().count();
            self.config.skip_len += 1;

            if current_line_len > self.line_len {
                line_of_text.pop();
                self.config.skip_len -= 1;

                let current_line = line_of_text.join(" ");
                return current_line;
            }
        }
    }

    pub fn update_id_field(&mut self) {
        // Number of characters the user typed, to compare with the charset
        let pos = self.input_chars.len() - 1;

        // If the input character matches the characters in the
        // charset replace the 0 in ids with 1 (correct), 2 (incorrect)
        if self.input_chars[pos] == self.charset[pos] {
            self.ids[pos] = 1;
        } else {
            self.ids[pos] = 2;
            
            // Add the mistyped character to mistyped characters list
            if self.config.save_mistyped {
                let count = self.config.mistyped_chars.entry(self.charset[pos].to_string()).or_insert(0);
                *count += 1;
            }
        }
    }

    pub fn update_lines(&mut self) {
        match self.current_typing_option {
            
             // For ASCII option
            CurrentTypingOption::Ascii => {

                // If reached the end of the second line, remove line_len
                // (the first line) characters from the character set, the user
                // inputted characters, and ids. Then push the same amount of
                // new random characters to charset, and that amount of 0's to ids
                if self.input_chars.len() == self.line_len*2 {
                    for _ in 0..self.line_len {
                        self.charset.pop_front();
                        self.input_chars.pop_front();
                        self.ids.pop_front();
                    
                        self.charset.push_back(gen_random_ascii_char());
                        self.ids.push_back(0);
                    }
                }
            }
            
             // For Words and Text options
            _ => {

                // If reached the end of the second line, remove first line amount
                // of characters (words) from the character set, the user
                // inputted characters, and ids. Then push new line amount of 
                // characters (words) to charset, and that amount of 0's to ids
                if self.input_chars.len() == self.lines_len[0] + self.lines_len[1] {
                    for _ in 0..self.lines_len[0] {
                        self.charset.pop_front();
                        self.input_chars.pop_front();
                        self.ids.pop_front();
                    }
                
                    let one_line = match self.current_typing_option {
                        CurrentTypingOption::Words => { self.gen_one_line_of_words() },
                        CurrentTypingOption::Text => { self.gen_one_line_of_text() },
                        _ => String::new(),
                    };
                
                    let characters: Vec<char> = one_line.chars().collect();
                
                    // Remove the length of the first line of words from the front, 
                    // and push the new one to the back.
                    self.lines_len.pop_front();
                    self.lines_len.push_back(characters.len());
                
                    for char in characters {
                        self.charset.push_back(char.to_string());
                        self.ids.push_back(0);
                    }
                }
            }
        }
    }
}
