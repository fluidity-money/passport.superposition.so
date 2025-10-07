#!/bin/sh

cargo test --features std,storage-gen-admin,storage-gen-apply -- $@
