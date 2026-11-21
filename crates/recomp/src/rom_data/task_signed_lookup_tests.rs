//! EUR signed task lookup: physical projection and uncapped original indexing.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_task_parameters as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

const ENTRY: u32 = 0x0801_2834;
const STOP: u32 = 0x0801_284a;
const TASK: u32 = 0x0200_2000;
const INITIAL: u16 = 0xa234;

fn setup(machine: &mut Machine, index: u8) {
    machine.bus.ffta_sprites.enabled = false;
    let task = (TASK - 0x0200_0000) as usize;
    machine.bus.ewram[task + 0x40] = index;
    machine.bus.ewram[task + 0x36..task + 0x38].copy_from_slice(&INITIAL.to_le_bytes());
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[1] = TASK;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = ENTRY;
    machine.cpu.cpsr = 0xbf;
}

fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (start, length) in [(0x37101cusize, 3usize), (0x371020, 71), (0x37111f, 1)] {
        if altered {
            for at in start..start + length { machine.bus.rom[at] = !rom[at]; }
        } else {
            machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E205 predecessor; Wine only, Linux/USA pending"]
fn european_signed_task_lookup_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_TASK_SIGNED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TASK_SIGNED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (205, 204, 123));
    let old = parse_observed_task_parameters(&fs::read(previous.join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap()).unwrap();
    assert!(old.signed_lookup_projection.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (206, 205, 123));
    for prior in &old_manifest.resources {
        if prior.id == OBSERVED_TASK_PARAMETERS_ID { continue; }
        let current = source.manifest.resources.iter().find(|item| item.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let parameters = &source.observed_task_parameters.as_ref().unwrap().decoded;
    let mut without = parameters.clone();
    assert_eq!(without.signed_lookup_projection.take().unwrap(), [1, 0, -1, 0]);
    assert_eq!(without, old);
    let start = model::SIGNED_LOOKUP_SOURCE as usize;
    let end = start + model::SIGNED_LOOKUP_BYTES;
    assert_eq!(gba_core::ffta_rom::digest(&rom[start..end]), model::SIGNED_LOOKUP_SHA256);
    let current = source.manifest.resources.iter().find(|item| item.id == OBSERVED_TASK_PARAMETERS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|item| item.id == OBSERVED_TASK_PARAMETERS_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 1);
    assert_eq!((current.sources.last().unwrap().offset, current.sources.last().unwrap().length), (model::SIGNED_LOOKUP_SOURCE, 4));
    assert_eq!(parse_observed_task_parameters(&fs::read(prepared.directory.join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap()).unwrap(), *parameters);
    let mut forged = parameters.clone();
    forged.signed_lookup_projection.as_mut().unwrap()[2] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    backing(&mut managed, &rom, true);
    let before = owner.observed_task_parameters_stats().unwrap();
    for at in start..end { assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]); }
    let after = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(after.signed_lookup_bytes - before.signed_lookup_bytes, 4);
    assert_eq!(after.signed_lookup_indices_mask, 15);
    assert_eq!((after.phases_used, after.corrections_used), (before.phases_used, before.corrections_used));
    let geometry_before = owner.tilemap_geometry_stats().unwrap().eight_row_lookup_bytes;
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())), "read16 {at:x}");
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()), "read32 {at:x}");
            }
        }
    }
    assert!(owner.tilemap_geometry_stats().unwrap().eight_row_lookup_bytes > geometry_before);
    backing(&mut managed, &rom, false);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (ENTRY..STOP).step_by(2).map(|address| Block {
        addr: address, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(address), address)],
    }).collect();
    let object = root.join("signed-task-lookup-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let initial = raw.save_state().unwrap();
    let indices = [0u8, 1, 2, 3, 4, 19, 20, 67, 68, 70, 71, 255];
    let mut cases = 0;
    for native in [false, true] {
        for &index in &indices {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            for machine in [&mut raw, &mut managed] { setup(machine, index); }
            backing(&mut managed, &rom, true);
            let before = owner.observed_task_parameters_stats().unwrap().signed_lookup_bytes;
            for _ in 0..100 {
                if raw.cpu.regs[15] == STOP { break; }
                raw.bus.frame_ready = false;
                raw.step();
            }
            for _ in 0..100 {
                if managed.cpu.regs[15] == STOP { break; }
                managed.bus.frame_ready = false;
                if native {
                    table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                } else { managed.step(); }
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (STOP, STOP));
            assert_eq!(owner.observed_task_parameters_stats().unwrap().signed_lookup_bytes - before, u64::from(index < 4));
            let task = (TASK - 0x0200_0000) as usize;
            let result = u16::from_le_bytes(raw.bus.ewram[task + 0x36..task + 0x38].try_into().unwrap());
            let signed = rom[start + usize::from(index)] as i8 as i16 as u16;
            assert_eq!(result, INITIAL.wrapping_add(signed), "original signed value for index {index}");
            backing(&mut managed, &rom, false);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "index {index}, native {native}");
            cases += 1;
        }
    }
    let snapshot = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&snapshot).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[206,205], "projection_bytes":4, "prior_resources_preserved":122,
        "all_source_bytes_and_mixed_boundaries":true, "original_interpreter_native_cases":cases,
        "uncapped_indices_through_255":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
