from __future__ import division
import scipy.linalg
import scipy.special
import random
import numpy as np
import enum

from functools import lru_cache
from multiprocessing import Pool


from manipulate_ct import manipulate_coefficient
from error_term import calc_error_term, calculate_error_term_from_secret, calc_delta_u, calc_delta_v, calculate_error_term_from_secret_naiv_0
from flippedbits import count_flipped_bits
from python_kyber import Poly, Polyvec, KyberSample, Ciphertext, PublicKey, SecretKey, KyberConstants


q = 3329
#dv = KyberConstants.DV()

def create_matrix_threaded(thread_count, number=20000, tries=10, add_in_vec=True):
    sample = KyberSample.generate(verify_decaps=True)
    pool = Pool(thread_count)
    chunck_size = 20
    arg_count = number//chunck_size
    key_tuple = (sample.pk.to_bytes(), sample.sk.to_bytes(), sample.e.to_lists())
    args = [(key_tuple, th_no, chunck_size, tries, add_in_vec) for th_no in range(arg_count)]
    left_overs = number - chunck_size * arg_count
    if left_overs > 0:
        args += [(key_tuple, arg_count+1, left_overs, tries, add_in_vec)]

    #results = pool.map(create_matrix_tup, args)
    results = []
    total = 0
    for _, res in enumerate(pool.imap_unordered(create_matrix_tup, args)):
        results.append(res)
        total += len(res[0]) + len(res[1])
        print("{}/{}".format(total, number), end='\r')
    print(" "*30, end='\r')
    pool.close()
    pool.join()

    mat_le = [] 
    mat_ge = []
    vec_le = []
    vec_ge = []
    eq_le = []
    eq_ge = []
    for mat_ge_r, mat_le_r, vec_ge_r, vec_le_r, _, eq_ge_r, eq_le_r in results:
        mat_le += mat_le_r
        mat_ge += mat_ge_r
        vec_le += vec_le_r
        vec_ge += vec_ge_r
        eq_le += eq_le_r
        eq_ge += eq_ge_r
    print("Retrieved {} inequalities.".format(len(mat_le)+ len(mat_ge)))
    print("Checking inequalities..")
    key = check_inequalities(sample, mat_ge, mat_le, vec_ge, vec_le, None, add_in_vec, eq_ge, eq_le)
    assert(len(set(tuple(map(lambda l: tuple(l), mat_le)))) == len(mat_le))
    assert(len(set(tuple(map(lambda l: tuple(l), mat_ge)))) == len(mat_ge))
    assert(len(mat_le) + len(mat_ge) == number)
    print("Passed checks.")
    
    return mat_ge, mat_le, vec_ge, vec_le, key, eq_ge, eq_le
    

def create_matrix_tup(tup):
    return create_matrix(*tup)


def create_matrix(key, th_no, number=20000, tries=10, add_in_vec=True):
    mat_le = []
    mat_ge = []
    vec_le = []
    vec_ge = []
    eq_le = []
    eq_ge = []
    #######
    pk = PublicKey.from_bytes(bytes(key[0]))
    sk = SecretKey.from_bytes(bytes(key[1]))
    e = Polyvec.new_from_list([Poly.from_list(p) for p in key[2]])
    sample = KyberSample.generate_with_key(False, pk, sk, e)
    #######
    found = 0
    #print(f"Chunck {th_no} started..")
    while found < number:
        found_one = False
        ieqtype, row, b, eq = create_inequalities_from_sample(sample, add_in_vec=add_in_vec) 
        if ieqtype == IneqType.LE:
            mat_le.append(row)
            vec_le.append(b)
            eq_le.append(eq)
            found += 1
        elif ieqtype == IneqType.RE:
            mat_ge.append(row)
            vec_ge.append(b)
            eq_ge.append(eq)
            found += 1
        sample = KyberSample.generate_with_key(False, sample.pk, sample.sk, sample.e) 
    #print("Chunck {}: Found {} <= inequalities and {} >= inequalities".format(th_no, len(vec_le), len(vec_ge)))
    return mat_ge, mat_le, vec_ge, vec_le, key, eq_ge, eq_le


def calc_row(sample, coeff_index, delta_u, delta_v, add_in_vec):
    sign = lambda i,j: 1 if ((i%256)+(j%256)) < 256 else -1
    assert(coeff_index == 0)
    e_list = [sign(i, 256-i)*sample.r.to_lists()[j][(256-i)%256] for j in range(KyberConstants.K()) for i in range(256)]
    e1_list = [sign(i, 256-i)*sample.e1.to_lists()[j][(256-i)%256] for j in range(KyberConstants.K()) for i in range(256)]
    du_list = [sign(i, 256-i)*delta_u.to_lists()[j][(256-i)%256] for j in range(KyberConstants.K()) for i in range(256)]
    s_list = [-(duj+e1j) for duj, e1j in zip(e1_list, du_list)]

    add = 0
    if add_in_vec:
        row = e_list + s_list 
        add = -reduce_sym(sample.e2.to_list()[coeff_index] + delta_v.to_list()[coeff_index])
    else:
        row = e_list + s_list + [sample.e2.to_list()[coeff_index] + delta_v.to_list()[coeff_index]]
    #map(lambda x: x % q, row)
    row = reduce_sym_list(row)
    return row, add

