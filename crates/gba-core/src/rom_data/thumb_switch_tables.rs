//! EUR Thumb switch-table roots and destinations, authenticated against original readers.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_COUNT: usize = 23;
pub const TOTAL_DESTINATIONS: usize = 696;
pub const SOURCE_ROLE: &str = "thumb-switch-table";

#[derive(Clone, Copy, Debug)]
pub struct TableSpec {
    pub root: u32,
    pub count: usize,
    pub guard: u32,
    pub reader: u32,
    pub mov: u32,
    pub special: bool,
    pub code_sha256: &'static str,
    pub source_sha256: &'static str,
}
impl TableSpec {
    pub fn end(self) -> u32 { self.root + 4 + (self.count as u32) * 4 }
}

// Physical source bounds follow the original unsigned guards and switch readers.
pub const TABLES: [TableSpec; TABLE_COUNT] = [
    TableSpec { root: 0x11974, count: 12, guard: 0x11962, reader: 0x11968, mov: 0x11970, special: false, code_sha256: "59922697cd7c5b999b2c025c6d8db0fa79186d12fae3f73491f5c55c2d2fd176", source_sha256: "0fcbc255633bf674f40d3b4490b308616ff380ad0972041bc422b4efcf7137cd" },
    TableSpec { root: 0x12274, count: 89, guard: 0x1225e, reader: 0x12266, mov: 0x1226e, special: false, code_sha256: "af183d688b81e67335b2b181211eef1c01fee908874fa75d05ce184e58ac7ccd", source_sha256: "75f55193727469a7b08ca467b1575d38ac5c12cfaf8f01ab442c90b7aaafc631" },
    TableSpec { root: 0x14198, count: 89, guard: 0x14186, reader: 0x1418c, mov: 0x14194, special: false, code_sha256: "83d06bcfa111d28ca57661f5fd57abe0d10f91d00f68cb420b303ac04ad7a4ee", source_sha256: "53d6270dd6e6da068c2d4bbe3fe51e058b98ed9e7db20364d79c377aac055f29" },
    TableSpec { root: 0x14620, count: 89, guard: 0x14610, reader: 0x14616, mov: 0x1461e, special: false, code_sha256: "310cb4be627e091c1883071c06ff42df4d36f1854a23907abf461f7bbeb71283", source_sha256: "16ce6b8e1cf693e0b47c098c0c842df68b052fbfb8f26fe45d49c8bec943fbb6" },
    TableSpec { root: 0x191a4, count: 26, guard: 0x1918e, reader: 0x19194, mov: 0x1919c, special: false, code_sha256: "ad5ccfe50cf5f12bb12977b8b34995d1a9ff175c8baff7f48f3cd24f3b866789", source_sha256: "f9c9da087b11767f4da823de96698750b39143a50f3ae9564eba28a0548fd61d" },
    TableSpec { root: 0x22270, count: 8, guard: 0x22260, reader: 0x22266, mov: 0x2226e, special: false, code_sha256: "6d99c59be5d57735e837629376b8c1f765fede91065808c0b1d1197d0a19b8cf", source_sha256: "ee06cd4896eebf1176858b7e755b769cf3cf88ac0f29939b1e08e6530a22163e" },
    TableSpec { root: 0x25408, count: 35, guard: 0x253f2, reader: 0x253f8, mov: 0x25400, special: false, code_sha256: "f6fb37231d76f5a137d43e0dfc982b11e218f832123f79214f32de787c447941", source_sha256: "87f3de9fb6e26146f1418bc108ca391cefddb56bbe5cc52e6d9c68bb43ddc375" },
    TableSpec { root: 0x9be04, count: 19, guard: 0x9bdf4, reader: 0x9bdf8, mov: 0x9be00, special: false, code_sha256: "6634b59d5f05d4423b5d5a8223ee32aa7902534e246c672ab74353d3a828679e", source_sha256: "a66318b5fed9d1a45150c9d167c0f1b00a88a987e0fc131bffd2eb7033916a08" },
    TableSpec { root: 0x9be88, count: 12, guard: 0x9be76, reader: 0x9be7c, mov: 0x9be84, special: false, code_sha256: "886ed2fca237a27fbbe0062d52390a2f9928af0bd23cee76e4a06b5c57a30470", source_sha256: "7a88424b2ccfa9821d1e69c8ac08633bae4079e698776701ba22e5facab527ed" },
    TableSpec { root: 0x9bf10, count: 5, guard: 0x9bf02, reader: 0x9bf06, mov: 0x9bf0e, special: false, code_sha256: "c02b96ce64e8d5e38e0bbb82211fca44050b1674605f05f7632e3192a7454dd4", source_sha256: "bb9ef2a5d28b5014892019e382713c502ab57a10d22f37364cce03c183384d8a" },
    TableSpec { root: 0xa114c, count: 24, guard: 0xa113a, reader: 0xa1140, mov: 0xa1148, special: false, code_sha256: "e79a95ad3eaae421fe55833d517c8d6ed51350b69f54f71da9efae746d1b1dfb", source_sha256: "2c11c5826f7988bd308878916bdc3715efa2235021c70118432653a256db7b08" },
    TableSpec { root: 0xa556c, count: 8, guard: 0xa555c, reader: 0xa5562, mov: 0xa556a, special: false, code_sha256: "c5cd35f6517a1c75fb44f1d9bca5df6406fc74d9ec09cc8065b3f49e05b05586", source_sha256: "78b2b5f4c31449bec11d12a942adaa89c54a8a83d8c3987fa6ccb0e7b13b7e68" },
    TableSpec { root: 0xa8fd0, count: 19, guard: 0xa8fc2, reader: 0xa8fc6, mov: 0xa8fce, special: false, code_sha256: "6ea046ca917681ac2a8fc3682dfbfeb060c7b467632a6beb44581a5c899a7492", source_sha256: "03d569b83acca2e50491a3a096d3353cf64c5b455f0f5a64d738b80b52d4fcac" },
    TableSpec { root: 0xa9548, count: 37, guard: 0xa9524, reader: 0xa952c, mov: 0xa9534, special: false, code_sha256: "b5c40ab887b3de00551d36ccc53d66971ddadba29538f7dd361909e28e3318b8", source_sha256: "ebeedcbfcc0aa6116e6b453f4a700c11aae72e0233eb2f385436e16f69ade025" },
    TableSpec { root: 0xac01c, count: 8, guard: 0xac00c, reader: 0xac010, mov: 0xac018, special: false, code_sha256: "360c48a8f1d1490b295373d5e2a33339a38b052ab6a48538fbbb31a818599119", source_sha256: "f1c1b6378870b926f54d33f5960750fb4a9f480be9698ef5cfe0d0970f35ce1f" },
    TableSpec { root: 0xb94dc, count: 32, guard: 0xb94c6, reader: 0xb94ce, mov: 0xb94d6, special: false, code_sha256: "757e8321aa082826e8d864744fdc1fdac539179ba795dcb47d1c5575c35555c1", source_sha256: "a1b82940022660bc62bbe344fddd811bac3814f45f3564d9539e9726b66592cd" },
    TableSpec { root: 0xbf058, count: 13, guard: 0xbf046, reader: 0xbf04c, mov: 0xbf054, special: false, code_sha256: "926a1ebf9de30219f028c1ed976183c3e8aa8737e11597728ce4a16bfa3148d2", source_sha256: "e83f297f411db837aebad1cbb1e9aa46370395cdd0e7fbd22f5b85cec4847a87" },
    TableSpec { root: 0xcb814, count: 69, guard: 0xcb804, reader: 0xcb80a, mov: 0xcb812, special: false, code_sha256: "572f7a74ba62f8013bafd3c0a9c889ba6b3ece86940790054eaeb89c066badd0", source_sha256: "3a7ae3142d6c62d901c9a503b28e271f583e1f1ddd34c1b8fdf82cbf327d3e80" },
    TableSpec { root: 0xcbef4, count: 48, guard: 0xcbee2, reader: 0xcbee0, mov: 0xcbeec, special: true, code_sha256: "97f2ab10a3f5bd0f0e0d2232d3232d6c16abd0e33a16eea15c95f0e649992873", source_sha256: "d9df46574e8482ebfa3f7cea06952b67b64dcc376bad6ae8795cdbe64932b169" },
    TableSpec { root: 0xdb2cc, count: 6, guard: 0xdb2bc, reader: 0xdb2c0, mov: 0xdb2c8, special: false, code_sha256: "22b279bba637729dfe469127b84971a92ddd7a5abf1762a0f4c364d2188fee1d", source_sha256: "5450ddf61bd8b11b67f11b7d486936c68aae387e73c7795b1155d872398f4e31" },
    TableSpec { root: 0xe3188, count: 23, guard: 0xe317a, reader: 0xe317e, mov: 0xe3186, special: false, code_sha256: "d3667cfc57101f574d7ff28c570a482247dc6afb88cf11022050c6e5d57bea48", source_sha256: "39ca2fd32c9f6b5f7568fa9745363b65aab8aabedce8caec76774c65b7424fc4" },
    TableSpec { root: 0x12542c, count: 5, guard: 0x12541e, reader: 0x125422, mov: 0x12542a, special: false, code_sha256: "0b969313d12d651450cb7b549e091ff01778aec691d419b137c9ea411a4688ff", source_sha256: "22de0b44c9765d5c1255bcaf6077122ca1781ed5e907a3aa5a28189616bfbda2" },
    TableSpec { root: 0x1380a0, count: 20, guard: 0x13808e, reader: 0x138094, mov: 0x13809c, special: false, code_sha256: "b6024e25c9e2dbd748be5c1e6d2aa361eafd5e0ce4c85de063a10c53216fe773", source_sha256: "a7f5c582c87f00f15eb7a05553eb0040dc5caaf385e7ab16ee4eaa8dd2653bb7" },
];


