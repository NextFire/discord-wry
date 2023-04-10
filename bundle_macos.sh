#!/bin/sh
cd "$(dirname "$0")"

cargo build --release
rm -rf Discord.app
mkdir -p Discord.app/Contents/MacOS
cp target/release/discord Discord.app/Contents/MacOS/
open .
