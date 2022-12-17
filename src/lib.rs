pub mod sync_read;
pub mod mmap;

// A simple hash function copied from CDB
pub(crate) fn cdb_hash(key: &[u8]) -> u32 {
    let mut h: u32 = 5381;
    for &c in key {
        h = h.wrapping_mul(33) ^ c as u32;
    }
    h
}
