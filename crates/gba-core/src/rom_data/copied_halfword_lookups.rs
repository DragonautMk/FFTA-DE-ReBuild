//! EUR copied halfword lookups and an adjacent signed physical projection.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub mod initializer_bank;
pub mod initializer_variants;
pub mod initializer_pairs;
pub mod stack_byte_copies;
pub mod fixed_byte_copies;

pub const TABLE_OFFSETS: [u32; 3] = [0x14cf18, 0x14cf20, 0x14d84c];
pub const ADDITIONAL_ROOTS: [u32; 2] = [0x3658, 0x3680];
pub const TABLE_BYTES: usize = 8;
/// The root at 0f0558 is owned by the Thumb PC-literal resource.
pub const SOIL_EVIDENCE_OFFSET: u32 = 0x14d6d8;
pub const SOIL_EVIDENCE_ROOT: u32 = 0x0f0558;
pub const SOIL_EVIDENCE_SHA256: &str = "245bbd9d484dcf27c714e2690cd6544973de5d54aa9cd82eab23d6046a65faa8";
pub const SIGNED_PROJECTION_OFFSET: u32 = 0x14cf30;
pub const SIGNED_PROJECTION_VALUES: usize = 320;
pub const SIGNED_PROJECTION_BYTES: usize = SIGNED_PROJECTION_VALUES * 2;
pub const SIGNED_PROJECTION_SECOND_VIEW: u32 = SIGNED_PROJECTION_OFFSET + 0x80;
pub const SIGNED_PROJECTION_SHA256: &str = "dbdf49974fb8a6f319acc84a54ad90b3167aa588b2a3eafc62a6289c4d926205";
pub const STAR_CROSS_OFFSET: u32 = 0x14d69c;
pub const STAR_CROSS_BYTES: usize = 60;
pub const STAR_CROSS_SHA256: &str = "033f71729949bf130e095e99109ddd0f6ec77d293a948dc7a08fea784decec83";
pub const STAR_CROSS_ROOTS: [u32; 4] = [0x0eeb58, 0x0eeca4, 0x0eeca8, 0x0eecac];
const SOURCE_HASHES: [&str; 3] = [
    "2305fc2744df53d32db7b3015b54bef236ea42d3bc9eddf7e7f259b88b1613cb",
    "3d9f65f630695e727bd27f9037a20f7c693469ce276bbf5e17f5d8ad23f2d61b",
    "d89bb2241d6d843bd754c4c66c76c5fe5b419c25104a32bb352ec4c5872756d0",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HalfwordTable {
    pub source_offset: u32,
    pub values: [u16; 4],
}

/// Physical union of two low-byte indexed readers; no semantic extent is inferred.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedByteIndexedProjection {
    pub source_offset: u32,
    pub values: Vec<i16>,
}

/// Four source groups copied by the original Star Cross task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarCrossCopiedTables {
    pub source_offset: u32,
    pub first_coordinates: [i16; 8],
    pub second_coordinates: [i16; 8],
    pub copied_bytes: [u8; 4],
    pub comparisons: [u16; 12],
}

impl StarCrossCopiedTables {
    fn source_bytes(&self) -> Vec<u8> {
        self.first_coordinates.iter().flat_map(|value| value.to_le_bytes())
            .chain(self.second_coordinates.iter().flat_map(|value| value.to_le_bytes()))
            .chain(self.copied_bytes)
            .chain(self.comparisons.iter().flat_map(|value| value.to_le_bytes()))
            .collect()
    }

