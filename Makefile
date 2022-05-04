# Version control options
GIT 	 := git
VERSION  := $(shell $(GIT) describe --match 'v[0-9]*' --dirty='.m' --always --tags)
REVISION := $(shell $(GIT) rev-parse HEAD)$(shell if ! $(GIT) diff --no-ext-diff --quiet --exit-code; then echo .m; fi)

# Rust options
CARGO	 ?= cargo

.DEFAULT_GOAL := build

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: run-auth-service
run-auth-service:
	$(CARGO) run --package myblog-api --bin auth-service -- \
		--mongodb-uri="${MONGODB_URI}"

.PHONY: run-blog-service
run-blog-service:
	$(CARGO) run --package myblog-api --bin blog-service -- \
		--mongodb-uri="${MONGODB_URI}"
	
.PHONY: run-bot-service 
run-bot-service:
	$(CARGO) run --package myblog-api --bin bot-service -- \
		--mongodb-uri="${MONGODB_URI}" \
		--line-channel-id="${LINE_CHANNEL_ID}" \
		--line-channel-secret="${LINE_CHANNEL_SECRET}"
	
.PHONY: run-discussion-service
run-discussion-service:
	$(CARGO) run --package myblog-api --bin discussion-service -- \
		--mongodb-uri="${MONGODB_URI}"

.PHONY: build
build:
	$(CARGO) build --release