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
FFMPEG_2_2_CFGS="--cfg avcodec55 --cfg avformat55"
CFGS="--cfg sdl --cfg ffmpeg $FFMPEG_2_2_CFGS"

rustc $LIBS src/v2/main.rs -o bin/v2 $CFGS || exit 1
rustc --test $LIBS src/v2/main.rs -o bin/v2_test $CFGS || exit 1
bin/v2_test