    fn byte_at(&self, n: usize) -> Option<u8> {
        match n {
            0..16 => Some(self.first_coordinates[n / 2].to_le_bytes()[n % 2]),
            16..32 => Some(self.second_coordinates[(n - 16) / 2].to_le_bytes()[n % 2]),
            32..36 => Some(self.copied_bytes[n - 32]),
            36..60 => Some(self.comparisons[(n - 36) / 2].to_le_bytes()[n % 2]),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopiedHalfwordLookups {
    pub tables: [HalfwordTable; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soil_evidence_table: Option<HalfwordTable>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_byte_index_projection: Option<SignedByteIndexedProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star_cross_tables: Option<StarCrossCopiedTables>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initializer_bank: Option<initializer_bank::InitializerBank>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initializer_variants: Option<initializer_variants::VariantBank>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initializer_pairs: Option<initializer_pairs::PairBank>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_byte_copies: Option<stack_byte_copies::StackByteCopies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_byte_copies: Option<fixed_byte_copies::FixedByteCopies>,
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
        if self.soil_evidence_table.as_ref().map(|table| table.source_offset) != Some(SOIL_EVIDENCE_OFFSET) {
            return Err("Invalid Soil Evidence copied halfword source".into());
        }
        if self.signed_byte_index_projection.as_ref().is_some_and(|projection| {
            projection.source_offset != SIGNED_PROJECTION_OFFSET
                || projection.values.len() != SIGNED_PROJECTION_VALUES
        }) {
            return Err("Invalid signed byte-indexed halfword projection".into());
        }
        if self.star_cross_tables.as_ref().is_some_and(|tables| tables.source_offset != STAR_CROSS_OFFSET) {
            return Err("Invalid Star Cross copied table source".into());
        }
        if let Some(bank) = &self.initializer_bank { bank.validate()?; }
        if let Some(bank) = &self.initializer_variants { bank.validate()?; }
        if let Some(bank) = &self.initializer_pairs { bank.validate()?; }
        if let Some(bank) = &self.stack_byte_copies { bank.validate()?; }
        if let Some(bank) = &self.fixed_byte_copies { bank.validate()?; }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        for (i, table) in self.tables.iter().take(2).enumerate() {
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
        let table = &self.tables[2];
        spans.push((
            table.source_offset,
            table.values.iter().flat_map(|value| value.to_le_bytes()).collect(),
            "third-halfword-lookup",
        ));
        let table = self.soil_evidence_table.as_ref().unwrap();
        spans.push((table.source_offset,
            table.values.iter().flat_map(|value| value.to_le_bytes()).collect(),
            "soil-evidence-halfword-lookup"));
        if let Some(projection) = &self.signed_byte_index_projection {
            spans.push((projection.source_offset,
                projection.values.iter().flat_map(|value| value.to_le_bytes()).collect(),
                "signed-byte-index-halfword-projection"));
        }
        if let Some(tables) = &self.star_cross_tables {
            spans.push((tables.source_offset, tables.source_bytes(), "star-cross-copied-tables"));
        }
        if let Some(bank) = &self.initializer_bank { spans.extend(bank.source_spans()); }
        if let Some(bank) = &self.initializer_variants { spans.extend(bank.source_spans()); }
        if let Some(bank) = &self.initializer_pairs { spans.extend(bank.source_spans()); }
        if let Some(bank) = &self.stack_byte_copies { spans.extend(bank.source_spans()); }
        if let Some(bank) = &self.fixed_byte_copies { spans.extend(bank.source_spans()); }
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
        if let Some(table) = self.soil_evidence_table.as_ref() {
            if let Some(n) = offset.checked_sub(table.source_offset) {
                if n < TABLE_BYTES as u32 {
                    return Some((table.values[(n / 2) as usize].to_le_bytes()[(n % 2) as usize], 3, false));
                }
            }
        }
        if let Some(projection) = self.signed_byte_index_projection.as_ref() {
            if let Some(n) = offset.checked_sub(projection.source_offset) {
                let index = (n / 2) as usize;
                if let Some(value) = projection.values.get(index) {
                    return Some((value.to_le_bytes()[(n % 2) as usize], 4, false));
                }
            }
        }
        if let Some(tables) = self.star_cross_tables.as_ref() {
            if let Some(n) = offset.checked_sub(tables.source_offset) {
                if let Some(byte) = tables.byte_at(n as usize) { return Some((byte, 5, false)); }
            }
        }
        if let Some((byte, root)) = self.initializer_bank.as_ref().and_then(|bank| bank.byte_at(offset)) {
            return Some((byte, if root { 2 } else { 6 }, root));
        }
        if let Some((byte, root)) = self.initializer_variants.as_ref().and_then(|bank| bank.byte_at(offset)) {
            return Some((byte, if root { 3 } else { 7 }, root));
        }
        if let Some((byte, root)) = self.initializer_pairs.as_ref().and_then(|bank| bank.byte_at(offset)) {
            return Some((byte, if root { 3 } else { 7 }, root));
        }
        if let Some((byte, root)) = self.stack_byte_copies.as_ref().and_then(|bank| bank.byte_at(offset)) {
            return Some((byte, if root { 3 } else { 7 }, root));
        }
        if let Some((byte, root)) = self.fixed_byte_copies.as_ref().and_then(|bank| bank.byte_at(offset)) {
            return Some((byte, if root { 3 } else { 7 }, root));
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
        if self != &Catalog::from_rom_with_initializers(
            rom, &[], self.initializer_bank.is_some(), self.initializer_variants.is_some(),
            self.initializer_pairs.is_some(), self.stack_byte_copies.is_some(),
            self.fixed_byte_copies.as_ref().map(fixed_byte_copies::FixedByteCopies::layout),
        )?.decoded {
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
        Self::from_rom_with_initializers(
            rom, previous, true, true, true, true,
            Some(fixed_byte_copies::Layout::WithSharedSlotLiterals),
        )
    }
    pub(crate) fn from_rom_with_initializers(
        rom: &[u8], previous: &[Resource], enabled: bool, variants_enabled: bool,
        pairs_enabled: bool, stack_bytes_enabled: bool, fixed_bytes: Option<fixed_byte_copies::Layout>,
    ) -> Result<Self, String> {
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
                0x109630,
                0x10963a,
                "71b8eee172a10bed08be917b72e18bc353a622d14100556943344b4714bbbf1b",
            ),
            (
                0x147dc0,
                0x147e22,
                "aa822d873bad381a62e3c3e07b06003a41beea59bc3edc791f7e316eb08b7ebb",
            ),
            (0x0f042c, 0x0f0436, "12450bc298e85922ede38e633cdf1651e8eb33caa9f9ea5fa8e73eb99cbe1831"),
            (0x0f04d2, 0x0f04dc, "88d40952b4a654aada37c2303601a8b07c0db893e6b2b6ac3ecc87cdbfcf9a77"),
            (0x7ab0, 0x7abe, "629bdda2d2dcf13d9c410e7abb281215a931ac2b84b42fdea9b162aaebc2908b"),
            (0x7ac4, 0x7ad4, "25317e5cbae12e515c2a432869dcee1b91242544b9d76fa55b57bf6d459ce137"),
            (0x0eeb0c, 0x0eeb88, "3f8e8092b8729df41b46a3149cbca412a1620c294d950707fd829783fcc42397"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Copied halfword lookup original reader mismatch".into());
            }
        }
        for (reader, literal, target) in [
            (0x3616u32, 0x3630u32, TABLE_OFFSETS[0]),
            (0x363e, 0x3658, TABLE_OFFSETS[1]),
            (0x3666, 0x3680, TABLE_OFFSETS[1]),
            (0x109630, 0x109710, TABLE_OFFSETS[2]),
            (0x0f042c, SOIL_EVIDENCE_ROOT, SOIL_EVIDENCE_OFFSET),
            (0x7ab2, 0x7ac0, SIGNED_PROJECTION_OFFSET),
            (0x7ac6, 0x7ad4, SIGNED_PROJECTION_OFFSET),
            (0x0eeb0e, STAR_CROSS_ROOTS[0], STAR_CROSS_OFFSET),
            (0x0eeb66, STAR_CROSS_ROOTS[1], STAR_CROSS_OFFSET + 16),
            (0x0eeb72, STAR_CROSS_ROOTS[2], STAR_CROSS_OFFSET + 32),
            (0x0eeb7e, STAR_CROSS_ROOTS[3], STAR_CROSS_OFFSET + 36),
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
        if !previous.is_empty() {
            let owner = previous.iter().find(|resource| resource.id == THUMB_PC_LITERALS_ID)
                .ok_or("Missing signed halfword projection literal owner")?;
            if [0x7ac0, 0x7ad4].into_iter().chain(STAR_CROSS_ROOTS).any(|root| {
                !owner.sources.iter().any(|source| source.offset == root && source.length == 4)
            }) {
                return Err("Missing signed halfword projection roots".into());
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
        let start = SOIL_EVIDENCE_OFFSET as usize;
        let soil_evidence_table = Some(HalfwordTable {
            source_offset: SOIL_EVIDENCE_OFFSET,
            values: std::array::from_fn(|i| u16::from_le_bytes(rom[start + i * 2..start + i * 2 + 2].try_into().unwrap())),
        });
        let start = SIGNED_PROJECTION_OFFSET as usize;
        let signed_byte_index_projection = Some(SignedByteIndexedProjection {
            source_offset: SIGNED_PROJECTION_OFFSET,
            values: (0..SIGNED_PROJECTION_VALUES).map(|i| {
                i16::from_le_bytes(rom[start + i * 2..start + i * 2 + 2].try_into().unwrap())
            }).collect(),
        });
        let start = STAR_CROSS_OFFSET as usize;
        let star_cross_tables = Some(StarCrossCopiedTables {
            source_offset: STAR_CROSS_OFFSET,
            first_coordinates: std::array::from_fn(|i| i16::from_le_bytes(rom[start + i * 2..start + i * 2 + 2].try_into().unwrap())),
            second_coordinates: std::array::from_fn(|i| i16::from_le_bytes(rom[start + 16 + i * 2..start + 18 + i * 2].try_into().unwrap())),
            copied_bytes: rom[start + 32..start + 36].try_into().unwrap(),
            comparisons: std::array::from_fn(|i| u16::from_le_bytes(rom[start + 36 + i * 2..start + 38 + i * 2].try_into().unwrap())),
        });
        let initializer_bank = if enabled { Some(initializer_bank::InitializerBank::from_rom(rom)?) } else { None };
        let initializer_variants = if variants_enabled { Some(initializer_variants::VariantBank::from_rom(rom)?) } else { None };
        let initializer_pairs = if pairs_enabled { Some(initializer_pairs::PairBank::from_rom(rom)?) } else { None };
        let stack_byte_copies = stack_bytes_enabled
            .then(|| stack_byte_copies::StackByteCopies::from_rom(rom))
            .transpose()?;
        let fixed_byte_copies = fixed_bytes
            .map(|layout| fixed_byte_copies::FixedByteCopies::from_rom(rom, layout))
            .transpose()?;
        let decoded = CopiedHalfwordLookups {
            tables, soil_evidence_table, signed_byte_index_projection, star_cross_tables,
            initializer_bank, initializer_variants, initializer_pairs, stack_byte_copies, fixed_byte_copies,
        };
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
            if offset == SOIL_EVIDENCE_OFFSET && sha256 != SOIL_EVIDENCE_SHA256 {
                return Err("Soil Evidence copied halfword source hash mismatch".into());
            }
            if offset == SIGNED_PROJECTION_OFFSET && sha256 != SIGNED_PROJECTION_SHA256 {
                return Err("Signed halfword projection source hash mismatch".into());
            }
            if offset == STAR_CROSS_OFFSET && sha256 != STAR_CROSS_SHA256 {
                return Err("Star Cross copied table source hash mismatch".into());
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
