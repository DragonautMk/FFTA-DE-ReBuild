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
pub const ACTOR_LOOKUP_EXTENSION_ROOT: u32 = ACTOR_LOOKUP_ROOT + 4;
pub const ACTOR_LOOKUP_EXTENSION_SHA256: &str = "cdd3d0cb62d5d9450779b8cd7378b02ccdc126cb53e17d542dc1abec20565f91";
pub const LAYOUT_OFFSETS_ROOT: u32 = 0x398de8;
pub const LAYOUT_OFFSETS_LENGTH: usize = 8;
pub const LAYOUT_OFFSETS_SHA256: &str = "1bd03f2982e14e5c256e49cf9432f967e48d3aa69b863e120a1d1d3589e4909a";
/// Five observed Thumb callbacks into the guest-copied IWRAM OBJ routines.
pub const IWRAM_DRAW_CALLBACKS_ROOT: u32 = 0x397f80;
pub const IWRAM_DRAW_CALLBACKS_COUNT: usize = 5;
pub const IWRAM_DRAW_CALLBACKS_SHA256: &str = "4673dedee715e28da29fdc9e3db2b9ef233d956e70847774cae8922a13011f60";
/// Two independently observed callback words outside the contiguous draw vector.
pub const IWRAM_EXTRA_CALLBACK_ROOTS: [u32; 2] = [0x397fa4, 0x397fb8];
pub const IWRAM_EXTRA_CALLBACK_SHA256: [&str; 2] = [
    "c5059998b08db05ec6cfe0b8d7263dfb32447670eebfb269712a55e6dc24799c",
    "6f64a62b102700d15e8548ad7f43d998c60bc5d2160e7c4834f47ed378749f1b",
];
/// Independently rooted guest callbacks; the intervening 397fb4 word remains raw.
pub const IWRAM_ADDITIONAL_CALLBACK_ROOTS: [u32; 7] = [
    0x397f94, 0x397f98, 0x397f9c, 0x397fa0, 0x397fa8, 0x397fac, 0x397fb0,
];
pub const TITLE_OPENING_DIRECTORY_ROOT: u32 = 0x3ade64;
pub const TITLE_OPENING_DIRECTORY_COUNT: usize = 19;
pub const BATTLE_DIRECTORY_ROOT: u32 = 0x40d300;
pub const BATTLE_DIRECTORY_COUNT: usize = 103;
pub const BATTLE_DIRECTORY_SHA256: &str = "2c35f140bd943bb98726767c2a4032be05dee531b2c965b727a3dce867cd27da";

/// Count/payload pairs observed on the EUR Soil Evidence continuation.
/// These selected lists do not imply a complete producer or archive domain.
pub const SOIL_EVIDENCE_LISTS: [(u32, usize, &str); 20] = [
    (0x6c1768, 1, "c7643215665196f1a27786dda0524ae54d33b322e2590e46fd4f13ab58c6f33a"),
    (0x6c1770, 3, "16d09b6ddd897559eb634d5966cbe1f31b58ab2f403702e826ab218f149e07eb"),
    (0x6c1784, 4, "cb1e24aff52dc673b1ae52c9863b7815eca234e0733706853ff69495446036c4"),
    (0x6c179e, 5, "e726e3e689800762be87512020775007e5ede8fa814108dba29a6f55897616d5"),
    (0x6c17be, 5, "46884ac7f5593ffc4077ce2142cb22a4b70bacce1a2a604645cf555beca60205"),
    (0x6c17de, 6, "5cefcc083846e790ca3d6055bc97715ba417067d1a16aa65efbb14fdca044662"),
    (0x6c1804, 7, "4d3ba0da1862d16b1cdee32776998b7445d2ecb4001ce708699f16e0454600a8"),
    (0x6c1830, 6, "796d1c7732d74b5b91c195b54415021e08d80314082f0f315994e28c56a851a9"),
    (0x6c1856, 7, "adb5ecd8849c8cc51c078b14870b0289b43c7147eb41e443adada55ff55885dd"),
    (0x6c1882, 7, "e7951bde3cf10a16fb68179a4044a5badd25a10381de5dd31b57b3a77d7364e5"),
    (0x6c18ae, 6, "7f378c787e12656e0683cac8ce9c5262b0df4116a2fa2d3d0cf3d2294af17669"),
    (0x6c18d4, 5, "7a245f8dccfa03b0569e19a813999e55204c3158f6c566e0dccb55fae69083a5"),
    (0x6c18f4, 5, "96a881f06516f4bbb8959ec1b737051d404d1b9b8c2704e5b2a48b5494c81d6c"),
    (0x6c1914, 4, "17b872bc51a52a573e96bad980954e1586c881a12a67b0ed2d002679a7b8d815"),
    (0x6c192e, 4, "460ff8b74627dfd2d40b1fe8faaf191133125129957a01c45f1f06268fb7f5bd"),
    (0x6c1948, 4, "79b99f010034430ba11b897188649f4a22ffe7153c7dd77572dfabcdbf4dd608"),
    (0x6c1962, 3, "6ae8b229ed41117f149c116d655869c13b4953b3c90f43707571a99873ed3c93"),
    (0x6c1976, 3, "366e3f7d5852cb7332db5acb3ad83ae9059afcc690c1ecabcf00f78025257646"),
    (0x6c198a, 3, "f60d2e0b48eb79df8137e69466688b2d0cd60597b38c11f5eab43881f993a7d2"),
    (0x6c199e, 3, "5b8a6168f07cb59dde25338a8ec42f35f054487408810a0ea2d4d2f01ffd348f"),
];

