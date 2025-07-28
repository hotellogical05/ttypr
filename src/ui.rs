use crate::app::{App, CurrentMode, CurrentTypingOption};
use ratatui::{
    layout::{Alignment, Direction, Flex}, 
    prelude::{Constraint, Layout, Rect}, 
    style::{Color, Style}, 
    text::{Line, Span}, 
    widgets::{Clear, List, ListItem}, 
    Frame
};
use crate::utils::{get_sorted_mistakes};

/// Renders the entire user interface based on the application's current state.
///
/// This function acts as a dispatcher, determining which screen to render based on the app's
/// state flags like `first_boot`, `show_help`, and `show_mistyped`.
pub fn render(frame: &mut Frame, app: &App) {
    if app.config.first_boot || app.show_help {
        render_help_screen(frame);
        return;
    }

    if app.show_mistyped {
        render_mistakes_screen(frame, app);
        return;
    }

    render_main_ui(frame, app);
}

/// Renders the main user interface, including the typing area and notifications.
fn render_main_ui(frame: &mut Frame, app: &App) {
    // Where to display the lines
    let area = center(
        frame.area(), // The area of the entire frame
        Constraint::Length(app.line_len as u16), // Width depending on set line length
        Constraint::Length(5), // Height, 5 - because spaces between them
    );

    render_notifications(frame, app);
    render_typing_area(frame, app, area);
}

/// Renders the help screen, which displays keybindings and instructions.
///
/// This screen is shown on the first boot or when the user explicitly requests it.
fn render_help_screen(frame: &mut Frame) {
    let first_boot_message_area = center(
        frame.area(),
        Constraint::Length(65),
        Constraint::Length(30),
    );

    let first_boot_message = vec![
        Line::from("The application starts in the Menu mode.").alignment(Alignment::Center),
        Line::from(""),
        Line::from("For larger font - increase the terminal font size.").alignment(Alignment::Center),
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
        Line::from("            Character keys - Type the corresponding characters"),
        Line::from("            Backspace - Remove characters"),
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
}

/// Renders the screen displaying the user's most frequently mistyped characters.
fn render_mistakes_screen(frame: &mut Frame, app: &App) {
    let sorted_mistakes = get_sorted_mistakes(&app.config.mistyped_chars);
    // Limit the display to the top 15 most frequent mistakes.
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
}

/// Renders transient notifications at various positions on the screen.
///
/// These notifications provide feedback for actions like toggling settings, changing modes, etc.
fn render_notifications(frame: &mut Frame, app: &App) {
    // WPM display
    if app.notifications.wpm {
        let wpm_notification_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Min(1),
                Constraint::Min(0),
            ]).split(frame.area());
        let wpm_notification_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(60),
                Constraint::Length(10),
                Constraint::Min(0),
            ]).split(wpm_notification_area[1]);

        frame.render_widget(Line::from(format!("{} wpm", app.wpm.wpm)), wpm_notification_area[1]);
    }

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
        // Position the typing option selector in the top-right corner.
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
}

/// Renders the core typing area where the user practices.
///
/// This function handles the display of the character set, user input, and messages for
/// missing word/text files.
fn render_typing_area(frame: &mut Frame, app: &App, area: Rect) {
    // A vector of colored characters
    let span: Vec<Span> = app.charset.iter().enumerate().map(|(i, c)| {
        match app.ids[i] {
            1 => { // Correct
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(10)))
            }
            2 => { // Incorrect
                // Render incorrect spaces as underscores for better visibility.
                let char_to_render = if app.input_chars[i] == " " || c == " " {
                    "_"
                } else {
                    c
                };
                Span::styled(char_to_render.to_string(), Style::new().fg(Color::Indexed(9)))
            }
            _ => { // Untyped
                Span::styled(c.to_string(), Style::new().fg(Color::Indexed(8)))
            }
        }
    }).collect();

    // Draw the typing area itself
    match app.current_typing_option {
        CurrentTypingOption::Ascii => {
            render_typing_lines(frame, app, area, span);
        }
        CurrentTypingOption::Words => {
            if app.words.is_empty() {
                render_file_not_found_message(frame, "Words", "~/.config/ttypr/words.txt", Some("The formatting is just words separated by spaces"));
            } else {
                render_typing_lines(frame, app, area, span);
            }
        }
        CurrentTypingOption::Text => {
            if app.text.is_empty() {
                render_file_not_found_message(frame, "Text", "~/.config/ttypr/text.txt", None);
            } else {
                render_typing_lines(frame, app, area, span);
            }        
        }
    } 
}

/// Renders a message indicating that a required file (e.g., for words or text) was not found.
///
/// # Arguments
///
/// * `frame` - The mutable frame to draw on.
/// * `option_name` - The name of the typing option (e.g., "Words", "Text").
/// * `file_path` - The expected path of the missing file.
/// * `extra_line` - An optional extra line of context, like formatting instructions.
fn render_file_not_found_message(frame: &mut Frame, option_name: &str, file_path: &str, extra_line: Option<&str>) {
    let area = center(
        frame.area(),
        Constraint::Length(50),
        Constraint::Length(15),
    );

    let mut message_lines = vec![
        Line::from(format!("In order to use the {} typing option", option_name)).alignment(Alignment::Center),
        Line::from("you need to have a:").alignment(Alignment::Center),
        Line::from(""), // Push an empty space to separate lines
        Line::from(file_path).alignment(Alignment::Center),
        Line::from(""),
    ];

    if let Some(line) = extra_line {
        message_lines.push(Line::from(line).alignment(Alignment::Center));
        message_lines.push(Line::from(""));
    }

    message_lines.extend(vec![
        Line::from("Or you can use the default one by pressing Enter").alignment(Alignment::Center),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("<Enter>", Style::new().bg(Color::White).fg(Color::Black))).alignment(Alignment::Center)
    ]);

    let list_items: Vec<_> = message_lines
        .into_iter()
        .map(ListItem::new)
        .collect();

    let list = List::new(list_items);
    frame.render_widget(list, area);
}

/// Renders the lines of text for the user to type.
///
/// This function takes the application state, a frame, a rendering area, and a vector of styled
/// characters (`Span`s). It then splits the characters into three lines and displays them
/// centered in the provided area.
pub fn render_typing_lines(frame: &mut Frame, app: &App, area: Rect, span: Vec<Span>) {
    // Separating vector of all the colored characters into vector of 3 lines, each line_len long
    // and making them List items, to display as a List widget
    let mut three_lines = vec![];
    let mut skip_len = 0;
    // The UI displays three lines of text at a time.
    for i in 0..3 {
        // Use `skip()` and `take()` to create a view into the full character buffer for each line.
        let line_span: Vec<Span> = span.iter().skip(skip_len).take(app.lines_len[i]).map(|c| {
            c.clone()
        }).collect();
        let line = Line::from(line_span).alignment(Alignment::Center);
        let item = ListItem::new(line);
        three_lines.push(item);
        // Add an empty `ListItem` to create visual spacing between the lines.
        three_lines.push(ListItem::new(""));
        skip_len += app.lines_len[i];
    }

    // Make a List widget out of list items and render it in the middle
    let list = List::new(three_lines);
    frame.render_widget(list, area);
}

/// Helper function to center a layout area
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// Clear the entire area
pub fn draw_on_clear(f: &mut Frame) {
    let area = f.area(); // The area of the entire frame
    f.render_widget(Clear, area);
}