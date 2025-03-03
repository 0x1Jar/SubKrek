use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct WordlistLoader;

impl WordlistLoader {
    pub fn load_wordlist<P: AsRef<Path>>(path: Option<P>) -> io::Result<Vec<String>> {
        let default_prefixes = vec![
            "www", "mail", "remote", "blog", "webmail", "server", "ns1", "ns2",
            "smtp", "secure", "vpn", "m", "shop", "ftp", "mail2", "test", "portal",
            "web", "dev", "staging", "api", "corp", "admin", "mobile", "mx", "wiki",
        ];

        if let Some(path) = path {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let wordlist: Vec<String> = reader
                .lines()
                .filter_map(|line| line.ok())
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .collect();

            if wordlist.is_empty() {
                Ok(default_prefixes.into_iter().map(String::from).collect())
            } else {
                Ok(wordlist)
            }
        } else {
            Ok(default_prefixes.into_iter().map(String::from).collect())
        }
    }

    pub fn merge_wordlists(lists: Vec<Vec<String>>) -> Vec<String> {
        let mut combined: Vec<String> = lists.into_iter().flatten().collect();
        combined.sort();
        combined.dedup();
        combined
    }
}

pub fn format_url(url: &str) -> String {
    let url = url.trim().to_lowercase();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}

pub fn extract_domain(url: &str) -> Option<String> {
    let url = format_url(url);
    url.split("://")
        .nth(1)
        .map(|s| s.split('/').next().unwrap_or(s).to_string())
}