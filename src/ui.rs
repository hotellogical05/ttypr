use crate::app::{App, CurrentTypingMode};
use ratatui::{
    layout::Flex, 
    prelude::{Constraint, Layout, Rect}, 
    style::{Color, Style}, 
    text::{Line, Span}, 
    widgets::{Clear, List, ListItem}, 
    Frame
};

// Render the user interface
pub fn render(frame: &mut Frame, app: &App) {
    // Where to display the lines
    let area = center(
        frame.area(), // The area of the entire frame
        Constraint::Length(app.line_len as u16), // Width depending on set line length
        Constraint::Length(5), // Height, 5 - because spaces between them
    );

    // A vector of colored characters
    let span: Vec<Span> = app.charset.iter().enumerate().map(|(i, c)| {
        // If inputted character matches charset character add a green colored character that user inputted
        if app.ids[i] == 1 {
            Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))
        // If inputted character doesn't match charset character add a red colored character that user inputted
        } else if app.ids[i] == 2 {
            if app.input_chars[i] == " " {
                Span::styled("_".to_string(), Style::new().fg(Color::Indexed(9)))
            } else {
                Span::styled(app.input_chars[i].to_string(), Style::new().fg(Color::Indexed(9)))
            }
        // Otherwise add a grey colored character (hasn't been typed yet)
        } else {
            Span::styled(c.to_string(), Style::new().fg(Color::Indexed(8)))
        }
    }).collect();

    match app.current_typing_mode {
        CurrentTypingMode::Ascii => {
            // Separating vector of all the colored characters into vector of 3 lines, each line_len long
            // and making them List itelet block = Block::bordered().title("Block");ms, to display as a List widget
            let mut three_lines = vec![];
            for i in 0..3 {
                // Skip 0, 1, 2 lines, take line length of characters, and make a vector out of them
                let line_span: Vec<Span> = span.iter().skip(i*app.line_len).take(app.line_len).map(|c| {
                    c.clone()
                }).collect();
                let line = Line::from(line_span);
                let item = ListItem::new(line);
                three_lines.push(item); // Push the line
                three_lines.push(ListItem::new("")); // Push an empty space to separate lines
            }

            // Make a List widget out of list items and render it in the middle
            let list = List::new(three_lines);
            frame.render_widget(list, area);
        }
        CurrentTypingMode::Words => {
            if app.words.len() == 0 {
                let area = center(
                    frame.area(),
                    Constraint::Length(50),
                    Constraint::Length(15),
                );

                let no_words_message = vec![
                    Line::from("In order to use the Words typing option").alignment(ratatui::layout::Alignment::Center),
                    Line::from("you need to have a:").alignment(ratatui::layout::Alignment::Center),
                    Line::from(""), // Push an empty space to separate lines
                    Line::from("~/.config/ttypr/words.txt").alignment(ratatui::layout::Alignment::Center),
                    Line::from(""),
                    Line::from("The formatting is just words separated by spaces").alignment(ratatui::layout::Alignment::Center),
                    Line::from(""),
                    Line::from("Or you can use the default one by pressing Enter").alignment(ratatui::layout::Alignment::Center),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(ratatui::layout::Alignment::Center)
                ];

                // * Understand what happens here:
                let no_words_message: Vec<_> = no_words_message
                    .into_iter()
                    .map(ListItem::new)
                    .collect();

                let no_words_message = List::new(no_words_message);
                frame.render_widget(no_words_message, area);
            } else {
                // Separating vector of all the colored characters into vector of 3 lines, each line_len long
                // and making them List itelet block = Block::bordered().title("Block");ms, to display as a List widget
                let mut three_lines = vec![];
                let mut skip_len = 0;
                for i in 0..3 {
                    let line_span: Vec<Span> = span.iter().skip(skip_len).take(app.lines_len[i]).map(|c| {
                        c.clone()
                    }).collect();
                    let line = Line::from(line_span).alignment(ratatui::layout::Alignment::Center);
                    let item = ListItem::new(line);
                    three_lines.push(item); // Push the line
                    three_lines.push(ListItem::new("")); // Push an empty space to separate lines
                    skip_len += app.lines_len[i];
                }
            
                // Make a List widget out of list items and render it in the middle
                let list = List::new(three_lines);
                frame.render_widget(list, area);
            }
        }
    } 
}

// Helper function to center a layout area
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

// Clear the entire area
pub fn draw_on_clear(f: &mut Frame) {
    let area = f.area(); // The area of the entire frame
    f.render_widget(Clear, area);
}