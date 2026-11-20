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
install-sys-deps: ## Install Linux system dependencies for Tauri v2 (Ubuntu/Debian)
	@echo -e "$(CYAN)--> Installing Linux system libraries for Tauri v2...$(RESET)"
	sudo apt update && sudo apt install -y \
		libwebkit2gtk-4.1-dev \
		libayatana-appindicator3-dev \
		librsvg2-dev \
		libxdo-dev \
		build-essential \
		curl \
		wget \
		file \
		libssl-dev

.PHONY: setup
setup: install ## Complete initial setup (installs frontend packages)
	@echo -e "$(GREEN)--> Initial setup complete!$(RESET)"

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
build: ## Build complete Tauri desktop bundle for production
	@echo -e "$(CYAN)--> Building production Tauri application...$(RESET)"
	$(PNPM) tauri build

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

##@ Quality & Testing

.PHONY: check
check: typecheck fmt-check lint test ## Run all checks (typecheck, format check, lint, tests)
	@echo -e "$(GREEN)--> All checks passed!$(RESET)"

.PHONY: typecheck
typecheck: ## Run TypeScript type checking on frontend
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

##@ Database & Utilities

.PHONY: db-path
db-path: ## Print the path to the local SQLite database
	@echo "SQLite DB: $(DB_PATH)"

.PHONY: db-clean
db-clean: ## Delete the local SQLite database file (resets tasks)
	@echo -e "$(YELLOW)--> Removing SQLite database at $(DB_PATH)...$(RESET)"
	rm -f $(DB_PATH) $(DB_PATH)-wal $(DB_PATH)-shm
	@echo -e "$(GREEN)--> Database reset completed.$(RESET)"

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
