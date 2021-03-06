#include "indcpa.h"
#include "kem.h"
#include "params.h"
#include "randombytes.h"
#include "symmetric.h"
#include "verify.h"
#include <stddef.h>
#include <string.h>
#include <stdint.h>

/*************************************************
* Name:        PQCLEAN_KYBER512_CLEAN_crypto_kem_keypair
*
* Description: Generates public and private key
*              for CCA-secure Kyber key encapsulation mechanism
*
* Arguments:   - unsigned char *pk: pointer to output public key
*                (an already allocated array of KYBER_PUBLICKEYBYTES bytes)
*              - unsigned char *sk: pointer to output private key
*                (an already allocated array of KYBER_SECRETKEYBYTES bytes)
*
* Returns 0 (success)
**************************************************/
int PQCLEAN_KYBER512_CLEAN_crypto_kem_keypair(unsigned char pk[KYBER_PUBLICKEYBYTES],
        unsigned char sk[KYBER_SECRETKEYBYTES]) {
    size_t i;
    PQCLEAN_KYBER512_CLEAN_indcpa_keypair(pk, sk);
    for (i = 0; i < KYBER_INDCPA_PUBLICKEYBYTES; i++) {
        sk[i + KYBER_INDCPA_SECRETKEYBYTES] = pk[i];
    }
    hash_h(sk + KYBER_SECRETKEYBYTES - 2 * KYBER_SYMBYTES, pk, KYBER_PUBLICKEYBYTES);
    /* Value z for pseudo-random output on reject */
    randombytes(sk + KYBER_SECRETKEYBYTES - KYBER_SYMBYTES, KYBER_SYMBYTES);
    return 0;
}

int PQCLEAN_KYBER512_CLEAN_crypto_kem_keypair_manip(unsigned char pk[KYBER_PUBLICKEYBYTES],
        unsigned char sk[KYBER_SECRETKEYBYTES], polyvec* ecp) {
    size_t i;
    PQCLEAN_KYBER512_CLEAN_indcpa_keypair_manip(pk, sk, ecp);
    for (i = 0; i < KYBER_INDCPA_PUBLICKEYBYTES; i++) {
        sk[i + KYBER_INDCPA_SECRETKEYBYTES] = pk[i];
    }
    hash_h(sk + KYBER_SECRETKEYBYTES - 2 * KYBER_SYMBYTES, pk, KYBER_PUBLICKEYBYTES);
    /* Value z for pseudo-random output on reject */
    randombytes(sk + KYBER_SECRETKEYBYTES - KYBER_SYMBYTES, KYBER_SYMBYTES);
    return 0;
}

/*************************************************
* Name:        PQCLEAN_KYBER512_CLEAN_crypto_kem_enc
*
* Description: Generates cipher text and shared
*              secret for given public key
*
* Arguments:   - unsigned char *ct: pointer to output cipher text
*                (an already allocated array of KYBER_CIPHERTEXTBYTES bytes)
*              - unsigned char *ss: pointer to output shared secret
*                (an already allocated array of KYBER_SSBYTES bytes)
*              - const unsigned char *pk: pointer to input public key
*                (an already allocated array of KYBER_PUBLICKEYBYTES bytes)
*
* Returns 0 (success)
**************************************************/
int PQCLEAN_KYBER512_CLEAN_crypto_kem_enc(unsigned char ct[KYBER_CIPHERTEXTBYTES],
        unsigned char ss[KYBER_SSBYTES],
        const unsigned char pk[KYBER_PUBLICKEYBYTES]) {
    uint8_t buf[2 * KYBER_SYMBYTES];
    /* Will contain key, coins */
    uint8_t kr[2 * KYBER_SYMBYTES];

    randombytes(buf, KYBER_SYMBYTES);
    /* Don't release system RNG output */
    hash_h(buf, buf, KYBER_SYMBYTES);

    /* Multitarget countermeasure for coins + contributory KEM */
    hash_h(buf + KYBER_SYMBYTES, pk, KYBER_PUBLICKEYBYTES);
    hash_g(kr, buf, 2 * KYBER_SYMBYTES);

    /* coins are in kr+KYBER_SYMBYTES */
    PQCLEAN_KYBER512_CLEAN_indcpa_enc(ct, buf, pk, kr + KYBER_SYMBYTES);

    /* overwrite coins in kr with H(c) */
    hash_h(kr + KYBER_SYMBYTES, ct, KYBER_CIPHERTEXTBYTES);
    /* hash concatenation of pre-k and H(c) to k */
    kdf(ss, kr, 2 * KYBER_SYMBYTES);
    return 0;
}

int PQCLEAN_KYBER512_CLEAN_crypto_kem_enc_manip(unsigned char ct[KYBER_CIPHERTEXTBYTES],
        unsigned char ss[KYBER_SSBYTES],
        const unsigned char pk[KYBER_PUBLICKEYBYTES],
                                        uint8_t* nu,
                                        polyvec* r,
                                        polyvec* e1,
                                        poly* e2) {
    uint8_t buf[2 * KYBER_SYMBYTES];
    /* Will contain key, coins */
    uint8_t kr[2 * KYBER_SYMBYTES];

    randombytes(buf, KYBER_SYMBYTES);
    /* Don't release system RNG output */
    hash_h(buf, buf, KYBER_SYMBYTES);

    /* Multitarget countermeasure for coins + contributory KEM */
    hash_h(buf + KYBER_SYMBYTES, pk, KYBER_PUBLICKEYBYTES);
    hash_g(kr, buf, 2 * KYBER_SYMBYTES);

    memcpy(nu, buf, KYBER_SYMBYTES);

    /* coins are in kr+KYBER_SYMBYTES */
    PQCLEAN_KYBER512_CLEAN_indcpa_enc_manip(ct, buf, pk, kr + KYBER_SYMBYTES, r, e1, e2);

    /* overwrite coins in kr with H(c) */
    hash_h(kr + KYBER_SYMBYTES, ct, KYBER_CIPHERTEXTBYTES);
    /* hash concatenation of pre-k and H(c) to k */
    kdf(ss, kr, 2 * KYBER_SYMBYTES);
    return 0;
}

