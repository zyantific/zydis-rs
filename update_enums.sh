#!/usr/bin/sh
#
# Updates the enum definitions in `src/enums/generated.rs`.
# First argument needs to be the path to the `zydis-bindgen` clone.

python "$1/gen.py" "$(pwd)/zydis-c" rust > src/enums/generated.rs

