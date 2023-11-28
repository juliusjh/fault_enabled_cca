import enum
import math
import numpy as np
import scipy.special
from .version import get_imports

verbose = True


def set_verbose(v):
    global verbose
    verbose = v


def print_v(*args, end=None):
    global verbose
    if verbose:
        print(*args, end=end)


def most_likely_coeff(dist):
    return max(dist, key=dist.get)


def most_likely_list(probs_list):
    return [most_likely_coeff(d) for d in probs_list]


def most_likely(props_dicts):
    res = [0] * len(props_dicts)
    for d in props_dicts:
        res[d] = max(props_dicts[d][0], key=lambda v: props_dicts[d][0][v])
    return res


def flatten(ls):
    return (li for l in ls for li in l)


def dot(a, b, q=None):
    if q:
        return sum((ai * bi % q for ai, bi in zip(a, b)))
    else:
        return sum((ai * bi for ai, bi in zip(a, b)))


def norm(a):
    return math.sqrt(dot(a, a))


class IneqType(enum.Enum):
    LE = 0
    GE = 1


def rlwe_to_lwe(poly, q=3329):
    n = len(poly)
    mat = [[0 for _ in range(n)] for _ in range(n)]
    for i in range(n):
        mat[i] = poly.copy()
        new_poly = [None for _ in range(n)]
        poly_last = poly[n - 1]
        for i, c in enumerate(poly[: n - 1]):
            new_poly[i + 1] = c
        new_poly[0] = (-poly_last) % q
        poly = new_poly
    return np.transpose(np.array(mat)).tolist()


def mlwe_to_lwe(A, b):
    assert len(A[0][0]) == len(b[0])
    poly_len = len(b[0])
    k = len(b)
    n = k * poly_len
    m = len(A) * poly_len
    mat = np.zeros(shape=(m, n), dtype=np.int64)
    for i, row_a in enumerate(A):
        for j, a in enumerate(row_a):
            mat[
                i * poly_len : i * poly_len + poly_len,
                j * poly_len : j * poly_len + poly_len,
            ] = rlwe_to_lwe(a)
    b = list(flatten(b))
    return mat.tolist(), b


def poly_to_matrix(poly):
    k = len(poly)
    sign = lambda i, j: 1 if ((i % k) + (j % k)) < k else -1
    res = [[None for _ in range(k)] for _ in range(k)]
    for i in range(len(poly)):
        for j in range(len(poly)):
            idx = (k - i + j) % k
            res[i][j] = sign(i, idx) * poly[idx]
    return res


def transpose(A):
    KYBER_VERSION, _, python_kyber = get_imports()
    Alist = [A[i].to_list() for i in range(len(A))]
    return [
        python_kyber.Polyvec.new_from_list([Alist[i][j] for i in range(len(A))])
        for j in range(len(A))
    ]


def transpose_mat(a):
    res = [[None for _ in range(len(a))] for _ in range(len(a[0]))]
    for i, ai in enumerate(a):
        for j, aij in enumerate(ai):
            res[j][i] = aij
    return res


def mat_mat_mul(mat0, mat1, q=None):
    assert all((len(mi) == len(mat1) for mi in mat0))
    res = [[0 for _ in range(len(mat1))] for _ in range(len(mat0))]
    for i in range(len(mat0)):
        for j in range(len(mat1)):
            for k in range(len(mat1)):
                if q:
                    res[i][j] += (mat0[i][k] * mat1[k][j]) % q
                else:
                    res[i][j] += mat0[i][k] * mat1[k][j]
            if q:
                res[i][j] %= q
    return res


def mat_mul(mat, vec, q=None):
    if q is None:
        return (sum((m_ij * v_j for m_ij, v_j in zip(m_i, vec))) for m_i in mat)
    else:
        return (
            sum(((m_ij * v_j) % q for m_ij, v_j in zip(m_i, vec))) % q for m_i in mat
        )


def add_vec(v0, v1, q=None):
    if q is None:
        return map(lambda x: x[0] + x[1], zip(v0, v1))
    else:
        return map(lambda x: (x[0] + x[1]) % q, zip(v0, v1))


def sub_vec(v0, v1, q=None):
    if q is None:
        return map(lambda x: x[0] - x[1], zip(v0, v1))
    else:
        return map(lambda x: (x[0] - x[1]) % q, zip(v0, v1))


def euclidean_dist(b0, b1):
    assert len(b0) == len(b1)
    return math.sqrt(sum(((b0i - b1i) ** 2) for b0i, b1i in zip(b0, b1)))


def bino(eta):
    binp = lambda x: scipy.special.binom(eta, x) / 2**eta
    return {
        i: sum([binp(x) * binp(x + i) for x in range(-eta, eta + 1)])
        for i in range(-eta, eta + 1)
    }


def expected(dist):
    return sum((i * p for i, p in dist.items()))


def var(dist):
    exp = expected(dist)
    return sum((p * (exp - i) ** 2 for i, p in dist.items()))


def reduce_sym(a, q=3329):
    a %= q
    if a > q // 2:
        a -= q
    if a <= -q // 2:
        a += q
    return a


def reduce_sym_list(a, q=3329):
    return list(map(lambda x: reduce_sym(x, q), a))


def flatten_key(sample):
    s = [si.to_list() for si in sample.sk.sk.intt().montgomery_reduce().to_list()]
    sflat = [sik for si in s for sik in si]
    e = [ei.to_list() for ei in sample.e.to_list()]
    eflat = [eik for ei in e for eik in ei]
    key = eflat + sflat
    key = reduce_sym_list(key)
    return key


def check_inequalities(key, ineqs):
    for ineq in ineqs:
        # for ineq, veci, sign, _, is_correct in ineqs:
        if not ineq.is_correct:
            continue
        dot = sum([a * b for a, b in zip(ineq.coefficients, key)])
        if ineq.sign == IneqType.LE:
            if dot > ineq.b:
                return False
        else:
            if dot < ineq.b:
                return False
    return True
