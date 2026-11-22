//! Reconstruct original tilemap runs and selected halfword rectangles.
use super::super::{observed_tilemap as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapStats { pub bus_reads: u64, pub source_bytes: u64, pub rectangle_bytes: [u64; 2], pub root_bytes: [u64; 2], pub alternate_rectangle_bytes: [u64; 2], pub alternate_selector_bytes: u64, pub indexed_pointer_bytes: u64, pub additional_stream_bytes: [u64; 7], pub additional_rectangle_bytes: [u64; 5], pub additional_rectangle_pointer_bytes: u64, pub soil_list_bytes: u64, pub soil_entry_bytes: u64, pub soil_rectangle_bytes: [u64; 2], pub soil_mode7_header_bytes: u64, pub soil_mode7_run_bytes: u64 }

pub(super) struct ObservedTilemapResources {
    data: model::ObservedTilemap,
    reads: AtomicU64,
    bytes: AtomicU64,
    selected_bytes: [AtomicU64; 27],
}

impl ObservedTilemapResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| matches!(page, 0x39e | 0x3a4 | 0xae6 | 0xae7 | 0xaf2))
    }

    pub(super) fn new(data: model::ObservedTilemap, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans();
        if sources.len() != spans.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) { return Err("Observed tilemap source binding mismatch".into()); }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0), selected_bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }

    fn byte(&self, index: usize) -> Option<(u8, Option<usize>)> {
        if let Some(n) = index.checked_sub(self.data.source_offset as usize) {
            let value = match n {
                0 => Some(self.data.width), 1 => Some(self.data.height),
                2..=3 => Some(self.data.opaque_header.to_le_bytes()[n - 2]),
                4 => Some(self.data.mode), 5 => Some(self.data.flags),
                6..model::SOURCE_LENGTH => {
                    let run = &self.data.runs[(n - 6) / 4]; let at = (n - 6) % 4;
                    Some(if at < 2 { run.control.to_le_bytes()[at] } else { run.value.to_le_bytes()[at - 2] })
                }
                _ => None,
            };
            if let Some(value) = value { return Some((value, None)); }
        }
        self.data.rectangles.iter().enumerate().find_map(|(index_in_data, rectangle)| {
            let at = index.checked_sub(rectangle.source_offset as usize)?;
            let value = match at {
                0 => Some(rectangle.height), 1 => Some(rectangle.width),
                _ => rectangle.cells.get((at - 2) / 2).map(|cell| cell.to_le_bytes()[(at - 2) % 2]),
            }?;
            Some((value, Some(index_in_data)))
        }).or_else(|| {
            self.data.alternate_rectangles.as_ref()?.iter().enumerate().find_map(|(which, rectangle)| {
                let at = index.checked_sub(rectangle.source_offset as usize)?;
                let value = match at {
                    0 => Some(rectangle.height), 1 => Some(rectangle.width),
                    _ => rectangle.cells.get((at - 2) / 2).map(|cell| cell.to_le_bytes()[(at - 2) % 2]),
                }?;
                Some((value, Some(4 + which)))
            })
        }).or_else(|| {
            index.checked_sub(model::ALTERNATE_SELECTOR_OFFSET as usize)
                .and_then(|at| self.data.alternate_selector.as_ref()?.get(at).copied()).map(|value| (value, Some(6)))
        }).or_else(|| {
            let at = index.checked_sub(model::INDEXED_POINTER_OFFSET as usize)?;
            let pointer = self.data.indexed_pointers.as_ref()?.get(at / 4)?;
            Some((pointer.to_le_bytes()[at % 4], Some(7)))
        }).or_else(|| {
            self.data.additional_streams.as_ref()?.iter().enumerate()
                .find_map(|(which, stream)| stream.source_byte(index).map(|value| (value, Some(8 + which))))
        }).or_else(|| {
            let at = index.checked_sub(model::ADDITIONAL_RECTANGLE_POINTER_OFFSET as usize)?;
            let pointer = self.data.additional_rectangle_pointers.as_ref()?.get(at / 4)?;
            Some((pointer.to_le_bytes()[at % 4], Some(20)))
        }).or_else(|| {
            self.data.additional_rectangles.as_ref()?.iter().enumerate().find_map(|(which, rectangle)| {
                let at = index.checked_sub(rectangle.source_offset as usize)?;
                let value = match at {
                    0 => Some(rectangle.height), 1 => Some(rectangle.width),
                    _ => rectangle.cells.get((at - 2) / 2).map(|cell| cell.to_le_bytes()[(at - 2) % 2]),
                }?;
                Some((value, Some(15 + which)))
            })
        }).or_else(|| {
            let soil = self.data.soil_transfer_sources.as_ref()?;
            index.checked_sub(model::SOIL_RECTANGLE_LIST_OFFSET as usize)
                .and_then(|at| soil.list_bytes().get(at).copied()).map(|value| (value, Some(21)))
                .or_else(|| index.checked_sub(model::SOIL_RECTANGLE_ENTRIES_OFFSET as usize)
                    .and_then(|at| soil.entry_bytes().get(at).copied()).map(|value| (value, Some(22))))
                .or_else(|| soil.rectangles.iter().enumerate().find_map(|(which, rectangle)| {
                    let at = index.checked_sub(rectangle.source_offset as usize)?;
                    let value = match at {
                        0 => Some(rectangle.height), 1 => Some(rectangle.width),
                        _ => rectangle.cells.get((at - 2) / 2).map(|cell| cell.to_le_bytes()[(at - 2) % 2]),
                    }?;
                    Some((value, Some(23 + which)))
                }))
                .or_else(|| index.checked_sub(model::SOIL_MODE7_OFFSET as usize)
                    .and_then(|at| soil.mode7.header_bytes().get(at).copied()).map(|value| (value, Some(25))))
                .or_else(|| index.checked_sub(model::SOIL_MODE7_OFFSET as usize + 6)
                    .and_then(|at| soil.mode7.run_bytes().get(at).copied()).map(|value| (value, Some(26))))
        }).or_else(|| {
            index.checked_sub(model::RECTANGLE_CHAIN_OFFSET as usize)
                .and_then(|at| self.data.rectangle_roots.chain_bytes().get(at).copied()).map(|value| (value, Some(2)))
        }).or_else(|| {
            index.checked_sub(model::RECTANGLE_TABLE_OFFSET as usize)
                .and_then(|at| self.data.rectangle_roots.table_bytes().get(at).copied()).map(|value| (value, Some(3)))
        })
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].map(|(value, _)| value).unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        let mut selected = [0u64; 27];
        for (_, index) in cells.iter().flatten() { if let Some(index) = index { selected[*index] += 1; } }
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        for (counter, count) in self.selected_bytes.iter().zip(selected) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }

    pub(super) fn stats(&self) -> ObservedTilemapStats {
        let [rect0, rect1, root0, root1, alt0, alt1, selector, pointers, stream0, stream1, stream2, stream3, stream4, stream5, stream6, extra0, extra1, extra2, extra3, extra4, extra_pointers, soil_list, soil_entries, soil_rect0, soil_rect1, soil_header, soil_run] = self.selected_bytes.each_ref().map(|counter| counter.load(Ordering::Relaxed));
        ObservedTilemapStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed),
            rectangle_bytes: [rect0, rect1], root_bytes: [root0, root1], alternate_rectangle_bytes: [alt0, alt1], alternate_selector_bytes: selector, indexed_pointer_bytes: pointers,
            additional_stream_bytes: [stream0, stream1, stream2, stream3, stream4, stream5, stream6],
            additional_rectangle_bytes: [extra0, extra1, extra2, extra3, extra4], additional_rectangle_pointer_bytes: extra_pointers,
            soil_list_bytes: soil_list, soil_entry_bytes: soil_entries, soil_rectangle_bytes: [soil_rect0, soil_rect1], soil_mode7_header_bytes: soil_header, soil_mode7_run_bytes: soil_run }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.rectangles.iter().map(|rectangle| rectangle.cells.capacity() * std::mem::size_of::<u16>()).sum::<usize>()
            + self.data.alternate_rectangles.as_ref().map_or(0, |rectangles| rectangles.iter().map(|rectangle| rectangle.cells.capacity() * std::mem::size_of::<u16>()).sum())
            + self.data.additional_rectangles.as_ref().map_or(0, |rectangles| rectangles.iter().map(|rectangle| rectangle.cells.capacity() * std::mem::size_of::<u16>()).sum())
            + self.data.soil_transfer_sources.as_ref().map_or(0, |soil| soil.rectangles.iter().map(|rectangle| rectangle.cells.capacity() * std::mem::size_of::<u16>()).sum())
            + self.data.additional_streams.as_ref().map_or(0, |streams| streams.iter().map(|stream| {
                stream.tokens.capacity() * std::mem::size_of::<model::StreamToken>()
                    + stream.tokens.iter().map(|token| token.values.capacity() * std::mem::size_of::<u16>()).sum::<usize>()
            }).sum())
    }
}
