//! Bounded EUR static OBJ lists from direct, observed and conditional indirect roots.
//! Animation state, OAM, matrices and lists constructed in RAM are not assets.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod roots;
pub use roots::REFERENCES;
pub mod observed;
pub mod constructor_animation;
pub mod conditional_markers;
pub mod context_selector;

pub const ACTOR_LOOKUP_ROOT: u32 = 0x398df0;
pub const TITLE_OPENING_DIRECTORY_ROOT: u32 = 0x3ade64;
pub const TITLE_OPENING_DIRECTORY_COUNT: usize = 19;
pub const BATTLE_DIRECTORY_ROOT: u32 = 0x40d300;
pub const BATTLE_DIRECTORY_COUNT: usize = 103;
pub const BATTLE_DIRECTORY_SHA256: &str = "2c35f140bd943bb98726767c2a4032be05dee531b2c965b727a3dce867cd27da";

/// Bounded relative index used by the observed battle draw producer. Its
/// targets are not necessarily all classified OBJ payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleDirectory {
    pub source_offset: u32,
    pub kind: u8,
    pub opaque_header_byte: u8,
    pub offsets: Vec<u16>,
}

impl BattleDirectory {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != BATTLE_DIRECTORY_ROOT || self.kind != 2 || self.offsets.len() != BATTLE_DIRECTORY_COUNT {
            return Err("Invalid battle relative directory".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = vec![self.kind, self.opaque_header_byte];
        out.extend_from_slice(&(self.offsets.len() as u16).to_le_bytes());
        for value in &self.offsets { out.extend_from_slice(&value.to_le_bytes()); }
        Ok(out)
    }

