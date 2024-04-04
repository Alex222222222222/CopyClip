# optimize wasm files
# get wasm files from ./dist
# output to ./dist
# verify system is linux or macos
if [ "$(uname)" != "Linux" ] && [ "$(uname)" != "Darwin" ]; then
  echo "This script only works on Linux or macOS"
else
    echo "Optimizing wasm files..."
    # list all wasm files
    wasm_files=$(find ./dist -name "*.wasm")
    # optimize wasm files
    for wasm_file in $wasm_files
    do
        echo "Optimizing $wasm_file..."
        wasm-opt -Oz $wasm_file -o $wasm_file
        wasm-opt -Os $wasm_file -o $wasm_file
    done
fi