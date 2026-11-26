//! Original EUR literal-rooted initializer copies with altered backing.
use super::{attach_from_directory, cmd, encode_copied_halfword_lookups,
    indexed_reader_bank_tests::{execute_until, initialize}, parse, parse_copied_halfword_lookups,
    Manifest, COPIED_HALFWORD_LOOKUPS_FILE, COPIED_HALFWORD_LOOKUPS_ID, MANIFEST_FILE};
use gba_core::{capi::RT_API, ffta_rom::{digest, EUROPE_SHA256}, rom_data::{copied_halfword_lookups::initializer_bank::SPECS, SourceRange}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

#[derive(Deserialize)]
struct Plan { cases: Vec<Case> }
#[derive(Deserialize)]
struct Case { site: u32, literal: u32, source: u32, words: usize, source_sha256: String, source_owners: Vec<serde_json::Value> }
fn input(name: &str) -> PathBuf { std::env::var_os(name).unwrap().into() }

#[test]
#[ignore = "exact EUR ROM, authenticated Wine census and e334 predecessor manifest"]
fn european_initializer_copy_bank_uses_extracted_sources() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_INITIALIZER_WORK");
    let plan: Plan = serde_json::from_slice(&fs::read(input("FFTA_INITIALIZER_PLAN")).unwrap()).unwrap();
    let cases: Vec<_> = plan.cases.iter().filter(|case| case.source_owners.is_empty()).collect();
    assert_eq!((plan.cases.len(), cases.len()), (17, 14));
    for (case, &(site, literal, source, words, hash)) in cases.iter().zip(&SPECS) {
        assert_eq!((case.site, case.literal, case.source, case.words, case.source_sha256.as_str()), (site, literal, source, words, hash));
    }
    let previous_bytes = fs::read(input("FFTA_INITIALIZER_PREDECESSOR")).unwrap();
    assert_eq!(digest(&previous_bytes), "f16de91ae893a0bbad6584c430b5b86a58e45eda879051986e9ba6593091e351");
    let previous: Manifest = parse(&previous_bytes, "e334 manifest").unwrap();
    assert_eq!((previous.extractor_revision, previous.decoder_revision), (334, 333));
    let resources = tempfile::tempdir_in(&work).unwrap();
    let args = ["prepare".into(), rom_path.to_str().unwrap().into(), resources.path().to_str().unwrap().into()];
    cmd(&args).unwrap();
    cmd(&args).unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e335-d334");
    let dataset = revision.join(fs::read_to_string(revision.join("current")).unwrap().trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(COPIED_HALFWORD_LOOKUPS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "e335 manifest").unwrap();
    let mut packet = parse_copied_halfword_lookups(&packet_bytes).unwrap();
    let bank = packet.initializer_bank.take().unwrap();
    bank.validate().unwrap();
    let spans = bank.source_spans();
    assert_eq!((spans.len(), spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>()), (28, 260));
    let index = previous.resources.iter().position(|r| r.id == COPIED_HALFWORD_LOOKUPS_ID).unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 335;
    expected.decoder_revision = 334;
    expected.resources[index].sha256 = digest(&packet_bytes);
    expected.resources[index].sources.extend(spans.iter().map(|(offset, bytes, role)| SourceRange {
        offset: *offset, length: bytes.len() as u32, sha256: digest(bytes), role: (*role).into(),
    }));
    assert_eq!(manifest, expected);
    let old_packet = encode_copied_halfword_lookups(&packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, &previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 335;
    incomplete.decoder_revision = 334;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut raw = Machine::new(rom.clone());
    let origin = raw.save_state().unwrap();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let mut addresses = BTreeSet::new();
    for case in &cases {
        for pc in (case.site - 2..case.site + 4 * (case.words as u32 / 3)).step_by(2) { addresses.insert(0x0800_0000 + pc); }
    }
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = work.join("initializer-copy-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let alter = |machine: &mut Machine, poison: bool| {
        for (offset, bytes, _) in &spans {
            let start = *offset as usize;
            for (i, byte) in bytes.iter().enumerate() { machine.bus.rom[start + i] = if poison { !byte } else { *byte }; }
        }
    };
    for native in [false, true] {
        for case in &cases {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&origin).unwrap();
                initialize(machine, 0x0800_0000 + case.site - 2);
                machine.cpu.regs[1] = 0x0300_7000;
            }
            alter(&mut managed, true);
            let before = owner.copied_halfword_lookups_stats().unwrap().initializer_bytes;
            let stop = 0x0800_0000 + case.site + 4 * (case.words as u32 / 3);
            execute_until(&mut raw, stop, 32);
            for _ in 0..32 {
                let pc = managed.cpu.regs[15];
                if pc == stop { break; }
                managed.bus.frame_ready = false;
                if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
                else { managed.step(); }
            }
            assert_eq!(managed.cpu.regs[15], stop);
            assert_eq!(owner.copied_halfword_lookups_stats().unwrap().initializer_bytes - before, (4 + 4 * case.words) as u64);
            let start = case.source as usize;
            assert_eq!(&managed.bus.iwram[0x7000..0x7000 + 4 * case.words], &rom[start..start + 4 * case.words]);
            alter(&mut managed, false);
            assert_eq!(raw.bus.clock, managed.bus.clock);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "site {:06x}, native={native}", case.site);
        }
    }
    alter(&mut managed, true);
    for (offset, bytes, _) in &spans {
        for at in offset.saturating_sub(3)..offset + bytes.len() as u32 + 3 {
            assert_eq!(raw.bus.read32(0x0800_0000 + at), managed.bus.read32(0x0800_0000 + at), "boundary {at:06x}");
        }
    }
    alter(&mut managed, false);
    let stats = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(stats.initializer_mask, 0x3fff);
    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(work.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[335,334], "manifest_sha256":digest(&manifest_bytes),
        "new_cases":cases.len(), "new_source_bytes":204, "new_literal_bytes":56,
        "initializer_bytes":stats.initializer_bytes, "initializer_mask":stats.initializer_mask,
        "predecessor_attach":true, "missing_group_rejected":true, "interpreter_and_generated_full_state_clock_equal":true,
    })).unwrap()).unwrap();
}
