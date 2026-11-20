//! Conditional primary-task alias of the compact OBJ selector and position.
use super::{
    compact_obj_catalog::CompactObjCatalog,
    script39_registry_poses::PriorSources,
    task_descriptors::{primary_source_offset, TaskDescriptors},
    Resource, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const LOOKUP_OFFSET: u32 = 0x3aba8b;
pub const POSITION_OFFSET: u32 = 0x3abd78;
const LOOKUP_SHA256: &str = "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d";
const POSITION_SHA256: &str = "9d9f290527a6be626a8f5985b26e19b237b44872b03631811df4416fc1713178";

/// The guest may select other indices; these fields own only the observed alias.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactPrimaryAlias {
    pub lookup_offset: u32,
    pub position_offset: u32,
    pub selector: u8,
    pub x: i16,
    pub y: i16,
}

impl CompactPrimaryAlias {
    pub fn validate(&self) -> Result<(), String> {
        if self.lookup_offset != LOOKUP_OFFSET || self.position_offset != POSITION_OFFSET {
            return Err("Invalid compact primary alias source offsets".into());
        }
        Ok(())
    }

    pub fn validate_dependencies(
        &self,
        tasks: &TaskDescriptors,
        catalog: &CompactObjCatalog,
    ) -> Result<(), String> {
        self.validate()?;
        let descriptor = tasks
            .descriptor_at(primary_source_offset(362))
            .ok_or("Missing primary task 362 descriptor")?;
        if descriptor.callback_word() != Some(0x0811_eaa9) || descriptor.context_size != 68 {
            return Err("Primary task alias callback mismatch".into());
        }
        let bank = catalog
            .banks
            .iter()
            .find(|bank| bank.root == 0x6cb278)
            .ok_or("Missing compact primary alias bank")?;
        if bank.sequence(u16::from(self.selector)).is_none() {
            return Err("Compact primary alias selector is not in the dependent bank".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut position = self.x.to_le_bytes().to_vec();
        position.extend(self.y.to_le_bytes());
        Ok(vec![
            (
                LOOKUP_OFFSET,
                vec![self.selector],
                "primary-slot-255-selector",
            ),
            (POSITION_OFFSET, position, "primary-slot-255-position"),
        ])
    }
}

pub struct Catalog {
    pub decoded: CompactPrimaryAlias,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Compact primary alias requires the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (
                0xd7018,
                0xd70ca,
                "23c9fb5d4013242673d8be2f2cc43e1b4ab8c11c27ac68965753e21f4045b254",
            ),
            (
                0x11eaa8,
                0x11eb16,
                "dc3d9a1ffa833e89409ceda723644f31746bd5dec021f212b0932b28cc6d3ae5",
            ),
            (
                0x11eb26,
                0x11eba6,
                "7ea620c0e53db85a95f1684710699f2dc10f1386e08aee08c8db331ed9fdf323",
            ),
            (
                0x11ebf4,
                0x11ec10,
                "684d79714e13c3ae051b7cb966c0f0c93ee29a84c49cfe2369f0c4c5fc377df3",
            ),
            (
                0x39a2f8,
                0x39a318,
                "e7a4c03b58d4878ca50726defef9ed56a286709cea7c346924e49cf4e03f5025",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Compact primary alias original reader mismatch".into());
            }
        }
        let lookup = LOOKUP_OFFSET as usize;
        let position = POSITION_OFFSET as usize;
        if ffta_rom::digest(&rom[lookup..lookup + 1]) != LOOKUP_SHA256
            || ffta_rom::digest(&rom[position..position + 4]) != POSITION_SHA256
        {
            return Err("Compact primary alias source hash mismatch".into());
        }
        let decoded = CompactPrimaryAlias {
            lookup_offset: LOOKUP_OFFSET,
            position_offset: POSITION_OFFSET,
            selector: rom[lookup],
            x: i16::from_le_bytes(rom[position..position + 2].try_into().unwrap()),
            y: i16::from_le_bytes(rom[position + 2..position + 4].try_into().unwrap()),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Compact primary alias source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Compact primary alias source reconstruction mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
