#!/bin/sh

make -B

arbos-forge test -vv

cargo test --features std,errors-extra-context,storage-gen-admin,storage-gen-apply -- $@
