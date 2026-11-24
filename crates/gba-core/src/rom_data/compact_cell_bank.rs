//! The authenticated European compact-cell bank adjacent to the tilemap streams.
use super::{
    script39_registry_poses::{half, word, PriorSources},
    tilemap_stream_bank, Resource,
};
use std::{collections::BTreeSet, ops::Range};

const ROM_BASE: u32 = 0x0800_0000;
const START: u32 = 0xae9ebc;
const END: u32 = tilemap_stream_bank::START;

// E489/E490 literal loads passing a record root or its split pointers.
const ROOT_LITERALS: [u32; 78] = [
    0x0e5b98, 0x0e5de4, 0x0e6914, 0x0e6a2c, 0x0e6e1c, 0x0e71d4, 0x0e7f14, 0x0e8ec8, 0x0e92e8,
    0x0e93fc, 0x0ea398, 0x0eabc4, 0x0eaf14, 0x0ec770, 0x0ed244, 0x0f1f98, 0x0f2390, 0x0f25ec,
    0x0f2ba4, 0x0f3034, 0x0f3bb8, 0x0f4024, 0x0f5520, 0x0f5de8, 0x0f6008, 0x0f71ec, 0x0f9cd4,
    0x0f9e0c, 0x0fa2c0, 0x0fb8bc, 0x0fc5cc, 0x0fe050, 0x0ff104, 0x0ffdf4, 0x102650, 0x103bc4,
    0x1052f8, 0x105504, 0x105a84, 0x10819c, 0x109078, 0x10a210, 0x10a3d4, 0x10aedc, 0x10b538,
    0x10ca30, 0x10cc50, 0x10d93c, 0x10e2e4, 0x10eea0, 0x10f0e0, 0x10f170, 0x10fec4, 0x110154,
    0x110a70, 0x110d84, 0x110f10, 0x111af4, 0x111cec, 0x111ecc, 0x11219c, 0x114ed8, 0x117684,
    0x11824c, 0x119e10, 0x11a70c, 0x11d290, 0x11d6cc, 0x11db18, 0x11de48, 0x11e3e0, 0x11e594,
    0x11f858, 0x1206e0, 0x1211c8, 0x122ba8, 0x123610, 0x124208,
];

// Pointer-table literal, parameter-table literal and number of physical slots.
const SELECTOR_TABLES: [(u32, u32, u32); 9] = [
    (0x0e73dc, 0x0e73e0, 9),
    (0x0e746c, 0x0e7470, 9),
    (0x0ebde0, 0x0ebde4, 3),
    (0x0ec218, 0x0ec21c, 3),
    (0x0fbc9c, 0x0fbca0, 2),
    (0x0fe504, 0x0fe508, 2),
    (0x1001d8, 0x1001dc, 2),
    (0x10225c, 0x102260, 5),
    (0x10c41c, 0x10c420, 6),
];

/// Row-major cell values before the renderer applies its transparency, palette
/// and placement settings. Byte cells are widened to `u16`.
#[derive(Debug, PartialEq, Eq)]
pub struct CompactCellFrame {
    pub record_offset: usize,
    pub encoded_len: usize,
    pub width: u8,
    pub height: u8,
    pub placement: [u8; 2],
    pub cells: Vec<u16>,
}

fn decode_frame(
    frame: &[u8],
    record_offset: usize,
    value_bytes: usize,
) -> Result<CompactCellFrame, String> {
    let &[width, height, x, y, ref runs @ ..] = frame else {
        return Err("Truncated compact-cell frame header".into());
    };
    let count = usize::from(width) * usize::from(height);
    if count == 0 {
        return Err("Empty compact-cell frame".into());
    }

    let mut cells = Vec::with_capacity(count);
    let mut remaining = runs;
    while cells.len() < count {
        let &[low, high, ref payload @ ..] = remaining else {
            return Err("Truncated compact-cell run control".into());
        };
        let control = u16::from_le_bytes([low, high]);
        let length = usize::from(control & 0x3fff);
        if length == 0 || length > count - cells.len() {
            return Err("Invalid compact-cell run length".into());
        }

        let mode = control & 0xc000;
        let payload_len = value_bytes * if mode == 0 { length } else { 1 };
        let stored_len = payload_len.next_multiple_of(2);
        remaining = payload
            .get(stored_len..)
            .ok_or("Truncated compact-cell run payload")?;
        let mut values = payload[..payload_len]
            .chunks_exact(value_bytes)
            .map(|value| {
                if value_bytes == 1 {
                    u16::from(value[0])
                } else {
                    u16::from_le_bytes([value[0], value[1]])
                }
            });
        if mode == 0 {
            cells.extend(values);
        } else {
            let first = values.next().unwrap();
            let mask = if value_bytes == 1 { 0xff } else { 0xffff };
            let step = u16::from(mode & 0x8000 != 0);
            cells.extend((0..length as u16).map(|i| first.wrapping_add(step * i) & mask));
        }
    }
    Ok(CompactCellFrame {
        record_offset,
        encoded_len: frame.len() - remaining.len(),
        width,
        height,
        placement: [x, y],
        cells,
    })
}

