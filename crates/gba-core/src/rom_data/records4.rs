//! Ordered EUR four-byte slots. Physical ownership is separate from guest indexing.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x7133f8;
pub const SOURCE_END: u32 = 0x713744;
pub const RECORD_BYTES: usize = 4;
pub const RECORD_COUNT: usize = 211;
pub const SOURCE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;
pub const SOURCE_SHA256: &str = "85d556e39da8baaec49308c815125532aa8947a476e2c59131532a5bf0334fa9";
/// Three independently rooted physical callback partitions; guest indices remain unbounded.
pub const CALLBACK_ROOTS: [u32; 3] = [0x3ad638, 0x3ad6ac, 0x3ad72c];
pub const CALLBACK_ENDS: [u32; 3] = [0x3ad6ac, 0x3ad72c, 0x3ad7e4];
pub const CALLBACK_COUNTS: [usize; 3] = [29, 32, 46];
pub const CALLBACK_SHA256: [&str; 3] = [
    "21c1de2f4cf3c3b2538be11437608527ef639e6c646fa09afed601b100126a52",
    "a23f5918182840e2dc62549a188c33a79dd6fd5553fd7c4c1010a9d2d711209e",
    "39fe6c95af36146f298155406ba0845c17d16db3fdaadbcd773d6798d70fb483",
];
/// Independently rooted 12-byte action-callback records; guest indices remain unbounded.
pub const ACTION_ROOT: u32 = 0x3ad7e4;
pub const ACTION_END: u32 = 0x3adc40;
pub const ACTION_COUNT: usize = 93;
pub const ACTION_RECORD_BYTES: usize = 12;
pub const ACTION_SHA256: &str = "921665ecf6f9533d6267860b6083af6820a33756b43b2019b8c6f70812d1a971";
pub type SourceSpan = (u32, Vec<u8>, String);

/// One callback and two original words whose full semantics remain guest-owned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCallback {
    pub callback: u32,
    pub opaque_first: u32,
    pub opaque_second: u32,
}
impl ActionCallback {
    pub fn source_bytes(&self) -> [u8; ACTION_RECORD_BYTES] {
        let mut bytes = [0; ACTION_RECORD_BYTES];
        bytes[0..4].copy_from_slice(&self.callback.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.opaque_first.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.opaque_second.to_le_bytes());
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u8,
    pub field00: u8,
    pub field01: u8,
    pub field02: u8,
    pub field03: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        [self.field00, self.field01, self.field02, self.field03]
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }
}

