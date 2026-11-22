//! One selected EUR Firaga queued halfword-transfer view.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSETS: [u32; 3] = [0x0ac9a0, 0x0ac9a8, 0x40e844];
pub const SOURCE_LENGTHS: [usize; 3] = [4, 4, 256];
const HASHES: [&str; 3] = [
    "02bcb9ab0dbc79eb74c9623f610bb0dc60154b75ac1f9e4fb2993457430106c4",
    "cae56e1190dba440800b431fec1d337f3486ebb51145d194b8d28a2190821727",
    "d94dd96e037b61ea2cd68d3b4c0181af03ae6d804874d901c7d8f746126762ed",
];
const ROLES: [&str; 3] = [
    "firaga-queue-source-base",
    "firaga-queue-destination",
    "firaga-queue-halfwords",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaQueuedTransfer {
    pub source_base: u32,
    pub destination: u32,
    pub selected_index: u8,
    pub halfwords: Vec<u16>,
}
impl FiragaQueuedTransfer {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.source_base,
            self.destination,
            self.selected_index,
            self.halfwords.len(),
        ) != (0x0840e344, 0x06013700, 5, 128)
            || self.source_base + (u32::from(self.selected_index) << 8) != 0x0840e844
        {
            return Err("Invalid Firaga queued halfword-transfer shape".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let payloads = [
            self.source_base.to_le_bytes().to_vec(),
            self.destination.to_le_bytes().to_vec(),
            self.halfwords
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
        ];
        Ok(SOURCE_OFFSETS
            .into_iter()
            .zip(payloads)
            .zip(ROLES)
            .map(|((at, bytes), role)| (at, bytes, role))
            .collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga queued transfer differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaQueuedTransfer,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga queued transfer requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xac8ee, 0x1c28),
            (0xac8f0, 0x210a),
            (0xac8f2, 0xf099),
            (0xac8f4, 0xfbc3),
            (0xac8f8, 0x0090),
            (0xac8fa, 0x1880),
            (0xac8fc, 0x0040),
            (0xac8fe, 0x1a2e),
            (0xac94e, 0x0231),
            (0xac950, 0x4a13),
            (0xac952, 0x1889),
            (0xac954, 0x4a14),
            (0xac956, 0x2380),
            (0xac958, 0xf754),
            (0xac95a, 0xf8ba),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value) {
            return Err("Firaga queued transfer original producer mismatch".into());
        }
        for (pc, at) in [(0xac950, 0xac9a0), (0xac954, 0xac9a8)] {
            if ((pc + 4) & !3) + usize::from(half(pc) & 255) * 4 != at {
                return Err("Firaga queued transfer literal reader mismatch".into());
            }
        }
        let decoded = FiragaQueuedTransfer {
            source_base: word(0xac9a0),
            destination: word(0xac9a8),
            selected_index: 5,
            halfwords: (0..128).map(|i| half(0x40e844 + 2 * i)).collect(),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga queued source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga queued source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
