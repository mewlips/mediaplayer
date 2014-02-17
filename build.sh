#!/bin/sh

mkdir -p bin

LIBS=
old_ifs="$IFS"
IFS=":"
for pkg_path in $RUST_PATH; do
    LIBS="$LIBS -L $pkg_path/lib"
done
IFS="$old_ifs"

#rustc $LIBS --link-args -lSDL src/mediaplayer/main.rs -o bin/mediaplayer
rustc $LIBS src/mediaplayer/main.rs -o bin/mediaplayer
