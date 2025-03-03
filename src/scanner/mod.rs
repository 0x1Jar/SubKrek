use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::Name;
use trust_dns_resolver::Resolver;

#[derive(Debug)]
pub enum ScanError {
    EmptyInput,
    ConfigError(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::EmptyInput => write!(f, "No subdomains provided for scanning"),
            ScanError::ConfigError(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl std::error::Error for ScanError {}

#[derive(Debug, PartialEq)]
enum ScanStatus {
    Valid,
    Invalid,
    Error,
}

pub struct Scanner {
    resolver: Resolver,
    concurrency: usize,
}

impl Scanner {
    pub async fn new(concurrency: usize) -> Result<Self, ScanError> {
        let resolver = Resolver::from_system_conf()
            .map_err(|e| ScanError::ConfigError(e.to_string()))?;

        Ok(Scanner {
            resolver,
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

        let pb = self.create_progress_bar(total_domains as u64);
        let results = self.perform_scan(&subdomains, &pb).await;
        pb.finish_with_message("scan completed");

        let mut valid_count = 0;
        let mut invalid_count = 0;
        let mut error_count = 0;

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
                    ScanStatus::Error => {
                        error_count += 1;
                        None
                    }
                }
            })
            .collect();

        println!("\n{}", "Scan Summary:".bright_blue().bold());
        println!("{} {:.2?}", "Time elapsed:".blue(), start_time.elapsed());
        println!("{} {}", "Valid subdomains:".green(), valid_count);
        println!("{} {}", "Invalid subdomains:".yellow(), invalid_count);
        if error_count > 0 {
            println!("{} {}", "Scan errors:".red(), error_count);
        }
        println!("{} {}", "Total processed:".blue(), valid_count + invalid_count + error_count);

        Ok(valid_subdomains)
    }

    async fn perform_scan(&self, subdomains: &[String], pb: &ProgressBar) -> Vec<(String, ScanStatus)> {
        stream::iter(subdomains.to_vec())
            .map(|subdomain| {
                let resolver = &self.resolver;
                let pb = &pb;
                async move {
                    let status = self.check_subdomain(resolver, &subdomain);
                    pb.inc(1);
                    match &status {
                        ScanStatus::Valid => pb.println(format!("{} {}", "✓".green(), subdomain.green())),
                        ScanStatus::Invalid => pb.println(format!("{} {}", "✗".yellow(), subdomain.yellow())),
                        ScanStatus::Error => pb.println(format!("{} {}", "!".red(), subdomain.red())),
                    }
                    (subdomain, status)
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
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Scanning subdomains...");
        pb
    }

    fn check_subdomain(&self, resolver: &Resolver, subdomain: &str) -> ScanStatus {
        match Name::from_ascii(subdomain) {
            Ok(name) => match resolver.lookup_ip(name) {
                Ok(response) => {
                    if response.iter().next().is_some() {
                        ScanStatus::Valid
                    } else {
                        ScanStatus::Invalid
                    }
                }
                Err(e) => match e.kind() {
                    ResolveErrorKind::NoRecordsFound { .. } => ScanStatus::Invalid,
                    _ => ScanStatus::Error,
                }
            },
            Err(_) => ScanStatus::Error,
        }
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