
comma := ,
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_HARNESS_BACKEND},harness-stylus-interpreter)
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_DRYRUN},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})dryrun,${CARGO_EXTRA_FEATURES})
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_NETWORK_TESTNET},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})network-testnet,${CARGO_EXTRA_FEATURES})
CARGO_EXTRA_FEATURES := \
	$(if ${SPN_NETWORK_CUSTOM},${CARGO_EXTRA_FEATURES}$(if ${CARGO_EXTRA_FEATURES},${comma})network-custom,${CARGO_EXTRA_FEATURES})

CARGO_OPT_COMMA := \
	$(if ${CARGO_EXTRA_FEATURES},${comma})

CARGO_OPT_FEATURES_FLAG := \
	$(if ${CARGO_EXTRA_FEATURES},--features )

CARGO_BIN_WASM32 := \
	cargo build \
		--release \
		--target wasm32-unknown-unknown

CARGO_BUILD_NATIVE := cargo build --release

RELEASE_WASM := target/wasm32-unknown-unknown/release

all: build

.PHONY: build wasm all release-wasm

wasm: \
	solver.passport-superposition-so.wasm \
	setter.passport-superposition-so.wasm \
	admin.passport-superposition-so.wasm \
	vault.passport-superposition-so.wasm \
	apply.passport-superposition-so.wasm

build: wasm passport-cli

${RELEASE_WASM}/contract-solver.wasm: $(shell find src -type f -name '*.rs')
	@${CARGO_BIN_WASM32}

release-wasm: ${RELEASE_WASM}/contract-solver.wasm

solver.passport-superposition-so.wasm: release-wasm
	@rm -f solver.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32} \
		${CARGO_OPT_FEATURES_FLAG} \
		${CARGO_EXTRA_FEATURES}
	@./wasm-post.rc \
		${RELEASE_WASM}/contract-solver.wasm \
		solver.passport-superposition-so.wasm

setter.passport-superposition-so.wasm: release-wasm
	@rm -f setter.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32} \
		--features storage-gen-apply${CARGO_OPT_COMMA}${CARGO_EXTRA_FEATURES}
	@./wasm-post.rc \
		${RELEASE_WASM}/contract-setter.wasm \
		setter.passport-superposition-so.wasm

admin.passport-superposition-so.wasm: $(shell find src -type f -name '*.rs')
	@rm -f admin.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32} \
		--features storage-gen-admin${CARGO_OPT_COMMA}${CARGO_EXTRA_FEATURES}
	@./wasm-post.rc \
		${RELEASE_WASM}/contract-admin.wasm \
		admin.passport-superposition-so.wasm

vault.passport-superposition-so.wasm: release-wasm
	@rm -f vault.passport-superposition-so.wasm
	@./wasm-post.rc \
		${RELEASE_WASM}/contract-vault.wasm \
		vault.passport-superposition-so.wasm

apply.passport-superposition-so.wasm: release-wasm
	@rm -f apply.passport-superposition-so.wasm
	@${CARGO_BIN_WASM32} \
		--features storage-gen-apply${CARGO_OPT_COMMA}${CARGO_EXTRA_FEATURES}
	@./wasm-post.rc \
		${RELEASE_WASM}/contract-apply.wasm \
		apply.passport-superposition-so.wasm

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
