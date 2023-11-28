use crate::constants::*;
use crate::wrapper::*;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Poly {
    coeffs: [i16; N],
}

#[pymethods]
impl Poly {
    #[new]
    pub fn new() -> Self {
        Poly { coeffs: [0; N] }
    }
    #[staticmethod]
    pub fn new_from_value(value: i16) -> Self {
        Poly { coeffs: [value; N] }
    }

    #[staticmethod]
    pub fn from_list(coeffs: Vec<i16>) -> PyResult<Self> {
        let mut coeffsar = [0; N];
        if coeffs.len() != N {
            return Err(PyValueError::new_err("List has wrong length."));
        }
        for (i, c) in coeffs.iter().enumerate() {
            coeffsar[i] = *c;
        }
        Ok(Poly { coeffs: coeffsar })
    }

    #[pyo3(name = "from_bytes_uncompressed")]
    #[staticmethod]
    pub fn from_bytes_python(bytes: Vec<u8>) -> Self {
        Self::from_bytes_uncompressed(&bytes[..])
    }
    #[pyo3(name = "from_bytes_compressed")]
    #[staticmethod]
    pub fn from_bytes_python_compressed(bytes: Vec<u8>) -> Self {
        Self::from_bytes_compressed(&bytes[..])
    }
    #[getter]
    pub fn get_coeffs(&self) -> Vec<i16> {
        self.coeffs.to_vec()
    }
    pub fn set_coeffs(&mut self, coeffs: Vec<i16>) -> PyResult<()> {
        if coeffs.len() != N {
            return Err(PyValueError::new_err("List has wrong length."));
        }
        for (i, c) in coeffs.iter().enumerate() {
            self.coeffs[i] = *c;
        }
        Ok(())
    }
    #[pyo3(name = "to_bytes_uncompressed")]
    pub fn to_bytes_python(&self) -> Vec<u8> {
        self.to_bytes_uncompressed().to_vec()
    }

    #[pyo3(name = "to_bytes_compressed")]
    pub fn to_bytes_compressed_python(&mut self) -> Vec<u8> {
        self.to_bytes_compressed().to_vec()
    }

    pub fn to_list(&self) -> Vec<i16> {
        self.coeffs.to_vec()
    }

    pub fn reduce(&self) -> Poly {
        let mut r = self.clone();
        poly_reduce(&mut r);
        r
    }

    pub fn to_msg(&self) -> Vec<u8> {
        let mut res = [0 as u8; 32];
        poly_tomsg(&mut res, &self);
        res.to_vec()
    }

    #[staticmethod]
    #[pyo3(name = "from_msg")]
    pub fn from_msg_python(msg: Vec<u8>) -> Self {
        Poly::from_msg(&msg[..])
    }

    pub fn intt(&self) -> Poly {
        let mut res = self.clone();
        poly_invntt(&mut res);
        res
    }

    pub fn ntt(&self) -> Poly {
        let mut res = self.clone();
        poly_ntt(&mut res);
        res
    }
    #[staticmethod]
    pub fn basemul(lhs: &Poly, rhs: &Poly) -> Poly {
        let mut r = Poly::new();
        poly_basemul(&mut r, &lhs, &rhs);
        r
    }

    pub fn to_mont(&self) -> Poly {
        let mut res = self.clone();
        poly_tomont(&mut res);
        res
    }

    pub fn montgomery_reduce(&self) -> Poly {
        let mut res = self.clone();
        for x in res.coeffs.iter_mut() {
            *x = montgomery_reduce(*x as i32);
        }
        res
    }
    #[staticmethod]
    pub fn mul(lhs: Poly, rhs: Poly) -> Poly {
        let mut r = Poly::new();
        for i in 0..256 {
            for j in 0..256 {
                let fac: i16 = if (i + j) as i16 >= 256 {-1} else {1};
                r.coeffs[(i + j) % 256] =
                        (
                        r.coeffs[(i + j) % 256] % 3329 +
                        (fac * i16::wrapping_mul(lhs.coeffs[i], rhs.coeffs[j])) % 3329
                    ) % 3329;
            }
        }
        r
    }
    #[staticmethod]
    pub fn mul_ntt(lhs: Poly, rhs: Poly) -> Poly {
        let mut r = Poly::new();
        poly_basemul(&mut r, &lhs, &rhs);
        r
    }
    pub fn __add__(&self, rhs: Poly) -> PyResult<Poly> {
        let mut r = Poly::new();
        poly_add(&mut r, self, &rhs);
        Ok(r)
    }
    pub fn __sub__(&self, rhs: Poly) -> PyResult<Poly> {
        let mut r = Poly::new();
        poly_sub(&mut r, &self, &rhs);
        Ok(r)
    }
    pub fn __mul__(&self, rhs: Poly) -> PyResult<Poly> {
        Ok(Poly::mul(*self, rhs))
    }
}

impl Poly {
    pub fn to_bytes_uncompressed(&self) -> [u8; POLY_BYTES] {
        let mut bytes = [0; POLY_BYTES];
        poly_tobytes(&mut bytes, self);
        bytes
    }
    pub fn to_bytes_compressed(&mut self) -> [u8; POLY_COMPRESSED_BYTES] {
        let mut bytes = [0; POLY_COMPRESSED_BYTES];
        poly_compress(&mut bytes, self);
        bytes
    }
    pub fn from_msg(msg: &[u8]) -> Self {
        let mut res = Self::new();
        poly_frommsg(&mut res, msg);
        res
    }
    pub fn from_bytes_uncompressed(bytes: &[u8]) -> Self {
        let mut res = Poly::new();
        poly_frombytes(&mut res, &bytes);
        res
    }

    pub fn from_bytes_compressed(bytes: &[u8]) -> Self {
        let mut res = Poly::new();
        poly_decompress(&mut res, &bytes);
        res
    }
}
