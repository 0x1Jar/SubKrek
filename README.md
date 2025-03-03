# SubKrek - Subdomain Scanner

A fast and efficient subdomain scanner written in Rust that uses the Wayback Machine to discover historical subdomains.

## Features

- Fast concurrent subdomain scanning
- Historical subdomain discovery using Wayback Machine
- Customizable concurrency level
- Output results to file
- Clean and intuitive CLI interface

## Installation

1. Make sure you have Rust installed. If not, install it from [https://rustup.rs](https://rustup.rs)

2. Clone the repository:
```bash
git clone https://github.com/0x1jar/SubKrek.git
cd SubKrek
```

3. Build the project:
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

- `-h, --help`: Show help message and exit
- `-d, --domain <DOMAIN>`: Target domain to scan (required)
- `-c, --concurrency <NUMBER>`: Number of concurrent connections (default: 50)
- `-b, --wayback`: Use Wayback Machine to find historical subdomains
- `-o, --output <FILE>`: Save results to a file

To see all available options:
```bash
cargo run -- --help
```

### Examples

1. Simple scan with default settings:
```bash
cargo run -- -d google.com
```

2. Scan with increased concurrency:
```bash
cargo run -- -d google.com -c 200
```

3. Use Wayback Machine to find historical subdomains:
```bash
cargo run -- -d google.com -b
```

4. Save results to a file:
```bash
cargo run -- -d google.com -b -o results.txt
```

## Output

The program will display:
- Progress bar showing scan status
- Valid subdomains found
- Statistics including:
  - Number of valid subdomains
  - Number of invalid subdomains
  - Total scan time
  - Total domains processed

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.