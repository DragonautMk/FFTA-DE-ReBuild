//! One reduced Wine check for the new FFFF numeric reader fields.
use super::*;
use gba_core::{
    rom_data::{metadata52, script_archive, unit_obj_ffff_reader_fields::*},
    Machine,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const STACK: u32 = 0x03007e00;
fn cpu(m: &mut Machine, pc: u32) {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
fn original_reader(m: &mut Machine, pc: u32, animation: u32, covered: &mut BTreeSet<u32>) -> usize {
    cpu(m, pc);
    m.cpu.regs[0] = u32::from(SPRITE_ID);
    m.cpu.regs[1] = animation;
    let callee = m.cpu.regs[4..12].to_vec();
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut instructions = 0;
    while m.cpu.regs[15] != 0x08000300 {
        assert!(instructions < 64, "numeric reader {:08x}", m.cpu.regs[15]);
        let width = match m.cpu.regs[15] {
            0x080221ca => 2,
            0x0802217e => 4,
            _ => 0,
        };
        if width != 0 {
            let at = m.cpu.regs[0] - 0x08000000;
            covered.extend(at..at + width);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
    }
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(&m.cpu.regs[4..12], callee.as_slice());
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw;
    if pc == 0x08022174 {
        // The complete helper pushes and pops only its original LR word.
        actual[0x7dfc..0x7e00].fill(0);
        expected[0x7dfc..0x7e00].fill(0);
    }
    assert_eq!(actual, expected);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR FFFF reader-field model/original loads; Wine only, Linux/USA pending"]
fn european_ffff_reader_fields_match_source_producers_and_original_loads() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FFFF_FIELDS_ROOT").unwrap());
    let output = root.join("windows-model-europe");
    assert!(!output.exists());
    fs::create_dir(&output).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: UnitObjFfffReaderFields =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &independent);
    data.validate().unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let jobs: metadata52::Metadata52 =
        serde_json::from_slice(&fs::read(dataset.join("linked-metadata52.json")).unwrap()).unwrap();
    let scripts: script_archive::ScriptArchive =
        serde_json::from_slice(&fs::read(dataset.join("script-archive.json")).unwrap()).unwrap();
    data.validate_dependencies(&scripts, &jobs).unwrap();
    let manifest: gba_core::rom_data::Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (69, 68)
    );
    assert_eq!(manifest.resources.len(), 63);
    assert_eq!(gba_core::rom_data::EXTRACTOR_REV, 69);
    assert_eq!(gba_core::rom_data::DECODER_REV, 68);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 2);
    let mut bytes = BTreeSet::new();
    for ((at, raw, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(raw, &rom[*at as usize..*at as usize + raw.len()]);
        assert_eq!((source.offset, source.length), (*at, raw.len() as u32));
        assert_eq!(&source.role, role);
        assert_eq!(source.sha256, gba_core::ffta_rom::digest(raw));
        assert!(manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .all(|s| { s.offset + s.length <= *at || *at + raw.len() as u32 <= s.offset }));
        assert_eq!(data.byte_at(at - 1), None);
        assert_eq!(data.byte_at(at + raw.len() as u32), None);
        for (i, &byte) in raw.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(byte));
            bytes.insert(*at + i as u32);
        }
    }
    assert_eq!(bytes.len(), 6);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UnitObjFfffReaderFields>(&encoded).unwrap(),
        *data
    );
    fs::write(output.join("decoded.json"), &encoded).unwrap();

    let mut rejected = 0;
    let mut reject = |invalid: UnitObjFfffReaderFields| {
        assert!(invalid.validate().is_err());
        rejected += 1;
    };
    let mut invalid = data.clone();
    invalid.sprite_id = 255;
    reject(invalid);
    let mut invalid = data.clone();
    invalid.property_word.source_offset += 2;
    reject(invalid);
    let mut invalid = data.clone();
    invalid.directory_word.source_offset -= 4;
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers.pop();
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers.swap(0, 1);
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers[0].effective_job = 2;
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers[0].alternate_job = Some(0);
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers[0].primary_routing_source += 1;
    reject(invalid);
    let mut invalid = data.clone();
    invalid.producers[0].script_sources[0] += 1;
    reject(invalid);
    let mut malformed: Value = serde_json::from_slice(&encoded).unwrap();
    malformed["directory_word"]["extra"] = json!(0);
    assert!(serde_json::from_value::<UnitObjFfffReaderFields>(malformed).is_err());
    rejected += 1;
    let mut wrong_jobs = jobs.clone();
    wrong_jobs.records[80].routing = 0;
    assert!(data.validate_dependencies(&scripts, &wrong_jobs).is_err());
    rejected += 1;
    let mut wrong_scripts = scripts.clone();
    let instruction = wrong_scripts
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 49 && i.parameters[0] >= 128)
        .unwrap();
    instruction.parameters[0] = 127;
    assert!(data.validate_dependencies(&wrong_scripts, &jobs).is_err());
    rejected += 1;
    let mut edited = data.clone();
    edited.property_word.value ^= 0x8001;
    edited.directory_word.value ^= 0x80000001;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    rejected += 1;

    // Numeric values remain lossless, including zero, every high bit and low
    // pointer bits. Structural validation does not fabricate an asset target.
    for (property, directory) in [(0, 0), (u16::MAX, u32::MAX), (0x8001, 0x14000002)] {
        let mut numeric = data.clone();
        numeric.property_word.value = property;
        numeric.directory_word.value = directory;
        numeric.validate().unwrap();
        let reconstructed = numeric.source_spans().unwrap();
        assert_eq!(reconstructed[0].1, property.to_le_bytes());
        assert_eq!(reconstructed[1].1, directory.to_le_bytes());
        let roundtrip: UnitObjFfffReaderFields =
            serde_json::from_slice(&serde_json::to_vec(&numeric).unwrap()).unwrap();
        assert_eq!(roundtrip, numeric);
    }

    let mut machine = Machine::new(rom.clone());
    machine.bus.ewram.fill(0x5a);
    machine.bus.iwram.fill(0x3c);
    machine.bus.vram.fill(0xa5);
    machine.bus.oam.fill(0xc3);
    machine.bus.palette.fill(0x24);
    let mut covered = BTreeSet::new();
    let mut instructions = original_reader(&mut machine, 0x080221c4, 0, &mut covered);
    assert_eq!(machine.cpu.regs[0], u32::from(data.property_word.value));
    instructions += original_reader(&mut machine, 0x08022174, 1, &mut covered);
    assert_eq!(machine.cpu.regs[3], data.directory_word.value);
    assert_eq!(
        machine.cpu.regs[0],
        data.directory_word.value.wrapping_add(12)
    );
    assert_eq!(covered, bytes);
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "producers": data.producers.len(), "references": 2240,
            "spans": spans.len(), "source_bytes": bytes.len(), "original_read_bytes": covered.len(),
            "original_reader_calls": 2, "original_instructions": instructions,
            "guards": rejected, "numeric_roundtrips": 3, "JSON_exact": true,
            "source_and_dependencies_exact": true, "all63_current_owners_disjoint": true,
            "production_revision_unchanged": [1,69,68], "USA_revision_unchanged": [8,7],
            "unmapped_target_not_decoded": true, "native_clock_ordinary": "pending integration",
            "linux": "pending", "usa": "pending", "performance_measurements": false
        }))
        .unwrap(),
    )
    .unwrap();
}
