# `make fmt` and `make check-fmt` call mdtablefix directly. `--git` selects the
# Markdown files Git tracks and `--include-untracked` adds the untracked files
# Git does not ignore, so a new document is formatted before it is staged.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at its
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences

.PHONY: help all clean test build release lint fmt check-fmt markdownlint nixie


TARGET ?= splitters

CARGO ?= cargo
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
# The build standard: every `rustflags` source in `.cargo/config.toml` carries
# the parallel frontend, and the Linux source adds `mold`. Assigning
# `RUSTFLAGS` replaces those sources outright, so the targets that assign it
# restate the flags here. Coverage and release builds deliberately take neither.
# They add them to any inherited `RUSTFLAGS` (setup-rust exports one in CI)
# instead of replacing it.
STANDARD_THREADS_FLAG ?= -Zthreads=8
STANDARD_MOLD_FLAG ?= -Clink-arg=-fuse-ld=mold
BUILD_HOST_OS ?= $(shell uname -s)
STANDARD_RUSTFLAGS = $(STANDARD_THREADS_FLAG)$(if $(filter Linux,$(BUILD_HOST_OS)), $(STANDARD_MOLD_FLAG))
# Release builds take neither flag: assigning `RUSTFLAGS`, even to an empty
# inherited value, displaces every `rustflags` source in the configuration.
RELEASE_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS-}"
# Debug builds keep a caller's exported flags and add the standard ones,
# since an inherited `RUSTFLAGS` would otherwise displace the configuration.
DEBUG_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)"
# Whitaker's Dylint driver runs on its own pinned toolchain, which need not
# carry the Cranelift component the development profile selects, so its
# check builds take LLVM.
WHITAKER_CODEGEN_BACKEND ?= llvm
RUSTDOC_FLAGS ?=
RUSTDOC_FLAGS := -D warnings $(RUSTDOC_FLAGS)
CARGO_FLAGS ?= --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
TEST_CMD := $(if $(shell $(CARGO) nextest --version 2>/dev/null),nextest run,test)
MDLINT ?= markdownlint-cli2
NIXIE ?= nixie

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests with warnings treated as errors
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) $(TEST_CMD) $(TEST_FLAGS) $(BUILD_JOBS)
ifneq ($(TEST_CMD),test)
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) test --doc --workspace --all-features
endif

target/%/$(TARGET): ## Build binary in debug or release mode
	$(if $(findstring release,$(@)),$(RELEASE_RUSTFLAGS),$(DEBUG_RUSTFLAGS)) $(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release) --bin $(TARGET)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --no-deps
	$(CARGO) clippy $(CLIPPY_FLAGS)
	@command -v whitaker >/dev/null 2>&1 && \
		CARGO_PROFILE_DEV_CODEGEN_BACKEND=$(WHITAKER_CODEGEN_BACKEND) RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" whitaker --all -- $(CARGO_FLAGS) || \
		{ echo "whitaker not found on PATH; skipping whitaker lint. Install whitaker to run this check."; }

typecheck: ## Type-check without building
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) +nightly fmt --all
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	$(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
