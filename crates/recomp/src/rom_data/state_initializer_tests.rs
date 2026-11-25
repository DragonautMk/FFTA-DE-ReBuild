//! Verify the six E522 sources through the original European readers.
use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Extraction,
    Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const EWRAM_BASE: u32 = 0x0200_0000;
const BLEND_AND_GAP: [usize; 4] = [0x39db84, 0x39db85, 0x39db86, 0x39db87];
const OTHER_TEMPLATES: [usize; 3] = [0x3954d4, 0x39550c, 0x395528];

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
    machine.cpu.regs[13] = 0x0300_7c00;
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

fn check_ewram(machine: &Machine, address: u32, expected: &[u8]) {
    let offset = (address - EWRAM_BASE) as usize;
    assert_eq!(
        &machine.bus.ewram[offset..offset + expected.len()],
        expected,
        "copy to {address:08x}"
    );
}

fn run_direct_readers(machine: &mut Machine, rom: &[u8]) {
    let defaults = 0x0201_7000;
    machine.cpu.regs[0] = defaults;
    machine.cpu.regs[1] = defaults + 0x300;
    run_thumb(machine, 0x03eb5c, 0x03eb70);
    check_ewram(machine, defaults, &rom[0x3954f0..0x39550c]);

    let rectangles = 0x0201_7800;
    machine.cpu.regs[0] = 0;
    machine.cpu.regs[1] = rectangles - 4;
    machine.cpu.regs[2] = rectangles;
    run_thumb(machine, 0x0ee204, 0x0ee274);
    check_ewram(machine, rectangles + 8, &rom[0x39db64..0x39db7c]);

    // Run the original root load and message constructor up to the scheduler.
    machine.cpu.regs[5] = 0x0201_8000;
    run_thumb(machine, 0x0ee53a, 0x0dd3d4);
    assert_eq!(machine.cpu.regs[0], 0x080d_cc81);
    let start = (machine.cpu.regs[2] - 0x0300_0000) as usize;
    let mut message = [0; 0x1c];
    message.copy_from_slice(&machine.bus.iwram[start..start + 0x1c]);
    assert_eq!(
        u32::from_le_bytes(message[4..8].try_into().unwrap()),
        0x0839_db24
    );
    assert_eq!((message[0x14], message[0x16], message[0x18]), (3, 0, 8));
    message[..2].fill(0);
    machine.bus.ewram[0x18000..0x1801c].copy_from_slice(&message);

    let state = 0x0201_9000;
    machine.cpu.regs[1] = 0x0201_8000;
    machine.cpu.regs[2] = state;
    run_thumb(machine, 0x0dcc80, 0x0dcd12);
    check_ewram(machine, state + 8, &[3, 0, 8]);
    for row in 0..8 {
        let start = 0x39db24 + row * 8;
        let dest = state + 0x16c + row as u32 * 8;
        check_ewram(machine, dest, &rom[start..start + 6]);
        check_ewram(machine, dest + 6, &[0; 2]);
    }
}

fn run_indirect_readers(machine: &mut Machine, rom: &[u8]) {
    let context = 0x0201_a000;
    for (entry, start, count) in [
        (0x0ee558, 0x39db88, 1),
        (0x0ee32e, 0x39db90, 4),
        (0x0ee3a6, 0x39dbb0, 3),
    ] {
        run_thumb(machine, entry, 0x0e389c);
        let root = machine.cpu.regs[0];
        assert_eq!((root, machine.cpu.regs[1]), (ROM_BASE + start, count));
        machine.bus.write32(context + 0xc, root);
        for index in 0..count {
            let record = root + index * 8;
            let bytes = &rom[(record - ROM_BASE) as usize..][..8];
            machine.cpu.regs[5] = context;
            machine.cpu.regs[2] = index;
            run_thumb(machine, 0x0e3682, 0x0e368c);
            assert_eq!(
                machine.cpu.regs[0],
                u32::from(u16::from_le_bytes([bytes[0], bytes[1]])),
                "first halfword at {record:08x}"
            );

            for lane in 0..2 {
                machine.cpu.regs[5] = context;
                machine.cpu.regs[6] = lane;
                machine.cpu.regs[8] = index * 8;
                run_thumb(machine, 0x0e36a0, 0x0e36ac);
                let at = 4 + lane as usize * 2;
                assert_eq!(
                    machine.cpu.regs[2],
                    u32::from(u16::from_le_bytes([bytes[at], bytes[at + 1]])),
                    "halfword {lane} at {record:08x}"
                );

                machine.cpu.regs[1] = record;
                run_thumb(machine, 0x0e36b0, 0x0e36b4);
                assert_eq!(
                    machine.cpu.regs[0],
                    u32::from(bytes[2 + lane as usize]),
                    "byte {lane} at {record:08x}"
                );
            }
        }
    }
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
#[ignore = "exact EUR ROM, E522 source plan and e321-d320 predecessor manifest"]
fn european_state_initializer_publication_and_original_readers() {
    let work = input("FFTA_STATE_INITIALIZER_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_STATE_INITIALIZER_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_STATE_INITIALIZER_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E520 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (321, 320)
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
        (12, 204)
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
        .join("v1/e322-d321");
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
    let manifest: Manifest = parse(&manifest_bytes, "E523 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (322, 321)
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
        .state_initializers
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
    let boundaries = sample_altered(&mut managed, &BLEND_AND_GAP);
    assert_eq!(
        boundaries,
        [rom[0x39db84], rom[0x39db85], !rom[0x39db86], !rom[0x39db87]]
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
    assert_eq!(sample_altered(&mut managed, &BLEND_AND_GAP), boundaries);
    let expected_peers = OTHER_TEMPLATES.map(|offset| !rom[offset]);
    assert_eq!(
        sample_altered(&mut managed, &OTHER_TEMPLATES),
        expected_peers
    );

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
        run_direct_readers(machine, &rom);
        run_indirect_readers(machine, &rom);
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
    assert_eq!((payload, selectors), (164, 24));
    assert_eq!(
        after.source_bytes - before.source_bytes,
        payload + selectors
    );

    let result = serde_json::json!({
        "status": "PASS", "revision": [322, 321], "spans": added.len(), "new_bytes": 204,
        "direct_readers": 3, "indirect_groups": [1, 4, 3],
        "payload_bytes_read": payload, "selector_bytes_read": selectors,
        "full_state_equal": true, "predecessor_attach": true, "blend_and_gap_preserved": true,
        "peer_fallback_preserved": true, "missing_group_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
