//! Verify the E519 source families through the original copy paths.
use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Extraction,
    Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const TABLE: usize = 0x3adc88;
const FIRST_OTHER_OWNER: u32 = 0xb57334;
const MODE3_WORDS: [u32; 2] = [0x3adca0, 0x3adca4];

#[derive(Deserialize)]
struct Plan {
    spans: Vec<ReferenceSpan>,
    colors: [u8; 3],
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

pub(super) fn run_thumb(machine: &mut Machine, entry: u32, stop: u32) {
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == stop {
            return;
        }
        machine.step();
    }
    panic!(
        "Reader {entry:08x} did not reach {stop:08x}. PC {:08x}",
        machine.cpu.regs[15]
    );
}

fn check_bytes(machine: &Machine, address: u32, expected: &[u8]) {
    let bytes = match address >> 24 {
        2 => &machine.bus.ewram[..],
        3 => &machine.bus.iwram[..],
        6 => &machine.bus.vram[..],
        _ => panic!("Unexpected copy destination {address:08x}"),
    };
    let offset = (address & 0x00ff_ffff) as usize;
    assert_eq!(
        &bytes[offset..offset + expected.len()],
        expected,
        "copy to {address:08x}"
    );
}

fn run_fixed_copies(machine: &mut Machine, rom: &[u8]) {
    let rows = [
        (0x0300_0e60, 0x0600_2000, 0xb558d4, 32),
        (0x0300_0e80, 0x0600_3000, 0xb55dd4, 30),
        (0x0300_08b0, 0x0600_4000, 0xb56284, 30),
    ];
    for (context, destination, _, _) in rows {
        machine.bus.write32(context + 0x14, destination);
    }
    run_thumb(machine, 0x0813_9a38, 0x0813_9a70);

    run_thumb(machine, 0x0814_0490, 0x0814_049c);
    check_bytes(machine, 0x0600_0000, &rom[0xb54f54..0xb55900]);

    machine.bus.write32(0x0201_c3d0, 0x0600_5000);
    machine.cpu.regs[4] = 0x0201_c3d0;
    run_thumb(machine, 0x0814_04b0, 0x0814_04c4);

    for (_, destination, source, columns) in
        rows.into_iter().chain([(0, 0x0600_5002, 0xb54a58, 30)])
    {
        let row_bytes = columns * 2;
        for row in 0..20 {
            let start = source + row * row_bytes;
            let target = destination + row as u32 * 64;
            check_bytes(machine, target, &rom[start..start + row_bytes]);
            check_bytes(
                machine,
                target + row_bytes as u32,
                &[0; 4][..64 - row_bytes],
            );
        }
    }
}

fn run_color_mode(machine: &mut Machine, rom: &[u8], mode: u8) {
    let banks: [usize; 2] = std::array::from_fn(|i| {
        let at = TABLE + usize::from(mode) * 8 + i * 4;
        let address = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        (address - ROM_BASE) as usize
    });
    machine.bus.write8(0x0200_2fc2, mode | 0xa0);
    // This is the original relocated Thumb callback at 03005ee8.
    machine.bus.iwram[0x5ee8..0x5f28].copy_from_slice(&rom[0xd968a8..0xd968e8]);

    for (entry, stop, destination) in [
        (0x0813_9a70, 0x0813_9abe, 0x0201_8000),
        (0x0813_9b50, 0x0813_9ba2, 0x0201_9000),
    ] {
        machine.bus.write32(0x0200_f3b0, destination - 0x8000);
        machine.cpu.regs[5] = 600;
        machine.cpu.regs[6] = 0x0200_0000;
        run_thumb(machine, entry, stop);
        for (i, at) in banks.into_iter().enumerate() {
            check_bytes(machine, destination + 512 * i as u32, &rom[at..at + 512]);
        }
    }

    run_thumb(machine, 0x0813_9be4, 0x0813_9c28);
    for (i, at) in banks.into_iter().enumerate() {
        check_bytes(machine, 0x0300_3860 + 512 * i as u32, &rom[at..at + 512]);
    }

    for (reader, destination) in [(0x0813_b26a, 0x0201_4000), (0x0813_b8e4, 0x0201_4200)] {
        // The second reader originally retains the table root in r9 across its loop.
        if reader == 0x0813_b8e4 {
            run_thumb(machine, 0x0813_b894, 0x0813_b898);
        }
        machine.cpu.regs[12] = destination;
        machine.cpu.regs[5] = destination;
        machine.cpu.regs[0] = 0;
        machine.cpu.regs[1] = 1;
        machine.cpu.regs[4] = 0;
        run_thumb(machine, reader, 0x0813_b732);
        let odd = banks[1];
        check_bytes(machine, destination + 0xc, &rom[odd + 0x40..odd + 0x60]);
        check_bytes(machine, destination + 0x2c, &rom[odd + 0x1e0..odd + 0x200]);
    }
}

