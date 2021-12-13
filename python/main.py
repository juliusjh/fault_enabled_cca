#!/usr/bin/env python3

import numpy as np
import scipy.linalg
import scipy.special
import scipy.stats
import random
import argparse
import multiprocessing
import json
import os

from zipfile import ZipFile

from datetime import datetime

from solver import create_graph

from test import test
from test_check_bp import test_check_bp
from test_bin_tree import test_bin_tree

from python_kyber import KyberConstants

from inequalities import create_matrix_threaded, key_from_file, mat_from_file, key_to_file, mat_to_file, check_inequalities_no_sample


def bino(eta):
    binp = lambda x: scipy.special.binom(eta, x)/2**eta
    return {i: sum([binp(x)*binp(x+i) for x in range(-eta, eta+1)])  for i in range(-eta, eta+1)}
 

def norm_res(res):
    avg = sum(res.values())
    res = {v: p/avg for v, p in res.items()}
    return res
    
def rank(res, val):
    rk = 0
    pval = res[val]
    for v, p in res.items():
        if v == val:
            continue
        if p > pval:
            rk += 1
    return rk

def get_args():
    parser = argparse.ArgumentParser(description='CCA attack')
    parser.add_argument('--generate', action='store_true')
    parser.add_argument('--save', action='store_true') 
    parser.add_argument('--save-keys', action='store_true') 
    parser.add_argument('--numbers', nargs='+', type=int) 
    parser.add_argument('--file', default='ineqs', type=str)
    parser.add_argument('--iterations', type=int, default=10)
    parser.add_argument('--threads', type=int, default=0)
    parser.add_argument('--seed', type=int, default=0)
    parser.add_argument('--runs', type=int, default=1)
    parser.add_argument('--no-improve-abort', type=int, default=5)
    parser.add_argument('--results-file', type=str, default="results/results")
    parser.add_argument('--zip-file', type=str, default=None)
    args = parser.parse_args()
    if args.threads <= 0:
        args.threads = multiprocessing.cpu_count()
    if args.save and not args.generate:
        print("Incompatible arguments. Saving inequalities is only possible when generating.")
        exit(-1)
    if args.runs <= 0:
        print("Cannot run <= 0 runs.")
        exit(-1)
    if args.generate and not args.save:
        print("\n\nWARNING: Equations are not being saved.")
        if args.save_keys:
            print("Keys are being saved.")
    if args.save and args.file == "ineqs":
        c = input("WARNING: Overwritting default file ineqs. Continue [y/N]? ")
        if c != 'y':
            exit(0)
    return args

def add_to_zip(filename, zip_filename, delete=False):
    if zip_filename is None:
        return
    print(f"Adding {filename} to {zip_filename}..")
    with ZipFile(zip_filename, 'a') as myzip:
        myzip.write(filename)
    if delete:
        print(f"Deleting {filename}..")
        os.remove(filename)

