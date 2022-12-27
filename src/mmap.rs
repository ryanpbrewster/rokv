use std::fs::File;

use byteorder::{ByteOrder, LittleEndian};

use crate::farmhash_fingerprint;

pub struct Reader<'a> {
    _file: &'a File, // this field is only here to ensure that the file isn't dropped while we have it mapped
    buf: memmap::Mmap,
    table: Vec<u32>,
}
impl<'a> Reader<'a> {
    pub fn new(file: &'a mut File) -> anyhow::Result<Self> {
        let buf = unsafe { memmap::Mmap::map(file)? };
        let table_offset = LittleEndian::read_u32(&buf[0..4]) as usize;
        let table_len = LittleEndian::read_u32(&buf[4..8]) as usize;

        let mut table_reader = &buf[table_offset..];
        let mut table = vec![0; table_len];
        for slot in table.iter_mut() {
            *slot = LittleEndian::read_u32(&table_reader[..4]);
            table_reader = &table_reader[4..];
        }

        Ok(Reader {
            _file: file,
            buf,
            table,
        })
    }
    pub fn read(&mut self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let (h1, h2) = farmhash_fingerprint(key);

        let s1 = self.table[h1 as usize % self.table.len()];
        if s1 > 0 {
            println!("looking for {:?} at {} -> {}", key, h1, s1);
            if let Some(v) = self.try_read(key, s1)? {
                return Ok(Some(v));
            }
        } else {
            println!("skipping {:?} slot 1: {}", key, h1);
        }

        let s2 = self.table[h2 as usize % self.table.len()];
        if s2 > 0 {
            println!("looking for {:?} at {} -> {}", key, h2, s2);
            if let Some(v) = self.try_read(key, s2)? {
                return Ok(Some(v));
            }
        } else {
            println!("skipping {:?} slot 1: {}", key, h2);
        }

        Ok(None)
    }
    fn try_read(&mut self, key: &[u8], offset: u32) -> anyhow::Result<Option<Vec<u8>>> {
        let mut block = &self.buf[offset as usize..];

        let key_len = LittleEndian::read_u32(&block[..4]) as usize;
        block = &block[4..];

        let key_buf = &block[..key_len];
        block = &block[key_len..];

        if key_buf != key {
            return Ok(None);
        }

        let value_len = LittleEndian::read_u32(&block[..4]) as usize;
        block = &block[4..];

        let value_buf = &block[..value_len];
        Ok(Some(value_buf.to_owned()))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn smoke_test() -> anyhow::Result<()> {
        let mut file = tempfile::tempfile()?;

        {
            let mut w = crate::sync_read::Writer::new(&mut file)?;
            w.append(b"hello", b"world")?;
            w.finish()?;
        }

        let mut r = super::Reader::new(&mut file)?;
        assert_eq!(r.read(b"hello")?, Some(b"world".to_vec()));
        assert_eq!(r.read(b"foo")?, None);

        Ok(())
    }
}
