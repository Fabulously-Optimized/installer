#!/usr/bin/env bash
mkdir -p dist

declare -A -r GOOS_LIST=(
    ["linux"]="unknown-linux-gnu"
    ["darwin"]="apple-darwin"
    ["windows"]="pc-windows-msvc.exe"
)

declare -A GOARCH_LIST=(
    ["amd64"]="x86_64"
    ["arm64"]="aarch64"
)
for GOOS in "${!GOOS_LIST[@]}"; do
    for GOARCH in "${!GOARCH_LIST[@]}"; do
        export GOOS GOARCH
        echo Building for $GOOS $GOARCH
        go build -o "dist/verifier-${GOARCH_LIST[$GOARCH]}-${GOOS_LIST[$GOOS]}"
    done
done