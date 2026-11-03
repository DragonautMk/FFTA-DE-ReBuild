//! File-derived typed controls and tile words for original transfer readers.
//! Live animation flags, countdowns, cursors and partial DMA remain guest state.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransferStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub control_bytes: u64,
    pub tile_bytes: u64,
}

struct Binding {
    offset: usize,
    length: usize,
    item: usize,
    tiles: bool,
}

pub(super) struct TransferResources {
    transfers: bg_transfers::Transfers,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    control_bytes: AtomicU64,
    tile_bytes: AtomicU64,
}

impl TransferResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x778..=0x897)
    }

    pub(super) fn new(
        transfers: bg_transfers::Transfers,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        transfers.validate()?;
        if sources.len() != transfers.controls.len() + transfers.tiles.len() {
            return Err("Transfer source count mismatch".into());
        }
        let mut bindings = Vec::new();
        let mut ranges = sources.iter();
        for (tiles, rows) in [
            (
                false,
                transfers
                    .controls
                    .iter()
                    .map(|c| c.source_bytes())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            (
                true,
                transfers.tiles.iter().map(|t| t.source_bytes()).collect(),
            ),
        ] {
            for (item, bytes) in rows.into_iter().enumerate() {
                let source = ranges.next().ok_or("Missing transfer binding")?;
                let kind = if tiles { "tiles" } else { "control" };
                if source.length as usize != bytes.len()
                    || source.sha256 != ffta_rom::digest(&bytes)
                    || source.role != format!("bg-transfer-{kind}/{item}")
                {
                    return Err("Transfer source/file binding mismatch".into());
                }
                bindings.push(Binding {
                    offset: source.offset as usize,
                    length: bytes.len(),
                    item,
                    tiles,
                });
            }
        }
        bindings.sort_by_key(|b| b.offset);
        for (index, binding) in bindings.iter().enumerate() {
            let end = binding
                .offset
                .checked_add(binding.length)
                .ok_or("Transfer binding overflow")?;
            if binding.length == 0
                || end > 0x0200_0000
                || !Self::source_page(binding.offset)
                || !Self::source_page(end - 1)
                || bindings
                    .get(index + 1)
                    .is_some_and(|next| end > next.offset)
            {
                return Err("Invalid or overlapping transfer bindings".into());
            }
        }
        let last = bindings.last().ok_or("Missing transfer bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            transfers,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            control_bytes: AtomicU64::new(0),
            tile_bytes: AtomicU64::new(0),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, bool)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        let byte = if binding.tiles {
            self.transfers.tiles[binding.item].words[at / 4].to_le_bytes()[at % 4]
        } else {
            let control = &self.transfers.controls[binding.item];
            match at {
                0 | 1 => control.transfer_bytes.to_le_bytes()[at],
                2 => control.frames.len() as u8,
                3 => control.header_byte3,
                _ => {
                    let at = at - 4;
                    let frame = &control.frames[at / 4];
                    if at % 4 < 2 {
                        frame.duration_ticks.to_le_bytes()[at % 2]
                    } else {
                        frame.source_tile.to_le_bytes()[at % 2]
                    }
                }
            }
        };
        Some((byte, binding.tiles))
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= self.end || index.saturating_add(N) <= self.bindings[0].offset {
            return prior;
        }
        let (mut controls, mut tiles) = (0, 0);
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, tile)) => {
                if tile {
                    tiles += 1;
                } else {
                    controls += 1;
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if controls + tiles == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes
            .fetch_add(controls + tiles, Ordering::Relaxed);
        self.control_bytes.fetch_add(controls, Ordering::Relaxed);
        self.tile_bytes.fetch_add(tiles, Ordering::Relaxed);
        Some(bytes)
    }

    pub(super) fn stats(&self) -> TransferStats {
        TransferStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            control_bytes: self.control_bytes.load(Ordering::Relaxed),
            tile_bytes: self.tile_bytes.load(Ordering::Relaxed),
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.transfers.descriptor_links.capacity()
                * std::mem::size_of::<[Option<bg_transfers::Link>; 5]>()
            + self.transfers.variant_sources.capacity() * std::mem::size_of::<Option<u32>>()
            + self.transfers.controls.capacity() * std::mem::size_of::<bg_transfers::Control>()
            + self
                .transfers
                .controls
                .iter()
                .map(|c| c.frames.capacity() * std::mem::size_of::<bg_transfers::Frame>())
                .sum::<usize>()
            + self.transfers.tiles.capacity() * std::mem::size_of::<bg_transfers::Tiles>()
            + self
                .transfers
                .tiles
                .iter()
                .map(|t| t.words.capacity() * 4)
                .sum::<usize>()
    }
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
