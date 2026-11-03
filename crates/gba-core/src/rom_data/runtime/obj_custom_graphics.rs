//! Typed custom OBJ sources. Original decode, RAM transforms and uploads stay live.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjGraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub decoded_literal_bytes: u64,
    pub selector_bytes: u64,
}

#[derive(Clone, Copy)]
enum Kind {
    Bank(usize),
    Table(usize),
}
struct Binding {
    offset: usize,
    length: usize,
    kind: Kind,
}
pub(super) struct ObjGraphicsResources {
    graphics: obj_graphics::Graphics,
    literals: Vec<Vec<u16>>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    decoded_literal_bytes: AtomicU64,
    selector_bytes: AtomicU64,
}
impl ObjGraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x395 | 0x397 | 0x3b0..=0x3b1 | 0x3b9 | 0x3ba | 0x40e..=0x40f
            | 0x431..=0x433 | 0x43a..=0x43b | 0x43c | 0x442..=0x444 | 0x44c..=0x44e
            | 0x454..=0x455 | 0x45d..=0x45f | 0x465..=0x467 | 0x46e..=0x470
            | 0x476..=0x478 | 0x480 | 0x482 | 0xb7f..=0xb80)
    }
    pub(super) fn new(
        graphics: obj_graphics::Graphics,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        graphics.validate()?;
        if sources.len() != graphics.banks.len() + graphics.pointer_tables.len() {
            return Err("OBJ graphics source count mismatch".into());
        }
        let mut literals = Vec::new();
        let mut bindings = Vec::new();
        for (row, bank) in graphics.banks.iter().enumerate() {
            let block = custom_lz::decode(
                &bank.encoded_source,
                obj_graphics::MAX_OUTPUT,
                obj_graphics::MAX_ENCODED,
            )?;
            let mut positions = vec![u16::MAX; bank.encoded_source.len()];
            for span in block.literal_spans {
                for i in 0..span.length {
                    positions[span.encoded_offset + i] = (span.output_offset + i) as u16;
                }
            }
            literals.push(positions);
            bindings.push(Binding {
                offset: bank.source_offset as usize,
                length: bank.encoded_source.len(),
                kind: Kind::Bank(row),
            });
        }
        for (row, table) in graphics.pointer_tables.iter().enumerate() {
            bindings.push(Binding {
                offset: table.source_offset as usize,
                length: table.bank_offsets.len() * 4,
                kind: Kind::Table(row),
            });
        }
        // Validate exact provenance in catalog order before sorting the bus view.
        for (binding, source) in bindings.iter().zip(sources) {
            let (bytes, role) = match binding.kind {
                Kind::Bank(row) => (
                    graphics.banks[row].encoded_source.clone(),
                    format!("obj-custom-bank/{:08x}", binding.offset),
                ),
                Kind::Table(row) => (
                    graphics.pointer_tables[row].source_bytes()?,
                    format!("obj-graphics-selector/{:08x}", binding.offset),
                ),
            };
            if source.offset as usize != binding.offset
                || source.length as usize != binding.length
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("OBJ graphics source/file binding mismatch".into());
            }
        }
        bindings.sort_by_key(|b| b.offset);
        for (i, b) in bindings.iter().enumerate() {
            let end = b
                .offset
                .checked_add(b.length)
                .ok_or("OBJ graphics binding overflow")?;
            if b.length == 0
                || end > 0x02000000
                || !Self::source_page(b.offset)
                || !Self::source_page(end - 1)
                || bindings.get(i + 1).is_some_and(|next| end > next.offset)
            {
                return Err("Invalid OBJ graphics binding extent".into());
            }
        }
        let last = bindings.last().ok_or("Missing OBJ graphics bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            graphics,
            literals,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            decoded_literal_bytes: AtomicU64::new(0),
            selector_bytes: AtomicU64::new(0),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, bool, bool)> {
        let b = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - b.offset;
        if at >= b.length {
            return None;
        }
        match b.kind {
            Kind::Table(row) => Some((
                (0x08000000 + self.graphics.pointer_tables[row].bank_offsets[at / 4]).to_le_bytes()
                    [at % 4],
                false,
                true,
            )),
            Kind::Bank(row) => {
                let output = self.literals[row][at];
                let bank = &self.graphics.banks[row];
                if output == u16::MAX {
                    Some((bank.encoded_source[at], false, false))
                } else {
                    let output = output as usize;
                    Some((
                        bank.words[output / 4].to_le_bytes()[output % 4],
                        true,
                        false,
                    ))
                }
            }
        }
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
        let (mut mapped, mut literals, mut selectors) = (0, 0, 0);
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, literal, selector)) => {
                mapped += 1;
                literals += u64::from(literal);
                selectors += u64::from(selector);
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |v| v[i]),
        });
        if mapped == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        self.decoded_literal_bytes
            .fetch_add(literals, Ordering::Relaxed);
        self.selector_bytes.fetch_add(selectors, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ObjGraphicsStats {
        ObjGraphicsStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            decoded_literal_bytes: self.decoded_literal_bytes.load(Ordering::Relaxed),
            selector_bytes: self.selector_bytes.load(Ordering::Relaxed),
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.graphics.banks.capacity() * std::mem::size_of::<obj_graphics::Graphic>()
            + self
                .graphics
                .banks
                .iter()
                .map(|b| b.words.capacity() * 4 + b.encoded_source.capacity())
                .sum::<usize>()
            + self.graphics.pointer_tables.capacity()
                * std::mem::size_of::<obj_graphics::PointerTable>()
            + self
                .graphics
                .pointer_tables
                .iter()
                .map(|t| t.bank_offsets.capacity() * 4)
                .sum::<usize>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.literals.capacity() * std::mem::size_of::<Vec<u16>>()
            + self
                .literals
                .iter()
                .map(|v| v.capacity() * 2)
                .sum::<usize>()
    }
}
