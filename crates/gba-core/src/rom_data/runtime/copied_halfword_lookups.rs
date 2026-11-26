//! Serve original copied halfword lookup bytes and their second-table roots.
use super::*;
use crate::rom_data::copied_halfword_lookups as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopiedHalfwordLookupsStats {
    pub bus_reads: u64,
    pub table_bytes: u64,
    pub root_bytes: u64,
    pub table_mask: u8,
    pub root_mask: u8,
    pub soil_source_mask: u8,
    pub signed_projection_bytes: u64,
    pub signed_projection_regions_mask: u8,
    pub star_cross_bytes: u64,
    pub star_cross_regions_mask: u8,
    pub initializer_bytes: u64,
    pub initializer_mask: u16,
    pub variant_bytes: u64,
    pub variant_mask: u16,
    pub pair_bytes: u64,
    pub pair_sources_mask: u16,
    pub pair_roots_mask: u16,
}

pub(super) struct CopiedHalfwordLookupsResources {
    data: model::CopiedHalfwordLookups,
    reads: AtomicU64,
    table_bytes: AtomicU64,
    root_bytes: AtomicU64,
    tables: AtomicU64,
    roots: AtomicU64,
    soil_source: AtomicU64,
    signed_projection_bytes: AtomicU64,
    signed_projection_regions: AtomicU64,
    star_cross_bytes: AtomicU64,
    star_cross_regions: AtomicU64,
    initializer_bytes: AtomicU64,
    initializers: AtomicU64,
    variant_bytes: AtomicU64,
    variants: AtomicU64,
    pair_bytes: AtomicU64,
    pair_sources: AtomicU64,
    pair_roots: AtomicU64,
}
impl CopiedHalfwordLookupsResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        (first..=last).any(|page| matches!(page, 0x3 | 0xc | 0xd | 0x1e | 0x8f | 0xd8 | 0xdc | 0xdd | 0xde | 0xe5 | 0xe7 | 0xfa | 0xfb | 0x101 | 0x103 | 0x106 | 0x109 | 0x10d | 0x10e | 0x11d | 0x11f | 0x122 | 0x124 | 0x13f | 0x14c | 0x14d))
    }

    pub(super) fn new(
        data: model::CopiedHalfwordLookups,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len()
            || spans
                .iter()
                .zip(sources)
                .any(|((offset, bytes, role), source)| {
                    source.offset != *offset
                        || source.length as usize != bytes.len()
                        || source.role != *role
                        || source.sha256 != ffta_rom::digest(bytes)
                })
        {
            return Err("Copied halfword lookup source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            table_bytes: AtomicU64::new(0),
            root_bytes: AtomicU64::new(0),
            tables: AtomicU64::new(0),
            roots: AtomicU64::new(0),
            soil_source: AtomicU64::new(0),
            signed_projection_bytes: AtomicU64::new(0),
            signed_projection_regions: AtomicU64::new(0),
            star_cross_bytes: AtomicU64::new(0),
            star_cross_regions: AtomicU64::new(0),
            initializer_bytes: AtomicU64::new(0),
            initializers: AtomicU64::new(0),
            variant_bytes: AtomicU64::new(0),
            variants: AtomicU64::new(0),
            pair_bytes: AtomicU64::new(0),
            pair_sources: AtomicU64::new(0),
            pair_roots: AtomicU64::new(0),
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let (mut table_bytes, mut root_bytes, mut tables, mut roots, mut soil_source) = (0u64, 0u64, 0u64, 0u64, 0u64);
        let (mut projection_bytes, mut projection_regions) = (0u64, 0u64);
        let (mut star_cross_bytes, mut star_cross_regions) = (0u64, 0u64);
        let (mut initializer_bytes, mut initializers) = (0u64, 0u64);
        let (mut variant_bytes, mut variants) = (0u64, 0u64);
        let (mut pair_bytes, mut pair_sources, mut pair_roots) = (0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry, root)) => {
                if self.data.initializer_bank.is_some() {
                    let at = (index + i) as u32;
                    for (n, &(_, literal, source, words, _)) in model::initializer_bank::SPECS.iter().enumerate() {
                        if (source..source + words as u32 * 4).contains(&at) || (literal..literal + 4).contains(&at) {
                            initializer_bytes += 1;
                            initializers |= 1 << n;
                            break;
                        }
                    }
                }
                if self.data.initializer_variants.is_some() {
                    let at = (index + i) as u32;
                    for (n, spec) in model::initializer_variants::SPECS.iter().enumerate() {
                        if (spec.added_offset..spec.added_offset + spec.added_words as u32 * 4).contains(&at)
                            || spec.owns_literal && (spec.literal..spec.literal + 4).contains(&at) {
                            variant_bytes += 1;
                            variants |= 1 << n;
                            break;
                        }
                    }
                }
                if self.data.initializer_pairs.is_some() {
                    let at = (index + i) as u32;
                    for (n, &(source, _)) in model::initializer_pairs::SOURCES.iter().enumerate() {
                        if (source..source + 8).contains(&at) {
                            pair_bytes += 1;
                            pair_sources |= 1 << n;
                            break;
                        }
                    }
                    for (n, reader) in model::initializer_pairs::READERS.iter().enumerate() {
                        if (reader.literal..reader.literal + 4).contains(&at) {
                            pair_bytes += 1;
                            pair_roots |= 1 << n;
                            break;
                        }
                    }
                }
                if root {
                    root_bytes += 1;
                    roots |= 1 << entry;
                } else {
                    table_bytes += 1;
                    tables |= 1 << entry;
                    if entry == 3 { soil_source |= 1 << (index + i - model::SOIL_EVIDENCE_OFFSET as usize); }
                    if entry == 4 {
                        projection_bytes += 1;
                        projection_regions |= if index + i < model::SIGNED_PROJECTION_SECOND_VIEW as usize { 1 } else { 2 };
                    }
                    if entry == 5 {
                        star_cross_bytes += 1;
                        let relative = index + i - model::STAR_CROSS_OFFSET as usize;
                        star_cross_regions |= if relative < 16 { 1 } else if relative < 32 { 2 } else if relative < 36 { 4 } else { 8 };
                    }
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.table_bytes.fetch_add(table_bytes, Ordering::Relaxed);
        self.root_bytes.fetch_add(root_bytes, Ordering::Relaxed);
        self.tables.fetch_or(tables, Ordering::Relaxed);
        self.roots.fetch_or(roots, Ordering::Relaxed);
        self.soil_source.fetch_or(soil_source, Ordering::Relaxed);
        self.signed_projection_bytes.fetch_add(projection_bytes, Ordering::Relaxed);
        self.signed_projection_regions.fetch_or(projection_regions, Ordering::Relaxed);
        self.star_cross_bytes.fetch_add(star_cross_bytes, Ordering::Relaxed);
        self.star_cross_regions.fetch_or(star_cross_regions, Ordering::Relaxed);
        self.initializer_bytes.fetch_add(initializer_bytes, Ordering::Relaxed);
        self.initializers.fetch_or(initializers, Ordering::Relaxed);
        self.variant_bytes.fetch_add(variant_bytes, Ordering::Relaxed);
        self.variants.fetch_or(variants, Ordering::Relaxed);
        self.pair_bytes.fetch_add(pair_bytes, Ordering::Relaxed);
        self.pair_sources.fetch_or(pair_sources, Ordering::Relaxed);
        self.pair_roots.fetch_or(pair_roots, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> CopiedHalfwordLookupsStats {
        CopiedHalfwordLookupsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            table_bytes: self.table_bytes.load(Ordering::Relaxed),
            root_bytes: self.root_bytes.load(Ordering::Relaxed),
            table_mask: self.tables.load(Ordering::Relaxed) as u8,
            root_mask: self.roots.load(Ordering::Relaxed) as u8,
            soil_source_mask: self.soil_source.load(Ordering::Relaxed) as u8,
            signed_projection_bytes: self.signed_projection_bytes.load(Ordering::Relaxed),
            signed_projection_regions_mask: self.signed_projection_regions.load(Ordering::Relaxed) as u8,
            star_cross_bytes: self.star_cross_bytes.load(Ordering::Relaxed),
            star_cross_regions_mask: self.star_cross_regions.load(Ordering::Relaxed) as u8,
            initializer_bytes: self.initializer_bytes.load(Ordering::Relaxed),
            initializer_mask: self.initializers.load(Ordering::Relaxed) as u16,
            variant_bytes: self.variant_bytes.load(Ordering::Relaxed),
            variant_mask: self.variants.load(Ordering::Relaxed) as u16,
            pair_bytes: self.pair_bytes.load(Ordering::Relaxed),
            pair_sources_mask: self.pair_sources.load(Ordering::Relaxed) as u16,
            pair_roots_mask: self.pair_roots.load(Ordering::Relaxed) as u16,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.signed_byte_index_projection.as_ref()
            .map_or(0, |projection| projection.values.capacity() * std::mem::size_of::<i16>())
            + self.data.initializer_bank.as_ref().map_or(0, |bank| {
                bank.records.capacity() * std::mem::size_of::<model::initializer_bank::Initializer>()
                    + bank.records.iter().map(|record| record.words.capacity() * std::mem::size_of::<u32>()).sum::<usize>()
            })
            + self.data.initializer_variants.as_ref().map_or(0, |bank| {
                bank.records.capacity() * std::mem::size_of::<model::initializer_variants::Variant>()
                    + bank.records.iter().map(|record| record.words.capacity() * std::mem::size_of::<u32>()).sum::<usize>()
            })
            + self.data.initializer_pairs.as_ref().map_or(0, |bank| {
                bank.pairs.capacity() * std::mem::size_of::<model::initializer_pairs::Pair>()
            })
    }
}
