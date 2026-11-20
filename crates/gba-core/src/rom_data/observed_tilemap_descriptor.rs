//! Authenticated EUR descriptors and their linked LZ wrappers.
use super::{observed_lz, observed_tilemap, script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_LZ_ID, OBSERVED_TILEMAP_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3a4b70;
pub const SOURCE_LENGTH: usize = 8;
pub const SOURCE_SHA256: &str = "1a6f836689bf342c63de1ecab160ca4e134120ae3670782998696e02242f7a05";
pub const WRAPPER_LENGTH: usize = 4;
pub const TILEMAP_DESCRIPTOR_SHA256: [&str; 7] = [
    "8a85155dd9c49f25a62e317de1bea1c16408eb8e49263a869094b18e8886b410",
    "b2ac9e364624ab0bdc38e744ce75ba7c63ac3bfcdeea801018ffe5fb03c27c82",
    "0ece24698c19ab28faa9dfec6fa1bd7348854370b408a09822edd4edd159fd90",
    "b95f3c18219c1247db0128ccecee92a112e15408be4578f0cc8f59bf043e848d",
    "ad040f4652914f454b00b2023a34034a413dcebd8c6bffc537d9623c225fef0c",
    "f93fe3088e0367691e98d3341fade76895e033a95351e2d7ea6e3a8bdb3e21c2",
    "2d54e7fb29569f3d12318bb15bcd36f1080f278fad9fb8fdc0de68d867a8d538",
];
/// Selected descriptor and wrapper roots; the compressed streams have a separate owner.
pub const ADDITIONAL_SPECS: [(u32, &str, u32, &str); 4] = [
    (0x3a4b18, "4d41b742909b6b74166dc4a9da10958203010961d902f1a72e995cf81d3530e0", 0xb0e2b4, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b20, "cfb5f0da300004be29361875fbeecfc9b1fe02b76412d676c1c480b42841163a", 0xb0dd58, "6554507a2423f81b672d1cc607ed142449ed621e72ec74dec669d31b7730951e"),
    (0x3a4b28, "6e4e1c13d6a976ba67b150ba1424322c386681fa69f51430b62ade63302a66d4", 0xb0df28, "6554507a2423f81b672d1cc607ed142449ed621e72ec74dec669d31b7730951e"),
    (0x3a4b30, "a5ba89936790a9e6f704142475f31735deace17a82b42fc2cdfdccae5242177d", 0xb0eccc, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
];
/// Transfer descriptors and wrappers selected by the Red Spring rows.
pub const TRANSFER_SPECS: [(u32, &str, u32, &str); 3] = [
    (0x399774, "1e2b07e65c23762cf89bc63f87c23f78833cd01bd99c60f97efafcee05369b5c", 0xb02ea0, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x39977c, "4463dd11a5014a50c1be9712f36000206dc8c7699ab18b50ffe6e5ad63936828", 0xb030bc, "8a53a04d258d253d29a4035818bbe3fb8130138a0fe327563e42e0ff020de37e"),
    (0x399784, "1828ce907e40e0fc7041797a08cb7e7fa37f49d2a39bc5d84645c676d3542c8d", 0xb151d4, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
];
/// Seven LZ descriptors paired with the E200 direct tilemap rows.
pub const INTERLEAVED_LZ_SPECS: [(u32, &str, u32, &str); 7] = [
    (0x3a4b38, "0677d43723a8f37de758cfa6ebceeeb39ba58b5f5b2c08b9612aaa2f5f3127fa", 0xb0efd8, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b40, "fc9c0e60d21317c0ca8d7af6eb147395a95ba7b97229faf8ed29296e0a8d2db7", 0xb0f1b8, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b48, "2326d75fda5cef0c9e6110c31632af0819998e2396577c13e63f81ae0611d6c8", 0xb0f4d4, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b50, "bf1dd9128894b565b215d4d33cf39edae67b58e4b09f68a81000b39564ca5b1a", 0xb0f75c, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b58, "b4d74acbb476e8a269b8f66ad3d130bae42eef180af75b241b2a8b879db40617", 0xb0fa10, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b60, "840b0825bb1fb0deb70fe9bef3f5dd067d979efc96c2ebd557417a9c471928d4", 0xb0fbd8, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
    (0x3a4b68, "eac3397f8eae08655b6b39d7f0ca7ced172653a6e9827a6dbbd452bf9132631b", 0xb0fdec, "94c10d40b71da4f993f690c1fde54c00d00d0767c916543d093e93df848b8c93"),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LzWrapper {
    pub source_offset: u32,
    pub kind: u8,
    pub flags: u8,
    pub opaque_halfword: u16,
}

impl LzWrapper {
    pub fn source_bytes(&self) -> [u8; WRAPPER_LENGTH] {
        let [lo, hi] = self.opaque_halfword.to_le_bytes();
        [self.kind, self.flags, lo, hi]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalDescriptor {
    pub source_offset: u32,
    pub asset_pointer: u32,
    pub parameter16: u16,
    pub parameter8: u8,
    pub opaque_byte: u8,
    pub wrapper: LzWrapper,
}

impl AdditionalDescriptor {
    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        descriptor_bytes(self.asset_pointer, self.parameter16, self.parameter8, self.opaque_byte)
    }
}

fn descriptor_bytes(pointer: u32, parameter16: u16, parameter8: u8, opaque: u8) -> [u8; SOURCE_LENGTH] {
    let mut bytes = [0; SOURCE_LENGTH];
    bytes[..4].copy_from_slice(&pointer.to_le_bytes());
    bytes[4..6].copy_from_slice(&parameter16.to_le_bytes());
    bytes[6] = parameter8;
    bytes[7] = opaque;
    bytes
}

/// A direct tilemap descriptor; its parameters remain guest-interpreted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectTilemapDescriptor {
    pub source_offset: u32,
    pub tilemap_pointer: u32,
    pub parameter16: u16,
    pub parameter8: u8,
    pub opaque_byte: u8,
}

impl DirectTilemapDescriptor {
    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        descriptor_bytes(self.tilemap_pointer, self.parameter16, self.parameter8, self.opaque_byte)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTilemapDescriptor {
    pub source_offset: u32,
    pub tilemap_pointer: u32,
    pub parameter16: u16,
    pub parameter8: u8,
    pub opaque_byte: u8,
    pub additional_descriptors: [AdditionalDescriptor; 4],
    pub transfer_descriptors: [AdditionalDescriptor; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_tilemap_descriptors: Option<[DirectTilemapDescriptor; 7]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interleaved_lz_descriptors: Option<[AdditionalDescriptor; 7]>,
}

impl ObservedTilemapDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.tilemap_pointer != 0x0800_0000 + observed_tilemap::SOURCE_OFFSET {
            return Err("Invalid observed tilemap descriptor source or pointer".into());
        }
        for (entry, &(offset, _, wrapper_offset, _)) in self.additional_descriptors.iter()
            .chain(&self.transfer_descriptors).zip(ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS)) {
            if entry.source_offset != offset || entry.asset_pointer != 0x0800_0000 + wrapper_offset
                || entry.wrapper.source_offset != wrapper_offset || entry.wrapper.flags & 2 == 0 {
                return Err("Invalid additional tilemap descriptor or LZ wrapper".into());
            }
        }
        if let Some(entries) = &self.additional_tilemap_descriptors {
            for (i, (entry, &(target, _, _, _))) in entries.iter().zip(&observed_tilemap::ADDITIONAL_STREAM_SPECS).enumerate() {
                if entry.source_offset != SOURCE_OFFSET + ((i + 1) * SOURCE_LENGTH) as u32
                    || entry.tilemap_pointer != 0x0800_0000 + target {
                    return Err("Invalid direct tilemap descriptor".into());
                }
            }
        }
        if let Some(entries) = &self.interleaved_lz_descriptors {
            for (entry, &(offset, _, wrapper_offset, _)) in entries.iter().zip(&INTERLEAVED_LZ_SPECS) {
                if entry.source_offset != offset || entry.asset_pointer != 0x0800_0000 + wrapper_offset
                    || entry.wrapper.source_offset != wrapper_offset || entry.wrapper.flags & 2 == 0 {
                    return Err("Invalid interleaved LZ descriptor or wrapper".into());
                }
            }
        }
        Ok(())
    }

    pub fn validate_dependencies(&self, tilemap: &observed_tilemap::ObservedTilemap) -> Result<(), String> {
        self.validate()?;
        tilemap.validate()?;
        if self.tilemap_pointer != 0x0800_0000 + tilemap.source_offset {
            return Err("Observed tilemap descriptor dependency mismatch".into());
        }
        if let Some(entries) = &self.additional_tilemap_descriptors {
            let streams = tilemap.additional_streams.as_ref().ok_or("Missing additional tilemap dependencies")?;
            if entries.iter().zip(streams).any(|(entry, stream)| entry.tilemap_pointer != 0x0800_0000 + stream.source_offset) {
                return Err("Additional tilemap descriptor dependency mismatch".into());
            }
        }
        Ok(())
    }

    pub fn validate_lz_dependencies(&self, lz: &observed_lz::ObservedLzBlock) -> Result<(), String> {
        self.validate()?;
        lz.validate()?;
        for entry in self.additional_descriptors.iter().chain(&self.transfer_descriptors)
            .chain(self.interleaved_lz_descriptors.iter().flatten()) {
            if !lz.streams().any(|(offset, _, _)| offset == entry.wrapper.source_offset + WRAPPER_LENGTH as u32) {
                return Err("Missing linked observed LZ stream".into());
            }
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        descriptor_bytes(self.tilemap_pointer, self.parameter16, self.parameter8, self.opaque_byte)
    }

    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        let mut spans = vec![(self.source_offset, self.source_bytes().to_vec(), "tilemap-descriptor")];
        let roles = [("tilemap-descriptor-0", "lz-wrapper-0"), ("tilemap-descriptor-1", "lz-wrapper-1"),
                     ("tilemap-descriptor-2", "lz-wrapper-2"), ("tilemap-descriptor-3", "lz-wrapper-3")];
        for (entry, (descriptor_role, wrapper_role)) in self.additional_descriptors.iter().zip(roles) {
            spans.push((entry.source_offset, entry.source_bytes().to_vec(), descriptor_role));
            spans.push((entry.wrapper.source_offset, entry.wrapper.source_bytes().to_vec(), wrapper_role));
        }
        let roles = [("transfer-descriptor-0", "transfer-lz-wrapper-0"), ("transfer-descriptor-1", "transfer-lz-wrapper-1"),
                     ("transfer-descriptor-2", "transfer-lz-wrapper-2")];
        for (entry, (descriptor_role, wrapper_role)) in self.transfer_descriptors.iter().zip(roles) {
            spans.push((entry.source_offset, entry.source_bytes().to_vec(), descriptor_role));
            spans.push((entry.wrapper.source_offset, entry.wrapper.source_bytes().to_vec(), wrapper_role));
        }
        if let Some(entries) = &self.additional_tilemap_descriptors {
            for entry in entries { spans.push((entry.source_offset, entry.source_bytes().to_vec(), "additional-tilemap-descriptor")); }
        }
        if let Some(entries) = &self.interleaved_lz_descriptors {
            for entry in entries {
                spans.push((entry.source_offset, entry.source_bytes().to_vec(), "interleaved-lz-descriptor"));
                spans.push((entry.wrapper.source_offset, entry.wrapper.source_bytes().to_vec(), "interleaved-lz-wrapper"));
            }
        }
        spans
    }
}

pub struct Catalog { pub decoded: ObservedTilemapDescriptor, pub sources: Vec<SourceRange> }

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed tilemap descriptor requires the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0xdb30a..0xdb326]) != "0ac3d1f008eb9ff01ae13ba635e54c40af5f41ddcc4150750a63c5dd100a7174"
            || ffta_rom::digest(&rom[0xdb09c..0xdb0e2]) != "88212fa97c214b1fa9cba491d25881a0cda455c176957fa7363ab8a0c8e25a58"
            || ffta_rom::digest(&rom[0xdb14e..0xdb178]) != "2781f4bde987916b93cc153dc721b82a2503bbf89d5d1834ade0be69e71d2313"
            || u32::from_le_bytes(rom[0x3a4bf0..0x3a4bf4].try_into().unwrap()) != 0x083a4b70 {
            return Err("Observed tilemap descriptor original reader mismatch".into());
        }
        let map = previous.iter().find(|resource| resource.id == OBSERVED_TILEMAP_ID).ok_or("Missing observed tilemap dependency")?;
        if !map.sources.iter().any(|source| source.offset == observed_tilemap::SOURCE_OFFSET && source.length as usize == observed_tilemap::SOURCE_LENGTH) {
            return Err("Observed tilemap descriptor source dependency mismatch".into());
        }
        for &(offset, length, _, _) in &observed_tilemap::ADDITIONAL_STREAM_SPECS {
            if !map.sources.iter().any(|source| source.offset == offset && source.length as usize == length) {
                return Err("Missing additional tilemap descriptor source dependency".into());
            }
        }
        let lz = previous.iter().find(|resource| resource.id == OBSERVED_LZ_ID).ok_or("Missing observed LZ dependency")?;
        for &(offset, hash, wrapper_offset, wrapper_hash) in ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS).chain(&INTERLEAVED_LZ_SPECS) {
            let at = offset as usize;
            let wrapper = wrapper_offset as usize;
            if ffta_rom::digest(&rom[at..at + SOURCE_LENGTH]) != hash
                || ffta_rom::digest(&rom[wrapper..wrapper + WRAPPER_LENGTH]) != wrapper_hash
                || !lz.sources.iter().any(|source| source.offset == wrapper_offset + WRAPPER_LENGTH as u32) {
                return Err("Additional descriptor or linked LZ source mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap descriptor source hash mismatch".into()); }
        let read_descriptor = |offset: u32, wrapper_offset: u32| {
            let entry = &rom[offset as usize..offset as usize + SOURCE_LENGTH];
            let wrapper = &rom[wrapper_offset as usize..wrapper_offset as usize + WRAPPER_LENGTH];
            AdditionalDescriptor {
                source_offset: offset, asset_pointer: u32::from_le_bytes(entry[..4].try_into().unwrap()),
                parameter16: u16::from_le_bytes(entry[4..6].try_into().unwrap()), parameter8: entry[6], opaque_byte: entry[7],
                wrapper: LzWrapper { source_offset: wrapper_offset, kind: wrapper[0], flags: wrapper[1], opaque_halfword: u16::from_le_bytes(wrapper[2..4].try_into().unwrap()) },
            }
        };
        let additional_descriptors = std::array::from_fn(|i| read_descriptor(ADDITIONAL_SPECS[i].0, ADDITIONAL_SPECS[i].2));
        let transfer_descriptors = std::array::from_fn(|i| read_descriptor(TRANSFER_SPECS[i].0, TRANSFER_SPECS[i].2));
        let interleaved_lz_descriptors = std::array::from_fn(|i| read_descriptor(INTERLEAVED_LZ_SPECS[i].0, INTERLEAVED_LZ_SPECS[i].2));
        let mut tilemap_descriptors = Vec::new();
        for (i, expected) in TILEMAP_DESCRIPTOR_SHA256.iter().enumerate() {
            let offset = SOURCE_OFFSET + ((i + 1) * SOURCE_LENGTH) as u32;
            let entry = &rom[offset as usize..offset as usize + SOURCE_LENGTH];
            if ffta_rom::digest(entry) != *expected { return Err("Additional tilemap descriptor source mismatch".into()); }
            tilemap_descriptors.push(DirectTilemapDescriptor {
                source_offset: offset, tilemap_pointer: u32::from_le_bytes(entry[..4].try_into().unwrap()),
                parameter16: u16::from_le_bytes(entry[4..6].try_into().unwrap()), parameter8: entry[6], opaque_byte: entry[7],
            });
        }
        let additional_tilemap_descriptors: [DirectTilemapDescriptor; 7] = tilemap_descriptors.try_into().map_err(|_| "Incomplete tilemap descriptors")?;
        let decoded = ObservedTilemapDescriptor {
            source_offset: SOURCE_OFFSET,
            tilemap_pointer: u32::from_le_bytes(bytes[..4].try_into().unwrap()),
            parameter16: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            parameter8: bytes[6], opaque_byte: bytes[7], additional_descriptors, transfer_descriptors,
            additional_tilemap_descriptors: Some(additional_tilemap_descriptors),
            interleaved_lz_descriptors: Some(interleaved_lz_descriptors),
        };
        decoded.validate()?;
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans() {
            if prior.contains(offset, bytes.len())? { return Err("Observed tilemap descriptor source already has an owner".into()); }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes { return Err("Observed tilemap descriptor reconstruction mismatch".into()); }
            sources.push(SourceRange { offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role: role.into() });
        }
        Ok(Self { decoded, sources })
    }
}
