#!/usr/bin/env python3

import argparse
import glob
import re
import json
import numpy as np
import matplotlib as mpl
from matplotlib import pyplot as plt
from matplotlib import rc
from statistics import median, mean


def get_args():
    parser = argparse.ArgumentParser(description='CCA attack')
    parser.add_argument('--dir', '-d', type=str, default='results')

    args = parser.parse_args()

    files_path = f"{args.dir}/*.json"

    return files_path

def read_files(files_path):
    files = glob.glob(files_path) 
    results = {'512': {}, '768': {}, '1024': {}}
    for f in files:
        with open(f) as current_file:
            data = json.load(current_file)  
        assert(type(data) == list)
        assert(len(data) > 0)
        assert('number' in data[0])
        assert('parameter_set' in data[0])
        ver = data[0]['parameter_set'] 
        number = data[0]['number']
        assert(all([d['number'] == number for d in data]))
        assert(all([d['parameter_set'] == ver for d in data]))
        if number in results[ver]:
            print(f"WARNING: Duplicate files for {number} inequalities")
            results[ver][number] += data
        else:
            results[ver][number] = data
    return results


def plot_sr(results):
    successes = {'512': {}, '768': {}, '1024': {}}
    seeds = {'512': {}, '768': {}, '1024': {}}
    coeffs = {'512': {}, '768': {}, '1024': {}}
    for ver in results:
        for number in results[ver]:
            for experiment in results[ver][number]:
                if any([number != ex['number'] for ex in results[ver][number]]):
                    raise ValueError("Number differs from number in data set")
                if not number in seeds[ver]:
                    seeds[ver][number] = []
                if not number in successes[ver]:
                    successes[ver][number] = {'successes': 0, 'total': 0}
                if experiment['seed'] in seeds[ver][number]:
                    print("WARNING: Duplicate seed. Ignoring experiment.")
                    continue
                seeds[ver][number].append(experiment['seed'])
                #########SR#########
                if experiment['success']:
                    successes[ver][number]['successes'] += 1
                successes[ver][number]['total'] += 1
                #########/SR#########
                #########Coeffs#########
                if not number in coeffs[ver]:
                    coeffs[ver][number] = []
                coeffs[ver][number].append(max(experiment['len_correct_prob'], experiment['len_correct_ent'], experiment['len_correct_ent_diff'], experiment['len_correct_prob_ent']))
                #########/Coeffs#########
    
    to_plot_sr = {}
    to_plot_coeffs = {}
    for ver in ('512', '768', '1024'):
        to_plot_sr[ver] = sorted([(number, successes[ver][number]['successes']/successes[ver][number]['total']) for number in successes[ver]], key=lambda x: x[0])
        to_plot_coeffs[ver] = sorted([(number, mean(coeffs[ver][number])) for number in successes[ver]], key=lambda x: x[0])
    return to_plot_sr, to_plot_coeffs


def main():
    mpl.rc('font', family = 'serif', serif = 'cmr10')

    color = {'512': 'tab:blue', '768': 'tab:green', '1024': 'tab:orange'}
    marker = {'512': 'x', '768': 'o', '1024': '>'}

    ticks = [2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000]

    files_path = get_args()
    results = read_files(files_path)
    to_plot_sr, to_plot_coeffs = plot_sr(results)
    fig, ax1 = plt.subplots(1, 1, figsize=(4.8, 2.4))
    fig.tight_layout()
    #plt.subplots_adjust(left=None, bottom=0.1, right=None, top=None, wspace=None, hspace=0.5)

    #ax1.set_title('Success rate without additional lattice reduction')
    ax1.set(xlabel='Number of inequalities', ylabel='Success rate')
    ax1.set_ymargin(0.1)
    ax1.set_xmargin(0.1)
    ax1.set_xticks(ticks)
    #ax1.set_xlim(0, 9000)

    for ver in ('512', '768', '1024'):
        ax1.plot(*zip(*to_plot_sr[ver]), marker=marker[ver], label=f"Kyber{ver}", color=color[ver])
    ax1.vlines(x=ticks, ymin=0, ymax=1, colors='lightgrey', ls='--', lw=1)
    legend = ax1.legend()
    legend.get_frame().set_alpha(None)
    plt.savefig('results_sr.pdf', bbox_inches='tight')

    fig, ax2 = plt.subplots(1, 1, figsize=(4.8, 2.4))
    fig.tight_layout()
    plt.subplots_adjust(left=None, bottom=0.1, right=None, top=None, wspace=None, hspace=0.5)

    #ax2.set_title('Recovered coefficients (mean)')
    ax2.set(xlabel='Number of inequalities', ylabel='Recovered coefficients')
    ax2.set_ymargin(0.1)
    ax2.set_xmargin(0.1)
    ax2.set_xticks(ticks)
    #ax2.set_xlim(0, 10000)
    for ver in ('512', '768', '1024'):
        ax2.plot([2000, 9000], [int(ver)]*2, linestyle='dashed', color=color[ver])# label=f"{ver} coefficients")
        ax2.plot(*zip(*to_plot_coeffs[ver]), marker=marker[ver], color=color[ver], label=f"Kyber{ver}")
    ax2.vlines(x=ticks, ymin=0, ymax=1024, colors='lightgrey', ls='--', lw=1)
    legend = ax2.legend()
    legend.get_frame().set_alpha(None)
    #plt.show()
    plt.savefig('results_rec.pdf', bbox_inches='tight')


if __name__ == '__main__':
    main()
