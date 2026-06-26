.PHONY: build release run clean combine

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

clean:
	cargo clean
	rm -f combined.txt

combine:
	{ \
	  echo "=== Cargo.toml ===" && cat Cargo.toml && \
	  for f in src/*.rs; do echo "=== $$f ===" && cat "$$f"; done && \
	  for f in src/db/*.rs; do echo "=== $$f ===" && cat "$$f"; done && \
	  for f in src/ui/*.rs; do echo "=== $$f ===" && cat "$$f"; done && \
	  echo "=== doc.md ===" && cat doc.md; \
	} > combined.txt
	@echo "combined.txt generado"
