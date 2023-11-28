import multiprocessing
import math
from .helpers import (
    print_v,
    most_likely_list,
)


def get_recovered(results_hashmap, key):
    kv = [(i, results_hashmap[i], k) for i, k in enumerate(key)]
    sorted_kv = list(sorted(kv, key=lambda x: x[1][1]))
    rec = 0
    for _, (r, _), k in sorted_kv:
        d = [(v, r[v]) for v in r.keys()]
        c = max(d, key=lambda x: x[1])[0]
        if c != k:
            break
        rec += 1
    return rec, sorted_kv


def propagate(
    key,
    graph,
    count_steps=1,
    step_size=1,
    thread_count=None,
):
    if thread_count is None:
        thread_count = multiprocessing.cpu_count()
    success = False
    saved_results = []
    step_size *= 2
    results_list = graph.get_results(thread_count)
    print_v(f"Using {thread_count} threads.\n")
    for step in range(0, 2 * count_steps, step_size):
        print_v(f"----Propagation step {step//2}----")
        graph.propagate(step_size, thread_count)
        print_v("Fetching results..")
        result_hashmap = graph.get_results(thread_count)
        results_list = [result_hashmap[i] for i in range(len(key))]
        guessed_key = most_likely_list([r[0] for r in results_list])
        entropies = [r[1] for r in results_list]
        entropy = sum(entropies)/len(entropies)
        max_entropy = max(entropies)
        print_v(f"Average entropy {entropy}; max entropy {max_entropy}")
        if guessed_key == key:
            print_v("Found correct key.")
            success = True
            break
        correct_coeffs = [1 if gki == ki else 0 for gki,
                          ki in zip(guessed_key, key)]
        recovered_coeffs, _ = get_recovered(result_hashmap, key)
        distance = math.sqrt(sum((gki - ki)**2 for gki, ki in zip(guessed_key, key)))
        saved_results.append((results_list, recovered_coeffs, sum(correct_coeffs), distance))
        print(
            f"Correct coefficients: {sum(correct_coeffs)}; recovered: {recovered_coeffs}")
        # TODO: Implement recovered coefficients checking
        if correct_coeffs == len(key):
            print_v("Found all coefficients")
            success = True
            break
        if recovered_coeffs >= len(key)//2:
            print_v("Found more than half of all coefficients")
            success = True
            break
    if success:
        print_v("BP alone: Success!")
    else:
        print_v("BP alone: Failure!")
    return success, saved_results
