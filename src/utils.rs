use std::{collections::HashMap, fs, io, path::{Path, PathBuf}};
use serde::{ser::SerializeMap, Serialize, Deserialize, Serializer};
use sha2::{Sha256, Digest};

/// Config struct to store all config values, is a part of the App struct
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

/// Custom serializer that uses the reusable sorting logic
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

/// Gets the application's configuration directory path.
pub fn get_config_dir() -> io::Result<PathBuf> {
    home::home_dir()
        .map(|path| path.join(".config/ttypr"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))
}

/// Loads config from a specified directory.
/// If it doesn't exist, it creates a default config file.
pub fn load_config(config_dir: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("config");

    // Create the directory if it doesn't exist
    fs::create_dir_all(config_dir)?;

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

/// Saves the config to a specified directory.
pub fn save_config(config: &Config, config_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config_dir.join("config");
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(config_path, toml_string)?;
    Ok(())
}

/// Loads a list of items from a given file in a specified directory.
fn load_items_from_file(dir: &Path, filename: &str) -> io::Result<Vec<String>> {
    let file_path = dir.join(filename);
    let content = fs::read_to_string(file_path)?;
    let items = content
        .split_whitespace()
        .map(String::from)
        .collect();
    Ok(items)
}

/// Reads the contents of words.txt from a specified directory.
pub fn read_words_from_file(dir: &Path) -> io::Result<Vec<String>> {
    load_items_from_file(dir, "words.txt")
}

/// Reads the contents of text.txt from a specified directory.
pub fn read_text_from_file(dir: &Path) -> io::Result<Vec<String>> {
    load_items_from_file(dir, "text.txt")
}

/// Just returns the default words set in a vector
pub fn default_words() -> Vec<String> {
    let default_words = vec!["the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now", "look", "only", "come", "over", "think", "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any", "these", "give", "day", "most", "us", "thing", "man", "find", "part", "eye", "place", "week", "case", "point", "government", "company", "number", "group", "problem", "fact", "leave", "while", "mean", "keep", "student", "great", "seem", "same", "tell", "begin", "help", "talk", "where", "turn", "start", "might", "show", "hear", "play", "run", "move", "live", "believe", "hold", "bring", "happen", "must", "write", "provide", "sit", "stand", "lose", "pay", "meet", "include", "continue", "set", "learn", "change", "lead", "understand", "watch", "follow", "stop", "create", "speak", "read", "allow", "add", "spend", "grow", "open", "walk", "win", "offer", "remember", "love", "consider", "appear", "buy", "wait", "serve", "die", "send", "expect", "build", "stay", "fall", "cut", "reach", "kill", "remain", "suggest", "raise", "pass", "sell", "require", "report", "decide", "pull", "return", "explain", "hope", "develop", "carry", "break", "receive", "agree", "support", "hit", "produce", "eat", "cover", "catch", "draw", "choose", "cause", "listen", "maybe", "until", "without", "probably", "around", "small", "green", "special", "difficult", "available", "likely", "short", "single", "medical", "current", "wrong", "private", "past", "foreign", "fine", "common", "poor", "natural", "significant", "similar", "hot", "dead", "central", "happy", "serious", "ready", "simple", "left", "physical", "general", "environmental", "financial", "blue", "democratic", "dark", "various", "entire", "close", "legal", "religious", "cold", "final", "main", "huge", "popular", "traditional", "cultural", "choice", "high", "big", "large", "particular", "tiny", "enormous"];
    default_words.iter().map(|s| s.to_string()).collect()
}

/// Just returns the default sentences (a vector of words and punctuation)
pub fn default_text() -> Vec<String> {
    let default_text = vec!["The", "shimmering", "dragonfly", "hovered", "over", "the", "tranquil", "pond.", "Ancient", "mountains", "guard", "secrets", "of", "a", "time", "long", "forgotten.", "A", "melancholic", "melody", "drifted", "from", "the", "old,", "forgotten", "gramophone.", "The", "bustling", "city", "market", "was", "a", "kaleidoscope", "of", "colors,", "sounds,", "and", "smells.", "Through", "the", "fog,", "a", "lone", "lighthouse", "cast", "a", "guiding", "beam", "for", "lost", "sailors.", "The", "philosopher", "pondered", "the", "intricate", "dance", "between", "fate", "and", "free", "will.", "A", "child's", "laughter", "echoed", "in", "the", "empty", "playground,", "a", "ghost", "of", "happier", "times.", "The", "weathered", "fisherman", "mended", "his", "nets,", "his", "face", "a", "map", "of", "the", "sea.", "Cryptic", "symbols", "adorned", "the", "walls", "of", "the", "newly", "discovered", "tomb.", "The", "scent", "of", "rain", "on", "dry", "earth", "filled", "the", "air,", "a", "promise", "of", "renewal.", "A", "weary", "traveler", "sought", "refuge", "from", "the", "relentless", "storm", "in", "a", "deserted", "cabin.", "The", "artist's", "canvas", "held", "a", "chaotic", "explosion", "of", "emotions,", "rendered", "in", "oil", "and", "acrylic.", "Stars,", "like", "scattered", "diamonds,", "adorned", "the", "velvet", "canvas", "of", "the", "night", "sky.", "The", "old", "librarian", "cherished", "the", "silent", "companionship", "of", "his", "leather-bound", "books.", "A", "forgotten", "diary", "revealed", "the", "secret", "love", "story", "of", "a", "bygone", "era.", "The", "chef", "meticulously", "arranged", "the", "dish,", "transforming", "food", "into", "a", "work", "of", "art.", "In", "the", "heart", "of", "the", "forest,", "a", "hidden", "waterfall", "cascaded", "into", "a", "crystal-clear", "pool.", "The", "politician's", "speech", "was", "a", "carefully", "constructed", "fortress", "of", "half-truths", "and", "promises.", "A", "sudden", "gust", "of", "wind", "scattered", "the", "autumn", "leaves", "like", "a", "flurry", "of", "colorful", "confetti.", "The", "detective", "followed", "a", "labyrinthine", "trail", "of", "clues,", "each", "one", "more", "perplexing", "than", "the", "last.", "The", "scent", "of", "jasmine", "hung", "heavy", "in", "the", "humid", "evening", "air.", "Time", "seemed", "to", "slow", "down", "in", "the", "sleepy,", "sun-drenched", "village.", "The", "blacksmith's", "hammer", "rang", "out", "a", "rhythmic", "chorus", "against", "the", "glowing", "steel.", "A", "lone", "wolf", "howled", "at", "the", "full", "moon,", "its", "call", "a", "lament", "for", "its", "lost", "pack.", "The", "mathematician", "found", "elegance", "and", "beauty", "in", "the", "complex", "simplicity", "of", "equations.", "From", "the", "ashes", "of", "defeat,", "a", "spark", "of", "resilience", "began", "to", "glow.", "The", "antique", "clock", "ticked", "with", "a", "solemn,", "unhurried", "rhythm,", "marking", "the", "passage", "of", "time.", "A", "hummingbird,", "a", "jeweled", "marvel", "of", "nature,", "darted", "from", "flower", "to", "flower.", "The", "decrepit", "mansion", "on", "the", "hill", "was", "rumored", "to", "be", "haunted", "by", "a", "benevolent", "spirit.", "Sunlight", "streamed", "through", "the", "stained-glass", "windows,", "painting", "the", "cathedral", "floor", "in", "vibrant", "hues.", "The", "aroma", "of", "freshly", "baked", "bread", "wafted", "from", "the", "cozy", "little", "bakery.", "A", "complex", "network", "of", "roots", "anchored", "the", "ancient", "oak", "tree", "to", "the", "earth.", "The", "programmer", "stared", "at", "the", "screen,", "searching", "for", "the", "single,", "elusive", "bug", "in", "a", "million", "lines", "of", "code.", "The", "waves", "crashed", "against", "the", "rocky", "shore", "in", "a", "timeless,", "powerful", "rhythm.", "A", "flock", "of", "geese", "flew", "south", "in", "a", "perfect", "V-formation,", "a", "testament", "to", "their", "instinctual", "harmony.", "The", "historian", "pieced", "together", "the", "fragments", "of", "the", "past", "to", "tell", "a", "coherent", "story.", "In", "the", "quiet", "solitude", "of", "the", "desert,", "one", "could", "hear", "the", "whisper", "of", "the", "wind.", "The", "gardener", "tended", "to", "her", "roses", "with", "a", "gentle,", "nurturing", "touch.", "A", "crackling", "fireplace", "provided", "a", "warm", "and", "inviting", "centerpiece", "to", "the", "rustic", "living", "room.", "The", "mountaineer", "stood", "at", "the", "summit,", "humbled", "by", "the", "breathtaking", "vista", "below.", "A", "single,", "perfect", "snowflake", "landed", "on", "the", "child's", "outstretched", "mitten."];
    default_text.iter().map(|s| s.to_string()).collect()
}

/// Calculates the hash of text.txt in a specified directory.
pub fn calculate_text_txt_hash(dir: &Path) -> io::Result<Vec<u8>> {
    let path = dir.join("text.txt");
    let file_bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    Ok(hasher.finalize().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_config() {
        // Create a temporary directory to avoid interfering with actual config files.
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // --- Test saving and loading an existing config ---
        let mut config_to_save = Config::default();
        config_to_save.first_boot = false;
        config_to_save.save_mistyped = false;
        config_to_save.mistyped_chars.insert("a".to_string(), 100);

        // Save the custom config and assert it was successful.
        assert!(save_config(&config_to_save, dir_path).is_ok());

        // Load the config back from the directory.
        let loaded_config = load_config(dir_path).unwrap();

        // Check that the loaded values match what we saved.
        assert_eq!(loaded_config.first_boot, false);
        assert_eq!(loaded_config.save_mistyped, false);
        assert_eq!(*loaded_config.mistyped_chars.get("a").unwrap(), 100);

        // --- Test loading a config when none exists ---
        // `load_config` should create a default one automatically.
        let new_dir = tempdir().unwrap();
        let new_dir_path = new_dir.path();
        let default_config = load_config(new_dir_path).unwrap();
        
        // Check that the created config has default values.
        assert_eq!(default_config.first_boot, true);
        assert!(default_config.mistyped_chars.is_empty());
    }

    #[test]
    fn test_read_items_from_file() {
        // Create a temporary directory.
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // --- Test reading a words.txt file ---
        let words_content = "hello world from ttypr";
        fs::write(dir_path.join("words.txt"), words_content).unwrap();
        
        let words = read_words_from_file(dir_path).unwrap();
        assert_eq!(words, vec!["hello", "world", "from", "ttypr"]);

        // --- Test reading a text.txt file ---
        let text_content = "this is a line of text";
        fs::write(dir_path.join("text.txt"), text_content).unwrap();

        let text = read_text_from_file(dir_path).unwrap();
        assert_eq!(text, vec!["this", "is", "a", "line", "of", "text"]);

        // --- Test error handling for missing files ---
        assert!(read_words_from_file(dir.path().join("non_existent_dir").as_path()).is_err());
        assert!(read_text_from_file(dir.path().join("another_fake_dir").as_path()).is_err());
    }

    #[test]
    fn test_calculate_text_txt_hash() {
        // Create a temporary directory.
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // --- Test hashing an existing file ---
        let content = "hello ttypr";
        fs::write(dir_path.join("text.txt"), content).unwrap();

        // Calculate the hash using our function.
        let file_hash = calculate_text_txt_hash(dir_path).unwrap();

        // Calculate the hash manually to get the expected result.
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let expected_hash = hasher.finalize().to_vec();

        assert_eq!(file_hash, expected_hash);

        // --- Test error handling for a missing file ---
        let new_dir = tempdir().unwrap();
        assert!(calculate_text_txt_hash(new_dir.path()).is_err());
    }
    
    #[test]
    fn test_get_sorted_mistakes() {
        // Create a sample map of mistyped characters
        let mut mistyped_chars = HashMap::new();
        mistyped_chars.insert("a".to_string(), 5);
        mistyped_chars.insert("c".to_string(), 10);
        mistyped_chars.insert("b".to_string(), 10); // Same count as 'c'
        mistyped_chars.insert("d".to_string(), 2);

        // Get the sorted list of mistakes (which is a Vec of references)
        let sorted_result = get_sorted_mistakes(&mistyped_chars);

        // Convert the result from a Vec of references to a Vec of owned values for comparison
        let actual: Vec<(String, usize)> = sorted_result.iter().map(|(k, v)| ((*k).clone(), **v)).collect();

        // Define the expected order directly with owned values
        let expected: Vec<(String, usize)> = vec![
            ("b".to_string(), 10),
            ("c".to_string(), 10),
            ("a".to_string(), 5),
            ("d".to_string(), 2),
        ];

        assert_eq!(actual, expected);

        // Test with an empty map
        let empty_map = HashMap::new();
        let sorted_empty = get_sorted_mistakes(&empty_map);
        assert!(sorted_empty.is_empty());
    }

    #[test]
    fn test_default_words() {
        let words = default_words();
        // Check that it returns a non-empty list
        assert!(!words.is_empty(), "The default word list should not be empty.");
        // Check for a specific known value to guard against accidental changes
        assert_eq!(words[0], "the");
        assert_eq!(words.last().unwrap(), "enormous");
    }

    #[test]
    fn test_default_text() {
        let text = default_text();
        // Check that it returns a non-empty list
        assert!(!text.is_empty(), "The default text list should not be empty.");
        // Check for specific known values to guard against accidental changes
        assert_eq!(text[0], "The");
        assert_eq!(text.last().unwrap(), "mitten.");
    }
}