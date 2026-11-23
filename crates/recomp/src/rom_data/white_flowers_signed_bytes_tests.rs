//! EUR White Flowers signed-byte projection and original cursor reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::white_flowers_signed_bytes as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, index: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[4] = 0x0203_0100;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0808_3bf2;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0203_0112, index);
    machine.bus.write16(0x0203_0108, 0x1234);
}

#[test]
#[ignore = "exact EUR ROM and E289 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_signed_bytes_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_SIGNED_BYTES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_SIGNED_BYTES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (289, 288, 146));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (290, 289, 147));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let new = source.manifest.resources.iter().find(|entry| entry.id == WHITE_FLOWERS_SIGNED_BYTES_ID).unwrap();
    assert_eq!((&new.path, &new.format), (&WHITE_FLOWERS_SIGNED_BYTES_FILE.to_string(), &gba_core::rom_data::WHITE_FLOWERS_SIGNED_BYTES_FORMAT.to_string()));
    assert_eq!(new.dependencies, vec![THUMB_PC_LITERALS_ID.to_string()]);
    assert_eq!(new.sources.len(), 1);
    assert_eq!((new.sources[0].offset, new.sources[0].length), (model::SOURCE_OFFSET, model::SOURCE_LENGTH as u32));
    let data = &source.white_flowers_signed_bytes.as_ref().unwrap().decoded;
    let start = model::SOURCE_OFFSET as usize;
    let end = start + model::SOURCE_LENGTH;
    assert_eq!(data.source_bytes().unwrap(), rom[start..end]);
    assert_eq!(parse_white_flowers_signed_bytes(&fs::read(prepared.directory.join(WHITE_FLOWERS_SIGNED_BYTES_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.values.pop();
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.values[20] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_white_flowers_signed_bytes(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == WHITE_FLOWERS_SIGNED_BYTES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.white_flowers_signed_bytes = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.white_flowers_signed_bytes_stats().unwrap();
    for at in start..end {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
        }
    }
    let after = owner.white_flowers_signed_bytes_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 144);
    assert_eq!(after.values_mask, (1u64 << 48) - 1);
    let current_owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for at in (start - 3..start).chain(end..end + 4) {
        if !current_owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    }
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at+n)) { rom[at+n] ^ 0x5a } else { rom[at+n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])), "read16 {at:x}");
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]), "read32 {at:x}");
            }
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    for at in changed { managed.bus.rom[at] = rom[at]; }

    // Translate the original reader; its original normalization helper stays interpreted.
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = (0x0808_3bf2..0x0808_3c12).step_by(2).chain((0x0808_3c32..0x0808_3c38).step_by(2));
    let blocks = addresses.map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("white-flowers-signed-bytes-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for index in [0u8, 20, 21, 22, 47, 48, 255] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, index); }
            for at in start..end { managed.bus.rom[at] = !rom[at]; }
            let before = owner.white_flowers_signed_bytes_stats().unwrap().source_bytes;
            let stop = 0x0808_3c38;
            let mut native_calls = 0;
            for machine_kind in 0..2 {
                let machine = if machine_kind == 0 { &mut raw } else { &mut managed };
                for _ in 0..512 {
                    if machine.cpu.regs[15] == stop { break; }
                    machine.bus.frame_ready = false;
                    let pc = machine.cpu.regs[15];
                    if machine_kind == 1 && native {
                        if let Some(block) = table.get(pc | 1) {
                            block(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
                            native_calls += 1;
                            continue;
                        }
                    }
                    machine.step();
                }
                assert_eq!(machine.cpu.regs[15], stop, "reader did not finish for index {index} native={native}");
            }
            if native { assert!(native_calls > 0, "generated reader must run"); }
            assert_eq!(owner.white_flowers_signed_bytes_stats().unwrap().source_bytes - before, u64::from(index < 48));
            let expected = (0x1234u16).wrapping_add((rom[start + index as usize] as i8 as i16) as u16);
            for machine in [&mut raw, &mut managed] {
                assert_eq!(machine.bus.read16(0x0203_0108), expected);
                assert_eq!(machine.bus.read8(0x0203_0112), index.wrapping_add(1) % 48);
            }
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "index={index} native={native}");
            cases += 1;
        }
    }
    assert_eq!(cases, 14);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[290,289], "resources":147, "new_source_bytes":48,
        "prior_resources_unchanged":146, "original_interpreter_generated_reader_cases":cases,
        "original_helper_interpreted":true, "out_of_projection_fallback_indices":[48,255],
        "poisoned_mixed_boundaries_and_mirrors":true, "full_state_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
