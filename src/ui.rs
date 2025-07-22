use crate::app::{App, CurrentMode, CurrentTypingOption};
use ratatui::{
    layout::{Alignment, Direction, Flex}, 
    prelude::{Constraint, Layout, Rect}, 
    style::{Color, Style}, 
    text::{Line, Span}, 
    widgets::{Clear, List, ListItem}, 
    Frame
};
use ttypr::get_sorted_mistakes;

// Render the user interface
pub fn render(frame: &mut Frame, app: &App) {
    // If first boot or the help page toggled display only this
    if app.config.first_boot || app.show_help {
        let first_boot_message_area = center(
            frame.area(),
            Constraint::Length(60),
            Constraint::Length(25),
        );

        let first_boot_message = vec![
            Line::from("The application starts in the Menu mode.").alignment(Alignment::Center),
            Line::from(""),
            Line::from(""),
            Line::from("Menu mode:").alignment(Alignment::Center),
            Line::from(""),
            Line::from("            h - access the help page"),
            Line::from("            q - exit the application"),
            Line::from("            i - switch to Typing mode"),
            Line::from("            o - switch Typing option (ASCII, Words, Text)"),
            Line::from("            n - toggle notifications"),
            Line::from("            c - toggle counting mistyped characters"),
            Line::from("            w - display top mistyped characters"),
            Line::from("            r - clear mistyped characters count"),
            Line::from(""),
            Line::from(""),
            Line::from("Typing mode:").alignment(Alignment::Center),
            Line::from(""),
            Line::from("            ESC - switch to Menu mode"),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(Alignment::Center)
        ];

        let first_boot_message: Vec<_> = first_boot_message
            .into_iter()
            .map(ListItem::new)
            .collect();

        let first_boot_message = List::new(first_boot_message);
        frame.render_widget(first_boot_message, first_boot_message_area);

        return;
    }

    // Most mistyped characters display
    if app.show_mistyped {
        let sorted_mistakes = get_sorted_mistakes(&app.config.mistyped_chars);
        let sorted_mistakes: Vec<(String, usize)> = sorted_mistakes.iter().take(15).map(|(k, v)| (k.to_string(), **v)).collect();

        let mut mistake_lines: Vec<ListItem> = vec![];

        let mistyped_title = vec![
            ListItem::new(Line::from("Most mistyped characters")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("")),
        ];
        for item in mistyped_title { mistake_lines.push(item) }

        for (mistake, count) in sorted_mistakes {
            let line = Line::from(format!("{}: {}", mistake, count)).alignment(Alignment::Center);
            mistake_lines.push(ListItem::new(line));
        }

        let enter_button = vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(Alignment::Center)),
        ];
        for item in enter_button { mistake_lines.push(item) }

        let mistakes_area = center(
            frame.area(),
            Constraint::Length(25),
            Constraint::Length(25),
        );

        let list = List::new(mistake_lines);
        frame.render_widget(list, mistakes_area);
        return;
    }

    // Where to display the lines
    let area = center(
        frame.area(), // The area of the entire frame
        Constraint::Length(app.line_len as u16), // Width depending on set line length
        Constraint::Length(5), // Height, 5 - because spaces between them
    );

    // Cleared mistyped characters count display
    if app.notifications.clear_mistyped && app.config.show_notifications {
        let clear_mistyped_notification_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(65),
                Constraint::Percentage(10),
                Constraint::Percentage(25),
            ]).split(frame.area());
        
        frame.render_widget(Line::from("Cleared mistyped characters count").alignment(Alignment::Center), clear_mistyped_notification_area[1]);
    }

    // Mistyped characters count toggle display
    if app.notifications.mistyped && app.config.show_notifications {
        let mistyped_chars_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ]).split(frame.area());

        let mistyped_chars_on = Line::from(vec![Span::from("  Counting mistyped characters "), Span::styled("on", Style::new().fg(Color::Green))]).alignment(Alignment::Center);
        let mistyped_chars_off = Line::from(vec![Span::from("  Counting mistyped characters "), Span::styled("off", Style::new().fg(Color::Red))]).alignment(Alignment::Center);

        if app.config.save_mistyped {
            frame.render_widget(mistyped_chars_on, mistyped_chars_area[1]);
        } else {
            frame.render_widget(mistyped_chars_off, mistyped_chars_area[1]);
        }
    }

    // Notification toggle display
    if app.notifications.toggle {
        let notification_toggle_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ]).split(frame.area());
        let notification_toggle_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(25),
                Constraint::Min(0),
            ]).split(notification_toggle_area[1]);

        let notifications_on = Line::from(vec![Span::from("  Notifications "), Span::styled("on", Style::new().fg(Color::Green))]).alignment(Alignment::Left);
        let notifications_off = Line::from(vec![Span::from("  Notifications "), Span::styled("off", Style::new().fg(Color::Red))]).alignment(Alignment::Left);

        if app.config.show_notifications {
            frame.render_widget(notifications_on, notification_toggle_area[0]);
        } else {
            frame.render_widget(notifications_off, notification_toggle_area[0]);
        }
    }

    // Typing mode selection display (Menu, Typing)
    if app.notifications.mode && app.config.show_notifications {
        let mode_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(60),
                Constraint::Percentage(10),
                Constraint::Percentage(30),
            ]).split(frame.area());
        
        match app.current_mode {
            CurrentMode::Menu => {
                frame.render_widget(Line::from("- Menu mode -").alignment(Alignment::Center), mode_area[1]);
            }
            CurrentMode::Typing => {
                frame.render_widget(Line::from("- Typing mode -").alignment(Alignment::Center), mode_area[1]);
            }
        }
    }
    
    // Typing option selection display (Ascii, Words, Text)
    if app.notifications.option && app.config.show_notifications {
        let option_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Min(0),
            ]).split(frame.area());
        let option_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(5),
            ]).split(option_area[1]);

        let mut option_span: Vec<ListItem> = vec![];

        match app.current_typing_option {
            CurrentTypingOption::Ascii => {
                option_span.push(ListItem::new(Span::styled("Ascii", Style::new().fg(Color::Black).bg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Words", Style::new().fg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Text", Style::new().fg(Color::White))));
            }
            CurrentTypingOption::Words => {
                option_span.push(ListItem::new(Span::styled("Ascii", Style::new().fg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Words", Style::new().fg(Color::Black).bg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Text", Style::new().fg(Color::White))));
            }
            CurrentTypingOption::Text => {
                option_span.push(ListItem::new(Span::styled("Ascii", Style::new().fg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Words", Style::new().fg(Color::White))));
                option_span.push(ListItem::new(Span::styled("Text", Style::new().fg(Color::Black).bg(Color::White))));
            }
        }
        
        frame.render_widget(List::new(option_span), option_area[1]);
    }

    // A vector of colored characters
    let span: Vec<Span> = app.charset.iter().enumerate().map(|(i, c)| {

        // If inputted character matches charset character add a green colored character that user inputted
        if app.ids[i] == 1 {
            Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))

        // If inputted character doesn't match charset character add a red colored character that user inputted
        } else if app.ids[i] == 2 {
            // If incorrectly inputted character is a space - add a "_" instead
            if app.input_chars[i] == " " {
                Span::styled("_".to_string(), Style::new().fg(Color::Indexed(9)))
            } else {
                // If incorrectly typed a space - add a "_" instead
                if app.charset[i] == " " {
                    Span::styled("_".to_string(), Style::new().fg(Color::Indexed(9)))
                // Otherwise add a red colored mistyped character
                } else {
                    Span::styled(app.charset[i].to_string(), Style::new().fg(Color::Indexed(9)))
                }
            }

        // Otherwise add a grey colored character (hasn't been typed yet)
        } else {
            Span::styled(c.to_string(), Style::new().fg(Color::Indexed(8)))
        }
    }).collect();

    // Draw the typing area itself
    match app.current_typing_option {
        CurrentTypingOption::Ascii => {
            // Separating vector of all the colored characters into vector of 3 lines, each line_len long
            // and making them List items, to display as a List widget
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
        CurrentTypingOption::Words => {
            // If no words file provided
            if app.words.len() == 0 {
                let area = center(
                    frame.area(),
                    Constraint::Length(50),
                    Constraint::Length(15),
                );

                let no_words_message = vec![
                    Line::from("In order to use the Words typing option").alignment(Alignment::Center),
                    Line::from("you need to have a:").alignment(Alignment::Center),
                    Line::from(""), // Push an empty space to separate lines
                    Line::from("~/.config/ttypr/words.txt").alignment(Alignment::Center),
                    Line::from(""),
                    Line::from("The formatting is just words separated by spaces").alignment(Alignment::Center),
                    Line::from(""),
                    Line::from("Or you can use the default one by pressing Enter").alignment(Alignment::Center),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(Alignment::Center)
                ];

                let no_words_message: Vec<_> = no_words_message
                    .into_iter()
                    .map(ListItem::new)
                    .collect();

                let no_words_message = List::new(no_words_message);
                frame.render_widget(no_words_message, area);
            } else {
                // Separating vector of all the colored characters into vector of 3 lines, each line_len long
                // and making them List items, to display as a List widget
                let mut three_lines = vec![];
                let mut skip_len = 0;
                for i in 0..3 {
                    let line_span: Vec<Span> = span.iter().skip(skip_len).take(app.lines_len[i]).map(|c| {
                        c.clone()
                    }).collect();
                    let line = Line::from(line_span).alignment(Alignment::Center);
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
        CurrentTypingOption::Text => {
            if app.text.len() == 0 {
                let area = center(
                    frame.area(),
                    Constraint::Length(50),
                    Constraint::Length(15),
                );

                let no_text_message = vec![
                    Line::from("In order to use the Text typing option").alignment(Alignment::Center),
                    Line::from("you need to have a:").alignment(Alignment::Center),
                    Line::from(""), // Push an empty space to separate lines
                    Line::from("~/.config/ttypr/text.txt").alignment(Alignment::Center),
                    Line::from(""),
                    Line::from("Or you can use the default one by pressing Enter").alignment(Alignment::Center),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(Alignment::Center)
                ];

                let no_text_message: Vec<_> = no_text_message
                    .into_iter()
                    .map(ListItem::new)
                    .collect();

                let no_words_message = List::new(no_text_message);
                frame.render_widget(no_words_message, area);
            } else {
                let mut three_lines = vec![];
                let mut skip_len = 0;
                for i in 0..3 {
                    let line_span: Vec<Span> = span.iter().skip(skip_len).take(app.lines_len[i]).map(|c| {
                        c.clone()
                    }).collect();
                    let line = Line::from(line_span).alignment(Alignment::Center);
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