import math
from .version import get_imports
from .helpers import (
    print_v,
    IneqType,
)


def create_graph(ineqs, dist, eta=None):
    KYBER_VERSION, check_bp, python_kyber = get_imports()
    if eta is not None:
        if (KYBER_VERSION == "512" and eta != 3) or (KYBER_VERSION != "512" and eta != 2):
            print("WARNING: ETA seems to be incorrect!")
    print_v("Building check graph..")
    g = check_bp.CheckGraph()
    g.add_var_nodes(dist)
    lineno = 0
    total = len(ineqs)
    corrects = 0
    llo_dist_lengths = []
    for ineq in ineqs:
        if ineq.sign == IneqType.LE:
            cmp = check_bp.PyCmpOperator.SmallerEq
        elif ineq.sign == IneqType.GE:
            cmp = check_bp.PyCmpOperator.GreaterEq
        else:
            raise ValueError
        if eta:
            sum_max = 0
            for c in ineq.coefficients:
                sum_max += abs(c)*eta
            assert sum_max >= 0
            length_llo_dist = int(math.pow(2, int(math.log2(sum_max) + 1)))
            if not ineq.length_dist:
                ineq.length_dist = length_llo_dist
            llo_dist_lengths.append(ineq.length_dist)
        g.add_check_node(
            f"Line {lineno}",
            ineq.coefficients,
            ineq.b,
            cmp,
            ineq.p_correct if ineq.p_correct != 1.0 else None,
            ineq.length_dist,
        )
        if ineq.p_correct == 1.0:
            corrects += 1
        lineno += 1
        print_v(f"{lineno}/{total}\t\t", end="\r")
    print_v("                                          ")
    print_v(
        f"Created {total} inequalities, {corrects} are certainly correct, {total-corrects} might be incorrect.")
    print_v(f"LLO lengths: {set(llo_dist_lengths)}")
    print_v("Initializing graph..")
    g.ini()
    return g