/// Original readers load the index as u8. This returns an encoded guest address,
/// including indices outside this catalog's physical ownership.
pub fn source_address(index: u8) -> u32 {
    0x08000000 + SOURCE + u32::from(index) * RECORD_BYTES as u32
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records4 {
    pub source_offset: u32,
    pub records: Vec<Record>,
    /// Original encoded Thumb pointers; selection and dispatch remain guest code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_tables: Option<[Vec<u32>; 3]>,
    /// Ordered action callbacks and their original opaque words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_callbacks: Option<Vec<ActionCallback>>,
}
impl Records4 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid declared EUR records4 source/count".into());
        }
        if self
            .records
            .iter()
            .enumerate()
            .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid records4 physical slot order/index".into());
        }
        if let Some(tables) = &self.callback_tables {
            for (table, &count) in tables.iter().zip(&CALLBACK_COUNTS) {
                if table.len() != count || table.iter().any(|&pointer| pointer & 1 == 0
                    || !(0x0800_0000..0x0900_0000).contains(&pointer)) {
                    return Err("Invalid records4 callback table projection".into());
                }
            }
        }
        if let Some(actions) = &self.action_callbacks {
            if actions.len() != ACTION_COUNT || actions.iter().any(|action| action.callback & 1 == 0
                || !(0x0800_0000..0x0900_0000).contains(&action.callback)) {
                return Err("Invalid records4 action-callback projection".into());
            }
        }
        Ok(())
    }
    /// None means unowned; it does not replace an original guest read with zero.
    pub fn record(&self, index: u8) -> Option<&Record> {
        self.records.get(usize::from(index))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn callback_byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        let tables = self.callback_tables.as_ref()?;
        for (i, (&start, &end)) in CALLBACK_ROOTS.iter().zip(&CALLBACK_ENDS).enumerate() {
            if (start..end).contains(&offset) {
                let relative = (offset - start) as usize;
                return Some((tables[i].get(relative / 4)?.to_le_bytes()[relative % 4], i));
            }
        }
        None
    }
    pub fn action_byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        if !(ACTION_ROOT..ACTION_END).contains(&offset) { return None; }
        let relative = (offset - ACTION_ROOT) as usize;
        let byte = self.action_callbacks.as_ref()?.get(relative / ACTION_RECORD_BYTES)?
            .source_bytes()[relative % ACTION_RECORD_BYTES];
        Some((byte, (relative % ACTION_RECORD_BYTES) / 4))
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if let Some((byte, _)) = self.callback_byte_at(offset) { return Some(byte); }
        if let Some((byte, _)) = self.action_byte_at(offset) { return Some(byte); }
        if !(SOURCE..SOURCE_END).contains(&offset) {
            return None;
        }
        let relative = (offset - SOURCE) as usize;
        self.records
            .get(relative / RECORD_BYTES)?
            .byte(relative % RECORD_BYTES)
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        if let Some(tables) = &self.callback_tables {
            for (index, table) in tables.iter().enumerate() {
                spans.push((CALLBACK_ROOTS[index], table.iter().flat_map(|word| word.to_le_bytes()).collect(),
                    format!("linked-records4-callback-table-{index}")));
            }
        }
        if let Some(actions) = &self.action_callbacks {
            spans.push((ACTION_ROOT, actions.iter().flat_map(ActionCallback::source_bytes).collect(),
                "linked-records4-action-callbacks".into()));
        }
        spans.push((SOURCE, self.source_bytes(), "linked-records4-slots".into()));
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Records4 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Records4,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Records4 requires exact EUR ROM; USA pending".into());
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Records4 source fingerprint mismatch".into());
        }
        for &(start, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Records4 original reader/boundary fingerprint mismatch".into());
            }
        }
        for (position, expected) in [(0x134cb4, CALLBACK_ROOTS[0]), (0x134d0c, CALLBACK_ROOTS[0]),
            (0x135100, CALLBACK_ROOTS[1]), (0x1357e4, CALLBACK_ROOTS[2]), (0x136030, CALLBACK_ENDS[2])] {
            let word = u32::from_le_bytes(rom[position..position + 4].try_into().unwrap());
            if word != 0x0800_0000 + expected { return Err("Records4 callback root mismatch".into()); }
        }
        let mut callback_tables: [Vec<u32>; 3] = std::array::from_fn(|_| Vec::new());
        for index in 0..3 {
            let bytes = &rom[CALLBACK_ROOTS[index] as usize..CALLBACK_ENDS[index] as usize];
            if ffta_rom::digest(bytes) != CALLBACK_SHA256[index] {
                return Err("Records4 callback source mismatch".into());
            }
            callback_tables[index] = bytes.chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().unwrap())).collect();
        }
        for (position, expected) in [(0x135d5c, ACTION_END), (0x137598, ACTION_ROOT),
            (0x136030, ACTION_ROOT), (0x1360a8, ACTION_ROOT), (0x1375fc, ACTION_ROOT), (0x13761c, ACTION_ROOT)] {
            let word = u32::from_le_bytes(rom[position..position + 4].try_into().unwrap());
            if word != 0x0800_0000 + expected { return Err("Records4 action callback root mismatch".into()); }
        }
        let action_bytes = &rom[ACTION_ROOT as usize..ACTION_END as usize];
        if ffta_rom::digest(action_bytes) != ACTION_SHA256 {
            return Err("Records4 action callback source mismatch".into());
        }
        let action_callbacks = action_bytes.chunks_exact(ACTION_RECORD_BYTES).map(|raw| ActionCallback {
            callback: u32::from_le_bytes(raw[0..4].try_into().unwrap()),
            opaque_first: u32::from_le_bytes(raw[4..8].try_into().unwrap()),
            opaque_second: u32::from_le_bytes(raw[8..12].try_into().unwrap()),
        }).collect();
        let decoded = Records4 {
            source_offset: SOURCE,
            records: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, r)| Record {
                    index: i as u8,
                    field00: r[0],
                    field01: r[1],
                    field02: r[2],
                    field03: r[3],
                })
                .collect(),
            callback_tables: Some(callback_tables),
            action_callbacks: Some(action_callbacks),
        };
        decoded.validate()?;
        let sources = decoded.source_spans()?.into_iter().map(|(offset, bytes, role)| SourceRange {
            offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role,
        }).collect();
        Ok(Self { decoded, sources })
    }
}

