//! EUR tilemap row publication and original indexed pointer reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap_row as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E178 predecessor and Wine; Linux/USA pending"]
fn european_observed_tilemap_row_publication_and_native_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROW_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROW_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (178, 177, 119));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (179, 178, 120));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let start = model::SOURCE_OFFSET as usize;
    let end = start + model::SOURCE_LENGTH;
    assert_eq!(data.source_bytes(), &rom[start..end]);
    assert_eq!(parse_observed_tilemap_row(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap(), *data);
    data.validate_dependencies(&source.observed_tilemap_descriptor.as_ref().unwrap().decoded).unwrap();
    let mut bad = data.clone(); bad.descriptor_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.field8 ^= 1;
    let bad_bytes = encode_observed_tilemap_row(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources[119].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.observed_tilemap_row = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    assert!(model::Catalog::from_rom(&rom, &source.manifest.resources).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let visited: BTreeSet<u32> = (0x080db30a..0x080db318).step_by(2).collect();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("observed-tilemap-row-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.observed_tilemap_row_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    assert_eq!(owner.observed_tilemap_row_stats().unwrap().source_bytes - before, model::SOURCE_LENGTH as u64);
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        machine.bus.ffta_sprites.enabled = false;
        machine.bus.write32(0x02018000, 0x08000000 + model::SOURCE_OFFSET - 12);
        machine.bus.write8(0x02018007, 1);
        machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
        machine.cpu.regs[4] = 0x02018000;
        machine.cpu.regs[13] = 0x03007c00;
        machine.cpu.regs[15] = 0x080db30a;
        machine.cpu.cpsr = 0xbf;
    }
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.observed_tilemap_row_stats().unwrap().source_bytes;
    let mut steps = 0;
    while raw.cpu.regs[15] != 0x080db318 {
        assert!(steps < 12, "original row reader did not reach the descriptor load");
        raw.bus.frame_ready = false; raw.step(); steps += 1;
    }
    let mut steps = 0;
    while managed.cpu.regs[15] != 0x080db318 {
        assert!(steps < 12, "native row reader did not reach the descriptor load");
        managed.bus.frame_ready = false;
        table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        steps += 1;
    }
    assert_eq!(owner.observed_tilemap_row_stats().unwrap().source_bytes - before, 4);
    assert_eq!(raw.cpu.regs[0], 0x08000000 + model::SOURCE_OFFSET);
    assert_eq!(raw.cpu.regs[3], data.descriptor_pointer);
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[179,178], "resources":120, "preserved_resources":119,
        "source_bytes":12, "poisoned_mirrored_and_boundary_reads":true,
        "coherent_edit_rejected":true, "original_interpreter_and_native_reader_full_state_clock_equal":true,
        "native_reader_source_bytes":4, "ordinary_use":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