/// Additional battle and home tables use roots owned by Thumb PC literals.
/// Only destinations are new sources; guest out-of-range behavior remains live.
pub const ADDITIONAL_TABLE_COUNT: usize = 6;
pub const ADDITIONAL_DESTINATION_COUNT: usize = 198;
pub const ADDITIONAL_SOURCE_ROLE: &str = "thumb-switch-destinations";

#[derive(Clone, Copy, Debug)]
pub struct AdditionalTableSpec {
    pub root: u32,
    pub count: usize,
    pub code_start: u32,
    pub guard: u32,
    pub reader: u32,
    pub mov: u32,
    pub code_sha256: &'static str,
    pub source_sha256: &'static str,
}
impl AdditionalTableSpec {
    pub fn start(self) -> u32 { self.root + 4 }
    pub fn end(self) -> u32 { self.start() + self.count as u32 * 4 }
}

// Counts follow original selector comparisons; no guest index cap is added.
// At 0960da the out-of-range helper may return, so reads beyond this projection
// deliberately continue through the other owners or the original backing ROM.
pub const ADDITIONAL_TABLES: [AdditionalTableSpec; ADDITIONAL_TABLE_COUNT] = [
    AdditionalTableSpec { root: 0x960ec, count: 127, code_start: 0x960da, guard: 0x960da, reader: 0x960e2, mov: 0x960ea, code_sha256: "1da3401798e2dcb3dc32e814c1f42bca78fb28ba0b07c68335097b446dd5c481", source_sha256: "fd5a42f3ed75bc05188a750d4b5307977c48cf0bb3adcffdfc656cab8975aa3a" },
    AdditionalTableSpec { root: 0x9bacc, count: 12, code_start: 0x9baba, guard: 0x9baba, reader: 0x9bac0, mov: 0x9bac8, code_sha256: "5b5a4ee5487f10dd6949bc76a08dbe93fa3850d2b4af7641edca716d0e0b37fe", source_sha256: "375b9ee5c4a70a93139821719f10a261ba7db9ceccfe64a7f134362693e4af91" },
    AdditionalTableSpec { root: 0xac2e4, count: 5, code_start: 0xac2c4, guard: 0xac2cc, reader: 0xac2d0, mov: 0xac2d6, code_sha256: "ee6d8780350f6903451bc45902ca98e3d854d3cfc9099a8a486d3df8357cd701", source_sha256: "1fd7ecf8dc5343eb28e647413b516c70bdc7a268ffff92e19b0c22e8b2b30498" },
    AdditionalTableSpec { root: 0xce210, count: 19, code_start: 0xce1fc, guard: 0xce1fc, reader: 0xce202, mov: 0xce20a, code_sha256: "b4dbb6272a609edec70de4595e5c9a216512625549a290c39ff19553690923c3", source_sha256: "8bda2ee3ee78592db066ed81f98c265b740eb403dac217f914aed889fc07845d" },
    AdditionalTableSpec { root: 0x9efec, count: 28, code_start: 0x9efda, guard: 0x9efde, reader: 0x9efe2, mov: 0x9efea, code_sha256: "ce6b2bed9506302bc11be09268e52545c5d2d853904c48fc200e03aec9681350", source_sha256: "0582cc7d3478cbe4d1ebcd2e4a2ba2f1433f0c06079dcdf9c068f5500a2f9926" },
    AdditionalTableSpec { root: 0x9f078, count: 7, code_start: 0x9f066, guard: 0x9f068, reader: 0x9f06e, mov: 0x9f076, code_sha256: "99f790c7098ca61a01eab744839576815e3631012c72583ded91db72b3b0b8f0", source_sha256: "8752cd336fd222cf89576e352c47275fc0a01ef3ca4ebf733ac398984e5c3f3e" },
];

