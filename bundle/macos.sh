#!/bin/sh
cd "$(dirname "$0")"

cargo build --release

rm -rf Discord.app
mkdir -p Discord.app/Contents/{MacOS,Resources}
cp Info.plist Discord.app/Contents/
cp ../target/release/discord Discord.app/Contents/MacOS/
cp discord.icns Discord.app/Contents/Resources/

open .
