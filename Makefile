# LiteTerm-Web Makefile
# Comprehensive build and development automation

.PHONY: all help install dev server client build clean test lint fmt check update check-deps

# Default target
all: help

# Directories
CLIENT_DIR := client
SERVER_DIR := server
DIST_DIR := client/dist
TARGET_DIR := server/target

# Binary names
SERVER_BIN := liteterm-web

# Help target
help: ## Show this help message
	@echo "LiteTerm-Web Build System"
	@echo "========================="
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-25s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Environment variables:"
	@echo "  RUST_LOG        Set Rust logging level (e.g., debug, info)"
	@echo "  CARGO_TARGET_DIR Set custom cargo target directory"
	@echo "  FEATURES        Set cargo features (e.g., --features hdf5)"
	@echo ""

# =============================================================================
# Setup and Installation
# =============================================================================

check-deps: ## Check if all dependencies are installed
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "Error: Rust not installed"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "Error: Cargo not installed"; exit 1; }
	@command -v node >/dev/null 2>&1 || { echo "Error: Node.js not installed"; exit 1; }
	@command -v npm >/dev/null 2>&1 || { echo "Error: npm not installed"; exit 1; }
	@echo "✓ All dependencies are installed"

install: ## Install all dependencies
	@echo "Installing dependencies..."
	@echo "Installing client dependencies..."
	cd $(CLIENT_DIR) && npm install
	@echo "✓ Dependencies installed successfully"

install-ci: ## Install dependencies for CI (uses package-lock.json)
	@echo "Installing CI dependencies..."
	cd $(CLIENT_DIR) && npm ci
	@echo "✓ CI dependencies installed"

# =============================================================================
# Development
# =============================================================================

dev: ## Start both client and server in development mode
	@echo "Starting development servers..."
	@echo "Starting server on port 3000..."
	@echo "Starting client on port 5173..."
	@$(MAKE) -j2 server client

dev-server: ## Start only the server in development mode
	@echo "Starting server in development mode..."
	cd $(SERVER_DIR) && RUST_LOG=$(RUST_LOG) cargo run

dev-client: ## Start only the client in development mode
	@echo "Starting client in development mode..."
	cd $(CLIENT_DIR) && npm run dev

watch: ## Start development with file watching (requires cargo-watch and npm-run-all)
	@echo "Starting development with file watching..."
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Error: cargo-watch not installed. Install with: cargo install cargo-watch"; exit 1; }
	@command -v npm-run-all >/dev/null 2>&1 || { echo "Error: npm-run-all not installed. Install with: npm install -g npm-run-all"; exit 1; }
	cargo-watch -x "run" -w $(SERVER_DIR)/src &
	cd $(CLIENT_DIR) && npm-run-all --parallel dev watch:client

# =============================================================================
# Build
# =============================================================================

build: ## Build both client and server for production
	@echo "Building for production..."
	@echo "Building server..."
	@$(MAKE) build-server
	@echo "Building client..."
	@$(MAKE) build-client
	@echo "✓ Build completed successfully"

build-release: ## Build both client and server in release mode
	@echo "Building release binaries..."
	@echo "Building server (release)..."
	cd $(SERVER_DIR) && cargo build --release $(FEATURES)
	@echo "Building client (production)..."
	cd $(CLIENT_DIR) && npm run build
	@echo "✓ Release build completed"
	@echo "Server: $(SERVER_DIR)/target/release/$(SERVER_BIN)"
	@echo "Client: $(CLIENT_DIR)/dist/"

build-server: ## Build the server
	@echo "Building server..."
	cd $(SERVER_DIR) && cargo build $(FEATURES)
	@echo "✓ Server built: $(SERVER_DIR)/target/debug/$(SERVER_BIN)"

build-client: ## Build the client
	@echo "Building client..."
	cd $(CLIENT_DIR) && npm run build
	@echo "✓ Client built: $(CLIENT_DIR)/dist/"

