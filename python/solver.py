
from check_bp import CheckGraph

def create_graph(mat_ge, mat_le, vec_ge, vec_le, dist, is_equals_le, is_equals_ge):
    print("Building check graph..")
    g = CheckGraph()
    g.add_var_nodes(dist) 
    lineno = 0
    maxv = 0
    for row, val, eq in zip(mat_ge, vec_ge, is_equals_ge):
        maxv = max(maxv, max(row))  
        g.add_equation("Line {}".format(lineno), row, val, False, eq)
        lineno += 1
    for row, val, eq in zip(mat_le, vec_le, is_equals_le):
        maxv = max(maxv, max(row))  
        g.add_equation("Line {}".format(lineno), row, val, True, eq)
        lineno += 1
    print("Maximal value in equations: ", maxv)
    return g



