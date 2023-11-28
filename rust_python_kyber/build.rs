fn main() {
    println!("cargo:rustc-link-search=PQClean/crypto_kem/kyber1024/clean/");
    println!("cargo:rustc-link-search=PQClean/crypto_kem/kyber768/clean/");
    println!("cargo:rustc-link-search=PQClean/crypto_kem/kyber512/clean/");
}
