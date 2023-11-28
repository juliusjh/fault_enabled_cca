import subprocess
import os

def bkz_fplll(basis, block_size=20, algo="lll", path="data", filename="fplll_mat.txt", add_fplll=""):
    basis = [[int(b) for b in bi] for bi in basis]
    basis = str(basis).replace(",", "")
    with open(filename, "w") as f:
        f.write(basis)
    cmd = f"fplll {add_fplll} -a {algo} -b {block_size} {filename}"
    print(f"Running '{cmd}'")
    output = subprocess.check_output(cmd, shell=True)
    output = output.decode("ascii").replace("\n", ",").replace(" ", ", ")
    mat = eval(output)[0]
    return mat[0][:-1]
