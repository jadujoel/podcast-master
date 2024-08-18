#!/bin/bash

cargo tauri build --debug

./target/debug/bundle/macos/podcast-master.app/Contents/MacOS/podcast-master
