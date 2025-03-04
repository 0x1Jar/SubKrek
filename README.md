# SubTahu - Subdomain Discovery Tool

SubTahu is a high-performance, asynchronous subdomain discovery tool written in Rust. It combines modern subdomain scanning techniques with historical data from the Wayback Machine to provide comprehensive domain reconnaissance capabilities.

## What's New

-   **Version 0.1.0:** Initial release of SubTahu.
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

## Installation

1.  Make sure you have Rust installed. If not, install it from [https://rustup.rs](https://rustup.rs)

2.  Clone the repository:

    ```bash
    git clone https://github.com/0x1Jar/SubTahu.git
    cd SubTahu
    ```

3.  Build the project:

    ```bash
    cargo build --release
    ```

    The binary will be available at `target/release/SubTahu`

4.  **Make the binary executable from anywhere:**

    To run the program directly with `SubTahu -d example.com`, you can choose one of the following methods:

    *   **Method 1: Copy the binary to a directory in your system's PATH (recommended for system-wide access):**

        1.  Ensure the binary is executable:

            ```bash
            chmod +x target/release/SubTahu
            ```

        2.  Copy the binary to a directory in your system's PATH (e.g., `/usr/local/bin`):

            ```bash
            sudo cp target/release/SubTahu /usr/local/bin
            ```

            You might be prompted for your password.

    *   **Method 2: Add the `target/release` directory to your PATH environment variable (for access within your user account):**

        1.  Add this line to your `.bashrc` or `.zshrc` file:

            ```bash
            export PATH="$PATH:/yourpath/SubTahu/target/release"
            ```

        2.  After modifying the PATH, restart your terminal or run `source ~/.bashrc` or `source ~/.zshrc` to apply the changes.

## Usage

SubTahu is a command-line tool. To use it, open your terminal. If you followed step 4 in the installation instructions, you can run the program directly. Otherwise, navigate to the `target/release` directory first.

Basic usage:

```bash
SubTahu -d example.com
```

This command will scan the domain `example.com` for subdomains using the default settings.

### Command Line Options

-   `-h, --help`: Show help message and exit
-   `-d, --domain <DOMAIN>`: Target domain to scan (required).  Specify the domain you want to scan for subdomains.
-   `-c, --concurrency <NUMBER>`: Number of concurrent connections (default: 50).  Increase this number for faster scanning, but be mindful of your system's resources and the target server's rate limiting.
-   `-b, --wayback`: Use Wayback Machine to find historical subdomains.  This option enables the tool to search the Wayback Machine for historical subdomain records.
-   `-o, --output <FILE>`: Save results to a file.  Specify a file path to save the discovered subdomains to a file.

To see all available options:

```bash
SubTahu --help
```

This command will display a help message with all available options and their descriptions.

### Examples

1.  Simple scan with default settings:

    ```bash
    SubTahu -d google.com
    ```

    This command will scan `google.com` for subdomains using the default concurrency level (50).

2.  Scan with increased concurrency:

    ```bash
    SubTahu -d google.com -c 200
    ```

    This command will scan `google.com` for subdomains using 200 concurrent connections.

3.  Use Wayback Machine to find historical subdomains:

    ```bash
    SubTahu -d google.com -b
    ```

    This command will scan `google.com` for subdomains and also search the Wayback Machine for historical subdomains.

4.  Save results to a file:

    ```bash
    SubTahu -d google.com -b -o results.txt
    ```

    This command will scan `google.com` for subdomains, search the Wayback Machine, and save the results to a file named `results.txt`.

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
-   Implements asynchronous TCP connections for validation
-   Implements efficient concurrent processing with buffered streams
-   Progress tracking with customizable display format

### Wayback Module

-   Standardized logging methods (info/success/warn/error)
-   Robust error handling with custom error types
-   Efficient subdomain extraction using regex patterns

### Error Handling

-   Custom error types for different failure scenarios
-   Clear error messages with context
-   Proper error propagation through the Result type

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.