/// Count/payload pairs observed on the EUR Firaga battle continuation.
/// These selected lists do not establish a complete producer or archive domain.
pub const FIRAGA_SPRITE_LISTS: [(u32, usize, &str); 44] = [
    (0x418ef8, 1, "06b9a3e4e4b69f0740355269c6c08e3aed2078e016c469fbbb953c809b9fa15d"),
    (0x418f00, 2, "ac888ed9d701d06ddcbd365a06432c5e57e1b6f8cc97d74595961c7c1ff0b3e7"),
    (0x418f10, 2, "1f3f9ab2aa16b272f1d5330f42bffcfad5b4a5f88c8b040f0c493d8861820761"),
    (0x418f20, 2, "87273bd53830625ec7b23b799bff0a1de845921c7aadc9382e32716bd6fe252c"),
    (0x418f30, 3, "852e0d71eace6239910d6c89151106286d93c4590da9a49bcbd47f673517d844"),
    (0x418f44, 3, "63bee83223ebbaad92463d6265d8a35fc151f51c890ee1c2e584e947484e824d"),
    (0x418f58, 4, "820373e91024f608b552579e8ae175d792c8e37135215a14bbd7e843a5ef9399"),
    (0x418f74, 4, "0dcdde33b53230ca3de7a4492600b2cba8235834cd66cc655d6c5afc6c96240c"),
    (0x418f90, 4, "d64bf1a2b3e3e373b8e75536081943e0b5d4f11f89c031185690e023d7cb5d29"),
    (0x418fac, 4, "ad8a1a4c87ccbd3c989d7fa8db8cea44925a66bdb43e357404aeb3327b9aef08"),
    (0x418fc8, 4, "e559efb6ce3ddfb3c54f20324f08df41750e7d030a042cd8c0d3205d2871cef2"),
    (0x418fe4, 4, "7fe20f762f5cf064561a744c348cd99006d4d79c95ad58b0d77905a4ee408493"),
    (0x419000, 5, "2826da249f056ddd04aefdfa7351b46bb812c4bac484a69281b3465a319d1634"),
    (0x419020, 5, "d554aa6ccf82cd8cc8ce6dc8cff2d5848b864bb5a1365498f1d7baf0af01b1d2"),
    (0x419040, 5, "a087ef95fc7214ac468c9dbebb92e25eefee20a9ff623c2ff027c35b8d1b7441"),
    (0x419060, 5, "4df1c0aa0a8680ed584a6d045f19d6f791f4c1466692a85003dcead69a11da2a"),
    (0x419080, 5, "d2303fd932238523ceb8446d5cf1fa9f50e450a017812d401be74a97b833c801"),
    (0x4190a0, 5, "a99cd8ffd331c98d124e9ab848bed368d24a4473186a4ec858fe19e27e0bc8b0"),
    (0x6b3590, 1, "0ed4ee712bd4b54e39d3a6ee3be378a3080495bffcd253f29c6f940f5483334f"),
    (0x6b3598, 2, "82e0e01a4a8fef822802faaae9e2366493d044848e0b0ec4d46c08bc085f4429"),
    (0x6b35a6, 4, "d7dd7e4e89fc5f31ec5e5a03aa3ecaec3c5f2ebf397460d0de5a3511614d5827"),
    (0x6b35c0, 6, "a06a2e72b0dad08d693168fe4ce4223878aebb936a279c98b5f0c8bb96260fbc"),
    (0x6b35e6, 9, "9874756399c7f636e61bedc6577af17d8aa4a1d91d89c716c3cbd2f075f0747a"),
    (0x6b361e, 9, "671bb5e4ad78b47832e6d65fbbfc253f4fcb1f27ce742fdf077c37d898662f60"),
    (0x6b3656, 9, "91ee372a3ecb7ce4890323691869385a7641c4728112b170243a31ceae1e00d3"),
    (0x6b36f4, 18, "6a83d8dff0848c7cb7346440855b322d596c90a02f9da3e0ae8bc971fad3d284"),
    (0x6b3762, 18, "6fa68326acd3977caa3276c6349f112e4e3bde42eba23bed128d06db28f49384"),
    (0x6b37d0, 19, "02fb77871b04faa451ca2ee08e0465d9706ca43649d6f5b57fc94704e9d870ca"),
    (0x6b3844, 19, "9982f83671315ae5566b679218ccaf1be058137bd19079f217e21a258b89132a"),
    (0x6b38b8, 19, "33e098f92448e77f711a5036229d7c4b2daafceb76dcc0ff31a4b75504fec04c"),
    (0x6b392c, 20, "97d54ac6d17327aca61283522118afe564f020d95d2a316835907ceb49ad15ce"),
    (0x6b39a6, 22, "b5aba616f1f34344c441c5dbc013c0c45ad5a2419e76fa6009afd3608f2a5d54"),
    (0x6b3a2c, 22, "8267a7ceccc0723f7d206a450da744bde803d12fb3abff30f84e075759781f22"),
    (0x6b3ab2, 22, "60a1b681068458fc48d096f5e95087bf281ba274918c4db03afad42e5fb0afaf"),
    (0x6b3b3a, 22, "fcd33f197fd2edd28ae503a910dc0a79a56f46dbe73c72add90f19d5ae0e0f98"),
    (0x6c12e0, 3, "4b12f280814093d7b68a1e11f5d18088c848911b91eccc1a2fc62384ff8db009"),
    (0x6c12f4, 3, "f42cbae33391a049f4459046ac319955d95dfdd5b6a5b44329b028b30209b1aa"),
    (0x6c1308, 6, "6b6b6ea048674eebe633d3ad4b3bd4de29b7def30d685ddc2a6aaf312634d4ca"),
    (0x6c132e, 6, "07b7a773125a100e51f4f40a33e50c8396ea70e64763206d95a76b9d3ca22a8b"),
    (0x6c1354, 6, "8f87d292ef8a680a87bf01ca9dd070ef6535a3ffea3cc6204ea6c7e6ee813dd2"),
    (0x6c137a, 6, "db29e376a09f7f2b5f6946c536da41f7c45a13b32ee086885f19a8db5ad42064"),
    (0x6c13a0, 3, "58b96bdbf456c1f26375f4e96424e1db9ab5d1910a5514cfcb7815b2076023c8"),
    (0x6c13b4, 3, "7cec261c12884540afaf81fee89beb6f01980d91e15a0f89ff616a9f02ef184b"),
    (0x6c13c8, 3, "397269b25f0937733edeccfae9b2a677fca1d3acee04db6dd5960771f5c9c5fd"),
];