build-cross: ## Cross-compile server for multiple targets
	@echo "Cross-compiling server..."
	cd $(SERVER_DIR) && \
	cargo build --release --target x86_64-unknown-linux-gnu && \
	cargo build --release --target x86_64-apple-darwin && \
	cargo build --release --target x86_64-pc-windows-msvc && \
	cargo build --release --target aarch64-unknown-linux-gnu
	@echo "✓ Cross-compilation completed"

# =============================================================================
# Testing
# =============================================================================

test: ## Run all tests
	@echo "Running all tests..."
	@$(MAKE) test-server
	@$(MAKE) test-client
	@echo "✓ All tests passed"

test-server: ## Run server tests
	@echo "Running server tests..."
	cd $(SERVER_DIR) && cargo test --verbose

test-client: ## Run client tests
	@echo "Running client tests..."
	cd $(CLIENT_DIR) && npm test

test-integration: ## Run integration tests
	@echo "Running integration tests..."
	cd $(SERVER_DIR) && cargo test --test integration

test-coverage: ## Generate test coverage report
	@echo "Generating test coverage..."
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { echo "Error: cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"; exit 1; }
	cd $(SERVER_DIR) && cargo tarpaulin --out html --output-dir coverage

# =============================================================================
# Code Quality
# =============================================================================

lint: ## Run all linting tools
	@echo "Running linting..."
	@$(MAKE) lint-server
	@$(MAKE) lint-client
	@echo "✓ Linting completed"

lint-server: ## Lint server code
	@echo "Linting server code..."
	cd $(SERVER_DIR) && cargo clippy --all-targets --all-features -- -D warnings

lint-client: ## Lint client code
	@echo "Linting client code..."
	cd $(CLIENT_DIR) && npm run lint

fmt: ## Format all code
	@echo "Formatting code..."
	@$(MAKE) fmt-server
	@$(MAKE) fmt-client
	@echo "✓ Code formatted"

fmt-server: ## Format server code
	@echo "Formatting server code..."
	cd $(SERVER_DIR) && cargo fmt --all
	cd $(SERVER_DIR) && cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features

fmt-client: ## Format client code
	@echo "Formatting client code..."
	cd $(CLIENT_DIR) && npm run format

check: ## Run all checks (format, lint, type-check)
	@echo "Running all checks..."
	@$(MAKE) fmt-check
	@$(MAKE) lint
	@$(MAKE) type-check
	@echo "✓ All checks passed"

fmt-check: ## Check if code is formatted
	@echo "Checking code formatting..."
	cd $(SERVER_DIR) && cargo fmt --all -- --check
	cd $(CLIENT_DIR) && npm run format:check

type-check: ## Run TypeScript type checking
	@echo "Running TypeScript type checking..."
	cd $(CLIENT_DIR) && npm run check

# =============================================================================
# Server Management
# =============================================================================

server: ## Run server (debug mode)
	@echo "Running server..."
	cd $(SERVER_DIR) && RUST_LOG=$(RUST_LOG) cargo run

server-release: ## Run server (release mode)
	@echo "Running server (release)..."
	cd $(SERVER_DIR) && ./target/release/$(SERVER_BIN)

server-clean: ## Clean server build artifacts
	@echo "Cleaning server artifacts..."
	cd $(SERVER_DIR) && cargo clean

# =============================================================================
# Client Management
# =============================================================================

client: ## Run client dev server
	@echo "Running client dev server..."
	cd $(CLIENT_DIR) && npm run dev

client-preview: ## Preview production client build
	@echo "Previewing production client build..."
	cd $(CLIENT_DIR) && npm run preview

client-clean: ## Clean client build artifacts
	@echo "Cleaning client artifacts..."
	rm -rf $(DIST_DIR)
	rm -rf $(CLIENT_DIR)/node_modules/.vite

# =============================================================================
# Utilities
# =============================================================================

clean: ## Clean all build artifacts
	@echo "Cleaning all build artifacts..."
	@$(MAKE) client-clean
	@$(MAKE) server-clean
	@echo "✓ Cleaned successfully"

