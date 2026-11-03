//! Catalog-wide original loader oracles; private ROM bodies stay outside Git.
use super::*;
use gba_core::{
    bus::Bus,
    rom_data::{bg_descriptors, bg_packets, lz77},
    Machine,
};
use std::collections::BTreeMap;

const RETURN: u32 = 0x0800_0300;
const STACK: u32 = 0x0300_7c00;
const BLANK: u16 = 0x6421;

enum Writes {
    Pairs(bg_packets::PlanePatches),
    Halfwords(bg_packets::Patches<u16>),
}
struct Program {
    source: usize,
    parent: Option<usize>,
    tag: u8,
    encoded_bytes: usize,
    body: Vec<u8>,
    writes: Writes,
}
impl Program {
    fn apply(&self, target: &mut [u8]) {
        match &self.writes {
            Writes::Pairs(value) => {
                for packet in &value.calculated_plane_writes().unwrap().packets {
                    for (index, word) in packet.values.iter().enumerate() {
                        let at = packet.byte_offset as usize + index * 4;
                        target[at..at + 4].copy_from_slice(&word.to_le_bytes());
                    }
                }
            }
            Writes::Halfwords(value) => {
                for packet in &value.packets {
                    for (index, word) in packet.values.iter().enumerate() {
                        let at = packet.byte_offset as usize + index * 2;
                        target[at..at + 2].copy_from_slice(&word.to_le_bytes());
                    }
                }
            }
        }
    }
}

fn parse_program(rom: &[u8], source: usize, role: usize, size: usize) -> Program {
    let header = u32::from_le_bytes(rom[source..source + 4].try_into().unwrap());
    let tag = header as u8;
    assert!(matches!(tag, 1 | 0x11), "verified wrapper tag");
    let parent = (header >> 8 != 0x00ff_ffff).then_some((header >> 8) as usize);
    assert!(parent.is_none_or(|id| id < bg_descriptors::BASE_RECORDS));
    let input = &rom[source + 4..(source + 4 + 0x10000).min(rom.len())];
    let (mut body, consumed) = if tag == 0x11 {
        let decoded = lz77::decode(input, 0x4000).unwrap();
        (decoded.bytes, Some(decoded.consumed))
    } else {
        (input.to_vec(), None)
    };
    let limits = bg_packets::Limits {
        destination_bytes: size,
        written_bytes: 0x10000,
        packets: 4096,
    };
    let (writes, extent) = if role == 0 {
        let value = bg_packets::decode_planes(&body, limits).unwrap();
        value.calculated_plane_writes().unwrap();
        let extent = value.referenced_extent;
        (Writes::Pairs(value), extent)
    } else {
        let value = bg_packets::decode_halfwords(&body, None, limits).unwrap();
        let extent = value.consumed;
        (Writes::Halfwords(value), extent)
    };
    if consumed.is_none() {
        body.truncate(extent);
    }
    Program {
        source,
        parent,
        tag,
        encoded_bytes: 4 + consumed.unwrap_or(extent),
        body,
        writes,
    }
}

fn invoke(machine: &mut Machine, entry: u32) -> usize {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    for steps in 1..=2_000_000 {
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return steps;
        }
        machine.bus.frame_ready = false;
    }
    panic!("original catalog loader did not return at {entry:08x}");
}

