import sys
import numpy as np

from .inequalities import (
    extract_inequality_coefficients,
    get_ineqsign,
    Ineq,
)
from .helpers import (
    flatten_key,
    check_inequalities,
    IneqType,
    print_v,
)
from .version import get_imports


def mult_dist_by_fac(fac, dist):
    new = {}
    for i in dist:
        v = fac*i
        if v not in new:
            new[v] = 0
        new[v] += dist[i]
    return new


def mult_dists_pointwise(dist0, dist1):
    retv = [a*b for a, b in zip(dist0, dist1)]
    return retv


def dist_to_array(dist, ll):
    retv = [0 for _ in range(2*ll+1)]
    for v in dist:
        p = dist[v]
        if v < 0:
            v += len(retv)
        retv[v] = p
    return retv


def naive_mult(dist0, dist1):
    retv = {}
    for v0 in dist0:
        for v1 in dist1:
            v = v0 + v1
            if v not in retv:
                retv[v] = 0
            retv[v] += dist0[v0] * dist1[v1]
    return retv


def ifft(dist):
    dist = np.fft.ifft(np.array(dist, dtype=np.complex256))
    dist = [cn.real for cn in dist]
    return dist


def clean(d):
    return list(map(lambda x: 0 if x < 0.0001 else x, d))


def sample_cca_inequalities_ge(*args, **kwargs):
    key, ineqs, sample = sample_cca_inequalities(*args, **kwargs)
    c = []
    v = []
    for ineq in ineqs:
        coeffs = None
        b = None
        if ineq.sign == IneqType.LE:
            coeffs = [-x for x in ineq.coefficients]
            b = -ineq.b
        else:
            coeffs = ineq.coefficients.copy()
            b = ineq.b
        c.append(coeffs)
        v.append(b)
    return (c, v), key, sample


def sample_cca_inequalities(
    number_faults,
    p_correct=None,
    max_delta_v=None,
    num_certain_correct=None,
    sample=None,
):
    KYBER_VERSION, _, python_kyber = get_imports()
    if p_correct is None:
        p_correct = 1.0
    if num_certain_correct is None:
        if p_correct < 1.0:
            num_certain_correct = 0
        else:
            num_certain_correct = number_faults
    if sample is None:
        sample = python_kyber.KyberSample.generate(True)
    print_v("Sampling inequalities..")
    # sample = sample_from_key_bytes(sk_bytes, pk_bytes, e_lists)
    print_v(
        f"kyber_version={KYBER_VERSION}, number_faults={number_faults}, p={p_correct}, certainly_correct={num_certain_correct}, max_delta_v={max_delta_v}"
    )
    key = flatten_key(sample)
    inequalities = []
    no_ineqs = 0
    errors = 0
    filtered_cts = 0
    for i in range(number_faults):
        is_correct = True
        coeffs = None
        b = None
        first = True
        while not coeffs:
            sample = python_kyber.KyberSample.generate_with_key(
                False, sample.pk, sample.sk, sample.e
            )
            coeffs, b = extract_inequality_coefficients(
                sample, max_delta_v=max_delta_v)
            if not first:
                filtered_cts += 1
            first = False
        sign = get_ineqsign(sample)
        if i >= ((number_faults - num_certain_correct) * p_correct) + num_certain_correct:
            is_correct = False
            p_correct_ineq = p_correct
            errors += 1
            if sign == IneqType.LE:
                sign = IneqType.GE
            else:
                sign = IneqType.LE
        else:
            p_correct_ineq = 1 if i < num_certain_correct else p_correct
        inequalities.append(Ineq(
            coeffs, sign, b, is_correct, p_correct_ineq))
        no_ineqs += 1
        print_v(f"{i}/{number_faults}\t\t", end="\r")
    assert check_inequalities(key, inequalities)
    print_v("                                          ")
    print_v("Number of inequalities: ", no_ineqs)
    print_v(f"Wrong inequalities: {errors}")
    print_v(f"Filtered cts: {filtered_cts}")
    print_v("Finished sampling inequalities.")
    sys.stdout.flush()
    return key, inequalities, sample
