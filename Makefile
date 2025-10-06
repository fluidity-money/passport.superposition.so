
comma := ,
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_HARNESS_BACKEND},harness-stylus-interpreter)
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_DRYRUN},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})dryrun,${CARGO_EXTRA_FEATURES})
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_NETWORK_TESTNET},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})network-testnet,${CARGO_EXTRA_FEATURES})
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_NETWORK_CUSTOM},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})network-custom,${CARGO_EXTRA_FEATURES})
CARGO_EXTRA_FEATURES := \
	$(if ${CARGO_EXTRA_FEATURES},--features ${CARGO_EXTRA_FEATURES})

CARGO_BIN_WASM32 := \
	cargo build \
		--release \
		${CARGO_EXTRA_FEATURES} \
		--target wasm32-unknown-unknown

CARGO_BUILD_NATIVE := cargo build --release

RELEASE_PASSPORT_WASM := \
	wasm-opt \
		--dce \
		--rse \
		--signature-pruning \
		--strip-debug \
		--enable-bulk-memory \
		--strip-producers \
		-Oz

RELEASE_WASM := target/wasm32-unknown-unknown/release

all: build

.PHONY: build wasm all

OUT_SHARE := out/Share.sol/Share.json

wasm: \
	solver.passport-superposition-so.wasm \
	setter.passport-superposition-so.wasm \
	admin.passport-superposition-so.wasm \
	vault.passport-superposition-so.wasm \

build: wasm passport-cli

solver.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f solver.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} \
		${RELEASE_WASM}/contract-solver.wasm \
		-o solver.passport-superposition-so.wasm

setter.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f setter.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} \
		${RELEASE_WASM}/contract-setter.wasm \
		-o setter.passport-superposition-so.wasm

admin.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f admin.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} \
		${RELEASE_WASM}/contract-admin.wasm \
		-o admin.passport-superposition-so.wasm

vault.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f vault.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${RELEASE_PASSPORT_WASM} \
		${RELEASE_WASM}/contract-vault.wasm \
		-o vault.passport-superposition-so.wasm

passport-cli: $(shell find src -type f -name '*.rs')
	@rm -f passport-cli
	@${CARGO_BUILD_NATIVE} --bin passport-cli --features std
	@cp target/release/passport-cli passport-cli

clean:
	@rm -rf \
		solver.passport-superposition-so.wasm \
		setter.passport-superposition-so.wasm \
		admin.passport-superposition-so.wasm \
		vault.passport-superposition-so.wasm \
		liblib9lives.rlib \
		ninelives.wasm \
		target
