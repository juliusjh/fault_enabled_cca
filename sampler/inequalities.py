from .helpers import (
    reduce_sym,
    reduce_sym_list,
    IneqType,
)
from .error_term import calc_delta_u, calc_delta_v, calc_error_term
from .version import get_imports


class Ineq:
    def __init__(self, coefficients, ineq_type, b, is_correct, p_correct, length_dist=None):
        self.coefficients = coefficients
        self.sign = ineq_type
        self.b = b
        self.is_correct = is_correct
        self.p_correct = p_correct
        self.length_dist = length_dist


def sample_from_key_bytes(sk_bytes, pk_bytes, e_lists):
    _, _, python_kyber = get_imports()
    sk = python_kyber.SecretKey.from_bytes(bytes(sk_bytes))
    pk = python_kyber.PublicKey.from_bytes(bytes(pk_bytes))
    e = python_kyber.Polyvec.from_lists(e_lists)
    sample = python_kyber.KyberSample.generate_with_key(True, pk, sk, e)
    return sample


def get_ineqsign(sample, coeff_index=0):
    r, _ = calc_error_term(sample)
    r = reduce_sym_list(r.to_list())
    if r[coeff_index] <= 0:
        return IneqType.LE
    else:
        return IneqType.GE


def extract_inequality_coefficients(sample, coeff_index=0, max_delta_v=None):
    _, _, python_kyber = get_imports()
    delta_u = calc_delta_u(sample)
    delta_v = calc_delta_v(sample)

    if max_delta_v is not None and abs(delta_v.to_list()[coeff_index]) >= max_delta_v:
        return None, None

    sign = lambda i, j: 1 if ((i % 256) + (j % 256)) < 256 else -1
    e_list = [
        sign(i, 256 - i + coeff_index)
        * sample.r.to_lists()[j][(256 - i + coeff_index) % 256]
        for j in range(python_kyber.KyberConstants.K())
        for i in range(256)
    ]
    e1_list = [
        sign(i, 256 - i + coeff_index)
        * sample.e1.to_lists()[j][(256 - i + coeff_index) % 256]
        for j in range(python_kyber.KyberConstants.K())
        for i in range(256)
    ]
    du_list = [
        sign(i, 256 - i + coeff_index)
        * delta_u.to_lists()[j][(256 - i + coeff_index) % 256]
        for j in range(python_kyber.KyberConstants.K())
        for i in range(256)
    ]
    s_list = [-(duj + e1j) for duj, e1j in zip(e1_list, du_list)]

    row = e_list + s_list
    b = -reduce_sym(sample.e2.to_list()[coeff_index] + delta_v.to_list()[coeff_index])
    row = reduce_sym_list(row)
    return row, b
