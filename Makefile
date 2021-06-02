# Version control options
GIT 	 := git
VERSION  := $(shell $(GIT) describe --match 'v[0-9]*' --dirty='.m' --always --tags)
REVISION := $(shell $(GIT) rev-parse HEAD)$(shell if ! $(GIT) diff --no-ext-diff --quiet --exit-code; then echo .m; fi)

# Rust options
CARGO	?= cargo

.DEFAULT_GOAL := run

.PHONY: run
run:
	$(CARGO) run --color=always