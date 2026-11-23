//! Authenticated EUR Soil Evidence records used through an indirect pointer.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const OFFSETS: [u32; 5] = [0x39aaac, 0x39aad4, 0x39ab5c, 0x39e214, 0x39e21c];
pub const RECORD_BYTES: usize = 8;
pub const FIRAGA_INDEX2_OFFSET: u32 = 0x39aac4;
pub const FIRAGA_CCEC_OFFSET: u32 = 0x39ccec;
const ROLES: [&str; 5] = [
    "soil-indirect-record-0",
    "soil-indirect-record-1",
    "soil-indirect-record-2",
    "soil-indirect-record-3",
    "soil-indirect-record-4",
];
const HASHES: [&str; 7] = [
    "4d8aef96c6d8c1d6428723dfede1dcd3587a7f64511f44d5a0ec72c0f023472a",
    "9521db027bf863bcfa16500b438045789f55c55e95db189f09b4ff8617072ab9",
    "08264f8b86ee077c13000c74df52736899e8feca41d9ee526b3610afd218531e",
    "80e64384f9f84c0908408db53db8e983b86b1d89b0afca60b00c9374363c0e37",
    "9998fe3fa03b15880d32b9894730f28d79f9a80d956b198d5ad72af211956103",
    "2984269080c40f5c6dbdc8e8ee6e2312f2feb1d606430caf0e4dcca4eb06051a",
    "5650272d09ffd65ab9ee764ac2e7bd32f19fa26c0179cc264bbd26f772de9147",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndirectRecord {
    pub offset: u32,
    pub field0: u16,
    pub fields: [u8; 2],
    pub values: [u16; 2],
}
impl IndirectRecord {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let first = self.field0.to_le_bytes();
        let value0 = self.values[0].to_le_bytes();
        let value1 = self.values[1].to_le_bytes();
        [
            first[0],
            first[1],
            self.fields[0],
            self.fields[1],
            value0[0],
            value0[1],
            value1[0],
            value1[1],
        ]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilIndirectRecords {
    pub records: [IndirectRecord; 5],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firaga_index2: Option<IndirectRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firaga_ccec: Option<IndirectRecord>,
}
impl SoilIndirectRecords {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .records
            .iter()
            .zip(OFFSETS)
            .any(|(row, offset)| row.offset != offset)
        {
            return Err("Invalid Soil indirect record offsets".into());
        }
        if self.firaga_index2.as_ref().is_some_and(|row| row.offset != FIRAGA_INDEX2_OFFSET) {
            return Err("Invalid Firaga indirect index-2 record offset".into());
        }
        if self.firaga_ccec.as_ref().is_some_and(|row| row.offset != FIRAGA_CCEC_OFFSET) {
            return Err("Invalid Firaga CCEC indirect record offset".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut spans: Vec<_> = self
            .records
            .iter()
            .zip(ROLES)
            .map(|(row, role)| (row.offset, row.source_bytes().to_vec(), role))
            .collect();
        if let Some(row) = &self.firaga_index2 {
            spans.push((row.offset, row.source_bytes().to_vec(), "firaga-indirect-index2-record"));
        }
        if let Some(row) = &self.firaga_ccec {
            spans.push((row.offset, row.source_bytes().to_vec(), "firaga-ccec-indirect-record"));
        }
        Ok(spans)
    }
    /// These are the owned physical records, not a bound on guest pointers.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        self.records.iter().enumerate().find_map(|(i, row)| {
            let at = offset.checked_sub(row.offset)? as usize;
            row.source_bytes().get(at).copied().map(|byte| (byte, i))
        }).or_else(|| {
            let row = self.firaga_index2.as_ref()?;
            let at = offset.checked_sub(row.offset)? as usize;
            row.source_bytes().get(at).copied().map(|byte| (byte, 5))
        }).or_else(|| {
            let row = self.firaga_ccec.as_ref()?;
            let at = offset.checked_sub(row.offset)? as usize;
            row.source_bytes().get(at).copied().map(|byte| (byte, 6))
        })
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil indirect records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SoilIndirectRecords,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil indirect records require the exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0xe361c,
                0xe37a8,
                "010f22d92b37313798c8588cb7ac202032d39a2d17a961b0d57faf2d3a0df0a8",
            ),
            (
                0xe389c,
                0xe38e8,
                "81c927910e9b10723babf7e7081b3b0978c6bc33875c927a24aed6a204299e17",
            ),
            (
                0xe4b0c,
                0xe4b22,
                "4db6c9a3234135ed7db640c851e51390d8dc4f8e177f1c1ed9b3cf070e794946",
            ),
            (
                0xe4b46,
                0xe4b56,
                "b022ed0085ca47d97bdc3d61751e2d590e7cc550ce5f62d90a28c67c3be8c186",
            ),
            (
                0xe4c6e,
                0xe4c7e,
                "3e248f63fdb851193f155ef6f19976c7999ab5506ce4b9faf672470db202b1db",
            ),
            (
                0xf05d2,
                0xf05e0,
                "06d153c5a50172ad761ead1b83d4e152d2ea343d4fe0e9d106627c68417d2c51",
            ),
            (
                0xf0a02,
                0xf0a10,
                "323c5497d9c5a28b66c2716ded3dcd064ea73d4004174dfba8859b4bddda6f42",
            ),
            (
                0xeb988,
                0xeb9a2,
                "e87a9408b43880215412f41e0a12df78be4b28ccc5af2c3ccfec9d39a5e90302",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Soil indirect reader fingerprint mismatch".into());
            }
        }
        let roots = [
            (0xe4b10u32, 0xe4b84u32, 0x39aab4u32, 4usize),
            (0xe4b46, 0xe4b90, 0x39aaac, 0),
            (0xe4c6e, 0xe4c88, 0x39ab5c, 0),
            (0xf0a02, 0xf0a24, 0x39e214, 0),
            (0xf05d2, 0xf0600, 0x39e21c, 0),
            (0xe4b10, 0xe4b84, 0x39aab4, 2),
        ];
        let literals = previous
            .iter()
            .find(|resource| resource.id == THUMB_PC_LITERALS_ID);
        if !previous.is_empty() && literals.is_none() {
            return Err("Missing Soil indirect literal dependency".into());
        }
        for (reader, literal, base, index) in roots {
            let op = u16::from_le_bytes(
                rom[reader as usize..reader as usize + 2]
                    .try_into()
                    .unwrap(),
            );
            let target = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if op & 0xf800 != 0x4800
                || ((reader + 4) & !3) + u32::from(op & 255) * 4 != literal
                || target != 0x0800_0000 + base
                || !(OFFSETS.contains(&(base + index as u32 * RECORD_BYTES as u32))
                    || base + index as u32 * RECORD_BYTES as u32 == FIRAGA_INDEX2_OFFSET)
            {
                return Err("Soil indirect reader literal mismatch".into());
            }
            if literals.is_some_and(|owner| {
                !owner
                    .sources
                    .iter()
                    .any(|source| source.offset == literal && source.length == 4)
            }) {
                return Err("Missing separately owned Soil indirect literal".into());
            }
        }
        // The indexed pointer is published by the later compact-cells resource;
        // only its separately owned instruction literal is needed here.
        let (reader, literal) = (0xeb988u32, 0xeb9d8u32);
        let op = u16::from_le_bytes(rom[reader as usize..reader as usize + 2].try_into().unwrap());
        if op & 0xf800 != 0x4800
            || ((reader + 4) & !3) + u32::from(op & 255) * 4 != literal
            || u32::from_le_bytes(rom[literal as usize..literal as usize + 4].try_into().unwrap()) != 0x0839_ccf4
            || u32::from_le_bytes(rom[0x39ccfc..0x39cd00].try_into().unwrap()) != 0x0800_0000 + FIRAGA_CCEC_OFFSET
            || literals.is_some_and(|owner| !owner.sources.iter().any(|source| source.offset == literal && source.length == 4)) {
            return Err("Firaga CCEC indexed pointer provenance mismatch".into());
        }
        let record_at = |offset: u32| {
            let at = offset as usize;
            IndirectRecord {
                offset,
                field0: u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()),
                fields: [rom[at + 2], rom[at + 3]],
                values: [
                    u16::from_le_bytes(rom[at + 4..at + 6].try_into().unwrap()),
                    u16::from_le_bytes(rom[at + 6..at + 8].try_into().unwrap()),
                ],
            }
        };
        let records = std::array::from_fn(|i| record_at(OFFSETS[i]));
        let decoded = SoilIndirectRecords { records, firaga_index2: Some(record_at(FIRAGA_INDEX2_OFFSET)), firaga_ccec: Some(record_at(FIRAGA_CCEC_OFFSET)) };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil indirect source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Soil indirect record source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
