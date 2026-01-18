.PHONY: all
all:
	@cargo build
	@cargo build --release

.PHONY: test
test:
	@cargo test

.PHONY: clean_build
clean_build:
	@target/release/rsb clean
	@target/release/rsb build -j 4

.PHONY: graph
graph:
	@target/release/rsb graph --view mermaid

.PHONY: rsb_build
rsb_build:
	@target/release/rsb build -v

.PHONY: rsb_clean
rsb_clean:
	@target/release/rsb clean -v

.PHONY: clean
clean:
	@rm -rf release
