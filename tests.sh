#!/bin/sh

#make -B

#arbos-forge test -vv

export \
	PROPTEST_MAX_SHRINK_ITERS=1 \
	RUST_BACKTRACE=1

cargo nextest run \
	--features std,errors-extra-context,storage-gen-admin,storage-gen-apply,tracing \
	-- $@
