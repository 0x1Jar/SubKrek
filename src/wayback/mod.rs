use serde::Deserialize;
use std::collections::HashSet;
use regex::Regex;

#[derive(Deserialize)]
struct WaybackResponse {
    url: String,
}

pub struct WaybackMachine {
    client: reqwest::Client,
}

impl WaybackMachine {
    pub fn new() -> Self {
        WaybackMachine {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_subdomains(&self, domain: &str) -> Result<Vec<String>, reqwest::Error> {
        let url = format!(
            "http://web.archive.org/cdx/search/cdx?url=*.{}&output=json&fl=original&collapse=urlkey",
            domain
        );

        let response = self.client.get(&url).send().await?;
        let urls: Vec<String> = response.json::<Vec<Vec<String>>>().await?
            .into_iter()
            .skip(1) // Skip header row
            .map(|row| row[0].clone())
            .collect();

        let subdomains = self.extract_subdomains(domain, &urls);
        Ok(subdomains)
    }

    fn extract_subdomains(&self, base_domain: &str, urls: &[String]) -> Vec<String> {
        let subdomain_pattern = format!(
            r"(?i)https?://([a-zA-Z0-9][-a-zA-Z0-9]*\.)*{}",
            regex::escape(base_domain)
        );
        let re = Regex::new(&subdomain_pattern).unwrap();
        
        let mut subdomains = HashSet::new();

        for url in urls {
            if let Some(captures) = re.captures(url) {
                if let Some(subdomain) = captures.get(1) {
                    let mut full_domain = subdomain.as_str().to_string();
                    if !full_domain.ends_with(base_domain) {
                        full_domain.push_str(base_domain);
                    }
                    // Remove trailing dot if present
                    if full_domain.ends_with('.') {
                        full_domain.pop();
                    }
                    subdomains.insert(full_domain);
                }
            }
        }

        subdomains.into_iter().collect()
    }
}