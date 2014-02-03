#!/bin/sh

mkdir -p bin

LIBS=
old_ifs="$IFS"
IFS=":"
for pkg_path in $RUST_PATH; do
    LIBS="$LIBS -L $pkg_path/lib"
done
IFS="$old_ifs"

rustc $LIBS --out-dir bin src/mediaplayer/main.rs
