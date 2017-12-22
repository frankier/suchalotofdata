use std::cmp::max;

use streamcard::StreamCard;

static ALPHAS: [f64; 11] = [
    0.0176262649461293,
    0.3511939471167616,
    0.5324346139959726,
    0.6256087109372577,
    0.6731020238676673,
    0.6971226338010236,
    0.7092084528700252,
    0.7152711899613444,
    0.7183076381917926,
    0.7198271478203816,
    0.7205872259765094
];

pub struct HLL {
    addr_bits: u8,
    buckets: Vec<u8>,
    // cached
    alpha: f64,
    num_buckets: u32
}

impl HLL {
    pub fn new(addr_bits: u8) -> HLL {
        assert!(addr_bits <= 31);
        let num_buckets: u32 = 1 << addr_bits;
        let alpha;
        if (addr_bits as usize) < ALPHAS.len() {
            alpha = ALPHAS[addr_bits as usize];
        } else {
            alpha = 0.7213 / (1.0 + 1.079 / (num_buckets as f64));
        }
        HLL {
            addr_bits,
            buckets: vec![0; num_buckets as usize],
            alpha,
            num_buckets
        }
    }
}


impl StreamCard for HLL {
    fn observe(&mut self, obs: u64) {
        let mask = (self.num_buckets - 1) as u64;
        let idx = (obs & mask) as usize;
        let val = (obs & !mask) << self.addr_bits;
        let first_1 = (val.leading_zeros() + 1) as u8;
        self.buckets[idx] = max(self.buckets[idx], first_1);
    }

    fn card(&self) -> u64 {
        let mut empty_buckets: u8 = 0;
        let mut sum: f64 = 0.0;
        for bucket in self.buckets.iter() {
            if *bucket == 0 {
                empty_buckets += 1;
            }
            sum += (-(*bucket as f64)).exp2();
        }
        let num_buckets = self.num_buckets as f64;
        let est = (self.alpha * num_buckets * num_buckets * sum.recip()) as u64;
        if (2 * est <= 5 * (self.num_buckets as u64)) &&
                empty_buckets == 0 {
            (num_buckets *
             (num_buckets / (empty_buckets as f64)).ln()) as u64
        } else {
            est
        }
    }
}
