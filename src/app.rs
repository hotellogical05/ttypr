use std::collections::VecDeque;

pub struct App {
    pub running: bool,
    pub needs_redraw: bool,
    pub typed: bool,
    pub charset: VecDeque<String>, // the random ASCII character set
    pub input_chars: VecDeque<String>, // the characters user typed
    pub ids: VecDeque<u8>, // identifiers to display colored characters (0 - untyped, 1 - correct, 2 - incorrect)
    pub line_len: usize,
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
        }
    }

    // Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
