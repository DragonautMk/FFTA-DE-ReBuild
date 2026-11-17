//! EUR range lookup publication and original scan.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::range_lookup::{RECORD_COUNT, SOURCE, SOURCE_SHA256};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, e160 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_range_lookup_publication_and_original_scan() {
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (160, 159));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (161, 160));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|r| r.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    let entry = source.manifest.resources.iter().find(|r| r.id == RANGE_LOOKUP_ID).unwrap();
    assert_eq!((entry.sources.len(), entry.sources[0].offset, entry.sources[0].length, entry.sources[0].sha256.as_str()), (1, SOURCE, 88, SOURCE_SHA256));
    let model = &source.range_lookup.as_ref().unwrap().decoded;
    assert_eq!(model.records.len(), RECORD_COUNT);
    assert_eq!(model.source_bytes().unwrap(), rom[SOURCE as usize..SOURCE as usize + 88]);
    let mut bad = model.clone(); bad.records.pop(); assert!(bad.validate().is_err());
    let mut edited = model.clone(); edited.records[0].opaque_01 ^= 1; edited.validate().unwrap(); assert!(edited.verify_source(&rom).is_err());
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = SOURCE as usize; let end = start + 88;
    managed.bus.rom[start..end].fill(0xcd);
    let before = owner.range_lookup_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    assert_eq!(owner.range_lookup_stats().unwrap().source_bytes - before, 88);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0801_abc0..0x0801_abfa).step_by(2)
        .map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("range-lookup.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut values: Vec<u16> = model.records.iter().flat_map(|r| [r.lower, r.upper]).collect();
    if let Some(value) = (0..=u16::MAX).find(|v| model.select(*v).is_none()) { values.push(value); }
    let mut cases = 0;
    for native in [false, true] { for &value in &values {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[0] = u32::from(value);
            m.cpu.regs[1] = 0x0201_0000;
            m.cpu.regs[2] = 0x0201_0002;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0801_abc0;
            m.cpu.cpsr = 0xbf;
            m.bus.write8(0x0201_0000, 0xa6);
            m.bus.write16(0x0201_0002, 0xb7c8);
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.range_lookup_stats().unwrap().source_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x0800_0300 { assert!(steps < 300); raw.step(); steps += 1; }
        steps = 0;
        while managed.cpu.regs[15] != 0x0800_0300 {
            assert!(steps < 300, "scan pc {:08x}", managed.cpu.regs[15]);
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            steps += 1;
        }
        assert!(owner.range_lookup_stats().unwrap().source_bytes > before);
        if let Some((selector, offset)) = model.select(value) {
            assert_eq!(raw.bus.read8(0x0201_0000), selector);
            assert_eq!(raw.bus.read16(0x0201_0002), offset);
        } else {
            assert_eq!(raw.bus.read8(0x0201_0000), 0xa6);
            assert_eq!(raw.bus.read16(0x0201_0002), 0xb7c8);
        }
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "value {value}, native {native}");
        cases += 1;
    }}
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[161,160], "previous_resources_preserved":old.resources.len(),
        "records":11, "owned_bytes":88, "source_sha256":SOURCE_SHA256, "poisoned_raw_rom":true,
        "original_scan_cases":cases, "compiled_dispatch":"may use interpreter fallback", "full_state_clock_equal":true,
        "ordinary_use":"pending", "production_cli":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
