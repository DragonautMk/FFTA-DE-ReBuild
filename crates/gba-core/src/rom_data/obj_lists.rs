//! Bounded EUR static OBJ lists from direct, observed and conditional indirect roots.
//! Animation state, OAM, matrices and lists constructed in RAM are not assets.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod roots;
pub use roots::REFERENCES;
pub mod observed;
pub mod constructor_animation;

#[derive(Clone, Copy, Debug)]
pub struct RootReference {
    pub reader: u32,
    pub literal: u32,
    pub call: u32,
    pub wrapper: u32,
    pub source_offset: u32,
}

/// The five transform bits retain either the affine index or flip/reserved bits.
/// Do not normalize them: their interpretation depends on affine_mode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub x: i16,
    pub y: i8,
    pub affine_mode: u8,
    pub object_mode: u8,
    pub mosaic: bool,
    pub color_256: bool,
    pub shape: u8,
    pub transform: u8,
    pub size: u8,
    pub tile: u16,
    pub priority: u8,
    pub palette: u8,
}

impl Object {
    pub fn from_attributes([a0, a1, a2]: [u16; 3]) -> Self {
        Self {
            x: ((a1 << 7) as i16) >> 7,
            y: a0 as i8,
            affine_mode: ((a0 >> 8) & 3) as u8,
            object_mode: ((a0 >> 10) & 3) as u8,
            mosaic: a0 & 0x1000 != 0,
            color_256: a0 & 0x2000 != 0,
            shape: (a0 >> 14) as u8,
            transform: ((a1 >> 9) & 31) as u8,
            size: (a1 >> 14) as u8,
            tile: a2 & 1023,
            priority: ((a2 >> 10) & 3) as u8,
            palette: (a2 >> 12) as u8,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(-256..=255).contains(&self.x)
            || self.affine_mode > 3
            || self.object_mode > 3
            || self.shape > 3
            || self.transform > 31
            || self.size > 3
            || self.tile > 1023
            || self.priority > 3
            || self.palette > 15
        {
            return Err("OBJ field exceeds its original bit width".into());
        }
        Ok(())
    }

    pub fn attributes(&self) -> [u16; 3] {
        [
            u16::from(self.y as u8)
                | (u16::from(self.affine_mode) << 8)
                | (u16::from(self.object_mode) << 10)
                | (u16::from(self.mosaic) << 12)
                | (u16::from(self.color_256) << 13)
                | (u16::from(self.shape) << 14),
            (self.x as u16 & 511) | (u16::from(self.transform) << 9) | (u16::from(self.size) << 14),
            self.tile | (u16::from(self.priority) << 10) | (u16::from(self.palette) << 12),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct List {
    /// Stable identity and source provenance; file offset, not a host pointer.
    pub source_offset: u32,
    pub objects: Vec<Object>,
}

impl List {
    pub fn decode(source_offset: u32, bytes: &[u8]) -> Result<Self, String> {
        if source_offset & 1 != 0 {
            return Err("Unaligned OBJ list source".into());
        }
        let header: [u8; 2] = bytes
            .get(..2)
            .ok_or("Missing OBJ list count")?
            .try_into()
            .unwrap();
        let count = u16::from_le_bytes(header) as usize;
        // Both original draw routines agree on these nonnegative counts. A
        // negative signed header has a distinct contract, outside this catalog.
        if count >= 32768
            || bytes.len() != 2 + count * 6
            || source_offset.checked_add(bytes.len() as u32).is_none()
        {
            return Err("Invalid OBJ list count/extent".into());
        }
        Ok(Self {
            source_offset,
            objects: bytes[2..]
                .chunks_exact(6)
                .map(|v| {
                    Object::from_attributes([
                        u16::from_le_bytes([v[0], v[1]]),
                        u16::from_le_bytes([v[2], v[3]]),
                        u16::from_le_bytes([v[4], v[5]]),
                    ])
                })
                .collect(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset & 1 != 0
            || self.objects.len() >= 32768
            || self
                .source_offset
                .checked_add(self.source_len() as u32)
                .is_none()
        {
            return Err("Invalid OBJ list offset/count/overflow".into());
        }
        for object in &self.objects {
            object.validate()?;
        }
        Ok(())
    }

    pub fn source_len(&self) -> usize {
        2 + self.objects.len() * 6
    }

    pub fn source_byte(&self, at: usize) -> Option<u8> {
        if at < 2 {
            return Some((self.objects.len() as u16).to_le_bytes()[at]);
        }
        let at = at - 2;
        let object = self.objects.get(at / 6)?;
        Some(object.attributes()[at % 6 / 2].to_le_bytes()[at % 2])
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok((0..self.source_len())
            .map(|at| self.source_byte(at).unwrap())
            .collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lists {
    pub lists: Vec<List>,
}

impl Lists {
    pub fn validate(&self) -> Result<(), String> {
        let mut end = 0;
        for list in &self.lists {
            list.validate()?;
            if list.source_offset < end {
                return Err("Overlapping or unordered OBJ lists".into());
            }
            end = list.source_offset + list.source_len() as u32;
        }
        Ok(())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("OBJ lists differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Lists,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Static OBJ lists require exact EUR ROM; USA unverified".into());
        }
        let word = |at: u32| -> Result<u32, String> {
            let bytes = rom
                .get(at as usize..at as usize + 4)
                .ok_or("Truncated OBJ root literal/call")?;
            Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
        };
        let mut roots = std::collections::BTreeSet::new();
        for reference in REFERENCES {
            let opcode = word(reference.reader)? as u16;
            let literal = ((reference.reader + 4) & !3) + u32::from(opcode & 255) * 4;
            let call = word(reference.call)?;
            let hi = call as u16;
            let lo = (call >> 16) as u16;
            let displacement = ((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32;
            let displacement = (displacement << 9) >> 9;
            let target = reference
                .call
                .wrapping_add(4)
                .wrapping_add_signed(displacement);
            if opcode & 0xf800 != 0x4800
                || literal != reference.literal
                || word(literal)? != 0x08000000 + reference.source_offset
                || hi & 0xf800 != 0xf000
                || lo & 0xf800 != 0xf800
                || target != reference.wrapper
            {
                return Err("OBJ static root reader/literal/call mismatch".into());
            }
            roots.insert(reference.source_offset);
        }
        let mut lists = Vec::new();
        let mut sources = Vec::new();
        for offset in roots {
            let count = word(offset)? as u16 as usize;
            let length = 2 + count * 6;
            let bytes = rom
                .get(offset as usize..offset as usize + length)
                .ok_or("Truncated static OBJ list")?;
            let list = List::decode(offset, bytes)?;
            if list.objects.iter().any(|o| o.shape >= 3) {
                return Err("Declared static OBJ catalog has an unavailable shape".into());
            }
            lists.push(list);
            sources.push(SourceRange {
                offset,
                length: length as u32,
                sha256: ffta_rom::digest(bytes),
                role: "static-obj-list".into(),
            });
        }
        // The two additional identities have observed indirect provenance; do
        // not pretend they are direct RootReferences or infer a whole archive.
        let extra = observed::Catalog::from_rom(rom)?;
        lists.extend(extra.decoded.lists);
        sources.extend(extra.sources);
        // Four conditional pointer-table roots retain their indirect provenance.
        let animation = constructor_animation::Catalog::from_rom(rom)?;
        lists.extend(animation.lists.lists);
        sources.extend(animation.sources.into_iter().filter(|s| s.role == "static-obj-list"));
        lists.sort_by_key(|list| list.source_offset);
        sources.sort_by_key(|source| source.offset);
        let decoded = Lists { lists };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn object_fields_list_extents_and_catalog_guards() {
        // Independently vary every packed bit, including negative coordinates,
        // affine/flip bits, reserved modes and unavailable shape 3.
        for word in 0..=u16::MAX {
            let attrs = [word, word.rotate_left(5), word.rotate_left(11)];
            let object = Object::from_attributes(attrs);
            object.validate().unwrap();
            assert_eq!(object.attributes(), attrs);
        }
        let mut bytes = 1u16.to_le_bytes().to_vec();
        bytes.extend([0xff, 0xff, 0, 0x81, 0x34, 0xf2]);
        let list = List::decode(0x100, &bytes).unwrap();
        assert_eq!(list.objects[0].x, -256);
        assert_eq!(list.objects[0].y, -1);
        assert_eq!(list.source_bytes().unwrap(), bytes);
        assert!(list.source_byte(8).is_none());
        for len in 0..bytes.len() {
            assert!(List::decode(0x100, &bytes[..len]).is_err());
        }
        assert!(List::decode(0x101, &bytes).is_err());
        assert!(List::decode(u32::MAX - 1, &bytes).is_err());
        assert!(List::decode(0, &[0, 0x80]).is_err());
        let empty = List::decode(0x102, &[0, 0]).unwrap();
        assert_eq!(empty.source_bytes().unwrap(), [0, 0]);
        assert!(Lists {
            lists: vec![list.clone(), empty]
        }
        .validate()
        .is_err());
        let mut invalid = list;
        invalid.objects[0].x = 256;
        assert!(invalid.validate().is_err());
        assert!(Catalog::from_rom(&bytes).is_err());
    }
}
