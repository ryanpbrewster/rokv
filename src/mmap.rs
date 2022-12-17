use std::fs::File;

use byteorder::{ByteOrder, LittleEndian};

use crate::cdb_hash;


pub struct Reader<'a> {
    _file: &'a File, // this field is only here to ensure that the file isn't dropped while we have it mapped
    buf: memmap::Mmap,
    table: Vec<usize>,
}
impl<'a> Reader<'a> {
    pub fn new(file: &'a mut File) -> anyhow::Result<Self> {
        let buf = unsafe { memmap::Mmap::map(file)? };
        let table_offset = LittleEndian::read_u32(&buf[0..4]) as usize;
        let table_len = LittleEndian::read_u32(&buf[4..8]) as usize;

        let mut table_reader = &buf[table_offset..];
        let mut table = vec![0; table_len];
        for slot in table.iter_mut() {
            *slot = LittleEndian::read_u32(&table_reader[..4]) as usize;
            table_reader = &table_reader[4..];
        }

        Ok(Reader { _file: file, buf, table })
    }
    pub fn read(&mut self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let mut slot = cdb_hash(key) as usize % self.table.len();
        while self.table[slot] > 0 {
            let mut block = &self.buf[self.table[slot]..];

            let key_len = LittleEndian::read_u32(&block[..4]) as usize;
            block = &block[4..];

            let key_buf = &block[..key_len];
            block = &block[key_len..];

            if key_buf != key {
                slot = (slot + 1) % self.table.len();
                continue;
            }

            let value_len = LittleEndian::read_u32(&block[..4]) as usize;
            block = &block[4..];

            let value_buf = &block[..value_len];
            return Ok(Some(value_buf.to_owned()));
        }
        Ok(None)
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