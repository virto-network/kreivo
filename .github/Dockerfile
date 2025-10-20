# Kreivo CI Dockerfile - GitHub Actions Runner Base Image
# Provides Rust toolchain, Node.js, and smart contract tools for CI

FROM node:22-slim

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies required for GitHub Actions runner and builds
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    clang \
    llvm \
    jq \
    ca-certificates \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain with CI optimizations
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none \
    && rustup install stable nightly \
    && rustup default stable \
    && rustup target add wasm32-unknown-unknown --toolchain stable \
    && rustup target add wasm32-unknown-unknown --toolchain nightly \
    && rustup component add rust-src clippy --toolchain stable \
    && rustup component add rust-src clippy --toolchain nightly

# Install official Solidity compiler binary
RUN curl -L -o /usr/local/bin/solc https://github.com/ethereum/solidity/releases/download/v0.8.30/solc-static-linux && \
    chmod +x /usr/local/bin/solc

# Install Parity resolc binary
RUN curl -L -o /usr/local/bin/resolc https://github.com/paritytech/revive/releases/download/v0.4.1/resolc-x86_64-unknown-linux-musl && \
    chmod +x /usr/local/bin/resolc

# Set up environment for WASM builds
ENV WASM_BUILD_TOOLCHAIN=nightly

# Create docker user (GitHub Actions runner requirement)
RUN useradd -m -s /bin/bash docker \
    && mkdir -p /home/docker/actions-runner \
    && chown -R docker:docker /home/docker/actions-runner /usr/local/cargo /usr/local/rustup

USER docker
WORKDIR /home/docker/actions-runner

# Health check for container validation
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cargo --version && node --version && solc --version || exit 1

# Default command for testing the environment
CMD ["cargo", "--version"]