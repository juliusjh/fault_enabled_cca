use crate::constants::*;
use crate::kyber::wrapper::*;
use crate::{Poly, Polyvec};
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone)]
pub struct Ciphertext {
    #[pyo3(get)]
    pub b: Polyvec,
    #[pyo3(get)]
    pub v: Poly,
}

#[pymethods]
impl Ciphertext {
    #[new]
    pub fn new(b: Polyvec, v: Poly) -> Self {
        Self { b: b, v: v }
    }
    #[staticmethod]
    pub fn zero() -> Self {
        Ciphertext {
            b: Polyvec::new(),
            v: Poly::new(),
        }
    }
    #[staticmethod]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut res = Self::zero();
        polyvec_decompress(&mut res.b, bytes);
        poly_decompress(&mut res.v, &bytes[POLYVEC_COMPRESSED_BYTES..SZ_CT]);
        res
    }
    #[staticmethod]
    pub fn from_bytes_list(bytes: Vec<u8>) -> Self {
        let mut res = Self::from_bytes(&bytes.to_vec());
        res
    }
    #[pyo3(name = "to_bytes_list")]
    pub fn to_bytes_python(&mut self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}
impl Ciphertext {
    pub fn to_bytes(&mut self) -> [u8; SZ_CT] {
        let mut bytes = [0; SZ_CT];
        polyvec_compress(&mut bytes, &mut self.b);
        poly_compress(&mut bytes[POLYVEC_COMPRESSED_BYTES..SZ_CT], &mut self.v);
        bytes
    }
}
