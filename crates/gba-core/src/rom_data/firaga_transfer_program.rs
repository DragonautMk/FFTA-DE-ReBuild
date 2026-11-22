//! Authenticated controls for one EUR Firaga transfer program.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_LZ_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSETS: [u32; 12] = [0x0ebb08, 0x39cbd4, 0x39cbda, 0x39cb9c,
    0x39cafc, 0x39caec, 0x39cb04, 0x39cb2c, 0x39cb34, 0xaf565c, 0xaf4994, 0xaf5450];
pub const SOURCE_LENGTHS: [usize; 12] = [8, 4, 1, 48, 8, 16, 8, 8, 8, 4, 4, 4];
const ROLES: [&str; 12] = ["firaga-transfer-caller-literals", "firaga-transfer-program-pointer",
    "firaga-transfer-program-count", "firaga-transfer-records", "firaga-transfer-palette-wrapper",
    "firaga-transfer-palette-entries", "firaga-transfer-lz-wrapper-0", "firaga-transfer-lz-wrapper-1",
    "firaga-transfer-lz-wrapper-2", "firaga-transfer-format-header-0", "firaga-transfer-format-header-1",
    "firaga-transfer-format-header-2"];
const HASHES: [&str; 12] = [
    "afe165c64bdfa9cac6e9b587da2b307f3283cc054bca371029103b8a4ca064ef",
    "72c5f664be70453d086f137493bed55fd7250264d0adf27efb45438a962424ec",
    "e52d9c508c502347344d8c07ad91cbd6068afc75ff6292f062a09ca381c89e71",
    "879ca97c64fd3cb83d521da99324f40f90b982c875b356079e33a968471a9fd5",
    "8938f732838f009be3210910ffab2519ca3a78f5aab3a6098d6f82ed0f365346",
    "3a5416f7e4c426d7ca3e1ffeb6072cbb5ac778e586b09b1f2e48c38e3506e5b9",
    "4079c53e62bf31e100f00233587e6c24742eb3b394360d8c34bb26a07f3dcefc",
    "c9e8eb1fc74d9868f39cbd91c531e3a578b0782e2eff11967fa26c9971a34296",
    "096ce6b2e9e0b25641700c468878a79e7294828b81312abdce4543d95d91d91e",
    "6554507a2423f81b672d1cc607ed142449ed621e72ec74dec669d31b7730951e",
    "6554507a2423f81b672d1cc607ed142449ed621e72ec74dec669d31b7730951e",
    "8a53a04d258d253d29a4035818bbe3fb8130138a0fe327563e42e0ff020de37e",
];
const PRIOR_LZ: [(u32, u32, &str); 3] = [
    (0xaf5660, 29, "d0c14fd944f302b7970694a8b1d2df2dd15a2e7dce7e0f5a491bb749818a12e0"),
    (0xaf4998, 2744, "2cadde0263ee4f58e84479fe1383d607a27e44af2f9b9837538771388f7f8ad6"),
    (0xaf5454, 517, "e8cf9c78d56245c0a4f1178e74c60afeef3663649fb73a6ce0906aa646319e2e"),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferRecord { pub kind: u8, pub flags: u8, pub parameter: u16, pub payload: u32, pub destination: u32 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteEntry { pub source: u32, pub parameters: [u8; 4] }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LzWrapper { pub header: u32, pub argument: u32 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatHeader { pub kind: u8, pub flags: u8, pub reserved: u16 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaTransferProgram {
    pub caller_literals: [u32; 2],
    pub program_pointer: u32,
    pub program_count: u8,
    pub records: [TransferRecord; 4],
    pub palette_count: u8,
    pub palette_reserved: [u8; 3],
    pub palette_entries_pointer: u32,
    pub palette_entries: [PaletteEntry; 2],
    pub lz_wrappers: [LzWrapper; 3],
    pub format_headers: [FormatHeader; 3],
}
impl FiragaTransferProgram {
    pub fn validate(&self) -> Result<(), String> {
        let record_shape: Vec<_> = self.records.iter().map(|r| (r.kind, r.flags, r.parameter, r.payload, r.destination)).collect();
        if self.caller_literals != [0x0839cbd8,0x0839cbcc] || self.program_pointer != 0x0839cb9c || self.program_count != 4
            || record_shape != [(2,0,0,0x0839cafc,0), (3,0,0x60,0x0839cb04,0x06017f80),
                (3,0,0x2820,0x0839cb2c,0x0600d7c0), (3,0,0x0c40,0x0839cb34,0x06017340)]
            || self.palette_count != 2 || self.palette_reserved != [0;3] || self.palette_entries_pointer != 0x0839caec
            || self.palette_entries.iter().map(|e| (e.source,e.parameters)).collect::<Vec<_>>() != [(0x08ae6db8,[7,1,0,1]),(0x08ae6db0,[6,1,0,1])]
            || self.lz_wrappers.iter().map(|w| (w.header,w.argument)).collect::<Vec<_>>() != [(0x08af565c,0),(0x08af4994,0),(0x08af5450,3)]
            || self.format_headers.iter().map(|h| (h.kind,h.flags,h.reserved)).collect::<Vec<_>>() != [(2,2,0x7ffd),(2,2,0x7ffd),(1,2,0x7ffd)] {
            return Err("Invalid Firaga transfer program contract".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut payloads = Vec::with_capacity(12);
        payloads.push(self.caller_literals.iter().flat_map(|x| x.to_le_bytes()).collect());
        payloads.push(self.program_pointer.to_le_bytes().to_vec());
        payloads.push(vec![self.program_count]);
        let mut records = Vec::new();
        for r in &self.records { records.extend([r.kind,r.flags]); records.extend(r.parameter.to_le_bytes()); records.extend(r.payload.to_le_bytes()); records.extend(r.destination.to_le_bytes()); }
        payloads.push(records);
        let mut wrapper=vec![self.palette_count]; wrapper.extend(self.palette_reserved); wrapper.extend(self.palette_entries_pointer.to_le_bytes()); payloads.push(wrapper);
        let mut entries=Vec::new(); for e in &self.palette_entries { entries.extend(e.source.to_le_bytes()); entries.extend(e.parameters); } payloads.push(entries);
        for w in &self.lz_wrappers { let mut bytes=w.header.to_le_bytes().to_vec(); bytes.extend(w.argument.to_le_bytes()); payloads.push(bytes); }
        for h in &self.format_headers { let mut bytes=vec![h.kind,h.flags]; bytes.extend(h.reserved.to_le_bytes()); payloads.push(bytes); }
        Ok(SOURCE_OFFSETS.into_iter().zip(payloads).zip(ROLES).map(|((offset,bytes),role)| (offset,bytes,role)).collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded { return Err("Firaga transfer program differs from authenticated EUR source".into()); }
        Ok(())
    }
}
pub struct Catalog { pub decoded: FiragaTransferProgram, pub sources: Vec<SourceRange> }
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len()!=0x0100_0000 || ffta_rom::digest(rom)!=ffta_rom::EUROPE_SHA256 { return Err("Firaga transfer program requires exact EUR ROM; USA pending".into()); }
        let half=|at:usize| u16::from_le_bytes(rom[at..at+2].try_into().unwrap());
        let word=|at:usize| u32::from_le_bytes(rom[at..at+4].try_into().unwrap());
        let checks=[(0xebade,0x480a),(0xebaee,0x4907),(0xebaf4,0x6801),(0xebb00,0xf7f8),(0xebb02,0xfa02),
            (0xdb4a4,0x0048),(0xdb4a6,0x1840),(0xdb4a8,0x0080),(0xdb4aa,0x1882),
            (0xdb4f2,0xf7ff),(0xdb4f4,0xfdd3),(0xdb4fe,0xf7ff),(0xdb500,0xfed3),(0xdb510,0xf7ff),(0xdb512,0xff30)];
        if checks.iter().any(|&(at,value)|half(at)!=value) || word(0xe4028)!=0x080e3a15 || word(0xdb5a8)!=0x080db42d {
            return Err("Firaga transfer program reader fingerprint mismatch".into());
        }
        let decoded=FiragaTransferProgram {
            caller_literals: [word(0xebb08),word(0xebb0c)], program_pointer:word(0x39cbd4), program_count:rom[0x39cbda],
            records:std::array::from_fn(|i| {let a=0x39cb9c+12*i; TransferRecord {kind:rom[a],flags:rom[a+1],parameter:half(a+2),payload:word(a+4),destination:word(a+8)}}),
            palette_count:rom[0x39cafc], palette_reserved:rom[0x39cafd..0x39cb00].try_into().unwrap(), palette_entries_pointer:word(0x39cb00),
            palette_entries:std::array::from_fn(|i| {let a=0x39caec+8*i; PaletteEntry {source:word(a),parameters:rom[a+4..a+8].try_into().unwrap()}}),
            lz_wrappers:std::array::from_fn(|i| {let a=[0x39cb04,0x39cb2c,0x39cb34][i]; LzWrapper {header:word(a),argument:word(a+4)}}),
            format_headers:std::array::from_fn(|i| {let a=[0xaf565c,0xaf4994,0xaf5450][i]; FormatHeader {kind:rom[a],flags:rom[a+1],reserved:half(a+2)}}),
        };
        let prior=PriorSources::new(previous)?;
        if !previous.is_empty() {
            let lz=previous.iter().find(|r|r.id==OBSERVED_LZ_ID).ok_or("Missing Firaga LZ stream owner")?;
            for (offset,length,hash) in PRIOR_LZ {
                if !lz.sources.iter().any(|s| s.offset==offset && s.length==length && s.sha256==hash) { return Err("Missing authenticated Firaga LZ stream".into()); }
            }
        }
        let mut sources=Vec::new();
        for (i,(offset,bytes,role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset,bytes.len())? || rom[offset as usize..offset as usize+bytes.len()]!=bytes { return Err("Firaga transfer source overlap or reconstruction mismatch".into()); }
            let sha256=ffta_rom::digest(&bytes); if sha256!=HASHES[i] {return Err("Firaga transfer source hash mismatch".into());}
            sources.push(SourceRange {offset,length:bytes.len() as u32,sha256,role:role.into()});
        }
        Ok(Self {decoded,sources})
    }
}
