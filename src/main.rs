use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    DefaultTerminal,
    layout::Flex,
    widgets::{List, ListItem},
};
use std::collections::VecDeque;
use ttypr::gen_random_ascii_char;

#[derive(Debug)]
pub struct App {
    running: bool,
    typed: bool,
    charset: VecDeque<String>, // the random ASCII character set
    input_chars: VecDeque<String>, // the characters user typed
    ids: VecDeque<u8>, // ids to display characters (0 - untyped, 1 - correct, 2 - incorrect)
    needs_redraw: bool,
    line_len: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app= App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

impl App {
   // Construct a new instance of App
    pub fn new() -> Self {
        Self { 
            running: true, 
            typed: false,
            charset: VecDeque::new(),
            input_chars: VecDeque::new(),
            ids: VecDeque::new(),
            needs_redraw: true,
            line_len: 40,
        }
    }

    // Run the application's main loop
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // generate initial random charset and all ids set to 0
        for _ in 0..self.line_len*3 {
            self.charset.push_back(gen_random_ascii_char());
            self.ids.push_back(0);
        }
        
        while self.running {
            if self.needs_redraw {
                // all this if block does is update the ids vector
                if self.typed {
                    // however many characters the user typed, to compare with the charset
                    let position = self.input_chars.len();

                    // if the input character matches the characters in the
                    // charset replace the 0 in ids with 1 (correct), 2 (incorrect)
                    if self.input_chars[position-1] == self.charset[position-1] {
                        self.ids[position-1] = 1;
                    } else {
                        self.ids[position-1] = 2;
                    }

                    // if reached the end of the second line, remove line_len
                    // (the first line) characters from the user inputted characters vector
                    if self.input_chars.len() == self.line_len*2 {
                        for _ in 0..self.line_len {
                            let _ = self.charset.pop_front();
                            let _ = self.input_chars.pop_front();
                            let _ = self.ids.pop_front();
                            self.charset.push_back(gen_random_ascii_char());
                            self.ids.push_back(0);
                        }
                    }
                    self.typed = false;
                }
                terminal.draw(|frame| self.render(frame))?;
                self.needs_redraw = false;
            }
            self.handle_crossterm_events()?;
        }
        Ok(())
    }
    
    // Render the user interface, where to add new widgets
    fn render(&mut self, frame: &mut Frame) {
        // where to display the lines
        let area = center(
            frame.area(),
            Constraint::Length(self.line_len as u16), // width depending on set line length
            Constraint::Length(5), // height, 5 - because spaces between them
        );

        // a vector of colored characters
        let span: Vec<Span> = self.charset.iter().enumerate().map(|(i, c)| {
            // if inputted character matches charset character add a green colored character that user inputted
            if self.ids[i] == 1 {
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))
            // if inputted character doesn't match charset character add a red colored character that user inputted
            } else if self.ids[i] == 2 { // 
                Span::styled(self.input_chars[i].to_string(), Style::new().fg(Color::Indexed(9)))
            // otherwise add a grey colored character (hasn't been typed yet)
            } else {
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(8)))
            }
        }).collect();
    
        // separating vector of all the colored characters into vector of 3 lines, each line_len long
        // and making them List items, to display as a List widget
        let mut three_lines = vec![];
        for i in 0..3 {
            // skip 0, 1, 2 lines, take line length of characters, and make a vector out of them
            let line_span: Vec<Span> = span.iter().skip(i*self.line_len).take(self.line_len).map(|c| {
                c.clone()
            }).collect();
            let line = Line::from(line_span);
            let item = ListItem::new(line);
            three_lines.push(item); // push the line
            three_lines.push(ListItem::new("")); // push an empty space to separate lines
        }
        
        // make a List widget out of list items and render it in the middle
        let list = List::new(three_lines);
        frame.render_widget(list, area);
        
        }

    // Reads the crossterm events
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key), // handle keyboard input
            Event::Mouse(_) => {}
            Event::Resize(_, _) => { self.needs_redraw = true; } // re-render if terminal window resized
            _ => {}
        }
        Ok(())
    }

    // What happens on key presses
    fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.quit(), // stop the application if ESC pressed
            KeyCode::Char(c) => {
                self.input_chars.push_back(c.to_string()); // add to input characters
                self.needs_redraw = true;
                self.typed = true;
            }
            KeyCode::Backspace => {
                let position = self.input_chars.len();
                if position > 0 { // if there are no input characters - don't do anything
                    self.input_chars.pop_back();
                    self.ids[position-1] = 0;
                    self.needs_redraw = true;
                }
            }
            _ => {}
        }
    }
    
    // Stop the application
    fn quit(&mut self) {
        self.running = false;
    }
}

// Helper function to center a layout area
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
