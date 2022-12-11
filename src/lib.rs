use std::{io::{self, Write, Seek, SeekFrom, Read, BufWriter}, fs::File};

use byteorder::ByteOrder;

pub struct Writer<'a> {
    file: BufWriter<&'a mut File>,
}
impl <'a> Writer<'a> {
    pub fn new(file: &'a mut File) -> Self {
        Writer { file: BufWriter::new(file) }
    }
    pub fn append(&mut self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
        let mut buf = [0; 4];

        byteorder::LittleEndian::write_u32(&mut buf, key.len() as u32);
        self.file.write_all(&buf)?;
        self.file.write_all(key)?;

        byteorder::LittleEndian::write_u32(&mut buf, value.len() as u32);
        self.file.write_all(&buf)?;
        self.file.write_all(value)?;

        Ok(())
    }
}

pub struct Reader<'a> {
    file: &'a mut File,
}
impl <'a> Reader<'a> {
    pub fn new(file: &'a mut File) -> Self {
        Reader { file }
    }
    pub fn read(&mut self, key: &[u8]) -> io::Result<Vec<u8>> {
        let mut buf = [0; 4];

        self.file.seek(SeekFrom::Start(0))?;
        loop {
            self.file.read_exact(&mut buf)?;
            let key_len = byteorder::LittleEndian::read_u32(&buf) as usize;
            let mut key_buf = vec![0; key_len];
            self.file.read_exact(&mut key_buf)?;

            self.file.read_exact(&mut buf)?;
            let value_len = byteorder::LittleEndian::read_u32(&buf) as usize;
            let mut value_buf = vec![0; value_len];
            self.file.read_exact(&mut value_buf)?;

            if key_buf == key {
                return Ok(value_buf);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_test() -> anyhow::Result<()> {
        let mut file = tempfile::tempfile()?;

        {
            let mut w = Writer::new(&mut file);
            w.append(b"hello", b"world")?;
        }

        let mut r = Reader::new(&mut file);
        assert_eq!(r.read(b"hello")?, b"world");
        assert!(r.read(b"foo").is_err(), "expected reading foo to error out");

        Ok(())
    }
}