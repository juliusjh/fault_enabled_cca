from python_kyber import Polyvec

def transpose(A):
    Alist = [A[i].to_list() for i in range(len(A))]
    return [Polyvec.new_from_list([Alist[i][j] for i in range(len(A))]) for j in range(len(A))]
