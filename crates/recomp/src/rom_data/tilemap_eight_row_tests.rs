//! EUR eight-row tile lookup: publication, mixed ownership and original drawing.
use super::*;
use gba_core::{capi::RT_API, rom_data::tilemap_geometry as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const ENTRY: u32 = 0x0801_7708;
const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0202_0000;
const STACK: u32 = 0x0300_7c00;

fn setup(machine: &mut Machine, width: u32, height: u32, flags: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram[0x20000..0x20200].fill(0x77);
    for (i, value) in [16u32, 0x100, 0x200, 0].into_iter().enumerate() {
        let at = (STACK - 0x0300_0000) as usize + i * 4;
        machine.bus.iwram[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = DESTINATION;
    machine.cpu.regs[1] = width;
    machine.cpu.regs[2] = height;
    machine.cpu.regs[3] = flags;
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = ENTRY;
    machine.cpu.cpsr = 0xbf;
}

fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    let start = model::EIGHT_ROW_LOOKUP_SOURCE as usize;
    let end = start + model::EIGHT_ROW_LOOKUP_BYTES;
    if altered {
        for at in start..end { machine.bus.rom[at] = !rom[at]; }
    } else {
        machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    }
}

fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..200_000 {
        if machine.cpu.regs[15] == RETURN { return; }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original tilemap drawing did not return at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E205 dataset; Wine only, Linux/USA pending"]
fn european_eight_row_lookup_preserves_legacy_group_counter() {
    let root = PathBuf::from(std::env::var_os("FFTA_EIGHT_ROW_COUNTER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_EIGHT_ROW_COUNTER_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let lookup = model::EIGHT_ROW_LOOKUP_SOURCE as usize;
    for at in lookup..lookup + model::EIGHT_ROW_LOOKUP_BYTES { managed.bus.rom[at] = !rom[at]; }
    for at in lookup..lookup + model::EIGHT_ROW_LOOKUP_BYTES {
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]);
    }
    let after_lookup = owner.tilemap_geometry_stats().unwrap();
    assert_eq!(after_lookup.groups_used, 0);
    assert_eq!(after_lookup.eight_row_lookup_rows, 4);
    assert_eq!(after_lookup.eight_row_lookup_bytes, 16);
    for group in 0..6 {
        let at = model::SOURCE as usize + group * 8;
        managed.bus.rom[at] = !rom[at];
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]);
        assert_eq!(owner.tilemap_geometry_stats().unwrap().groups_used, (group + 1) as u64);
    }
    let after_groups = owner.tilemap_geometry_stats().unwrap();
    assert_eq!((after_groups.groups_used, after_groups.eight_row_lookup_rows, after_groups.eight_row_lookup_bytes), (6, 4, 16));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"E205 EUR Wine: old six-group counter independent of four lookup rows",
        "legacy_groups":6, "lookup_rows":4, "lookup_bytes":16,
        "Linux":"pending", "USA":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E204 predecessor; Wine only, Linux/USA pending"]
fn european_eight_row_tilemap_lookup_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_EIGHT_ROW_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_EIGHT_ROW_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (204, 203, 123));
    let old = parse_tilemap_geometry(&fs::read(previous.join(TILEMAP_GEOMETRY_FILE)).unwrap()).unwrap();
    assert!(old.eight_row_tile_offsets.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (205, 204, 123));
    for prior in &old_manifest.resources {
        if prior.id == TILEMAP_GEOMETRY_ID { continue; }
        let current = source.manifest.resources.iter().find(|item| item.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let geometry = &source.tilemap_geometry.as_ref().unwrap().decoded;
    let mut without = geometry.clone();
    let values = without.eight_row_tile_offsets.take().unwrap();
    assert_eq!(without, old);
    assert_eq!(values, [[-2, -2, 1, 0], [-2, 4, 3, 2], [6, 5, -1, -1], [7, -1, -1, -1]]);
    let start = model::EIGHT_ROW_LOOKUP_SOURCE as usize;
    let end = start + model::EIGHT_ROW_LOOKUP_BYTES;
    assert_eq!(geometry.eight_row_lookup_bytes().unwrap(), rom[start..end]);
    assert_eq!(gba_core::ffta_rom::digest(&rom[start..end]), model::EIGHT_ROW_LOOKUP_SHA256);
    let current = source.manifest.resources.iter().find(|item| item.id == TILEMAP_GEOMETRY_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|item| item.id == TILEMAP_GEOMETRY_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 1);
    assert_eq!((current.sources.last().unwrap().offset, current.sources.last().unwrap().length), (model::EIGHT_ROW_LOOKUP_SOURCE, 16));
    assert_eq!(parse_tilemap_geometry(&fs::read(prepared.directory.join(TILEMAP_GEOMETRY_FILE)).unwrap()).unwrap(), *geometry);
    let mut malformed = geometry.clone();
    malformed.state.source_offset += 2;
    assert!(malformed.validate().is_err());
    let mut forged = geometry.clone();
    forged.eight_row_tile_offsets.as_mut().unwrap()[0][0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    backing(&mut managed, &rom, true);
    let before = owner.tilemap_geometry_stats().unwrap();
    for at in start..end {
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at], "source byte {at:x}");
    }
    let after = owner.tilemap_geometry_stats().unwrap();
    assert_eq!(after.eight_row_lookup_bytes - before.eight_row_lookup_bytes, 16);
    assert_eq!(after.eight_row_lookup_rows, 4);
    for at in end..end + 4 { managed.bus.rom[at] = !rom[at]; }
    let previous_before = owner.task_cycle_deltas_stats().unwrap().source_bytes;
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
    assert!(owner.task_cycle_deltas_stats().unwrap().source_bytes > previous_before);
    managed.bus.rom[end..end + 4].copy_from_slice(&rom[end..end + 4]);
    backing(&mut managed, &rom, false);

    let initial = raw.save_state().unwrap();
    let cases = [(4u32, 8u32, 0u32), (8, 8, 0), (4, 8, 16), (8, 8, 16), (8, 7, 0)];
    let mut states = Vec::new();
    let mut visited = BTreeSet::new();
    for &(width, height, flags) in &cases {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for machine in [&mut raw, &mut managed] { setup(machine, width, height, flags); }
        backing(&mut managed, &rom, true);
        let before = owner.tilemap_geometry_stats().unwrap().eight_row_lookup_bytes;
        interpret(&mut raw, &mut visited);
        interpret(&mut managed, &mut BTreeSet::new());
        let used = owner.tilemap_geometry_stats().unwrap().eight_row_lookup_bytes - before;
        if height == 8 { assert!(used > 0, "original drawing must read the lookup"); }
        else { assert_eq!(used, 0, "non-eight-row path must not read the lookup"); }
        backing(&mut managed, &rom, false);
        let state = raw.save_state().unwrap();
        assert_eq!(managed.save_state().unwrap(), state, "interpreter width {width} height {height} flags {flags}");
        states.push(state);
    }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let mut instrs = vec![if thumb { armv4t::decode_thumb(view.read16(address), address) } else { armv4t::decode_arm(view.read32(address), address) }];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("eight-row-tilemap-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (&(width, height, flags), expected) in cases.iter().zip(&states) {
        managed.load_state(&initial).unwrap();
        setup(&mut managed, width, height, flags);
        backing(&mut managed, &rom, true);
        for _ in 0..200_000 {
            if managed.cpu.regs[15] == RETURN { break; }
            managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb())).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        }
        assert_eq!(managed.cpu.regs[15], RETURN);
        backing(&mut managed, &rom, false);
        assert_eq!(&managed.save_state().unwrap(), expected, "native width {width} height {height} flags {flags}");
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
        "status":"PASS", "revision":[205,204], "rows":4, "source_bytes":16,
        "prior_resources_preserved":122, "all_source_bytes_and_mixed_boundaries":true,
        "original_interpreter_native_cases":cases.len(), "wrong_height_uses_no_lookup":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
