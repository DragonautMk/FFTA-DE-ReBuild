//! Original custom-codec reads backed by typed alternate banks and metadata.
//! Literal bytes come from decoded banks; header/token bytes retain their exact
//! authenticated representation. No guest decoder, instruction or write is skipped.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlternatePaletteStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub table_bytes: u64,
    pub decoded_literal_bytes: u64,
}

struct Binding {
    offset: usize,
    length: usize,
    /// 0/1 are link tables; 2.. are group-major palette indices.
    item: usize,
}

pub(super) struct AlternatePaletteResources {
    palettes: bg_alternate_palettes::Palettes,
    /// u16::MAX denotes an original control byte; all other entries select a
    /// byte of a decoded bank. Construct once with the shared verified codec.
    literals: Vec<Vec<u16>>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    table_bytes: AtomicU64,
    decoded_literal_bytes: AtomicU64,
}

impl AlternatePaletteResources {
    #[inline]
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x427..=0x431 | 0x489..=0x493)
    }

    pub(super) fn new(
        palettes: bg_alternate_palettes::Palettes,
        catalog: &bg_alternate_palettes::Catalog,
        file_sha256: String,
    ) -> Result<Self, String> {
        palettes.validate()?;
        if catalog.sources.len() != 2 * bg_alternate_palettes::BASE_KEYS {
            return Err("Alternate palette source count mismatch".into());
        }
        let mut bindings = Vec::new();
        for (group, source) in catalog.tables.iter().enumerate() {
            let bytes: Vec<_> = palettes.relative_sources[group]
                .iter()
                .flat_map(|link| link.to_le_bytes())
                .collect();
            if source.length as usize != bytes.len() || ffta_rom::digest(&bytes) != source.sha256 {
                return Err("Alternate palette link-table binding mismatch".into());
            }
            bindings.push(Binding {
                offset: source.offset as usize,
                length: bytes.len(),
                item: group,
            });
        }
        let mut literals = Vec::new();
        for (row, (palette, source)) in palettes
            .catalogs
            .iter()
            .flatten()
            .zip(&catalog.sources)
            .enumerate()
        {
            if source.length as usize != palette.encoded_source.len()
                || ffta_rom::digest(&palette.encoded_source) != source.sha256
            {
                return Err("Alternate palette source binding mismatch".into());
            }
            let block = bg_alternate_palettes::decode(
                &palette.encoded_source,
                bg_alternate_palettes::OUTPUT_BYTES,
            )?;
            let mut positions = vec![u16::MAX; block.consumed];
            for span in block.literal_spans {
                for index in 0..span.length {
                    positions[span.encoded_offset + index] = (span.output_offset + index) as u16;
                }
            }
            literals.push(positions);
            bindings.push(Binding {
                offset: source.offset as usize,
                length: source.length as usize,
                item: row + 2,
            });
        }
        bindings.sort_by_key(|binding| binding.offset);
        for (index, binding) in bindings.iter().enumerate() {
            let end = binding
                .offset
                .checked_add(binding.length)
                .ok_or("Alternate binding overflow")?;
            if binding.length == 0
                || end > 0x0200_0000
                || !Self::source_page(binding.offset)
                || !Self::source_page(end - 1)
                || bindings
                    .get(index + 1)
                    .is_some_and(|next| end > next.offset)
            {
                return Err("Invalid or overlapping alternate palette bindings".into());
            }
        }
        let last = bindings
            .last()
            .ok_or("Missing alternate palette bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            palettes,
            literals,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            table_bytes: AtomicU64::new(0),
            decoded_literal_bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.palettes.descriptor_keys.capacity()
            + self
                .palettes
                .relative_sources
                .iter()
                .map(|links| links.capacity() * 2)
                .sum::<usize>()
            + self
                .palettes
                .catalogs
                .iter()
                .map(|entries| {
                    entries.capacity() * std::mem::size_of::<bg_alternate_palettes::Palette>()
                })
                .sum::<usize>()
            + self
                .palettes
                .catalogs
                .iter()
                .flatten()
                .map(|palette| palette.encoded_source.capacity())
                .sum::<usize>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.literals.capacity() * std::mem::size_of::<Vec<u16>>()
            + self
                .literals
                .iter()
                .map(|positions| positions.capacity() * 2)
                .sum::<usize>()
    }

    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn stats(&self) -> AlternatePaletteStats {
        AlternatePaletteStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            table_bytes: self.table_bytes.load(Ordering::Relaxed),
            decoded_literal_bytes: self.decoded_literal_bytes.load(Ordering::Relaxed),
        }
    }

    fn byte(&self, index: usize) -> Option<(u8, bool, bool)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|binding| binding.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        if binding.item < 2 {
            let link = self.palettes.relative_sources[binding.item][at / 2];
            return Some((link.to_le_bytes()[at % 2], true, false));
        }
        let row = binding.item - 2;
        let palette = &self.palettes.catalogs[row / bg_alternate_palettes::BASE_KEYS]
            [row % bg_alternate_palettes::BASE_KEYS];
        let literal = self.literals[row][at];
        if literal == u16::MAX {
            Some((palette.encoded_source[at], false, false))
        } else {
            let literal = literal as usize;
            Some((
                palette.banks[literal / 32][literal / 2 % 16].to_le_bytes()[literal % 2],
                false,
                true,
            ))
        }
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8]) -> Option<[u8; N]> {
        if index >= self.end || index.saturating_add(N) <= self.bindings[0].offset {
            return None;
        }
        let (mut mapped, mut table, mut literal) = (0, 0, 0);
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, link, decoded)) => {
                mapped += 1;
                table += u64::from(link);
                literal += u64::from(decoded);
                byte
            }
            None => rom[index + i],
        });
        if mapped == 0 {
            return None;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        self.table_bytes.fetch_add(table, Ordering::Relaxed);
        self.decoded_literal_bytes
            .fetch_add(literal, Ordering::Relaxed);
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{capi::RT_API, Bus, Machine};

    fn fixture(version: Version) -> (Machine, Arc<RuntimeResources>, Vec<(usize, usize)>, Vec<u8>) {
        let roots = if version == Version::Usa {
            [0x427a04usize, 0x42cc80]
        } else {
            [0x489bb0usize, 0x48ee2c]
        };
        let mut rom = vec![0x53; 0x1000000];
        let links: Vec<_> = (0..157).map(|key| (316 + key * 16) as u16).collect();
        let mut catalogs = [Vec::new(), Vec::new()];
        let mut sources = Vec::new();
        let mut spans = Vec::new();
        let tables = std::array::from_fn(|group| {
            let bytes: Vec<_> = links.iter().flat_map(|link| link.to_le_bytes()).collect();
            rom[roots[group]..roots[group] + bytes.len()].copy_from_slice(&bytes);
            spans.push((roots[group], bytes.len()));
            SourceRange {
                offset: roots[group] as u32,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: format!("links/{group}"),
            }
        });
        for group in 0..2 {
            for key in 0..157 {
                let value = 0x81u8.wrapping_add(key as u8).wrapping_add(group as u8);
                let encoded = vec![0, 0, 1, 0, 0x40, value, 0, 250, 0, 0];
                let start = roots[group] + links[key] as usize;
                rom[start..start + encoded.len()].copy_from_slice(&encoded);
                spans.push((start, encoded.len()));
                sources.push(SourceRange {
                    offset: start as u32,
                    length: encoded.len() as u32,
                    sha256: ffta_rom::digest(&encoded),
                    role: format!("stream/{group}/{key}"),
                });
                catalogs[group].push(bg_alternate_palettes::Palette::decode(&encoded).unwrap().0);
            }
        }
        let catalog = bg_alternate_palettes::Catalog {
            decoded: bg_alternate_palettes::Palettes {
                descriptor_keys: (0..157u8).chain([255; 5]).collect(),
                relative_sources: [links.clone(), links],
                catalogs,
            },
            tables,
            sources,
        };
        let mut alternate = AlternatePaletteResources::new(
            catalog.decoded.clone(),
            &catalog,
            "synthetic-alternate".into(),
        )
        .unwrap();
        // Deliberately poison only retained encoded literal payloads after
        // validation. These reads must use the typed bank, not that metadata.
        for palette in alternate.palettes.catalogs.iter_mut().flatten() {
            palette.encoded_source[5] ^= 0xff;
        }
        let mut resources = RuntimeResources::new(
            ffta_rom::digest(&rom),
            "synthetic-dataset".into(),
            "synthetic-obj".into(),
            version,
            super::super::tests::dimensions(),
        );
        resources.alternate_palettes = Some(Box::new(alternate));
        let resources = Arc::new(resources);
        let mut machine = Machine::new(rom.clone());
        machine
            .bus
            .set_rom_resources(Some(resources.clone()))
            .unwrap();
        for &(start, length) in &spans {
            machine.bus.rom[start..start + length].fill(0xcd);
        }
        (machine, resources, spans, rom)
    }

    #[test]
    fn both_regional_source_views_use_banks_links_and_original_control_bytes() {
        for version in [Version::Usa, Version::Europe] {
            let (mut machine, resources, spans, rom) = fixture(version);
            for &(start, length) in &spans {
                for index in start - 3..start + length + 3 {
                    for mirror in [0x0800_0000u32, 0x0a00_0000, 0x0c00_0000] {
                        assert_eq!(machine.bus.read8(mirror + index as u32), rom[index]);
                        let at = index;
                        assert_eq!(
                            machine.bus.read16(mirror + index as u32),
                            u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
                        );
                        let at = index;
                        assert_eq!(
                            machine.bus.read32(mirror + index as u32),
                            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                        );
                    }
                }
            }
            let at = spans[2].0 + 5;
            let before = resources.alternate_palette_stats().unwrap();
            let pointer = &mut machine as *mut Machine as *mut std::ffi::c_void;
            assert_eq!(
                (RT_API.read8)(pointer, 0x0800_0000 + at as u32),
                rom[at] as u32
            );
            let after = resources.alternate_palette_stats().unwrap();
            assert_eq!(
                after.decoded_literal_bytes,
                before.decoded_literal_bytes + 1
            );
            assert!(after.table_bytes > 0 && after.bus_reads > 0);
            machine
                .bus
                .write32(0x0400_00d4, 0x0800_0000 + spans[2].0 as u32);
            machine.bus.write32(0x0400_00d8, 0x0200_0100);
            machine.bus.write16(0x0400_00dc, 5);
            machine.bus.write16(0x0400_00de, 0x8000);
            assert_eq!(
                &machine.bus.ewram[0x100..0x10a],
                &rom[spans[2].0..spans[2].0 + 10]
            );
            let count = resources.alternate_palette_stats().unwrap();
            let state = machine.save_state().unwrap();
            machine.load_state(&state).unwrap();
            assert!(Arc::ptr_eq(
                machine.bus.rom_resources().unwrap(),
                &resources
            ));
            assert_eq!(resources.alternate_palette_stats().unwrap(), count);
        }
    }
}
