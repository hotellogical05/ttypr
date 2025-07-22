use std::{collections::HashMap, fs, io};
use rand::Rng;
use serde::{ser::SerializeMap, Serialize, Deserialize, Serializer};
use sha2::{Sha256, Digest};

// Config struct to store all config values, is a part of the App struct
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub first_boot: bool,
    pub show_notifications: bool,
    #[serde(serialize_with = "serialize_sorted_by_value")]
    pub mistyped_chars: HashMap<String, usize>,
    pub save_mistyped: bool,
    pub skip_len: usize,
    pub use_default_word_set: bool,
    pub use_default_text_set: bool,
    pub last_text_txt_hash: Option<Vec<u8>>,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            first_boot: true, 
            show_notifications: true,
            mistyped_chars: HashMap::new(),
            save_mistyped: true,
            skip_len: 0, // (For the text option) - To save position in the text
            use_default_word_set: false,
            use_default_text_set: false,
            last_text_txt_hash: None,
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

// Loads config from .config/ttypr/config
// If it doesn't exist - creates it with default values
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

// Saves the config to .config/ttypr/config
// (the fields in the Config struct)
pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let config_path = home_path.join(".config/ttypr/config");
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(config_path, toml_string)?;
    Ok(())
}

// Reads the contents of .config/ttypr/words.txt
// parses and returns it
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

// Reads the contents of .config/ttypr/text.txt
// parses and returns it
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

// Just returns the default words set in a vector
pub fn default_words() -> Vec<String> {
    let default_words = vec!["the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now", "look", "only", "come", "over", "think", "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any", "these", "give", "day", "most", "us", "thing", "man", "find", "part", "eye", "place", "week", "case", "point", "government", "company", "number", "group", "problem", "fact", "leave", "while", "mean", "keep", "student", "great", "seem", "same", "tell", "begin", "help", "talk", "where", "turn", "start", "might", "show", "hear", "play", "run", "move", "live", "believe", "hold", "bring", "happen", "must", "write", "provide", "sit", "stand", "lose", "pay", "meet", "include", "continue", "set", "learn", "change", "lead", "understand", "watch", "follow", "stop", "create", "speak", "read", "allow", "add", "spend", "grow", "open", "walk", "win", "offer", "remember", "love", "consider", "appear", "buy", "wait", "serve", "die", "send", "expect", "build", "stay", "fall", "cut", "reach", "kill", "remain", "suggest", "raise", "pass", "sell", "require", "report", "decide", "pull", "return", "explain", "hope", "develop", "carry", "break", "receive", "agree", "support", "hit", "produce", "eat", "cover", "catch", "draw", "choose", "cause", "listen", "maybe", "until", "without", "probably", "around", "small", "green", "special", "difficult", "available", "likely", "short", "single", "medical", "current", "wrong", "private", "past", "foreign", "fine", "common", "poor", "natural", "significant", "similar", "hot", "dead", "central", "happy", "serious", "ready", "simple", "left", "physical", "general", "environmental", "financial", "blue", "democratic", "dark", "various", "entire", "close", "legal", "religious", "cold", "final", "main", "huge", "popular", "traditional", "cultural", "choice", "high", "big", "large", "particular", "tiny", "enormous"];
    let default_words: Vec<String> = default_words
        .iter()
        .map(|w| w.to_string())
        .collect();
    default_words
}

