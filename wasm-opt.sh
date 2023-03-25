# optimize wasm files
# get wasm files from ./dist
# output to ./dist

# verify system is linux or macos
if [ "$(uname)" != "Linux" ] && [ "$(uname)" != "Darwin" ]; then
  echo "This script only works on Linux or macOS"
else
    echo "Optimizing wasm files..."

    # install wasm-opt
    if ! [ -x "$(command -v wasm-opt)" ]; then
    cargo install wasm-opt
    fi

    # list all wasm files
    wasm_files=$(find ./dist -name "*.wasm")

    # optimize wasm files
    for wasm_file in $wasm_files
    do
        wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code $wasm_file -o $wasm_file annoying_space_waster
        wasm-opt -Oz $wasm_file -o $wasm_file
    done

    twiggy top -n 20 ./dist/*.wasm
fi