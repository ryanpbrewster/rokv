mod cuckoo;
pub mod mmap;
pub mod sync_read;

pub mod multihash;

pub(crate) fn farmhash_fingerprint(key: &[u8]) -> (u32, u32) {
    let h = farmhash::fingerprint64(key);
    let lo = (h & 0xFFFFFFFF) as u32;
    let hi = (h >> 32) as u32;
    (lo, hi)
}
