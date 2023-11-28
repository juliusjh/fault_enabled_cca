use crate::constants::*;
use crate::{Poly, Polyvec};
use libc::c_int;

#[link(name = "kyber1024_clean")]
extern "C" {
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_keypair(pk: *mut u8, sk: *mut u8) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_keypair_manip(
        pk: *mut u8,
        sk: *mut u8,
        e: *mut Polyvec,
    ) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_enc(ct: *mut u8, ss: *mut u8, pk: *const u8) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_dec(ss: *mut u8, ct: *const u8, sk: *const u8) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_enc_manip(
        ct: *mut u8,
        ss: *mut u8,
        pk: *const u8,
        nu: *mut u8,
        r: *mut Polyvec,
        e1: *mut Polyvec,
        e2: *mut Poly,
    ) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_crypto_kem_dec_glitch(
        ss: *mut u8,
        ct: *const u8,
        sk: *const u8,
        manipulated_reenc_ct: *const u8,
    ) -> c_int;
    fn PQCLEAN_KYBER1024_CLEAN_poly_compress(r: *mut u8, a: *mut Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_decompress(r: *mut Poly, a: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_poly_tobytes(r: *mut u8, a: *const Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_frombytes(r: *mut Poly, a: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_poly_frommsg(k: *mut Poly, msg: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_poly_ntt(r: *mut Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_invntt_tomont(r: *mut Poly);

    fn PQCLEAN_KYBER1024_CLEAN_polyvec_compress(r: *mut u8, a: *mut Polyvec);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_decompress(r: *mut Polyvec, a: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_frombytes(r: *mut Polyvec, a: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_tobytes(r: *mut u8, a: *const Polyvec);
    fn unpack_pk(pk: *mut Polyvec, seed: *mut u8, packedpk: *const u8);
    fn pack_pk(r: *mut u8, pk: *mut Polyvec, seed: *const u8);
    fn PQCLEAN_KYBER1024_CLEAN_gen_matrix(a: *mut Polyvec, seed: *const u8, transposed: c_int);
    fn PQCLEAN_KYBER1024_CLEAN_poly_reduce(r: *mut Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_basemul_montgomery(
        r: *mut Poly,
        a: *const Poly,
        b: *const Poly,
    );
    fn PQCLEAN_KYBER1024_CLEAN_poly_add(r: *mut Poly, a: *const Poly, b: *const Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_sub(r: *mut Poly, a: *const Poly, b: *const Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_tomont(r: *mut Poly);
    fn PQCLEAN_KYBER1024_CLEAN_poly_csubq(r: *mut Poly);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_basemul_acc_montgomery(
        r: *mut Poly,
        a: *const Polyvec,
        b: *const Polyvec,
    );
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_ntt(r: *mut Polyvec);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_invntt_tomont(r: *mut Polyvec);
    fn PQCLEAN_KYBER1024_CLEAN_poly_tomsg(msg: *mut u8, a: *const Poly);
    fn PQCLEAN_KYBER1024_CLEAN_polyvec_add(r: *mut Polyvec, a: *const Polyvec, b: *const Polyvec);
    fn PQCLEAN_KYBER1024_CLEAN_montgomery_reduce(x: i32) -> i16;
}

pub fn montgomery_reduce(x: i32) -> i16 {
    unsafe { PQCLEAN_KYBER1024_CLEAN_montgomery_reduce(x) }
}

pub fn poly_tomsg(msg: &mut [u8], a: &Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_tomsg(msg.as_mut_ptr(), a);
    }
}

pub fn polyvec_ntt(r: &mut Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_ntt(r);
    }
}

pub fn polyvec_invntt(r: &mut Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_invntt_tomont(r);
    }
}

pub fn polyvec_pointwise_acc(r: &mut Poly, a: &Polyvec, b: &Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_basemul_acc_montgomery(r, a, b);
    }
}

pub fn poly_csubq(r: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_csubq(r);
    }
}

pub fn poly_tomont(r: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_tomont(r);
    }
}

pub fn poly_sub(r: &mut Poly, a: &Poly, b: &Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_sub(r, a, b);
    }
}

pub fn polyvec_add(r: &mut Polyvec, a: &Polyvec, b: &Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_add(r, a, b);
    }
}

pub fn poly_add(r: &mut Poly, a: &Poly, b: &Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_add(r, a, b);
    }
}

pub fn poly_basemul(r: &mut Poly, a: &Poly, b: &Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_basemul_montgomery(r, a, b);
    }
}

pub fn poly_reduce(r: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_reduce(r);
    }
}

