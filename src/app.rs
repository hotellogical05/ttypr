use std::collections::VecDeque;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::utils::{Config};

/// Manages the state and display timer for transient notifications in the UI.
pub struct Notifications {
    pub mode: bool,
    pub option: bool,
    pub toggle: bool,
    pub mistyped: bool,
    pub clear_mistyped: bool,
    pub time_count: Option<Instant>,
}

impl Notifications {
    /// Creates a new `Notifications` instance with all flags turned off.
    pub fn new() -> Notifications {
        Notifications {
            mode: false,
            option: false,
            toggle: false,
            mistyped: false,
            clear_mistyped: false,
            time_count: None,
        }
    }

    /// Call this on each application tick to manage notification visibility.
    /// Returns true if the UI needs to be updated.
    pub fn on_tick(&mut self) -> bool {
        if let Some(shown_at) = self.time_count {
            if shown_at.elapsed() > Duration::from_secs(2) {
                self.hide_all();
                return true; // Indicates an update is needed
            }
        }
        false
    }

    /// Hides all notifications and resets the timer.
    fn hide_all(&mut self) {
        self.mode = false;
        self.option = false;
        self.toggle = false;
        self.mistyped = false;
        self.clear_mistyped = false;
        self.time_count = None;
    }

    /// Starts the visibility timer for the currently active notification.
    fn trigger(&mut self) {
        self.time_count = Some(Instant::now());
    }

    /// Shows a notification indicating a mode change.
    pub fn show_mode(&mut self) {
        self.mode = true;
        self.trigger();
    }

    /// Shows a notification indicating a typing option change.
    pub fn show_option(&mut self) {
        self.option = true;
        self.trigger();
    }

    /// Shows a notification indicating that notifications have been toggled.
    pub fn show_toggle(&mut self) {
        self.toggle = true;
        self.trigger();
    }

    /// Shows a notification indicating that counting mistyped characters has been toggled.
    pub fn show_mistyped(&mut self) {
        self.mistyped = true;
        self.trigger();
    }

    /// Shows a notification that the mistyped characters count has been cleared.
    pub fn show_clear_mistyped(&mut self) {
        self.clear_mistyped = true;
        self.trigger();
    }
}

/// Represents the main application state and logic.
///
/// This struct holds all the data necessary for the application to run, including
/// UI state, typing data, user input, and configuration settings. It is
/// responsible for handling user input, updating the state, and managing the
/// overall application lifecycle.
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
    pub text: Vec<String>,
    pub notifications: Notifications,
    pub config: Config,
    pub show_help: bool,
    pub show_mistyped: bool,
    pub first_text_gen_len: usize,
}

/// Defines the major operational modes of the application.
pub enum CurrentMode {
    /// The menu mode , is used for managing settings, switching typing options,
    /// viewing mistyped characters, and accessing the help page.
    Menu,
    /// The typing mode, where the user actively practices typing.
    Typing,
}

/// Defines the different types of content the user can practice typing.
pub enum CurrentTypingOption {
    Ascii,
    Words,
    Text,
}

/// A constant array of ASCII characters used for generating lines of random ASCII characters.
const ASCII_CHARSET: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "~", "`", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "_", "+", "=", "{", "}", "[", "]", "|", "\\", ":", ";", "\"", "'", "<", ">", ",", ".", "?", "/"];

impl App {
    /// Construct a new instance of App
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
            text: vec![],
            notifications: Notifications::new(),
            config: Config::default(),
            show_help: false,
            show_mistyped: false,
            first_text_gen_len: 0,
        }
    }

    /// Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Timer for notifications display
    pub fn on_tick(&mut self) {
        if self.notifications.on_tick() {
            self.needs_clear = true;
            self.needs_redraw = true;
        }
    }

    /// Constructs a line of random ASCII characters that fits within the configured line length.
    pub fn gen_one_line_of_ascii(&mut self) -> String {
        let mut line_of_ascii = vec![];
        for _ in 0..self.line_len {
            let index = rand::rng().random_range(0..ASCII_CHARSET.len());
            let character = ASCII_CHARSET[index];
            line_of_ascii.push(character.to_string())
        }
        line_of_ascii.join("")
    }

    /// Constructs a line of random words that fits within the configured line length.
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

    /// Retrieves the next line of text from the source, respecting the configured line length.
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

    /// Set the ID for the last typed character to determine its color,
    /// and record it if it was a mistype.
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

    /// Manages the scrolling display by updating the character buffers.
    ///
    /// When the user finishes typing the second line, this function removes the
    /// first line's data from the buffers and appends a new line, creating a
    /// continuous scrolling effect.
    pub fn update_lines(&mut self) {
        // If reached the end of the second line
        if self.input_chars.len() == self.lines_len[0] + self.lines_len[1] {
            // Remove first line amount of characters from the character set, 
            // the user inputted characters, and ids. 
            for _ in 0..self.lines_len[0] {
                self.charset.pop_front();
                self.input_chars.pop_front();
                self.ids.pop_front();
            }
        
            // One line of ascii characters/words/text
            let one_line = match self.current_typing_option {
                CurrentTypingOption::Ascii => { self.gen_one_line_of_ascii() },
                CurrentTypingOption::Words => { self.gen_one_line_of_words() },
                CurrentTypingOption::Text => { self.gen_one_line_of_text() },
            };
        
            // Convert that line into characters
            let characters: Vec<char> = one_line.chars().collect();
        
            // Remove the length of the first line of characters from the front, 
            // and push the new one to the back.
            self.lines_len.pop_front();
            self.lines_len.push_back(characters.len());
        
            // Push new amount of characters (words) to charset, and that amount of 0's to ids
            for char in characters {
                self.charset.push_back(char.to_string());
                self.ids.push_back(0);
            }
        }
    }

    /// Empties the buffers that store the character set, user input, IDs and line lengths.
    ///
    /// This is called when the typing option is switched - to reset the buffers for 
    /// the new content.
    pub fn clear_typing_buffers(&mut self) {
        self.charset.clear();
        self.input_chars.clear();
        self.ids.clear();
        self.lines_len.clear();
    }
}