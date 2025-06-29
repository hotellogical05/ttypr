use crate::app::App;
use ratatui::{
    Frame,
    prelude::{Rect, Constraint, Layout},
    layout::Flex,
    style::{Color, Style},
    widgets::{List, ListItem},
    text::{Line, Span},
};

// Render the user interface
pub fn render(frame: &mut Frame, app: &App) {
    // where to display the lines
    let area = center(
        frame.area(),
        Constraint::Length(app.line_len as u16), // width depending on set line length
        Constraint::Length(5), // height, 5 - because spaces between them
    );

    // a vector of colored characters
    let span: Vec<Span> = app.charset.iter().enumerate().map(|(i, c)| {
        // if inputted character matches charset character add a green colored character that user inputted
        if app.ids[i] == 1 {
            Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))
        // if inputted character doesn't match charset character add a red colored character that user inputted
        } else if app.ids[i] == 2 { // 
            Span::styled(app.input_chars[i].to_string(), Style::new().fg(Color::Indexed(9)))
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
        let line_span: Vec<Span> = span.iter().skip(i*app.line_len).take(app.line_len).map(|c| {
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

// Helper function to center a layout area
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}