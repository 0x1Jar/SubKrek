use url::Url;

/// Extract the base domain from a URL or domain string
pub fn extract_domain(input: &str) -> Option<String> {
    // If input is a URL, parse it
    if input.contains("://") {
        if let Ok(url) = Url::parse(input) {
            return url.host_str().map(String::from);
        }
    }
    
    // Otherwise, treat it as a domain name
    let domain = input.trim().to_lowercase();
    if domain.contains('.') {
        Some(domain)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("https://sub.example.com"), Some("sub.example.com".to_string()));
        assert_eq!(extract_domain("invalid"), None);
    }
}