/*************************************************
* Name:        PQCLEAN_KYBER512_CLEAN_crypto_kem_dec
*
* Description: Generates shared secret for given
*              cipher text and private key
*
* Arguments:   - unsigned char *ss: pointer to output shared secret
*                (an already allocated array of KYBER_SSBYTES bytes)
*              - const unsigned char *ct: pointer to input cipher text
*                (an already allocated array of KYBER_CIPHERTEXTBYTES bytes)
*              - const unsigned char *sk: pointer to input private key
*                (an already allocated array of KYBER_SECRETKEYBYTES bytes)
*
* Returns 0.
*
* On failure, ss will contain a pseudo-random value.
**************************************************/
int PQCLEAN_KYBER512_CLEAN_crypto_kem_dec(unsigned char ss[KYBER_SSBYTES],
        const unsigned char ct[KYBER_CIPHERTEXTBYTES],
        const unsigned char sk[KYBER_SECRETKEYBYTES]) {
    size_t i;
    int fail;
    uint8_t buf[2 * KYBER_SYMBYTES];
    /* Will contain key, coins */
    uint8_t kr[2 * KYBER_SYMBYTES];
    uint8_t cmp[KYBER_CIPHERTEXTBYTES];
    const uint8_t *pk = sk + KYBER_INDCPA_SECRETKEYBYTES;

    PQCLEAN_KYBER512_CLEAN_indcpa_dec(buf, ct, sk);

    /* Multitarget countermeasure for coins + contributory KEM */
    for (i = 0; i < KYBER_SYMBYTES; i++) {
        buf[KYBER_SYMBYTES + i] = sk[KYBER_SECRETKEYBYTES - 2 * KYBER_SYMBYTES + i];
    }
    hash_g(kr, buf, 2 * KYBER_SYMBYTES);

    /* coins are in kr+KYBER_SYMBYTES */
    PQCLEAN_KYBER512_CLEAN_indcpa_enc(cmp, buf, pk, kr + KYBER_SYMBYTES);

    fail = PQCLEAN_KYBER512_CLEAN_verify(ct, cmp, KYBER_CIPHERTEXTBYTES);

    /* overwrite coins in kr with H(c) */
    hash_h(kr + KYBER_SYMBYTES, ct, KYBER_CIPHERTEXTBYTES);

    /* Overwrite pre-k with z on re-encryption failure */
    PQCLEAN_KYBER512_CLEAN_cmov(kr, sk + KYBER_SECRETKEYBYTES - KYBER_SYMBYTES, KYBER_SYMBYTES, (uint8_t)fail);

    /* hash concatenation of pre-k and H(c) to k */
    kdf(ss, kr, 2 * KYBER_SYMBYTES);
    return 0;
}

int PQCLEAN_KYBER512_CLEAN_crypto_kem_dec_glitch(unsigned char ss[KYBER_SSBYTES],
        const unsigned char ct[KYBER_CIPHERTEXTBYTES],
        const unsigned char sk[KYBER_SECRETKEYBYTES],
        const unsigned char ct_manip[KYBER_CIPHERTEXTBYTES]
        ) {
    size_t i;
    int fail;
    uint8_t buf[2 * KYBER_SYMBYTES];
    /* Will contain key, coins */
    uint8_t kr[2 * KYBER_SYMBYTES];
    uint8_t cmp[KYBER_CIPHERTEXTBYTES];
    const uint8_t *pk = sk + KYBER_INDCPA_SECRETKEYBYTES;

    PQCLEAN_KYBER512_CLEAN_indcpa_dec(buf, ct, sk);

    /* Multitarget countermeasure for coins + contributory KEM */
    for (i = 0; i < KYBER_SYMBYTES; i++) {
        buf[KYBER_SYMBYTES + i] = sk[KYBER_SECRETKEYBYTES - 2 * KYBER_SYMBYTES + i];
    }
    hash_g(kr, buf, 2 * KYBER_SYMBYTES);

    /* coins are in kr+KYBER_SYMBYTES */
    PQCLEAN_KYBER512_CLEAN_indcpa_enc(cmp, buf, pk, kr + KYBER_SYMBYTES);

    fail = PQCLEAN_KYBER512_CLEAN_verify(ct_manip, cmp, KYBER_CIPHERTEXTBYTES);

    /* overwrite coins in kr with H(c) */
    hash_h(kr + KYBER_SYMBYTES, ct_manip, KYBER_CIPHERTEXTBYTES);

    /* Overwrite pre-k with z on re-encryption failure */
    PQCLEAN_KYBER512_CLEAN_cmov(kr, sk + KYBER_SECRETKEYBYTES - KYBER_SYMBYTES, KYBER_SYMBYTES, (uint8_t)fail);

    /* hash concatenation of pre-k and H(c) to k */
    kdf(ss, kr, 2 * KYBER_SYMBYTES);
    return 0;
}
