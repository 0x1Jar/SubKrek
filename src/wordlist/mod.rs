use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug)]
pub struct WordlistManager {
    wordlist_paths: Vec<PathBuf>,
    default_directory: PathBuf,
    loaded_words: HashSet<String>,
}

#[derive(Debug)]
pub enum WordlistError {
    IoError(io::Error),
    InvalidFormat(String),
    EmptyWordlist(PathBuf),
    DirectoryNotFound(PathBuf),
}

impl std::fmt::Display for WordlistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WordlistError::IoError(e) => write!(f, "IO error: {}", e),
            WordlistError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            WordlistError::EmptyWordlist(path) => write!(f, "Empty wordlist: {}", path.display()),
            WordlistError::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path.display()),
        }
    }
}

impl Error for WordlistError {}

impl From<io::Error> for WordlistError {
    fn from(error: io::Error) -> Self {
        WordlistError::IoError(error)
    }
}

impl WordlistManager {
    pub fn new<P: AsRef<Path>>(default_directory: P) -> Self {
        let default_dir = default_directory.as_ref().to_path_buf();
        // Create the directory if it doesn't exist
        if !default_dir.exists() {
            fs::create_dir_all(&default_dir).expect("Failed to create wordlist directory");
        }
        
        WordlistManager {
            wordlist_paths: Vec::new(),
            default_directory: default_dir,
            loaded_words: HashSet::new(),
        }
    }

