#!/bin/sh

#make -B

#arbos-forge test -vv

cargo nextest run --features std,errors-extra-context,storage-gen-admin,storage-gen-apply -- $@
