use crate::constants::*;
use crate::wrapper::*;
#[allow(unused_imports)]
use crate::*;
#[test]
fn test_kyber_sample() {
    let _ = KyberSample::generate(true);
}
#[test]
fn test_kyber() {
    let mut pk: [u8; SZ_PK] = [0; SZ_PK];
    let mut sk: [u8; SZ_SK] = [0; SZ_SK];
    let mut ct: [u8; SZ_CT] = [0; SZ_CT];
    let mut ct2: [u8; SZ_CT] = [0; SZ_CT];
    let mut ss: [u8; SZ_SS] = [0; SZ_SS];
    let mut ss2: [u8; SZ_SS] = [0; SZ_SS];
    let mut ss3: [u8; SZ_SS] = [0; SZ_SS];
    let mut ss4: [u8; SZ_SS] = [0; SZ_SS];
    let mut ss5: [u8; SZ_SS] = [0; SZ_SS];
    let mut nu: [u8; 32] = [0; 32];
    let mut r = Polyvec::new();
    let mut ep = Polyvec::new();
    let mut e2 = Poly::new();
    keygen(&mut pk, &mut sk);
    encaps(&mut ct, &mut ss, &pk);
    encaps_manipulated(
        &mut ct2, &mut ss4, &pk, &mut nu, &mut ep, &mut e2, &mut r,
    );
    decaps(&mut ss2, &ct, &sk);
    decaps(&mut ss5, &ct2, &sk);
    decaps_glitch(&mut ss3, &ct, &sk, &ct);
    let mut pk_st = PublicKey::from_bytes(&pk);
    let pk2 = pk_st.to_bytes();
    let mut ct_st = Ciphertext::from_bytes(&ct);
    let ct3 = ct_st.to_bytes();
    let mut sk2_st = SecretKey::from_bytes(&sk);
    let sk2 = sk2_st.to_bytes();
    assert_eq!(&sk.to_vec(), &sk2.to_vec());
    assert_eq!(&ct.to_vec(), &ct3.to_vec());
    assert_eq!(&pk.to_vec(), &pk2.to_vec());
    assert_eq!(ss, ss2);
    assert_eq!(ss2, ss3);
    assert_eq!(ss4, ss5);
}
