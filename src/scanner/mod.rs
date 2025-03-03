use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;
use crate::wordlist::WordlistManager;
use std::path::Path;
use std::fs;

pub struct Scanner {
    resolver: TokioAsyncResolver,
    concurrency: usize,
    wordlist_manager: WordlistManager,
}

impl Scanner {
    pub async fn new<P: AsRef<Path>>(concurrency: usize, wordlist_dir: P) -> Self {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );

        // Ensure wordlist directory exists
        let dir_path = wordlist_dir.as_ref();
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).expect("Failed to create wordlist directory");
        }

        let wordlist_manager = WordlistManager::new(dir_path);

        // Create scanner instance
        let mut scanner = Scanner {
            resolver,
            concurrency,
            wordlist_manager,
        };

        // Try to load wordlists from the directory
        match scanner.wordlist_manager.add_directory(".") {
            Ok(_) => println!("Successfully loaded wordlists from directory"),
            Err(e) => eprintln!("Warning: Could not load wordlists from directory: {}", e),
        }

        scanner
    }

    pub fn add_wordlist<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.wordlist_manager.add_wordlist(path)?;
        Ok(())
    }

    pub async fn scan_domains(&mut self, domain: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Load all wordlists
        match self.wordlist_manager.load_all() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error loading wordlists: {}", e);
                // Try to load from current directory as fallback
                self.wordlist_manager.add_directory(".")?;
                self.wordlist_manager.load_all()?;
            }
        }

        let wordlist: Vec<String> = self.wordlist_manager.get_words().iter().cloned().collect();
        
        if wordlist.is_empty() {
            println!("{}", "No valid subdomains found in wordlists".yellow());
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        let subdomains = self.generate_subdomains(domain, &wordlist);
        let total_domains = subdomains.len();

        println!("{} {}", "Total subdomains to scan:".yellow(), total_domains);

        let pb = self.create_progress_bar(total_domains as u64);
        let results = self.perform_scan(&subdomains, &pb).await;
        pb.finish_with_message("scan completed");

        let valid_subdomains: Vec<String> = results
            .into_iter()
            .filter_map(|(subdomain, exists)| if exists { Some(subdomain) } else { None })
            .collect();

        println!("\n{}", "Scan Summary:".bright_blue().bold());
        println!("Time elapsed: {:.2?}", start_time.elapsed());
        println!("Valid subdomains found: {}", valid_subdomains.len());

        Ok(valid_subdomains)
    }

    async fn perform_scan(&self, subdomains: &[String], pb: &ProgressBar) -> Vec<(String, bool)> {
        stream::iter(subdomains.to_vec())
            .map(|subdomain| {
                let resolver = &self.resolver;
                let pb = &pb;
                async move {
                    let result = self.check_subdomain(resolver, &subdomain).await;
                    pb.inc(1);
                    (subdomain, result)
                }
            })
            .buffered(self.concurrency)
            .collect::<Vec<_>>()
            .await
    }

    fn create_progress_bar(&self, total: u64) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }

    async fn check_subdomain(&self, resolver: &TokioAsyncResolver, subdomain: &str) -> bool {
        match resolver.lookup_ip(subdomain).await {
            Ok(response) => !response.iter().next().is_none(),
            Err(_) => false,
        }
    }

    fn generate_subdomains(&self, domain: &str, wordlist: &[String]) -> Vec<String> {
        wordlist
            .iter()
            .map(|prefix| format!("{}.{}", prefix, domain))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_scanner_with_wordlist() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_wordlist.txt");
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "www\nmail\ntest").unwrap();

        let mut scanner = Scanner::new(10, temp_dir.path()).await;
        scanner.add_wordlist(test_file_path).unwrap();
        
        // Since we can't reliably test actual DNS resolution in unit tests,
        // we'll just verify that the scanner properly loads and uses the wordlist
        let generated = scanner.generate_subdomains("example.com", &vec!["www".to_string(), "mail".to_string(), "test".to_string()]);
        assert_eq!(generated.len(), 3);
        assert!(generated.contains(&"www.example.com".to_string()));
        assert!(generated.contains(&"mail.example.com".to_string()));
        assert!(generated.contains(&"test.example.com".to_string()));
    }
}