class IneqType(enum.Enum):
    LE = 0,
    RE = 1,
    NOT_FOUND = 2

def create_inequalities_from_sample(sample, add_in_vec=True, max_v=10): 

    msg = sample.get_msg()

    err = q//4

    delta_u = calc_delta_u(sample)
    delta_v = calc_delta_v(sample)

    #for i in coefficients:
    i = 0
    if abs(delta_v.to_list()[i]) >= max_v:
        return IneqType.NOT_FOUND, None, None, None
    ct_manip = manipulate_coefficient(sample.ct, i, err)
    flipped = count_flipped_bits(sample.ct, ct_manip)
    if flipped != 1:
        return IneqType.NOT_FOUND, None, None, None
    is_valid = sample.is_valid_ct(ct_manip)
    row, b = calc_row(sample, i, delta_u, delta_v, add_in_vec) 
    bit_is_0 = sample.nu[0] & 1 == 0
    if is_valid: #< or <=
        return IneqType.LE, row, b, bit_is_0
    else:
        return IneqType.RE, row, b, not bit_is_0

def reduce_sym(a):
    a %= q
    if a > q//2:
        a -= q
    if a <= -q//2:
        a += q
    return a

def reduce_sym_list(a):
    return list(map(lambda x: reduce_sym(x), a))

def check_inequalities(sample, inequalities_ge, inequalities_le, values_ge, values_le, error_term, add_in_vec, eq_ge, eq_le):
    s = [si.to_list() for si in sample.sk.sk.intt().montgomery_reduce().to_list()]
    sflat = [sik for si in s for sik in si]
    e = [ei.to_list() for ei in sample.e.to_list()]
    eflat = [eik for ei in e for eik in ei]
    key = eflat + sflat
    if not add_in_vec:
        key += [1]
    key = reduce_sym_list(key)
    comps = []
    for i, row in enumerate(inequalities_ge):
        assert(len(key) == len(row))
        comp = sum([(c*k) for c, k in zip(row, key)])
        comps.append(comp)
        #print("Error term comp: ", comp)
        #print("Error term: ", error_term)
        #print("Should be ge: ", values_ge[i])
        if error_term != None:
            assert(error_term+values_ge[i] == comp)
        assert(comp >= values_ge[i])
        if not eq_ge[i]:
            assert(comp > values_ge[i])
    #print("Comps ge: ", comps)
    #print("Values ge: ", values_ge)
    #print("\n")
    comps = []
    for i, row in enumerate(inequalities_le):
        assert(len(key) == len(row))
        comp = sum([(c*k) for c, k in zip(row, key)])
        comps.append(comp)
        #print("Error term comp: ", comp)
        #print("Error term: ", error_term)
        #print("Should be le: ", values_le[i])
        if error_term != None:
            assert(error_term + values_le[i] == comp)
        assert(comp <= values_le[i])
        if not eq_le[i]:
            assert(comp < values_le[i])
    #print("Comps le: ", comps)
    #print("Values le: ", values_le)
    #print("")
    return key

def check_inequalities_no_sample(key, inequalities_ge, inequalities_le, values_ge, values_le):
    if len(inequalities_ge) > 0:
        left = np.matmul(inequalities_ge, key)
        left_eq = [l >= vl for l, vl in zip(left, values_ge)]
        if not all(left_eq):
            return False
    if len(inequalities_le) > 0:
        right = np.matmul(inequalities_le, key)
        right_eq = [r <= vr for r, vr in zip(right, values_le)]
        if not all(right_eq):
            return False
    #print("Comps left: ", left)
    #print("Values left: ", values_le)
    #print("Comps right: ", right)
    #print("Values right: ", values_ge)
    #print(left_eq)
    #print(right_eq)
    return True


def key_from_file(filename="ineqs_es.txt"):
    f = open(filename, 'r')
    line = filter(lambda x: x, f.readline().split(' '))
    key = list(map(lambda c: int(c), line))
    f.close()
    return key

@lru_cache(maxsize=1)
def mat_from_file(filename="ineqs.txt"):
    f = open(filename, 'r')
    lines = f.readlines()
    f.close()
    mat_ge = []
    mat_le = []
    vec_le = []
    vec_ge = []
    for l in lines:
        if '<' in l:
            parts = l.split('<')
            le = True
        else:
            parts = l.split('>')
            le = False
        row = list(map(lambda c: int(c), filter(lambda x: x, parts[0].split(' '))))
        val = int(parts[1].strip())
        if le:
            mat_le.append(row)
            vec_le.append(val)
        else:
            mat_ge.append(row)
            vec_ge.append(val)
    return mat_ge, mat_le, vec_ge, vec_le
     

def key_to_file(key, filename="ineqs_es.txt"):
    f = open(filename, "w")
    for v in key:
        f.write(str(v))
        f.write(" ")
    f.close()


def mat_to_file(mat, sign, b, filename="ineqs.txt", mode='w'):
    f = open(filename, mode)
    for row, bi in zip(mat, b):
        f.write(str(row).replace(',', ' ').replace("[", '').replace(']', ' ')) 
        f.write(sign)
        f.write(" " + str(bi))
        f.write("\n")
    f.close()
