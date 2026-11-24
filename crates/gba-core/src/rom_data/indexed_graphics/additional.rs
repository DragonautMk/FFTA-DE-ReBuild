//! Three source-rooted EUR indexed families; original selectors and decoders remain active.
use super::{Archive, PointerGroup, SourceSpan};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const GROUPS: [u32; 3] = [0x39554c, 0x3955d8, 0x397d94];
pub const POINTERS: [[u32; 5]; 3] = [
    [0x43b864, 0x44cea0, 0x45e9d0, 0x46f9ec, 0x481504],
    [0x439490, 0x44a690, 0x45c280, 0x46d51c, 0x47ebc4],
    [0x43d990, 0x44efb8, 0x460b20, 0x471b58, 0x483634],
];
pub const EMBEDDED_PREFIX_LENGTH: usize = 0xb4;
pub const CALLER_LITERALS: [(u32, u32); 6] = [
    (0x037258, 0x0839554c), (0x059d98, 0x0839554c),
    (0x037c24, 0x083955d8), (0x037e98, 0x083955d8),
    (0x0cf450, 0x08397d94), (0x0cf490, 0x08397d94),
];
pub const PROGRAMS: usize = 1588;
const SOURCE_SHA256: &str = "8700f34be8fd4f9bd20757921c581c05b7dbb0ad87841a5ce34bde6184401c30";

#[derive(Clone, Copy)]
pub struct ArchiveSpec {
    pub offset: u32,
    pub length: usize,
    pub programs: usize,
    pub decoded_bytes: u16,
    pub sha256: &'static str,
}

pub const ARCHIVES: [ArchiveSpec; 15] = [
    ArchiveSpec { offset: 0x43b864, length: 963, programs: 30, decoded_bytes: 64, sha256: "a0e3a4a2ea3321ec6ff61ed535cab0286cccdf2e5040bbffdb42166f407350e4" },
    ArchiveSpec { offset: 0x44cea0, length: 905, programs: 30, decoded_bytes: 64, sha256: "9a79956918fb5e67ffe33e18ed7ce83d2eea53f9003cf812af89066eef52c44b" },
    ArchiveSpec { offset: 0x45e9d0, length: 869, programs: 30, decoded_bytes: 64, sha256: "30367f740fbbd90f6d782a72abe968e62aefca81bd2c9cc1636644fb5c031e18" },
    ArchiveSpec { offset: 0x46f9ec, length: 985, programs: 30, decoded_bytes: 64, sha256: "b4a50e2722b9d75b5b4313fd8bed3ef0f36099484ce58b4074cc4486d350a74e" },
    ArchiveSpec { offset: 0x481504, length: 945, programs: 30, decoded_bytes: 64, sha256: "b8e0be53cb212c29a5e4a89619e2445f5bcd0116afe1bd8ba6bf222777769aa0" },
    ArchiveSpec { offset: 0x439490, length: 6459, programs: 225, decoded_bytes: 64, sha256: "45b35f3d8a1527fdd7162f4d2c50faeac9031855cc838b912e2ca7b92c565b85" },
    ArchiveSpec { offset: 0x44a690, length: 7487, programs: 249, decoded_bytes: 64, sha256: "fb206b5b9d9937b74a193138bc822555333e654561d54a1eb6deee176780907e" },
    ArchiveSpec { offset: 0x45c280, length: 7279, programs: 231, decoded_bytes: 64, sha256: "f480241b93c99a413f8140273afd2e7fd1fb65aa25f7b7bdf7020ef66d9fa82d" },
    ArchiveSpec { offset: 0x46d51c, length: 6645, programs: 235, decoded_bytes: 64, sha256: "b6605878490c3001cd1baf25d11b3d23ee05812655d590089db6b2b4ca5c94dd" },
    ArchiveSpec { offset: 0x47ebc4, length: 7816, programs: 278, decoded_bytes: 64, sha256: "c8e93f543d23dfa74e45ba930b0979f5214e258bf552103715325157032ffd1b" },
    ArchiveSpec { offset: 0x43da44, length: 10174, programs: 44, decoded_bytes: 256, sha256: "24fed08e19711fbcb8278300c045ed2fbe581e641795d597fa7ec7e567ee00f3" },
    ArchiveSpec { offset: 0x44f06c, length: 10194, programs: 44, decoded_bytes: 256, sha256: "0cf5f47c05f82849659600bac5da72cb52f7b6e8ff3375b93a6b431ac9ecb087" },
    ArchiveSpec { offset: 0x460bd4, length: 10255, programs: 44, decoded_bytes: 256, sha256: "860e64ac2a7084e831feeff5ade85d5751b8a1724a0c86e40a5d647cd99e04ec" },
    ArchiveSpec { offset: 0x471c0c, length: 10179, programs: 44, decoded_bytes: 256, sha256: "56869781f1bc72a2b339f013b3749c02d20274fc3d25c0981284517ae33332fe" },
    ArchiveSpec { offset: 0x4836e8, length: 10165, programs: 44, decoded_bytes: 256, sha256: "5986787d9c0f82d1dd6bdac7a51b06dc5b48621015ac184767397c70e855d00b" },
];