#[test]
#[ignore = "requires exact private ROM and absent FFTA_STATE_TEST_PACKET_CATALOG_ROOT; no boot replay"]
fn complete_packet_catalog_matches_original_loaders() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PACKET_CATALOG_ROOT").unwrap());
    assert!(!root.exists(), "preserve catalog oracle evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = bg_descriptors::Catalog::from_rom(&rom).unwrap();
    let entries = if gba_core::ffta_rom::digest(&rom) == gba_core::ffta_rom::SHA256 {
        [0x0801_f1a8, 0x0801_f2ec, 0x0801_f438, 0x0801_ed24]
    } else {
        [0x0802_0318, 0x0802_045c, 0x0802_05a8, 0x0801_fe94]
    };
    let sources: Vec<_> = catalog
        .decoded
        .entries
        .iter()
        .map(|d| {
            [
                d.paired_planes_source,
                d.halfword_planes_source,
                d.auxiliary_patches_source,
            ]
        })
        .collect();
    fs::create_dir_all(&root).unwrap();
    let mut machine = crate::make_machine(rom.clone(), None);
    let mut reports = Vec::new();
    let mut invocations = 0;
    let mut instructions = 0;
    let mut continuations = 0;
    for (role, name, base, size, expected_sources) in [
        (0, "paired", 0x91a0, 0x4000, 122),
        (1, "halfwords", 0xd1a0, 0x2000, 117),
        (2, "auxiliary", 0x7cb0, 0x200, 113),
    ] {
        let offsets: Vec<_> = sources
            .iter()
            .map(|v| catalog.source.offset as usize + v[role] as usize)
            .collect();
        let programs: BTreeMap<_, _> = offsets
            .iter()
            .map(|&offset| (offset, parse_program(&rom, offset, role, size)))
            .collect();
        assert_eq!(programs.len(), expected_sources);
        let mut maximum_chain = 0;
        for (id, &source) in offsets.iter().enumerate() {
            let mut chain = Vec::new();
            let mut current = Some(id);
            while let Some(index) = current {
                assert!(!chain.contains(&index), "cyclic descriptor inheritance");
                chain.push(index);
                current = programs[&offsets[index]].parent;
            }
            maximum_chain = maximum_chain.max(chain.len());
            let mut expected = vec![0; size];
            if role == 0 {
                for word in expected.chunks_exact_mut(2) {
                    word.copy_from_slice(&BLANK.to_le_bytes());
                }
            }
            for &index in chain.iter().rev() {
                programs[&offsets[index]].apply(&mut expected);
            }
            machine.bus.ewram.fill(0);
            machine.bus.ewram[base - 16..base + size + 16].fill(0x5a);
            machine.bus.write16(0x0200_7f5e, BLANK);
            machine.cpu.regs[0] = 0x0800_0000 + source as u32;
            instructions += invoke(&mut machine, entries[role]);
            invocations += 1;
            if role == 0 {
                for _ in 0..32 {
                    let cursor = machine.bus.read32(0x0200_80a8);
                    if cursor == 0 {
                        break;
                    }
                    machine.cpu.regs[0] = cursor;
                    machine.cpu.regs[1] = machine.bus.read32(0x0200_80b0);
                    machine.cpu.regs[2] = machine.bus.read8(0x0200_80c0) as u32;
                    machine.cpu.regs[3] = 0;
                    for (offset, value) in [(0, 0), (4, 0), (8, 0), (12, 2048)] {
                        machine.bus.write32(STACK + offset, value);
                    }
                    instructions += invoke(&mut machine, entries[3]);
                    machine.bus.write32(0x0200_80a8, machine.cpu.regs[0]);
                    continuations += 1;
                }
                assert_eq!(machine.bus.read32(0x0200_80a8), 0, "all chunks complete");
            }
            assert!(
                machine.bus.ewram[base..base + size] == expected,
                "{name} id={id} complete destination"
            );
            assert_eq!(
                &machine.bus.ewram[base - 16..base],
                &[0x5a; 16],
                "{name} id={id} prefix guard"
            );
            assert_eq!(
                &machine.bus.ewram[base + size..base + size + 16],
                &[0x5a; 16],
                "{name} id={id} suffix guard"
            );
        }
        assert_eq!(maximum_chain, 2);
        let mut values = Vec::new();
        for value in programs.values() {
            let body_name = format!("{name}-{:08x}.bin", value.source);
            fs::write(root.join(&body_name), &value.body).unwrap();
            values.push(serde_json::json!({
                "source":value.source,"parent":value.parent,"tag":value.tag,
                "encoded_bytes":value.encoded_bytes,
                "encoded_sha256":gba_core::ffta_rom::digest(&rom[value.source..value.source+value.encoded_bytes]),
                "body_bytes":value.body.len(),"body_sha256":gba_core::ffta_rom::digest(&value.body),
                "body_file":body_name,
            }));
        }
        reports.push(serde_json::json!({"role":name,"records":offsets,"sources":values,"maximum_chain_nodes":maximum_chain}));
        eprintln!("BG_PACKET_CATALOG role={name} records=162 sources={expected_sources} maximum_chain_nodes={maximum_chain} complete_destinations=true guards=true");
    }
    let report = serde_json::json!({"families":reports,"wrapper_invocations":invocations,
        "paired_continuations":continuations,"original_instructions":instructions,
        "complete_destinations":true,"guards":true,"stack_restored":true,"no_boot_replay":true});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_PACKET_CATALOG complete wrapper_invocations={invocations} paired_continuations={continuations} original_instructions={instructions}");
}
