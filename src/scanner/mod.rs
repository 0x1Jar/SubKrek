use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use tokio::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub enum ScanError {
    EmptyInput,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::EmptyInput => write!(f, "No subdomains provided for scanning"),
        }
    }
}

impl std::error::Error for ScanError {}

#[derive(Debug, PartialEq)]
enum ScanStatus {
    Valid,
    Invalid,
}

pub struct Scanner {
    concurrency: usize,
}

impl Scanner {
    pub async fn new(concurrency: usize) -> Result<Self, ScanError> {
        Ok(Scanner {
            concurrency,
        })
    }

    pub async fn scan_domains(&self, subdomains: Vec<String>) -> Result<Vec<String>, ScanError> {
        if subdomains.is_empty() {
            println!("{} {}", "[!]".yellow(), "No subdomains to scan");
            return Err(ScanError::EmptyInput);
        }

        let start_time = Instant::now();
        let total_domains = subdomains.len();

        println!("{}", "[*] Initializing scan...".blue());
        println!("{} {}", "[*]".blue(), format!("Found {} subdomains to scan", total_domains));
        println!("{} {}", "[*]".blue(), format!("Using {} concurrent connections", self.concurrency));

        let progress = self.create_progress_bar(total_domains as u64);
        let results = self.check_subdomains(&subdomains, &progress).await;
        progress.finish_with_message("scan completed");

        let mut valid_count = 0;
        let mut invalid_count = 0;

        let valid_subdomains: Vec<String> = results
            .into_iter()
            .filter_map(|(subdomain, status)| {
                match status {
                    ScanStatus::Valid => {
                        valid_count += 1;
                        Some(subdomain)
                    }
                    ScanStatus::Invalid => {
                        invalid_count += 1;
                        None
                    }
                }
            })
            .collect();

        println!("\n{}", "Scan Summary:".bright_blue().bold());
        println!("{} {:.2?}", "Time elapsed:".blue(), start_time.elapsed());
        println!("{} {}", "Valid subdomains:".green(), valid_count);
        println!("{} {}", "Invalid subdomains:".yellow(), invalid_count);
        println!("{} {}", "Total processed:".blue(), valid_count + invalid_count);

        Ok(valid_subdomains)
    }

    async fn check_subdomains(&self, subdomains: &[String], progress: &ProgressBar) -> Vec<(String, ScanStatus)> {
        stream::iter(subdomains.to_vec())
            .map(|subdomain| {
                let progress = progress.clone();
                async move {
                    let endpoint = format!("{}:80", subdomain);
                    let status = match tokio::time::timeout(
                        Duration::from_secs(5),
                        TcpStream::connect(&endpoint)
                    ).await {
                        Ok(Ok(_)) => ScanStatus::Valid,
                        Ok(Err(e)) => match e.kind() {
                            std::io::ErrorKind::ConnectionRefused => ScanStatus::Valid, // Host exists but port is closed
                            _ => ScanStatus::Invalid,
                        },
                        Err(_) => ScanStatus::Invalid,
                    };
                    
                    progress.inc(1);
                    match &status {
                        ScanStatus::Valid => progress.println(format!("{} {}", "✓".green(), subdomain.green())),
                        ScanStatus::Invalid => progress.println(format!("{} {}", "✗".yellow(), subdomain.yellow())),
                    }
                    (subdomain, status)
                }
            })
            .buffered(self.concurrency)
            .collect::<Vec<_>>()
            .await
    }

    fn create_progress_bar(&self, total: u64) -> ProgressBar {
        let progress = ProgressBar::new(total);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        progress.set_message("Scanning subdomains...");
        progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scanner() {
        let scanner = Scanner::new(10).await.expect("Failed to create scanner");
        
        // Test empty subdomains case
        let empty_result = scanner.scan_domains(vec![]).await;
        assert!(matches!(empty_result, Err(ScanError::EmptyInput)));

        // Test with some domains
        let test_subdomains = vec![
            "www.example.com".to_string(),
            "mail.example.com".to_string(),
            "test.example.com".to_string()
        ];
        assert_eq!(test_subdomains.len(), 3);
    }
}