use std::collections::VecDeque;

pub struct App {
    pub running: bool,
    pub needs_redraw: bool,
    pub typed: bool,
    pub charset: VecDeque<String>, // the random ASCII character set
    pub input_chars: VecDeque<String>, // the characters user typed
    pub ids: VecDeque<u8>, // identifiers to display colored characters (0 - untyped, 1 - correct, 2 - incorrect)
    pub line_len: usize,
    pub current_mode: CurrentMode,
    pub current_typing_mode: CurrentTypingMode,
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
            typed: false,
            charset: VecDeque::new(),
            input_chars: VecDeque::new(),
            ids: VecDeque::new(),
            line_len: 40,
            current_mode: CurrentMode::Menu,
            current_typing_mode: CurrentTypingMode::Ascii,
        }
    }

    // Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