// Just returns the default sentences (a vector of words and punctuation)
pub fn default_text() -> Vec<String> {
    let default_text = vec!["The", "shimmering", "dragonfly", "hovered", "over", "the", "tranquil", "pond.", "Ancient", "mountains", "guard", "secrets", "of", "a", "time", "long", "forgotten.", "A", "melancholic", "melody", "drifted", "from", "the", "old,", "forgotten", "gramophone.", "The", "bustling", "city", "market", "was", "a", "kaleidoscope", "of", "colors,", "sounds,", "and", "smells.", "Through", "the", "fog,", "a", "lone", "lighthouse", "cast", "a", "guiding", "beam", "for", "lost", "sailors.", "The", "philosopher", "pondered", "the", "intricate", "dance", "between", "fate", "and", "free", "will.", "A", "child's", "laughter", "echoed", "in", "the", "empty", "playground,", "a", "ghost", "of", "happier", "times.", "The", "weathered", "fisherman", "mended", "his", "nets,", "his", "face", "a", "map", "of", "the", "sea.", "Cryptic", "symbols", "adorned", "the", "walls", "of", "the", "newly", "discovered", "tomb.", "The", "scent", "of", "rain", "on", "dry", "earth", "filled", "the", "air,", "a", "promise", "of", "renewal.", "A", "weary", "traveler", "sought", "refuge", "from", "the", "relentless", "storm", "in", "a", "deserted", "cabin.", "The", "artist's", "canvas", "held", "a", "chaotic", "explosion", "of", "emotions,", "rendered", "in", "oil", "and", "acrylic.", "Stars,", "like", "scattered", "diamonds,", "adorned", "the", "velvet", "canvas", "of", "the", "night", "sky.", "The", "old", "librarian", "cherished", "the", "silent", "companionship", "of", "his", "leather-bound", "books.", "A", "forgotten", "diary", "revealed", "the", "secret", "love", "story", "of", "a", "bygone", "era.", "The", "chef", "meticulously", "arranged", "the", "dish,", "transforming", "food", "into", "a", "work", "of", "art.", "In", "the", "heart", "of", "the", "forest,", "a", "hidden", "waterfall", "cascaded", "into", "a", "crystal-clear", "pool.", "The", "politician's", "speech", "was", "a", "carefully", "constructed", "fortress", "of", "half-truths", "and", "promises.", "A", "sudden", "gust", "of", "wind", "scattered", "the", "autumn", "leaves", "like", "a", "flurry", "of", "colorful", "confetti.", "The", "detective", "followed", "a", "labyrinthine", "trail", "of", "clues,", "each", "one", "more", "perplexing", "than", "the", "last.", "The", "scent", "of", "jasmine", "hung", "heavy", "in", "the", "humid", "evening", "air.", "Time", "seemed", "to", "slow", "down", "in", "the", "sleepy,", "sun-drenched", "village.", "The", "blacksmith's", "hammer", "rang", "out", "a", "rhythmic", "chorus", "against", "the", "glowing", "steel.", "A", "lone", "wolf", "howled", "at", "the", "full", "moon,", "its", "call", "a", "lament", "for", "its", "lost", "pack.", "The", "mathematician", "found", "elegance", "and", "beauty", "in", "the", "complex", "simplicity", "of", "equations.", "From", "the", "ashes", "of", "defeat,", "a", "spark", "of", "resilience", "began", "to", "glow.", "The", "antique", "clock", "ticked", "with", "a", "solemn,", "unhurried", "rhythm,", "marking", "the", "passage", "of", "time.", "A", "hummingbird,", "a", "jeweled", "marvel", "of", "nature,", "darted", "from", "flower", "to", "flower.", "The", "decrepit", "mansion", "on", "the", "hill", "was", "rumored", "to", "be", "haunted", "by", "a", "benevolent", "spirit.", "Sunlight", "streamed", "through", "the", "stained-glass", "windows,", "painting", "the", "cathedral", "floor", "in", "vibrant", "hues.", "The", "aroma", "of", "freshly", "baked", "bread", "wafted", "from", "the", "cozy", "little", "bakery.", "A", "complex", "network", "of", "roots", "anchored", "the", "ancient", "oak", "tree", "to", "the", "earth.", "The", "programmer", "stared", "at", "the", "screen,", "searching", "for", "the", "single,", "elusive", "bug", "in", "a", "million", "lines", "of", "code.", "The", "waves", "crashed", "against", "the", "rocky", "shore", "in", "a", "timeless,", "powerful", "rhythm.", "A", "flock", "of", "geese", "flew", "south", "in", "a", "perfect", "V-formation,", "a", "testament", "to", "their", "instinctual", "harmony.", "The", "historian", "pieced", "together", "the", "fragments", "of", "the", "past", "to", "tell", "a", "coherent", "story.", "In", "the", "quiet", "solitude", "of", "the", "desert,", "one", "could", "hear", "the", "whisper", "of", "the", "wind.", "The", "gardener", "tended", "to", "her", "roses", "with", "a", "gentle,", "nurturing", "touch.", "A", "crackling", "fireplace", "provided", "a", "warm", "and", "inviting", "centerpiece", "to", "the", "rustic", "living", "room.", "The", "mountaineer", "stood", "at", "the", "summit,", "humbled", "by", "the", "breathtaking", "vista", "below.", "A", "single,", "perfect", "snowflake", "landed", "on", "the", "child's", "outstretched", "mitten."];
    let default_text: Vec<String> = default_text
        .iter()
        .map(|w| w.to_string())
        .collect();
    default_text
}

// Calculates the hash of .config/ttypr/text.txt
pub fn calculate_text_txt_hash() -> io::Result<Vec<u8>> {
    // Get the home directory path
    let home_path = home::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;

    // Construct the full path to the text.txt file
    let path = home_path.join(".config/ttypr/text.txt");

    let file_bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    Ok(hasher.finalize().to_vec())
}