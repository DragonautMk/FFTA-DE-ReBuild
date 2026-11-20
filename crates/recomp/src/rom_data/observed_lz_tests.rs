//! Publication and original SWI consumption of the observed EUR LZ block.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_lz::{OFFSET, ENCODED_LENGTH, DECODED_LENGTH}, Bus, Machine};

#[test]
#[ignore = "exact EUR ROM and E173 predecessor; Wine only"]
fn european_red_spring_lz_publication_and_swi() {
    let root = PathBuf::from(std::env::var_os("FFTA_OBSERVED_LZ_ROOT").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_OBSERVED_LZ_PRIOR").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (173,172,115));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (174,173,116));
    for entry in &old.resources {
        let now = source.manifest.resources.iter().find(|r| r.id == entry.id).unwrap();
        assert_eq!(now, entry);
        assert_eq!(fs::read(prepared.directory.join(&now.path)).unwrap(), fs::read(prior.join(&entry.path)).unwrap());
    }
    let data = &source.observed_lz.as_ref().unwrap().decoded;
    assert_eq!(data.encoded_source.len(), ENCODED_LENGTH);
    assert_eq!(data.decoded.len(), DECODED_LENGTH);
    assert_eq!(&data.encoded_source, &rom[OFFSET as usize..OFFSET as usize + ENCODED_LENGTH]);
    assert_eq!(parse_observed_lz(&fs::read(prepared.directory.join(OBSERVED_LZ_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.decoded[0] ^= 1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.encoded_source.pop(); assert!(bad.validate().is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = OFFSET as usize; let end = start + ENCODED_LENGTH;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.observed_lz_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    assert_eq!(owner.observed_lz_stats().unwrap().source_bytes - before, ENCODED_LENGTH as u64);
    for edge in [start,end] { for delta in [-3isize,-1,0,1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    } }
    for machine in [&mut raw, &mut managed] {
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs[0] = 0x08000000 + OFFSET;
        machine.cpu.regs[1] = 0x020242bc;
        machine.cpu.regs[15] = 0x08145254;
        machine.cpu.cpsr = 0xbf;
        machine.step();
    }
    assert_eq!(owner.observed_lz_stats().unwrap().hle_calls, 1);
    assert_eq!(&raw.bus.ewram[0x242bc..0x242bc+DECODED_LENGTH], &data.decoded);
    assert_eq!(&managed.bus.ewram[0x242bc..0x242bc+DECODED_LENGTH], &data.decoded);
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[174,173], "resources":116, "preserved_resources":115,
        "encoded_bytes":ENCODED_LENGTH, "decoded_bytes":DECODED_LENGTH,
        "poisoned_mirrored_and_boundary_reads":true, "original_swi_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E174 predecessor; Wine only"]
fn european_red_spring_lz_streams_publication_and_swi() {
    use gba_core::rom_data::observed_lz::{ObservedLzBlock, ADDITIONAL_SPECS};
    let root = PathBuf::from(std::env::var_os("FFTA_OBSERVED_LZ_ROOT").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_OBSERVED_LZ_PRIOR").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (174,173,116));
    let old_block: ObservedLzBlock = serde_json::from_slice(&fs::read(prior.join(OBSERVED_LZ_FILE)).unwrap()).unwrap();
    assert!(old_block.additional_blocks.is_empty());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (175,174,116));
    for entry in &old.resources {
        if entry.id == OBSERVED_LZ_ID { continue; }
        let now = source.manifest.resources.iter().find(|r| r.id == entry.id).unwrap();
        assert_eq!(now, entry);
        assert_eq!(fs::read(prepared.directory.join(&now.path)).unwrap(), fs::read(prior.join(&entry.path)).unwrap());
    }
    let data = &source.observed_lz.as_ref().unwrap().decoded;
    assert_eq!((data.source_offset, &data.encoded_source, &data.decoded),
        (old_block.source_offset, &old_block.encoded_source, &old_block.decoded));
    assert_eq!(data.additional_blocks.len(), 6);
    assert_eq!(ADDITIONAL_SPECS.iter().map(|s| s.encoded_length).sum::<usize>(), 2291);
    assert_eq!(parse_observed_lz(&fs::read(prepared.directory.join(OBSERVED_LZ_FILE)).unwrap()).unwrap(), *data);
    for (offset, encoded, _) in data.streams() {
        let start = offset as usize;
        assert_eq!(encoded, &rom[start..start + encoded.len()]);
    }
    let mut bad = data.clone(); bad.additional_blocks[0].decoded[0] ^= 1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.additional_blocks.swap(0,1); assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.additional_blocks.pop(); assert!(bad.validate().is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let ranges: Vec<_> = data.streams().map(|(offset, encoded, _)| (offset as usize, encoded.len())).collect();
    for &(start, len) in &ranges { for at in start..start+len { managed.bus.rom[at] = !rom[at]; } }
    let before = owner.observed_lz_stats().unwrap().source_bytes;
    for &(start, len) in &ranges { for at in start..start+len {
        assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]);
    } }
    assert_eq!(owner.observed_lz_stats().unwrap().source_bytes - before, 4869);
    for &(start, len) in &ranges { for edge in [start, start+len] { for delta in [-3isize,-1,0,1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    } } }
    for &(start, len) in &ranges { managed.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]); }
    let initial = raw.save_state().unwrap();
    for (index, (offset, _, decoded)) in data.streams().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs[0] = 0x08000000 + offset;
            machine.cpu.regs[1] = 0x020242bc;
            machine.cpu.regs[15] = 0x08145254;
            machine.cpu.cpsr = 0xbf;
        }
        for &(start, len) in &ranges { for at in start..start+len { managed.bus.rom[at] = !rom[at]; } }
        let before = owner.observed_lz_stats().unwrap().hle_calls_by_block;
        raw.step(); managed.step();
        let after = owner.observed_lz_stats().unwrap().hle_calls_by_block;
        for i in 0..after.len() { assert_eq!(after[i] - before[i], u64::from(i == index)); }
        assert_eq!(&raw.bus.ewram[0x242bc..0x242bc+decoded.len()], decoded);
        assert_eq!(&managed.bus.ewram[0x242bc..0x242bc+decoded.len()], decoded);
        for &(start, len) in &ranges { managed.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]); }
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    assert!(managed.bus.decoded_observed_lz(0x02000000).is_none());
    assert!(managed.bus.decoded_observed_lz(0x08000000 + OFFSET + 1).is_none());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[175,174], "resources":116, "preserved_other_resources":115,
        "previous_lz_block_preserved":true, "new_streams":6, "new_encoded_bytes":2291,
        "all_encoded_bytes":4869, "poisoned_mirrored_and_boundary_reads":true,
        "original_swi_state_clock_cases":7,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
