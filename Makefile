##
# Toolchain

NAME  = ruc
CARGO ?= cargo
BATS  ?= bats

##
# Targets

.PHONY: all
all: clean deps test

.PHONY: clean
clean:
	@$(CARGO) clean

.PHONY: deps
deps:
	@which $(CARGO) >/dev/null 2>/dev/null || (echo "ERROR: $(CARGO) not found." && false)
	@which $(BATS) >/dev/null 2>/dev/null || (echo "ERROR: $(CARGO) not found." && false)

.PHONY: test
test: fmt clippy bats
	$(CARGO) test

.PHONY: bats
bats:
	$(BATS) --verbose-run tests/*.bats

.PHONY: fmt
fmt:
	$(CARGO) fmt --all --check

.PHONY: clippy
clippy:
	$(CARGO) clippy --workspace -- -D warnings

##
# CI: same as `all` but it ensures that `bats` is installed by cloning the repo
# inside of the `tests` directory.

setup-bats:
	if [ ! -d "tests/bats" ]; then git clone https://github.com/bats-core/bats-core tests/bats; fi

.PHONY: ci
ci: setup-bats
	BATS=$(realpath tests/bats/bin/bats) $(MAKE) all