clean-all: clean ## Clean all artifacts including node_modules
	@echo "Cleaning all artifacts..."
	rm -rf $(CLIENT_DIR)/node_modules
	rm -rf $(CLIENT_DIR)/package-lock.json
	rm -rf .cargo
	@echo "✓ Deep clean completed"

update: ## Update dependencies
	@echo "Updating dependencies..."
	@echo "Updating Rust dependencies..."
	cd $(SERVER_DIR) && cargo update
	@echo "Updating Node.js dependencies..."
	cd $(CLIENT_DIR) && npm update
	@echo "✓ Dependencies updated"

security-audit: ## Run security audit
	@echo "Running security audit..."
	@echo "Auditing Rust dependencies..."
	cd $(SERVER_DIR) && cargo audit
	@echo "Auditing Node.js dependencies..."
	cd $(CLIENT_DIR) && npm audit

# =============================================================================
# Documentation
# =============================================================================

docs: ## Generate documentation
	@echo "Generating documentation..."
	@echo "Generating Rust documentation..."
	cd $(SERVER_DIR) && cargo doc --no-deps
	@echo "✓ Documentation generated"
	@echo "Open: $(SERVER_DIR)/target/doc/liteterm_web/index.html"

docs-serve: docs ## Generate and serve documentation
	@echo "Serving documentation at http://localhost:3001"
	cd $(SERVER_DIR) && cargo doc --no-deps --open

# =============================================================================
# Docker
# =============================================================================

docker-build: ## Build Docker image
	@echo "Building Docker image..."
	docker build -t liteterm-web:latest .
	@echo "✓ Docker image built: liteterm-web:latest"

docker-run: ## Run Docker container
	@echo "Running Docker container..."
	docker run -d --name liteterm-web -p 3000:3000 \
		-v $$(pwd)/server/config:/opt/liteterm/config \
		-v $$(pwd)/data:/workspace \
		liteterm-web:latest

docker-stop: ## Stop Docker container
	@echo "Stopping Docker container..."
	docker stop liteterm-web || true

docker-clean: ## Remove Docker image
	@echo "Removing Docker image..."
	docker rmi liteterm-web:latest || true

# =============================================================================
# Deployment
# =============================================================================

package: build-release ## Create distribution package
	@echo "Creating distribution package..."
	mkdir -p dist
	cp -r $(SERVER_DIR)/target/release/$(SERVER_BIN) dist/
	cp -r $(CLIENT_DIR)/dist dist/client
	cp -r server/config dist/
	tar -czf liteterm-web-$(shell date +%Y%m%d).tar.gz -C dist .
	@echo "✓ Package created: liteterm-web-$(shell date +%Y%m%d).tar.gz"

# =============================================================================
# CI/CD
# =============================================================================

ci: install-ci check test ## Run CI pipeline locally
	@echo "✓ CI pipeline completed successfully"

# =============================================================================
# Development Tools
# =============================================================================

install-tools: ## Install development tools
	@echo "Installing development tools..."
	cargo install cargo-watch cargo-audit cargo-expand
	npm install -g npm-run-all typescript
	@echo "✓ Development tools installed"

# =============================================================================
# Project Statistics
# =============================================================================

info: ## Show project information
	@echo "Project Information"
	@echo "=================="
	@echo "Project: LiteTerm-Web"
	@echo "Client: $(CLIENT_DIR) (Svelte 5 + TypeScript)"
	@echo "Server: $(SERVER_DIR) (Rust + Axum)"
	@echo ""
	@command -v rustc >/dev/null 2>&1 && echo "Rust: $$(rustc --version)" || echo "Rust: Not installed"
	@command -v node >/dev/null 2>&1 && echo "Node.js: $$(node --version)" || echo "Node.js: Not installed"
	@command -v npm >/dev/null 2>&1 && echo "npm: $$(npm --version)" || echo "npm: Not installed"
	@echo ""
	@echo "Build Directories:"
	@echo "  Server target: $(SERVER_DIR)/target"
	@echo "  Client dist:   $(CLIENT_DIR)/dist"
	@echo ""
	@echo "Current Configuration:"
	@echo "  Features: $(FEATURES)"
	@echo "  RUST_LOG: $(RUST_LOG)"

