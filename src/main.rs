use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    DefaultTerminal,
    layout::Flex,
    widgets::Paragraph,
    widgets::List,
};
use std::collections::VecDeque;
use ttypr::gen_random_ascii_char;
use ratatui::widgets::ListItem;

#[derive(Debug)]
pub struct App {
    running: bool,
    charset: VecDeque<String>,
    input_chars: VecDeque<String>,
    ids: VecDeque<u8>,
    whether_first: bool,
    line: usize,
    typed: bool,
    needs_redraw: bool,
    line_len: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

impl App {
   // Construct a new instance of App
    pub fn new() -> Self {
        Self { 
            running: true, 
            charset: VecDeque::new(),
            input_chars: VecDeque::new(),
            ids: VecDeque::new(),  // 0 - untyped, 1 - correct, 2 - incorrect
            whether_first: true,
            line: 0,
            typed: false,
            needs_redraw: true,
            line_len: 0,
        }
    }

    // Run the application's main loop
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // generate initial charset and ids
        for _ in 0..9 {
            self.charset.push_back(gen_random_ascii_char());
            self.ids.push_back(0);
        }
        
        while self.running {
            if self.needs_redraw {
                if self.typed {
                    // position, whether 3 chars check
                    let position = self.input_chars.len();
                    if self.input_chars.len() == 3 {
                        self.whether_first = false;
                    }
    
                    if self.whether_first {
                        if self.input_chars[position-1] == self.charset[position-1] {
                            self.ids[position-1] = 1;
                        } else {
                            self.ids[position-1] = 2;
                        }
                    } else {
                        if self.input_chars[position-1] == self.charset[position-1] {
                            self.ids[position-1] = 1;
                        } else {
                            self.ids[position-1] = 2;
                        }
                        if self.input_chars.len() == 6 {
                            for _ in 0..3 {
                                let _ = self.charset.pop_front();
                                let _ = self.input_chars.pop_front();
                                let _ = self.ids.pop_front();
                                self.charset.push_back(gen_random_ascii_char());
                                self.ids.push_back(0);
                            }
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
        let area = center(
            frame.area(),
            Constraint::Length(9),
            Constraint::Length(3),
        );

        let span: Vec<Span> = self.charset.iter().enumerate().map(|(i, c)| {
            if self.ids[i] == 1 {
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))
            } else if self.ids[i] == 2 {
                Span::styled(self.input_chars[i].to_string(), Style::new().fg(Color::Indexed(9)))
            } else {
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(8)))
            }
        }).collect();
    
        let mut three_lines = vec![];
        for i in 0..3 {
            let line_spans: Vec<Span> = span.iter().skip(i*3).take(3).map(|c| {
                c.clone()
            }).collect();
            let line = Line::from(line_spans);
            let item = ListItem::new(line);
            three_lines.push(item);
        }
        
        let list = List::new(three_lines);
        frame.render_widget(list, area);
        
        }

    // Reads the crossterm events
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => { self.needs_redraw = true; }
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char(c) => {
                self.input_chars.push_back(c.to_string());
                self.needs_redraw = true;
                self.typed = true;
            }
            KeyCode::Backspace => {
                let position = self.input_chars.len();
                if position > 0 {
                    self.input_chars.pop_back();
                    self.ids[position-1] = 0;
                    self.needs_redraw = true;
                }
            }
            _ => {}
        }
    }
    
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
