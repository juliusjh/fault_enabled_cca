use pyo3::class::number::PyNumberProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::constants::*;
use crate::wrapper::*;
use crate::Poly;

use itertools::izip;

#[pyclass]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Polyvec {
    pub vec: [Poly; K],
}

#[pymethods]
impl Polyvec {
    #[new]
    pub fn new() -> Self {
        Polyvec {
            vec: [Poly::new(); K],
        }
    }

    #[staticmethod]
    pub fn new_from(value: i16) -> Self {
        Polyvec {
            vec: [Poly::new_from_value(value); K],
        }
    }

    #[staticmethod]
    #[name = "from_bytes_uncompressed"]
    pub fn from_bytes_python_uncompressed(bytes: Vec<u8>) -> Self {
        Self::from_bytes_uncompressed(&bytes[..])
    }

    #[staticmethod]
    #[name = "from_bytes_compressed"]
    pub fn from_bytes_python_compressed(bytes: Vec<u8>) -> Self {
        Self::from_bytes_compressed(&bytes[..])
    }

    #[name = "to_bytes_compressed"]
    pub fn to_bytes_compressed_python(&mut self) -> Vec<u8> {
        self.to_bytes_compressed().to_vec()
    }

    #[name = "to_bytes_uncompressed"]
    pub fn to_bytes_uncompressed_python(&self) -> Vec<u8> {
        self.to_bytes_uncompressed().to_vec()
    }
    pub fn to_list(&self) -> Vec<Poly> {
        self.vec.to_vec()
    }
    pub fn to_lists(&self) -> Vec<Vec<i16>> {
        self.vec.iter().map(|poly| poly.to_list()).collect()
    }
    #[staticmethod]
    pub fn new_from_list(vec: Vec<Poly>) -> PyResult<Self> {
        let mut vecar = [Poly::new(); K];
        if vec.len() != K {
            return Err(PyValueError::new_err("List has wrong length."));
        }
        for (i, c) in vec.iter().enumerate() {
            vecar[i] = *c;
        }
        Ok(Polyvec { vec: vecar })
    }
    pub fn intt(&self) -> Polyvec {
        let mut res = self.clone();
        polyvec_invntt(&mut res);
        res
    }

    pub fn ntt(&self) -> Polyvec {
        let mut res = self.clone();
        polyvec_ntt(&mut res);
        res
    }
    #[staticmethod]
    pub fn scalar(lhs: &Polyvec, rhs: &Polyvec) -> Poly {
        let mut r = Poly::new();
        polyvec_pointwise_acc(&mut r, &lhs, &rhs);
        r
    }

    #[staticmethod]
    pub fn scalar_naiv(lhs: &Polyvec, rhs: &Polyvec) -> Poly {
        let mut r = Poly::new();
        for (vl, vr) in lhs.vec.iter().zip(rhs.vec.iter()) {
            r = Poly::__add__(r, Poly::mul(*vl, *vr)).unwrap();
        }
        r
    }

    pub fn apply_matrix_left_ntt(&self, mat: Vec<Polyvec>) -> Polyvec {
        let mut res = Polyvec::new();
        for i in 0..res.vec.len() {
            polyvec_pointwise_acc(&mut res.vec[i], &mat[i], self);
        }
        res
    }

    pub fn reduce(&self) -> Polyvec {
        let mut res = Polyvec::new();
        for (mut ri, si) in res.vec.iter_mut().zip(self.vec.iter()) {
            *ri = si.reduce();
        }
        res
    }

    pub fn montgomery_reduce(&self) -> Self {
        let mut res = self.clone();
        for pol in res.vec.iter_mut() {
            *pol = pol.montgomery_reduce();
        }
        res
    }
}

#[pyproto]
impl PyNumberProtocol for Polyvec {
    fn __add__(lhs: Polyvec, rhs: Polyvec) -> PyResult<Polyvec> {
        let mut res = Polyvec::new();
        polyvec_add(&mut res, &lhs, &rhs);
        Ok(res)
    }

    fn __sub__(lhs: Polyvec, rhs: Polyvec) -> PyResult<Polyvec> {
        let mut res = Polyvec::new();
        for (mut resi, lhsi, rhsi) in izip!(res.vec.iter_mut(), lhs.vec.iter(), rhs.vec.iter()) {
            poly_sub(resi, &lhsi, &rhsi);
        }
        Ok(res)
    }
}

impl Polyvec {
    pub fn to_bytes_uncompressed(&self) -> [u8; POLYVEC_BYTES] {
        let mut bytes = [0; POLYVEC_BYTES];
        polyvec_tobytes(&mut bytes, self);
        bytes
    }

    pub fn from_bytes_uncompressed(bytes: &[u8]) -> Self {
        let mut res = Polyvec::new();
        polyvec_frombytes(&mut res, &bytes);
        res
    }

    pub fn from_bytes_compressed(bytes: &[u8]) -> Self {
        let mut res = Polyvec::new();
        polyvec_decompress(&mut res, &bytes);
        res
    }

    pub fn to_bytes_compressed(&mut self) -> [u8; POLYVEC_COMPRESSED_BYTES] {
        let mut bytes = [0; POLYVEC_COMPRESSED_BYTES];
        polyvec_compress(&mut bytes, self);
        bytes
    }
}
