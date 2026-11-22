//! Authenticated EUR matrices copied into the Firaga battle context.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

/// Physical source extents only; guest copy lengths and control flow remain original.
pub const MATRICES: [(u32, u8, u8); 2] = [(0x3b9e80, 18, 7), (0x3b9f7c, 17, 6)];
pub const LITERALS: [(u32, u32); 4] = [
    (0x02de00, 0x08370eb4), (0x02de04, 0x083b9f7c),
    (0x02e0e0, 0x08370eb4), (0x02e0e8, 0x083b9e80),
];
const HASHES: [&str; 6] = [
    "4493998a6bbc70cc0f35f5f16b5682abbbfa9a5efe0882c605e84439cd8af14b",
    "3e04963c1166b590b80df37da91515ff8431de5d905f22dc44b92f091ce2335a",
    "4493998a6bbc70cc0f35f5f16b5682abbbfa9a5efe0882c605e84439cd8af14b",
    "f2387f52060a7760e0955cc64494b20975377d392dea18255743ae735ecebbde",
    "a804e40220edeca52571445cb3e59bebb4257b9b90ae97ab461ac77d51f00833",
    "d7ba1fd48dcec4ae337171c98c89387600ec0773264ae808c4db35cc5b23430e",
];
const HELPER: [u8; 62] = [
    0x10,0xb5,0x03,0x1c,0x00,0x2a,0x17,0xdd,0x03,0x2a,0x0e,0xd9,0x03,0x24,0x20,0x40,
    0x00,0x28,0x0a,0xd1,0x08,0x1c,0x20,0x40,0x00,0x28,0x06,0xd1,0x01,0xc9,0x01,0xc3,
    0x04,0x3a,0x03,0x2a,0xfa,0xd8,0x00,0x2a,0x06,0xdd,0x08,0x78,0x18,0x70,0x01,0x31,
    0x01,0x33,0x01,0x3a,0x00,0x2a,0xf8,0xdc,0x10,0xbc,0x01,0xbc,0x00,0x47,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    pub source_offset: u32,
    pub width: u8,
    pub height: u8,
    pub cells: Vec<u16>,
}
impl Matrix {
    fn source_bytes(&self) -> Vec<u8> {
        self.cells.iter().flat_map(|value| value.to_le_bytes()).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaCopiedMatrices {
    pub literals: [u32; 4],
    pub matrices: [Matrix; 2],
}
impl FiragaCopiedMatrices {
    pub fn validate(&self) -> Result<(), String> {
        if self.literals != LITERALS.map(|(_, value)| value) {
            return Err("Invalid Firaga copy literals".into());
        }
        for (matrix, (offset, width, height)) in self.matrices.iter().zip(MATRICES) {
            if (matrix.source_offset, matrix.width, matrix.height) != (offset, width, height)
                || matrix.cells.len() != usize::from(width) * usize::from(height) {
                return Err("Invalid Firaga copied matrix geometry".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut spans = Vec::with_capacity(6);
        for (i, (offset, _)) in LITERALS.iter().enumerate() {
            spans.push((*offset, self.literals[i].to_le_bytes().to_vec(), "firaga-copy-literal"));
        }
        for (i, matrix) in self.matrices.iter().enumerate() {
            spans.push((matrix.source_offset, matrix.source_bytes(),
                if i == 0 { "firaga-copy-matrix-0" } else { "firaga-copy-matrix-1" }));
        }
        Ok(spans)
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        for (i, (at, _)) in LITERALS.iter().enumerate() {
            if let Some(n) = offset.checked_sub(*at) {
                if let Some(byte) = self.literals[i].to_le_bytes().get(n as usize) {
                    return Some((*byte, i));
                }
            }
        }
        for (i, matrix) in self.matrices.iter().enumerate() {
            if let Some(n) = offset.checked_sub(matrix.source_offset) {
                let n = n as usize;
                if let Some(cell) = matrix.cells.get(n / 2) {
                    return Some((cell.to_le_bytes()[n % 2], 4 + i));
                }
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga copied matrices differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaCopiedMatrices,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga copied matrices require the exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at+2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at+4].try_into().unwrap());
        let checks = [
            (0x2dea4,0x20ca),(0x2dea6,0x0080),(0x2dea8,0x1837),
            (0x2dfa8,0x4d4d),(0x2dfc0,0x4949),(0x2dfc2,0x682b),
            (0x2dfc4,0x1c38),(0x2dfc6,0x22fc),(0x2dfc8,0xf117),(0x2dfca,0xfe24),
            (0x2e080,0x22ca),(0x2e082,0x0092),(0x2e084,0x18b1),(0x2e092,0x2307),
            (0x2e094,0x9300),(0x2e09e,0x2312),(0x2e0a0,0xf7d2),(0x2e0a2,0xfda0),
            (0x2ddae,0x2385),(0x2ddb0,0x00db),(0x2ddb2,0x18e7),
            (0x2dde0,0x4807),(0x2dde2,0x4908),(0x2dde4,0x6803),(0x2dde6,0x1c38),
            (0x2dde8,0x22cc),(0x2ddea,0xf117),(0x2ddec,0xff13),
            (0x2de44,0x8811),(0x2de4c,0x8010),(0x2de66,0x2106),(0x2de68,0x9100),
            (0x2de72,0x1c39),(0x2de74,0x2311),(0x2de76,0xf7d2),(0x2de78,0xfeb5),
            (0x145c14,0x4718),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || word(0x370eb4) != 0x03005ee9 || rom[0xd968a8..0xd968e6] != HELPER {
            return Err("Firaga matrix copy/queue reader fingerprint mismatch".into());
        }
        for (pc, expected) in [(0x2dde0,0x2de00),(0x2dde2,0x2de04),
            (0x2dfa8,0x2e0e0),(0x2dfc0,0x2e0e8)] {
            let opcode = half(pc);
            if opcode & 0xf800 != 0x4800 || ((pc + 4) & !3) + usize::from(opcode & 255) * 4 != expected {
                return Err("Firaga matrix literal reader mismatch".into());
            }
        }
        let matrices = std::array::from_fn(|i| {
            let (offset, width, height) = MATRICES[i];
            let start = offset as usize;
            let cells = (0..usize::from(width) * usize::from(height)).map(|j| half(start + 2*j)).collect();
            Matrix { source_offset: offset, width, height, cells }
        });
        let decoded = FiragaCopiedMatrices {
            literals: LITERALS.map(|(at, _)| word(at as usize)), matrices,
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())? || rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Firaga matrix source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] { return Err("Firaga matrix source hash mismatch".into()); }
            sources.push(SourceRange { offset, length: bytes.len() as u32, sha256, role: role.into() });
        }
        Ok(Self { decoded, sources })
    }
}
