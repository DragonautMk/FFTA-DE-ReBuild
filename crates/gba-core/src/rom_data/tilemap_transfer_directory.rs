//! Authenticated controls and input extents for the European task-345 transfer bank.
use super::{
    lz77,
    observed_tilemap_descriptor::{AdditionalDescriptor, LzWrapper},
    observed_tilemap_row::AdditionalRow,
    script39_registry_poses::PriorSources,
    Resource, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

pub const ROM_SIZE: u32 = 0x0100_0000;
pub const ROM_BASE: u32 = 0x0800_0000;
pub const DIRECTORY_LITERALS: u32 = 0x0e61e8;
pub const POINTER_DIRECTORY: u32 = 0x39a818;
pub const COUNT_DIRECTORY: u32 = 0x39a874;
pub const SLOT_COUNT: usize = 23;
pub const ROW_BYTES: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub row_pointer: u32,
    pub row_count: u8,
}

impl Slot {
    pub fn row_offsets(&self) -> Result<impl Iterator<Item = u32>, String> {
        let count = u32::from(self.row_count);
        let offset = match (self.row_pointer, count) {
            (0, 0) => 0,
            (0, _) | (_, 0) => return Err("Inconsistent tilemap bank slot".into()),
            (pointer, count) => target(pointer, count * ROW_BYTES)?,
        };
        Ok((0..count).map(move |index| offset + index * ROW_BYTES))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TilemapTransferDirectory {
    pub slots: [Slot; SLOT_COUNT],
    pub rows: Vec<AdditionalRow>,
    pub descriptors: Vec<AdditionalDescriptor>,
}

#[derive(Clone, Copy)]
pub(super) enum Role {
    ListPointer,
    ListCount,
    Row,
    Descriptor,
    Wrapper,
}

impl Role {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::ListPointer => "list-pointer",
            Self::ListCount => "list-count",
            Self::Row => "transfer-row",
            Self::Descriptor => "transfer-descriptor",
            Self::Wrapper => "stream-wrapper",
        }
    }
}

pub(super) struct Field {
    pub(super) offset: u32,
    pub(super) bytes: Vec<u8>,
    pub(super) role: Role,
}

impl Field {
    pub(super) fn source_range(&self) -> SourceRange {
        SourceRange {
            offset: self.offset,
            length: self.bytes.len() as u32,
            sha256: ffta_rom::digest(&self.bytes),
            role: self.role.name().into(),
        }
    }
}

pub struct Catalog {
    pub decoded: TilemapTransferDirectory,
    pub sources: Vec<SourceRange>,
}

pub(super) fn target(address: u32, length: u32) -> Result<u32, String> {
    address
        .checked_sub(ROM_BASE)
        .filter(|&offset| offset % 4 == 0 && offset <= ROM_SIZE - length)
        .ok_or_else(|| format!("Invalid tilemap bank pointer {address:#x}"))
}

pub(super) fn bytes<const N: usize>(rom: &[u8], offset: u32) -> Result<[u8; N], String> {
    let start = offset as usize;
    rom.get(start..start + N)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| format!("Truncated tilemap bank field at {offset:#x}"))
}

pub(super) fn row_at(rom: &[u8], source_offset: u32) -> Result<AdditionalRow, String> {
    let row = bytes::<12>(rom, source_offset)?;
    Ok(AdditionalRow {
        source_offset,
        field0: row[0],
        field1: row[1],
        field2: u16::from_le_bytes(row[2..4].try_into().unwrap()),
        descriptor_pointer: u32::from_le_bytes(row[4..8].try_into().unwrap()),
        field8: u32::from_le_bytes(row[8..12].try_into().unwrap()),
    })
}

fn descriptor_at(rom: &[u8], source_offset: u32) -> Result<AdditionalDescriptor, String> {
    let [p0, p1, p2, p3, a, b, parameter8, opaque_byte] = bytes(rom, source_offset)?;
    let asset_pointer = u32::from_le_bytes([p0, p1, p2, p3]);
    let wrapper = target(asset_pointer, 4)?;
    let [kind, flags, c, d] = bytes(rom, wrapper)?;
    Ok(AdditionalDescriptor {
        source_offset,
        asset_pointer,
        parameter16: u16::from_le_bytes([a, b]),
        parameter8,
        opaque_byte,
        wrapper: LzWrapper {
            source_offset: wrapper,
            kind,
            flags,
            opaque_halfword: u16::from_le_bytes([c, d]),
        },
    })
}

