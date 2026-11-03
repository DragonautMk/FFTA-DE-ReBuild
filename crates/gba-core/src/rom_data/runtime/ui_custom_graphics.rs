//! Custom UI graphics reads use decoded words for literals and exact codec metadata.
//! Original decompression, scratch writes and staged DMA keep executing normally.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiGraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub decoded_literal_bytes: u64,
    pub selector_bytes: u64,
    pub localized_source_bytes: u64,
    pub localized_literal_bytes: u64,
}

/// Usage of the eight additional raw-language tables and their forty banks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableGraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub decoded_literal_bytes: u64,
    pub selector_bytes: u64,
    pub banks_used: u64,
    pub groups_used: u64,
}

#[derive(Clone, Copy)]
enum Kind {
    Bank(usize),
    Pointer(usize, usize),
}
struct Binding {
    offset: usize,
    length: usize,
    kind: Kind,
    table_bank: Option<usize>,
    table_group: Option<usize>,
}

pub(super) struct UiGraphicsResources {
    graphics: ui_graphics::Graphics,
    /// Exact encoded-source position -> decoded word byte; MAX is metadata.
    literals: Vec<Vec<u16>>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    reads: Vec<AtomicU64>,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    decoded_literal_bytes: AtomicU64,
    selector_bytes: AtomicU64,
    localized_source_bytes: AtomicU64,
    localized_literal_bytes: AtomicU64,
    table_reads: AtomicU64,
    table_bytes: AtomicU64,
    table_literals: AtomicU64,
    table_selectors: AtomicU64,
    table_banks: AtomicU64,
    table_groups: AtomicU64,
}

