use crate::constants::*;
use crate::kyber::wrapper::*;
use crate::{Ciphertext, Poly, Polyvec, PublicKey, SecretKey};
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone)]
pub struct KyberSample {
    #[pyo3(get)]
    pk: PublicKey, //NTT
    #[pyo3(get)]
    sk: SecretKey, //NTT
    #[pyo3(get)]
    ct: Ciphertext, //Normal
    #[pyo3(get)]
    ss: [u8; 32],
    #[pyo3(get)]
    nu: [u8; 32],
    #[pyo3(get)]
    e1: Polyvec, //Normal
    #[pyo3(get)]
    e2: Poly, //Normal
    #[pyo3(get)]
    r: Polyvec, //Normal
    #[pyo3(get)]
    e: Polyvec, //Normal
}

#[pymethods]
impl KyberSample {
    #[staticmethod]
    pub fn zero() -> Self {
        KyberSample {
            pk: PublicKey::zero(),
            sk: SecretKey::zero(),
            ct: Ciphertext::zero(),
            ss: [0; SZ_SS],
            e1: Polyvec::new(),
            e2: Poly::new(),
            nu: [0; 32],
            r: Polyvec::new(),
            e: Polyvec::new(),
        }
    }
    #[staticmethod]
    pub fn from_bytes(
        pkb: &[u8],
        skp: &[u8],
        ctb: &[u8],
        ss: [u8; 32],
        nu: [u8; 32],
        e1: Polyvec,
        e2: Poly,
        r: Polyvec,
        e: Polyvec,
    ) -> Self {
        KyberSample {
            pk: PublicKey::from_bytes(pkb),
            sk: SecretKey::from_bytes(skp),
            ct: Ciphertext::from_bytes(ctb),
            ss: ss,
            nu: nu,
            e1: e1,
            e2: e2,
            r: r,
            e: e,
        }
    }
    #[staticmethod]
    pub fn generate(verify_decaps: bool) -> Self {
        let mut pk: [u8; SZ_PK] = [0; SZ_PK];
        let mut sk: [u8; SZ_SK] = [0; SZ_SK];
        let mut ct: [u8; SZ_CT] = [0; SZ_CT];
        let mut ss: [u8; SZ_SS] = [0; SZ_SS];
        let mut ss2: [u8; SZ_SS] = [0; SZ_SS];
        let mut nu: [u8; 32] = [0; 32];
        let mut e1 = Polyvec::new();
        let mut e = Polyvec::new();
        let mut r = Polyvec::new();
        let mut e2 = Poly::new();
        keygen_manipulated(&mut pk, &mut sk, &mut e);
        encaps_manipulated(&mut ct, &mut ss, &pk, &mut nu, &mut e1, &mut e2, &mut r);
        if verify_decaps {
            decaps(&mut ss2, &ct, &sk);
            assert_eq!(ss, ss2);
        }
        //TODO Make move version
        KyberSample {
            ss: ss,
            pk: PublicKey::from_bytes(&pk),
            sk: SecretKey::from_bytes(&sk),
            ct: Ciphertext::from_bytes(&ct),
            nu: nu,
            e1: e1,
            e2: e2,
            r: r,
            e: e,
        }
    }
    #[staticmethod]
    pub fn generate_with_key(
        verify_decaps: bool,
        pk_k: &mut PublicKey,
        sk_k: &mut SecretKey,
        e: &mut Polyvec,
    ) -> Self {
        let mut pk: [u8; SZ_PK] = [0; SZ_PK];
        let mut sk: [u8; SZ_SK] = [0; SZ_SK];
        let mut ct: [u8; SZ_CT] = [0; SZ_CT];
        let mut ss: [u8; SZ_SS] = [0; SZ_SS];
        let mut ss2: [u8; SZ_SS] = [0; SZ_SS];
        let mut nu: [u8; 32] = [0; 32];
        let mut e1 = Polyvec::new();
        let mut r = Polyvec::new();
        let mut e2 = Poly::new();
        sk = sk_k.to_bytes();
        pk = pk_k.to_bytes();
        encaps_manipulated(&mut ct, &mut ss, &pk, &mut nu, &mut e1, &mut e2, &mut r);
        if verify_decaps {
            decaps(&mut ss2, &ct, &sk);
            assert_eq!(ss, ss2);
        }
        //TODO Make move version
        KyberSample {
            ss: ss,
            pk: PublicKey::from_bytes(&pk),
            sk: SecretKey::from_bytes(&sk),
            ct: Ciphertext::from_bytes(&ct),
            nu: nu,
            e1: e1,
            e2: e2,
            r: r,
            e: e.clone(),
        }
    }
    //Do we need to copy?
    pub fn is_valid_ct(&self, ct: &Ciphertext) -> bool {
        let mut ss: [u8; SZ_SS] = [0; SZ_SS];
        decaps_glitch(
            &mut ss,
            &ct.clone().to_bytes(),
            &self.sk.clone().to_bytes(),
            &self.ct.clone().to_bytes(),
        );
        ss == self.ss
    }
    pub fn get_msg(&self) -> Poly {
        let mut msg = Poly::new();
        poly_frommsg(&mut msg, &self.nu);
        msg
    }
    pub fn clone_me(&self) -> Self {
        self.clone()
    }
}