/// Decode the bounded frames in one complete European `compact-record` span.
/// The records with header value 12 contain halfword runs and those with
/// value 14 contain byte runs. The guest keeps its original cartridge view.
pub fn decode_frames(record: &[u8]) -> Result<Vec<CompactCellFrame>, String> {
    let &[low, high, split_low, split_high, ..] = record else {
        return Err("Truncated compact-cell record header".into());
    };
    let value_bytes = match u16::from_le_bytes([low, high]) {
        12 => 2,
        14 => 1,
        _ => return Err("Unsupported compact-cell record encoding".into()),
    };
    let sequence = usize::from(u16::from_le_bytes([split_low, split_high]));
    let directory = record
        .get(4..sequence)
        .ok_or("Invalid compact-cell frame directory")?;
    let prefix = u16::from_le_bytes(
        *directory
            .first_chunk::<2>()
            .ok_or("Truncated compact-cell frame directory")?,
    ) as usize;
    if !(2..=128).contains(&prefix) || prefix % 2 != 0 || prefix >= directory.len() {
        return Err("Invalid compact-cell frame directory length".into());
    }
    let offsets: Vec<_> = directory[..prefix]
        .chunks_exact(2)
        .map(|offset| usize::from(u16::from_le_bytes([offset[0], offset[1]])))
        .collect();
    let mut frames = Vec::with_capacity(offsets.len());
    for (&start, end) in offsets
        .iter()
        .zip(offsets.iter().skip(1).copied().chain([directory.len()]))
    {
        if start % 2 != 0 || start < prefix || start >= end {
            return Err("Invalid compact-cell frame offset".into());
        }
        let bytes = directory
            .get(start..end)
            .ok_or("Compact-cell frame extends outside the directory")?;
        let frame = decode_frame(bytes, 4 + start, value_bytes)?;
        let padding = bytes.len() - frame.encoded_len;
        if padding != 0 && (end != directory.len() || padding != 2) {
            return Err("Unexpected bytes after compact-cell frame runs".into());
        }
        frames.push(frame);
    }
    Ok(frames)
}

fn pointer(rom: &[u8], cell: u32, length: u32, alignment: u32) -> Result<u32, String> {
    word(rom, cell)?
        .checked_sub(ROM_BASE)
        .filter(|offset| offset % alignment == 0 && (*offset + length) as usize <= rom.len())
        .ok_or_else(|| format!("Invalid compact-cell pointer at {cell:#x}"))
}

fn record_end(rom: &[u8], root: u32, next: u32) -> Result<u32, String> {
    let sequence = root + u32::from(half(rom, root + 2)?);
    if sequence <= root + 4 || sequence + 2 > next {
        return Err(format!("Invalid compact-cell record header at {root:#x}"));
    }
    let count = u32::from(half(rom, sequence)?);
    let end = sequence + 2 + count * 2;
    if count == 0 || (end + 3) & !3 != next {
        return Err(format!("Invalid compact-cell record end at {root:#x}"));
    }
    let record = rom
        .get(root as usize..end as usize)
        .ok_or("Truncated compact-cell record")?;
    decode_frames(record).map_err(|error| format!("Compact-cell record {root:#x}: {error}"))?;
    Ok(end)
}

pub(super) fn uncovered_ranges(
    rom: &[u8],
    existing: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let mut ranges = Vec::new();
    let mut roots = BTreeSet::new();
    for literal in ROOT_LITERALS {
        roots.insert(pointer(rom, literal, 4, 4)?);
        ranges.push((literal..literal + 4, "constructor-literal"));
    }
    for (root_literal, parameter_literal, slots) in SELECTOR_TABLES {
        let table = pointer(rom, root_literal, 4 * slots, 4)?;
        let parameters = pointer(rom, parameter_literal, 2 * slots, 2)?;
        ranges.extend([
            (root_literal..root_literal + 4, "constructor-literal"),
            (
                parameter_literal..parameter_literal + 4,
                "constructor-literal",
            ),
            (table..table + 4 * slots, "indexed-root-pointers"),
            (parameters..parameters + 2 * slots, "indexed-parameters"),
        ]);
        for slot in 0..slots {
            roots.insert(pointer(rom, table + 4 * slot, 4, 4)?);
        }
    }

    if roots.len() != 91 || roots.first() != Some(&START) || roots.last() >= Some(&END) {
        return Err("Invalid European compact-cell bank roots".into());
    }
    // Preserve both original frame encodings and any offsets into the records.
    for (&root, next) in roots.iter().zip(roots.iter().skip(1).copied().chain([END])) {
        ranges.push((root..record_end(rom, root, next)?, "compact-record"));
    }

    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping compact-cell bank source ranges".into());
    }
    let prior = PriorSources::new(existing)?;
    Ok(ranges
        .into_iter()
        .flat_map(|(range, role)| {
            prior
                .uncovered(range)
                .into_iter()
                .map(move |span| (span, role))
        })
        .collect())
}
