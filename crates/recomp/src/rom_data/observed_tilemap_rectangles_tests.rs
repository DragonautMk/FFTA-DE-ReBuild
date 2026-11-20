//! EUR tile rectangles with their original width reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

fn set_rect_backing(machine: &mut Machine, rom: &[u8], invert: bool) {
    for &(offset, height, width, _, _) in &model::RECT_SPECS {
        let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
        for at in start..start + length { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM, E182 predecessor and Wine; Linux/USA pending"]
fn european_tile_rectangles_publication_and_native_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_RECTANGLES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RECTANGLES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (182, 181, 120));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (183, 182, 120));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    assert_eq!(&source.manifest.resources[117].sources[..1], &old.resources[117].sources);
    assert_eq!(source.manifest.resources[117].sources.len(), 3);
    for (offset, bytes, _) in data.source_spans() { let at = offset as usize; assert_eq!(bytes, &rom[at..at + bytes.len()]); }
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.rectangles[0].width ^= 1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.rectangles[0].cells[0] ^= 1;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone(); altered_manifest.resources[117].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    drop(dataset);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x080d8136..0x080d8142).step_by(2).map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("observed-rectangles-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    set_rect_backing(&mut managed, &rom, true);
    for &(offset, height, width, _, _) in &model::RECT_SPECS {
        let start = offset as usize; let end = start + 2 + usize::from(height) * usize::from(width) * 2;
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
            }
        }
    }
    set_rect_backing(&mut managed, &rom, false);
    let initial = raw.save_state().unwrap();
    for (index, rect) in data.rectangles.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[5] = 0x08000000 + rect.source_offset + 2;
            machine.cpu.regs[6] = 0x03003862; machine.cpu.regs[7] = 0x08000000 + rect.source_offset;
            machine.cpu.regs[9] = 0x08370eb4; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = 0x080d8136; machine.cpu.cpsr = 0xbf;
        }
        set_rect_backing(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().rectangle_bytes[index];
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x080d8142 { assert!(steps < 16); raw.bus.frame_ready = false; raw.step(); steps += 1; }
        let mut steps = 0;
        while managed.cpu.regs[15] != 0x080d8142 {
            assert!(steps < 16); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        assert_eq!(owner.observed_tilemap_stats().unwrap().rectangle_bytes[index] - before, 1);
        assert_eq!(raw.cpu.regs[2], u32::from(rect.width) * 2);
        assert_eq!(raw.cpu.regs[1], 0x08000000 + rect.source_offset + 2);
        set_rect_backing(&mut managed, &rom, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[183,182], "resources":120, "preserved_other_resources":119,
        "previous_tilemap_preserved":true, "rectangles":2, "source_bytes":88, "cells":42,
        "poisoned_mirrored_and_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_width_reader_cases":2, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
