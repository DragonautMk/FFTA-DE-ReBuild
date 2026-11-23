//! EUR White Flowers conditional queued halfword sources and original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const NODE: u32 = 0x0203_0000;
const CONTEXT: u32 = 0x0201_0000;
const DESTINATION: u32 = 0x0601_018a;
fn spans() -> [(u32, usize); 3] {
    [(model::WHITE_FLOWERS_QUEUED_LITERALS, 12),
     (model::WHITE_FLOWERS_QUEUED_OFFSETS[0], model::WHITE_FLOWERS_QUEUED_LENGTH),
     (model::WHITE_FLOWERS_QUEUED_OFFSETS[1], model::WHITE_FLOWERS_QUEUED_LENGTH)]
}
fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (start, length) in spans() {
        let at = start as usize;
        if altered { for i in at..at + length { machine.bus.rom[i] = !rom[i]; } }
        else { machine.bus.rom[at..at + length].copy_from_slice(&rom[at..at + length]); }
    }
}
fn setup(machine: &mut Machine, flag: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.write8(CONTEXT + 4, flag);
    machine.bus.write32(0x0300_09a4, 0x0601_0000);
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[3] = NODE;
    machine.cpu.regs[5] = CONTEXT;
    machine.cpu.regs[13] = 0x0300_7d00;
    machine.cpu.regs[15] = 0x0812_bf10;
    machine.cpu.cpsr = 0xbf;
}
fn interpret(machine: &mut Machine, stop: u32) {
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == stop { return; }
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original queue path escaped at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E276 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_queued_rows_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_QUEUED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_QUEUED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (276, 275, 146));
    let old = parse_observed_tilemap(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    assert!(old.white_flowers_queued.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (277, 276, 146));
    for prior in &old_manifest.resources {
        if prior.id == OBSERVED_TILEMAP_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let mut without = data.clone();
    let selected = without.white_flowers_queued.take().unwrap();
    assert_eq!(without, old);
    let current = source.manifest.resources.iter().find(|r| r.id == OBSERVED_TILEMAP_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|r| r.id == OBSERVED_TILEMAP_ID).unwrap();
    assert_eq!(current.sources.len(), prior.sources.len() + 3);
    assert!(prior.sources.iter().all(|entry| current.sources.contains(entry)));
    for (i, (offset, length)) in spans().iter().copied().enumerate() {
        let entry = current.sources.iter().find(|s| s.offset == offset && s.length as usize == length).unwrap();
        assert_eq!(entry.sha256, model::WHITE_FLOWERS_QUEUED_HASHES[i]);
    }
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone(); invalid.white_flowers_queued.as_mut().unwrap().halfwords[0].pop(); assert!(invalid.validate().is_err());
    let mut forged = data.clone(); forged.white_flowers_queued.as_mut().unwrap().halfwords[0][0] ^= 1; forged.validate().unwrap();
    let bytes = encode_observed_tilemap(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed = dataset.manifest.clone(); changed.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &changed; files.observed_tilemap = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err()); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    backing(&mut managed, &rom, true);
    let before = owner.observed_tilemap_stats().unwrap();
    for (start, length) in spans() {
        for at in start as usize..start as usize + length {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at], "source byte {at:x}");
            }
        }
    }
    let after = owner.observed_tilemap_stats().unwrap();
    for (i, (_, length)) in spans().iter().enumerate() { assert_eq!(after.white_flowers_queued_bytes[i] - before.white_flowers_queued_bytes[i], (*length * 3) as u64); }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|e| &e.sources).any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (start, length) in spans() {
        for edge in [start as usize, start as usize + length] {
            for at in edge.saturating_sub(3)..edge + 3 { if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; } }
        }
    }
    for (start, length) in spans() {
        for edge in [start as usize, start as usize + length] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
                assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
            }
        }
    }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    backing(&mut managed, &rom, false);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0812_bf10..0x0812_bf1c).step_by(2).map(|addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("white-flowers-queued-selector-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for flag in [0u8, 1, 255] {
        for native in [false, true] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, flag); }
            backing(&mut managed, &rom, true);
            interpret(&mut raw, 0x0812_bf1c);
            if native {
                for _ in 0..10 {
                    if managed.cpu.regs[15] == 0x0812_bf1c { break; }
                    managed.bus.frame_ready = false;
                    table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                }
                assert_eq!(managed.cpu.regs[15], 0x0812_bf1c);
            } else { interpret(&mut managed, 0x0812_bf1c); }
            let which = usize::from(flag != 0);
            assert_eq!(raw.cpu.regs[1], 0x0800_0000 + model::WHITE_FLOWERS_QUEUED_OFFSETS[which]);
            assert_eq!(raw.cpu.regs[0], 0x0300_0990);
            backing(&mut managed, &rom, false);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "selector flag {flag} native {native}");
        }
    }
    // Execute the original selector, registration helper and queued DMA executor.
    for flag in [0u8, 1, 255] {
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, flag); }
        backing(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().white_flowers_queued_bytes;
        for machine in [&mut raw, &mut managed] {
            interpret(machine, 0x0812_bf38);
            let which = usize::from(flag != 0);
            assert_eq!(machine.bus.read32(0x0300_2bcc), NODE);
            assert_eq!(machine.bus.read32(NODE + 4), 0x0800_0000 + model::WHITE_FLOWERS_QUEUED_OFFSETS[which]);
            assert_eq!(machine.bus.read32(NODE + 8), DESTINATION);
            assert_eq!(machine.bus.read32(NODE + 12), 0x8000_0014);
            assert_eq!(machine.bus.read16(NODE + 16), 8);
            assert_eq!(machine.bus.read16(NODE + 18), 40);
            assert_eq!(machine.bus.read16(NODE + 20), 64);
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[13] = 0x0300_7d00;
            machine.cpu.regs[14] = 0x0300_7f01;
            machine.cpu.regs[15] = 0x0800_0800;
            machine.cpu.cpsr = 0xbf;
            interpret(machine, 0x0300_7f00);
            assert_eq!(machine.bus.read32(0x0300_2bcc), 0);
            for row in 0..8 {
                let start = (DESTINATION - 0x0600_0000) as usize + row * 64;
                let cells = &selected.halfwords[which][row * 20..row * 20 + 20];
                let expected: Vec<u8> = cells.iter().flat_map(|v| v.to_le_bytes()).collect();
                assert_eq!(&machine.bus.vram[start..start + 40], expected);
            }
        }
        let after = owner.observed_tilemap_stats().unwrap().white_flowers_queued_bytes;
        assert!(after[0] > before[0] && after[1 + usize::from(flag != 0)] >= before[1 + usize::from(flag != 0)] + 320);
        backing(&mut managed, &rom, false);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "queued transfer flag {flag}");
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[277,276], "resources":146, "new_source_bytes":652,
        "rows_per_projection":8, "halfwords_per_row":20, "prior_other_resources_unchanged":145,
        "prior_representation_preserved":true, "source_mirrors_mixed_boundaries":true,
        "original_interpreter_generated_selector_cases":6, "original_selector_queue_dma_cases":3,
        "both_projections_and_nonzero_selector_verified":true, "coherent_edit_rejected":true, "snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
