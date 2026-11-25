//! Exercise all E512 source banks through the original copy and selector code.
use super::{
    Extraction, MANIFEST_FILE, Manifest, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers,
};
use gba_core::{Bus, Machine, ffta_rom::digest};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const VRAM: u32 = 0x0600_0000;
const DESTINATION: u32 = VRAM + 0x6500;
const VIDEO_CONTEXT: u32 = 0x0300_09e0;
const SELECTOR_CONTEXT: u32 = 0x0201_0000;

#[derive(Deserialize)]
struct Plan {
    direct: Vec<DirectCall>,
    localized: Vec<LocalizedCall>,
    sources: Vec<SourceWitness>,
}

#[derive(Deserialize)]
struct DirectCall {
    site: u32,
    load: u32,
    offset: u32,
    halfwords: usize,
    bias: u16,
}

#[derive(Deserialize)]
struct LocalizedCall {
    site: u32,
    pointers: [u32; 5],
    rows: [u32; 2],
    halfwords_per_row: usize,
    bias: u16,
}

#[derive(Deserialize)]
struct SourceWitness {
    offset: u32,
    length: u32,
    role: String,
    sha256: String,
}

struct CopyCase {
    entry: u32,
    call: u32,
    source: u32,
    halfwords: usize,
    bias: u16,
    language: u8,
    row: Option<u32>,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_original(machine: &mut Machine, case: &CopyCase) {
    machine.bus.write8(0x0200_1f6e, case.language);
    for field in [0xb4, 0x100, 0x148, 0x14c, 0x2c4] {
        machine.bus.write32(VIDEO_CONTEXT + field, DESTINATION);
    }
    if case.row.is_some() {
        machine.bus.write32(0x0300_2818, SELECTOR_CONTEXT);
        let flag = if case.call == 0x075c86 { 0x20 } else { 0 };
        machine.bus.write8(SELECTOR_CONTEXT + 0x98b, flag);
    }
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[1] = case.row.unwrap_or(VIDEO_CONTEXT);
    machine.cpu.regs[4] = VIDEO_CONTEXT;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[15] = ROM_BASE + case.entry;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    let end = ROM_BASE + case.call + 4;
    for _ in 0..3500 {
        if machine.cpu.regs[15] == end {
            return;
        }
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!(
        "Halfword copy {:06x} escaped at {:08x}",
        case.call, machine.cpu.regs[15]
    );
}

#[test]
#[ignore = "exact EUR ROM, E512 source plan and E510 predecessor manifest"]
fn european_halfword_copy_bank_publication_and_original_paths() {
    let work = input("FFTA_HALFWORD_COPY_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_HALFWORD_COPY_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_HALFWORD_COPY_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E510 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (317, 316)
    );

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
        .join("v1/e318-d317");
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
    let manifest: Manifest = parse(&manifest_bytes, "E513 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (318, 317)
    );
    assert_eq!(manifest.coverage, prior.coverage);
    let index = manifest
        .resources
        .iter()
        .position(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    assert_eq!(manifest.resources.len(), prior.resources.len());
    for (i, (current, previous)) in manifest.resources.iter().zip(&prior.resources).enumerate() {
        if i != index {
            assert_eq!(current, previous);
        }
    }
    let old = &prior.resources[index];
    let new = &manifest.resources[index];
    assert_eq!(
        (&new.path, &new.format, &new.dependencies),
        (&old.path, &old.format, &old.dependencies)
    );
    assert!(
        old.sources
            .iter()
            .all(|source| new.sources.contains(source))
    );
    let added: Vec<_> = new
        .sources
        .iter()
        .filter(|source| !old.sources.contains(source))
        .collect();
    assert_eq!((added.len(), plan.sources.len()), (29, 29));
    for (source, witness) in added.iter().zip(&plan.sources) {
        assert_eq!(
            (source.offset, source.length, &source.role, &source.sha256),
            (
                witness.offset,
                witness.length,
                &witness.role,
                &witness.sha256
            )
        );
        assert!(prior.resources.iter().flat_map(|r| &r.sources).all(|span| {
            span.offset + span.length <= source.offset
                || source.offset + source.length <= span.offset
        }));
    }

    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    assert_eq!(packet.halfword_copies.take().unwrap().banks.len(), 16);
    let prior_packet = Extraction::from_rom_for_manifest(&rom, &prior)
        .unwrap()
        .ui_raw_transfers
        .unwrap()
        .decoded;
    assert_eq!(packet, prior_packet);
    let prior_packet_bytes = encode_ui_raw_transfers(&prior_packet).unwrap();
    assert_eq!(digest(&prior_packet_bytes), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    fs::write(&packet_path, &prior_packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let first = added[0].offset as usize;
    managed.bus.rom[first] ^= 0xff;
    assert_eq!(managed.bus.read8(ROM_BASE + first as u32), !rom[first]);
    managed.bus.rom[first] ^= 0xff;
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let mut missing = manifest.clone();
    missing.resources[index] = old.clone();
    fs::write(&manifest_path, toml::to_string_pretty(&missing).unwrap()).unwrap();
    fs::write(&packet_path, prior_packet_bytes).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();

    let poison = |machine: &mut Machine| {
        for span in &added {
            let offset = span.offset as usize;
            for byte in &mut machine.bus.rom[offset..offset + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    poison(&mut managed);
    let before = owner.ui_raw_transfers_stats().unwrap();
    for span in &added {
        for offset in span.offset..span.offset + span.length {
            assert_eq!(
                managed.bus.read8(ROM_BASE + offset),
                rom[offset as usize],
                "{offset:06x}"
            );
        }
    }
    let after = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 2316);
    assert_eq!(after.payload_bytes - before.payload_bytes, 2232);
    assert_eq!(after.selector_bytes - before.selector_bytes, 84);
    assert_eq!((after.banks_used, after.groups_used), (16, 2));
    // The next word belongs to the earlier indexed catalog, immediately
    // after the second E513 selector table.
    let next = 0x397d6c;
    let raw_after_bank = 0xb537a8;
    for value in &mut managed.bus.rom[next..next + 4] {
        *value ^= 0xff;
    }
    managed.bus.rom[raw_after_bank] ^= 0xff;
    for mirror in [ROM_BASE, 0x0a00_0000, 0x0c00_0000] {
        assert_eq!(
            managed.bus.read16(mirror + next as u32 - 2).to_le_bytes(),
            rom[next - 2..next]
        );
        assert_eq!(
            managed.bus.read32(mirror + next as u32).to_le_bytes(),
            rom[next..next + 4]
        );
        assert_eq!(
            managed.bus.read8(mirror + raw_after_bank as u32),
            !rom[raw_after_bank]
        );
    }
    for value in &mut managed.bus.rom[next..next + 4] {
        *value ^= 0xff;
    }
    managed.bus.rom[raw_after_bank] ^= 0xff;
    poison(&mut managed);

    let direct = plan.direct.iter().map(|call| CopyCase {
        entry: match call.site {
            0x039d08 | 0x039d4a => 0x039cee,
            0x08edbe => 0x08edac,
            _ => call.load,
        },
        call: call.site,
        source: call.offset,
        halfwords: call.halfwords,
        bias: call.bias,
        language: if call.site == 0x039d4a { 3 } else { 0 },
        row: None,
    });
    let localized_sources = plan.localized.iter().flat_map(|call| {
        call.pointers
            .iter()
            .enumerate()
            .flat_map(move |(language, pointer)| {
                call.rows.map(|row| CopyCase {
                    entry: 0x075c40,
                    call: call.site,
                    source: *pointer + 60 * row,
                    halfwords: call.halfwords_per_row,
                    bias: call.bias,
                    language: language as u8,
                    row: Some(row),
                })
            })
    });
    let mut raw = Machine::new(rom);
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    let mut copied_bytes = 0;
    for case in direct.chain(localized_sources) {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        poison(&mut managed);
        run_original(&mut raw, &case);
        run_original(&mut managed, &case);
        let start = case.source as usize;
        let length = 2 * case.halfwords;
        let expected: Vec<_> = raw.bus.rom[start..start + length]
            .chunks_exact(2)
            .flat_map(|bytes| {
                u16::from_le_bytes(bytes.try_into().unwrap())
                    .wrapping_add(case.bias)
                    .to_le_bytes()
            })
            .collect();
        let destination = (DESTINATION - VRAM) as usize;
        assert_eq!(&raw.bus.vram[destination..destination + length], expected);
        assert_eq!(
            &managed.bus.vram[destination..destination + length],
            expected
        );
        poison(&mut managed);
        assert_eq!(
            digest(&raw.save_state().unwrap()),
            digest(&managed.save_state().unwrap()),
            "call {:06x} language {} row {:?}",
            case.call,
            case.language,
            case.row
        );
        cases += 1;
        copied_bytes += length;
    }
    assert_eq!((cases, copied_bytes), (29, 2728));
    let result = serde_json::json!({
        "status": "PASS", "revision": [318, 317], "spans": added.len(), "source_bytes": 2316,
        "banks": 16, "selector_groups": 2, "original_copy_cases": cases, "copied_bytes": copied_bytes,
        "predecessor_attach": true, "missing_banks_rejected": true, "adjacent_owners_preserved": true,
        "full_state_equal": true, "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
