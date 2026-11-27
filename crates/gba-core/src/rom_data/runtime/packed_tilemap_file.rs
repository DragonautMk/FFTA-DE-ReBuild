//! Guest views of authenticated tilemap transfer files.
use super::super::{tilemap_type3::LZ77_INPUT_ROLE, SourceRange};
use std::{
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

struct FileSource {
    offset: usize,
    file: Range<usize>,
    lz77: bool,
}

impl FileSource {
    fn end(&self) -> usize {
        self.offset + self.file.len()
    }
}

pub(super) struct PackedTilemapFile {
    file: Box<[u8]>,
    sources: Vec<FileSource>,
    bus_bytes: AtomicU64,
}

impl PackedTilemapFile {
    pub(super) fn new(file: &[u8], ranges: &[SourceRange]) -> Result<Self, String> {
        let mut sources = Vec::with_capacity(ranges.len());
        let mut cursor = 0usize;
        let mut previous_end = 0;
        for range in ranges {
            let offset = range.offset as usize;
            let length = range.length as usize;
            if length == 0 || offset < previous_end || cursor + length > file.len() {
                return Err("Invalid packed tilemap source layout".into());
            }
            let lz77 = range.role == LZ77_INPUT_ROLE;
            sources.push(FileSource {
                offset,
                file: cursor..cursor + length,
                lz77,
            });
            cursor += length;
            previous_end = offset + length;
        }
        if cursor != file.len() {
            return Err("Trailing bytes in packed tilemap transfer file".into());
        }
        Ok(Self {
            file: file.into(),
            sources,
            bus_bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn overlaps(&self, index: usize, width: usize) -> bool {
        let first = self.sources.partition_point(|source| source.end() <= index);
        self.sources
            .get(first)
            .is_some_and(|source| source.offset < index + width)
    }

    fn byte_at(&self, index: usize) -> Option<u8> {
        let first = self.sources.partition_point(|source| source.end() <= index);
        let source = self.sources.get(first)?;
        let relative = index.checked_sub(source.offset)?;
        Some(self.file[source.file.start + relative])
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, true)
    }

    pub(super) fn host_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, false)
    }

    fn read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
        count_guest_bytes: bool,
    ) -> Option<[u8; N]> {
        if !self.overlaps(index, N) {
            return prior;
        }
        let mut added = 0;
        let bytes = std::array::from_fn(|lane| {
            let address = index + lane;
            match self.byte_at(address) {
                Some(byte) => {
                    added += 1;
                    byte
                }
                None => prior
                    .as_ref()
                    .map_or_else(|| rom[address], |bytes| bytes[lane]),
            }
        });
        if count_guest_bytes {
            self.bus_bytes.fetch_add(added, Ordering::Relaxed);
        }
        Some(bytes)
    }

    pub(super) fn lz77_index(&self, offset: usize) -> Option<usize> {
        let index = self
            .sources
            .binary_search_by_key(&offset, |source| source.offset)
            .ok()?;
        self.sources[index].lz77.then_some(index)
    }

    pub(super) fn input_bytes(&self, index: usize) -> &[u8] {
        &self.file[self.sources[index].file.clone()]
    }

    pub(super) fn bus_bytes(&self) -> u64 {
        self.bus_bytes.load(Ordering::Relaxed)
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file.len()
            + self.sources.capacity() * std::mem::size_of::<FileSource>()
    }
}
