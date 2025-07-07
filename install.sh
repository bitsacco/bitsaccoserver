#!/bin/bash
set -e

echo "🚀 Installing Bitsacco Server Development Environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check OS
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    print_error "Unsupported OS: $OSTYPE"
    exit 1
fi

print_status "Detected OS: $OS"

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_status "Rust installed"
else
    print_status "Rust already installed"
fi

# Ensure we have at least Rust 1.83
RUST_VERSION=$(rustc --version | awk '{print $2}')
REQUIRED_VERSION="1.83.0"
if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    print_warning "Rust version $RUST_VERSION is too old. Updating to latest stable..."
    rustup update stable
    rustup default stable
fi

# Add WASM target
echo "📦 Adding WASM target..."
rustup target add wasm32-unknown-unknown
print_status "WASM target added"

# Install cargo-leptos
if ! command -v cargo-leptos &> /dev/null; then
    echo "📦 Installing cargo-leptos..."
    cargo install cargo-leptos@0.2.28
    print_status "cargo-leptos installed"
else
    print_status "cargo-leptos already installed"
fi

# Install Node.js if not present
if ! command -v node &> /dev/null; then
    echo "📦 Installing Node.js..."
    if [[ "$OS" == "linux" ]]; then
        # Install via NodeSource
        curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
        sudo apt-get install -y nodejs
    elif [[ "$OS" == "macos" ]]; then
        # Check for Homebrew
        if command -v brew &> /dev/null; then
            brew install node
        else
            print_warning "Homebrew not found. Installing via installer..."
            # Download and suggest manual installation
            echo "Please install Node.js from https://nodejs.org/"
            exit 1
        fi
    fi
    print_status "Node.js installed"
else
    print_status "Node.js already installed"
fi

# Install TailwindCSS and dependencies
echo "📦 Installing TailwindCSS..."
npm install -g tailwindcss @tailwindcss/forms @tailwindcss/typography
print_status "TailwindCSS installed"

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    echo "📦 Installing Docker..."
    if [[ "$OS" == "linux" ]]; then
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        sudo usermod -aG docker $USER
        rm get-docker.sh
        print_warning "Docker installed. Please log out and back in for group changes to take effect."
    elif [[ "$OS" == "macos" ]]; then
        if command -v brew &> /dev/null; then
            brew install --cask docker
        else
            print_warning "Please install Docker Desktop from https://www.docker.com/products/docker-desktop/"
        fi
    fi
    print_status "Docker installation completed"
else
    print_status "Docker already installed"
fi

# Install Docker Compose if not present
if ! command -v docker-compose &> /dev/null; then
    echo "📦 Installing Docker Compose..."
    if [[ "$OS" == "linux" ]]; then
        sudo apt-get update
        sudo apt-get install -y docker-compose-plugin
    fi
    print_status "Docker Compose installed"
else
    print_status "Docker Compose already installed"
fi

# Install project dependencies
echo "📦 Installing project dependencies..."
npm install

# Set up environment file
if [ ! -f .env ]; then
    cp .env.example .env
    print_status "Environment file created"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "Next steps:"
echo "1. Start services: docker-compose up -d postgres keycloak"
echo "2. Run development server: npm run dev"
echo "3. Open http://localhost:3000"
echo ""
echo "Note: If Docker was just installed, you may need to log out and back in."