const READER_WINDOWS: &[(usize, usize, &str)] = &[
    (0x37188, 0x38, "f316b6158a9691409d199e6715eee729307c8830504a531a779f295875da8a05"),
    (0x59d42, 0x14, "32b1a717443830e8947978aa5ebe2282e6416ac9ed6a882de440dc5522ec54b5"),
    (0x37bb6, 0x24, "3ff99d67ff949bddbeabbdeedffa7298cc3e4e7b06cef16044fac369ab2b1779"),
    (0x37e1a, 0x2c, "8d81152f2a6a955419138ea7426ed933b68487ca0c1e58609df748aa831d7549"),
    (0xcf42a, 0x20, "62dc56a73dc669f507642695ceba9e1f7254eef1cb263520557b02365bfc197d"),
    (0xcf470, 0x1e, "378ab915b8f6cf733d8701b3db711f564c55d8892b5cc1bde1f13523e3b5a37b"),
];

fn invalid() -> String { "Invalid additional indexed graphics families".into() }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedPrefix {
    pub source_offset: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalIndexed {
    pub groups: Vec<PointerGroup>,
    pub caller_literals: Vec<u32>,
    pub embedded_prefixes: Vec<EmbeddedPrefix>,
    pub archives: Vec<Archive>,
}

impl AdditionalIndexed {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.groups.len() != GROUPS.len() || self.caller_literals.len() != CALLER_LITERALS.len()
            || self.embedded_prefixes.len() != 5 || self.archives.len() != ARCHIVES.len() {
            return Err(invalid());
        }
        let mut spans = Vec::new();
        for (i, group) in self.groups.iter().enumerate() {
            if group.source_offset != GROUPS[i] || group.pointers != POINTERS[i] { return Err(invalid()); }
            spans.push((group.source_offset, group.source_bytes(), format!("additional-indexed-selector/{:06x}", group.source_offset)));
        }
        for (value, (offset, expected)) in self.caller_literals.iter().zip(CALLER_LITERALS) {
            if *value != expected { return Err(invalid()); }
            spans.push((offset, value.to_le_bytes().to_vec(), format!("additional-indexed-literal/{offset:06x}")));
        }
        for (prefix, expected) in self.embedded_prefixes.iter().zip(POINTERS[2]) {
            if prefix.source_offset != expected || prefix.bytes.len() != EMBEDDED_PREFIX_LENGTH { return Err(invalid()); }
            spans.push((prefix.source_offset, prefix.bytes.clone(), format!("additional-indexed-prefix/{expected:06x}")));
        }
        for (archive, spec) in self.archives.iter().zip(ARCHIVES) {
            if archive.source_offset != spec.offset || archive.programs.len() != spec.programs
                || archive.fixed_decoded_bytes != spec.decoded_bytes { return Err(invalid()); }
            let bytes = archive.source_bytes()?;
            if bytes.len() != spec.length { return Err(invalid()); }
            spans.push((spec.offset, bytes, format!("additional-indexed-archive/{:06x}", spec.offset)));
        }
        if self.archives.iter().map(|a| a.programs.len()).sum::<usize>() != PROGRAMS { return Err(invalid()); }
        spans.sort_by_key(|span| span.0);
        if spans.windows(2).any(|pair| pair[0].0 as usize + pair[0].1.len() > pair[1].0 as usize) { return Err(invalid()); }
        Ok(spans)
    }
}

pub fn from_rom(rom: &[u8]) -> Result<AdditionalIndexed, String> {
    if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
        return Err("Additional indexed families require exact EUR ROM; USA pending".into());
    }
    for &(at, length, hash) in READER_WINDOWS {
        if ffta_rom::digest(&rom[at..at + length]) != hash { return Err("Additional indexed reader fingerprint mismatch".into()); }
    }
    let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
    let groups = GROUPS.into_iter().map(|source_offset| {
        let mut pointers = [0; 5];
        for (i, pointer) in pointers.iter_mut().enumerate() {
            *pointer = word(source_offset as usize + i * 4).checked_sub(0x0800_0000).ok_or_else(invalid)?;
        }
        Ok(PointerGroup { source_offset, pointers })
    }).collect::<Result<Vec<_>, String>>()?;
    let caller_literals = CALLER_LITERALS.iter().map(|&(at, _)| word(at as usize)).collect();
    let embedded_prefixes = POINTERS[2].into_iter().map(|source_offset| EmbeddedPrefix {
        source_offset,
        bytes: rom[source_offset as usize..source_offset as usize + EMBEDDED_PREFIX_LENGTH].to_vec(),
    }).collect();
    let archives = ARCHIVES.into_iter().map(|spec| {
        let start = spec.offset as usize;
        let bytes = &rom[start..start + spec.length];
        if ffta_rom::digest(bytes) != spec.sha256 { return Err("Additional indexed archive fingerprint mismatch".into()); }
        let archive = Archive::from_source(bytes, spec.offset)?;
        if archive.source_bytes()? != bytes { return Err(invalid()); }
        Ok(archive)
    }).collect::<Result<Vec<_>, String>>()?;
    let data = AdditionalIndexed { groups, caller_literals, embedded_prefixes, archives };
    let bytes: Vec<_> = data.source_spans()?.into_iter().flat_map(|span| span.1).collect();
    if ffta_rom::digest(&bytes) != SOURCE_SHA256 { return Err("Additional indexed source fingerprint mismatch".into()); }
    Ok(data)
}

pub fn source_page(index: usize) -> bool {
    let page = index >> 12;
    let contains = |start: u32, length: usize| {
        page >= (start as usize >> 12) && page <= ((start as usize + length - 1) >> 12)
    };
    GROUPS.into_iter().any(|at| contains(at, 20))
        || CALLER_LITERALS.into_iter().any(|(at, _)| contains(at, 4))
        || POINTERS[2].into_iter().any(|at| contains(at, EMBEDDED_PREFIX_LENGTH))
        || ARCHIVES.into_iter().any(|spec| contains(spec.offset, spec.length))
}