/// Even target address used by the original Thumb `MOV pc` instruction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThumbDestination {
    pub address: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwitchTable {
    pub root_offset: u32,
    pub table_offset: u32,
    pub destinations: Vec<ThumbDestination>,
}
impl SwitchTable {
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.destinations.len() * 4);
        out.extend_from_slice(&(0x0800_0000u32 + self.table_offset).to_le_bytes());
        out.extend_from_slice(&self.destination_bytes());
        out
    }

    pub fn destination_bytes(&self) -> Vec<u8> {
        self.destinations.iter().flat_map(|destination| destination.address.to_le_bytes()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThumbSwitchTables {
    pub tables: Vec<SwitchTable>,
    pub additional_tables: Vec<SwitchTable>,
}
impl ThumbSwitchTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.tables.len() != TABLE_COUNT { return Err("Invalid EUR Thumb switch-table count".into()); }
        for (table, spec) in self.tables.iter().zip(TABLES) {
            if table.root_offset != spec.root || table.table_offset != spec.root + 4
                || table.destinations.len() != spec.count
                || table.destinations.iter().any(|d| d.address & 1 != 0
                    || !(0x0800_0000..0x0820_0000).contains(&d.address)) {
                return Err("Invalid EUR Thumb switch-table source, target or order".into());
            }
        }
        if self.additional_tables.len() != ADDITIONAL_TABLE_COUNT {
            return Err("Invalid additional EUR Thumb switch-table count".into());
        }
        for (table, spec) in self.additional_tables.iter().zip(ADDITIONAL_TABLES) {
            if table.root_offset != spec.root || table.table_offset != spec.start()
                || table.destinations.len() != spec.count
                || table.destinations.iter().any(|d| d.address & 1 != 0
                    || !(0x0800_0000..0x0820_0000).contains(&d.address)) {
                return Err("Invalid additional EUR Thumb switch-table source, target or order".into());
            }
        }
        Ok(())
    }

    /// Returns a byte from a root or target; guest guards and control flow stay original.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, bool)> {
        for (index, table) in self.tables.iter().enumerate() {
            let Some(relative) = offset.checked_sub(table.root_offset) else { continue };
            let relative = relative as usize;
            if relative < 4 {
                return Some(((0x0800_0000 + table.table_offset).to_le_bytes()[relative], index, true));
            }
            let target = (relative - 4) / 4;
            if let Some(destination) = table.destinations.get(target) {
                return Some((destination.address.to_le_bytes()[(relative - 4) % 4], index, false));
            }
        }
        for (index, table) in self.additional_tables.iter().enumerate() {
            let Some(relative) = offset.checked_sub(table.table_offset) else { continue };
            let relative = relative as usize;
            if let Some(destination) = table.destinations.get(relative / 4) {
                return Some((destination.address.to_le_bytes()[relative % 4], TABLE_COUNT + index, false));
            }
        }
        None
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Thumb switch tables differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ThumbSwitchTables,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Thumb switch tables require exact EUR ROM; USA pending".into());
        }
        let half = |at: u32| u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        let word = |at: u32| u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
        let prior = PriorSources::new(previous)?;
        let mut tables = Vec::with_capacity(TABLE_COUNT);
        let mut sources = Vec::with_capacity(TABLE_COUNT + ADDITIONAL_TABLE_COUNT);
        for spec in TABLES {
            let code_start = spec.guard.min(spec.reader) as usize;
            if ffta_rom::digest(&rom[code_start..spec.mov as usize + 2]) != spec.code_sha256
                || half(spec.mov) != 0x4687 || word(spec.root) != 0x0800_0000 + spec.root + 4 {
                return Err("Thumb switch reader or root fingerprint mismatch".into());
            }
            if spec.special {
                // This reader keeps an out-of-range selector in its original compare loop.
                if half(spec.reader) != 0x4e04 || half(spec.guard) != 0x2d2f
                    || half(spec.guard + 2) != 0xd8fd || half(spec.guard + 4) != 0x00a8
                    || half(spec.guard + 6) != 0x1980 || half(spec.guard + 8) != 0x6800 {
                    return Err("Special Thumb switch guard mismatch".into());
                }
            } else {
                let compare = half(spec.guard);
                let branch = half(spec.guard + 2);
                let signed_displacement = (branch as u8 as i8 as i32) * 2;
                let target = (spec.guard as i64 + 6 + i64::from(signed_displacement)) as u32;
                let ldr = half(spec.reader + 2);
                if compare & 0xff00 != 0x2800 || usize::from(compare & 255) + 1 != spec.count
                    || !((branch & 0xff00 == 0xd900 && target == spec.reader)
                        || (branch & 0xff00 == 0xd800 && target >= spec.end()))
                    || half(spec.reader) != 0x0080 || ldr & 0xff00 != 0x4900
                    || ((spec.reader + 6) & !3) + u32::from(ldr & 255) * 4 != spec.root
                    || half(spec.reader + 4) != 0x1840 || half(spec.reader + 6) != 0x6800 {
                    return Err("Thumb switch guard, base or load mismatch".into());
                }
            }
            let raw = &rom[spec.root as usize..spec.end() as usize];
            if ffta_rom::digest(raw) != spec.source_sha256 || prior.contains(spec.root, raw.len())? {
                return Err("Thumb switch source fingerprint or owner mismatch".into());
            }
            let destinations = (0..spec.count).map(|i| ThumbDestination {
                address: word(spec.root + 4 + i as u32 * 4),
            }).collect();
            let table = SwitchTable { root_offset: spec.root, table_offset: spec.root + 4, destinations };
            if table.source_bytes() != raw { return Err("Thumb switch reconstruction mismatch".into()); }
            sources.push(SourceRange { offset: spec.root, length: raw.len() as u32,
                sha256: spec.source_sha256.into(), role: SOURCE_ROLE.into() });
            tables.push(table);
        }
        let mut additional_tables = Vec::with_capacity(ADDITIONAL_TABLE_COUNT);
        for spec in ADDITIONAL_TABLES {
            if ffta_rom::digest(&rom[spec.code_start as usize..spec.mov as usize + 2]) != spec.code_sha256
                || half(spec.mov) != 0x4687 || word(spec.root) != 0x0800_0000 + spec.start() {
                return Err("Additional Thumb switch reader or root fingerprint mismatch".into());
            }
            let raw = &rom[spec.start() as usize..spec.end() as usize];
            if ffta_rom::digest(raw) != spec.source_sha256 || prior.contains(spec.start(), raw.len())?
                || TABLES.iter().any(|old| spec.start() < old.end() && old.root < spec.end()) {
                return Err("Additional Thumb switch source fingerprint or owner mismatch".into());
            }
            let destinations = (0..spec.count).map(|i| ThumbDestination {
                address: word(spec.start() + i as u32 * 4),
            }).collect();
            let table = SwitchTable { root_offset: spec.root, table_offset: spec.start(), destinations };
            if table.destination_bytes() != raw { return Err("Additional Thumb switch reconstruction mismatch".into()); }
            sources.push(SourceRange { offset: spec.start(), length: raw.len() as u32,
                sha256: spec.source_sha256.into(), role: ADDITIONAL_SOURCE_ROLE.into() });
            additional_tables.push(table);
        }
        let decoded = ThumbSwitchTables { tables, additional_tables };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}
