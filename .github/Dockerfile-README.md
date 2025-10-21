# Kreivo CI Dockerfile

This Dockerfile provides a **GitHub Actions runner base image** for the Kreivo blockchain project CI environment, including Rust toolchain, Node.js, and smart contract tools.

## Features

- **GitHub Actions optimized** - Designed as a runner container image
- **Latest stable and nightly Rust toolchains** with WASM32 target support
- **Node.js 22** with essential packages for smart contract development
- **Solidity compiler** (`solc`) and **Parity tools** (`resolc`)
- **Rust components**: `rust-src` and `clippy` for both stable and nightly
- **Health checks** and security best practices
- **Optimized for CI/CD** with proper caching and layer optimization

## GitHub Actions Usage

### Basic Job Configuration

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    container:
      image: kreivo-ci:latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --release

  build-runtime:
    runs-on: ubuntu-latest
    container:
      image: kreivo-ci:latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release -p kreivo-runtime
```

### With Caching

```yaml
jobs:
  quality-checks:
    runs-on: ubuntu-latest
    container:
      image: kreivo-ci:latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test --release
```

## Local Development

### Building the Image

```bash
# Build the CI image
docker build -f .github/Dockerfile -t kreivo-ci:latest .
```

### Testing Locally

```bash
# Test the environment
docker run --rm kreivo-ci:latest cargo --version
docker run --rm kreivo-ci:latest node --version
docker run --rm kreivo-ci:latest solc --version

# Run commands with volume mount
docker run --rm -v $(pwd):/workspace -w /workspace kreivo-ci:latest cargo build --release
```

### Using with Docker Compose

```yaml
version: '3.8'
services:
  kreivo-dev:
    build: .
    volumes:
      - .:/workspace
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/workspace/target
    working_directory: /workspace
    environment:
      - USER=docker  # Match container user

volumes:
  cargo-cache:
  target-cache:
```

## Environment Variables

- `WASM_BUILD_TOOLCHAIN=nightly` - Toolchain used for WASM builds
- `RUSTUP_HOME=/usr/local/rustup` - Rust toolchain location
- `CARGO_HOME=/usr/local/cargo` - Cargo home directory

## Included Tools

### Rust Toolchain
- **Stable** and **Nightly** versions
- **wasm32-unknown-unknown** target for both toolchains
- **rust-src** and **clippy** components

### Node.js 22
- **solc** - Solidity compiler
- **resolc** - Parity Solidity compiler
- **yarn** - Package manager

### System Dependencies
- **build-essential** - C/C++ build tools
- **clang** & **llvm** - Additional compilers
- **protobuf-compiler** - Protocol buffer compiler
- **git** - Version control

## Security Features

- Non-root docker user execution
- Minimal Debian base image (node:22-slim)
- Proper dependency management
- Health check monitoring

## Performance Optimizations

- Single-stage build for faster runner startup
- Pre-installed toolchains reduce setup time
- Optimized layer caching
- GitHub Actions runner compatible