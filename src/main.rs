mod scanner;
mod utils;
mod wayback;

use clap::Parser;
use colored::*;
use scanner::Scanner;
use std::path::PathBuf;
use std::time::Instant;
use wayback::WaybackMachine;
use utils::extract_domain;

#[derive(Parser, Debug)]
#[command(
    name = "SubKrek",
    about = "A subdomain scanner using Wayback Machine"
)]
struct Args {
    #[arg(short, long)]
    domain: String,

    #[arg(short, long, default_value = "50")]
    concurrency: usize,

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

    // Initialize scanner
    let scanner = Scanner::new(args.concurrency).await;
    
    // Fetch historical subdomains if wayback option is enabled
    let mut subdomains = Vec::new();
    if args.wayback {
        println!("{}", "Fetching historical subdomains from Wayback Machine...".cyan());
        let wayback = WaybackMachine::new();
        match wayback.fetch_subdomains(&domain).await {
            Ok(historical_subdomains) => {
                println!("Found {} historical subdomains", historical_subdomains.len());
                subdomains.extend(historical_subdomains);
            }
            Err(e) => eprintln!("Error fetching from Wayback Machine: {}", e),
        }
    }

    // Perform scan
    let valid_subdomains = scanner.scan_domains(subdomains).await?;

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