    pub fn add_wordlist<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WordlistError> {
        let path = self.resolve_path(path)?;
        if path.exists() && !self.wordlist_paths.contains(&path) {
            self.wordlist_paths.push(path);
        }
        Ok(())
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, directory: P) -> Result<(), WordlistError> {
        let dir_path = self.resolve_path(directory)?;
        
        if !dir_path.exists() {
            return Err(WordlistError::DirectoryNotFound(dir_path));
        }
        if !dir_path.is_dir() {
            return Err(WordlistError::DirectoryNotFound(dir_path));
        }

        let mut new_paths = Vec::new();
        for entry in fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                if !self.wordlist_paths.contains(&path) {
                    new_paths.push(path);
                }
            }
        }
        self.wordlist_paths.extend(new_paths);
        Ok(())
    }

    pub fn load_all(&mut self) -> Result<(), WordlistError> {
        self.loaded_words.clear();

        // If no wordlists are added yet, try the default directory
        if self.wordlist_paths.is_empty() {
            let default_dir = self.default_directory.clone();
            if default_dir.exists() && default_dir.is_dir() {
                self.add_directory(&default_dir)?;
            }
        }

        // Clone paths to avoid borrow checker issues
        let paths_to_load: Vec<PathBuf> = self.wordlist_paths.clone();
        
        // Load each wordlist
        for path in paths_to_load {
            if path.exists() {
                self.load_wordlist(&path)?;
            }
        }

        if self.loaded_words.is_empty() {
            return Err(WordlistError::EmptyWordlist(self.default_directory.clone()));
        }
        Ok(())
    }

    pub fn get_words(&self) -> &HashSet<String> {
        &self.loaded_words
    }

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, WordlistError> {
        let path = path.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            // If path starts with "./", remove it
            let path_str = path.to_string_lossy();
            let cleaned_path = if path_str.starts_with("./") {
                Path::new(&path_str[2..])
            } else {
                path
            };
            Ok(self.default_directory.join(cleaned_path))
        }
    }

    fn load_wordlist(&mut self, path: &Path) -> Result<(), WordlistError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut any_valid = false;

        for line in reader.lines() {
            let line = line?;
            let word = line.trim();
            if !word.is_empty() && !word.starts_with('#') {
                // Skip words that likely aren't valid subdomains
                if word.contains('=') || word.contains('[') || word.contains(']') || 
                   word.contains('{') || word.contains('}') || word.contains('?') ||
                   word.contains('&') || word.starts_with('.') || word.ends_with('.') {
                    continue;
                }
                if self.validate_word(word) {
                    self.loaded_words.insert(word.to_string());
                    any_valid = true;
                }
            }
        }

        if !any_valid {
            eprintln!("Warning: No valid subdomains found in {}", path.display());
        }
        Ok(())
    }

    fn validate_word(&self, word: &str) -> bool {
        if word.is_empty() || word.len() > 63 {
            return false;
        }

        // Allow alphanumeric, hyphens, underscores, and dots, but with restrictions:
        // - Must start and end with alphanumeric
        // - No consecutive dots
        // - No consecutive hyphens
        let chars: Vec<char> = word.chars().collect();
        
        // Check first and last character
        if !chars[0].is_alphanumeric() || !chars[chars.len() - 1].is_alphanumeric() {
            return false;
        }

        // Check for consecutive special characters and validate each character
        let mut prev_char = chars[0];
        for &c in &chars[1..] {
            if !c.is_alphanumeric() && !matches!(c, '-' | '_' | '.') {
                return false;
            }
            if (c == '.' && prev_char == '.') || (c == '-' && prev_char == '-') {
                return false;
            }
            prev_char = c;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        writeln!(file, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_wordlist_loading() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(
            temp_dir.path(),
            "test.txt",
            "www\nmail\nftp\n# comment\ntest-domain\ntest_domain\n",
        );

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_wordlist(&test_file).unwrap();
        manager.load_all().unwrap();

        let words = manager.get_words();
        assert_eq!(words.len(), 5);
        assert!(words.contains("www"));
        assert!(words.contains("mail"));
        assert!(words.contains("ftp"));
        assert!(words.contains("test-domain"));
        assert!(words.contains("test_domain"));
        assert!(!words.contains("# comment"));
    }

    #[test]
    fn test_invalid_words_are_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(
            temp_dir.path(),
            "mixed.txt",
            "www\n_invalid\nmail\n[skip]\nftp\n",
        );

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_wordlist(&test_file).unwrap();
        manager.load_all().unwrap();

        let words = manager.get_words();
        assert_eq!(words.len(), 3);
        assert!(words.contains("www"));
        assert!(words.contains("mail"));
        assert!(words.contains("ftp"));
        assert!(!words.contains("_invalid"));
        assert!(!words.contains("[skip]"));
    }

    #[test]
    fn test_directory_loading() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "list1.txt", "www\nmail");
        create_test_file(temp_dir.path(), "list2.txt", "ftp\nsmtp");

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_directory(temp_dir.path()).unwrap();
        manager.load_all().unwrap();

        let words = manager.get_words();
        assert_eq!(words.len(), 4);
        assert!(words.contains("www"));
        assert!(words.contains("mail"));
        assert!(words.contains("ftp"));
        assert!(words.contains("smtp"));
    }

    #[test]
    fn test_empty_wordlist() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(temp_dir.path(), "empty.txt", "");

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_wordlist(&test_file).unwrap();
        assert!(matches!(
            manager.load_all(),
            Err(WordlistError::EmptyWordlist(_))
        ));
    }

    #[test]
    fn test_validate_word() {
        let manager = WordlistManager::new("test");
        
        // Valid formats
        assert!(manager.validate_word("www"));
        assert!(manager.validate_word("test-domain"));
        assert!(manager.validate_word("test_domain"));
        assert!(manager.validate_word("test.domain"));
        assert!(manager.validate_word("sub1.sub2"));
        
        // Invalid formats
        assert!(!manager.validate_word(""));  // empty
        assert!(!manager.validate_word("-test"));  // starts with hyphen
        assert!(!manager.validate_word("test-")); // ends with hyphen
        assert!(!manager.validate_word(".test")); // starts with dot
        assert!(!manager.validate_word("test.")); // ends with dot
        assert!(!manager.validate_word("test..domain")); // consecutive dots
        assert!(!manager.validate_word("test--domain")); // consecutive hyphens
        assert!(!manager.validate_word("test domain")); // contains space
    }
}