pub fn poly_frommsg(k: &mut Poly, msg: &[u8]) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_frommsg(k, msg.as_ptr());
    }
}

pub fn sf_gen_matrix(a: &mut [Polyvec; K], seed: &[u8], transposed: bool) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_gen_matrix(a.as_mut_ptr(), seed.as_ptr(), transposed as c_int);
    }
}
pub fn sf_unpack_pk(pk: &mut Polyvec, seed: &mut [u8], packedpk: &[u8]) {
    unsafe {
        unpack_pk(pk, seed.as_mut_ptr(), packedpk.as_ptr());
    }
}
pub fn sf_pack_pk(r: &mut [u8], pk: &mut Polyvec, seed: &[u8]) {
    unsafe {
        pack_pk(r.as_mut_ptr(), pk, seed.as_ptr());
    }
}
pub fn polyvec_compress(r: &mut [u8], a: &mut Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_compress(r.as_mut_ptr(), a);
    }
}
pub fn polyvec_decompress(r: &mut Polyvec, a: &[u8]) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_decompress(r, a.as_ptr());
    }
}
pub fn polyvec_frombytes(r: &mut Polyvec, a: &[u8]) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_frombytes(r, a.as_ptr());
    }
}

pub fn poly_compress(r: &mut [u8], a: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_compress(r.as_mut_ptr(), a);
    }
}

pub fn poly_decompress(r: &mut Poly, a: &[u8]) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_decompress(r, a.as_ptr());
    }
}
pub fn poly_tobytes(r: &mut [u8], a: &Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_tobytes(r.as_mut_ptr(), a);
    }
}
pub fn polyvec_tobytes(r: &mut [u8], a: &Polyvec) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_polyvec_tobytes(r.as_mut_ptr(), a);
    }
}
pub fn poly_frombytes(r: &mut Poly, a: &[u8]) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_frombytes(r, a.as_ptr());
    }
}
pub fn poly_ntt(r: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_ntt(r);
    }
}
pub fn poly_invntt(r: &mut Poly) {
    unsafe {
        PQCLEAN_KYBER1024_CLEAN_poly_invntt_tomont(r);
    }
}

pub fn keygen(pk: &mut [u8], sk: &mut [u8]) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_keypair(pk.as_mut_ptr(), sk.as_mut_ptr()),
            0
        );
    }
}

pub fn keygen_manipulated(pk: &mut [u8], sk: &mut [u8], e: &mut Polyvec) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_keypair_manip(pk.as_mut_ptr(), sk.as_mut_ptr(), e),
            0
        );
    }
}

pub fn encaps(ct: &mut [u8], ss: &mut [u8], pk: &[u8]) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_enc(ct.as_mut_ptr(), ss.as_mut_ptr(), pk.as_ptr()),
            0
        );
    }
}

pub fn encaps_manipulated(
    ct: &mut [u8],
    ss: &mut [u8],
    pk: &[u8],
    nu: &mut [u8],
    e1: &mut Polyvec,
    e2: &mut Poly,
    r: &mut Polyvec,
) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_enc_manip(
                ct.as_mut_ptr(),
                ss.as_mut_ptr(),
                pk.as_ptr(),
                nu.as_mut_ptr(),
                r,
                e1,
                e2
            ),
            0
        );
    }
}

pub fn decaps(ss: &mut [u8], ct: &[u8], sk: &[u8]) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_dec(ss.as_mut_ptr(), ct.as_ptr(), sk.as_ptr()),
            0
        );
    }
}

pub fn decaps_glitch(ss: &mut [u8], ct: &[u8], sk: &[u8], manipulated_ct: &[u8]) {
    unsafe {
        assert_eq!(
            PQCLEAN_KYBER1024_CLEAN_crypto_kem_dec_glitch(
                ss.as_mut_ptr(),
                ct.as_ptr(),
                sk.as_ptr(),
                manipulated_ct.as_ptr()
            ),
            0
        );
    }
}
