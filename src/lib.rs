//! Archive (.a / .lib) writer for static libraries

use std::io::{self, Write};

pub struct ArchiveEntry {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct ArchiveWriter<W: Write> {
    writer: W,
    entries: Vec<ArchiveEntry>,
}

impl<W: Write> ArchiveWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, entries: Vec::new() }
    }

    pub fn add_entry(&mut self, name: &str, data: Vec<u8>) {
        self.entries.push(ArchiveEntry { name: name.to_string(), data });
    }

    pub fn finish(mut self) -> io::Result<()> {
        self.writer.write_all(b"!<arch>\n")?;
        for entry in &self.entries {
            let size = entry.data.len();
            write!(self.writer, "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
                entry.name, 0u32, 0u16, 0u16, 0o100644, size)?;
            self.writer.write_all(&entry.data)?;
            if size % 2 != 0 {
                self.writer.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut buf = Vec::new();
        let mut w = ArchiveWriter::new(&mut buf);
        w.add_entry("test.o", vec![1, 2, 3]);
        w.finish().unwrap();
        assert!(buf.starts_with(b"!<arch>\n"));
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
