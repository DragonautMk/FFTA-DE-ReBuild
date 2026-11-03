//! One focused Wine source-model check; existing metadata52 oracles stay closed.
use super::*;
use gba_core::{
    rom_data::{metadata52, slot_job_lookup::*},
    Machine,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
const STACK: u32 = 0x03007e00;
fn original(
    m: &mut Machine,
    entry: u32,
    input: u32,
    expected: u32,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[0] = input;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + entry;
    m.cpu.cpsr = 0xbf;
    let prefix = entry == 0xd05c8;
    let stop = if prefix { 0x080d05dc } else { 0x08000300 };
    let callee = m.cpu.regs[4..12].to_vec();
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut n = 0;
    let mut loads = 0;
    while m.cpu.regs[15] != stop {
        assert!(n < 256, "entry{entry:06x} PC{:08x}", m.cpu.regs[15]);
        if matches!(
            m.cpu.regs[15],
            0x080d0540 | 0x080d0552 | 0x080d0572 | 0x080d0592 | 0x080d05b2 | 0x080d05d2
        ) {
            let at = m.cpu.regs[0] - 0x08000000;
            assert_eq!(at, SOURCE_OFFSET + (input & 255));
            covered.insert(at);
            loads += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
        assert!(m.cpu.regs[13] >= STACK - 20);
    }
    assert_eq!(loads, 1);
    assert_eq!(m.cpu.regs[0], expected);
    assert_eq!(m.cpu.regs[13], if prefix { STACK - 4 } else { STACK });
    assert_eq!(&m.cpu.regs[4..12], callee.as_slice());
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut before = iw;
    if entry != 0xd0538 {
        actual[0x7dec..0x7e00].fill(0);
        before[0x7dec..0x7e00].fill(0);
    }
    assert_eq!(actual, before);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    n
}
#[test]
#[ignore = "private EUR twenty-slot job source model/original lookups; Wine only, Linux/USA pending"]
fn european_slot_job_lookup_matches_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SLOT_JOB_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: SlotJobLookup =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let jobs: metadata52::Metadata52 =
        serde_json::from_slice(&fs::read(dataset.join("linked-metadata52.json")).unwrap()).unwrap();
    data.validate_dependencies(&jobs).unwrap();
    let manifest: gba_core::rom_data::Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            manifest.extractor_revision,
            manifest.decoder_revision,
            manifest.resources.len()
        ),
        (70, 69, 64)
    );
    assert_eq!(
        (
            gba_core::rom_data::EXTRACTOR_REV,
            gba_core::rom_data::DECODER_REV
        ),
        (70, 69)
    );
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    let (at, bytes, role) = &spans[0];
    assert_eq!(*at, SOURCE_OFFSET);
    assert_eq!(bytes.len(), 20);
    assert_eq!(*role, "slot-job-ids");
    assert_eq!(bytes, &rom[*at as usize..*at as usize + 20]);
    let source = &catalog.sources[0];
    assert_eq!((source.offset, source.length), (SOURCE_OFFSET, 20));
    assert_eq!(source.role, *role);
    assert_eq!(source.sha256, gba_core::ffta_rom::digest(bytes));
    assert!(manifest
        .resources
        .iter()
        .flat_map(|r| &r.sources)
        .all(|s| s.offset + s.length <= *at || *at + 20 <= s.offset));
    assert_eq!(data.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(data.byte_at(SOURCE_OFFSET + 20), None);
    for (i, &b) in bytes.iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE_OFFSET + i as u32), Some(b));
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<SlotJobLookup>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut invalid = data.clone();
        match case {
            0 => invalid.source_offset += 1,
            1 => {
                invalid.entries.pop();
            }
            2 => invalid.entries.swap(0, 1),
            3 => invalid.entries[1].slot = 0,
            _ => invalid.entries[0].job_id = 255,
        }
        assert!(invalid.validate().is_err());
        guards += 1;
    }
    for case in 0..3 {
        let mut invalid: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => invalid["extra"] = json!(true),
            1 => invalid["entries"][0]["extra"] = json!(true),
            _ => invalid["entries"][0]["job_id"] = json!(256),
        }
        assert!(serde_json::from_value::<SlotJobLookup>(invalid).is_err());
        guards += 1;
    }
    for case in 0..3 {
        let mut invalid = jobs.clone();
        match case {
            0 => invalid.source_offset += 1,
            1 => {
                invalid.records.pop();
            }
            _ => invalid.records[data.entries[0].job_id as usize].routing = 255,
        }
        assert!(data.validate_dependencies(&invalid).is_err());
        guards += 1;
    }
    let mut changed = data.clone();
    changed.entries[0].job_id = changed.entries[1].job_id;
    changed.validate_dependencies(&jobs).unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let mut m = Machine::new(rom);
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    let readers = [
        (0xd0538, None),
        (0xd0548, Some(1)),
        (0xd0568, Some(4)),
        (0xd0588, Some(6)),
        (0xd05a8, Some(9)),
        (0xd05c8, Some(0)),
    ];
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    let mut calls = 0;
    for e in &data.entries {
        for (entry, property) in readers {
            let value = property.map_or(u32::from(e.job_id), |p| {
                jobs.records[e.job_id as usize].packed_property(p).unwrap()
            });
            let value = if entry == 0xd05c8 { value } else { value & 255 };
            instructions += original(&mut m, entry, u32::from(e.slot), value, &mut covered);
            calls += 1;
        }
    }
    for (entry, input, property) in [
        (0xd0538, 256 + 19, None),
        (0xd0568, 0x80000000 | 19, Some(4)),
        (0xd05c8, 65536, Some(0)),
    ] {
        let job = data.entries[(input & 255) as usize].job_id;
        let value = property.map_or(u32::from(job), |p| {
            jobs.records[job as usize].packed_property(p).unwrap()
        });
        instructions += original(
            &mut m,
            entry,
            input,
            if entry == 0xd05c8 { value } else { value & 255 },
            &mut covered,
        );
        calls += 1;
    }
    assert_eq!(covered, (SOURCE_OFFSET..SOURCE_OFFSET + 20).collect());
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&json!({"entries":20,"source_bytes":20,"source_ranges":1,"original_read_bytes":covered.len(),"original_reader_paths":calls,"full_property0_identifier_prefixes":21,"property_wrappers_complete":81,"bare_job_getters":21,"high_input_aliases":3,"guards":guards,"strict_JSON_source_dependencies":true,"callee_fullRAM_outside_original20Bstack_VRAM_OAM_palette":true,"instructions_coverage_only":instructions,"all64_current_owners_disjoint":true,"production_revision_unchanged":[1,70,69],"USA_unchanged":[8,7],"native_clock_ordinary":"pending integration","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
