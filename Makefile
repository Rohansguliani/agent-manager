.PHONY: help install dev build test lint format clean

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install all dependencies
	cd backend && cargo fetch
	cd frontend && npm install
	npm install

dev: ## Start development servers with docker-compose
	docker-compose up

build: ## Build both backend and frontend
	cd backend && cargo build --release
	cd frontend && npm run build

test: ## Run all tests
	cd backend && cargo test
	cd frontend && npm test -- --run

test-backend: ## Run backend tests only
	cd backend && cargo test

test-frontend: ## Run frontend tests only
	cd frontend && npm test -- --run

lint: ## Run linters on both backend and frontend
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run lint

lint-backend: ## Run backend linter only
	cd backend && cargo clippy -- -D warnings

lint-frontend: ## Run frontend linter only
	cd frontend && npm run lint

format: ## Format code in both backend and frontend
	cd backend && cargo fmt
	cd frontend && npm run lint:fix

format-backend: ## Format backend code only
	cd backend && cargo fmt

format-frontend: ## Format frontend code only
	cd frontend && npm run lint:fix

type-check: ## Run TypeScript type checking
	cd frontend && npm run type-check

check: lint type-check test ## Run all checks (lint, type-check, test)

validate-docker: ## Validate docker-compose configuration
	docker-compose config

clean: ## Clean build artifacts
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules/.vite

clean-all: clean ## Clean everything including dependencies
	cd frontend && rm -rf node_modules
	cd backend && rm -rf target
	rm -rf node_modules

