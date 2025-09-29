# evm-proto-rs

A Rust project for compiling EVM and Cosmos SDK protobuf files to Rust code.

## Features

- **Proto Compiler**: Compile EVM and Cosmos SDK protobuf files to Rust using `cargo run -- compile`
- **Library**: Generated Rust code for EVM and Cosmos SDK protobuf types (when `proto-generated` feature is enabled)

## Getting Started

### Prerequisites

- Rust (https://rustup.rs/)

### Building

```bash
cargo build
```

### Compiling Proto Files

**EVM only:**
```bash
cargo run -- compile -e <path-to-evm-protos> -o src/prost
```

**With Cosmos SDK support:**
```bash
cargo run -- compile -e <path-to-evm-protos> -c <path-to-cosmos-sdk-protos> -o src/prost
```

**Sync from upstream repositories:**
```bash
./scripts/sync-protobuf.sh
```

Options:
- `-e, --evm`: Path to the EVM proto files directory
- `-c, --cosmos-sdk`: Path to the Cosmos SDK proto files directory (optional)
- `-o, --out`: Path to output the generated Rust sources 
- `-t, --transport`: Generate transport client/server code (optional)

### Using Generated Code

After compiling proto files, enable the `proto-generated` feature to use them:

```bash
cargo build --features proto-generated
```

### Development

This project includes VS Code tasks for building and running. Use `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) and run "Tasks: Run Task" to see available options.

## Project Structure

```
.
├── .github/
│   └── copilot-instructions.md
├── .vscode/
│   └── tasks.json
├── scripts/
│   └── sync-protobuf.sh      # Script to sync from upstream repos
├── src/
│   ├── lib.rs                # Library with proto modules
│   ├── main.rs               # Proto compiler binary
│   ├── prost/                # Generated proto files (after compilation)
│   ├── EVM_COMMIT            # EVM version info
│   └── COSMOS_SDK_COMMIT     # Cosmos SDK version info
├── target/
├── .gitignore
├── Cargo.toml
└── README.md
```