fn reads_with_altered_words(machine: &mut Machine, addresses: [u32; 2]) -> [u32; 2] {
    addresses.map(|offset| {
        let at = offset as usize;
        for byte in &mut machine.bus.rom[at..at + 4] {
            *byte ^= 0xff;
        }
        let value = machine.bus.read32(ROM_BASE + offset);
        for byte in &mut machine.bus.rom[at..at + 4] {
            *byte ^= 0xff;
        }
        value
    })
}

#[test]
#[ignore = "exact EUR ROM, E519 source plan and e320-d319 predecessor manifest"]
fn european_row_and_color_mode_bank_publication_and_original_copies() {
    let work = input("FFTA_ROW_PAIR_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_ROW_PAIR_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.colors, [0, 1, 2]);
    let prior_bytes = fs::read(input("FFTA_ROW_PAIR_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E518 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (320, 319)
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
        (14, 10_452)
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
        .join("v1/e321-d320");
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
    let manifest: Manifest = parse(&manifest_bytes, "E520 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (321, 320)
    );
    let index = manifest
        .resources
        .iter()
        .position(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let old = &prior.resources[index];
    let new = &manifest.resources[index];
    let mut previous_layout = manifest.clone();
    previous_layout.resources[index] = old.clone();
    let missing = previous_layout.clone();
    previous_layout.extractor_revision = prior.extractor_revision;
    previous_layout.decoder_revision = prior.decoder_revision;
    assert_eq!(previous_layout, prior);
    assert_eq!(
        (&new.id, &new.path, &new.format, &new.dependencies),
        (&old.id, &old.path, &old.format, &old.dependencies)
    );
    let mut expected_sources = old.sources.clone();
    expected_sources.extend(added.iter().cloned());
    expected_sources.sort_unstable_by_key(|span| span.offset);
    assert_eq!(new.sources, expected_sources);

    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    assert!(packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .row_pairs
        .take()
        .is_some());
    let previous_packet = Extraction::from_rom_for_manifest(&rom, &prior)
        .unwrap()
        .ui_raw_transfers
        .unwrap()
        .decoded;
    assert_eq!(packet, previous_packet);
    let previous_packet = encode_ui_raw_transfers(&previous_packet).unwrap();
    assert_eq!(digest(&previous_packet), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    fs::write(&packet_path, &previous_packet).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let previous_mode3 = reads_with_altered_words(&mut managed, MODE3_WORDS);
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(&manifest_path, toml::to_string_pretty(&missing).unwrap()).unwrap();
    fs::write(&packet_path, previous_packet).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();
    assert_eq!(
        reads_with_altered_words(&mut managed, MODE3_WORDS),
        previous_mode3
    );

    let alter_backing = |machine: &mut Machine| {
        for span in &added {
            let at = span.offset as usize;
            for byte in &mut machine.bus.rom[at..at + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    for mode in std::iter::once(None).chain(plan.colors.map(Some)) {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.bus.ffta_sprites.enabled = false;
        }
        alter_backing(&mut managed);
        for machine in [&mut raw, &mut managed] {
            match mode {
                Some(mode) => run_color_mode(machine, &rom, mode),
                None => run_fixed_copies(machine, &rom),
            }
        }
        alter_backing(&mut managed);
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(
            digest(&raw.save_state().unwrap()),
            digest(&managed.save_state().unwrap()),
            "mode {mode:?}"
        );
    }
    let stats = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.payload_bytes,
            stats.selector_bytes
        ),
        (17_160, 16_956, 204)
    );
    assert_eq!((stats.banks_used, stats.groups_used), (2, 1));

    for offset in [0xb54f08, 0xb54f53, 0xb5f334, 0xb5f41b] {
        managed.bus.rom[offset] ^= 0xff;
        assert_eq!(managed.bus.read8(ROM_BASE + offset as u32), !rom[offset]);
    }
    let at = FIRST_OTHER_OWNER as usize;
    managed.bus.rom[at] ^= 0xff;
    assert_eq!(managed.bus.read8(ROM_BASE + FIRST_OTHER_OWNER), rom[at]);

    let result = serde_json::json!({
        "status": "PASS", "revision": [321, 320], "spans": added.len(), "new_bytes": 10452,
        "fixed_copies": 5, "color_reader_cases": 15, "modes": plan.colors,
        "payload_bytes_read": stats.payload_bytes, "selector_bytes_read": stats.selector_bytes,
        "full_state_equal": true, "predecessor_attach": true, "gaps_preserved": true,
        "mode3_owner_preserved": true, "previous_bank_preserved": true, "missing_row_pair_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
