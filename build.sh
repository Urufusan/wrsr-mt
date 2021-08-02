#!/bin/bash
clear && cargo build || exit 1

cp target/debug/wrsr-mt.exe /z/wrsr-mt/
