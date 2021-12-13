
def to_bit_list(x, n=8):
    return [(x >> i) & 1 for i in range(n)]

def count_flipped_bits(ct, ct_manip):
#    bits_uncompressed = [b for bl in [to_bit_list(x, 16) for x in ct.v.to_list()] for b in bl]
#    bits_manip_uncompressed = [b for bl in [to_bit_list(x, 16) for x in ct_manip.v.to_list()] for b in bl]
#    flipped_uncompressed = [b ^ bm for b, bm in zip(bits_uncompressed, bits_manip_uncompressed)]
    bits = [b for bl in [to_bit_list(x) for x in ct.to_bytes_list()] for b in bl]
    bits_manip = [b for bl in [to_bit_list(x) for x in ct_manip.to_bytes_list()] for b in bl]
    flipped = [b ^ bm for b, bm in zip(bits, bits_manip)]
    return sum(flipped)#, sum(flipped_uncompressed), [i for i, b in enumerate(flipped) if b == 1], [i for i, b in enumerate(flipped_uncompressed) if b == 1]
