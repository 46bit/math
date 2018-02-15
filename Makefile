.DEFAULT_GOAL := help

.PHONY: build
build: ## build everything
	cargo build

.PHONY: test
test: build ## test everything
	set -ev; \
	cargo test; \
	for FILE in $$(ls examples/*.math); do \
		target/debug/mathc "$${FILE}" "$${FILE%.*}.out" > /dev/null; \
	done

.PHONY: clean
clean: ## remove binary artifacts
	rm -rf target
	rm -f **/*.out

help:
	@awk -F":.*## " '$$2&&$$1~/^[a-zA-Z_%-]+/{printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
