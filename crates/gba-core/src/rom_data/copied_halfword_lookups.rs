//! EUR halfword lookups copied by original guest routines before indexing.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_OFFSETS: [u32; 2] = [0x14cf18, 0x14cf20];
pub const ADDITIONAL_ROOTS: [u32; 2] = [0x3658, 0x3680];
pub const TABLE_BYTES: usize = 8;
const SOURCE_HASHES: [&str; 2] = [
    "2305fc2744df53d32db7b3015b54bef236ea42d3bc9eddf7e7f259b88b1613cb",
    "3d9f65f630695e727bd27f9037a20f7c693469ce276bbf5e17f5d8ad23f2d61b",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HalfwordTable {
    pub source_offset: u32,
    pub values: [u16; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopiedHalfwordLookups {
    pub tables: [HalfwordTable; 2],
}

impl CopiedHalfwordLookups {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .tables
            .iter()
            .zip(TABLE_OFFSETS)
            .any(|(table, offset)| table.source_offset != offset)
        {
            return Err("Invalid EUR copied halfword lookup sources or order".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        for (i, table) in self.tables.iter().enumerate() {
            spans.push((
                table.source_offset,
                table
                    .values
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect(),
                if i == 0 {
                    "first-halfword-lookup"
                } else {
                    "second-halfword-lookup"
                },
            ));
        }
        for root in ADDITIONAL_ROOTS {
            spans.push((
                root,
                (0x0800_0000u32 + self.tables[1].source_offset)
                    .to_le_bytes()
                    .to_vec(),
                "second-lookup-root",
            ));
        }
        Ok(spans)
    }

    /// Physical source bytes only; guest stack indexing and control flow stay original.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, bool)> {
        for (i, table) in self.tables.iter().enumerate() {
            let Some(n) = offset.checked_sub(table.source_offset) else {
                continue;
            };
            if n < TABLE_BYTES as u32 {
                return Some((
                    table.values[(n / 2) as usize].to_le_bytes()[(n % 2) as usize],
                    i,
                    false,
                ));
            }
        }
        for (i, root) in ADDITIONAL_ROOTS.into_iter().enumerate() {
            let Some(n) = offset.checked_sub(root) else {
                continue;
            };
            if n < 4 {
                return Some((
                    (0x0800_0000u32 + self.tables[1].source_offset).to_le_bytes()[n as usize],
                    i,
                    true,
                ));
            }
        }
        None
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Copied halfword lookups differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: CopiedHalfwordLookups,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Copied halfword lookups require the exact EUR ROM; USA pending".into());
        }
        for (start, end, expected) in [
            (
                0x360c,
                0x3630,
                "eac5baa343548996c9b1bd9045336380404745cc74167a0d6901be81c6918a58",
            ),
            (
                0x3634,
                0x3658,
                "06a24b653133837699174017baa37359d3ec56729e9ab7f8f9193c263a688f16",
            ),
            (
                0x365c,
                0x3680,
                "45cd85d76d89ab4727cdf730347d83d114ff66a7937929162b0d2bcdd0fc00df",
            ),
            (
                0x147dc0,
                0x147e22,
                "aa822d873bad381a62e3c3e07b06003a41beea59bc3edc791f7e316eb08b7ebb",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Copied halfword lookup original reader mismatch".into());
            }
        }
        for (reader, literal, target) in [
            (0x3616u32, 0x3630u32, TABLE_OFFSETS[0]),
            (0x363e, 0x3658, TABLE_OFFSETS[1]),
            (0x3666, 0x3680, TABLE_OFFSETS[1]),
        ] {
            let opcode = u16::from_le_bytes(
                rom[reader as usize..reader as usize + 2]
                    .try_into()
                    .unwrap(),
            );
            let value = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if opcode & 0xf800 != 0x4800
                || ((reader + 4) & !3) + u32::from(opcode & 255) * 4 != literal
                || value != 0x0800_0000 + target
            {
                return Err("Copied halfword lookup original root mismatch".into());
            }
        }
        let tables = std::array::from_fn(|i| {
            let start = TABLE_OFFSETS[i] as usize;
            HalfwordTable {
                source_offset: TABLE_OFFSETS[i],
                values: std::array::from_fn(|j| {
                    u16::from_le_bytes(rom[start + j * 2..start + j * 2 + 2].try_into().unwrap())
                }),
            }
        });
        let decoded = CopiedHalfwordLookups { tables };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err(
                    "Copied halfword lookup source overlap or reconstruction mismatch".into(),
                );
            }
            let sha256 = ffta_rom::digest(&bytes);
            if let Some(i) = TABLE_OFFSETS.iter().position(|&base| base == offset) {
                if sha256 != SOURCE_HASHES[i] {
                    return Err("Copied halfword lookup source hash mismatch".into());
                }
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
