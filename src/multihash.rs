use rand::{SeedableRng, Rng};

pub struct MultihashTable {
    num_hashes: u32,
    slot_capacity: usize,
    num_slots: usize,
    slots: Vec<Option<String>>, // slots.len() == slot_capacity * num_slots
    prng: rand::rngs::SmallRng,
}

const MAX_ATTEMPTS: usize = 50;

impl MultihashTable {
    pub fn new(num_hashes: u32, slot_capacity: usize, num_slots: usize) -> MultihashTable {
        MultihashTable {
            num_hashes,
            slot_capacity,
            num_slots,
            slots: vec![None; num_slots * slot_capacity],
            prng: rand::rngs::SmallRng::seed_from_u64(42),
         }
    }

    pub fn insert(&mut self, item: String) -> bool {
        // Try to find a slot with some vacant capacity
        for i in 0 .. self.num_hashes {
            let h = farmhash::hash32_with_seed(item.as_bytes(), i) as usize;
            let base = (h % self.num_slots) * self.slot_capacity;
            for j in 0 .. self.slot_capacity {
                if self.slots[base + j].is_none() {
                    self.slots[base + j] = Some(item);
                    return true;
                }
            }
        }

        // If not, evict a random item from a random slot and try to re-locate it.
        let mut cur = item;
        let mut loc = {
            let h = farmhash::hash32_with_seed(cur.as_bytes(), self.prng.gen_range(0..self.num_hashes)) as usize;
            let base = (h % self.num_slots) * self.slot_capacity;
            base + self.prng.gen_range(0 .. self.slot_capacity)
        };
        for _ in 0 .. MAX_ATTEMPTS {
            if let Some(prev) = self.slots[loc].replace(cur) {
                cur = prev;
                loc = loop {
                    let h = farmhash::hash32_with_seed(cur.as_bytes(), self.prng.gen_range(0..self.num_hashes)) as usize;
                    let base = (h % self.num_slots) * self.slot_capacity;
                    let candidate = base + self.prng.gen_range(0 .. self.slot_capacity);
                    if candidate != loc {
                        break candidate;
                    }
                }
            } else {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const BASE_CAPACITY: usize = 1_024;
    const TRIALS: usize = 100;
    fn achievable_load(num_hashes: u32, slot_capacity: usize) -> f64 {
        let mut total = 0.0;

        for trial in 0 .. TRIALS {
            let mut h = MultihashTable::new(num_hashes, slot_capacity, BASE_CAPACITY);
            let mut count = 0;
            while h.insert(format!("trial-{}-key-{}", trial, count)) {
                println!("inserted trial-{}-key-{}", trial, count);
                count += 1;
            }
            total += count as f64 / (BASE_CAPACITY * slot_capacity) as f64;
        }
        // Round to 3 decimal places
        (1e3 * total / TRIALS as f64).round() / 1e3
    }

    #[test]
    fn achievable_load_golden_test() {
        assert_eq!(achievable_load(2, 1), 0.564);
        assert_eq!(achievable_load(3, 1), 0.832);
        assert_eq!(achievable_load(4, 1), 0.853);
        assert_eq!(achievable_load(5, 1), 0.892);
        assert_eq!(achievable_load(6, 1), 0.892);
        assert_eq!(achievable_load(7, 1), 0.920);
        assert_eq!(achievable_load(8, 1), 0.919);

        assert_eq!(achievable_load(2, 2), 0.785);
        assert_eq!(achievable_load(3, 2), 0.858);
        assert_eq!(achievable_load(4, 2), 0.920);
        assert_eq!(achievable_load(5, 2), 0.914);
        assert_eq!(achievable_load(6, 2), 0.919);
        assert_eq!(achievable_load(7, 2), 0.927);
        assert_eq!(achievable_load(8, 2), 0.895);

        assert_eq!(achievable_load(2, 3), 0.806);
        assert_eq!(achievable_load(3, 3), 0.914);
        assert_eq!(achievable_load(4, 3), 0.885);
        assert_eq!(achievable_load(5, 3), 0.931);
        assert_eq!(achievable_load(6, 3), 0.905);
        assert_eq!(achievable_load(7, 3), 0.945);
        assert_eq!(achievable_load(8, 3), 0.928);

        assert_eq!(achievable_load(2, 4), 0.891);
        assert_eq!(achievable_load(3, 4), 0.880);
        assert_eq!(achievable_load(4, 4), 0.920);
        assert_eq!(achievable_load(5, 4), 0.943);
        assert_eq!(achievable_load(6, 4), 0.942);
        assert_eq!(achievable_load(7, 4), 0.913);
        assert_eq!(achievable_load(8, 4), 0.940);
    }

    #[test]
    fn farmhash_does_not_explode() {
        let _ = farmhash::hash32_with_seed("trial-0-key-27".as_bytes(), 1);
    }
}