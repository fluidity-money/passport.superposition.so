
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

WASMOPT_PASSPORT := \
	wasm-opt \
		--dce \
		--rse \
		--signature-pruning \
		--strip-debug \
		--enable-bulk-memory \
		--strip-producers \
		--strip \
		-Oz

RELEASE_WASM := target/wasm32-unknown-unknown/release

all: build

.PHONY: build wasm all

wasm: \
	solver.passport-superposition-so.wasm \
	setter.passport-superposition-so.wasm \
	admin.passport-superposition-so.wasm \
	vault.passport-superposition-so.wasm \
	apply.passport-superposition-so.wasm

build: wasm passport-cli

release-wasm: ${RELEASE_WASM}/contract-solver.wasm $(shell find src -type f -name '*.rs')
	@${CARGO_BIN_WASM32}

solver.passport-superposition-so.wasm: release-wasm
	@rm -f solver.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${WASMOPT_PASSPORT} \
		${RELEASE_WASM}/contract-solver.wasm \
		-o solver.passport-superposition-so.wasm
	@./check-codesize.rc solver.passport-superposition-so.wasm

setter.passport-superposition-so.wasm: release-wasm
	@rm -f setter.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32}
	@${WASMOPT_PASSPORT} \
		${RELEASE_WASM}/contract-setter.wasm \
		-o setter.passport-superposition-so.wasm
	@./check-codesize.rc setter.passport-superposition-so.wasm

admin.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f admin.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32} --features storage-gen-admin
	@${WASMOPT_PASSPORT} \
		${RELEASE_WASM}/contract-admin.wasm \
		-o admin.passport-superposition-so.wasm
	@./check-codesize.rc admin.passport-superposition-so.wasm

vault.passport-superposition-so.wasm: release-wasm
	@rm -f vault.passport-superposition-so.wasm
	@${WASMOPT_PASSPORT} \
		${RELEASE_WASM}/contract-vault.wasm \
		-o vault.passport-superposition-so.wasm
	@./check-codesize.rc vault.passport-superposition-so.wasm

apply.passport-superposition-so.wasm: release-wasm
	@rm -f apply.passport-superposition-so.wasm
	@${WASMOPT_PASSPORT} \
		${RELEASE_WASM}/contract-apply.wasm \
		-o apply.passport-superposition-so.wasm
	@./check-codesize.rc apply.passport-superposition-so.wasm

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
		apply.passport-superposition-so.wasm \
		target
