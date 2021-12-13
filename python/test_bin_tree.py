import random
from check_bp import PyBinMultTreeInt as IntTree
from check_bp import PyBinMultTreeList as ListTree

def test_int_tree():
    l = [1 for _ in range(1024)]
    l[0] = 2
    res = IntTree(l).calculate()
    for i, v in enumerate(res):
        if i != 0:
            assert(v == 2)
        else:
            assert(v == 1)

def test_int_tree_no_two_power():
    l = [1 for _ in range(1026)]
    l[0] = 2
    res = IntTree(l).calculate()
    for i, v in enumerate(res):
        if i != 0:
            assert(v == 2)
        else:
            assert(v == 1)


def test_int_tree_random():
    l = [random.randint(-1, 1) for _ in range(10)]
    for i, v in enumerate(l):
        if v == 0:
            l[i] = 1
    res = IntTree(l).calculate()
    res2 = [1]*len(l)
    for i in range(len(l)):
        for j, v in enumerate(l):
            if i != j:
                res2[i] *= v
    assert(res == res2)


def test_list_tree():
    l = [[random.randrange(1, 5) for _ in range(100)] for _ in range(100)]
    res = ListTree(l).calculate()
    res2 = [[1]*len(l[0]) for _ in range(len(l))]
    for i in range(len(l)):
        for j, v in enumerate(l):
            if i != j:
                zipped = list(zip(res2[i], v))
                res2[i] = [x*y for x,y in zip(res2[i], v)]
                v = max(res2[i])
                res2[i] = list(map(lambda x: x/v, res2[i]))
    diffs = [all([abs(r_elem - r2_elem) < 0.000001 for r_elem, r2_elem in zip(r, r2)]) for r, r2 in zip(res, res2)]
    assert(all(diffs))
    

def test_bin_tree():
    test_int_tree()
    test_int_tree_no_two_power()
    for _ in range(10):
        test_int_tree_random()
        test_list_tree()


