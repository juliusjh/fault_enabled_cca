from python_kyber import Poly, Ciphertext

def manipulate_coefficient(ct, index, add_error):
    v = ct.v.to_list()
    v[index] += add_error
    return Ciphertext(ct.b, Poly.from_list(v))