/// The zero-object list referenced by the selected Firaga animation sequence.
pub const FIRAGA_EMPTY_LIST: (u32, usize, &str) = (
    0x418eec,
    0,
    "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7",
);

/// Empty lists referenced by the selected compact Firaga sequence.
pub const FIRAGA_COMPACT_EMPTY_LISTS: [(u32, usize, &str); 3] = [
    (0x6b368e, 0, "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7"),
    (0x6b3690, 0, "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7"),
    (0x6b3692, 0, "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7"),
];

/// Empty list referenced by the adjacent compact Firaga sequence.
pub const FIRAGA_ADJACENT_EMPTY_LIST: (u32, usize, &str) = (
    0x6b3b38, 0, "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7",
);

/// Empty list referenced by the selected compact Firaga bank at 6c12c8.
pub const FIRAGA_6C12C8_EMPTY_LIST: (u32, usize, &str) = (
    0x6c13dc, 0, "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7",
);

/// Three separately rooted byte groups; the guest retains their interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutOffsets {
    pub first_pair: [u8; 2],
    pub second_pair: [u8; 2],
    pub transform_offsets: [u8; 4],
}
impl LayoutOffsets {
    pub fn source_bytes(&self) -> [u8; LAYOUT_OFFSETS_LENGTH] {
        let mut bytes = [0; LAYOUT_OFFSETS_LENGTH];
        bytes[..2].copy_from_slice(&self.first_pair);
        bytes[2..4].copy_from_slice(&self.second_pair);
        bytes[4..].copy_from_slice(&self.transform_offsets);
        bytes
    }
}

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
    /// Four further physical entries observed in a separate battle continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_lookup_extension: Option<[u16; 4]>,
    /// Absolute ROM pointers copied into the EUR title-opening records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_opening_directory: Option<Vec<u32>>,
    /// Encoded guest Thumb pointers; the live IWRAM functions remain guest-owned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iwram_draw_callbacks: Option<[u32; IWRAM_DRAW_CALLBACKS_COUNT]>,
    /// Independently rooted Thumb callbacks into mutable guest IWRAM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iwram_extra_callbacks: Option<[u32; 2]>,
    /// Independently rooted guest pointers; not a complete callback table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iwram_additional_callbacks: Option<[u32; 7]>,
    /// Original bytes used by the independently rooted OBJ readers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_offsets: Option<LayoutOffsets>,
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
        if self.actor_lookup_extension.is_some() && self.lists.iter().any(|list|
            list.source_offset < ACTOR_LOOKUP_EXTENSION_ROOT + 8 && ACTOR_LOOKUP_EXTENSION_ROOT < list.source_offset + list.source_len() as u32) {
            return Err("Actor lookup extension overlaps an OBJ list".into());
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
        if let Some(callbacks) = &self.iwram_draw_callbacks {
            let start = IWRAM_DRAW_CALLBACKS_ROOT;
            let end = start + (IWRAM_DRAW_CALLBACKS_COUNT * 4) as u32;
            if callbacks.iter().any(|&pointer| pointer & 1 == 0
                || !(0x0300_5f28..0x0300_6d68).contains(&(pointer & !1)))
                || self.lists.iter().any(|list| list.source_offset < end
                    && start < list.source_offset + list.source_len() as u32) {
                return Err("Invalid IWRAM OBJ callback projection".into());
            }
        }
        if let Some(callbacks) = &self.iwram_extra_callbacks {
            for (&root, &pointer) in IWRAM_EXTRA_CALLBACK_ROOTS.iter().zip(callbacks) {
                if pointer & 1 == 0 || !(0x0300_5f28..0x0300_6d68).contains(&(pointer & !1))
                    || self.lists.iter().any(|list| list.source_offset < root + 4
                        && root < list.source_offset + list.source_len() as u32) {
                    return Err("Invalid extra IWRAM OBJ callback projection".into());
                }
            }
        }
        if let Some(callbacks) = &self.iwram_additional_callbacks {
            for (&root, &pointer) in IWRAM_ADDITIONAL_CALLBACK_ROOTS.iter().zip(callbacks) {
                if pointer & 1 == 0 || !(0x0300_5f28..0x0300_6d68).contains(&(pointer & !1))
                    || self.lists.iter().any(|list| list.source_offset < root + 4
                        && root < list.source_offset + list.source_len() as u32) {
                    return Err("Invalid additional IWRAM OBJ callback projection".into());
                }
            }
        }
        if self.layout_offsets.is_some() && self.lists.iter().any(|list|
            list.source_offset < LAYOUT_OFFSETS_ROOT + LAYOUT_OFFSETS_LENGTH as u32
                && LAYOUT_OFFSETS_ROOT < list.source_offset + list.source_len() as u32) {
            return Err("OBJ layout offsets overlap an OBJ list".into());
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
        // Only the physical entries observed in battle are declared.
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
        let start = ACTOR_LOOKUP_EXTENSION_ROOT as usize;
        let raw = &rom[start..start + 8];
        if ffta_rom::digest(raw) != ACTOR_LOOKUP_EXTENSION_SHA256 {
            return Err("Actor lookup extension source identity mismatch".into());
        }
        let actor_lookup_extension = Some(std::array::from_fn(|i|
            u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]])));
        sources.push(SourceRange { offset: ACTOR_LOOKUP_EXTENSION_ROOT, length: 8,
            sha256: ACTOR_LOOKUP_EXTENSION_SHA256.into(), role: "actor-lookup-extension".into() });
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
        for (start, end, expected) in [
            (0x922d2, 0x92302, "ffab5e2951f4f37e600f1968527d3cb851ad13cad37a0b972b8b5d64fdf04d64"),
            (0x923f0, 0x923fa, "5204b54a551f7c30d3385f0156abff4193d3a16a23701635e4f7e0b19e5b40bc"),
            (0x91f58, 0x91f66, "552d4b30fc94e88b8e6f85b8ad6342b54988a538e322b0cdbf973f94a1a8558b"),
            (0x92098, 0x920ae, "da58ab11e2420411a7273b6c6d6caf3d48761d8fa65b911bb638391b4363b99e"),
            (0x925c2, 0x925d2, "7d323a5cf42cef8af143484bab817f6b284bea6a736d001b31058e608b903b5c"),
            (0x924f6, 0x9250c, "d615c45da59229e797580c2430b7f23634ae10a71ccb5341c6f3d9b0f3d4a9b1"),
            (0x145c08, 0x145c1a, "eebaaebafd19249b618c79b603a2da0a61261ceeae1703b9bf2d9afb3eff31c7"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("IWRAM OBJ callback reader or copy mismatch".into());
            }
        }
        let callback_start = IWRAM_DRAW_CALLBACKS_ROOT as usize;
        let callback_bytes = &rom[callback_start..callback_start + IWRAM_DRAW_CALLBACKS_COUNT * 4];
        if ffta_rom::digest(callback_bytes) != IWRAM_DRAW_CALLBACKS_SHA256 {
            return Err("IWRAM OBJ callback source mismatch".into());
        }
        let iwram_draw_callbacks = Some(std::array::from_fn(|index| {
            let at = index * 4;
            u32::from_le_bytes(callback_bytes[at..at + 4].try_into().unwrap())
        }));
        sources.push(SourceRange { offset: IWRAM_DRAW_CALLBACKS_ROOT, length: callback_bytes.len() as u32,
            sha256: ffta_rom::digest(callback_bytes), role: "iwram-obj-draw-callbacks".into() });
        for (start, end, expected) in [
            (0x9b6f2, 0x9b702, "64a9495198d4a6c1f68d57c2fb77a8ce0f0d5b12ff834e0ffeeebea8289d7cd0"),
            (0x9e404, 0x9e428, "56e852ca91b5faff9d1616a319def1725ac6cb59e2aeed46be160d515de6811d"),
            (0xd9757c, 0xd9759c, "a7ccf663d61d891100deaf772317c0409f803368bb4a4ec58b32f705cb5c0184"),
            (0xd976c4, 0xd976e4, "7a3c3042dfe2eafffbe4b0d677d3181dafb0dade66280b332acdfba277001b4d"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Extra IWRAM OBJ callback reader or copy mismatch".into());
            }
        }
        if word(0x9b730)? != 0x0800_0000 + IWRAM_EXTRA_CALLBACK_ROOTS[0]
            || word(0x9e430)? != 0x0800_0000 + IWRAM_EXTRA_CALLBACK_ROOTS[1] {
            return Err("Extra IWRAM OBJ callback literal mismatch".into());
        }
        let mut extra = [0; 2];
        for (index, &offset) in IWRAM_EXTRA_CALLBACK_ROOTS.iter().enumerate() {
            let raw = &rom[offset as usize..offset as usize + 4];
            if ffta_rom::digest(raw) != IWRAM_EXTRA_CALLBACK_SHA256[index] {
                return Err("Extra IWRAM OBJ callback source mismatch".into());
            }
            extra[index] = u32::from_le_bytes(raw.try_into().unwrap());
            sources.push(SourceRange { offset, length: 4, sha256: ffta_rom::digest(raw), role: "iwram-obj-extra-callback".into() });
        }
        let iwram_extra_callbacks = Some(extra);
        // Each pointer has its own literal, original load, branch bridge and
        // copied IWRAM code prefix. This does not imply a complete vector.
        let references = [
            (0x9d8f4, 0x9d828, 0x9d82e, 0x6822, 0x9d830, 0x145c10, "5cc79cc3c0fcba2bbeadeb9751b1c2a204a69e21a801ecc509a29b64ceaaee11", "a101cad40c61ebff12d6aebbbfea292f2e6061c11447772e0983fd5399ff24bf"),
            (0xa2088, 0xa206a, 0xa2074, 0x680c, 0xa207a, 0x145c18, "ef8ac5f858f2d22912f9236972839a1d141aaec814dd42e454adee06b5a6a6b7", "700452420c11ce8115790b63abb1b1f794346d3f6280502ed3dc1a3320ba5385"),
            (0x9d8f8, 0x9d8e6, 0x9d8ec, 0x6812, 0x9d8ee, 0x145c10, "9c9450edeb76f56ba69f125cc300d40f1e1ede97d43ffb3e50fad307eb059256", "98f2114a7fa17de43345e793a66a36074400db65d0c30f23d1cf0adfafc3d80d"),
            (0x9601c, 0x96000, 0x96008, 0x6809, 0x9600a, 0x145c0c, "9f8e60597a5c9ee28a3778ad25d1e114f8f0f2cab56a3df108c3553cc74e04a4", "7af689057d9df7f2274bed7283f59b51c56de176a8b67773086ded36c384c0a3"),
            (0xac7e4, 0xac7a4, 0xac7c8, 0x6824, 0xac7cc, 0x145c18, "bb774155c8f1bb95df88dce51a72a77d0694b25a1daee62234a0e9ea964efa9c", "9d5d8f6db8821d4361a3a3d5818b3b30175bf7d15c933ab6283ebd39caaa5845"),
            (0x9dd78, 0x9dd5c, 0x9dd60, 0x682a, 0x9dd64, 0x145c10, "37ce8ecbf4d2cda0a2b579c8374308920a78ed9bb5e3506ceaee0fb40a6c4ce0", "9475930b7ce05de58ff78317d3ebb264467b12c48c61ccfc5dca983b3882d7f3"),
            (0xd6d14, 0xd6cc0, 0xd6cc8, 0x681b, 0xd6cca, 0x145c14, "f034d982df59525b5a303373c24268f2b2284410ff5c3b85eff02ed0dd60c765", "5dbf50228d3a5ed7a6bc87249ab288d0b91a3b9412e2fd610a4ba07f99effda5"),
        ];
        let mut additional = [0u32; 7];
        for (index, &(literal, reader, load, load_opcode, call, bridge, reader_hash, target_hash)) in references.iter().enumerate() {
            let root = IWRAM_ADDITIONAL_CALLBACK_ROOTS[index];
            let opcode = word(reader)? as u16;
            let halves = word(call)?;
            let (hi, lo) = (halves as u16, (halves >> 16) as u16);
            let displacement = (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
            let pointer = word(root)?;
            let target = (pointer & !1).checked_sub(0x0300_5f28)
                .and_then(|offset| offset.checked_add(0xd968e8))
                .filter(|&at| at + 16 <= 0xd97728)
                .ok_or("Additional callback outside copied IWRAM")? as usize;
            if opcode & 0xf800 != 0x4800
                || ((reader + 4) & !3) + u32::from(opcode & 255) * 4 != literal
                || word(literal)? != 0x0800_0000 + root
                || word(load)? as u16 != load_opcode
                || hi & 0xf800 != 0xf000 || lo & 0xf800 != 0xf800
                || call.wrapping_add(4).wrapping_add_signed(displacement) != bridge
                || pointer & 1 == 0
                || ffta_rom::digest(&rom[reader as usize..call as usize + 4]) != reader_hash
                || ffta_rom::digest(&rom[target..target + 16]) != target_hash {
                return Err("Additional IWRAM callback reader or copied-code mismatch".into());
            }
            additional[index] = pointer;
            sources.push(SourceRange { offset: root, length: 4,
                sha256: ffta_rom::digest(&rom[root as usize..root as usize + 4]),
                role: "iwram-obj-additional-callback".into() });
        }
        let iwram_additional_callbacks = Some(additional);
        for (start, end, expected) in [
            (0x9ae58, 0x9ae90, "3a6689a01f7bd7bd9665e81cc693ac5c7260d0c6ee9dc655d34bff015f87cf44"),
            (0x9bfdc, 0x9c038, "07912b37f8837e18ec9266c55d4e2d938b330c34dbd22966c812054b0ff38ea6"),
            (0x9c048, 0x9c0c4, "ba5f3c12c5879623b8a07ec9204819ce3b9d68ee39d0d97aeb30c212523cca31"),
            (0x9e64a, 0x9e668, "6e22605062831783f0c6d01dad2295bdc65df5e517fd780700baebac7cac09f9"),
            (0xa3320, 0xa3354, "c135ca374af86bb78bea8bdf230fe2b551594daabc4c3605f65c86501d39d3ea"),
            (0xa339a, 0xa33b8, "073346fe38a1210451eb673da885434cb8ddbe357f0e3c9488280de2b79d827b"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected { return Err("OBJ layout offset reader mismatch".into()); }
        }
        for (at, expected) in [(0x9ae8c, LAYOUT_OFFSETS_ROOT + 2), (0x9c034, LAYOUT_OFFSETS_ROOT + 2),
            (0x9c0c0, LAYOUT_OFFSETS_ROOT), (0x9e724, LAYOUT_OFFSETS_ROOT + 4)] {
            if word(at)? != 0x0800_0000 + expected { return Err("OBJ layout offset literal mismatch".into()); }
        }
        let start = LAYOUT_OFFSETS_ROOT as usize; let bytes = &rom[start..start + LAYOUT_OFFSETS_LENGTH];
        if ffta_rom::digest(bytes) != LAYOUT_OFFSETS_SHA256 { return Err("OBJ layout offset source mismatch".into()); }
        let layout_offsets = Some(LayoutOffsets { first_pair: bytes[..2].try_into().unwrap(),
            second_pair: bytes[2..4].try_into().unwrap(), transform_offsets: bytes[4..].try_into().unwrap() });
        sources.push(SourceRange { offset: LAYOUT_OFFSETS_ROOT, length: LAYOUT_OFFSETS_LENGTH as u32,
            sha256: ffta_rom::digest(bytes), role: "obj-layout-offsets".into() });
        // These exact lists have count/payload observations in Soil Evidence;
        // their pointer producer and the surrounding archive remain unbounded.
        for (offset, count, expected) in SOIL_EVIDENCE_LISTS {
            let length = 2 + count * 6;
            let bytes = rom.get(offset as usize..offset as usize + length)
                .ok_or("Truncated Soil Evidence OBJ list")?;
            if ffta_rom::digest(bytes) != expected {
                return Err("Soil Evidence OBJ list source mismatch".into());
            }
            let list = List::decode(offset, bytes)?;
            if list.objects.len() != count || list.objects.iter().any(|object| object.shape >= 3) {
                return Err("Soil Evidence OBJ list count or shape mismatch".into());
            }
            lists.push(list);
            sources.push(SourceRange { offset, length: length as u32,
                sha256: expected.into(), role: "static-obj-list".into() });
        }
        // These exact lists have count/payload observations in the Firaga
        // continuation; their pointer producer and archive remain unbounded.
        for (offset, count, expected) in FIRAGA_SPRITE_LISTS.into_iter()
            .chain([FIRAGA_EMPTY_LIST]).chain(FIRAGA_COMPACT_EMPTY_LISTS).chain([FIRAGA_ADJACENT_EMPTY_LIST, FIRAGA_6C12C8_EMPTY_LIST]) {
            let length = 2 + count * 6;
            let bytes = rom.get(offset as usize..offset as usize + length)
                .ok_or("Truncated Firaga OBJ list")?;
            if ffta_rom::digest(bytes) != expected {
                return Err("Firaga OBJ list source mismatch".into());
            }
            let list = List::decode(offset, bytes)?;
            if list.objects.len() != count || list.objects.iter().any(|object| object.shape >= 3) {
                return Err("Firaga OBJ list count or shape mismatch".into());
            }
            lists.push(list);
            sources.push(SourceRange { offset, length: length as u32,
                sha256: expected.into(), role: "static-obj-list".into() });
        }
        lists.sort_by_key(|list| list.source_offset);
        sources.sort_by_key(|source| source.offset);
        let decoded = Lists { lists, battle_directory, actor_lookup, actor_lookup_extension,
            title_opening_directory: Some(title_opening_directory), iwram_draw_callbacks,
            iwram_extra_callbacks, iwram_additional_callbacks, layout_offsets };
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
            actor_lookup: None, actor_lookup_extension: None, title_opening_directory: None, iwram_draw_callbacks: None, iwram_extra_callbacks: None, iwram_additional_callbacks: None, layout_offsets: None,
        }
        .validate()
        .is_err());
        let mut invalid = list;
        invalid.objects[0].x = 256;
        assert!(invalid.validate().is_err());
        assert!(Catalog::from_rom(&bytes).is_err());
    }
}
