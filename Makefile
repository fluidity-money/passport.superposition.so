
comma=,
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_HARNESS_BACKEND},${comma}harness-stylus-interpreter)
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_ADJUST_TIME},${comma}e2e-adjust-time)${CARGO_EXTRA_FEATURES}

CARGO_BUILD_STYLUS := \
	cargo build \
		--release \
		--target wasm32-unknown-unknown \
		--bin \
		contract

RELEASE_PASSPORT_WASM := \
	wasm-opt \
		--dce \
		--rse \
		--signature-pruning \
		--strip-debug \
		--enable-bulk-memory \
		--strip-producers \
		-Oz target/wasm32-unknown-unknown/release/contract.wasm \
		-o

CARGO_BUILD_GENERATOR := \
	cargo build --bin generator

.PHONY: build

OUT_SHARE := out/Share.sol/Share.json

build: passport-superposition-so generator.out

passport-superposition-so: passport-superposition-so.wasm

passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f passport-superposition-so.wasm
	@${CARGO_BUILD_STYLUS}
	@${RELEASE_PASSPORT_WASM} passport-superposition-so.wasm

generator.out: $(shell find src -type f -name '*.rs')
	@rm -f generator.out
	@${CARGO_BUILD_GENERATOR}
	@cp target/debug/generator generator.out

clean:
	@rm -rf \
		passport-superposition-so.wasm \
		liblib9lives.rlib \
		ninelives.wasm \
		target
