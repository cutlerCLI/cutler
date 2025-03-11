#!/bin/bash
# This script reads from README.md, removes any lines that start with
# the ">" character, and writes the result back to README.md.
#
# Usage: ./generate_cargo_readme.md
#
# It assumes that README.md exists in the current directory.

file="README.md"

# Check if the input file exists
if [ ! -f "$file" ]; then
    echo "File '$file' not found."
    exit 1
fi

# Use sed in-place to delete any lines starting with ">".
sed -i '' '/^>/d' "$file"

echo "Processed '$file' and updated it."
