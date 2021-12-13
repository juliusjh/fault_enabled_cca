use pyo3::prelude::*;

#[cfg(feature = "kyber1024")]
pub const SZ_PK: usize = 1568;
#[cfg(feature = "kyber1024")]
pub const SZ_SK: usize = 3168;
#[cfg(feature = "kyber1024")]
pub const SZ_CT: usize = 1568;

#[cfg(feature = "kyber768")]
pub const SZ_PK: usize = 1184;
#[cfg(feature = "kyber768")]
pub const SZ_SK: usize = 2400;
#[cfg(feature = "kyber768")]
pub const SZ_CT: usize = 1088;

#[cfg(feature = "kyber512")]
pub const SZ_PK: usize = 800;
#[cfg(feature = "kyber512")]
pub const SZ_SK: usize = 1632;
#[cfg(feature = "kyber512")]
pub const SZ_CT: usize = 768;

pub const SZ_SS: usize = 32;
#[cfg(feature = "kyber1024")]
pub const POLYVEC_COMPRESSED_BYTES: usize = K * 352;
#[cfg(feature = "kyber768")]
pub const POLYVEC_COMPRESSED_BYTES: usize = K * 320;
#[cfg(feature = "kyber512")]
pub const POLYVEC_COMPRESSED_BYTES: usize = K * 320;
#[cfg(feature = "kyber1024")]
pub const POLY_COMPRESSED_BYTES: usize = 160;
#[cfg(feature = "kyber768")]
pub const POLY_COMPRESSED_BYTES: usize = 128;
#[cfg(feature = "kyber512")]
pub const POLY_COMPRESSED_BYTES: usize = 128;

pub const POLY_BYTES: usize = 384;
pub const POLYVEC_BYTES: usize = K * POLY_BYTES;
pub const SEEDA: usize = 32;

pub const Q: usize = 3329;
pub const N: usize = 256;
#[cfg(feature = "kyber1024")]
pub const K: usize = 4;
#[cfg(feature = "kyber768")]
pub const K: usize = 3;
#[cfg(feature = "kyber512")]
pub const K: usize = 2;

//5 ot 4?
#[cfg(feature = "kyber1024")]
pub const DV: usize = 5;
#[cfg(feature = "kyber768")]
pub const DV: usize = 4;
#[cfg(feature = "kyber512")]
pub const DV: usize = 3;

#[cfg(feature = "kyber512")]
pub const ETA: usize = 3;
#[cfg(feature = "kyber768")]
pub const ETA: usize = 2;
#[cfg(feature = "kyber1024")]
pub const ETA: usize = 2;


//Best way to do this?
//Keeps them bundled in one module like obj
#[pyclass]
pub struct KyberConstants {}

#[allow(non_snake_case)]
#[pymethods]
impl KyberConstants {
    #[staticmethod]
    pub fn SZ_PK() -> usize {
        SZ_PK
    }
    #[staticmethod]
    pub fn SZ_SK() -> usize {
        SZ_SK
    }
    #[staticmethod]
    pub fn SZ_CT() -> usize {
        SZ_CT
    }
    #[staticmethod]
    pub fn SZ_SS() -> usize {
        SZ_SS
    }
    #[staticmethod]
    pub fn POLY_BYTES() -> usize {
        POLY_BYTES
    }
    #[staticmethod]
    pub fn POLYVEC_BYTES() -> usize {
        POLYVEC_BYTES
    }
    #[staticmethod]
    pub fn POLYVEC_COMPRESSED_BYTES() -> usize {
        POLYVEC_COMPRESSED_BYTES
    }
    #[staticmethod]
    pub fn SZ_SEED() -> usize {
        SEEDA
    }
    #[staticmethod]
    pub fn Q() -> usize {
        Q
    }
    #[staticmethod]
    pub fn DV() -> usize {
        DV
    }

    #[staticmethod]
    pub fn K() -> usize {
        K
    }
    #[staticmethod]
    pub fn ETA() -> usize {
        ETA
    }

}
