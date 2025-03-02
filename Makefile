CARGO = cargo
CLIPPY_OPTS = -- -D warnings
DOC_OPTS = --no-deps --open

.PHONY: all
all: build

.PHONY: build
build:
	$(CARGO) build --workspace

.PHONY: release
release:
	$(CARGO) build --workspace --release

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: test
test:
	$(CARGO) test --workspace

.PHONY: test-verbose
test-verbose:
	$(CARGO) test --workspace -- --nocapture

.PHONY: test-one
test-one:
	@if [ -z "$(TEST)" ]; then \
		echo "Usage: make test-one TEST=test_name"; \
		exit 1; \
	fi
	$(CARGO) test $(TEST) -- --nocapture

.PHONY: test-main
test-main:
	$(CARGO) test --lib

.PHONY: test-macros
test-macros:
	$(CARGO) test -p nexus-events-macros

.PHONY: clippy
clippy:
	$(CARGO) clippy --workspace $(CLIPPY_OPTS)

.PHONY: doc
doc:
	$(CARGO) doc $(DOC_OPTS)

.PHONY: check-format
check-format:
	$(CARGO) fmt -- --check

.PHONY: format
format:
	$(CARGO) fmt

.PHONY: verify
verify: clippy test check-format

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all            : Build the library (default)"
	@echo "  build          : Build the library and proc macros"
	@echo "  release        : Build in release mode"
	@echo "  clean          : Remove build artifacts"
	@echo "  test           : Run all unit tests (both crates)"
	@echo "  test-verbose   : Run tests with detailed output"
	@echo "  test-main      : Test only the main crate"
	@echo "  test-macros    : Test only the macros crate"
	@echo "  test-one       : Run a specific test (use TEST=test_name)"
	@echo "  clippy         : Run clippy for linting"
	@echo "  doc            : Generate documentation"
	@echo "  check-format   : Check code formatting"
	@echo "  format         : Format code"
	@echo "  verify         : Run all verification (clippy, test, format check)"
	@echo "  help           : Show this help"