    pub fn source_byte(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.source_offset)? as usize;
        match relative {
            0 => Some(self.kind),
            1 => Some(self.opaque_header_byte),
            2..=3 => Some((self.offsets.len() as u16).to_le_bytes()[relative - 2]),
            _ => self.offsets.get((relative - 4) / 2).map(|v| v.to_le_bytes()[(relative - 4) % 2]),
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle_directory: Option<BattleDirectory>,
    /// Two observed unsigned lookup entries for the Llednar actor callback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_lookup: Option<[u16; 2]>,
    /// Absolute ROM pointers copied into the EUR title-opening records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_opening_directory: Option<Vec<u32>>,
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
        if let Some(directory) = &self.battle_directory {
            directory.validate()?;
            let start = directory.source_offset;
            let end = start + (4 + directory.offsets.len() * 2) as u32;
            if self.lists.iter().any(|list| list.source_offset < end && start < list.source_offset + list.source_len() as u32) {
                return Err("Battle directory overlaps an OBJ list".into());
            }
        }
        if self.actor_lookup.is_some() && self.lists.iter().any(|list|
            list.source_offset < ACTOR_LOOKUP_ROOT + 4 && ACTOR_LOOKUP_ROOT < list.source_offset + list.source_len() as u32) {
            return Err("Actor lookup overlaps an OBJ list".into());
        }
        if let Some(directory) = &self.title_opening_directory {
            if directory.len() != TITLE_OPENING_DIRECTORY_COUNT || directory.iter().any(|&pointer| {
                pointer.checked_sub(0x0800_0000).and_then(|offset| {
                    self.lists.binary_search_by_key(&offset, |list| list.source_offset).ok()
                }).is_none()
            }) {
                return Err("Invalid title opening OBJ directory".into());
            }
            let end = TITLE_OPENING_DIRECTORY_ROOT + (TITLE_OPENING_DIRECTORY_COUNT * 4) as u32;
            if self.lists.iter().any(|list| list.source_offset < end
                && TITLE_OPENING_DIRECTORY_ROOT < list.source_offset + list.source_len() as u32) {
                return Err("Title opening directory overlaps an OBJ list".into());
            }
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
        // These two indirect roots are selected through RAM slots by the
        // original writer; they are not direct RootReferences.
        let markers = conditional_markers::Catalog::from_rom(rom)?;
        lists.extend(markers.decoded.lists);
        sources.extend(markers.sources);
        // Four lists selected by the context constructor have their own
        // bounded selector proof; they are not direct RootReferences.
        let context = context_selector::Catalog::from_rom(rom)?;
        lists.extend(context.decoded.lists);
        sources.extend(context.sources);
        // This battle constructor passes one independent list through the
        // previously authenticated copied constructor/reader bridge.
        const BATTLE_ROOT: u32 = 0x395088;
        const BATTLE_HASH: &str = "c685f8be3478ab57ac1cd59834466c36a155553f5b1ce40933c45a5d932c1459";
        if ffta_rom::digest(&rom[0xbdfce..0xbdffc]) != "c887064f853182fcd06e535648f6a32dd24546d7c8ecabaed1b9df5cb8626e4f"
            || word(0xbe054)? != 0x0800_0000 + BATTLE_ROOT
            || word(0xbe048)? != 0x0839_7f9c || word(0x397f9c)? != 0x0300_6b6d {
            return Err("Battle OBJ constructor identity mismatch".into());
        }
        let bytes = &rom[BATTLE_ROOT as usize..BATTLE_ROOT as usize + 14];
        if ffta_rom::digest(bytes) != BATTLE_HASH { return Err("Battle OBJ source identity mismatch".into()); }
        let battle = List::decode(BATTLE_ROOT, bytes)?;
        if battle.objects.len() != 2 || battle.objects.iter().any(|object| object.shape >= 3) {
            return Err("Battle OBJ list count or shape mismatch".into());
        }
        lists.push(battle);
        sources.push(SourceRange { offset: BATTLE_ROOT, length: 14, sha256: BATTLE_HASH.into(), role: "static-obj-list".into() });
        // One independently observed one-object battle draw list. The
        // pointer producer remains outside this bounded source contract.
        const LLEDNAR_DRAW_ROOT: u32 = 0x395014;
        const LLEDNAR_DRAW_HASH: &str = "33c1fb663407cb81224d08192f2ae2cdab42656ec86bfbdc62784880d406cd08";
        let bytes = &rom[LLEDNAR_DRAW_ROOT as usize..LLEDNAR_DRAW_ROOT as usize + 8];
        if ffta_rom::digest(bytes) != LLEDNAR_DRAW_HASH { return Err("Llednar OBJ source identity mismatch".into()); }
        let llednar = List::decode(LLEDNAR_DRAW_ROOT, bytes)?;
        if llednar.objects.len() != 1 || llednar.objects[0].shape >= 3 { return Err("Llednar OBJ list count or shape mismatch".into()); }
        lists.push(llednar);
        sources.push(SourceRange { offset: LLEDNAR_DRAW_ROOT, length: 8, sha256: LLEDNAR_DRAW_HASH.into(), role: "static-obj-list".into() });
        // The established original draw reader was observed consuming this
        // distinct list in battle; its pointer producer is not yet known.
        const DRAW_ROOT: u32 = 0x40dcba;
        const DRAW_HASH: &str = "68880753b277e373d5bb81345fa01ce63650fd5e6c908c845dd1dd8f9b956715";
        let bytes = &rom[DRAW_ROOT as usize..DRAW_ROOT as usize + 50];
        if ffta_rom::digest(bytes) != DRAW_HASH { return Err("Observed battle draw source identity mismatch".into()); }
        let draw = List::decode(DRAW_ROOT, bytes)?;
        if draw.objects.len() != 8 || draw.objects.iter().any(|object| object.shape >= 3) {
            return Err("Observed battle draw list count or shape mismatch".into());
        }
        lists.push(draw);
        sources.push(SourceRange { offset: DRAW_ROOT, length: 50, sha256: DRAW_HASH.into(), role: "static-obj-list".into() });
        for (start, end, hash) in [
            (0x720c, 0x724c, "9e796bc8faa44d910a219471adb5dc1bcbea39706d9d0fb55b4737785d859c06"),
            (0xcf358, 0xcf398, "6d198b27bcb2b3bd50764861b3a671dde87292aeda76147e8340afab0c6fe469"),
            (0x2e10a, 0x2e112, "8120a071154079a121ee1c1cea5d702b2bb77f1deb45ef7ea19c4bfe676544d3"),
            (0x2e0f0, 0x2e114, "2a8f4b502280e8087c0287c202c87d5cddbf45c652f3d8293fb12cea381e7331"),
            (0x2e324, 0x2e370, "3975ae8e5a0dafd31589b52eaae77754a3ab54169081f3e44189970014bbd420"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash { return Err("Battle directory reader identity mismatch".into()); }
        }
        if word(0xcf378)? != 0x0800_0000 + BATTLE_DIRECTORY_ROOT { return Err("Battle directory literal mismatch".into()); }
        let at = BATTLE_DIRECTORY_ROOT as usize;
        let raw = &rom[at..at + 4 + 2 * BATTLE_DIRECTORY_COUNT];
        if ffta_rom::digest(raw) != BATTLE_DIRECTORY_SHA256 || raw[0] != 2
            || u16::from_le_bytes([raw[2], raw[3]]) as usize != BATTLE_DIRECTORY_COUNT {
            return Err("Battle directory source identity mismatch".into());
        }
        let battle_directory = Some(BattleDirectory {
            source_offset: BATTLE_DIRECTORY_ROOT,
            kind: raw[0],
            opaque_header_byte: raw[1],
            offsets: raw[4..].chunks_exact(2).map(|v| u16::from_le_bytes([v[0], v[1]])).collect(),
        });
        sources.push(SourceRange { offset: BATTLE_DIRECTORY_ROOT, length: raw.len() as u32,
            sha256: BATTLE_DIRECTORY_SHA256.into(), role: "battle-relative-directory".into() });
        // Conditional task+24 -> original draw path: preserve all 103
        // independently counted list extents, sharing existing exact owners.
        let mut all_list_bytes = Vec::new();
        let mut new_list_bytes = Vec::new();
        let mut shared_indices = Vec::new();
        for (index, &relative) in battle_directory.as_ref().unwrap().offsets.iter().enumerate() {
            let offset = BATTLE_DIRECTORY_ROOT + 4 + u32::from(relative);
            let start = offset as usize;
            let count = u16::from_le_bytes(rom.get(start..start + 2).ok_or("Truncated battle list count")?.try_into().unwrap()) as usize;
            let end = start.checked_add(2 + count * 6).ok_or("Battle list extent overflow")?;
            let bytes = rom.get(start..end).ok_or("Truncated battle directory list")?;
            let list = List::decode(offset, bytes)?;
            if list.objects.is_empty() || list.objects.iter().any(|object| object.shape >= 3) {
                return Err("Invalid battle directory list count or shape".into());
            }
            all_list_bytes.extend_from_slice(bytes);
            if let Some(existing) = lists.iter().find(|prior| prior.source_offset == offset) {
                if existing != &list { return Err("Conflicting shared battle list".into()); }
                shared_indices.push(index);
            } else {
                new_list_bytes.extend_from_slice(bytes);
                sources.push(SourceRange { offset, length: bytes.len() as u32,
                    sha256: ffta_rom::digest(bytes), role: "static-obj-list".into() });
                lists.push(list);
            }
        }
        if shared_indices != [32, 62]
            || ffta_rom::digest(&all_list_bytes) != "bfcadc626d308693563396b9030f7ae097895c64b491467b90b01aca256f62ef"
            || ffta_rom::digest(&new_list_bytes) != "934b25f8b031dbd8282f69f30d121c6f55ad7c25b25e9bbee12c11a7f9142973" {
            return Err("Battle directory list source projection mismatch".into());
        }
        // Only the two entries observed in the actor callback are declared.
        // The signed guest index is not capped or normalized by this fragment.
        for (start, end, hash) in [
            (0xa3070, 0xa3080, "d4e671f2f2e3a848368c21ef842d7fd6d35efb838825ddf5e4416035214cef8a"),
            (0x9b1b2, 0x9b1dc, "93fdd497505fc875b00e1a3bd3e4d4b3cfc8d2e085e1ac71fc9dcdfe3b480a79"),
            (0x22e7c, 0x22e82, "e9808aad624390bf678bb5e1ba63f6c03e7339a4370e944f33a6c8ee9ce802ff"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash { return Err("Actor lookup reader identity mismatch".into()); }
        }
        let raw = &rom[ACTOR_LOOKUP_ROOT as usize..ACTOR_LOOKUP_ROOT as usize + 4];
        if ffta_rom::digest(raw) != "f86c411fac6d8f5ffe1345cc245d732153d1193907d75e9f6b9a5ce46f9f5965" {
            return Err("Actor lookup source identity mismatch".into());
        }
        let actor_lookup = Some([u16::from_le_bytes([raw[0], raw[1]]), u16::from_le_bytes([raw[2], raw[3]])]);
        sources.push(SourceRange { offset: ACTOR_LOOKUP_ROOT, length: 4, sha256: ffta_rom::digest(raw), role: "actor-lookup".into() });
        // The original initializer copies these absolute pointers into 19
        // title records. The eleven distinct targets are counted OBJ lists.
        if ffta_rom::digest(&rom[0x13a1e0..0x13a274]) != "3f65e94b48916c97b18030f28a12ebd4a5b9aad20a47d653ce0856a40c7379ce"
            || ffta_rom::digest(&rom[0x13a5e8..0x13a64e]) != "09093f9997aa760aa3d36a4a68d14db5b7282faa17123a73c87cccbd34da8218"
            || word(0x13a28c)? != 0x0800_0000 + TITLE_OPENING_DIRECTORY_ROOT {
            return Err("Title opening OBJ reader identity mismatch".into());
        }
        let raw = &rom[TITLE_OPENING_DIRECTORY_ROOT as usize..TITLE_OPENING_DIRECTORY_ROOT as usize + TITLE_OPENING_DIRECTORY_COUNT * 4];
        if ffta_rom::digest(raw) != "350064eeea9a5cdd4b984762c57d5f5b05cadfc56311d904cc0463ca4f0f856c"
            || ffta_rom::digest(&rom[0x3ade0a..0x3ade62]) != "988bfca1254d75c8ab6e785da80e0d010295b8453df72e62d15aa7566080b953" {
            return Err("Title opening OBJ source identity mismatch".into());
        }
        let title_opening_directory: Vec<u32> = raw.chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap())).collect();
        let targets: std::collections::BTreeSet<u32> = title_opening_directory.iter()
            .map(|pointer| pointer.checked_sub(0x0800_0000).ok_or("Invalid title opening OBJ pointer"))
            .collect::<Result<_, _>>()?;
        if targets != (0..11).map(|i| 0x3ade0a + i * 8).collect() {
            return Err("Title opening OBJ targets mismatch".into());
        }
        for offset in targets {
            let bytes = &rom[offset as usize..offset as usize + 8];
            let list = List::decode(offset, bytes)?;
            if list.objects.len() != 1 || list.objects[0].shape >= 3 {
                return Err("Invalid title opening OBJ list".into());
            }
            sources.push(SourceRange { offset, length: 8, sha256: ffta_rom::digest(bytes), role: "static-obj-list".into() });
            lists.push(list);
        }
        sources.push(SourceRange { offset: TITLE_OPENING_DIRECTORY_ROOT, length: raw.len() as u32,
            sha256: ffta_rom::digest(raw), role: "title-opening-obj-directory".into() });
        lists.sort_by_key(|list| list.source_offset);
        sources.sort_by_key(|source| source.offset);
        let decoded = Lists { lists, battle_directory, actor_lookup, title_opening_directory: Some(title_opening_directory) };
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
            lists: vec![list.clone(), empty],
            battle_directory: None,
            actor_lookup: None, title_opening_directory: None,
        }
        .validate()
        .is_err());
        let mut invalid = list;
        invalid.objects[0].x = 256;
        assert!(invalid.validate().is_err());
        assert!(Catalog::from_rom(&bytes).is_err());
    }
}
