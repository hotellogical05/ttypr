use std::{collections::HashMap, fs, io};
use rand::Rng;
use serde::{ser::SerializeMap, Serialize, Deserialize, Serializer};

// Config struct to store all config values, is a part of the App struct
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub first_boot: bool,
    pub show_notifications: bool,
    #[serde(serialize_with = "serialize_sorted_by_value")]
    pub mistyped_chars: HashMap<String, usize>,
    pub save_mistyped: bool,
    pub skip_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            first_boot: true, 
            show_notifications: true,
            mistyped_chars: HashMap::new(),
            save_mistyped: true,
            skip_len: 0,
        }
    }
}

/// Takes a map of mistyped characters and returns them as a list
/// sorted by count (descending) and then character (ascending).
pub fn get_sorted_mistakes(map: &HashMap<String, usize>) -> Vec<(&String, &usize)> {
    let mut sorted: Vec<_> = map.iter().collect();
    // Sort by value (count) descending, then by key (char) ascending for ties.
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    sorted
}

// Custom serializer that uses the reusable sorting logic
fn serialize_sorted_by_value<S>(
    map: &HashMap<String, usize>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let sorted = get_sorted_mistakes(map);
    let mut map_serializer = serializer.serialize_map(Some(sorted.len()))?;
    for (key, value) in sorted {
        map_serializer.serialize_entry(key, value)?;
    }
    map_serializer.end()
}

// Generate a random ascii character
pub fn gen_random_ascii_char() -> String {
    let charset = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "~", "`", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "_", "+", "=", "{", "}", "[", "]", "|", "\\", ":", ";", "\"", "'", "<", ">", ",", ".", "?", "/"];
    let index = rand::rng().random_range(0..charset.len());
    let character = charset[index];
    character.to_string()
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Get the home directory path
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;

    // Construct the full path to the config file
    let config_path = home_path.join(".config/ttypr/config");

    // Create the directory if it doesn't exist
    fs::create_dir_all(home_path.join(".config/ttypr/"))?;

    // Check if file exists
    if !config_path.exists() {
        // If not, create it with default values
        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config)?;
        fs::write(&config_path, toml_string)?;
        return Ok(default_config);
    }

    // If it does exist, read, parse and return it
    let config_string = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_string)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let config_path = home_path.join(".config/ttypr/config");
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(config_path, toml_string)?;
    Ok(())
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

pub fn read_text_from_file() -> io::Result<Vec<String>> {
    // Get the home directory path
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;

    // Construct the full path to the text.txt file
    let config_path = home_path.join(".config/ttypr/text.txt");

    let content = fs::read_to_string(config_path)?;
    let text = content
        .split_whitespace()
        .map(String::from)
        .collect();
    Ok(text)
}
