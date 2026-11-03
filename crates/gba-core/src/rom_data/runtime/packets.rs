//! Immutable typed packet values and exact original decoder/HLE source views.
use super::*;
use crate::rom_data::{
    bg_packet_data::{self, Programs, Role, Writes},
    bg_packets, lz77,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PacketStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub typed_value_bytes: u64,
    pub lz77_calls: u64,
    pub decoded_body_bytes: u64,
    pub typed_hle_bytes: u64,
}

pub struct PacketRead {
    owner: Arc<RuntimeResources>,
    row: usize,
}
impl PacketRead {
    pub fn bytes(&self) -> &[u8] {
        &self.owner.packet_programs.as_ref().unwrap().rows[self.row].body
    }
}

struct Row {
    role: Role,
    program: usize,
    body: Vec<u8>,
    typed: Vec<bool>,
    /// Original source byte -> projected decoded-body byte, or a control byte.
    literals: Vec<u32>,
    typed_count: u64,
}
struct Binding {
    offset: usize,
    length: usize,
    row: usize,
}

pub(super) struct PacketResources {
    programs: Programs,
    rows: Vec<Row>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    typed_value_bytes: AtomicU64,
    lz77_calls: AtomicU64,
    decoded_body_bytes: AtomicU64,
    typed_hle_bytes: AtomicU64,
}

/// The input model has already been authenticated. Decode only source layout;
/// data cells come from typed writes, never the duplicated literal metadata.
fn project_body(
    program: &bg_packet_data::Program,
    role: Role,
) -> Result<(Vec<u8>, Vec<bool>), String> {
    let limits = bg_packets::Limits {
        destination_bytes: role.destination_bytes(),
        written_bytes: 64 * 1024,
        packets: 4096,
    };
    let sources = match (&program.writes, role) {
        (Writes::Paired(value), Role::Paired) => {
            value.calculated_plane_writes()?;
            bg_packets::plane_value_sources(&program.packet_body, limits)?
        }
        (Writes::Halfwords(_), Role::Halfwords | Role::Auxiliary) => {
            bg_packets::halfword_value_sources(&program.packet_body, limits)?
        }
        _ => return Err("Packet role/write kind mismatch".into()),
    };
    let mut bytes = program.packet_body.clone();
    let mut typed = vec![false; bytes.len()];
    for source in sources {
        let value = match &program.writes {
            Writes::Paired(value) => *value
                .payload
                .packets
                .get(source.packet)
                .and_then(|packet| packet.values.get(source.value))
                .ok_or("Missing typed packet pair")?,
            Writes::Halfwords(value) => *value
                .packets
                .get(source.packet)
                .and_then(|packet| packet.values.get(source.value))
                .ok_or("Missing typed packet halfword")?
                as u32,
        };
        let original = program
            .packet_body
            .get(source.offset..source.offset + source.width)
            .ok_or("Packet value source outside body")?;
        let mut word = [0; 4];
        word[..source.width].copy_from_slice(original);
        let projected = ((value & !source.marker_mask)
            | (u32::from_le_bytes(word) & source.marker_mask))
            .to_le_bytes();
        for (offset, &byte) in projected[..source.width].iter().enumerate() {
            let at = source.offset + offset;
            if typed[at] && bytes[at] != byte {
                return Err("Typed packet dictionary aliases disagree".into());
            }
            bytes[at] = byte;
            typed[at] = true;
        }
    }
    Ok((bytes, typed))
}

