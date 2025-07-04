use std::{io, fs};
use rand::Rng;

// Generate a random ascii character
pub fn gen_random_ascii_char() -> String {
    let charset = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "~", "`", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "_", "+", "=", "{", "}", "[", "]", "|", "\\", ":", ";", "\"", "'", "<", ">", ",", ".", "?", "/"];
    let index = rand::rng().random_range(0..charset.len());
    let character = charset[index];
    character.to_string()
}
    
pub fn read_words_from_file() -> io::Result<Vec<String>> {
    // Get the home directory path
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;

    // Construct the full path to the words.txt file
    let config_path = home_path.join(".config/ttypr/words.txt");

    let content = fs::read_to_string(config_path)?;
    let words = content
        .split_whitespace() // Split the string into an iterator of string slices (&str)
        .map(String::from) // Convert each string slice into an owned String
                           // this is because the original string read from the file will
                           // go out of scope, and we need the vector to own its data
        .collect(); // Collect the Strings into a Vec<String>
    Ok(words)
}

pub fn gen_one_line_of_words(line_len: usize, words: &Vec<String>) -> String {
    let mut line_of_words = vec![];
    loop {
        let index = rand::rng().random_range(0..words.len());
        let word = words[index].clone();
        line_of_words.push(word);

        let current_line_len = line_of_words.join(" ").chars().count();

        if current_line_len > line_len {
            line_of_words.pop();
            let current_line = line_of_words.join(" ");
            return current_line; 
        };
    };
}