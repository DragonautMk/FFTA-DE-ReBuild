//! EUR interleaved rows, descriptors, wrappers and LZ payloads.
use super::*;
use gba_core::{capi::RT_API, rom_data::{observed_lz, observed_tilemap_descriptor as desc, observed_tilemap_row as row}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn poison(machine: &mut Machine, rom: &[u8], streams: &[observed_lz::ObservedLzStream], invert: bool) {
    for &(offset, _, _) in &row::INTERLEAVED_LZ_ROW_SPECS {
        for at in offset as usize..offset as usize + row::SOURCE_LENGTH { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
    for &(offset, _, wrapper, _) in &desc::INTERLEAVED_LZ_SPECS {
        for at in offset as usize..offset as usize + desc::SOURCE_LENGTH { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
        for at in wrapper as usize..wrapper as usize + desc::WRAPPER_LENGTH { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
    for stream in streams {
        for at in stream.source_offset as usize..stream.source_offset as usize + stream.encoded_source.len() {
            machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM, E200 predecessor and Wine; Linux/USA pending"]
fn european_interleaved_lz_chain_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_INTERLEAVED_LZ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_INTERLEAVED_LZ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (200, 199, 123));
    let old_lz: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_LZ_FILE)).unwrap()).unwrap();
    let old_desc: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_DESCRIPTOR_FILE)).unwrap()).unwrap();
    let old_rows: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (201, 200, 123));
    for prior in &old.resources {
        if [OBSERVED_LZ_ID, OBSERVED_TILEMAP_DESCRIPTOR_ID, OBSERVED_TILEMAP_ROW_ID].contains(&prior.id.as_str()) { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let lz = &source.observed_lz.as_ref().unwrap().decoded;
    let descriptors = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let new_lz = serde_json::to_value(lz).unwrap();
    for key in ["source_offset", "encoded_source", "decoded"] { assert_eq!(new_lz[key], old_lz[key]); }
    assert_eq!(&new_lz["additional_blocks"].as_array().unwrap()[..6], old_lz["additional_blocks"].as_array().unwrap());
    for (old_value, new_value) in [(old_desc, serde_json::to_value(descriptors).unwrap()), (old_rows, serde_json::to_value(rows).unwrap())] {
        for (key, value) in old_value.as_object().unwrap() { assert_eq!(&new_value[key], value, "preserved field {key}"); }
    }
    for (id, extra) in [(OBSERVED_LZ_ID, 7), (OBSERVED_TILEMAP_DESCRIPTOR_ID, 14), (OBSERVED_TILEMAP_ROW_ID, 7)] {
        let before = old.resources.iter().find(|entry| entry.id == id).unwrap();
        let after = source.manifest.resources.iter().find(|entry| entry.id == id).unwrap();
        assert_eq!(&after.sources[..before.sources.len()], &before.sources);
        assert_eq!(after.sources.len(), before.sources.len() + extra);
    }
    let streams = &lz.additional_blocks[6..]; let descs = descriptors.interleaved_lz_descriptors.as_ref().unwrap();
    let selected_rows = rows.interleaved_lz_rows.as_ref().unwrap();
    assert_eq!(streams.len(), 7); assert_eq!(streams.iter().map(|s| s.encoded_source.len()).sum::<usize>(), 4123);
    assert_eq!(streams.iter().map(|s| s.decoded.len()).sum::<usize>(), 5728);
    descriptors.validate_lz_dependencies(lz).unwrap(); rows.validate_dependencies(descriptors).unwrap();
    let mut missing = descriptors.clone(); missing.interleaved_lz_descriptors = None;
    assert!(rows.validate_dependencies(&missing).is_err());
    let mut bad_lz = lz.clone(); bad_lz.additional_blocks[6].decoded[0] ^= 1; assert!(bad_lz.validate().is_err());
    let mut bad_rows = rows.clone(); bad_rows.interleaved_lz_rows.as_mut().unwrap()[0].field8 ^= 1;
    let bad_bytes = encode_observed_tilemap_row(&bad_rows).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ROW_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap_row = Some((&bad_rows, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected"); drop(dataset);

    let mut visited: BTreeSet<u32> = (0x080db0c6..0x080db0de).step_by(2).collect();
    visited.extend((0x080db14e..0x080db154).step_by(2));
    visited.extend((0x080db30a..0x080db318).step_by(2));
    visited.extend((0x080db4a0..0x080db4b2).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("interleaved-lz-chain-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    poison(&mut managed, &rom, streams, true);
    for stream in streams {
        let start = stream.source_offset as usize; let end = start + stream.encoded_source.len();
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        } }
    }
    for entry in descs {
        let before_desc = owner.observed_tilemap_descriptor_stats().unwrap(); let before_lz = owner.observed_lz_stats().unwrap().source_bytes;
        let at = entry.wrapper.source_offset as usize + 2; let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        assert_eq!(owner.observed_tilemap_descriptor_stats().unwrap().interleaved_lz_wrapper_bytes - before_desc.interleaved_lz_wrapper_bytes, 2);
        assert_eq!(owner.observed_lz_stats().unwrap().source_bytes - before_lz, 2);
    }
    poison(&mut managed, &rom, streams, false); let initial = raw.save_state().unwrap();
    for (index, stream) in streams.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs[0] = 0x08000000 + stream.source_offset; machine.cpu.regs[1] = 0x020242bc;
            machine.cpu.regs[15] = 0x08145254; machine.cpu.cpsr = 0xbf;
        }
        poison(&mut managed, &rom, streams, true);
        let before = owner.observed_lz_stats().unwrap().interleaved_hle_calls_by_block;
        raw.step(); managed.step();
        let after = owner.observed_lz_stats().unwrap().interleaved_hle_calls_by_block;
        for i in 0..7 { assert_eq!(after[i] - before[i], u64::from(i == index)); }
        assert_eq!(&raw.bus.ewram[0x242bc..0x242bc + stream.decoded.len()], &stream.decoded);
        assert_eq!(&managed.bus.ewram[0x242bc..0x242bc + stream.decoded.len()], &stream.decoded);
        poison(&mut managed, &rom, streams, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "LZ stream {index}");
    }
    for (index, entry) in descs.iter().enumerate() {
        for flag_only in [false, true] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.bus.write32(0x02018024, 0x020242bc);
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[1] = 0x08000000 + selected_rows[index].source_offset;
                machine.cpu.regs[4] = 0x02018000; machine.cpu.regs[5] = entry.asset_pointer;
                machine.cpu.regs[7] = 1; machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = if flag_only { 0x080db14e } else { 0x080db0c6 }; machine.cpu.cpsr = 0xbf;
            }
            poison(&mut managed, &rom, streams, true);
            let before = owner.observed_tilemap_descriptor_stats().unwrap();
            let stop = if flag_only { 0x080db154 } else { 0x080db0de };
            let mut steps = 0;
            while raw.cpu.regs[15] != stop { assert!(steps < 20); raw.bus.frame_ready = false; raw.step(); steps += 1; }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop { assert!(steps < 20); managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1; }
            let after = owner.observed_tilemap_descriptor_stats().unwrap();
            assert_eq!(after.interleaved_lz_descriptor_bytes - before.interleaved_lz_descriptor_bytes, if flag_only { 0 } else { 4 });
            assert_eq!(after.interleaved_lz_wrapper_bytes - before.interleaved_lz_wrapper_bytes, 1);
            assert_eq!(raw.cpu.regs[0], u32::from(entry.wrapper.flags & if flag_only { 2 } else { 1 }));
            if !flag_only { assert_eq!(raw.cpu.regs[5], entry.asset_pointer); }
            poison(&mut managed, &rom, streams, false);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "descriptor {index}, flag {flag_only}");
        }
    }
    let base = row::INTERLEAVED_LZ_ROW_SPECS[0].0;
    for (index, selected) in selected_rows.iter().enumerate() {
        for field_reader in [false, true] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.bus.write32(0x02018000, 0x08000000 + base); machine.bus.write8(0x02018007, (index * 2) as u8);
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[4] = 0x02018000; machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = if field_reader { 0x080db4a0 } else { 0x080db30a }; machine.cpu.cpsr = 0xbf;
            }
            poison(&mut managed, &rom, streams, true);
            let before = owner.observed_tilemap_row_stats().unwrap().interleaved_lz_row_bytes;
            let stop = if field_reader { 0x080db4b2 } else { 0x080db318 };
            let mut steps = 0;
            while raw.cpu.regs[15] != stop { assert!(steps < 16); raw.bus.frame_ready = false; raw.step(); steps += 1; }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop { assert!(steps < 16); managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1; }
            assert_eq!(owner.observed_tilemap_row_stats().unwrap().interleaved_lz_row_bytes - before, if field_reader { 1 } else { 4 });
            if field_reader { assert_eq!(raw.cpu.regs[0], u32::from(selected.field1 & 1)); }
            else { assert_eq!(raw.cpu.regs[3], selected.descriptor_pointer); }
            poison(&mut managed, &rom, streams, false);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "row {index}, field {field_reader}");
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[201,200], "resources":123, "preserved_other_resources":120,
        "previous_resources_preserved":true, "streams":7, "encoded_bytes":4123, "decoded_bytes":5728,
        "descriptors":7, "wrappers":7, "rows":7, "missing_dependency_and_coherent_edit_rejected":true,
        "poisoned_mirrored_and_mixed_owner_boundaries":true, "original_swi_hle_cases":7,
        "original_interpreter_native_descriptor_cases":14, "original_interpreter_native_row_cases":14,
        "full_state_clock_equal":true, "ordinary_scene":"pending", "real_bios_dma":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
