mod scanner;
mod utils;
mod wayback;
mod wordlist;

use clap::Parser;
use colored::*;
use scanner::Scanner;
use std::path::PathBuf;
use std::time::Instant;
use wayback::WaybackMachine;

#[derive(Parser, Debug)]
#[command(
    name = "SubKrek",
    about = "A fast subdomain scanner with Wayback Machine integration"
)]
struct Args {
    #[arg(short, long)]
    domain: String,

    #[arg(short, long, default_value = "50")]
    concurrency: usize,

    #[arg(short, long)]
    wordlist: Option<PathBuf>,

    #[arg(long, help = "Directory containing wordlist files")]
    wordlist_dir: Option<PathBuf>,

    #[arg(short = 'b', long, help = "Use Wayback Machine to find historical subdomains")]
    wayback: bool,

    #[arg(short, long, help = "Output file to save results")]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let start_time = Instant::now();

    println!("\n{}", "🔍 SubKrek Scanner".bright_blue().bold());
    
    // Extract and validate domain
    let domain = utils::extract_domain(&args.domain)
        .ok_or("Invalid domain format")?;
    println!("{} {}\n", "Target Domain:".yellow(), domain);

    // Initialize scanner with default wordlist directory
    let mut scanner = Scanner::new(
        args.concurrency,
        args.wordlist_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("wordlists"),
    ).await;

    // Add specific wordlist if provided
    if let Some(wordlist_path) = &args.wordlist {
        if let Some(path_str) = wordlist_path.to_str() {
            scanner.add_wordlist(path_str)?;
        }
    }

    // Add wordlist directory if provided
    if let Some(dir_path) = &args.wordlist_dir {
        if let Some(dir_str) = dir_path.to_str() {
            scanner.add_wordlist_directory(dir_str)?;
        }
    }

    // Fetch historical subdomains if wayback option is enabled
    if args.wayback {
        println!("{}", "Fetching historical subdomains from Wayback Machine...".cyan());
        let wayback = WaybackMachine::new();
        match wayback.fetch_subdomains(&domain).await {
            Ok(historical_subdomains) => {
                println!("Found {} historical subdomains", historical_subdomains.len());
                // Create a temporary file for historical subdomains
                let temp_dir = std::env::temp_dir();
                let temp_file = temp_dir.join("historical_subdomains.txt");
                std::fs::write(&temp_file, historical_subdomains.join("\n"))?;
                scanner.add_wordlist(temp_file.to_str().unwrap())?;
            }
            Err(e) => eprintln!("Error fetching from Wayback Machine: {}", e),
        }
    }

    // Perform scan
    let valid_subdomains = scanner.scan_domains(&domain).await?;

    // Display and save results
    if !valid_subdomains.is_empty() {
        println!("\n{}", "Valid Subdomains:".bright_green().bold());
        for subdomain in &valid_subdomains {
            println!("✅ {}", subdomain.green());
        }

        if let Some(output_path) = args.output {
            std::fs::write(output_path, valid_subdomains.join("\n"))?;
        }
    } else {
        println!("\n{}", "No valid subdomains found.".yellow());
    }

    let elapsed = start_time.elapsed();
    println!("\n{}", "Scan Complete!".bright_blue().bold());
    println!("Time elapsed: {:.2?}", elapsed);

    Ok(())
}
