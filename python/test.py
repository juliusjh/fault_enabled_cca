from python_kyber import Poly, Polyvec, KyberSample, Ciphertext, PublicKey, SecretKey, KyberConstants
from manipulate_ct import manipulate_coefficient
from compress import compress_decompress
from error_term import calculate_error_term_from_secret, calc_error_term, calculate_error_term_from_secret_naiv_0, calculate_error_term_from_secret_naiv_0_1
from helpers import transpose

def test():
    sample = KyberSample.generate(True)
    test_consistancy(sample)
    test_manipulation(sample)
    test_intt(sample)
    test_naiv_mult(sample)
    test_error_term(sample)
    test_vec_mult(sample)

def test_naiv_mult(sample):
    x0 = (sample.e2 * sample.e2).reduce()
    x1 = Poly.mul_ntt(sample.e2.ntt(), sample.e2.ntt()).intt().reduce()
    assert(x0.to_list() == x1.to_list())

    r0 = Polyvec.scalar(sample.e.ntt(), sample.e.ntt()).intt().reduce()
    r1 = Polyvec.scalar_naiv(sample.e, sample.e).reduce()
    assert(r0.to_list() == r1.to_list())
    

def test_consistancy(sample):
    v = Polyvec.scalar(sample.pk.pk, sample.r.ntt()).intt() + sample.e2
    v = v + Poly.from_msg(sample.nu)
    v = v.reduce()
    v_uncompressed = Poly.from_bytes_uncompressed(v.to_bytes_uncompressed()).reduce()
    assert(v.to_list() == v_uncompressed.to_list())
    v3 = v
    v = compress_decompress(v)
    #decomp(compress(v)) = v+delta_v
    deltav = v-v3
    assert((v3+deltav).to_list() == v.to_list())
    assert(v.to_list() == sample.ct.v.to_list())
    r = sample.r.ntt()
    u = r.apply_matrix_left_ntt(transpose(sample.pk.a)).intt() + sample.e1
    u = u.reduce()
    u_uncompressed = u
    u = compress_decompress(u)
    deltau = u-u_uncompressed
    assert((deltau+u_uncompressed).to_lists() == u.to_lists())
    assert(sample.ct.b.to_lists() == u.to_lists())

    su = Polyvec.scalar(u.ntt(), sample.sk.sk).intt()
    assert(Poly.from_msg(sample.nu).to_msg() == sample.nu)

    assert(((Polyvec.scalar(sample.pk.pk, sample.r.ntt()).intt() + sample.e2).reduce() + deltav).reduce().to_list() == (v-Poly.from_msg(sample.nu)).reduce().to_list())
    assert(((Polyvec.scalar(sample.pk.pk, sample.r.ntt()).intt() + sample.e2).reduce() + deltav + Poly.from_msg(sample.nu)).reduce().to_list() == v.reduce().to_list())
    
    su_uncompressed = Polyvec.scalar(sample.sk.sk, u_uncompressed.ntt()).intt()
    u_uncompressed_2 = sample.r.ntt().apply_matrix_left_ntt(transpose(sample.pk.a)).intt() + sample.e1
    u_uncompressed_2 = u_uncompressed_2.reduce()
    assert(u_uncompressed.to_lists() == u_uncompressed_2.to_lists())
    assert((u_uncompressed_2+deltau).to_lists() == u.to_lists())
    su_uncompressed_2 = Polyvec.scalar(sample.sk.sk, u.ntt()).intt() - Polyvec.scalar(sample.sk.sk, deltau.ntt()).intt()
    su_uncompressed_2 = su_uncompressed_2.reduce()
    su_uncompressed = su_uncompressed.reduce()
    assert(su_uncompressed_2.to_list() == su_uncompressed.to_list())
     
    er_minus_se1_plus_e2_compressed = Polyvec.scalar(sample.e.ntt(), sample.r.ntt()).intt() - Polyvec.scalar(sample.sk.sk, (sample.e1 + deltau).ntt()).intt() + sample.e2 + deltav
    v_minus_su_compressed = (v - su).reduce()

    assert(er_minus_se1_plus_e2_compressed.reduce().to_list() == (v_minus_su_compressed - Poly.from_msg(sample.nu)).reduce().to_list())
    
    
def test_error_term(sample): 
    err_term_from_secret = calculate_error_term_from_secret(sample).to_list()
    err_term_from_secret_naiv_0 = calculate_error_term_from_secret_naiv_0(sample)
    error_term = calc_error_term(sample).to_list()
    err_term_from_secret_naiv_0_1 = calculate_error_term_from_secret_naiv_0_1(sample)
    assert(err_term_from_secret == error_term)
    assert(err_term_from_secret[0] % 3329 == err_term_from_secret_naiv_0 % 3329)  
    assert(err_term_from_secret_naiv_0_1 == err_term_from_secret_naiv_0 % 3329)

def test_intt(sample):
    assert(sample.e2.ntt().intt().montgomery_reduce().to_list() == sample.e2.to_list())
    assert(sample.e.ntt().intt().montgomery_reduce().to_lists() == sample.e.to_lists())

def test_vec_mult(sample): 
    e2list = sample.e2.to_list()

    sign = lambda i,j: 1 if (i+j) < 256 else -1

    e2_squared_naiv_0 = sum([sign(i,(256-i)%256)*e2list[i]*e2list[(256-i)%256] for i in range(256)]) % 3329

    e2_squared_naiv_0_1 = 0
    for i in range(0, 256):
        e2_squared_naiv_0_1 += sign(i, (256-i)%256)*e2list[i]*e2list[(256-i)%256]
    e2_squared_naiv_0_1 %= 3329

    e2_squared = (sample.e2*sample.e2).reduce().to_list()

    assert(e2_squared[0] % 3329 == e2_squared_naiv_0)

    ##
    r_list = [sign(i%256, k)*sample.r.to_lists()[j][i%256] for j in range(KyberConstants.K()) for i, k in zip(reversed(range(1, 257)), range(256))]
    e_list = [sample.e.to_lists()[j][i] for j in range(KyberConstants.K()) for i in range(0, 256)]
    comp = sum([(ri*ei) % 3329 for ri, ei in zip(r_list, e_list)]) % 3329

    re = (Polyvec.scalar_naiv(sample.r, sample.e)).reduce().to_list()

    re_naiv = sum([sign(i,(256-i)%256)*sample.e.to_lists()[j][i]*sample.r.to_lists()[j][(256-i)%256] for j in range(KyberConstants.K()) for i in range(256)]) % 3329
    assert(comp == re[0]%3329)
 
    

def test_manipulation(sample):
    ctbytes = sample.ct.to_bytes_list()
    ct2 = Ciphertext.from_bytes_list(ctbytes)
    assert(sample.is_valid_ct(ct2))
    ct3 = Ciphertext(sample.ct.b, sample.ct.v)
    assert(sample.is_valid_ct(ct3))
    v_manip = ct3.v.to_list()
    v_manip[0] += 1243
    v_manip[0] %= 3329
    ct4 = Ciphertext(sample.ct.b, Poly.from_list(v_manip))
    assert(not sample.is_valid_ct(ct4))
    ct_manip = manipulate_coefficient(sample.ct, 0, 0)
    assert(sample.is_valid_ct(ct_manip))
    ct_manip = manipulate_coefficient(sample.ct, 0, 1000)
    assert(not sample.is_valid_ct(ct_manip)) 