def get_equations(number, filename, generate, save, th_no, zip_file, save_keys):
    if generate:
        print("Simulating faults on device to create inequalities..")
        mat_ge, mat_le, vec_ge, vec_le, key, eq_ge, eq_le = create_matrix_threaded(th_no, number)
        print("Found {} inequalities.".format(len(mat_ge) + len(mat_le)))
        if save or save_keys:
            print("\nSaving inequalities for {}.".format(filename))
            matfile = filename + ".txt"
            keyfile = filename + "_es.txt"
            if save:
                mat_to_file(mat_ge, ">", vec_ge, filename=matfile)
                mat_to_file(mat_le, "<", vec_le, filename=matfile, mode='a')
                add_to_zip(matfile, zip_file, delete=True)
            else:
                print("WARNING: Inequalities are not saved.")
            key_to_file(key, filename=keyfile)
            add_to_zip(keyfile, zip_file, delete=True)
            print("")
    else:
        print("Loading inequalities from file/cache..")
        mat_ge, mat_le, vec_ge, vec_le = mat_from_file(filename + ".txt")
        key = key_from_file(filename + "_es.txt")
        print("Loaded {} inequalities.".format(len(mat_ge) + len(mat_le)))
    noeqs = len(mat_ge) + len(mat_le)
    if noeqs  < number:
        print("Warning: Not enough inequalities in file (specified {} found {}).".format(number, noeqs))
    elif noeqs > number:
        print("Removing some inequalities..")
        to_rem = noeqs - number
        le_over = len(mat_le) - number/2
        ge_over = len(mat_ge) - number/2
        if le_over >= 0 and ge_over >= 0:
            indices_to_remove_ge = random.sample(range(0, len(mat_ge)), to_rem//2)
            indices_to_remove_le = random.sample(range(0, len(mat_le)), to_rem//2 + to_rem % 2)
            mat_ge = np.delete(mat_ge, indices_to_remove_ge, axis=0)
            mat_le = np.delete(mat_le, indices_to_remove_le, axis=0)
            vec_ge = np.delete(vec_ge, indices_to_remove_ge) 
            vec_le = np.delete(vec_le, indices_to_remove_le)
        elif le_over < 0:
            indices_to_remove_ge = random.sample(range(0, len(mat_ge)), to_rem)
            np.delete(mat_ge, to_rem)
            np.delete(vec_ge, to_rem)
            
        elif ge_over < 0:
            indices_to_remove_ge = random.sample(range(0, len(mat_le)), to_rem)
            np.delete(mat_le, to_rem)
            np.delete(vec_le, to_rem)
        else:
            raise ValueError("Weird input.")
    noeqs = len(mat_ge) + len(mat_le);
    print("Checking inequalities..")
    assert(check_inequalities_no_sample(key, mat_ge, mat_le, vec_ge, vec_le))
    print("Continuing with {} inequalities.\n".format(noeqs))
    return mat_ge, mat_le, vec_ge, vec_le, key, eq_ge, eq_le

def count_correct_list(cr):
    for i, val in enumerate(cr):
        if not val:
            return i
    return len(cr)

def check_abort_prob_ent(key, result_list):
    max_ent = max(map(lambda r: r[2], result_list)) 
    result_list = list(sorted(result_list, key=lambda r: r[1][1] - 2*r[2]/max_ent, reverse=True))
    correct_list = [rk[0] == 0 for i, rk in enumerate(result_list) if i < len(key)/2] 
    return all(correct_list), count_correct_list(correct_list), result_list

def check_abort_prob(key, result_list):
    result_list = list(sorted(result_list, key=lambda r: r[1][1], reverse=True))
    correct_list = [rk[0] == 0 for i, rk in enumerate(result_list) if i < len(key)/2] 
    return all(correct_list), count_correct_list(correct_list), result_list

def check_abort_ent(key, result_list):
    result_list = list(sorted(result_list, key=lambda r: r[2]))
    correct_list = [rk[0] == 0 for i, rk in enumerate(result_list) if i < len(key)/2] 
    return all(correct_list), count_correct_list(correct_list), result_list

def check_abort_ediff(key, result_list):
    if result_list[0][3] == None:
        return False, 0, []
    result_list = list(sorted(result_list, key=lambda r: r[3]))
    correct_list = [rk[0] == 0 for i, rk in enumerate(result_list) if i < len(key)/2] 
    return all(correct_list), count_correct_list(correct_list), result_list

def kyber_version():
    return KyberConstants.K() *256

def main():
    args = get_args()
    seed = args.seed
    starttime = datetime.now()
    datestr = starttime.strftime("%m%d%Y%H%M")
    ver = str(kyber_version())
    print("")
    print("Parameters: {}".format(str(args).split('(')[1].split(')')[0]))
    print("")
    for k, number in enumerate(args.numbers):
        results = []
        for i in range(args.runs):
            print("Experiment {} with seed {} and {} inequalities.\n".format(i, seed, number))
            res = run(args, seed, number, i, args.runs, k, len(args.numbers), starttime, datestr)
            results.append(res)
            seed = random.randint(0, 2**20)
            print("")
            print("-"*40)
            print("")
        print("Reseting seed..")
        seed = args.seed
        filen = args.results_file + '_' + ver + '_' + str(number) + '_' + datestr +'.json'
        print("Writing results to {}..".format(filen))
        with open(filen, 'a') as outfile:
            json.dump(results, outfile, indent=4)
        add_to_zip(filen, args.zip_file, delete=True)
        suc_rate = sum([1 for r in results if r['success']])/len(results)
        print("")
        print("Success rate with {} equations: {}".format(number, suc_rate))
        print("="*80)
        print("")
    runtime = datetime.now() - starttime 
    print("Testing {} counts of inequalities with {} runs and a maximum of {} iterations took {} minutes."
        .format(len(args.numbers), args.runs, args.iterations, runtime.total_seconds()//60))

def get_entropy(res):
    return scipy.stats.entropy(list(res.values()), base=2)

def print_progress(len_key, len_max, current_iteration, iterations, current_run, runs, current_no, nonumbers, lendis=70):
    prog_rec = len_max/(len_key//2)
    prog_it = current_iteration/iterations
    prog_exp = (current_run*iterations+current_iteration)/(runs*iterations)
    prog_tot = (current_no*runs*iterations+current_run*iterations+current_iteration)/(nonumbers*runs*iterations)
    done_disp = lambda x: int(lendis*x)
    def form(x):
        d = done_disp(x)    
        return ">"*d, " "*(lendis-d), int(x*100)
    print("Recovering: [{}{}] ({}%)".format(*form(prog_rec)))
    print("Iterations: [{}{}] ({}%)".format(*form(prog_it)))
    print("Experiment: [{}{}] ({}%)".format(*form(prog_exp)))
    print("Total     : [{}{}] ({}%)".format(*form(prog_tot)))
    print("")

def fix_coeffs(g, result_list):
    i = 0
    for r in result_list:
        if r[1][1] < 0.95:
            break
        if r[1][0] in [-3, -2, 2, 3] and r[3] < 0.1:
            if r[0] != 0:
                print("WARNING: Fixing incorrect value")
            g.set_fixed(r[4], r[1][0])
            i += 1
    print(f"Fixed {i} values.")


def check_abort(key, result_list):
    abortp, len_cor_p, _ = check_abort_prob(key, result_list)
    aborte, len_cor_e, res_ent = check_abort_ent(key, result_list)
    aborted, len_cor_ed, _ = check_abort_ediff(key, result_list)
    abortpe, len_cor_pe, _ = check_abort_prob_ent(key, result_list)
    print("Sorted ranks (Probability, Entropy, Entropy Delta, Entropy/Probability): {}, {}, {}, {}".format(len_cor_p, len_cor_e, len_cor_ed, len_cor_pe))
    return (abortp or aborte or aborted or abortpe), len_cor_e, len_cor_ed, len_cor_p, len_cor_pe, res_ent

def run(args, seed, number, current_run, runs, current_no, nonumbers, starttime, datestr):
    random.seed(seed)
    ver = str(kyber_version())
    run_file = args.file + '_' + ver + '_' + str(number) + '_' + str(current_run) + '_' + datestr
    mat_ge, mat_le, vec_ge, vec_le, key, eq_ge, eq_le = get_equations(number, run_file, args.generate, args.save, args.threads, args.zip_file, args.save_keys)
    g = create_graph(mat_ge, mat_le, vec_ge, vec_le, bino(eta=KyberConstants.ETA()), eq_ge, eq_le)
    print("\nInitializing graph..")
    g.ini()
    print("Beginning propagation with {} threads..\n".format(args.threads))
    success = False
    result_list = []
    starttimestr = starttime.strftime("%m.%d.%Y %H:%M:%S")
    fixed = []
    fixed_incorrect = 0
    last_improved = {'last_changed': 0, 'max_coeffs_correct': 0}
    best_coeff_correct = 0
    for i in range(args.iterations):
        print("Propagating {}-th step..".format(i))
        g.propagate(2, args.threads)
        print("Fetching results..")
        current_results_dict = g.get_results(args.threads)
        print("Done fetching results.")
        avgp = 0
        avgrk = 0
        results = []
        results_last = result_list
        result_list = []
        correct = 0
        for j, k in enumerate(key):
            res, ent = current_results_dict[j]
            prob = res[k]
            avgp += prob
            rk = rank(res, k)
            avgrk += rk
            results.append(res)
            ent_diff = None
            if len(results_last) > 0:
                ent_diff = abs(ent-results[j][2]) 

            result_list.append((rk,
                                max(res.items(), key=lambda r: r[1]),
                                ent,
                                ent_diff,
                                j))
            if rk == 0:
                correct += 1
            #print("{}: {}, {}".format(k, prob, rk)) 
        avgp /= KyberConstants.K()*512
        avgrk /= KyberConstants.K()*512
        print("Done propagating.")
        print("Averages (Probability, Rank, Count Rank 0): {} {} {}".format(avgp, avgrk, correct))
        abort_success, len_cor_p, len_cor_e, len_cor_ed, len_cor_pe, res_ent = check_abort(key, result_list)
        #fix_coeffs(g, res_ent)
        len_max = max(len_cor_e, len_cor_ed, len_cor_p, len_cor_pe) 
        best_coeff_correct = max(best_coeff_correct, len_max)
        print_progress(len(key), len_max, i+1, args.iterations, current_run, runs, current_no, nonumbers)
        print(f"Maximal correct coefficients in this run: {best_coeff_correct}")
        runtime = datetime.now() - starttime 
        abort_success = abort_success or (fixed_incorrect == 0 and len(fixed) >= 512)
        if abort_success:
            success = True
            break
        abort_fail = False
        #Check fail conditions
        if last_improved['max_coeffs_correct'] < len_max:
            last_improved['max_coeffs_correct'] = len_max
            last_improved['last_changed'] = i
        else:
            if i-last_improved['last_changed'] >= args.no_improve_abort:
                print("No improvement for {} steps. Aborting.".format(i-last_improved['last_changed']))
                abort_fail = True
        print("Running since {} for {} minutes.".format(starttimestr, runtime.total_seconds()//60))
        if abort_fail:
            success = False
            break
    runtime = datetime.now() - starttime 
    if success:
        print("Succeeded with {} inequalities in {} iterations ({} minutes).".format(number, i+1, runtime.total_seconds()//60))
    else:
        print("Failed with {} inequalities after {} iterations ({} minutes).".format(number, i+1, runtime.total_seconds()//60))
    return {'avgp': avgp,
            'avgrk': avgrk,
            'correct': correct,
            'success': success,
            'iterations': i,
            'max_iterations': args.iterations,
            'seed': seed,
            'number': number,
            'len_correct_prob': len_cor_p,
            'len_correct_ent': len_cor_e,
            'len_correct_ent_diff': len_cor_ed,
            'len_correct_prob_ent': len_cor_pe,
            'ineqs': run_file,
            'parameter_set': ver,
            'best_coeff_correct': best_coeff_correct}

if __name__ == '__main__':
    test_bin_tree()
   # test_check_bp()
    test()
    main()
