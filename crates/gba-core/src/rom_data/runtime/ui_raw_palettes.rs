//! Raw typed colors feed original copies; live palette RAM/fades remain mutable.
use super::*;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiPaletteStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
}
pub(super) struct UiPaletteResources {
    palettes: ui_palettes::Palettes,
    file_sha256: String,
    reads: Vec<AtomicU64>,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
}
impl UiPaletteResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index>>12,0x40f..=0x411|0xb4b|0xb8b..=0xb8c)
    }
    pub(super) fn new(
        palettes: ui_palettes::Palettes,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        palettes.validate()?;
        if palettes.banks.len() != sources.len() {
            return Err("UI palette source count mismatch".into());
        }
        for (b, s) in palettes.banks.iter().zip(sources) {
            let end = b.source_offset as usize + b.colors.len() * 2;
            if b.source_offset != s.offset
                || b.colors.len() * 2 != s.length as usize
                || ffta_rom::digest(&b.source_bytes()) != s.sha256
                || s.role != format!("ui-raw-palette/{:08x}", b.source_offset)
                || !Self::source_page(b.source_offset as usize)
                || !Self::source_page(end - 1)
            {
                return Err("UI palette source/file binding mismatch".into());
            }
        }
        let reads = palettes.banks.iter().map(|_| AtomicU64::new(0)).collect();
        Ok(Self {
            palettes,
            file_sha256,
            reads,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let row = self
            .palettes
            .banks
            .partition_point(|b| b.source_offset as usize <= index)
            .checked_sub(1)?;
        let bank = &self.palettes.banks[row];
        let at = index - bank.source_offset as usize;
        Some((bank.colors.get(at / 2)?.to_le_bytes()[at % 2], row))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        let mut mapped = 0;
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, row)) => {
                mapped += 1;
                self.reads[row].fetch_add(1, Ordering::Relaxed);
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |v| v[i]),
        });
        if mapped == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> UiPaletteStats {
        UiPaletteStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
        }
    }
    pub(super) fn source_bytes(&self, offset: u32) -> Option<u64> {
        let row = self
            .palettes
            .banks
            .binary_search_by_key(&offset, |b| b.source_offset)
            .ok()?;
        Some(self.reads[row].load(Ordering::Relaxed))
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.palettes.banks.capacity() * std::mem::size_of::<ui_palettes::Bank>()
            + self
                .palettes
                .banks
                .iter()
                .map(|b| b.colors.capacity() * 2)
                .sum::<usize>()
            + self.reads.capacity() * std::mem::size_of::<AtomicU64>()
    }
}
