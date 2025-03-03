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
use utils::extract_domain;
use std::{env, fs};

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

    #[arg(long, help = "Directory containing wordlist files")]
    wordlist_dir: Option<PathBuf>,

    #[arg(short, long)]
    wordlist: Option<PathBuf>,

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
    let domain = extract_domain(&args.domain)
        .ok_or("Invalid domain format")?;
    println!("{} {}\n", "Target Domain:".yellow(), domain);

    // Setup wordlist directory
    let wordlist_dir = if let Some(dir) = args.wordlist_dir {
        dir
    } else {
        let default_dir = PathBuf::from("wordlists");
        if !default_dir.exists() {
            fs::create_dir_all(&default_dir)?;
            // If no wordlist exists, create a default one
            if !default_dir.join("common.txt").exists() && env::current_dir()?.join("wordlists/common.txt").exists() {
                fs::copy(
                    env::current_dir()?.join("wordlists/common.txt"),
                    default_dir.join("common.txt"),
                )?;
            }
        }
        default_dir
    };

    println!("Using wordlist directory: {}", wordlist_dir.display());

    // Initialize scanner
    let mut scanner = Scanner::new(args.concurrency, &wordlist_dir).await;

    // Add specific wordlist if provided
    if let Some(wordlist_path) = args.wordlist {
        println!("Adding custom wordlist: {}", wordlist_path.display());
        scanner.add_wordlist(&wordlist_path)?;
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
                scanner.add_wordlist(&temp_file)?;
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
