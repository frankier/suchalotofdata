use std::collections::BTreeSet;

use streamcard::StreamCard;

pub struct KMV {
    k: u32,
    vals: BTreeSet<u64>
}

impl KMV {
    pub fn new(k: u32) -> KMV {
        assert!(k >= 1);
        KMV {
           k,
           // Underlying data structure needs to support:
           //   * Maintaining set property/uniqueness
           //   * Efficiently finding and the maximum
           //   * Efficiently replacing the maximum
           //       (with another element which may not be the maximum)
           //
           // BTreeSet supports these, but possibly something could be done with a min-heap.
           // A sorted vector might be more efficient in practice for small k.
           vals: BTreeSet::<u64>::new()
        }
    }
}

impl StreamCard for KMV {
    fn observe(&mut self, obs: u64) {
        if self.vals.len() < (self.k as usize) {
            self.vals.insert(obs);
        } else {
            let top = *self.vals.iter().next_back().unwrap();
            if obs < top {
                self.vals.insert(obs);
                if self.vals.len() >= (self.k as usize) {
                    assert!(self.vals.remove(&top));
                }
            }
        }
    }

    fn card(&self) -> u64 {
        let top = self.vals.iter().next_back();
        if let Some(t) = top {
            ((u64::max_value() as f64) / (*t as f64)
                    * (self.vals.len() as f64)) as u64
        } else {
            0
        }
    }
}
