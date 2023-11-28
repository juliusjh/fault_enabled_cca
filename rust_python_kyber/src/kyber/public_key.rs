use crate::constants::*;
use crate::wrapper::*;
use crate::Polyvec;
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone)]
pub struct PublicKey {
    pub a: [Polyvec; K],
    #[pyo3(get)]
    pub seeda: [u8; SEEDA],
    #[pyo3(get)]
    pub pk: Polyvec,
}

#[pymethods]
impl PublicKey {
    #[staticmethod]
    pub fn zero() -> Self {
        PublicKey {
            a: [Polyvec::new(); K],
            seeda: [0; SEEDA],
            pk: Polyvec::new(),
        }
    }
    #[staticmethod]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut res = Self::zero();
        sf_unpack_pk(&mut res.pk, &mut res.seeda, bytes);
        sf_gen_matrix(&mut res.a, &res.seeda, false);
        res
    }
    #[pyo3(name = "to_bytes")]
    pub fn to_bytes_python(&mut self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    #[getter]
    pub fn get_a(&self) -> Vec<Polyvec> {
        self.a.to_vec()
    }
}

impl PublicKey {
    pub fn to_bytes(&mut self) -> [u8; SZ_PK] {
        let mut res = [0; SZ_PK];
        sf_pack_pk(&mut res, &mut self.pk, &self.seeda);
        res
    }
}
