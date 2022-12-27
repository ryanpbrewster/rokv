#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Entry {
    h1: usize,
    h2: usize,
}

pub struct CuckooTable {
    cells: Vec<Option<Entry>>,
}

const MAX_ATTEMPTS: usize = 50;
impl CuckooTable {
    pub fn new(size: usize) -> CuckooTable {
        CuckooTable {
            cells: vec![None; size],
        }
    }

    /// Returns true if the entry could be successfully added to the table.
    /// If not, the table needs to be either re-hashed or re-sized.
    pub fn insert(&mut self, mut e: Entry) -> bool {
        for _ in 0 .. MAX_ATTEMPTS {
            let (l1, l2) = (e.h1 % self.cells.len(), e.h2 % self.cells.len());
            if self.cells[l1].is_none() {
                self.cells[l1] = Some(e);
                return true;
            }
            match self.cells[l2].clone() {
                None => {
                    self.cells[l2] = Some(e);
                    return true;
                }
                Some(prev) => {
                    self.cells[l2] = Some(e);
                    e = prev;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let input = vec![
            Entry { h1: 1, h2: 2 },
            Entry { h1: 3, h2: 4 },
            Entry { h1: 1, h2: 3 },
            Entry { h1: 2, h2: 4 },
        ];
        let mut table = CuckooTable::new(16);
        for e in &input {
            assert!(table.insert(e.clone()), "could not insert {:?}", e);
        }
        let cells = table.cells;
        for (i, e @ Entry { h1, h2 }) in input.iter().enumerate() {
            let s1 = &cells[h1 % cells.len()];
            let s2 = &cells[h2 % cells.len()];
            assert!(
                s1 == &Some(e.clone()) || s2 == &Some(e.clone()),
                "item {} == {{{}, {}}} -> [{:?}, {:?}]",
                i,
                h1,
                h2,
                s1,
                s2
            );
        }
    }
}
