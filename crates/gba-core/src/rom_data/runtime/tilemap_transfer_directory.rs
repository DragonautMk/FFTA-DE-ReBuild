//! Managed control reads and immutable LZ77 inputs from the authenticated ROM.
use super::super::{
    script39_registry_poses::PriorSources,
    tilemap_transfer_directory::{Field, Role, TilemapTransferDirectory},
    Resource, SourceRange,
};
use super::RuntimeResources;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapTransferDirectoryStats {
    pub bus_reads: u64,
    pub pointer_bytes: u64,
    pub count_bytes: u64,
    pub row_bytes: u64,
    pub descriptor_bytes: u64,
    pub wrapper_bytes: u64,
    pub bound_lz77_bus_bytes: u64,
    pub bound_lz77_calls: u64,
    pub bound_lz77_input_bytes: u64,
}

enum SourceByte {
    Control(u8, Role),
    Lz77(u8),
}

struct BoundInput {
    offset: u32,
    encoded: Box<[u8]>,
}

pub struct TilemapLz77Input {
    owner: Arc<RuntimeResources>,
    index: usize,
}

impl TilemapLz77Input {
    pub fn bytes(&self) -> &[u8] {
        &self
            .owner
            .tilemap_transfer_directory
            .as_ref()
            .unwrap()
            .inputs[self.index]
            .encoded
    }
}

pub(super) struct TilemapTransferDirectoryResources {
    fields: Vec<Field>,
    reads: AtomicU64,
    bytes: [AtomicU64; 5],
    inputs: Vec<BoundInput>,
    lz77_bus_bytes: AtomicU64,
    lz77_calls: AtomicU64,
    lz77_bytes: AtomicU64,
}

impl TilemapTransferDirectoryResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| {
            matches!(
                page,
                0x39a | 0xafa | 0xb0d | 0xb0e | 0xb1c | 0xb20 | 0xb27 | 0xb31 | 0xb3c
            )
        })
    }

    pub(super) fn new(
        data: &TilemapTransferDirectory,
        sources: &[SourceRange],
        rom: &[u8],
        existing: &[Resource],
    ) -> Result<Self, String> {
        let mut expected = sources.iter().peekable();
        let mut fields = Vec::with_capacity(sources.len());
        for field in data.fields()? {
            if let Some(source) = expected.next_if(|source| source.offset == field.offset) {
                if field.source_range() != *source || !Self::source_page(field.offset as usize) {
                    return Err("Tilemap transfer directory source binding mismatch".into());
                }
                fields.push(field);
            }
        }
        if expected.peek().is_some() {
            return Err("Missing tilemap transfer directory source field".into());
        }
        let prior = PriorSources::new(existing)?;
        let mut inputs = Vec::new();
        for input in data.lz77_input_ranges(rom)? {
            let encoded = &rom[input.start as usize..input.end as usize];
            let owned = prior
                .contains(input.start, encoded.len())
                .map_err(|error| {
                    format!("Tilemap LZ77 ownership at {:#x}: {error}", input.start)
                })?;
            if !owned {
                let first_page = input.start as usize & !0xfff;
                if (first_page..input.end as usize)
                    .step_by(0x1000)
                    .any(|page| !Self::source_page(page))
                {
                    return Err(format!(
                        "Tilemap LZ77 input outside the source pages at {:#x}",
                        input.start
                    ));
                }
                inputs.push(BoundInput {
                    offset: input.start,
                    encoded: encoded.into(),
                });
            }
        }
        Ok(Self {
            fields,
            reads: AtomicU64::new(0),
            bytes: std::array::from_fn(|_| AtomicU64::new(0)),
            inputs,
            lz77_bus_bytes: AtomicU64::new(0),
            lz77_calls: AtomicU64::new(0),
            lz77_bytes: AtomicU64::new(0),
        })
    }

    fn control_byte_at(&self, offset: usize) -> Option<(u8, Role)> {
        let i = self
            .fields
            .partition_point(|field| field.offset as usize <= offset);
        let field = self.fields.get(i.checked_sub(1)?)?;
        Some((
            *field.bytes.get(offset - field.offset as usize)?,
            field.role,
        ))
    }

    fn source_byte_at(&self, offset: usize) -> Option<SourceByte> {
        if let Some((byte, role)) = self.control_byte_at(offset) {
            return Some(SourceByte::Control(byte, role));
        }
        let i = self
            .inputs
            .partition_point(|input| input.offset as usize <= offset);
        let input = self.inputs.get(i.checked_sub(1)?)?;
        let byte = *input.encoded.get(offset - input.offset as usize)?;
        Some(SourceByte::Lz77(byte))
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.source_byte_at(index + i));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let result = std::array::from_fn(|i| match cells[i] {
            Some(SourceByte::Control(byte, role)) => {
                self.bytes[role as usize].fetch_add(1, Ordering::Relaxed);
                byte
            }
            Some(SourceByte::Lz77(byte)) => {
                self.lz77_bus_bytes.fetch_add(1, Ordering::Relaxed);
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> TilemapTransferDirectoryStats {
        let bytes = |role: Role| self.bytes[role as usize].load(Ordering::Relaxed);
        TilemapTransferDirectoryStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            pointer_bytes: bytes(Role::ListPointer),
            count_bytes: bytes(Role::ListCount),
            row_bytes: bytes(Role::Row),
            descriptor_bytes: bytes(Role::Descriptor),
            wrapper_bytes: bytes(Role::Wrapper),
            bound_lz77_bus_bytes: self.lz77_bus_bytes.load(Ordering::Relaxed),
            bound_lz77_calls: self.lz77_calls.load(Ordering::Relaxed),
            bound_lz77_input_bytes: self.lz77_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.fields.capacity() * std::mem::size_of::<Field>()
            + self
                .fields
                .iter()
                .map(|field| field.bytes.capacity())
                .sum::<usize>()
            + self.inputs.capacity() * std::mem::size_of::<BoundInput>()
            + self
                .inputs
                .iter()
                .map(|input| input.encoded.len())
                .sum::<usize>()
    }
}

impl RuntimeResources {
    pub(crate) fn bound_tilemap_lz77(self: &Arc<Self>, source: u32) -> Option<TilemapLz77Input> {
        if !matches!(source >> 24, 0x08..=0x0d) {
            return None;
        }
        let directory = self.tilemap_transfer_directory.as_ref()?;
        let offset = source & 0x01ff_ffff;
        let index = directory
            .inputs
            .binary_search_by_key(&offset, |input| input.offset)
            .ok()?;
        let input = &directory.inputs[index];
        directory.lz77_calls.fetch_add(1, Ordering::Relaxed);
        directory
            .lz77_bytes
            .fetch_add(input.encoded.len() as u64, Ordering::Relaxed);
        Some(TilemapLz77Input {
            owner: self.clone(),
            index,
        })
    }
}
