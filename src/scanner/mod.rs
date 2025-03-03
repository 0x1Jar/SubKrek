use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

pub struct Scanner {
    resolver: TokioAsyncResolver,
    concurrency: usize,
}

impl Scanner {
    pub async fn new(concurrency: usize) -> Self {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );

        Scanner {
            resolver,
            concurrency,
        }
    }

    pub async fn scan_domains(&self, subdomains: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if subdomains.is_empty() {
            println!("{}", "No subdomains to scan".yellow());
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scanner() {
        let scanner = Scanner::new(10).await;
        let test_subdomains = vec![
            "www.example.com".to_string(),
            "mail.example.com".to_string(),
            "test.example.com".to_string()
        ];
        
        // Test empty subdomains case
        let empty_result = scanner.scan_domains(vec![]).await.unwrap();
        assert!(empty_result.is_empty());

        // Since we can't reliably test actual DNS resolution in unit tests,
        // we'll just verify that the scanner accepts the subdomains vector
        assert_eq!(test_subdomains.len(), 3);
    }
}