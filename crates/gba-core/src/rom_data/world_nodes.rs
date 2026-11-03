//! Conditional EUR world-node projection; live placement and pathfinding stay guest code.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x395ca8;
/// The original coordinate loop's domain, not a cap on other guest lookups.
pub const NODE_COUNT: usize = 30;
pub const RECORD_BYTES: usize = 20;
pub const SOURCE_BYTES: usize = NODE_COUNT * RECORD_BYTES;
pub const SOURCE_SHA256: &str = "ed9d29c7b96b37537f0b8ac8e95fc17c57605c1056427e8dcbc58775e12d2e76";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// Zero disables the link in adjacency/pathfinding, but not in every reader.
    pub path_id: u8,
    /// Zero is a valid neighbor. Keep empty slots and their original order.
    pub neighbor: u8,
    pub opaque: [u8; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub index: u8,
    pub x_tile: u8,
    pub y_tile: u8,
    pub links: [Link; 4],
    pub opaque_tail: [u8; 2],
}
impl Node {
    fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut bytes = [0; RECORD_BYTES];
        bytes[0] = self.x_tile;
        bytes[1] = self.y_tile;
        for (i, link) in self.links.iter().enumerate() {
            let at = 2 + i * 4;
            bytes[at] = link.path_id;
            bytes[at + 1] = link.neighbor;
            bytes[at + 2..at + 4].copy_from_slice(&link.opaque);
        }
        bytes[18..20].copy_from_slice(&self.opaque_tail);
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldNodes {
    pub source_offset: u32,
    pub nodes: Vec<Node>,
}
impl WorldNodes {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.nodes.len() != NODE_COUNT
            || self
                .nodes
                .iter()
                .enumerate()
                .any(|(i, n)| usize::from(n.index) != i)
        {
            return Err("Invalid conditional EUR world-node projection".into());
        }
        // No inferred max path ID, neighbor cap or zero requirement for opaque bytes.
        Ok(())
    }
    /// Original 36E90/36EB4 scale tile coordinates by eight, without centering.
    pub fn pixel_position(&self, index: u8) -> Option<[u16; 2]> {
        let node = self.nodes.get(usize::from(index))?;
        Some([u16::from(node.x_tile) * 8, u16::from(node.y_tile) * 8])
    }
    /// Original 4DAD0 matches neighbors before inspecting path IDs. An empty
    /// slot can return zero. None means the node itself is outside this owner.
    pub fn path_to(&self, index: u8, neighbor: u8) -> Option<u8> {
        let node = self.nodes.get(usize::from(index))?;
        Some(
            node.links
                .iter()
                .find(|l| l.neighbor == neighbor)
                .map_or(255, |l| l.path_id),
        )
    }
    /// Original 46EB0 accepts equal nodes, otherwise tests nonzero path first.
    /// This typed helper covers declared nodes only; it does not cap the guest.
    pub fn adjacent(&self, index: u8, neighbor: u8) -> Option<bool> {
        let node = self.nodes.get(usize::from(index))?;
        Some(
            index == neighbor
                || node
                    .links
                    .iter()
                    .any(|l| l.path_id != 0 && l.neighbor == neighbor),
        )
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let byte = offset.checked_sub(self.source_offset)? as usize;
        Some(self.nodes.get(byte / RECORD_BYTES)?.source_bytes()[byte % RECORD_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            self.source_offset,
            self.nodes.iter().flat_map(Node::source_bytes).collect(),
            "world-node-records".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("World nodes differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: WorldNodes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("World nodes require exact EUR ROM; USA pending".into());
        }
        // Root stores, coordinate getters/loop and distinct path/adjacency readers.
        for (start, end, sha) in [
            (
                0x31488,
                0x31564,
                "5109bfd44f224aeea656cff23be099c3a8e79b51c3b322e4d9c335088c1b4aaa",
            ),
            (
                0x32bd6,
                0x32c7c,
                "b53a11034194c47630345ab85c8e50485b81a7e498b5e536174b86c7dfd7b4e7",
            ),
            (
                0x36e90,
                0x36ed8,
                "fd3602a85c95b31d4a618a0dc644f8b4380cdb393c3491dcbaed0c812b3e0bd7",
            ),
            (
                0x375f8,
                0x3770a,
                "4b7eeb75f33248ed21b6b9c230df58cbeba2cd63a17d934520bbb0ce8b69965d",
            ),
            (
                0x4dad0,
                0x4db1a,
                "ef9a0aef405e6340d90717791d9771a60a1b98b6c6ac3693bb2c14baa8e8fcfe",
            ),
            (
                0x46eb0,
                0x46f40,
                "fd978c5295c626b7c7cd46ca49852f45068a225b9b86ac96471a39086547e729",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != sha {
                return Err("Original world-node producer/reader fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("World-node source fingerprint mismatch".into());
        }
        let decoded = WorldNodes {
            source_offset: SOURCE,
            nodes: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, b)| Node {
                    index: i as u8,
                    x_tile: b[0],
                    y_tile: b[1],
                    links: std::array::from_fn(|slot| {
                        let at = 2 + slot * 4;
                        Link {
                            path_id: b[at],
                            neighbor: b[at + 1],
                            opaque: [b[at + 2], b[at + 3]],
                        }
                    }),
                    opaque_tail: [b[18], b[19]],
                })
                .collect(),
        };
        let sources = decoded
            .source_spans()?
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

#[cfg(test)]
mod tests;
