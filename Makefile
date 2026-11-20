.DEFAULT_GOAL := help

# ==============================================================================
# Variables & Settings
# ==============================================================================
SHELL       := /usr/bin/env bash
PNPM        ?= pnpm
CARGO       ?= cargo
TAURI_DIR   := src-tauri
CARGO_TOML  := $(TAURI_DIR)/Cargo.toml
DIST_DIR    := dist
TARGET_DIR  := $(TAURI_DIR)/target
DB_DIR      := $(HOME)/.local/share/rust-task-mgr
DB_PATH     := $(DB_DIR)/tasks.db

# ==============================================================================
# Color Definitions
# ==============================================================================
CYAN    := \033[36m
GREEN   := \033[32m
YELLOW  := \033[33m
BLUE    := \033[34m
MAGENTA := \033[35m
BOLD    := \033[1m
RESET   := \033[0m

##@ General

.PHONY: help
help: ## Show this help message with available commands
	@awk 'BEGIN {FS = ":.*##"; printf "\n$(BOLD)$(CYAN)TickTask (Tauri v2 + Vue 3 + Rust) - Available Make Commands$(RESET)\n"} \
		/^##@/ { printf "\n$(BOLD)$(YELLOW)%s$(RESET)\n", substr($$0, 5) } \
		/^[a-zA-Z0-9_-]+:.*##/ { printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2 } \
		END { printf "\n" }' $(MAKEFILE_LIST)

##@ Setup & Dependencies

.PHONY: install
install: ## Install frontend node dependencies (pnpm install)
	@echo -e "$(CYAN)--> Installing frontend dependencies...$(RESET)"
	$(PNPM) install

.PHONY: install-sys-deps
install-sys-deps: ## Install Linux system dependencies for Tauri v2 (Ubuntu 24.04 / Debian)
	@echo -e "$(CYAN)--> Installing Linux system libraries for Tauri v2...$(RESET)"
	sudo apt update && sudo apt install -y \
		libglib2.0-dev \
		libgtk-3-dev \
		libwebkit2gtk-4.1-dev \
		libayatana-appindicator3-dev \
		librsvg2-dev \
		libssl-dev \
		libxdo-dev \
		build-essential \
		pkg-config \
		curl \
		wget \
		file

.PHONY: setup
setup: install ## Complete initial setup (installs frontend packages)
	@echo -e "$(GREEN)--> Initial setup complete!$(RESET)"

.PHONY: env-init
env-init: ## Initialize src-tauri/.env from .env.example if not exists
	@if [ ! -f $(TAURI_DIR)/.env ]; then \
		echo -e "$(CYAN)--> Creating $(TAURI_DIR)/.env from template...$(RESET)"; \
		cp $(TAURI_DIR)/.env.example $(TAURI_DIR)/.env; \
		echo -e "$(GREEN)--> $(TAURI_DIR)/.env created! Please update with your Supabase credentials.$(RESET)"; \
	else \
		echo -e "$(YELLOW)--> $(TAURI_DIR)/.env already exists.$(RESET)"; \
	fi

##@ Development

.PHONY: dev
dev: ## Run Tauri desktop application in development mode (hot-reload)
	@echo -e "$(CYAN)--> Starting Tauri desktop dev server...$(RESET)"
	$(PNPM) tauri dev

.PHONY: dev-web
dev-web: ## Run Vue 3 frontend development server in browser (Mock data fallback)
	@echo -e "$(CYAN)--> Starting Vite frontend dev server...$(RESET)"
	$(PNPM) dev

.PHONY: preview
preview: ## Preview production frontend build locally
	@echo -e "$(CYAN)--> Previewing frontend build...$(RESET)"
	$(PNPM) preview

##@ Build & Packaging

.PHONY: build
build: ## Build complete production desktop bundles (.deb and .AppImage)
	@echo -e "$(CYAN)--> Building production Tauri bundles (.deb and .AppImage)...$(RESET)"
	$(PNPM) tauri build

.PHONY: build-deb
build-deb: ## Build only Debian package (.deb)
	@echo -e "$(CYAN)--> Building Debian (.deb) package...$(RESET)"
	$(PNPM) tauri build -b deb

.PHONY: build-appimage
build-appimage: ## Build only AppImage package (.AppImage)
	@echo -e "$(CYAN)--> Building AppImage package...$(RESET)"
	$(PNPM) tauri build -b appimage

.PHONY: build-mac
build-mac: ## Build macOS bundles (.dmg and .app) - run on macOS
	@echo -e "$(CYAN)--> Building macOS application (.dmg and .app)...$(RESET)"
	$(PNPM) tauri build --bundles dmg,app

.PHONY: build-mac-universal
build-mac-universal: ## Build Universal macOS binary for both Apple Silicon (M-series) and Intel
	@echo -e "$(CYAN)--> Building Universal macOS application (Apple Silicon + Intel)...$(RESET)"
	$(PNPM) tauri build --target universal-apple-darwin

.PHONY: build-debug
build-debug: ## Build Tauri desktop application in debug mode
	@echo -e "$(CYAN)--> Building debug Tauri application...$(RESET)"
	$(PNPM) tauri build --debug

.PHONY: build-web
build-web: ## Build frontend assets only (vue-tsc + vite build)
	@echo -e "$(CYAN)--> Building frontend assets...$(RESET)"
	$(PNPM) build

.PHONY: build-rust
build-rust: ## Build Rust backend in release mode
	@echo -e "$(CYAN)--> Building Rust backend (release)...$(RESET)"
	$(CARGO) build --manifest-path $(CARGO_TOML) --release

.PHONY: install-deb
install-deb: ## Install the generated .deb package on current system
	@deb_pkg=$$(find $(TARGET_DIR)/release/bundle/deb -name "*.deb" 2>/dev/null | head -n 1); \
	if [ -n "$$deb_pkg" ]; then \
		echo -e "$(CYAN)--> Installing $$deb_pkg...$(RESET)"; \
		sudo dpkg -i "$$deb_pkg"; \
		echo -e "$(GREEN)--> Installation finished! Launch 'TickTask' from application menu.$(RESET)"; \
	else \
		echo -e "$(YELLOW)--> No .deb file found in $(TARGET_DIR)/release/bundle/deb/. Run 'make build-deb' first.$(RESET)"; \
	fi

.PHONY: run-appimage
run-appimage: ## Launch the generated .AppImage package
	@app_img=$$(find $(TARGET_DIR)/release/bundle/appimage -name "*.AppImage" 2>/dev/null | head -n 1); \
	if [ -n "$$app_img" ]; then \
		echo -e "$(CYAN)--> Launching $$app_img...$(RESET)"; \
		chmod +x "$$app_img" && "$$app_img"; \
	else \
		echo -e "$(YELLOW)--> No .AppImage file found. Run 'make build-appimage' first.$(RESET)"; \
	fi

##@ Quality & Testing

.PHONY: check
check: typecheck fmt-check lint test ## Run all checks (typecheck, format check, lint, tests)
	@echo -e "$(GREEN)--> All checks passed!$(RESET)"

.PHONY: check-release
check-release: ## Verify Cargo compilation in release mode with embedded env
	@echo -e "$(CYAN)--> Checking Rust backend in release mode...$(RESET)"
	$(CARGO) check --manifest-path $(CARGO_TOML) --release

.PHONY: typecheck
typecheck: ## Run TypeScript type checking on frontend (vue-tsc)
	@echo -e "$(CYAN)--> Checking TypeScript types...$(RESET)"
	$(PNPM) exec vue-tsc --noEmit

.PHONY: test
test: ## Run Rust unit and integration tests
	@echo -e "$(CYAN)--> Running Rust tests...$(RESET)"
	$(CARGO) test --manifest-path $(CARGO_TOML)

.PHONY: lint
lint: ## Run Rust clippy linter
	@echo -e "$(CYAN)--> Running clippy linter...$(RESET)"
	$(CARGO) clippy --manifest-path $(CARGO_TOML)

.PHONY: fmt
fmt: ## Format Rust code with rustfmt
	@echo -e "$(CYAN)--> Formatting Rust code...$(RESET)"
	$(CARGO) fmt --manifest-path $(CARGO_TOML)

.PHONY: fmt-check
fmt-check: ## Check Rust code formatting without modifying
	@echo -e "$(CYAN)--> Checking Rust code formatting...$(RESET)"
	$(CARGO) fmt --manifest-path $(CARGO_TOML) -- --check

##@ Database & Sync

.PHONY: db-path
db-path: ## Print the path to the local SQLite database
	@echo "SQLite DB: $(DB_PATH)"

.PHONY: db-browse
db-browse: ## Open local SQLite database with sqlite3 CLI (if installed)
	@if command -v sqlite3 >/dev/null 2>&1; then \
		echo -e "$(CYAN)--> Opening SQLite database at $(DB_PATH)...$(RESET)"; \
		sqlite3 "$(DB_PATH)"; \
	else \
		echo -e "$(YELLOW)--> sqlite3 CLI is not installed. Run 'sudo apt install sqlite3' to use this command.$(RESET)"; \
	fi

.PHONY: db-clean
db-clean: ## Delete the local SQLite database file (resets tasks)
	@echo -e "$(YELLOW)--> Removing SQLite database at $(DB_PATH)...$(RESET)"
	rm -f $(DB_PATH) $(DB_PATH)-wal $(DB_PATH)-shm
	@echo -e "$(GREEN)--> Database reset completed.$(RESET)"

.PHONY: supabase-schema
supabase-schema: ## Display Supabase SQL migration script path and instructions
	@echo -e "$(CYAN)--> Supabase PostgreSQL Schema: supabase/schema.sql$(RESET)"
	@echo -e "Copy and execute the contents of $(BOLD)supabase/schema.sql$(RESET) in your Supabase SQL Editor."

##@ Cleanup

.PHONY: clean
clean: ## Remove build artifacts (dist, target)
	@echo -e "$(CYAN)--> Cleaning build artifacts...$(RESET)"
	rm -rf $(DIST_DIR) $(TARGET_DIR)
	$(CARGO) clean --manifest-path $(CARGO_TOML)
	@echo -e "$(GREEN)--> Clean complete!$(RESET)"

.PHONY: clean-all
clean-all: clean ## Remove build artifacts and node_modules
	@echo -e "$(CYAN)--> Cleaning node_modules...$(RESET)"
	rm -rf node_modules
	@echo -e "$(GREEN)--> Deep clean complete!$(RESET)"