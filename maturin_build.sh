#!/bin/bash

RET=`pwd` && cd python_kyber/PQClean/crypto_kem/kyber1024/clean && make && cd $RET 
RET=`pwd` && cd python_kyber/PQClean/crypto_kem/kyber768/clean && make && cd $RET 
RET=`pwd` && cd python_kyber/PQClean/crypto_kem/kyber512/clean && make && cd $RET 
source .env/bin/activate
RUSTFLAGS="-C target-cpu=native" && cd python_kyber && maturin develop --cargo-extra-args="--features ${VER}" --release
cd ..
RUSTFLAGS="-C target-cpu=native" && cd check_bp && maturin develop --cargo-extra-args="--features ${VER}" --release
cd ..
