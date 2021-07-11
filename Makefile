# Version control options
GIT 	 := git
VERSION  := $(shell $(GIT) describe --match 'v[0-9]*' --dirty='.m' --always --tags)
REVISION := $(shell $(GIT) rev-parse HEAD)$(shell if ! $(GIT) diff --no-ext-diff --quiet --exit-code; then echo .m; fi)

# Rust options
CARGO	 ?= cargo

.DEFAULT_GOAL := run

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: run
run:
	$(CARGO) run --package myblog-api --bin myblog-api -- \
		--mongodb-uri="${MONGODB_URI}" \
		--authority="${AUTHORITY}" --audience="${AUDIENCE}"

.PHONY: build
build:
	$(CARGO) build --release