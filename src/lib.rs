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
    let content = fs::read_to_string("words.txt")?;
    let words = content
        .split_whitespace() // Split the string into an iterator of string slices (&str)
        .map(String::from) // Convert each string slice into an owned String
                           // this is because the original string read from the file will
                           // go out of scope, and we need the vector to own its data
        .collect(); // Collect the Strings into a Vec<String>
    Ok(words)
}

pub fn word_line_len() {
    // app.line_len
}




//    let words = match read_words_from_file("words.txt") {
//        Ok(words) => words,
//        Err(e) => {
//            eprintln!("Error reading words file: {}", e);
//            // You can decide how to handle the error,
//            // for example, by exiting the program.
//            std::process::exit(1);
//        }
//    };