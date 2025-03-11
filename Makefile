
comma=,
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_HARNESS_BACKEND},${comma}harness-stylus-interpreter)
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_ADJUST_TIME},${comma}e2e-adjust-time)${CARGO_EXTRA_FEATURES}

CARGO_BUILD_STYLUS := \
	cargo build \
		--release \
		--target wasm32-unknown-unknown

RELEASE_WASM_OPT_9LIVES := \
	wasm-opt \
		--dce \
		--rse \
		--signature-pruning \
		--strip-debug \
		--enable-bulk-memory \
		--strip-producers \
		-Oz target/wasm32-unknown-unknown/release/passport-superposition-so.wasm \
		-o

.PHONY: build clean docs factory trading solidity

OUT_SHARE := out/Share.sol/Share.json

build: passport-superposition-so

passport-superposition-so: passport-superposition-so.wasm

passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f passport-superposition-so.wasm
	@${CARGO_BUILD_STYLUS}
	@${RELEASE_WASM_OPT_9LIVES} passport-superposition-so.wasm

clean:
	@rm -rf \
		passport-superposition-so.wasm \
		liblib9lives.rlib \
		ninelives.wasm \
		target
