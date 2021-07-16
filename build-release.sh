#!/bin/bash
clear && cargo build --release || exit 1

cp target/release/wrsr-mt.exe /z/wrsr-mt/