// Authenticated source-contract windows, including independent successor witnesses.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (0x135cdc, 0x84, "fcc1f6790acd6289fee1aa75a71c847a5f1e13e61d56c2e5113036ec390aa06e"),
    (0x137520, 0x16, "2247a4dfe4696252827363b6d3df0fcf938e53af269e74316d2c0fdb64097990"),
    (0x1375f0, 0x30, "714856bd4b914fe82e86390d6a950d305b2095bc760c5f606cc9a50794ecf2c1"),
    (0x135fe0, 0x54, "d30134b1c10513a1af020fac36e547118045d2784214fb0a3c74f91df2968971"),
    (0x145c08, 10, "ba1865e775c7470b5a07129f1da0dad3e8bf56b5054192dc2e06bedc4babaa6d"),
    (
        0xb8526,
        180,
        "a7c074fe46a684889c3f4086ee6f29e4fb8e7db6209cb6d3a651e3a2a0409206",
    ),
    (
        0xc5508,
        180,
        "e3fe001765794483c7c39477f6bd34ee80de460f5ca074a7ce0fcd33827dbfb2",
    ),
    (
        0xc5a26,
        180,
        "e7d52ed7d1ab6ee8eef12d227858046c52e2d55b0fa8938753a876196308baf1",
    ),
    (
        0x132efc,
        180,
        "d3aa2fea837fa08b2510ecd17971b23dee4f24d3ad48e1f2ff3bee80cbd2b96e",
    ),
    (
        0x132faa,
        180,
        "dff6369797cda55a78c60351b52eae0cf9f451227e2d5ab382fda6af6087074c",
    ),
    (
        0x133030,
        180,
        "b5ad1e957c1c1d7d15612113553d2913a968d4ab292a0fecf7b8de678cf9f978",
    ),
    (
        0xd56aa,
        180,
        "af4b1b981d48365083439ae48233adadedd00d73840946d1b3e42b38b2006a59",
    ),
    (
        0x132ea0,
        1520,
        "9c4ae6e42c8c9a26a4f590293616210401d537b25a23d22f9b853330609fc2f8",
    ),
    (
        0xd573e,
        160,
        "2807672ca77b65fd0fc874da2457c17f0b50ac71e8aa2526abe724545a3efff8",
    ),
    (
        0x134c20,
        1424,
        "a1bafc8880d36c42eeeca6f38da496872af9835c8fb02625d569fef2674aed6c",
    ),
    (
        0x135490,
        120,
        "79c2cdd6290375051cf139d2237e879bea40248357d5a30c7db723aa9c9dbee0",
    ),
    (
        0x135780,
        120,
        "f23d11ec127cbb806e59aec5da4fbb312a9e82f27889720bc47cd29b16e4a104",
    ),
    (
        0xc5b80,
        48,
        "87b0c79a1dfb7e774ffae6b38c555186e9113ff7d3b28eacfb1fe1fbc5875370",
    ),
];

#[cfg(test)]
mod tests;
