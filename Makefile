OPTIONS=

DARWIN_TARGET=x86_64-apple-darwin
LINUX_TARGET=x86_64-unknown-linux-musl
ROOT_DIR=${PWD}

BIN_NAME=noritama

LOG=noritama=debug

all: darwin
dev: darwin-build
darwin: clean darwin-build darwin-install
linux: clean linux-build linux-install
test:
	RUST_LOG=noritama=info cargo test --lib -- --nocapture $(OPTIONS)
bench:
	RUST_LOG=noritama=error cargo bench -- --nocapture $(OPTIONS)
run:
	RUST_LOG=$(LOG) cargo run --bin noritama $(OPTIONS)
decode:
	RUST_LOG=$(LOG) cargo run --bin noritama -- -d 06768284d600007f00000149 $(OPTIONS)
clean:
	cargo clean

local-build: darwin-build darwin-install linux-build linux-install
darwin-build:
	cargo build $(OPTIONS) \
	  --release \
	  --target $(DARWIN_TARGET)
darwin-install:
	cp -a assets/bin_template bin/
	cp -a target/$(DARWIN_TARGET)/release/$(BIN_NAME) bin/$(BIN_NAME)_Darwin_x86_64
linux-build:
	docker run \
	  -v $(ROOT_DIR)/.cargo-cache:/root/.cargo/registry \
	  -v $(ROOT_DIR):/root/src \
	  watawuwu/rust:nightly \
	    cargo build $(OPTIONS) --release --target $(LINUX_TARGET)
linux-install:
	cp -a target/$(LINUX_TARGET)/release/$(BIN_NAME) bin/$(BIN_NAME)_Linux_x86_64
wercker-build:
	cargo build $(OPTIONS) \
	  --release \
	  --target $(LINUX_TARGET)
