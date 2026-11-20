//! EUR alternate tile rectangles, selector and original branch/readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

fn set_backing(machine: &mut Machine, rom: &[u8], invert: bool) {
    for &(offset, height, width, _, _) in &model::ALTERNATE_RECT_SPECS {
        let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
        for at in start..start + length { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
    let start = model::ALTERNATE_SELECTOR_OFFSET as usize;
    for at in start..start + model::ALTERNATE_SELECTOR_LENGTH { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
}

#[test]
#[ignore = "exact EUR ROM, E194 predecessor and Wine; Linux/USA pending"]
fn european_tilemap_alternates_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ALTERNATES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ALTERNATES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (194, 193, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (195, 194, 123));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 3);
    let rectangles = data.alternate_rectangles.as_ref().unwrap();
    for rectangle in rectangles {
        let start = rectangle.source_offset as usize; let bytes = rectangle.source_bytes();
        assert_eq!(bytes, &rom[start..start + bytes.len()]);
    }
    let selector = data.alternate_selector.unwrap();
    assert_eq!(selector, rom[model::ALTERNATE_SELECTOR_OFFSET as usize..model::ALTERNATE_SELECTOR_OFFSET as usize + model::ALTERNATE_SELECTOR_LENGTH]);
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.alternate_rectangles.as_mut().unwrap()[0].width ^= 1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.alternate_selector = None; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.alternate_rectangles.as_mut().unwrap()[0].cells[0] ^= 1;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = [0x080d811c, 0x08109528, 0x0810950a, 0x0810950c, 0x0810950e, 0x08109510, 0x08109512, 0x08109520];
    let blocks = addresses.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("tilemap-alternates-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    set_backing(&mut managed, &rom, true);
    let mut spans: Vec<(usize, usize)> = model::ALTERNATE_RECT_SPECS.iter().map(|&(offset, height, width, _, _)| {
        let start = offset as usize; (start, start + 2 + usize::from(height) * usize::from(width) * 2)
    }).collect();
    spans.push((model::ALTERNATE_SELECTOR_OFFSET as usize, model::ALTERNATE_SELECTOR_OFFSET as usize + model::ALTERNATE_SELECTOR_LENGTH));
    for &(start, end) in &spans {
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
            }
        }
    }
    set_backing(&mut managed, &rom, false); let initial = raw.save_state().unwrap();
    for (index, rectangle) in rectangles.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[7] = 0x08000000 + rectangle.source_offset; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = 0x080d811c; machine.cpu.cpsr = 0xbf;
        }
        set_backing(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().alternate_rectangle_bytes[index];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(0x080d811d).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.observed_tilemap_stats().unwrap().alternate_rectangle_bytes[index] - before, 1);
        assert_eq!(raw.cpu.regs[0], u32::from(rectangle.width));
        set_backing(&mut managed, &rom, false); assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
        machine.cpu.regs[1] = 0x08000000 + model::ALTERNATE_SELECTOR_OFFSET;
        machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[15] = 0x08109528; machine.cpu.cpsr = 0xbf;
    }
    set_backing(&mut managed, &rom, true); let before = owner.observed_tilemap_stats().unwrap().alternate_selector_bytes;
    raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
    table.get(0x08109529).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
    assert_eq!(owner.observed_tilemap_stats().unwrap().alternate_selector_bytes - before, 1);
    assert_eq!(raw.cpu.regs[3], u32::from(selector[0]));
    set_backing(&mut managed, &rom, false); assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());

    // Exercise both original branch outcomes without replacing the guest selector.
    for (index, expected) in [(0u8, rectangles[0].source_offset), (6, rectangles[1].source_offset)] {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write8(0x02018008, index); machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[6] = 0x02018000; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = 0x0810950a; machine.cpu.cpsr = 0xbf;
        }
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x08109522 { assert!(steps < 16); raw.bus.frame_ready = false; raw.step(); steps += 1; }
        let mut steps = 0;
        while managed.cpu.regs[15] != 0x08109522 { assert!(steps < 16); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1; }
        assert_eq!(raw.cpu.regs[0], 0x08000000 + expected);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[195,194], "resources":123, "preserved_other_resources":122,
        "previous_tilemap_preserved":true, "rectangles":2, "selector_bytes":8, "new_source_bytes":126,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":3, "original_branch_cases":2, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
