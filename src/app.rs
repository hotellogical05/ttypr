use crate::utils::{default_text, default_words, Config};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

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

    /// Handles cleanup and saving before the application exits.
    ///
    /// This function is called just before the application terminates. It's
    /// responsible for persisting the application's state, such as saving the
    /// current configuration and adjusting any other relevant settings.
    pub fn on_exit(&mut self) {
        use crate::utils::{get_config_dir, save_config};

        // (If exited the application while being the Text option)
        // Subtract how many "words" there were on the first three lines
        match self.current_typing_option {
            CurrentTypingOption::Text => {
                if self.config.skip_len >= self.first_text_gen_len {
                    self.config.skip_len -= self.first_text_gen_len;
                } else {
                    self.config.skip_len = 0;
                }
            }
            _ => {}
        }

        // Save config (for mistyped characters) before exiting
        if let Ok(config_dir) = get_config_dir() {
            save_config(&self.config, &config_dir).unwrap_or_else(|err| {
                eprintln!("Failed to save config: {}", err);
            });
        }
    }

    /// Timer for notifications display
    pub fn on_tick(&mut self) {
        if self.notifications.on_tick() {
            self.needs_clear = true;
            self.needs_redraw = true;
        }
    }

    /// Initializes the application state at startup.
    ///
    /// This function is responsible for setting up the initial state of the
    /// application. It loads the configuration, populates the initial character
    /// sets for typing, and prepares the application to be run.
    pub fn setup(&mut self) -> color_eyre::Result<()> {
        use crate::utils::{
            calculate_text_txt_hash, default_text, default_words, get_config_dir, load_config,
            read_text_from_file, read_words_from_file,
        };

        // Get the config directory
        let config_dir = get_config_dir()?;

        // Load config file or create it
        self.config = load_config(&config_dir).unwrap_or_else(|_err| Config::default());

        // (For the ASCII option) - Generate initial random charset and set all ids to 0
        // (This for block is here because the default typing option is Ascii)
        for _ in 0..3 {
            let one_line = self.gen_one_line_of_ascii();

            let characters: Vec<char> = one_line.chars().collect();
            self.lines_len.push_back(characters.len());
            for char in characters {
                self.charset.push_back(char.to_string());
                self.ids.push_back(0);
            }
        }

        // (For the Words option) - Read the words from .config/ttypr/words.txt
        // If it doesn't exist, it will default to an empty vector.
        self.words = read_words_from_file(&config_dir).unwrap_or_default();

        // (For the Text option) - Read the text from .config/ttypr/text.txt
        // If it doesn't exist, it will default to an empty vector.
        self.text = read_text_from_file(&config_dir).unwrap_or_default();

        // If words file provided use that one instead of the default set
        if !self.words.is_empty() {
            self.config.use_default_word_set = false;
        }

        // Use the default word set if previously selected to use it
        if self.config.use_default_word_set {
            self.words = default_words();
        }

        // This is for if user decided to switch between using the default text set
        // and a provided one.
        // If text file was provided, and default text set was previously selected -
        // use the provided file contents instead from now on, and reset the
        // Text option position.
        if !self.text.is_empty() && self.config.use_default_text_set {
            self.config.use_default_text_set = false;
            self.config.skip_len = 0;
        }

        // This is for if user decided to switch between using the default text set
        // and a provided one.
        // If file was not provided, and default text set is not selected - set the
        // Text option position to the beginning.
        // (This is here because the user can delete the provided text set, so this
        // if block resets the position in the Text option to the beginning)
        if self.text.is_empty() && !self.config.use_default_text_set {
            self.config.skip_len = 0;
        }

        // Use the default text set if previously selected to use it
        if self.config.use_default_text_set {
            self.text = default_text();
        }

        // If the contents of the .config/ttypr/text.txt changed -
        // reset the position to the beginning
        if self.config.last_text_txt_hash != calculate_text_txt_hash(&config_dir).ok() {
            self.config.skip_len = 0;
        }

        // Calculate the hash of the .config/ttypr/text.txt to
        // compare to the previously generated one and determine
        // whether the file contents have changed
        self.config.last_text_txt_hash = calculate_text_txt_hash(&config_dir).ok();

        Ok(())
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

    /// Reads the terminal events.
    pub fn handle_crossterm_events(&mut self) -> Result<()> {
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

    /// Handles keyboard input.
    fn on_key_event(&mut self, key: KeyEvent) {
        // First boot page input (if toggled takes all input)
        // If Enter key is pressed sets first_boot to false in the config file
        if self.config.first_boot {
            match key.code {
                KeyCode::Enter => {
                    self.config.first_boot = false;
                    if let Ok(config_dir) = crate::utils::get_config_dir() {
                        crate::utils::save_config(&self.config, &config_dir).unwrap_or_else(|err| {
                            eprintln!("Failed to save config: {}", err);
                        });
                    }
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
                                    return;
                                }
                            }
                            CurrentTypingOption::Text => {
                                if self.text.len() == 0 {
                                    return;
                                }
                            }
                            _ => {}
                        }

                        self.current_mode = CurrentMode::Typing;
                        self.notifications.show_mode();
                        self.needs_redraw = true;
                    }

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
                                        let first_text_gen_len: Vec<String> = one_line
                                            .split_whitespace()
                                            .map(String::from)
                                            .collect();
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
            }

            // Typing mode input
            CurrentMode::Typing => {
                match key.code {
                    KeyCode::Esc => {
                        // Switch to Menu mode if ESC pressed
                        self.current_mode = CurrentMode::Menu;
                        self.notifications.show_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char(c) => {
                        // Add to input characters
                        self.input_chars.push_back(c.to_string());
                        self.needs_redraw = true;
                        self.typed = true;
                    }
                    KeyCode::Backspace => {
                        // Remove from input characters
                        let position = self.input_chars.len();
                        if position > 0 {
                            // If there are no input characters - don't do anything
                            self.input_chars.pop_back();
                            self.ids[position - 1] = 0;
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
                        let first_text_gen_len: Vec<String> =
                            one_line.split_whitespace().map(String::from).collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_notifications_on_tick() {
        let mut notifications = Notifications::new();

        // Should return false when no notification is active
        assert!(!notifications.on_tick());

        // Show a notification to start the timer
        notifications.show_mode();
        assert!(notifications.mode);
        assert!(notifications.time_count.is_some());

        // Should still return false immediately after
        assert!(!notifications.on_tick());

        // Wait for more than 2 seconds
        thread::sleep(Duration::from_secs(3));

        // Now on_tick should return true and hide notifications
        assert!(notifications.on_tick());
        assert!(!notifications.mode);
        assert!(notifications.time_count.is_none());
    }

    #[test]
    fn test_notifications_hide_all() {
        let mut notifications = Notifications::new();

        // Show some notifications
        notifications.show_mode();
        notifications.show_option();
        notifications.show_toggle();
        notifications.show_mistyped();
        notifications.show_clear_mistyped();

        // Hide them
        notifications.hide_all();

        // Check that all flags are false
        assert!(!notifications.mode);
        assert!(!notifications.option);
        assert!(!notifications.toggle);
        assert!(!notifications.mistyped);
        assert!(!notifications.clear_mistyped);
        assert!(notifications.time_count.is_none());
    }

    #[test]
    fn test_notifications_trigger() {
        let mut notifications = Notifications::new();

        // Timer should not be set initially
        assert!(notifications.time_count.is_none());

        // Trigger the timer
        notifications.trigger();

        // Timer should now be set
        assert!(notifications.time_count.is_some());
    }

    #[test]
    fn test_notifications_show_methods() {
        let mut notifications = Notifications::new();

        // Test show_mode
        notifications.show_mode();
        assert!(notifications.mode);
        assert!(notifications.time_count.is_some());
        notifications.hide_all(); // Reset for next test

        // Test show_option
        notifications.show_option();
        assert!(notifications.option);
        assert!(notifications.time_count.is_some());
        notifications.hide_all();

        // Test show_toggle
        notifications.show_toggle();
        assert!(notifications.toggle);
        assert!(notifications.time_count.is_some());
        notifications.hide_all();

        // Test show_mistyped
        notifications.show_mistyped();
        assert!(notifications.mistyped);
        assert!(notifications.time_count.is_some());
        notifications.hide_all();

        // Test show_clear_mistyped
        notifications.show_clear_mistyped();
        assert!(notifications.clear_mistyped);
        assert!(notifications.time_count.is_some());
    }

    #[test]
    fn test_app_gen_one_line_of_ascii() {
        let mut app = App::new();
        app.line_len = 50;
        let line = app.gen_one_line_of_ascii();
        assert_eq!(line.chars().count(), 50);

        app.line_len = 10;
        let line = app.gen_one_line_of_ascii();
        assert_eq!(line.chars().count(), 10);
    }

    #[test]
    fn test_app_gen_one_line_of_words() {
        let mut app = App::new();
        app.line_len = 50;
        app.words = vec!["hello".to_string(), "world".to_string(), "this".to_string(), "is".to_string(), "a".to_string(), "test".to_string()];

        let line = app.gen_one_line_of_words();
        
        // Check that the line is not empty
        assert!(!line.is_empty());

        // Check that the line length is within the limit
        assert!(line.chars().count() <= app.line_len);

        // Check with a smaller line length
        app.line_len = 10;
        let line = app.gen_one_line_of_words();
        assert!(!line.is_empty());
        assert!(line.chars().count() <= app.line_len);
    }

    #[test]
    fn test_app_gen_one_line_of_text() {
        let mut app = App::new();
        app.line_len = 20;
        app.text = "This is a sample text for testing purposes."
            .split_whitespace()
            .map(String::from)
            .collect();
        app.config.skip_len = 0;

        // First line generation
        let line1 = app.gen_one_line_of_text();
        assert_eq!(line1, "This is a sample");
        assert_eq!(app.config.skip_len, 4); // Should have processed 4 words

        // Second line generation
        let line2 = app.gen_one_line_of_text();
        assert_eq!(line2, "text for testing");
        assert_eq!(app.config.skip_len, 7);

        // Third line generation, testing wrap-around
        let line3 = app.gen_one_line_of_text();
        assert_eq!(line3, "purposes. This is a");
        assert_eq!(app.config.skip_len, 3); // Wrapped around and used 3 words
    }

    #[test]
    fn test_app_update_id_field() {
        let mut app = App::new();
        app.charset = VecDeque::from(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        app.ids = VecDeque::from(vec![0, 0, 0]);
        
        // --- Test 1: Correct character ---
        app.input_chars.push_back("a".to_string());
        app.update_id_field();
        assert_eq!(app.ids[0], 1);

        // --- Test 2: Incorrect character, without saving mistypes ---
        app.config.save_mistyped = false;
        app.input_chars.push_back("x".to_string()); // Correct char is "b"
        app.update_id_field();
        assert_eq!(app.ids[1], 2);
        assert!(app.config.mistyped_chars.is_empty()); // Should not record

        // --- Test 3: Incorrect character, with saving mistypes ---
        app.config.save_mistyped = true;
        app.input_chars.push_back("y".to_string()); // Correct char is "c"
        app.update_id_field();
        assert_eq!(app.ids[2], 2);
        assert_eq!(*app.config.mistyped_chars.get("c").unwrap(), 1); // "c" was mistyped once
    }

    #[test]
    fn test_app_update_lines() {
        let mut app = App::new();
        app.line_len = 5; // Use a short line length for easier testing

        // --- Setup initial state with 3 lines of content ---
        app.current_typing_option = CurrentTypingOption::Ascii;
        
        // Line 1: "aaaaa"
        app.charset.extend(vec!["a".to_string(); 5]);
        app.ids.extend(vec![1; 5]); // Simulate typed
        app.input_chars.extend(vec!["a".to_string(); 5]);
        app.lines_len.push_back(5);

        // Line 2: "bbbbb"
        app.charset.extend(vec!["b".to_string(); 5]);
        app.ids.extend(vec![1; 5]); // Simulate typed
        app.input_chars.extend(vec!["b".to_string(); 5]);
        app.lines_len.push_back(5);

        // Line 3: "ccccc" (not yet typed)
        app.charset.extend(vec!["c".to_string(); 5]);
        app.ids.extend(vec![0; 5]);
        app.lines_len.push_back(5);

        // At this point, input_chars length is 10, which equals lines_len[0] + lines_len[1]
        assert_eq!(app.input_chars.len(), app.lines_len[0] + app.lines_len[1]);

        // --- Call the function to test ---
        app.update_lines();

        // --- Assert the results ---
        // 1. First line's data should be removed from buffers
        assert_eq!(app.input_chars.len(), 5);
        assert_eq!(app.input_chars.front().unwrap(), "b");
        
        // 2. A new line should be generated and added
        assert_eq!(app.lines_len.len(), 3); // Still 3 lines
        assert_eq!(app.lines_len[0], 5); // Old line 2 is now line 1
        assert_eq!(app.lines_len[1], 5); // Old line 3 is now line 2
        assert_eq!(app.lines_len[2], 5); // New line 3 has been added
        
        assert_eq!(app.charset.len(), 15); // Total chars should be back to 15
        assert_eq!(app.ids.len(), 15);      // Total ids should be back to 15
        
        // 3. The newly added ids should be 0 (untyped)
        // (Check the last 5 elements of the ids VecDeque)
        assert!(app.ids.iter().skip(10).all(|&id| id == 0));
    }

    #[test]
    fn test_app_clear_typing_buffers() {
        let mut app = App::new();

        // Populate buffers with some data
        app.charset.push_back("a".to_string());
        app.input_chars.push_back("a".to_string());
        app.ids.push_back(1);
        app.lines_len.push_back(1);

        // Ensure they are not empty before clearing
        assert!(!app.charset.is_empty());
        assert!(!app.input_chars.is_empty());
        assert!(!app.ids.is_empty());
        assert!(!app.lines_len.is_empty());

        // Call the function
        app.clear_typing_buffers();

        // Assert that all buffers are empty
        assert!(app.charset.is_empty());
        assert!(app.input_chars.is_empty());
        assert!(app.ids.is_empty());
        assert!(app.lines_len.is_empty());
    }

    #[test]
    fn test_app_switch_typing_option() {
        let mut app = App::new();
        // Provide some data for words and text modes
        app.words = vec!["word1".to_string(), "word2".to_string()];
        app.text = vec!["text1".to_string(), "text2".to_string()];
        app.line_len = 10;
        
        // --- 1. Switch from ASCII (default) to Words ---
        assert!(matches!(app.current_typing_option, CurrentTypingOption::Ascii));
        app.switch_typing_option();
        assert!(matches!(app.current_typing_option, CurrentTypingOption::Words));
        assert!(!app.charset.is_empty()); // Should be populated with words
        assert!(!app.lines_len.is_empty());

        // --- 2. Switch from Words to Text ---
        app.switch_typing_option();
        assert!(matches!(app.current_typing_option, CurrentTypingOption::Text));
        assert!(!app.charset.is_empty()); // Should be populated with text
        assert_ne!(app.first_text_gen_len, 0); // Should be tracking generated text length

        // --- 3. Switch from Text back to ASCII ---
        app.switch_typing_option();
        assert!(matches!(app.current_typing_option, CurrentTypingOption::Ascii));
        assert!(!app.charset.is_empty()); // Should be populated with ASCII
        assert_eq!(app.first_text_gen_len, 0); // Should be reset
    }

    #[test]
    fn test_app_populate_charset_from_line() {
        let mut app = App::new();
        let line = "hello".to_string();
        
        app.populate_charset_from_line(line);

        // Check lines_len
        assert_eq!(app.lines_len.len(), 1);
        assert_eq!(app.lines_len[0], 5);

        // Check charset
        let expected_charset = VecDeque::from(vec!["h".to_string(), "e".to_string(), "l".to_string(), "l".to_string(), "o".to_string()]);
        assert_eq!(app.charset, expected_charset);

        // Check ids
        assert_eq!(app.ids.len(), 5);
        assert!(app.ids.iter().all(|&id| id == 0)); // All ids should be 0
    }
}