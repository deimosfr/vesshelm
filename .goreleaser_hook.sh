#!/usr/bin/env bash

# Map amd64->x86_64 and arm64->aarch64
arch=${1/amd64/x86_64}; arch=${arch/arm64/aarch64}
name=$3; [[ $2 == "windows" ]] && name+=".exe"

# Find source binary and destination directory
src=$(find artifacts -path "*$arch*$2*/$name" -print -quit)
dst=$(find dist -type d -name "${3}_${2}_${1}*" -print -quit)

# Copy and chmod if both found
[[ -f "$src" && -d "$dst" ]] && cp "$src" "$dst/$name" && chmod +x "$dst/$name"