# =============================================================================
# Quick Commands
# =============================================================================

# Quick dev with logs
dev-debug: ## Start development with debug logging
	RUST_LOG=debug $(MAKE) dev

# Quick build and run
run-prod: build-release server-release ## Build and run release server

# Quick test and lint
verify: lint test ## Run linting and tests

# Development cycle
watch-test: ## Run tests in watch mode
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Error: cargo-watch not installed"; exit 1; }
	cargo-watch -x "test" -w $(SERVER_DIR)/src &
	cd $(CLIENT_DIR) && npm run test:watch

# =============================================================================
# Platform-specific
# =============================================================================

# macOS specific
mac-setup: ## Setup development environment on macOS
	@echo "Setting up macOS development environment..."
	@command -v brew >/dev/null 2>&1 || { echo "Error: Homebrew not installed"; exit 1; }
	brew install rust cargo node
	$(MAKE) install-tools
	@echo "✓ macOS setup completed"

# Linux specific
linux-setup: ## Setup development environment on Linux (Ubuntu/Debian)
	@echo "Setting up Linux development environment..."
	sudo apt-get update
	sudo apt-get install -y build-essential curl wget git
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	source ~/.cargo/env
	curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
	sudo apt-get install -y nodejs
	$(MAKE) install-tools
	@echo "✓ Linux setup completed"

# =============================================================================
# Advanced
# =============================================================================

# Analyze bundle size
analyze-bundle: build-client ## Analyze client bundle size
	@echo "Analyzing bundle size..."
	@command -v npx >/dev/null 2>&1 || { echo "Error: npx not available"; exit 1; }
	npx vite-bundle-analyzer $(CLIENT_DIR)/dist

# Benchmark
benchmark: ## Run performance benchmarks
	@echo "Running performance benchmarks..."
	cd $(SERVER_DIR) && cargo bench
	@echo "✓ Benchmarks completed"

# Generate API client
gen-client: ## Generate API client from OpenAPI spec
	@command -v openapi-generator-cli >/dev/null 2>&1 || { echo "Error: openapi-generator-cli not installed"; exit 1; }
	@echo "Generating API client..."
	openapi-generator-cli generate -i api-spec.yaml -g typescript-axios -o $(CLIENT_DIR)/src/api/client
	@echo "✓ API client generated"

# =============================================================================
# Maintenance
# =============================================================================

# Check for outdated dependencies
outdated: ## Check for outdated dependencies
	@echo "Checking for outdated dependencies..."
	cd $(SERVER_DIR) && cargo outdated || echo "Install cargo-outdated: cargo install cargo-outdated"
	cd $(CLIENT_DIR) && npm outdated

# Remove unused dependencies
deptox: ## Remove unused dependencies
	@echo "Removing unused dependencies..."
	cd $(SERVER_DIR) && cargo machete
	cd $(CLIENT_DIR) && npm prune
	@echo "✓ Dependency cleanup completed"

# =============================================================================
# Shell Integration
# =============================================================================

# Add to shell profile for convenience
shell-setup: ## Add useful aliases to shell profile
	@echo "Adding aliases to ~/.bashrc or ~/.zshrc..."
	@echo "" >> ~/.bashrc
	@echo "# LiteTerm-Web aliases" >> ~/.bashrc
	@echo "alias ltm-dev='make dev'" >> ~/.bashrc
	@echo "alias ltm-build='make build'" >> ~/.bashrc
	@echo "alias ltm-test='make test'" >> ~/.bashrc
	@echo "alias ltm-clean='make clean'" >> ~/.bashrc
	@echo "✓ Shell aliases added. Reload with: source ~/.bashrc"
