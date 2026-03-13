#!/bin/bash
# Build and copy resources script

PROFILE="${1:-release}"

set -e

echo -e "\033[36mBuilding project...\033[0m"

# Build
if [ "$PROFILE" = "release" ]; then
    cargo build --release
else
    cargo build
fi

# Determine target directory
if [ "$PROFILE" = "release" ]; then
    TARGET_DIR="target/release"
else
    TARGET_DIR="target/debug"
fi

echo -e "\033[36mCopying resources to $TARGET_DIR...\033[0m"

# Remove existing resources directory
TARGET_RESOURCES="$TARGET_DIR/resources"
if [ -d "$TARGET_RESOURCES" ]; then
    rm -rf "$TARGET_RESOURCES"
    echo -e "\033[33mRemoved existing resources directory\033[0m"
fi

# Copy resources directory
if [ -d "resources" ]; then
    cp -r resources "$TARGET_RESOURCES"
    echo -e "\033[32mCopied resources to $TARGET_RESOURCES\033[0m"
    
    # List copied files
    find "$TARGET_RESOURCES" -type f | while read -r file; do
        echo -e "\033[90m  $file\033[0m"
    done
else
    echo -e "\033[33mResources directory not found: resources\033[0m"
fi

echo -e "\n\033[32mBuild and copy completed!\033[0m"
echo -e "\033[36mExecutable: $TARGET_DIR/self-evolving-agent\033[0m"