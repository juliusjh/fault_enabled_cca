use pyo3::prelude::*;

pub struct BinMultTree<T: Clone> {
    downward_tree: Vec<Vec<T>>,
    upward_tree: Option<Vec<Vec<T>>>,
    layer_down: usize, //Points to the last layer calculated
    layer_up: usize,
    mult: fn(&T, &T) -> T,
}

pub fn pointwise_mult(op0: &Vec<f64>, op1: &Vec<f64>) -> Vec<f64> {
    let mut res: Vec<f64> = op0.iter().zip(op1.iter()).map(|(x0, x1)| x0 * x1).collect();

    let max: f64 = res
        .iter()
        .max_by(|p0, p1| {
            p0.abs()
                .partial_cmp(&p1.abs())
                .unwrap_or(std::cmp::Ordering::Less)
        })
        .map(|p| p.abs())
        .unwrap_or(f64::NAN);
    if max.is_nan() || max == 0.0 {
        panic!("Could not normalize.");
    }
    res.iter_mut().for_each(|p| *p /= max);
    res
}

impl<T: Clone> BinMultTree<T> {
    pub fn new(leaves: Vec<T>, mult: fn(&T, &T) -> T) -> Self {
        assert_eq!(leaves.len() % 2, 0);
        assert!(leaves.len() >= 2);
        let downward_tree = vec![leaves];
        let upward_tree = None;
        let layer_down = 0;
        let layer_up = 0;
        Self {
            downward_tree,
            upward_tree,
            layer_down,
            layer_up,
            mult,
        }
    }
    fn next_downward_layer(&mut self) {
        assert!(self.upward_tree.is_none());
        let already_calc_layer = self.layer_down;
        let to_calc_layer = self.layer_down + 1;
        let down_tree = &mut self.downward_tree;
        down_tree.push(Vec::with_capacity(down_tree[already_calc_layer].len() * 2));
        assert_eq!(down_tree.len(), to_calc_layer + 1);
        assert!(down_tree[already_calc_layer].len() > 2);
        let mut last = down_tree[already_calc_layer].len();
        let pad = last % 2 != 0;
        if pad {
            last -= 1;
        }
        for i in (0..last).step_by(2) {
            let op0 = &down_tree[already_calc_layer][i];
            let op1 = &down_tree[already_calc_layer][i + 1];
            let prod = (self.mult)(op0, op1);
            down_tree[to_calc_layer].push(prod);
        }
        //Multiply the last, uneven entry with an imagined 1
        if pad {
            let last_elem = down_tree[already_calc_layer][last].clone();
            down_tree[to_calc_layer].push(last_elem);
        }
        self.layer_down += 1;
    }
    fn next_upward_layer(&mut self) {
        let down_tree = &mut self.downward_tree;
        let up_tree = match &mut self.upward_tree {
            Some(d) => d,
            None => panic!("Upward tree is none."),
        };
        assert_eq!(up_tree.len(), self.layer_up + 1);
        assert!(self.layer_down > self.layer_up);
        let already_calc_layer = self.layer_up;
        let to_calc_layer = self.layer_up + 1;
        let down = down_tree.pop().expect("Downtree empty.");
        let mut len = down.len();
        let pad = down.len() % 2 != 0;
        up_tree.push(Vec::with_capacity(down.len()));
        if pad {
            len -= 1;
        }
        //assert_eq!(len, 2*up_tree[already_calc_layer].len());
        for i in (0..len).step_by(2) {
            let op_down_0 = &down[i];
            let op_down_1 = &down[i + 1];
            let op_up = &up_tree[already_calc_layer][i / 2];
            let prod0 = (self.mult)(op_down_0, op_up);
            let prod1 = (self.mult)(op_down_1, op_up);
            up_tree[to_calc_layer].push(prod1);
            up_tree[to_calc_layer].push(prod0);
        }
        if pad {
            let last_elem = up_tree[already_calc_layer][len / 2].clone();
            up_tree[to_calc_layer].push(last_elem);
        }
        self.layer_up += 1;
    }

    fn calculate_down(&mut self, prior: Option<&T>) {
        while self.downward_tree[self.layer_down].len() > 2 {
            self.next_downward_layer();
        }
        let mut last_down = self.downward_tree.pop().expect("No downward tree.");
        assert_eq!(last_down.len(), 2);
        let last_down1 = last_down.pop().unwrap();
        let last_down0 = last_down.pop().unwrap();
        self.upward_tree = match prior {
            Some(prior) => Some(vec![vec![
                (self.mult)(&last_down1, &prior),
                (self.mult)(&last_down0, &prior),
            ]]),
            None => Some(vec![vec![last_down1, last_down0]]),
        };
    }

    fn calculate_up(&mut self) {
        while self.layer_up < self.layer_down {
            self.next_upward_layer();
        }
    }

    pub fn calculate(mut self) -> Vec<T> {
        self.calculate_down(None);
        self.calculate_up();
        self.upward_tree
            .expect("No upward tree.")
            .remove(self.layer_up)
    }

    pub fn calculate_with_prior(mut self, prior: &T) -> Vec<T> {
        self.calculate_down(Some(prior));
        self.calculate_up();
        self.upward_tree
            .expect("No upward tree.")
            .remove(self.layer_up)
    }
}

#[pyclass]
pub struct PyBinMultTreeList {
    tree: Option<BinMultTree<Vec<f64>>>,
}

#[pymethods]
impl PyBinMultTreeList {
    #[new]
    pub fn new(leaves: Vec<Vec<f64>>) -> Self {
        Self {
            tree: Some(BinMultTree::new(leaves, pointwise_mult)),
        }
    }
    pub fn calculate(&mut self) -> Vec<Vec<f64>> {
        let t = std::mem::replace(&mut self.tree, None).expect("Tree not valid.");
        t.calculate()
    }
}

fn mult_int(op0: &i32, op1: &i32) -> i32 {
    op0 * op1
}

#[pyclass]
pub struct PyBinMultTreeInt {
    tree: Option<BinMultTree<i32>>,
}

#[pymethods]
impl PyBinMultTreeInt {
    #[new]
    pub fn new(leaves: Vec<i32>) -> Self {
        Self {
            tree: Some(BinMultTree::new(leaves, mult_int)),
        }
    }
    pub fn calculate(&mut self) -> Vec<i32> {
        let t = std::mem::replace(&mut self.tree, None).expect("Tree not valid.");
        t.calculate()
    }
}
