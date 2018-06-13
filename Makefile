IMAGE ?= advancedtelematic/rust:x86-1.26.2
TARGET ?= x86_64-unknown-linux-gnu

.PHONY: help cli debug docker clean
.DEFAULT_GOAL := help

help: ## Print this message and exit
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%10s\033[0m : %s\n", $$1, $$2}' $(MAKEFILE_LIST)

cli: cmd-cargo ## Build the campaign manager
	@cargo build --release
	@cp target/release/campaign .

debug: cmd-cargo # Build a debug version of the campaign manager
	@cargo build
	@cp target/debug/campaign .

docker: cmd-docker ## Build the campaign manager using Docker
	@docker run --rm \
		--volume $(CURDIR):/src \
		--volume ~/.cargo/git:/root/.cargo/git \
		--volume ~/.cargo/registry:/root/.cargo/registry \
		$(IMAGE) bash -c "cargo build --release --target=$(TARGET); \
		strip target/$(TARGET)/release/campaign"
	@cp target/$(TARGET)/release/campaign .

clean: cmd-cargo ## Clean-up all build output
	@cargo clean
	@rm -f ./campaign

cmd-%: # Check that a command exists.
	@: $(if $$(command -v ${*} 2>/dev/null),,$(error Please install "${*}" first))
