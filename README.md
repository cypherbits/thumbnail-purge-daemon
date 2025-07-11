# thumbnail-purge-daemon

`thumbnail-purge-daemon` is a Rust daemon that automatically removes obsolete or unnecessary thumbnail images from your system, helping to keep storage clean and optimize disk space usage.

## Features
- Monitors thumbnail directories.
- Deletes old or unused files based on configurable criteria.
- Runs in the background as a daemon.

## Installation
1. Clone this repository:
   ```sh
   git clone <repository-url>
   cd thumbnail-purge-daemon
   ```
2. Build the project with Cargo:
   ```sh
   cargo build --release
   ```
3. The binary will be available at `target/release/thumbnail-purge-daemon`.

## Usage
Run the daemon with:
```sh
./target/release/thumbnail-purge-daemon
```

You can configure cleanup parameters by editing the configuration file (if applicable).

## Requirements
- Rust (https://www.rust-lang.org/tools/install)
- Linux

## Contributing
Contributions are welcome! Please open an issue or pull request for suggestions or improvements.

## License
This project is licensed under the MIT License.