impl UiGraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x8e | 0x143 | 0x3ba | 0x43b..=0x43c | 0x44d | 0x45e..=0x45f | 0x46f..=0x470 | 0x481 | 0xb6b..=0xb75 | 0xb7a..=0xb7f)
            || TABLE_SOURCE_PAGES.contains(&(index >> 12))
    }

    pub(super) fn new(
        graphics: ui_graphics::Graphics,
        sources: &[SourceRange],
        shared_obj: &obj_graphics::Graphics,
        file_sha256: String,
    ) -> Result<Self, String> {
        graphics.validate()?;
        graphics.validate_shared_obj(shared_obj)?;
        if sources.len() != graphics.banks.len() + graphics.selectors.len() * 5 {
            return Err("Custom UI graphics source count mismatch".into());
        }
        let table_selectors = ui_graphics::table_selected::ROOTS
            .iter()
            .map(|root| {
                graphics
                    .selectors
                    .iter()
                    .position(|s| s.id == format!("table-{root:06x}"))
                    .ok_or("Missing table-selected UI graphics")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut table_banks: Vec<_> = table_selectors
            .iter()
            .enumerate()
            .flat_map(|(group, &selector)| {
                graphics.selectors[selector]
                    .pointers
                    .iter()
                    .map(move |p| (p.bank_offset, group))
            })
            .collect();
        table_banks.sort_unstable();
        let mut literals = Vec::new();
        let mut bindings = Vec::new();
        for (row, (graphic, source)) in graphics.banks.iter().zip(sources).enumerate() {
            if source.offset != graphic.source_offset
                || source.length as usize != graphic.encoded_source.len()
                || ffta_rom::digest(&graphic.encoded_source) != source.sha256
                || source.role != format!("ui-custom-bank/{:08x}", graphic.source_offset)
            {
                return Err("Custom UI graphics source/file binding mismatch".into());
            }
            let block = custom_lz::decode(
                &graphic.encoded_source,
                obj_graphics::MAX_OUTPUT,
                obj_graphics::MAX_ENCODED,
            )?;
            let mut positions = vec![u16::MAX; graphic.encoded_source.len()];
            for span in block.literal_spans {
                for index in 0..span.length {
                    positions[span.encoded_offset + index] = (span.output_offset + index) as u16;
                }
            }
            literals.push(positions);
            bindings.push(Binding {
                offset: source.offset as usize,
                length: source.length as usize,
                kind: Kind::Bank(row),
                table_bank: table_banks
                    .binary_search_by_key(&source.offset, |p| p.0)
                    .ok(),
                table_group: table_banks
                    .binary_search_by_key(&source.offset, |p| p.0)
                    .ok()
                    .map(|i| table_banks[i].1),
            });
        }
        for (group, selector) in graphics.selectors.iter().enumerate() {
            for (language, pointer) in selector.pointers.iter().enumerate() {
                let source = &sources[graphics.banks.len() + group * 5 + language];
                if source.offset != pointer.source_offset
                    || source.length != 4
                    || source.sha256 != ffta_rom::digest(&pointer.source_bytes()?)
                    || source.role != format!("ui-custom-selector/{}/{language}", selector.id)
                {
                    return Err("Custom UI selector source/file binding mismatch".into());
                }
                bindings.push(Binding {
                    offset: pointer.source_offset as usize,
                    length: 4,
                    kind: Kind::Pointer(group, language),
                    table_bank: None,
                    table_group: table_selectors.iter().position(|&i| i == group),
                });
            }
        }
        bindings.sort_by_key(|binding| binding.offset);
        for (index, binding) in bindings.iter().enumerate() {
            let end = binding
                .offset
                .checked_add(binding.length)
                .ok_or("Custom UI graphics binding overflow")?;
            if binding.length == 0
                || end > 0x0200_0000
                || !Self::source_page(binding.offset)
                || !Self::source_page(end - 1)
                || bindings
                    .get(index + 1)
                    .is_some_and(|next| end > next.offset)
            {
                return Err("Invalid or overlapping custom UI graphics bindings".into());
            }
        }
        let last = bindings
            .last()
            .ok_or("Missing custom UI graphics bindings")?;
        let end = last.offset + last.length;
        let reads = graphics.banks.iter().map(|_| AtomicU64::new(0)).collect();
        Ok(Self {
            reads,
            graphics,
            literals,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            decoded_literal_bytes: AtomicU64::new(0),
            selector_bytes: AtomicU64::new(0),
            localized_source_bytes: AtomicU64::new(0),
            localized_literal_bytes: AtomicU64::new(0),
            table_reads: AtomicU64::new(0),
            table_bytes: AtomicU64::new(0),
            table_literals: AtomicU64::new(0),
            table_selectors: AtomicU64::new(0),
            table_banks: AtomicU64::new(0),
            table_groups: AtomicU64::new(0),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, bool, bool, bool, Option<usize>, Option<usize>)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        let (byte, literal, selector, localized) = match binding.kind {
            Kind::Pointer(group, language) => (
                (0x08000000 + self.graphics.selectors[group].pointers[language].bank_offset)
                    .to_le_bytes()[at],
                false,
                true,
                false,
            ),
            Kind::Bank(row) => {
                self.reads[row].fetch_add(1, Ordering::Relaxed);
                let graphic = &self.graphics.banks[row];
                let output = self.literals[row][at];
                let localized = ui_graphics::LOCALIZED_BANK_ROOTS
                    .binary_search(&graphic.source_offset)
                    .is_ok();
                if output == u16::MAX {
                    (graphic.encoded_source[at], false, false, localized)
                } else {
                    let output = output as usize;
                    (
                        graphic.words[output / 4].to_le_bytes()[output % 4],
                        true,
                        false,
                        localized,
                    )
                }
            }
        };
        Some((
            byte,
            literal,
            selector,
            localized,
            binding.table_bank,
            binding.table_group,
        ))
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
        let (mut mapped, mut literals, mut selectors, mut localized, mut localized_literals) =
            (0, 0, 0, 0, 0);
        let (
            mut table_bytes,
            mut table_literals,
            mut table_selectors,
            mut table_banks,
            mut table_groups,
        ) = (0, 0, 0, 0u64, 0u64);
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, literal, selector, local, table_bank, table_group)) => {
                mapped += 1;
                literals += u64::from(literal);
                selectors += u64::from(selector);
                localized += u64::from(local);
                localized_literals += u64::from(local && literal);
                if let Some(group) = table_group {
                    table_bytes += 1;
                    table_literals += u64::from(literal);
                    table_selectors += u64::from(selector);
                    table_groups |= 1 << group;
                }
                if let Some(bank) = table_bank {
                    table_banks |= 1 << bank;
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if mapped == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        self.decoded_literal_bytes
            .fetch_add(literals, Ordering::Relaxed);
        self.selector_bytes.fetch_add(selectors, Ordering::Relaxed);
        self.localized_source_bytes
            .fetch_add(localized, Ordering::Relaxed);
        self.localized_literal_bytes
            .fetch_add(localized_literals, Ordering::Relaxed);
        if table_bytes != 0 {
            self.table_reads.fetch_add(1, Ordering::Relaxed);
            self.table_bytes.fetch_add(table_bytes, Ordering::Relaxed);
            self.table_literals
                .fetch_add(table_literals, Ordering::Relaxed);
            self.table_selectors
                .fetch_add(table_selectors, Ordering::Relaxed);
            self.table_banks.fetch_or(table_banks, Ordering::Relaxed);
            self.table_groups.fetch_or(table_groups, Ordering::Relaxed);
        }
        Some(bytes)
    }

    pub(super) fn stats(&self) -> UiGraphicsStats {
        UiGraphicsStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            decoded_literal_bytes: self.decoded_literal_bytes.load(Ordering::Relaxed),
            selector_bytes: self.selector_bytes.load(Ordering::Relaxed),
            localized_source_bytes: self.localized_source_bytes.load(Ordering::Relaxed),
            localized_literal_bytes: self.localized_literal_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn table_stats(&self) -> TableGraphicsStats {
        TableGraphicsStats {
            bus_reads: self.table_reads.load(Ordering::Relaxed),
            source_bytes: self.table_bytes.load(Ordering::Relaxed),
            decoded_literal_bytes: self.table_literals.load(Ordering::Relaxed),
            selector_bytes: self.table_selectors.load(Ordering::Relaxed),
            banks_used: u64::from(self.table_banks.load(Ordering::Relaxed).count_ones()),
            groups_used: u64::from(self.table_groups.load(Ordering::Relaxed).count_ones()),
        }
    }

    pub(super) fn used_sources(&self) -> Vec<(u32, u64)> {
        self.graphics
            .banks
            .iter()
            .zip(&self.reads)
            .filter_map(|(bank, n)| {
                let n = n.load(Ordering::Relaxed);
                (n > 0).then_some((bank.source_offset, n))
            })
            .collect()
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.graphics.banks.capacity() * std::mem::size_of::<ui_graphics::Graphic>()
            + self
                .graphics
                .banks
                .iter()
                .map(|g| g.words.capacity() * 4 + g.encoded_source.capacity())
                .sum::<usize>()
            + self.graphics.selectors.capacity() * std::mem::size_of::<ui_graphics::Selector>()
            + self
                .graphics
                .selectors
                .iter()
                .map(|s| s.id.capacity())
                .sum::<usize>()
            + self.graphics.shared_obj_banks.capacity() * 4
            + self.reads.capacity() * std::mem::size_of::<AtomicU64>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.literals.capacity() * std::mem::size_of::<Vec<u16>>()
            + self
                .literals
                .iter()
                .map(|positions| positions.capacity() * 2)
                .sum::<usize>()
    }
}

// Page filter only; immutable bindings determine exact byte ownership.
const TABLE_SOURCE_PAGES: &[usize] = &[
    0x431, 0x436, 0x437, 0x43c, 0x440, 0x442, 0x447, 0x448, 0x44e, 0x451, 0x453, 0x454, 0x459,
    0x45f, 0x463, 0x465, 0x46a, 0x46b, 0x470, 0x474, 0x476, 0x47b, 0x47c, 0x482, 0x485, 0x486,
    0x71e, 0xb67, 0xb68, 0xb69, 0xb6a, 0xb6b,
];
