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
        WordlistManager {
            wordlist_paths: Vec::new(),
            default_directory: default_directory.as_ref().to_path_buf(),
            loaded_words: HashSet::new(),
        }
    }

    pub fn add_wordlist<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WordlistError> {
        let path = self.resolve_path(path)?;
        if !self.wordlist_paths.contains(&path) {
            self.wordlist_paths.push(path);
        }
        Ok(())
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, directory: P) -> Result<(), WordlistError> {
        let dir_path = self.resolve_path(directory)?;
        if !dir_path.is_dir() {
            return Err(WordlistError::DirectoryNotFound(dir_path));
        }

        for entry in fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "txt") {
                self.add_wordlist(&path)?;
            }
        }
        Ok(())
    }

    pub fn load_all(&mut self) -> Result<(), WordlistError> {
        self.loaded_words.clear();
        for path in &self.wordlist_paths {
            self.load_wordlist(path)?;
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
            Ok(self.default_directory.join(path))
        }
    }

    fn load_wordlist(&mut self, path: &Path) -> Result<(), WordlistError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let word = line.trim();
            if !word.is_empty() && !word.starts_with('#') {
                if !self.validate_word(word) {
                    return Err(WordlistError::InvalidFormat(
                        format!("Invalid word format: {} in file {}", word, path.display())
                    ));
                }
                self.loaded_words.insert(word.to_string());
            }
        }
        Ok(())
    }

    fn validate_word(&self, word: &str) -> bool {
        // Basic validation: no spaces, special characters except - and .
        word.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.')
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
            "www\nmail\nftp\n# comment\ntest-domain\n",
        );

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_wordlist(&test_file).unwrap();
        manager.load_all().unwrap();

        let words = manager.get_words();
        assert_eq!(words.len(), 4);
        assert!(words.contains("www"));
        assert!(words.contains("mail"));
        assert!(words.contains("ftp"));
        assert!(words.contains("test-domain"));
        assert!(!words.contains("# comment"));
    }

    #[test]
    fn test_invalid_word_format() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(
            temp_dir.path(),
            "invalid.txt",
            "valid\ninvalid word with space\nvalid-word",
        );

        let mut manager = WordlistManager::new(temp_dir.path());
        manager.add_wordlist(&test_file).unwrap();
        assert!(matches!(
            manager.load_all(),
            Err(WordlistError::InvalidFormat(_))
        ));
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
}