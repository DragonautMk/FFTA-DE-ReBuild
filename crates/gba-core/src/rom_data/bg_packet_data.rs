//! Ordered descriptor-linked packet resources, not flattened images or live RAM.
//! Offline only; publishing and runtime consumption are separate integrations.
use super::{bg_descriptors, bg_packets, lz77, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Malformed-input guards, independent of any extracted-data space target.
pub const MAX_ENCODED_BYTES: usize = 64 * 1024;
pub const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_WRITTEN_BYTES: usize = 64 * 1024;
const MAX_PACKETS: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Paired,
    Halfwords,
    Auxiliary,
}
impl Role {
    pub const ALL: [Self; 3] = [Self::Paired, Self::Halfwords, Self::Auxiliary];
    pub fn name(self) -> &'static str {
        match self {
            Self::Paired => "paired",
            Self::Halfwords => "halfwords",
            Self::Auxiliary => "auxiliary",
        }
    }
    pub fn destination_bytes(self) -> usize {
        match self {
            Self::Paired => 0x4000,
            Self::Halfwords => 0x2000,
            Self::Auxiliary => 0x200,
        }
    }
    fn relative_source(self, descriptor: &bg_descriptors::Descriptor) -> u32 {
        match self {
            Self::Paired => descriptor.paired_planes_source,
            Self::Halfwords => descriptor.halfword_planes_source,
            Self::Auxiliary => descriptor.auxiliary_patches_source,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Writes {
    Paired(bg_packets::PlanePatches),
    Halfwords(bg_packets::Patches<u16>),
}
impl Writes {
    fn decode(body: &[u8], role: Role) -> Result<(Self, usize), String> {
        let limits = bg_packets::Limits {
            destination_bytes: role.destination_bytes(),
            written_bytes: MAX_WRITTEN_BYTES,
            packets: MAX_PACKETS,
        };
        match role {
            Role::Paired => {
                let value = bg_packets::decode_planes(body, limits)?;
                // The catalog establishes indexed calculated-plane writes only.
                value.calculated_plane_writes()?;
                let extent = value.referenced_extent;
                Ok((Self::Paired(value), extent))
            }
            Role::Halfwords | Role::Auxiliary => {
                let value = bg_packets::decode_halfwords(body, None, limits)?;
                let extent = value.consumed;
                Ok((Self::Halfwords(value), extent))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    /// Root-relative provenance, not a resource ID or an absolute host address.
    pub relative_source: u32,
    /// Original wrapper's upper24bits. None means the role's initial fill:
    /// live background word for paired planes, zero for the two other roles.
    pub parent_descriptor: Option<u16>,
    /// Exact consumed wrapper/stream metadata, never recompressed.
    pub encoded_source: Vec<u8>,
    /// Exact post-wrapper packet representation, including declared LZ padding.
    /// Keep control/index/marker bytes beside the decoded ordered writes.
    pub packet_body: Vec<u8>,
    pub writes: Writes,
}
impl Program {
    pub fn decode(input: &[u8], role: Role, relative_source: u32) -> Result<(Self, usize), String> {
        let input = &input[..input.len().min(MAX_ENCODED_BYTES)];
        let header = input.get(..4).ok_or("Truncated packet wrapper")?;
        let parent = u32::from_le_bytes(header.try_into().unwrap()) >> 8;
        let parent_descriptor = if parent == 0x00ff_ffff {
            None
        } else if parent < bg_descriptors::MAX_RECORDS as u32 {
            Some(parent as u16)
        } else {
            return Err("Packet parent descriptor exceeds its bound".into());
        };
        let (mut packet_body, encoded_payload_bytes) = match header[0] {
            1 => (input[4..].to_vec(), None),
            0x11 => {
                let value = lz77::decode(&input[4..], MAX_BODY_BYTES)?;
                (value.bytes, Some(value.consumed))
            }
            _ => return Err("Unsupported packet wrapper tag".into()),
        };
        let (writes, extent) = Writes::decode(&packet_body, role)?;
        if encoded_payload_bytes.is_none() {
            packet_body.truncate(extent);
        }
        let consumed = 4 + encoded_payload_bytes.unwrap_or(extent);
        Ok((
            Self {
                relative_source,
                parent_descriptor,
                encoded_source: input[..consumed].to_vec(),
                packet_body,
                writes,
            },
            consumed,
        ))
    }

    pub fn validate(&self, role: Role) -> Result<(), String> {
        if self.encoded_source.len() > MAX_ENCODED_BYTES || self.packet_body.len() > MAX_BODY_BYTES
        {
            return Err("Packet source/body exceeds its bound".into());
        }
        let (expected, consumed) = Self::decode(&self.encoded_source, role, self.relative_source)?;
        if consumed != self.encoded_source.len() || *self != expected {
            return Err("Packet wrapper, body, parent and decoded writes disagree".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Family {
    /// Descriptor IDs select program IDs; aliases retain every original link.
    /// Program IDs follow first descriptor occurrence, independent of ROM root.
    pub descriptor_programs: Vec<u16>,
    pub programs: Vec<Program>,
}
impl Family {
    pub fn program(&self, descriptor: usize) -> Option<&Program> {
        self.programs
            .get(*self.descriptor_programs.get(descriptor)? as usize)
    }

    /// Return the inherited programs before the child's writes. Initial fill
    /// remains a separate live operation, never a serialized flattened image.
    pub fn ordered_programs(&self, descriptor: usize) -> Result<Vec<&Program>, String> {
        if self.descriptor_programs.is_empty()
            || self.descriptor_programs.len() > bg_descriptors::MAX_RECORDS
        {
            return Err("Invalid packet descriptor count".into());
        }
        let mut visited = vec![false; self.descriptor_programs.len()];
        let mut current = Some(descriptor);
        let mut chain = Vec::new();
        while let Some(id) = current {
            let seen = visited
                .get_mut(id)
                .ok_or("Packet descriptor outside family")?;
            if *seen {
                return Err("Cyclic packet inheritance".into());
            }
            *seen = true;
            let program = self.program(id).ok_or("Packet program ID outside family")?;
            chain.push(program);
            current = program.parent_descriptor.map(usize::from);
        }
        chain.reverse();
        Ok(chain)
    }

    fn validate(&self, role: Role) -> Result<(), String> {
        let count = self.descriptor_programs.len();
        if count == 0
            || count > bg_descriptors::MAX_RECORDS
            || self.programs.is_empty()
            || self.programs.len() > count
        {
            return Err("Invalid packet family/program count".into());
        }
        let mut seen = vec![false; self.programs.len()];
        let mut next = 0;
        for (id, &index) in self.descriptor_programs.iter().enumerate() {
            let used = seen
                .get_mut(index as usize)
                .ok_or("Packet program ID outside family")?;
            if !*used {
                if index as usize != next {
                    return Err("Packet program IDs must follow first descriptor occurrence".into());
                }
                *used = true;
                next += 1;
            }
            self.ordered_programs(id)?;
        }
        if seen.contains(&false) {
            return Err("Unreferenced packet program".into());
        }
        for program in &self.programs {
            program.validate(role)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Programs {
    pub paired: Family,
    pub halfwords: Family,
    pub auxiliary: Family,
}
impl Programs {
    pub fn family(&self, role: Role) -> &Family {
        match role {
            Role::Paired => &self.paired,
            Role::Halfwords => &self.halfwords,
            Role::Auxiliary => &self.auxiliary,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let count = self.paired.descriptor_programs.len();
        let mut ranges = Vec::new();
        for role in Role::ALL {
            let family = self.family(role);
            if family.descriptor_programs.len() != count {
                return Err("Packet families have different descriptor counts".into());
            }
            family.validate(role)?;
            for program in &family.programs {
                let start = program.relative_source;
                if (start as usize) < count * bg_descriptors::RECORD_BYTES {
                    return Err("Packet source overlaps descriptor metadata".into());
                }
                let end = start
                    .checked_add(program.encoded_source.len() as u32)
                    .ok_or("Packet source extent overflow")?;
                ranges.push((start, end));
            }
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err("Overlapping or duplicate packet sources".into());
        }
        Ok(())
    }

    /// Matching a file's own hashes is insufficient for an immutable base set.
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Packet data differs from the authenticated source ROM".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Programs,
    /// Family-major/program-ID-major provenance; aliases stay in the link maps.
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let descriptors = bg_descriptors::Catalog::from_rom(rom)?;
        let root = descriptors.source.offset;
        let mut families = Vec::new();
        let mut sources = Vec::new();
        for role in Role::ALL {
            let mut ids = BTreeMap::new();
            let mut family = Family {
                descriptor_programs: Vec::new(),
                programs: Vec::new(),
            };
            for descriptor in &descriptors.decoded.entries {
                let relative = role.relative_source(descriptor);
                let index = if let Some(&index) = ids.get(&relative) {
                    index
                } else {
                    let offset = root.checked_add(relative).ok_or("Packet source overflow")?;
                    let input = rom
                        .get(offset as usize..)
                        .ok_or("Packet source outside ROM")?;
                    let (program, consumed) = Program::decode(input, role, relative)?;
                    let index = family.programs.len() as u16;
                    sources.push(SourceRange {
                        offset,
                        length: consumed as u32,
                        sha256: ffta_rom::digest(&input[..consumed]),
                        role: format!("bg-packet-program/{}/{index}", role.name()),
                    });
                    family.programs.push(program);
                    ids.insert(relative, index);
                    index
                };
                family.descriptor_programs.push(index);
            }
            families.push(family);
        }
        let [paired, halfwords, auxiliary]: [Family; 3] = families.try_into().unwrap();
        let decoded = Programs {
            paired,
            halfwords,
            auxiliary,
        };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn halfwords() -> Vec<u8> {
        // The second packet overwrites the first packet's second halfword.
        vec![
            0, 0, 4, 0, 0x34, 0x12, 1, 0x80, 2, 0, 2, 0, 0xff, 0xff, 0, 0,
        ]
    }
    fn pairs(wide: bool) -> Vec<u8> {
        let header = 0x8000_0010u32 | if wide { 0x4000_0000 } else { 0 };
        let mut body = header.to_le_bytes().to_vec();
        body.extend_from_slice(&[0, 0, 2, 0]);
        if wide {
            body.push(0);
        }
        body.push(1);
        if wide {
            body.push(0);
        }
        body.extend_from_slice(&[0, 0]);
        body.resize(16, 0xcc);
        body.extend_from_slice(&[0x12, 0, 0x34, 0x92, 0xcd, 0xab]);
        body
    }
    fn raw(body: &[u8], role: Role, source: u32) -> Program {
        let mut input = vec![1, 255, 255, 255];
        input.extend_from_slice(body);
        Program::decode(&input, role, source).unwrap().0
    }
    fn fixture() -> Programs {
        let family = |program| Family {
            descriptor_programs: vec![0, 0, 0],
            programs: vec![program],
        };
        Programs {
            paired: family(raw(&pairs(false), Role::Paired, 0x1000)),
            halfwords: family(raw(&halfwords(), Role::Halfwords, 0x2000)),
            auxiliary: family(raw(&halfwords(), Role::Auxiliary, 0x3000)),
        }
    }

    #[test]
    fn raw_halfwords_preserve_order_high_bits_and_exact_extent() {
        let value = raw(&halfwords(), Role::Halfwords, 0x1000);
        value.validate(Role::Halfwords).unwrap();
        let Writes::Halfwords(writes) = &value.writes else {
            panic!("halfwords");
        };
        assert_eq!(writes.packets[0].values, [0x1234, 0x8001]);
        assert_eq!(writes.packets[1].byte_offset, 2);
        assert_eq!(writes.packets[1].values, [0xffff]);
        assert_eq!(writes.consumed, 16);
        let mut padded = value.encoded_source.clone();
        padded.extend_from_slice(&[0xcc; 8]);
        let (decoded, consumed) = Program::decode(&padded, Role::Halfwords, 0x1000).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(consumed, 20);
        for end in 0..consumed {
            assert!(Program::decode(&padded[..end], Role::Halfwords, 0x1000).is_err());
        }
    }

    #[test]
    fn compressed_body_keeps_declared_padding_without_recompressing() {
        let mut body = halfwords();
        body.extend_from_slice(&[0xa1, 0xb2, 0xc3]);
        let mut encoded = vec![0x11, 255, 255, 255];
        encoded.extend_from_slice(&(((body.len() as u32) << 8) | 0x10).to_le_bytes());
        for group in body.chunks(8) {
            encoded.push(0);
            encoded.extend_from_slice(group);
        }
        let consumed = encoded.len();
        encoded.extend_from_slice(&[0xee; 4]);
        let (value, actual) = Program::decode(&encoded, Role::Halfwords, 0x1000).unwrap();
        assert_eq!(actual, consumed);
        assert_eq!(value.packet_body, body);
        assert_eq!(value.encoded_source, encoded[..consumed]);
        assert_eq!(
            value.writes,
            raw(&halfwords(), Role::Halfwords, 0x1000).writes
        );
        value.validate(Role::Halfwords).unwrap();
    }

    #[test]
    fn paired_sources_keep_dictionary_metadata_and_full_decoded_pairs() {
        for wide in [false, true] {
            let mut encoded = vec![1, 255, 255, 255];
            encoded.extend_from_slice(&pairs(wide));
            encoded.extend_from_slice(&[0xcc; 4]);
            let (value, consumed) = Program::decode(&encoded, Role::Paired, 0x1000).unwrap();
            assert_eq!(consumed, 26);
            assert_eq!(value.packet_body, pairs(wide));
            let Writes::Paired(writes) = &value.writes else {
                panic!("paired");
            };
            assert_eq!(writes.referenced_extent, 22);
            assert_eq!(writes.payload.packets[0].values, [0x0013_0012, 0x2bcd_1234]);
            value.validate(Role::Paired).unwrap();
        }
        // Literal syntax does not establish the original calculated destination.
        let literal = [0, 0, 0, 0, 0, 0, 4, 0, 1, 2, 3, 4, 0, 0];
        let mut source = vec![1, 255, 255, 255];
        source.extend_from_slice(&literal);
        assert!(Program::decode(&source, Role::Paired, 0x1000).is_err());
    }

    #[test]
    fn aliases_and_parent_chains_preserve_live_initialization_and_order() {
        let mut data = fixture();
        let mut child = raw(&pairs(true), Role::Paired, 0x1100);
        child.encoded_source[1..4].fill(0);
        child.parent_descriptor = Some(0);
        data.paired.programs.push(child);
        data.paired.descriptor_programs[1] = 1;
        data.validate().unwrap();
        assert!(std::ptr::eq(
            data.paired.program(0).unwrap(),
            data.paired.program(2).unwrap()
        ));
        let chain = data.paired.ordered_programs(1).unwrap();
        assert_eq!(
            chain.iter().map(|v| v.relative_source).collect::<Vec<_>>(),
            [0x1000, 0x1100]
        );
        for blank in [0u16, 0xffff] {
            let mut destination = vec![blank as u32 | ((blank as u32) << 16); 4];
            for program in &chain {
                let Writes::Paired(writes) = &program.writes else {
                    panic!("paired");
                };
                for packet in &writes.payload.packets {
                    let start = packet.byte_offset as usize / 4;
                    destination[start..start + packet.values.len()].copy_from_slice(&packet.values);
                }
            }
            assert_eq!(
                destination,
                [
                    0x0013_0012,
                    0x2bcd_1234,
                    blank as u32 | ((blank as u32) << 16),
                    blank as u32 | ((blank as u32) << 16)
                ]
            );
        }
        assert!(data.paired.ordered_programs(3).is_err());
    }

    #[test]
    fn inconsistent_models_graphs_roles_and_source_ranges_are_rejected() {
        let data = fixture();
        data.validate().unwrap();
        for edit in 0..10 {
            let mut bad = data.clone();
            match edit {
                0 => bad.paired.descriptor_programs.clear(),
                1 => bad.halfwords.descriptor_programs.pop().map(|_| ()).unwrap(),
                2 => bad.paired.descriptor_programs[0] = 1,
                3 => bad.paired.programs.push(bad.paired.programs[0].clone()),
                4 => {
                    bad.paired.programs[0].parent_descriptor = Some(0);
                    bad.paired.programs[0].encoded_source[1..4].fill(0);
                }
                5 => bad.halfwords.programs[0].packet_body[4] ^= 1,
                6 => bad.auxiliary.programs[0].relative_source = 0,
                7 => bad.auxiliary.programs[0].relative_source = 0x2000,
                8 => bad.paired.programs[0].relative_source = u32::MAX,
                9 => bad.paired.programs[0]
                    .encoded_source
                    .resize(MAX_ENCODED_BYTES + 1, 0),
                _ => unreachable!(),
            }
            assert!(bad.validate().is_err(), "edit={edit}");
        }
        let mut bad = data.halfwords.programs[0].clone();
        let Writes::Halfwords(writes) = &mut bad.writes else {
            unreachable!()
        };
        writes.packets[0].values[0] ^= 0x8000;
        assert!(bad.validate(Role::Halfwords).is_err());
        for tag in [0, 0x10, 0x12, 0xff] {
            let mut bad = data.halfwords.programs[0].encoded_source.clone();
            bad[0] = tag;
            assert!(Program::decode(&bad, Role::Halfwords, 0x1000).is_err());
        }
        let mut bad = data.halfwords.programs[0].encoded_source.clone();
        bad[1..4].copy_from_slice(&[0, 2, 0]); // parent512
        assert!(Program::decode(&bad, Role::Halfwords, 0x1000).is_err());
        let mut body = halfwords();
        body[..2].copy_from_slice(&0x0200u16.to_le_bytes());
        let mut bad = vec![1, 255, 255, 255];
        bad.extend_from_slice(&body);
        assert!(Program::decode(&bad, Role::Auxiliary, 0x1000).is_err());
    }
}
