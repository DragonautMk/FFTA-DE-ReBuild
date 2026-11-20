//! EUR template publication and original fixed-size copy.
use super::*;
use gba_core::{capi::RT_API, rom_data::constructor_template64 as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E175 predecessor and Wine; Linux/USA pending"]
fn european_constructor_template64_publication_and_native_copy() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONSTRUCTOR64_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONSTRUCTOR64_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (175, 174, 116));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (176, 175, 117));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.constructor_template64.as_ref().unwrap().decoded;
    assert_eq!(data.source_bytes(), &rom[model::SOURCE_OFFSET as usize..model::SOURCE_OFFSET as usize + model::SOURCE_LENGTH]);
    assert_eq!(parse_constructor_template64(&fs::read(prepared.directory.join(CONSTRUCTOR_TEMPLATE64_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.source_offset += 4;
    assert!(invalid.validate().is_err());
    let mut bad_rom = rom.clone();
    bad_rom[model::SOURCE_OFFSET as usize] ^= 1;
    assert!(model::Catalog::from_rom(&bad_rom, &old.resources).is_err());
    assert!(model::Catalog::from_rom(&rom, &source.manifest.resources).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    raw.bus.iwram[0x5ee8..0x5f28].copy_from_slice(&rom[0xd968a8..0xd968e8]);
    let mut visited: BTreeSet<u32> = (0x08010aac..0x08010ab8).step_by(2).collect();
    visited.extend((0x03005ee8..0x03005f28).step_by(2));
    visited.insert(0x08145c14);
    let view = View { rom: &rom, ewram: None, iwram: Some(&raw.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("constructor-template64-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let baseline = raw.save_state().unwrap();
    let start = model::SOURCE_OFFSET as usize;
    let end = start + model::SOURCE_LENGTH;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.constructor_template64_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    assert_eq!(owner.constructor_template64_stats().unwrap().source_bytes - before, 64);
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&baseline).unwrap();
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
        machine.cpu.regs[4] = 0x02010000;
        machine.cpu.regs[6] = 0x08370eb4;
        machine.cpu.regs[8] = 0x08000000 + model::SOURCE_OFFSET;
        machine.cpu.regs[13] = 0x03007c00;
        machine.cpu.regs[14] = 0x03007ff1;
        machine.cpu.regs[15] = 0x08010aac;
        machine.cpu.cpsr = 0xbf;
    }
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.constructor_template64_stats().unwrap().source_bytes;
    let mut steps = 0;
    while raw.cpu.regs[15] != 0x08010ab8 {
        assert!(steps < 256, "original copy did not return");
        raw.bus.frame_ready = false;
        raw.step();
        steps += 1;
    }
    let mut steps = 0;
    while managed.cpu.regs[15] != 0x08010ab8 {
        assert!(steps < 256, "native copy did not return");
        managed.bus.frame_ready = false;
        table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        steps += 1;
    }
    assert_eq!(owner.constructor_template64_stats().unwrap().source_bytes - before, 64);
    assert_eq!(&raw.bus.ewram[0x10000..0x10040], data.source_bytes());
    assert_eq!(&managed.bus.ewram[0x10000..0x10040], data.source_bytes());
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[176,175], "resources":117, "preserved_resources":116,
        "source_bytes":64, "poisoned_mirrored_and_boundary_reads":true,
        "original_interpreter_and_native_copy_full_state_clock_equal":true,
        "ordinary_use":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
