use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ttypr::Config;

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
    pub current_typing_mode: CurrentTypingMode,
    pub words: Vec<String>,
    pub show_mode_notification: bool,
    pub show_option_notification: bool,
    pub notification_time_count: Option<Instant>,
    pub config: Option<Config>,
}

pub enum CurrentMode {
    Menu,
    Typing,
}

pub enum CurrentTypingMode {
    Ascii,
    Words,
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
            line_len: 40,
            lines_len: VecDeque::new(),
            current_mode: CurrentMode::Menu,
            current_typing_mode: CurrentTypingMode::Ascii,
            words: vec![],
            show_mode_notification: false,
            show_option_notification: false,
            notification_time_count: None,
            config: None,
        }
    }

    // Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn show_mode_notification(&mut self) {
        self.show_mode_notification = true;
        self.notification_time_count = Some(Instant::now());
    }

    pub fn show_option_notification(&mut self) {
        self.show_option_notification = true;
        self.notification_time_count = Some(Instant::now());
    }

    pub fn on_tick(&mut self) {
        if self.show_option_notification || self.show_mode_notification {
            if let Some(shown_at) = self.notification_time_count {
                if shown_at.elapsed() > Duration::from_secs(2) {
                    self.show_option_notification = false;
                    self.show_mode_notification = false;
                    self.notification_time_count = None;
                    self.needs_clear = true;
                    self.needs_redraw = true;
                }
            }
        }
    }
}
