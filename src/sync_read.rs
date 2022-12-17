use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::cdb_hash;

pub struct Writer<'a> {
    file: BufWriter<&'a mut File>,
    offset: u32,
    log: Vec<Entry>,
}
struct Entry {
    key_hash: u32,
    offset: u32,
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
            key_hash: cdb_hash(key),
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
        let mut table: Vec<u32> = vec![0; 2 * self.log.len()];
        for entry in self.log {
            let mut slot = entry.key_hash as usize % table.len();
            // In the case of collisions, perform linear probing to find an empty slot.
            while table[slot] > 0 {
                slot = (slot + 1) % table.len();
            }
            table[slot] = entry.offset;
        }

        // Write the hashtable.
        for &offset in &table {
            self.file.write_u32::<LittleEndian>(offset)?;
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
        let mut slot = cdb_hash(key) as usize % self.table.len();
        while self.table[slot] > 0 {
            self.file.seek(SeekFrom::Start(self.table[slot] as u64))?;

            let key_len = self.file.read_u32::<LittleEndian>()?;
            let mut key_buf = vec![0; key_len as usize];
            self.file.read_exact(&mut key_buf)?;

            if key_buf != key {
                slot = (slot + 1) % self.table.len();
                continue;
            }

            let value_len = self.file.read_u32::<LittleEndian>()?;
            let mut value_buf = vec![0; value_len as usize];
            self.file.read_exact(&mut value_buf)?;
            return Ok(Some(value_buf));
        }
        Ok(None)
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
}
