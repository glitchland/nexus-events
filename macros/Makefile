CARGO = cargo

.PHONY: all
all:
	@cd .. && $(MAKE) build

# Run tests just for the macros crate
.PHONY: test
test:
	$(CARGO) test

# Run verbose tests
.PHONY: test-verbose
test-verbose:
	$(CARGO) test -- --nocapture

.PHONY: clippy
clippy:
	$(CARGO) clippy -- -D warnings

# Forward any other targets to the parent makefile
.DEFAULT:
	@cd .. && $(MAKE) $(MAKECMDGOALS)

# Show help
.PHONY: help
help:
	@echo "This is a forwarding Makefile for nexus-events-macros."
	@echo "Run 'make' in the parent 'nexus-events' directory for full functionality."
	@echo ""
	@echo "Available targets in this directory:"
	@echo "  all    : Build the crates (forwards to parent)"
	@echo "  test   : Run macros tests specifically"
	@echo "  help   : Show this help"
	@echo ""
	@echo "All other targets are forwarded to the parent Makefile."