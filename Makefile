TARGET := $(shell rustc -vV | sed -n 's/host: //p')

.PHONY: build release release-small clean

build:
	cargo build

release:
	cargo build --release

release-small:
	RUSTFLAGS="-Zunstable-options -Cpanic=immediate-abort" \
	cargo +nightly build --release --target $(TARGET) \
		-Z build-std=std,panic_abort
	@echo ""
	@ls -lh target/$(TARGET)/release/portit
	@echo "Binary: target/$(TARGET)/release/portit"

clean:
	cargo clean
