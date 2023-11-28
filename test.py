import argparse
from sampler import sample_cca_inequalities, propagate, create_graph, bino, set_security_level
from sampler import get_security_level

parser = argparse.ArgumentParser()
parser.add_argument("-n", help="Number of inequalities", type=int, default=1000)
parser.add_argument("-i", help="BP iterations", type=int, default=100)
parser.add_argument("-t", help="Threats", type=int, default=None)
parser.add_argument("-s", help="Kyber security level", type=int, default=768)
parser.add_argument("-p", help="Correctness probability", type=float, default=None)
parser.add_argument("-l", help="LLO FFT length", type=int, default=None)
parser.add_argument("--certainly-correct", help="Number of probabilities with probability 1.0", type=float, default=None)
parser.add_argument("--ct-filter", help="Ciphertext filtering (delta v)", type=int, default=None)

args = parser.parse_args()
assert args.n >= 2
assert args.s in [512, 768, 1024]

set_security_level(args.s)
version = get_security_level()
assert version in ["512", "768", "1024"] and version == str(args.s)

ETA = 3 if version == "512" else 2
print(f"Kyber security level: Kyber{version}, {ETA=}")


key, ineqs, sample = sample_cca_inequalities(args.n, args.p, args.ct_filter, args.certainly_correct)
if args.l is not None:
    for ineq in ineqs:
        ineq.length_dist = args.l
g = create_graph(ineqs, bino(ETA), ETA)
success, saved_results = propagate(key, g, args.i, thread_count=args.t)
print(f"{success=}")