impl PacketResources {
    /// EUR is the current validated migration target; USA remains verified-only.
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x76c..=0x895)
    }
    pub(super) fn new(
        programs: Programs,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        programs.validate()?;
        let mut rows = Vec::new();
        let mut bindings = Vec::new();
        let mut ranges = sources.iter();
        for role in Role::ALL {
            for (index, program) in programs.family(role).programs.iter().enumerate() {
                let source = ranges.next().ok_or("Missing packet source binding")?;
                if program.encoded_source.len() != source.length as usize
                    || ffta_rom::digest(&program.encoded_source) != source.sha256
                    || source.role != format!("bg-packet-program/{}/{index}", role.name())
                {
                    return Err("Packet source/file binding mismatch".into());
                }
                let offset = source.offset as usize;
                let end = offset
                    .checked_add(source.length as usize)
                    .ok_or("Packet binding overflow")?;
                if source.length == 0
                    || end > 0x0200_0000
                    || !Self::source_page(offset)
                    || !Self::source_page(end - 1)
                {
                    return Err("Packet binding outside verified EUR asset pages".into());
                }
                let (body, typed) = project_body(program, role)?;
                if body != program.packet_body {
                    return Err("Typed packet body does not reproduce original metadata".into());
                }
                let mut literals = vec![u32::MAX; program.encoded_source.len()];
                if program.encoded_source[0] == 1 {
                    for (index, entry) in literals[4..].iter_mut().enumerate() {
                        *entry = index as u32;
                    }
                } else {
                    for (encoded, decoded) in lz77::literal_sources(
                        &program.encoded_source[4..],
                        bg_packet_data::MAX_BODY_BYTES,
                    )? {
                        literals[4 + encoded] = decoded as u32;
                    }
                }
                bindings.push(Binding {
                    offset,
                    length: source.length as usize,
                    row: rows.len(),
                });
                let typed_count = typed.iter().filter(|&&value| value).count() as u64;
                rows.push(Row {
                    role,
                    program: index,
                    body,
                    typed,
                    literals,
                    typed_count,
                });
            }
        }
        if ranges.next().is_some() {
            return Err("Unexpected packet source binding".into());
        }
        bindings.sort_by_key(|binding| binding.offset);
        if bindings
            .windows(2)
            .any(|pair| pair[0].offset + pair[0].length > pair[1].offset)
        {
            return Err("Overlapping packet source bindings".into());
        }
        let last = bindings.last().ok_or("Missing packet source bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            programs,
            rows,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            typed_value_bytes: AtomicU64::new(0),
            lz77_calls: AtomicU64::new(0),
            decoded_body_bytes: AtomicU64::new(0),
            typed_hle_bytes: AtomicU64::new(0),
        })
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn stats(&self) -> PacketStats {
        PacketStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            typed_value_bytes: self.typed_value_bytes.load(Ordering::Relaxed),
            lz77_calls: self.lz77_calls.load(Ordering::Relaxed),
            decoded_body_bytes: self.decoded_body_bytes.load(Ordering::Relaxed),
            typed_hle_bytes: self.typed_hle_bytes.load(Ordering::Relaxed),
        }
    }
    fn program(&self, row: &Row) -> &bg_packet_data::Program {
        &self.programs.family(row.role).programs[row.program]
    }
    fn byte(&self, index: usize) -> Option<(u8, bool)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        let row = &self.rows[binding.row];
        let output = row.literals[at];
        if output == u32::MAX {
            Some((self.program(row).encoded_source[at], false))
        } else {
            Some((row.body[output as usize], row.typed[output as usize]))
        }
    }
    /// Preserve old-family mapped bytes on reads straddling an asset boundary.
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= self.end || index.saturating_add(N) <= self.bindings[0].offset {
            return prior;
        }
        let mut mapped = 0;
        let mut typed = 0;
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, value)) => {
                mapped += 1;
                typed += u64::from(value);
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if mapped == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        self.typed_value_bytes.fetch_add(typed, Ordering::Relaxed);
        Some(bytes)
    }
    fn lz77_row(&self, index: usize) -> Option<usize> {
        let offset = index.checked_sub(4)?;
        let binding = &self.bindings[self
            .bindings
            .binary_search_by_key(&offset, |b| b.offset)
            .ok()?];
        (self.program(&self.rows[binding.row]).encoded_source[0] == 0x11).then_some(binding.row)
    }
    pub(super) fn resident_bytes(&self) -> usize {
        let mut bytes = std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.rows.capacity() * std::mem::size_of::<Row>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>();
        for row in &self.rows {
            bytes += row.body.capacity() + row.typed.capacity() + row.literals.capacity() * 4;
        }
        for role in Role::ALL {
            let family = self.programs.family(role);
            bytes += family.descriptor_programs.capacity() * 2
                + family.programs.capacity() * std::mem::size_of::<bg_packet_data::Program>();
            for program in &family.programs {
                bytes += program.encoded_source.capacity() + program.packet_body.capacity();
                bytes += match &program.writes {
                    Writes::Paired(value) => {
                        value.payload.packets.capacity()
                            * std::mem::size_of::<bg_packets::Patch<u32>>()
                            + value
                                .payload
                                .packets
                                .iter()
                                .map(|p| p.values.capacity() * 4)
                                .sum::<usize>()
                    }
                    Writes::Halfwords(value) => {
                        value.packets.capacity() * std::mem::size_of::<bg_packets::Patch<u16>>()
                            + value
                                .packets
                                .iter()
                                .map(|p| p.values.capacity() * 2)
                                .sum::<usize>()
                    }
                };
            }
        }
        bytes
    }
}

impl RuntimeResources {
    pub(crate) fn decoded_packet_body(self: &Arc<Self>, source: u32) -> Option<PacketRead> {
        if !matches!(source >> 24, 0x08..=0x0d) {
            return None;
        }
        let packets = self.packet_programs.as_ref()?;
        let row = packets.lz77_row((source & 0x01ff_ffff) as usize)?;
        packets.lz77_calls.fetch_add(1, Ordering::Relaxed);
        packets
            .decoded_body_bytes
            .fetch_add(packets.rows[row].body.len() as u64, Ordering::Relaxed);
        packets
            .typed_hle_bytes
            .fetch_add(packets.rows[row].typed_count, Ordering::Relaxed);
        Some(PacketRead {
            owner: self.clone(),
            row,
        })
    }
}

#[cfg(test)]
#[path = "packet_tests.rs"]
mod tests;
