.PHONY: help build clean
.DEFAULT_GOAL := help

help: ## Print this message and exit
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%10s\033[0m : %s\n", $$1, $$2}' $(MAKEFILE_LIST)

cli: cmd-cargo ## Build the campaign manager
	@cargo build --release
	@mv target/release/campaign .

debug: cmd-cargo # Build a debug version of the campaign manager
	@cargo build
	@mv target/debug/campaign .

clean: cmd-cargo ## Clean-up all build output
	@cargo clean
	@rm -f ./campaign

cmd-%: # Check that a command exists.
	@: $(if $$(command -v ${*} 2>/dev/null),,$(error Please install "${*}" first))
