#!/bin/bash

# Fix Algorithm::Sha256 -> Algorithm::Sha256d
find src -name "*.rs" -exec sed -i 's/Algorithm::Sha256\b/Algorithm::Sha256d/g' {} \;

echo "Fixed Algorithm references"
