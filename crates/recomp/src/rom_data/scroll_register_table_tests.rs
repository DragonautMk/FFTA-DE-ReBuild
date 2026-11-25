//! Verify both E524 scroll tables and the shadow alias through the original readers.
use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Extraction,
    Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const STACK: u32 = 0x0300_7100;
const IO_REGISTERS: u32 = 0x0400_0010;
const SHADOW_FIELDS: u32 = 0x0300_094a;
const NEIGHBORS: [usize; 4] = [0x14d4cb, 0x14d50c, 0x14d6df, 0x14d700];

#[derive(Deserialize)]
struct Plan {
    spans: Vec<ReferenceSpan>,
}

#[derive(Deserialize)]
struct ReferenceSpan {
    offset: u32,
    length: u32,
    sha256: String,
    role: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_thumb(machine: &mut Machine, entry: u32, stop: u32) {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[15] = ROM_BASE + entry;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    for _ in 0..4096 {
        if machine.cpu.regs[15] == ROM_BASE + stop {
            return;
        }
        machine.step();
    }
    panic!(
        "Reader {entry:06x} did not reach {stop:06x}. PC {:08x}",
        machine.cpu.regs[15]
    );
}

fn run_scroll_readers(machine: &mut Machine, rom: &[u8]) {
    let state = 0x0201_8000;
    let message = 0x0201_9000;
    let first_shadow_value = 0x81a0u16;
    for index in 0..8 {
        machine
            .bus
            .write16(SHADOW_FIELDS + index * 2, first_shadow_value + index as u16);
    }

    let cases = [
        (0x0dc9b0, 8, 0, state, 9),
        (0x0dcd1c, 8, 7, state, 8),
        (0x0dcfc0, 0, 2, state, 8),
        (0x0dd066, 0, 5, message, 9),
    ];
    for (entry, stack_offset, slot, r6, state_field) in cases {
        let background = slot / 2;
        let axis = slot % 2;
        machine.cpu.regs[4] = message;
        machine.cpu.regs[5] = state;
        machine.cpu.regs[6] = r6;
        machine.bus.write8(state + 8, 0);
        machine.bus.write8(state + state_field, background as u8);
        machine.bus.write8(state + state_field + 1, axis as u8);
        machine.bus.write16(message + 0x14, background as u16);
        machine.bus.write16(message + 0x16, axis as u16);
        machine.bus.write16(message + 0x18, 0);

        // The original constructor saves its MMIO argument in r8 at 0800cf00.
        run_thumb(machine, entry, 0x00cf02);
        let expected_shadow = i32::from((first_shadow_value + slot as u16) as i16) as u32;
        assert_eq!(
            machine.cpu.regs[1], expected_shadow,
            "shadow read at {entry:06x}"
        );
        assert_eq!(
            machine.cpu.regs[8],
            IO_REGISTERS + 2 * slot,
            "MMIO argument at {entry:06x}"
        );
        let at = (STACK - 0x0300_0000) as usize + stack_offset;
        assert_eq!(
            &machine.bus.iwram[at..at + 64],
            &rom[0x14d4cc..0x14d50c],
            "stack copies at {entry:06x}"
        );
    }

    run_thumb(machine, 0x0f1054, 0x0f1068);
    let at = (STACK - 0x0300_0000) as usize + 4;
    assert_eq!(&machine.bus.iwram[at..at + 32], &rom[0x14d6e0..0x14d700]);
    assert_eq!(machine.cpu.regs[0], u32::from(first_shadow_value + 4));
}

fn sample_altered(machine: &mut Machine, offsets: &[usize]) -> Vec<u8> {
    offsets
        .iter()
        .map(|&offset| {
            machine.bus.rom[offset] ^= 0xff;
            let byte = machine.bus.read8(ROM_BASE + offset as u32);
            machine.bus.rom[offset] ^= 0xff;
            byte
        })
        .collect()
}

#[test]
#[ignore = "exact EUR ROM, E524 source plan and e322-d321 predecessor manifest"]
fn european_scroll_register_table_publication_and_original_readers() {
    let work = input("FFTA_SCROLL_TABLE_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_SCROLL_TABLE_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_SCROLL_TABLE_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E523 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (322, 321)
    );
    let added: Vec<_> = plan
        .spans
        .into_iter()
        .map(|span| SourceRange {
            offset: span.offset,
            length: span.length,
            sha256: span.sha256,
            role: span.role,
        })
        .collect();
    assert_eq!(
        (added.len(), added.iter().map(|s| s.length).sum::<u32>()),
        (8, 132)
    );
    for span in &added {
        let at = span.offset as usize;
        assert_eq!(digest(&rom[at..at + span.length as usize]), span.sha256);
        assert!(prior
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .all(|other| {
                other.offset + other.length <= span.offset
                    || span.offset + span.length <= other.offset
            }));
    }

    let resources = tempfile::tempdir_in(&work).unwrap();
    let rom_arg = rom_path.to_str().unwrap().to_owned();
    let prepare = [
        "prepare".into(),
        rom_arg.clone(),
        resources.path().to_str().unwrap().into(),
    ];
    cmd(&prepare).unwrap();
    let revision = resources
        .path()
        .join(&prior.source_rom_sha256)
        .join("v1/e323-d322");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    cmd(&prepare).unwrap();
    assert_eq!(
        fs::read_to_string(revision.join("current")).unwrap(),
        current
    );
    cmd(&["verify".into(), rom_arg, dataset.to_str().unwrap().into()]).unwrap();

    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E525 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (323, 322)
    );
    let index = manifest
        .resources
        .iter()
        .position(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let old = &prior.resources[index];
    let new = &manifest.resources[index];
    let mut expected_sources = old.sources.clone();
    expected_sources.extend(added.iter().cloned());
    expected_sources.sort_by_key(|span| span.offset);
    assert_eq!(new.sources, expected_sources);
    assert_eq!(
        (&new.id, &new.path, &new.format, &new.dependencies),
        (&old.id, &old.path, &old.format, &old.dependencies)
    );
    let mut missing = manifest.clone();
    missing.resources[index] = old.clone();
    let mut previous_layout = missing.clone();
    previous_layout.extractor_revision = prior.extractor_revision;
    previous_layout.decoder_revision = prior.decoder_revision;
    assert_eq!(previous_layout, prior);

    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    assert!(packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .scroll_register_tables
        .take()
        .is_some());
    let previous = Extraction::from_rom_for_manifest(&rom, &prior)
        .unwrap()
        .ui_raw_transfers
        .unwrap()
        .decoded;
    assert_eq!(packet, previous);
    let previous = encode_ui_raw_transfers(&previous).unwrap();
    assert_eq!(digest(&previous), old.sha256);

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    fs::write(&packet_path, &previous).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let boundaries = sample_altered(&mut managed, &NEIGHBORS);
    assert_eq!(
        boundaries,
        [
            !rom[0x14d4cb],
            !rom[0x14d50c],
            rom[0x14d6df],
            !rom[0x14d700]
        ]
    );
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(&manifest_path, toml::to_string_pretty(&missing).unwrap()).unwrap();
    fs::write(&packet_path, previous).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();
    assert_eq!(sample_altered(&mut managed, &NEIGHBORS), boundaries);

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let alter_backing = |machine: &mut Machine| {
        for span in &added {
            let at = span.offset as usize;
            for byte in &mut machine.bus.rom[at..at + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        machine.bus.ffta_sprites.enabled = false;
    }
    // Loading a state authenticates the original ROM before the test alters it.
    alter_backing(&mut managed);
    let before = owner.ui_raw_transfers_stats().unwrap();
    for machine in [&mut raw, &mut managed] {
        run_scroll_readers(machine, &rom);
    }
    alter_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    let after = owner.ui_raw_transfers_stats().unwrap();
    let payload = after.payload_bytes - before.payload_bytes;
    let selectors = after.selector_bytes - before.selector_bytes;
    assert_eq!((payload, selectors), (288, 36));
    assert_eq!(
        after.source_bytes - before.source_bytes,
        payload + selectors
    );

    let result = serde_json::json!({
        "status": "PASS", "revision": [323, 322], "spans": added.len(), "new_bytes": 132,
        "paired_readers": 4, "alias_readers": 1, "copies": 9,
        "payload_bytes_read": payload, "selector_bytes_read": selectors,
        "full_state_equal": true, "predecessor_attach": true, "neighbors_preserved": true,
        "missing_group_rejected": true, "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
