use clap::Parser;
use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Parser, Debug)]
#[command(
    name = "domain-api-scann",
    about = "A fast subdomain scanner written in Rust"
)]
struct Args {
    #[arg(short, long)]
    domain: String,

    #[arg(short, long, default_value = "50")]
    concurrency: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let start_time = Instant::now();

    println!("\n{}", "🔍 Domain Scanner".bright_blue().bold());
    println!("{} {}\n", "Target Domain:".yellow(), args.domain);

    // Initialize DNS resolver
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    );

    // Generate subdomains
    let subdomains = generate_subdomains(&args.domain);
    let total_domains = subdomains.len();

    println!("{} {}", "Total subdomains to scan:".yellow(), total_domains);

    // Setup progress bar
    let pb = ProgressBar::new(total_domains as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Concurrent scanning
    let results = stream::iter(subdomains)
        .map(|subdomain| {
            let resolver = &resolver;
            let pb = &pb;
            async move {
                let result = check_subdomain(resolver, &subdomain).await;
                pb.inc(1);
                (subdomain, result)
            }
        })
        .buffered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    pb.finish_with_message("scan completed");

    // Display results
    println!("\n{}", "Results:".bright_green().bold());
    let mut valid_count = 0;

    for (subdomain, exists) in results {
        if exists {
            println!("✅ {}", subdomain.green());
            valid_count += 1;
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n{}", "Scan Summary:".bright_blue().bold());
    println!("Time elapsed: {:.2?}", elapsed);
    println!("Valid subdomains found: {}", valid_count);
}

async fn check_subdomain(resolver: &TokioAsyncResolver, subdomain: &str) -> bool {
    match resolver.lookup_ip(subdomain).await {
        Ok(response) => !response.iter().next().is_none(),
        Err(_) => false,
    }
}

fn generate_subdomains(domain: &str) -> Vec<String> {
    let prefixes = vec![
        "www", "mail", "remote", "blog", "webmail", "server", "ns1", "ns2",
        "smtp", "secure", "vpn", "m", "shop", "ftp", "mail2", "test", "portal",
        "web", "dev", "staging", "api", "corp", "admin", "mobile", "mx", "wiki",
    ];

    prefixes
        .into_iter()
        .map(|prefix| format!("{}.{}", prefix, domain))
        .collect()
}
