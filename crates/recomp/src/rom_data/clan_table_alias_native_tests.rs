//! Translated original selector prefixes against the managed EUR clan alias.
use super::*;
use gba_core::{capi::RT_API, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "private EUR ROM, dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_clan_alias_original_native_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_CLAN_ALIAS_NATIVE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (95, 94));
    let prefixes = [(0x23be6, 0x23bee, 4, 1), (0x32768, 0x32770, 0, 1),
        (0x4fc04, 0x4fc0c, 2, 1), (0x5e734, 0x5e73c, 0, 1),
        (0x70f26, 0x70f2e, 5, 6), (0x70f64, 0x70f6c, 5, 6)];
    let mut visited = BTreeSet::new();
    for (start, end, _, _) in prefixes {
        for at in (start..end).step_by(2) { visited.insert(0x08000000 + at); }
    }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("clan-alias-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    let initial = raw.save_state().unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut served = 0;
    for (start, end, address_reg, table_reg) in prefixes {
        for language in [0u8, 4, 5, 255] {
            for m in [&mut raw, &mut native] {
                m.load_state(&initial).unwrap();
                m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                m.cpu.regs[address_reg] = 0x02001f6e;
                m.cpu.regs[table_reg] = 0x0871e5e0;
                m.cpu.regs[15] = 0x08000000 + start;
                m.cpu.cpsr = 0xbf;
                m.bus.ewram[0x1f6e] = language;
            }
            native.bus.rom[0x71e5e0..0x71e5f4].fill(0xcd);
            while raw.cpu.regs[15] != 0x08000000 + end { raw.step(); }
            let before = owner.ui_graphics_stats().unwrap().selector_bytes;
            let mut dispatches = 0;
            while native.cpu.regs[15] != 0x08000000 + end {
                assert!(dispatches < 8);
                let pc = native.cpu.regs[15];
                table.get(pc | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
                dispatches += 1;
            }
            let used = owner.ui_graphics_stats().unwrap().selector_bytes - before;
            if language < 5 { assert_eq!(used, 4); served += used; }
            native.bus.rom.copy_from_slice(&rom);
            assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap(), "native prefix {start:06x}, language {language}");
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","prefixes":6,"languages":[0,4,5,255],"cases":24,"typed_alias_bytes_served":served,"full_state_clock":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
