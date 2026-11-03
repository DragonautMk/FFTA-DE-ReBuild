//! Declared EUR mode0/width5 archives. Original decoding and uploads stay guest work.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const GROUPS: [u32; 4] = [0x71e57c, 0x71e61c, 0x71e630, 0x71e658];
pub const ARCHIVES: [u32; 20] = [
    0x430248, 0x441608, 0x452c7c, 0x464818, 0x4757e0, 0x434418, 0x44566c, 0x457058, 0x468608,
    0x479b4c, 0x4357e4, 0x4469b0, 0x45862c, 0x46989c, 0x47b098, 0x4334c4, 0x444610, 0x455f18,
    0x46760c, 0x478a5c,
];
const COUNTS: [usize; 20] = [
    30, 30, 30, 30, 30, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 85, 91, 93, 91, 99,
];
const SIZES: [u16; 4] = [192, 7168, 3840, 64];
const ROM_LIMIT: usize = 0x1000000;
pub const SOURCE_SHA256: &str = "ab21ada958cc5bcb13f06b59218bf9b4757155684c794881dd9ca7c0f4859006";
pub type SourceSpan = (u32, Vec<u8>, String);
fn invalid() -> String {
    "Invalid declared indexed graphics archive".into()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Packet {
    Literal { bytes: Vec<u8> },
    Dictionary { length: u8, offset: u16 },
}
impl Packet {
    fn encoded(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Literal { bytes } if (1..=128).contains(&bytes.len()) => {
                let mut v = vec![0x80 | (bytes.len() - 1) as u8];
                v.extend(bytes);
                Ok(v)
            }
            Self::Dictionary { length, offset } if (3..=34).contains(length) && *offset <= 1023 => {
                Ok(((u16::from(*length - 3) << 10) | offset)
                    .to_be_bytes()
                    .to_vec())
            }
            _ => Err(invalid()),
        }
    }
    fn append_decoded(&self, dictionary: &[u8], out: &mut Vec<u8>) -> Result<usize, String> {
        self.encoded()?;
        match self {
            Self::Literal { bytes } => {
                out.extend(bytes);
                Ok(0)
            }
            Self::Dictionary { length, offset } => {
                let end = usize::from(*offset) + usize::from(*length);
                out.extend(
                    dictionary
                        .get(usize::from(*offset)..end)
                        .ok_or_else(invalid)?,
                );
                Ok(end)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub index: u16,
    pub relative_offset: u32,
    pub packets: Vec<Packet>,
    pub decoded_words: Vec<u32>,
}
impl Program {
    pub fn decoded_bytes(&self) -> Vec<u8> {
        self.decoded_words
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub source_offset: u32,
    pub opaque_header: [u8; 2],
    pub entry_offsets: Vec<u32>,
    pub dictionary_relative_offset: u32,
    pub dictionary_mode: u8,
    pub dictionary_length_bits: u8,
    pub fixed_decoded_bytes: u16,
    pub dictionary_bytes: Vec<u8>,
    pub programs: Vec<Program>,
}
impl Archive {
    /// Host ownership covers only the proved canonical format, not a guest mode/width cap.
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        if self.dictionary_mode != 0
            || self.dictionary_length_bits != 5
            || self.programs.is_empty()
            || self.programs.len() > u16::MAX as usize
            || self.entry_offsets.len() != self.programs.len()
            || self.source_offset as usize >= ROM_LIMIT
        {
            return Err(invalid());
        }
        let mut encoded = Vec::new();
        encoded.extend(self.opaque_header);
        encoded.extend((self.programs.len() as u16).to_be_bytes());
        encoded.extend(self.dictionary_relative_offset.to_be_bytes());
        for offset in &self.entry_offsets {
            encoded.extend(offset.to_be_bytes());
        }
        let mut max_reference = 0;
        for (index, program) in self.programs.iter().enumerate() {
            if usize::from(program.index) != index
                || program.relative_offset as usize != encoded.len()
                || self.entry_offsets[index] != program.relative_offset
            {
                return Err(invalid());
            }
            let mut decoded = Vec::new();
            for packet in &program.packets {
                max_reference =
                    max_reference.max(packet.append_decoded(&self.dictionary_bytes, &mut decoded)?);
                if decoded.len() > usize::from(self.fixed_decoded_bytes) {
                    return Err(invalid());
                }
                encoded.extend(packet.encoded()?);
            }
            if decoded.len() != usize::from(self.fixed_decoded_bytes)
                || decoded != program.decoded_bytes()
            {
                return Err(invalid());
            }
        }
        if encoded.len() != self.dictionary_relative_offset as usize
            || self.dictionary_bytes.len() != max_reference
        {
            return Err(invalid());
        }
        encoded.extend([self.dictionary_mode, self.dictionary_length_bits]);
        encoded.extend(self.fixed_decoded_bytes.to_be_bytes());
        encoded.extend(&self.dictionary_bytes);
        if (self.source_offset as usize)
            .checked_add(encoded.len())
            .is_none_or(|end| end > ROM_LIMIT)
        {
            return Err(invalid());
        }
        Ok(encoded)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.source_bytes().map(|_| ())
    }
    /// Parse only the source actually consumed by the selected canonical programs.
    pub fn from_source(input: &[u8], source_offset: u32) -> Result<Self, String> {
        fn be32(bytes: &[u8]) -> u32 {
            u32::from_be_bytes(bytes.try_into().unwrap())
        }
        if input.len() < 8 {
            return Err(invalid());
        }
        let count = usize::from(u16::from_be_bytes([input[2], input[3]]));
        let header_end = 8 + count * 4;
        let dictionary_relative_offset = be32(&input[4..8]);
        let dict = dictionary_relative_offset as usize;
        let dh = input
            .get(dict..dict.checked_add(4).ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        if count == 0 || header_end > dict || dh[0] != 0 || dh[1] != 5 {
            return Err(invalid());
        }
        let index = input.get(8..header_end).ok_or_else(invalid)?;
        let entry_offsets: Vec<_> = index.chunks_exact(4).map(be32).collect();
        let fixed_decoded_bytes = u16::from_be_bytes([dh[2], dh[3]]);
        if fixed_decoded_bytes % 4 != 0 {
            return Err(invalid());
        }
        let dictionary = &input[dict + 4..];
        let (mut cursor, mut max_reference) = (header_end, 0);
        let mut programs = Vec::new();
        for (i, &relative_offset) in entry_offsets.iter().enumerate() {
            if relative_offset as usize != cursor {
                return Err(invalid());
            }
            let mut packets = Vec::new();
            let mut decoded = Vec::new();
            while decoded.len() < usize::from(fixed_decoded_bytes) {
                let token = *input
                    .get(cursor)
                    .filter(|_| cursor < dict)
                    .ok_or_else(invalid)?;
                cursor += 1;
                let packet = if token & 0x80 != 0 {
                    let end = cursor + usize::from(token & 0x7f) + 1;
                    if end > dict {
                        return Err(invalid());
                    }
                    let bytes = input.get(cursor..end).ok_or_else(invalid)?.to_vec();
                    cursor = end;
                    Packet::Literal { bytes }
                } else {
                    let next = *input
                        .get(cursor)
                        .filter(|_| cursor < dict)
                        .ok_or_else(invalid)?;
                    cursor += 1;
                    let word = u16::from_be_bytes([token, next]);
                    Packet::Dictionary {
                        length: (word >> 10) as u8 + 3,
                        offset: word & 1023,
                    }
                };
                max_reference = max_reference.max(packet.append_decoded(dictionary, &mut decoded)?);
                if decoded.len() > usize::from(fixed_decoded_bytes) {
                    return Err(invalid());
                }
                packets.push(packet);
            }
            programs.push(Program {
                index: i as u16,
                relative_offset,
                packets,
                decoded_words: decoded
                    .chunks_exact(4)
                    .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                    .collect(),
            });
        }
        if cursor != dict {
            return Err(invalid());
        }
        let archive = Self {
            source_offset,
            opaque_header: [input[0], input[1]],
            entry_offsets,
            dictionary_relative_offset,
            dictionary_mode: dh[0],
            dictionary_length_bits: dh[1],
            fixed_decoded_bytes,
            dictionary_bytes: dictionary[..max_reference].to_vec(),
            programs,
        };
        let reconstructed = archive.source_bytes()?;
        if input.get(..reconstructed.len()) != Some(reconstructed.as_slice()) {
            return Err(invalid());
        }
        Ok(archive)
    }
    /// Same low16 argument projection and remaining-count clamp as original0537C.
    pub fn selected_programs(&self, start: u32, count: u32) -> Option<&[Program]> {
        let start = usize::from(start as u16);
        let count = usize::from(count as u16);
        let n = self.programs.len();
        if start >= n {
            return None;
        }
        let count = if count == 0 || count > n { n } else { count }.min(n - start);
        self.programs.get(start..start + count)
    }
    pub fn selected_bytes(&self, start: u32, count: u32) -> Option<Vec<u8>> {
        Some(
            self.selected_programs(start, count)?
                .iter()
                .flat_map(Program::decoded_bytes)
                .collect(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerGroup {
    pub source_offset: u32,
    /// ROM offsets; original pointer loads restore the08000000 base.
    pub pointers: [u32; 5],
}
impl PointerGroup {
    pub fn select(&self, language: u8) -> Option<u32> {
        self.pointers.get(usize::from(language)).copied()
    }
    pub fn encoded_view(&self, language: u8) -> u32 {
        (0x08000000 + self.source_offset).wrapping_add(u32::from(language) * 4)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.pointers
            .iter()
            .flat_map(|v| v.wrapping_add(0x08000000).to_le_bytes())
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedGraphics {
    pub groups: Vec<PointerGroup>,
    pub archives: Vec<Archive>,
}
impl IndexedGraphics {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.groups.len() != 4 || self.archives.len() != 20 {
            return Err(invalid());
        }
        let mut spans = Vec::new();
        for (i, group) in self.groups.iter().enumerate() {
            if group.source_offset != GROUPS[i] || group.pointers != ARCHIVES[i * 5..i * 5 + 5] {
                return Err(invalid());
            }
            spans.push((
                group.source_offset,
                group.source_bytes(),
                format!("indexed-archive-selector/{:x}", group.source_offset),
            ));
        }
        for (i, archive) in self.archives.iter().enumerate() {
            if archive.source_offset != ARCHIVES[i]
                || archive.programs.len() != COUNTS[i]
                || archive.fixed_decoded_bytes != SIZES[i / 5]
            {
                return Err(invalid());
            }
            spans.push((
                archive.source_offset,
                archive.source_bytes()?,
                format!("indexed-archive/{:x}", archive.source_offset),
            ));
        }
        spans.sort_by_key(|v| v.0);
        for pair in spans.windows(2) {
            if pair[0].0 as usize + pair[0].1.len() > pair[1].0 as usize {
                return Err(invalid());
            }
        }
        Ok(spans)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.source_spans().map(|_| ())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.source_spans()
            .ok()?
            .into_iter()
            .find_map(|(base, bytes, _)| bytes.get(offset.checked_sub(base)? as usize).copied())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Indexed archives differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: IndexedGraphics,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != ROM_LIMIT || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Indexed graphics require exact EUR ROM; USA pending".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("Indexed graphics reader fingerprint mismatch".into());
            }
        }
        let mut groups = Vec::new();
        for source_offset in GROUPS {
            let at = source_offset as usize;
            let mut pointers = [0; 5];
            for (i, pointer) in pointers.iter_mut().enumerate() {
                *pointer = u32::from_le_bytes(rom[at + i * 4..at + i * 4 + 4].try_into().unwrap())
                    .checked_sub(0x08000000)
                    .ok_or_else(invalid)?;
            }
            groups.push(PointerGroup {
                source_offset,
                pointers,
            });
        }
        let archives = ARCHIVES
            .iter()
            .map(|&at| Archive::from_source(&rom[at as usize..], at))
            .collect::<Result<_, _>>()?;
        let decoded = IndexedGraphics { groups, archives };
        let spans = decoded.source_spans()?;
        let source: Vec<u8> = spans
            .iter()
            .flat_map(|(_, b, _)| b.iter().copied())
            .collect();
        if ffta_rom::digest(&source) != SOURCE_SHA256 {
            return Err("Indexed archive source fingerprint mismatch".into());
        }
        let sources = spans
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}

// Populated from the independently authenticated static reader windows.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x19660,
        176,
        "374b9553e61568685700a2e7d17325df4cb92e2717c7a10c08e69321c5e4a9ea",
    ),
    (
        0x52f0,
        400,
        "df219150a0e4a69240af55b6727ae962e1ad8857d0b47f09cce7fda2b8e83506",
    ),
    (
        0x93ac0,
        272,
        "2384b64b4b17661b2fa36c368dff76cac1f05c3100348762d743d4352f127680",
    ),
    (
        0x93e04,
        272,
        "e56d46f74eb70d639124ef91d92dd960ec54292c8abe76008b75607dbd39e396",
    ),
    (
        0x9412c,
        288,
        "17c7aeb038c1f262de136b093f1b9f0ac055cffc8a014b6c9eec8de898ba0a71",
    ),
    (
        0x94470,
        288,
        "8500242a2b17ea84874cd7c32497cd762ba7c50e14a8b7442a39fc875ca7fae0",
    ),
    (
        0xac2e8,
        224,
        "67ee9798acddc72098808bb5034d280230c74083a3ae01794ef82dc4d634b207",
    ),
    (
        0x939b4,
        268,
        "0403924dd7378c64cc4efb206c01e46c384169640ac154eadae19b54e0f51e52",
    ),
    (
        0x93cf8,
        268,
        "6d0262a1487d57920a54f94642fd22440f34ca448721d7d612f45b4d1c7680a9",
    ),
    (
        0x94020,
        268,
        "801352fc48d2cb5d7afa36fb5815a0e836c29911eb19f9fe9696dcc354849854",
    ),
    (
        0x94364,
        268,
        "df7ad350511a205c49c1e510270b213b00311cad6833f68cf91be7e35705c9d9",
    ),
    (
        0xac3c8,
        56,
        "dbc7151090e2f5c805e5c26b38f7f8487019f305e76b2657b47c52b0c778adcb",
    ),
    (
        0x5228,
        200,
        "28700550aefa557d847f83d43abbb5b3819b96eef8f1bd52995f1b5c3d262c83",
    ),
    (
        0x93954,
        96,
        "39ec3452cc63cee77a31315a1c66d00d6db455c988540a06954f1869f9d66ac5",
    ),
    (
        0x93bd0,
        96,
        "40d140c9a662379086c361a5dc97a14c46bc42ad10f3cd4d3f3bd6eda97dd72e",
    ),
    (
        0x93c98,
        96,
        "23ed9f516332fbdfe77b6fe25f055559f12fbab8ec525dc8e1e2e0bb23ea35f2",
    ),
    (
        0x93ef4,
        96,
        "19b6ae5d1debdccd1f8520d287039d964431b81d4e1f8958d8de3dbaa2c9dc74",
    ),
    (
        0x93fc0,
        96,
        "2eef156db800ff854be62f35ece73106acfe5304ad7315f8accb5502a2b677dd",
    ),
    (
        0x94234,
        96,
        "a0ba362ce8141ef1e77be1d41b8c132c2d2cf055eff7ac963f360214ff95bf54",
    ),
    (
        0x94304,
        96,
        "e6b6f6f9295fbde16b95b03690033830c50d876ec8e98812eaa63b6e8b9b92ee",
    ),
    (
        0x94574,
        96,
        "ea6a9a2eaf39e8b7f1810d61ab4fdbcaa26a23c6cffded318090b8c569bc3b13",
    ),
    (
        0xac288,
        96,
        "78dfa121411befaca1e8dcf2fa47a1c2e541e2153b4347d5ef67ae9aa26bd48f",
    ),
];

#[cfg(test)]
mod tests;
