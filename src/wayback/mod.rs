use colored::*;
use std::collections::HashSet;
use regex::Regex;
use std::error::Error;

#[derive(Debug)]
pub enum WaybackError {
    NetworkError(String),
    EmptyResponse,
    InvalidResponse(String),
    RegexError(String),
    HttpError(String),
}

impl std::fmt::Display for WaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaybackError::NetworkError(e) => write!(f, "Network error: {}", e),
            WaybackError::EmptyResponse => write!(f, "Wayback Machine returned no data"),
            WaybackError::InvalidResponse(e) => write!(f, "Invalid response format: {}", e),
            WaybackError::RegexError(e) => write!(f, "Regex error: {}", e),
            WaybackError::HttpError(e) => write!(f, "HTTP error: {}", e),
        }
    }
}

impl Error for WaybackError {}

pub struct WaybackMachine {
    client: reqwest::Client,
}

impl WaybackMachine {
    pub fn new() -> Self {
        WaybackMachine {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_subdomains(&self, domain: &str) -> Result<Vec<String>, WaybackError> {
        self.log_info("Initializing Wayback Machine scan...");
        
        let url = format!(
            "http://web.archive.org/cdx/search/cdx?url=*.{}&output=json&fl=original&collapse=urlkey",
            domain
        );

        let domain_regex = Regex::new(r"url=\*\.([^&]+)").unwrap();
        let domain = domain_regex.captures(&url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("unknown");
        self.log_info(&format!("Searching for subdomains of: {}", domain));

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| WaybackError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_msg = format!("HTTP {} {}",
                response.status().as_str(),
                response.status().canonical_reason().unwrap_or("Unknown error")
            );
            self.log_error(&error_msg);
            return Err(WaybackError::HttpError(error_msg));
        }

        let urls: Vec<Vec<String>> = response.json()
            .await
            .map_err(|e| WaybackError::InvalidResponse(e.to_string()))?;

        if urls.is_empty() {
            self.log_warning("Wayback Machine returned empty response");
            return Err(WaybackError::EmptyResponse);
        }

        self.log_success(&format!("Retrieved {} URLs from Wayback Machine", urls.len()));
        
        let urls: Vec<String> = urls.into_iter()
            .skip(1) // Skip header row
            .map(|row| row[0].clone())
            .collect();

        let subdomains = self.extract_subdomains(domain, &urls)?;
        self.log_success(&format!("Found {} unique subdomains", subdomains.len()));
        
        Ok(subdomains)
    }

    fn extract_subdomains(&self, base_domain: &str, urls: &[String]) -> Result<Vec<String>, WaybackError> {
        self.log_info(&format!("Processing {} URLs for subdomain extraction", urls.len()));
        
        let subdomain_pattern = format!(
            r"(?i)https?://([a-zA-Z0-9][-a-zA-Z0-9]*\.)*{}",
            regex::escape(base_domain)
        );
        
        let re = Regex::new(&subdomain_pattern)
            .map_err(|e| WaybackError::RegexError(e.to_string()))?;
        
        let mut subdomains = HashSet::new();
        let mut invalid_count = 0;
        let mut processed = 0;

        for url in urls {
            processed += 1;
            if processed % 1000 == 0 {
                self.log_info(&format!("Processed {} URLs", processed));
            }

            match re.captures(url) {
                Some(captures) => {
                    if let Some(subdomain) = captures.get(1) {
                        let mut full_domain = subdomain.as_str().to_string();
                        if !full_domain.ends_with(base_domain) {
                            full_domain.push_str(base_domain);
                        }
                        if full_domain.ends_with('.') {
                            full_domain.pop();
                        }
                        subdomains.insert(full_domain);
                    } else {
                        invalid_count += 1;
                    }
                }
                None => {
                    invalid_count += 1;
                }
            }
        }

        if invalid_count > 0 {
            self.log_warning(&format!("Skipped {} invalid URLs", invalid_count));
        }

        let result: Vec<String> = subdomains.into_iter().collect();
        self.log_info(&format!("Extracted {} unique subdomains", result.len()));
        Ok(result)
    }

    fn log_info(&self, message: &str) {
        println!("{} {}", "[*]".blue(), message);
    }

    fn log_success(&self, message: &str) {
        println!("{} {}", "[+]".green(), message);
    }

    fn log_warning(&self, message: &str) {
        println!("{} {}", "[!]".yellow(), message);
    }

    fn log_error(&self, message: &str) {
        println!("{} {}", "[!]".red(), message);
    }
}