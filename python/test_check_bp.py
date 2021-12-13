import numpy as np
import random
from check_bp import CheckGraph, test_fft_2, test_fft_3

def test_check_bp_le():
    g = CheckGraph()
    g.add_var_nodes({0: 0.5, 1: 0.5}) 
    for i in range(5): 
        g.add_equation(str(i), [1 if j == i else 0 for j in range(2048)], 0, True, False)
    g.ini()
    print("Propagating")
    g.propagate(2, 1)
    for i in range(5):
        res = g.get_result(i)
        for v, p in res.items():
            if v == 0:
                assert(abs(p - 1) <= 0.000001)
            else:
                assert(abs(p) <= 0.0000001)
    print("At 100:")
    print(g.get_result(100))

def test_check_bp_ge():
    g = CheckGraph()
    g.add_var_nodes({0: 0.5, -1: 0.5}) 
    for i in range(5): 
        g.add_equation(str(i), [1 if j == i else 0 for j in range(2048)], 0, False, False)
    g.ini()
    print("Propagating")
    g.propagate(2, 1)
    for i in range(5):
        res = g.get_result(i)
        print(res)
        for v, p in res.items():
            if v == 0:
                assert(abs(p - 1) <= 0.000001)
            else:
                assert(abs(p) <= 0.0000001)
    print("At 100:")
    print(g.get_result(100))

def test_check_bp_2():
    g = CheckGraph()
    g.add_var_nodes({-1: 0.3, 0: 0.4, 1: 0.3}) 
    for i in range(5): 
        g.add_equation(str(i), [2 if j == i else 0 for j in range(2048)], 1, True, False)
    for i in range(5): 
        g.add_equation(str(i), [2 if j == i else 0 for j in range(2048)], -1, False, False)
    g.ini()
    print("Propagating")
    g.propagate(2, 1)
    print("At 0:")
    print(g.get_result(0))
    for i in range(5):
        res = g.get_result(i)
        print(res)
        for v, p in res.items():
            if v == 0:
                assert(abs(p - 1) <= 0.000001)
            else:
                assert(abs(p) <= 0.0000001)
    print("At 100:")
    print(g.get_result(100))

def test_check_bp_3():
    g = CheckGraph()
    g.add_var_nodes({-1: 0, 0: 0.5, 1: 0.5}) 
    for i in range(0, 6, 2): 
        g.add_equation(str(i), [1 if j == i or j == i+1 else 0 for j in range(2048)], 1, True, False)
    g.ini()
    print("Propagating")
    g.propagate(2, 1)
    for i in range(0, 6, 2):
        res = g.get_result(i)
        s = sum(res.values())
        res = {v: p/s for v, p in res.items()}
        for v, p in res.items():
            if v == 0:
                assert(abs(p - 2/3) <= 0.000001)
            elif v == 1:
                assert(abs(p - 1/3) <= 0.0000001)
            else:
                assert(abs(p) <= 0.0000001)
    print("At 100:")
    print(g.get_result(100))

def test_random_equations(key_len=1024, eq_count=8000, max_err=20):
    g = CheckGraph()
    g.set_check_validity(True)
    key = [random.randint(-2, 2) for _ in range(key_len)]
    print(key)
    equations = []
    #g.add_var_nodes({-2: 1, 2: 1, -1: 1, 1: 1, 0: 1});
    g.add_var_nodes({-2: 0.05, -1: 0.25, 0: 0.4, 1: 0.25, 2: 0.05}) 
    for i in range(eq_count):
        coeffs = [random.randint(-2, 2) for _ in range(key_len)] 
        val = sum([c*k for c, k in zip(coeffs, key)])
        le = bool(random.randint(0, 1))
        add = random.randint(0, max_err)
        if le:
            val += add
        else:
            val -= add
        equations.append((coeffs, le, val))    
        g.add_equation(str(i), coeffs, val, le, False);
    g.ini()
    for _ in range(5):
        g.propagate(2, 30)
        print(g.get_result(0))
    def most_likely(res):
        mp = 0
        mv = None
        for v, p in res.items():
            if p > mp:
                mv = v
                mp = p
        return mv
    k = [most_likely(g.get_result(i)) for i in range(key_len)] 
    print(key)
    print(k)
    assert(k == key)


def test_fft_py():
    op0 = [0.0, 0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0]
    op1 = [0.0, 0.0, 0.2, 0.2, 0.4, 0.0, 0.2, 0.0]
    conv, fft = test_fft_2(op0, op1)
    diffs = [abs(f-c) < 0.00001 for f,c in zip(fft, conv)]
    diffs2 = [abs(f-c) < 0.00001 for f,c in zip(np.convolve(op0, op1), conv)]
    assert(all(diffs))
    assert(all(diffs2))

    #return 
    op0 = [0.0, 0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0]
    op1 = [0.0, 0.0, 0.2, 0.2, 0.4, 0.0, 0.2, 0.0]
    op2 = [0.0, 0.0, 0.1, 0.3, 0.4, 0.0, 0.2, 0.0]
    conv, fft = test_fft_3(op0, op1, op2)
    diffs = [abs(f-c) < 0.00001 for f,c in zip(fft, conv)]
    diffs2 = [abs(f-c) < 0.00001 for f,c in zip(np.convolve(np.convolve(op0, op1), op2), conv)]
    assert(all(diffs2))
    assert(all(diffs))


def test_check_bp():
    test_random_equations()
    return
    test_fft_py()
    test_check_bp_ge()
    test_check_bp_le()
    test_check_bp_2()
    test_check_bp_3()
