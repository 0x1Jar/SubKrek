# SubTahu - Subdomain Discovery Tool

SubTahu is a high-performance, asynchronous subdomain discovery tool written in Rust. It combines modern subdomain scanning techniques with historical data from the Wayback Machine to provide comprehensive domain reconnaissance capabilities.

## What's New

-   **Enhanced Scanning Engine:** Improved subdomain validation and scanning for faster and more accurate results.
-   **Wayback Machine Integration:** Fetch historical subdomain data from the Wayback Machine to uncover hidden subdomains.
-   **Customizable Concurrency:** Adjust the number of concurrent connections to optimize scanning speed.
-   **Output to File:** Save scan results to a file for further analysis.
-   **Intuitive CLI:** Easy-to-use command-line interface with clear options and help messages.

## Features

- Fast concurrent subdomain scanning
- Historical subdomain discovery using Wayback Machine
- Customizable concurrency level
- Output results to file
- Clean and intuitive CLI interface

## Code Organization

The project is organized into the following modules:

- `src/main.rs`: Entry point of the application, handles argument parsing and orchestrates the scanning process.
- `src/scanner`: Handles subdomain validation and scanning.
    - Uses TCP connection checking to validate subdomains.
    - Supports concurrent scanning with progress tracking.
    - Provides clear status indicators for valid/invalid subdomains.
- `src/wayback`: Manages Wayback Machine integration.
    - Fetches historical URL data from web.archive.org.
    - Extracts and validates subdomains from historical data.
    - Uses standardized logging for operation status (info/success/warn/error).
- `src/utils`: Contains utility functions.
    - Domain extraction and validation.
    - URL parsing helpers.

## Installation

1.  Make sure you have Rust installed. If not, install it from [https://rustup.rs](https://rustup.rs)

2.  Clone the repository:

    ```bash
    git clone https://github.com/0x1Jar/SubTahu.git
    cd SubKrek
    ```

3.  Build the project:

    ```bash
    cargo build --release
    ```

    The binary will be available at `target/release/SubKrek`

## Usage

Basic usage:

```bash
cargo run -- -d example.com
```

### Command Line Options

-   `-h, --help`: Show help message and exit
-   `-d, --domain <DOMAIN>`: Target domain to scan (required)
-   `-c, --concurrency <NUMBER>`: Number of concurrent connections (default: 50)
-   `-b, --wayback`: Use Wayback Machine to find historical subdomains
-   `-o, --output <FILE>`: Save results to a file

To see all available options:

```bash
cargo run -- --help
```

### Examples

1.  Simple scan with default settings:

    ```bash
    cargo run -- -d google.com
    ```

2.  Scan with increased concurrency:

    ```bash
    cargo run -- -d google.com -c 200
    ```

3.  Use Wayback Machine to find historical subdomains:

    ```bash
    cargo run -- -d google.com -b
    ```

4.  Save results to a file:

    ```bash
    cargo run -- -d google.com -b -o results.txt
    ```

## Output

The program will display:

-   Progress bar showing scan status
-   Valid subdomains found with clear indicators:
    -   ✓ (green): Valid subdomain
    -   ✗ (yellow): Invalid subdomain
-   Statistics including:
    -   Number of valid subdomains
    -   Number of invalid subdomains
    -   Total scan time
    -   Total domains processed

## Technical Details

### Scanner Module

-   Uses asynchronous TCP connections for validation
-   Implements efficient concurrent processing with buffered streams
-   Progress tracking with customizable display format

### Wayback Module

-   Standardized logging methods (info/success/warn/error)
-   Robust error handling with custom error types
-   Efficient subdomain extraction using regex patterns

### Dependencies

-   tokio = "1.36"
-   trust-dns-resolver = "0.23"
-   futures = "0.3"
-   clap = "4.5"
-   indicatif = "0.17"
-   colored = "3.0.0"
-   reqwest = "0.11"
-   serde = "1.0"
-   serde\_json = "1.0"
-   regex = "1.9.4"
-   url = "2.5"

### Error Handling

-   Custom error types for different failure scenarios
-   Clear error messages with context
-   Proper error propagation through the Result type

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.