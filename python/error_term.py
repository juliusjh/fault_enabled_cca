from python_kyber import Poly, Polyvec
from helpers import transpose
from compress import compress_decompress

def calc_delta_u(sample):
    r = sample.r.ntt()
    u = r.apply_matrix_left_ntt(transpose(sample.pk.a)).intt() + sample.e1
    u = u.reduce()
    u_uncompressed = u
    u = compress_decompress(u)
    deltau = u-u_uncompressed
    assert(sample.ct.b.to_lists() == u.to_lists())
    return deltau

def calc_delta_v(sample):
    #Montg:
    #In: 0 0
    #Basemul: -1 -1 +1 -> -1
    #intt: +1 -> 0
    v = Polyvec.scalar(sample.pk.pk, sample.r.ntt()).intt() + sample.e2
    #Montg:
    #In: 0 0
    #intt: +1 -> +1 0
    #red: -1 -> 0 0
    #scalar_naiv: 0 0
    v2 = Polyvec.scalar(sample.pk.pk, sample.r.ntt()).intt() + sample.e2
    #assert(v2.reduce().to_list() == v.reduce().to_list())
    v = v + Poly.from_msg(sample.nu)
    v = v.reduce()
    v_uncompressed = v
    v = compress_decompress(v)
    deltav = v-v_uncompressed
    #assert(v.to_list() == sample.ct.v.to_list())
    return deltav

def calculate_error_term_from_secret(sample):
    delta_u = calc_delta_u(sample)
    delta_v = calc_delta_v(sample)
    er = Polyvec.scalar_naiv(sample.e, sample.r)
    se1 = Polyvec.scalar_naiv((sample.e1+delta_u), sample.sk.sk.intt().montgomery_reduce())
    res = er - se1 + sample.e2 + delta_v
    res = res.reduce()
    return res 


def calculate_error_term_from_secret_naiv_0(sample):

    sign = lambda i,j: 1 if (i+j) < 256 else -1

    delta_u = calc_delta_u(sample).to_lists()
    delta_v = calc_delta_v(sample).to_list()

    elist = sample.e.to_lists()
    rlist = sample.r.to_lists()
    slist = sample.sk.sk.intt().montgomery_reduce().to_lists()
    e1list = sample.e1.to_lists() 
    e2list = sample.e2.to_list()
    
    er0 = sum([sign(i, (256-i)%256)*elist[j][i]*rlist[j][(256-i)%256] for j in range(len(elist)) for i in range(256)])
    #er = Polyvec.scalar(sample.e.ntt(), sample.r.ntt()).intt()
    se10 = sum([sign(i, (256-i)%256)*(e1list[j][(256-i)%256]+delta_u[j][(256-i)%256])*slist[j][i] for j in range(len(e1list)) for i in range(256)])
    #se1 = Polyvec.scalar((sample.e1+delta_u).ntt(), sample.sk.sk).intt()
    res = er0 - se10 + e2list[0] + delta_v[0]
    #res = er - se1 + sample.e2 + delta_v
    res %= 3329
    #res = res.reduce()
    return res

def calculate_error_term_from_secret_naiv_0_1(sample):
    coeff_index = 0
    delta_u = calc_delta_u(sample)
    delta_v = calc_delta_v(sample)

    s = [si.to_list() for si in sample.sk.sk.intt().montgomery_reduce().to_list()]
    sflat = [sik for si in s for sik in si]
    e = [ei.to_list() for ei in sample.e.to_list()]
    eflat = [eik for ei in e for eik in ei]
    key = eflat + sflat + [1]

    rlist = sample.r.to_lists()
    e1list = sample.e1.to_lists() 
    e2list = sample.e2.to_list()

    sign = lambda i,j: 1 if ((i%256)+(j%256)) < 256 else -1

    e_list = [sign(i, 256-i)*sample.r.to_lists()[j][(256-i)%256] for j in range(len(rlist)) for i in range(256)]
    e1_list = [sign(i, 256-i)*sample.e1.to_lists()[j][(256-i)%256] for j in range(len(rlist)) for i in range(256)]
    du_list = [sign(i, 256-i)*delta_u.to_lists()[j][(256-i)%256] for j in range(len(rlist)) for i in range(256)]
    s_list = [-(duj+e1j) for duj, e1j in zip(e1_list, du_list)]

    row = e_list + s_list + [sample.e2.to_list()[coeff_index] + delta_v.to_list()[coeff_index]]

    comp = sum([(c*k) % 3329 for c, k in zip(row, key)]) % 3329

    return comp

    

def calc_error_term(sample):
    msg = sample.get_msg()
    su = Polyvec.scalar(sample.sk.sk, sample.ct.b.ntt()).intt().reduce()
    r = sample.ct.v - su 
    r = r.reduce()
#    assert(sample.nu == r.to_msg())
    r = (r - msg).reduce()
    return r



