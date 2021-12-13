from python_kyber import Poly, Polyvec

def compress_decompress(p):
    if type(p).__name__ == 'Polyvec':
        return Polyvec.from_bytes_compressed(p.to_bytes_compressed())
    elif type(p).__name__ == 'Poly':
        return Poly.from_bytes_compressed(p.to_bytes_compressed())
    raise ValueError("{} it not a poly or a polyvec".format(type(p)))
