.DEFAULT_GOAL := help

.PHONY: test
test: ## run the testsuite
	cargo build --bin mathc
	cargo test -- --test-threads=1

.PHONY: clean
clean: ## remove binary artifacts and containers
	rm -rf target

help:
	@awk -F":.*## " '$$2&&$$1~/^[a-zA-Z_%-]+/{printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
