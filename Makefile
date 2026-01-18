.PHONY: all
all:
	@cargo build
	@cargo build --release

.PHONY: test
test:
	@cargo test

.PHONY: clean
clean:
	@rm -rf release
