//! EUR tilemap geometry publication and original-reader checks.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::tilemap_geometry::{SOURCE, SOURCE_BYTES, SOURCE_SHA256};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, e161 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_tilemap_geometry_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_GEOMETRY_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_GEOMETRY_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (161, 160));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (163, 162));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|r| r.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 2);
    let entry = source.manifest.resources.iter().find(|r| r.id == TILEMAP_GEOMETRY_ID).unwrap();
    assert_eq!(entry.sources.len(), 6);
    for (index, span) in entry.sources.iter().enumerate() {
        assert_eq!((span.offset, span.length), (SOURCE + index as u32 * 8, 8));
    }
    let model = &source.tilemap_geometry.as_ref().unwrap().decoded;
    let start = SOURCE as usize;
    let end = start + SOURCE_BYTES;
    assert_eq!(model.source_bytes().unwrap(), rom[start..end]);
    assert_eq!(gba_core::ffta_rom::digest(&rom[start..end]), SOURCE_SHA256);
    let mut malformed = model.clone();
    malformed.state.source_offset += 2;
    assert!(malformed.validate().is_err());
    let mut edited = model.clone();
    edited.state.width_pixels[0] ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut json = serde_json::to_value(model).unwrap();
    json["constructor"]["unexpected"] = true.into();
    assert!(serde_json::from_value::<TilemapGeometry>(json).is_err());

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    managed.bus.rom[start..end].fill(0xcd);
    let before = owner.tilemap_geometry_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    assert_eq!(owner.tilemap_geometry_stats().unwrap().source_bytes - before, SOURCE_BYTES as u64);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = [(0x0801_c7a0, 0x0801_c7d6), (0x0801_f208, 0x0801_f23c)]
        .into_iter().flat_map(|(start, end)| (start..end).step_by(2))
        .map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("tilemap-geometry.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for &(entry, stop) in &[(0x0801_c7a0, 0x0801_c7d6), (0x0801_f208, 0x0800_0300)] {
            for index in [0u32, 1, 2, 3, 4, 255] {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[0] = if entry == 0x0801_f208 { index } else { 3 };
                    machine.cpu.regs[1] = 4;
                    machine.cpu.regs[2] = index;
                    machine.cpu.regs[3] = 0;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[14] = 0x0800_0301;
                    machine.cpu.regs[15] = entry;
                    machine.cpu.cpsr = 0xbf;
                }
                managed.bus.rom[start..end].fill(0xcd);
                let before = owner.tilemap_geometry_stats().unwrap().source_bytes;
                let mut steps = 0;
                while raw.cpu.regs[15] != stop { assert!(steps < 200); raw.step(); steps += 1; }
                steps = 0;
                while managed.cpu.regs[15] != stop {
                    assert!(steps < 200, "reader pc {:08x}", managed.cpu.regs[15]);
                    if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
                    else { managed.step(); }
                    steps += 1;
                }
                let served = owner.tilemap_geometry_stats().unwrap().source_bytes - before;
                if index <= 3 { assert_eq!(served, 6); }
                if index == 255 { assert_eq!(served, 0); }
                managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "entry {entry:08x}, index {index}, native {native}");
                cases += 1;
            }
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[163,162], "previous_resources_preserved":old.resources.len(),
        "groups":6, "owned_bytes":SOURCE_BYTES, "source_sha256":SOURCE_SHA256, "poisoned_raw_rom":true,
        "original_reader_cases":cases, "compiled_dispatch":"may use interpreter fallback", "full_state_clock_equal":true,
        "ordinary_use":"pending", "production_cli":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
