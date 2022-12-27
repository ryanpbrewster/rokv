use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    cuckoo::{self, try_assemble_cuckoo},
    farmhash_fingerprint,
};

pub struct Writer<'a> {
    file: BufWriter<&'a mut File>,
    offset: u32,
    log: Vec<Entry>,
}
#[derive(Copy, Clone, Debug)]
struct Entry {
    key_hash: (u32, u32),
    offset: u32,
}
impl cuckoo::Entry for Entry {
    fn h1(&self) -> usize {
        self.key_hash.0 as usize
    }

    fn h2(&self) -> usize {
        self.key_hash.1 as usize
    }
}

impl<'a> Writer<'a> {
    pub fn new(file: &'a mut File) -> anyhow::Result<Self> {
        // Leave room for the header: a pair of u32 values indicating the offset + length of the footer table.
        file.seek(SeekFrom::Start(8))?;
        Ok(Writer {
            file: BufWriter::new(file),
            offset: 8,
            log: Vec::new(),
        })
    }
    pub fn append(&mut self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
        self.log.push(Entry {
            key_hash: farmhash_fingerprint(key),
            offset: self.offset,
        });
        self.offset += key.len() as u32 + value.len() as u32 + 8;

        self.file.write_u32::<LittleEndian>(key.len() as u32)?;
        self.file.write_all(key)?;

        self.file.write_u32::<LittleEndian>(value.len() as u32)?;
        self.file.write_all(value)?;

        Ok(())
    }
    pub fn finish(mut self) -> anyhow::Result<()> {
        // Compute a hashtable. We're using linear probing, so make sure the
        // load factor is not too high. We'll go with 0.5 for now.
        let table = try_assemble_cuckoo(&self.log, (2..=5).map(|k| k * self.log.len()))?;

        // Write the hashtable.
        for &idx in &table {
            self.file.write_u32::<LittleEndian>(
                idx.map(|idx| self.log[idx].offset).unwrap_or(0) as u32,
            )?;
        }

        // Write the offset for the hashtable.
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_u32::<LittleEndian>(self.offset)?;
        self.file.write_u32::<LittleEndian>(table.len() as u32)?;
        self.file.flush()?;

        Ok(())
    }
}

pub struct Reader<'a> {
    file: BufReader<&'a mut File>,
    table: Vec<u32>,
}
impl<'a> Reader<'a> {
    pub fn new(file: &'a mut File) -> anyhow::Result<Self> {
        file.seek(SeekFrom::Start(0))?;
        let mut file = BufReader::new(file);
        let table_offset = file.read_u32::<LittleEndian>()?;
        let table_len = file.read_u32::<LittleEndian>()? as usize;
        file.seek(SeekFrom::Start(table_offset as u64))?;

        let mut table = vec![0; table_len];
        for slot in table.iter_mut() {
            *slot = file.read_u32::<LittleEndian>()?;
        }

        Ok(Reader { file, table })
    }
    pub fn read(&mut self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let (h1, h2) = farmhash_fingerprint(key);

        let s1 = self.table[h1 as usize % self.table.len()];
        if s1 > 0 {
            if let Some(v) = self.try_read(key, s1)? {
                return Ok(Some(v));
            }
        }

        let s2 = self.table[h2 as usize % self.table.len()];
        if s2 > 0 {
            if let Some(v) = self.try_read(key, s2)? {
                return Ok(Some(v));
            }
        }

        Ok(None)
    }
    fn try_read(&mut self, key: &[u8], offset: u32) -> anyhow::Result<Option<Vec<u8>>> {
        self.file.seek(SeekFrom::Start(offset as u64))?;

        let key_len = self.file.read_u32::<LittleEndian>()?;
        let mut key_buf = vec![0; key_len as usize];
        self.file.read_exact(&mut key_buf)?;

        if key_buf != key {
            return Ok(None);
        }

        let value_len = self.file.read_u32::<LittleEndian>()?;
        let mut value_buf = vec![0; value_len as usize];
        self.file.read_exact(&mut value_buf)?;
        Ok(Some(value_buf))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_test() -> anyhow::Result<()> {
        let mut file = tempfile::tempfile()?;

        {
            let mut w = Writer::new(&mut file)?;
            w.append(b"hello", b"world")?;
            w.finish()?;
        }

        let mut r = Reader::new(&mut file)?;
        assert_eq!(r.read(b"hello")?, Some(b"world".to_vec()));
        assert_eq!(r.read(b"foo")?, None);

        Ok(())
    }

    #[test]
    fn writer_fails_on_duplicate_keys() -> anyhow::Result<()> {
        let mut file = tempfile::tempfile()?;

        // Cuckoo hashing can handle a single unreconcilable collision, but once
        // there are 3 duplicate items it will explode.
        let mut w = Writer::new(&mut file)?;
        w.append(b"hello", b"a")?;
        w.append(b"hello", b"b")?;
        w.append(b"hello", b"c")?;
        assert!(w.finish().is_err());
        Ok(())
    }

    #[test]
    fn writer_smoke_test() -> anyhow::Result<()> {
        let mut file = tempfile::tempfile()?;

        let mut w = Writer::new(&mut file)?;
        for i in 0..1_000_000 {
            w.append(
                format!("key-{}", i).as_bytes(),
                format!("value-{}", i).as_bytes(),
            )?;
        }
        w.finish()?;
        Ok(())
    }
}
