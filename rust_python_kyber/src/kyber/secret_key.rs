use crate::constants::*;
use crate::wrapper::*;
use crate::{Polyvec, PublicKey};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone)]
pub struct SecretKey {
    #[pyo3(get)]
    pub sk: Polyvec,
    #[pyo3(get)]
    pub hashpk: [u8; 32],
    #[pyo3(get)]
    pub pk: PublicKey,
    #[pyo3(get)]
    pub z: [u8; SZ_SS],
}

#[pymethods]
impl SecretKey {
    #[staticmethod]
    pub fn zero() -> Self {
        SecretKey {
            sk: Polyvec::new(),
            pk: PublicKey::zero(),
            z: [0; SZ_SS],
            hashpk: [0; 32],
        }
    }
    #[pyo3(name = "to_bytes")]
    pub fn to_bytes_python(&mut self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    #[pyo3(name = "to_bytes_indcpa")]
    pub fn to_bytes_indcpa_python(&mut self) -> Vec<u8> {
        self.to_bytes_indcpa().to_vec()
    }

    #[staticmethod]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut res = SecretKey {
            sk: Polyvec::new(),
            pk: PublicKey::from_bytes(&bytes[POLYVEC_BYTES..SZ_PK + POLY_BYTES]),
            z: [0; SZ_SS],
            hashpk: [0; 32],
        };
        polyvec_frombytes(&mut res.sk, bytes);
        res.hashpk.copy_from_slice(&bytes[SZ_SK - 64..SZ_SK - 32]);
        res.z.copy_from_slice(&bytes[SZ_SK - 32..SZ_SK]);
        res
    }
}
impl SecretKey {
    pub fn to_bytes_indcpa(&mut self) -> [u8; POLYVEC_BYTES] {
        self.sk.to_bytes_uncompressed()
    }

    pub fn to_bytes(&mut self) -> [u8; SZ_SK] {
        let mut res: [u8; SZ_SK] = [0; SZ_SK];
        res[0..POLYVEC_BYTES].copy_from_slice(&self.to_bytes_indcpa());
        res[POLYVEC_BYTES..POLYVEC_BYTES + SZ_PK].copy_from_slice(&self.pk.to_bytes());
        res[POLYVEC_BYTES + SZ_PK..POLYVEC_BYTES + SZ_PK + 32].copy_from_slice(&self.hashpk);
        res[POLYVEC_BYTES + SZ_PK + 32..POLYVEC_BYTES + SZ_PK + 32 + 32].copy_from_slice(&self.z);
        res
    }
}
