#!/bin/sh

Vs="512 768 1024"
Cwd=$(pwd)
for V in $Vs; do
    echo "Building PQClean for $V.."
    cd ./PQClean/crypto_kem/kyber$V/clean/ && make
    cd ${Cwd}
    cp Cargo$V.toml Cargo.toml
    #echo "Bulding $V.." 
    #maturin build
    echo "Installing  python_kyber$V.."
    RUSTFLAGS='-C target-cpu=native' maturin develop --release
done
