//! Compare original EUR timed-sound callers and record reads with extracted data.
use super::{
    attach_from_directory, cmd, encode_audio_directory,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse, parse_audio_directory, Manifest, AUDIO_DIRECTORY_FILE, AUDIO_DIRECTORY_ID,
    MANIFEST_FILE,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const STACK: u32 = 0x0300_7c00;

#[derive(Deserialize)]
struct Plan {
    source_bytes: u32,
    cases: Vec<Case>,
    spans: Vec<SourceRange>,
    earlier: Vec<PriorSpan>,
}
#[derive(Deserialize)]
struct PriorSpan {
    offset: u32,
    length: u32,
    sha256: String,
}
#[derive(Deserialize)]
struct Case {
    site: u32,
    entry: u32,
    source: u32,
    count: u32,
    registers: Vec<(usize, u32)>,
    index: Option<(u32, u8)>,
    ram: bool,
    code_sha256: String,
}
fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_paths(machine: &mut Machine, plan: &Plan, rom: &[u8]) {
    for case in &plan.cases {
        initialize(machine, BASE + case.entry);
        for &(reg, value) in &case.registers {
            machine.cpu.regs[reg] = value;
        }
        if let Some((address, value)) = case.index {
            machine.bus.write8(address, value);
        }
        if case.site == 0x1096e8 {
            for index in 0..4u32 {
                let value = u16::from_le_bytes(
                    rom[0x14d84c + index as usize * 2..0x14d84e + index as usize * 2]
                        .try_into()
                        .unwrap(),
                );
                machine.bus.write16(STACK + 0x10 + 2 * index, value);
            }
        }
        execute_until(machine, BASE + case.site, 500);
        assert_eq!(
            machine.cpu.regs[0],
            if case.ram {
                case.source
            } else {
                BASE + case.source
            },
            "caller {:06x}",
            case.site
        );
        assert_eq!(machine.cpu.regs[1], case.count, "count {:06x}", case.site);
        if case.ram {
            continue;
        }
        for index in 0..case.count {
            let offset = case.source + 8 * index;
            for (pc, result_reg, expected) in [
                (
                    0x0e3688,
                    0,
                    u32::from(u16::from_le_bytes(
                        rom[offset as usize..offset as usize + 2]
                            .try_into()
                            .unwrap(),
                    )),
                ),
                (
                    0x0e36aa,
                    2,
                    u32::from(u16::from_le_bytes(
                        rom[offset as usize + 4..offset as usize + 6]
                            .try_into()
                            .unwrap(),
                    )),
                ),
                (0x0e36b2, 0, u32::from(rom[offset as usize + 2])),
            ] {
                initialize(machine, BASE + pc);
                machine.cpu.regs[0] = BASE + offset;
                machine.cpu.regs[1] = BASE + offset;
                machine.bus.frame_ready = false;
                machine.step();
                assert_eq!(
                    machine.cpu.regs[result_reg], expected,
                    "record {offset:06x} reader {pc:06x}"
                );
            }
        }
    }
}

#[test]
#[ignore = "exact EUR ROM, E539 sound plan and e333 predecessor manifest"]
fn european_timed_sound_banks_use_extracted_sources() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_TIMED_SOUND_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_TIMED_SOUND_PLAN")).unwrap()).unwrap();
    assert_eq!((plan.cases.len(), plan.source_bytes), (552, 6249));
    for (offset, length, sha256) in plan
        .spans
        .iter()
        .map(|span| (span.offset, span.length, &span.sha256))
        .chain(
            plan.earlier
                .iter()
                .map(|span| (span.offset, span.length, &span.sha256)),
        )
    {
        let at = offset as usize;
        assert_eq!(digest(&rom[at..at + length as usize]), *sha256);
    }
    for case in &plan.cases {
        assert_eq!(
            digest(&rom[case.entry as usize..case.site as usize + 4]),
            case.code_sha256
        );
    }
    let previous_bytes = fs::read(input("FFTA_TIMED_SOUND_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E538 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (333, 332)
    );
    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e334-d333");
    let dataset = revision.join(fs::read_to_string(revision.join("current")).unwrap().trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(AUDIO_DIRECTORY_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E539 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|r| r.id == AUDIO_DIRECTORY_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 334;
    expected.decoder_revision = 333;
    expected.resources[index].sha256 = digest(&packet_bytes);
    expected.resources[index]
        .sources
        .extend(plan.spans.iter().cloned());
    assert_eq!(manifest, expected);

    let mut packet = parse_audio_directory(&packet_bytes).unwrap();
    assert!(packet.timed_sounds.take().is_some());
    let old_packet = encode_audio_directory(&packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 334;
    incomplete.decoder_revision = 333;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);

    let mut raw = Machine::new(rom.clone());
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    for span in &plan.spans {
        let start = span.offset as usize;
        for byte in &mut managed.bus.rom[start..start + span.length as usize] {
            *byte ^= 0xff;
        }
    }
    run_paths(&mut raw, &plan, &rom);
    run_paths(&mut managed, &plan, &rom);
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .audio_directory_stats()
        .unwrap();
    assert!(stats.timed_sound_bytes > 0);
    for span in &plan.spans {
        let start = span.offset as usize;
        managed.bus.rom[start..start + span.length as usize]
            .copy_from_slice(&rom[start..start + span.length as usize]);
    }
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(work.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[334,333],"manifest_sha256":digest(&manifest_bytes),
        "cases":plan.cases.len(),"source_bytes":plan.source_bytes,"timed_sound_bytes":stats.timed_sound_bytes,
        "predecessor_attach":true,"missing_group_rejected":true,"full_state_clock_equal":true,
    })).unwrap()).unwrap();
}
