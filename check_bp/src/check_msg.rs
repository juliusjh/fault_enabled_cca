use belief_propagation::{BPError, BPResult, Msg};
use rustfft::{num_complex::Complex, Fft};
use std::fmt::Debug;
use std::sync::Arc;

//N = 2^k!
//ETA is actually (eta-1)/2, for eta=2 use 5!
#[derive(Copy, Clone, Debug)]
pub struct CheckMsg<const ETA: usize> {
    pub data: [f64; ETA],
}
//TODO: Use N instead of self.len() (currently only nightly)
impl<const ETA: usize> CheckMsg<ETA> {
    pub fn new() -> Self {
        CheckMsg { data: [0.0; ETA] }
    }
    pub fn from_data(data: [f64; ETA]) -> Self {
        CheckMsg { data: data }
    }
    pub fn get(&self, index: i16) -> Option<f64> {
        self.data
            .get((index + self.data.len() as i16 / 2) as usize)
            .map(|p| *p)
    }
    pub fn get_mut(&mut self, index: i16) -> Option<&mut f64> {
        let len = self.data.len();
        self.data.get_mut((index + len as i16 / 2) as usize)
    }
    pub fn set_fixed_value(&mut self, value: i16) {
        for p in self.data.iter_mut() {
            *p = 0.0;
        }
        self[value] = 1.0;
    }
    pub fn mult_and_transform(
        self,
        scalar: i16,
        size: usize,
        fft: &Arc<dyn Fft<f64>>,
    ) -> Vec<Complex<f64>> {
        let mut indata = vec![
            Complex {
                re: 0 as f64,
                im: 0 as f64
            };
            size
        ];
        for (v, p) in self {
            let mut idx = scalar * v;
            //TODO: Avoid if/move out of loop
            if idx < 0 {
                idx += size as i16;
            }
            indata[idx as usize] += Complex { re: p, im: 0.0 };
        }
        fft.process(&mut indata);
        indata
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn to_probabilities(&mut self) {
        let sum: f64 = self.data.iter().sum();
        if sum == 0.0 || sum.is_nan() {
            panic!("No valid message encountered in to_probabilities.");
        }
        self.data.iter_mut().for_each(|p| *p /= sum);
    }
}

impl<const ETA: usize> Msg<i16> for CheckMsg<ETA> {
    fn new() -> Self {
        CheckMsg { data: [0.0; ETA] }
    }
    fn get(&self, value: i16) -> Option<f64> {
        CheckMsg::<ETA>::get(self, value)
    }
    fn get_mut(&mut self, value: i16) -> Option<&mut f64> {
        CheckMsg::<ETA>::get_mut(self, value)
    }
    fn insert(&mut self, value: i16, p: f64) {
        self[value] = p;
        //self.data[(value + self.len() as i16/2) as usize] = p;
    }
    fn normalize(&mut self) -> BPResult<()> {
        let max = {
            *self
                .data
                .iter()
                .max_by(|p0, p1| p0.partial_cmp(p1).unwrap_or(std::cmp::Ordering::Less))
                .unwrap_or(&f64::NAN)
        };
        if max == 0 as f64 || max.is_nan() {
            println!("Dist: {:?}", self.data.to_vec());
            return Err(BPError::new(
                "CheckMsg::Msg".to_owned(),
                "Did not find a useful value to normalize by".to_owned(),
            ));
        }
        self.data.iter_mut().for_each(|p| *p /= max);
        Ok(())
    }
    fn is_valid(&self) -> bool {
        self.data.iter().all(|p| !p.is_nan()) // && *p >= 0.0 && *p <= 1.0)
    }
    fn mult_msg(&mut self, other: &Self) {
        self.data.iter_mut().zip(other.data.iter()).for_each(|(p_self, p_other)| *p_self *= p_other);
    }
}

impl<const ETA: usize> std::ops::Index<i16> for CheckMsg<ETA> {
    type Output = f64;
    fn index(&self, val: i16) -> &Self::Output {
        &self.data[(val + self.len() as i16 / 2) as usize]
    }
}
impl<const ETA: usize> std::ops::IndexMut<i16> for CheckMsg<ETA> {
    fn index_mut(&mut self, val: i16) -> &mut Self::Output {
        &mut self.data[(val + self.len() as i16 / 2) as usize]
    }
}

//Iterators
impl<const ETA: usize> IntoIterator for CheckMsg<ETA> {
    type Item = (i16, f64);
    type IntoIter = CheckMsgIntoIterator<ETA>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            msg: self,
            index: -(self.len() as i16) / 2,
        }
    }
}

impl<'a, const ETA: usize> IntoIterator for &'a CheckMsg<ETA> {
    type Item = (i16, f64);
    type IntoIter = CheckMsgIterator<'a, ETA>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            msg: self,
            index: -(self.len() as i16) / 2,
        }
    }
}

pub struct CheckMsgIterator<'a, const ETA: usize> {
    msg: &'a CheckMsg<ETA>,
    index: i16,
}

impl<'a, const ETA: usize> Iterator for CheckMsgIterator<'a, ETA> {
    type Item = (i16, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.msg.get(self.index).map(|p| (self.index, p));
        self.index += 1;
        res
    }
}

pub struct CheckMsgIntoIterator<const ETA: usize> {
    msg: CheckMsg<ETA>,
    index: i16,
}

impl<const ETA: usize> Iterator for CheckMsgIntoIterator<ETA> {
    type Item = (i16, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.msg.get(self.index as i16).map(|p| (self.index, p));
        self.index += 1;
        res
    }
}
