#!/bin/sh

Vs="512 768 1024"
Cwd=$(pwd)
for V in $Vs; do
    cp Cargo$V.toml Cargo.toml
    echo "Installing check_bp$V.."
    RUSTFLAGS='-C target-cpu=native' maturin develop --release
done

rm Cargo.toml