impl TilemapTransferDirectory {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != ROM_SIZE as usize
            || ffta_rom::version(rom) != Some(ffta_rom::Version::Europe)
        {
            return Err("The task-345 tilemap bank requires the exact European ROM".into());
        }
        let decoded = Self::from_verified_rom(rom)?;
        decoded.validate()?;
        Ok(decoded)
    }

    fn from_verified_rom(rom: &[u8]) -> Result<Self, String> {
        let literals = [
            u32::from_le_bytes(bytes(rom, DIRECTORY_LITERALS)?),
            u32::from_le_bytes(bytes(rom, DIRECTORY_LITERALS + 4)?),
        ];
        if literals != [ROM_BASE + POINTER_DIRECTORY, ROM_BASE + COUNT_DIRECTORY] {
            return Err("Invalid tilemap bank directory addresses".into());
        }
        let mut slots = [Slot {
            row_pointer: 0,
            row_count: 0,
        }; SLOT_COUNT];
        for (index, slot) in slots.iter_mut().enumerate() {
            let index = index as u32;
            slot.row_pointer = u32::from_le_bytes(bytes(rom, POINTER_DIRECTORY + 4 * index)?);
            slot.row_count = bytes::<1>(rom, COUNT_DIRECTORY + index)?[0];
        }
        let rows = row_offsets(&slots)?
            .into_iter()
            .map(|offset| row_at(rom, offset))
            .collect::<Result<Vec<_>, _>>()?;
        let descriptors = descriptor_offsets(&rows)?
            .into_iter()
            .map(|offset| descriptor_at(rom, offset))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            slots,
            rows,
            descriptors,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.fields().map(|_| ())
    }

    /// Locate every distinct encoded input from the authenticated directory
    /// and its source ROM. Shared wrappers produce one extent.
    pub fn lz77_input_ranges(&self, rom: &[u8]) -> Result<Vec<Range<u32>>, String> {
        let starts: BTreeSet<_> = self
            .descriptors
            .iter()
            .map(|descriptor| target(descriptor.asset_pointer, 4).map(|wrapper| wrapper + 4))
            .collect::<Result<_, _>>()?;
        let mut ranges = Vec::with_capacity(starts.len());
        for start in starts {
            let input = rom
                .get(start as usize..)
                .ok_or_else(|| format!("Tilemap LZ77 input outside the ROM at {start:#x}"))?;
            let length = lz77::encoded_length(input, rom.len())
                .map_err(|error| format!("Tilemap LZ77 input at {start:#x}: {error}"))?;
            ranges.push(start..start + length as u32);
        }
        if ranges.windows(2).any(|pair| pair[0].end > pair[1].start) {
            return Err("Overlapping tilemap LZ77 inputs".into());
        }
        Ok(ranges)
    }

    pub(super) fn fields(&self) -> Result<Vec<Field>, String> {
        let rows = row_offsets(&self.slots)?;
        if !rows
            .iter()
            .copied()
            .eq(self.rows.iter().map(|row| row.source_offset))
        {
            return Err("Tilemap bank rows must match the complete selector domain".into());
        }
        let descriptors = descriptor_offsets(&self.rows)?;
        if !descriptors
            .iter()
            .copied()
            .eq(self.descriptors.iter().map(|item| item.source_offset))
        {
            return Err("Tilemap bank descriptors must match the linked rows".into());
        }
        let mut fields = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            fields.push(Field {
                offset: POINTER_DIRECTORY + 4 * index as u32,
                bytes: slot.row_pointer.to_le_bytes().to_vec(),
                role: Role::ListPointer,
            });
            fields.push(Field {
                offset: COUNT_DIRECTORY + index as u32,
                bytes: vec![slot.row_count],
                role: Role::ListCount,
            });
        }
        fields.extend(self.rows.iter().map(|row| Field {
            offset: row.source_offset,
            bytes: row.source_bytes().to_vec(),
            role: Role::Row,
        }));
        let mut wrappers = BTreeMap::new();
        for descriptor in &self.descriptors {
            let wrapper = &descriptor.wrapper;
            if target(descriptor.asset_pointer, 4)? != wrapper.source_offset
                || !(1..=4).contains(&wrapper.kind)
                || wrapper.flags != 2
            {
                return Err("Invalid tilemap bank stream wrapper".into());
            }
            if wrappers
                .insert(wrapper.source_offset, wrapper)
                .is_some_and(|prior| prior != wrapper)
            {
                return Err("Conflicting tilemap bank wrapper alias".into());
            }
            fields.push(Field {
                offset: descriptor.source_offset,
                bytes: descriptor.source_bytes().to_vec(),
                role: Role::Descriptor,
            });
        }
        fields.extend(wrappers.into_values().map(|wrapper| Field {
            offset: wrapper.source_offset,
            bytes: wrapper.source_bytes().to_vec(),
            role: Role::Wrapper,
        }));
        fields.sort_unstable_by_key(|field| field.offset);
        if fields
            .windows(2)
            .any(|pair| pair[0].offset + pair[0].bytes.len() as u32 > pair[1].offset)
        {
            return Err("Overlapping tilemap bank control fields".into());
        }
        Ok(fields)
    }
}

fn row_offsets(slots: &[Slot]) -> Result<BTreeSet<u32>, String> {
    let mut offsets = BTreeSet::new();
    for slot in slots {
        offsets.extend(slot.row_offsets()?);
    }
    Ok(offsets)
}

fn descriptor_offsets(rows: &[AdditionalRow]) -> Result<BTreeSet<u32>, String> {
    rows.iter()
        .map(|row| {
            if row.field0 != 3 {
                return Err(format!(
                    "Unexpected tilemap transfer type at {:#x}",
                    row.source_offset
                ));
            }
            target(row.descriptor_pointer, 8)
        })
        .collect()
}

impl Catalog {
    pub(super) fn from_verified_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        let decoded = TilemapTransferDirectory::from_verified_rom(rom)?;
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for field in decoded.fields()? {
            if !prior.contains(field.offset, field.bytes.len())? {
                sources.push(field.source_range());
            }
        }
        Ok(Self { decoded, sources })
    }
}
