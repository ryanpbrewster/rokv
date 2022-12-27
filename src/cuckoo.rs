const MAX_ATTEMPTS: usize = 50;
pub trait Entry: Copy + std::fmt::Debug {
    fn h1(&self) -> usize;
    fn h2(&self) -> usize;
}
pub fn assemble_cuckoo<T: Entry>(input: &[T], cap: usize) -> anyhow::Result<Vec<Option<usize>>> {
    let mut table = vec![None; cap];
    for i in 0..input.len() {
        let mut cur = i;
        for attempt in 0.. {
            if attempt > MAX_ATTEMPTS {
                return Err(anyhow::format_err!("could not place {:?}", i));
            }
            let l1 = input[cur].h1() % cap;
            if table[l1].is_none() {
                table[l1] = Some(cur);
                break;
            }
            let l2 = input[cur].h2() % cap;
            if let Some(prev) = table[l2].replace(cur) {
                cur = prev;
            } else {
                break;
            }
        }
    }
    Ok(table)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct TestEntry {
        h1: u64,
        h2: u64,
    }
    impl Entry for TestEntry {
        fn h1(&self) -> usize {
            self.h1 as usize
        }
        fn h2(&self) -> usize {
            self.h2 as usize
        }
    }

    #[test]
    fn smoke() -> anyhow::Result<()> {
        let input = vec![
            TestEntry { h1: 1, h2: 2 },
            TestEntry { h1: 3, h2: 4 },
            TestEntry { h1: 1, h2: 3 },
            TestEntry { h1: 2, h2: 4 },
        ];
        let table = assemble_cuckoo(&input, 2 * input.len())?;
        for (i, e) in input.iter().enumerate() {
            let s1 = table[e.h1() % table.len()];
            let s2 = table[e.h2() % table.len()];
            assert!(
                s1 == Some(i) || s2 == Some(i),
                "item {} == {:?} -> [{:?}, {:?}]",
                i,
                e,
                s1,
                s2
            );
        }
        Ok(())
    }
}
