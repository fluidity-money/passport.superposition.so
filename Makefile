
comma=,
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_HARNESS_BACKEND},${comma}harness-stylus-interpreter)
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_ADJUST_TIME},${comma}e2e-adjust-time)${CARGO_EXTRA_FEATURES}

CARGO_BIN_WASM32 := \
	cargo build \
		--release \
		--target wasm32-unknown-unknown

CARGO_BIN_WASI := \
	cargo build \
		--release \
		--target wasm-wasi-wasi

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

.PHONY: build

OUT_SHARE := out/Share.sol/Share.json

build: \
	solver.passport-superposition-so.wasm \
	setter.passport-superposition-so.wasm

solver.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f solver.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} solver.passport-superposition-so.wasm

setter.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f setter.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} setter.passport-superposition-so.wasm

generator.out: $(shell find src -type f -name '*.rs')
	@rm -f generator.out
	@${CARGO_BIN_WASI}
	@cp target/debug/generator generator.out

clean:
	@rm -rf \
		solver.passport-superposition-so.wasm \
		liblib9lives.rlib \
		ninelives.wasm \
		target
