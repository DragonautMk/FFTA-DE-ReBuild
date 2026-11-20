//! EUR additional compressed tilemaps and the original per-cell reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn poison(machine: &mut Machine, rom: &[u8], invert: bool) {
    for &(offset, length, _, _) in &model::ADDITIONAL_STREAM_SPECS {
        for at in offset as usize..offset as usize + length {
            machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM, E197 predecessor and Wine; Linux/USA pending"]
fn european_additional_tilemap_streams_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_STREAM_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_STREAM_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (197, 196, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (198, 197, 123));
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
    assert_eq!(current.sources.len(), prior.sources.len() + 7);
    let streams = data.additional_streams.as_ref().unwrap();
    let mut literal_tokens = 0usize;
    for (stream, &(offset, length, source_hash, decoded_hash)) in streams.iter().zip(&model::ADDITIONAL_STREAM_SPECS) {
        let start = offset as usize; let bytes = stream.source_bytes();
        assert_eq!(bytes, rom[start..start + length]);
        assert_eq!(gba_core::ffta_rom::digest(&bytes), source_hash);
        let cells = stream.decoded_cells().unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&cells.iter().flat_map(|cell| cell.to_le_bytes()).collect::<Vec<_>>()), decoded_hash);
        literal_tokens += stream.tokens.iter().filter(|token| token.control & 0xc000 == 0).count();
    }
    assert_eq!(literal_tokens, 5);
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    let literal = invalid.additional_streams.as_mut().unwrap().iter_mut().flat_map(|stream| &mut stream.tokens)
        .find(|token| token.control & 0xc000 == 0).unwrap();
    literal.values.pop(); assert!(invalid.validate().is_err());
    let mut bad = data.clone(); bad.additional_streams.as_mut().unwrap()[0].tokens[0].values[0] ^= 1;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let visited: BTreeSet<u32> = (0x080da14c..0x080da1fe).step_by(2).collect();
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("tilemap-streams-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    poison(&mut managed, &rom, true);
    for &(offset, length, _, _) in &model::ADDITIONAL_STREAM_SPECS {
        let start = offset as usize; let end = start + length;
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
            }
        }
    }
    poison(&mut managed, &rom, false);
    let initial = raw.save_state().unwrap();
    let mut consumed = [0u64; 7];
    for (which, stream) in streams.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.bus.write32(0x02010000, 0x08000000 + stream.source_offset + 6);
            machine.bus.write16(0x02010004, 0);
        }
        poison(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().additional_stream_bytes[which];
        for (index, expected) in stream.decoded_cells().unwrap().into_iter().enumerate() {
            for machine in [&mut raw, &mut managed] {
                machine.cpu.regs[0] = 0x02010000; machine.cpu.regs[1] = 0x02010004; machine.cpu.regs[2] = 2;
                machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x03007ff1;
                machine.cpu.regs[15] = 0x080da14c; machine.cpu.cpsr = 0xbf;
            }
            let mut steps = 0;
            while raw.cpu.regs[15] != 0x03007ff0 {
                assert!(steps < 100, "original tilemap reader did not return at {which}:{index}");
                raw.bus.frame_ready = false; raw.step(); steps += 1;
            }
            let mut steps = 0;
            while managed.cpu.regs[15] != 0x03007ff0 {
                assert!(steps < 100, "native tilemap reader did not return at {which}:{index}");
                managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                steps += 1;
            }
            assert_eq!(raw.cpu.regs[0], u32::from(expected), "interpreter cell {which}:{index}");
            assert_eq!(managed.cpu.regs[0], u32::from(expected), "native cell {which}:{index}");
        }
        consumed[which] = owner.observed_tilemap_stats().unwrap().additional_stream_bytes[which] - before;
        assert!(consumed[which] >= (model::ADDITIONAL_STREAM_SPECS[which].1 - 6) as u64);
        poison(&mut managed, &rom, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "tilemap reader {which}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[198,197], "resources":123, "preserved_other_resources":122,
        "previous_tilemap_preserved":true, "streams":7, "source_bytes":722, "decoded_cells":1792,
        "literal_tokens":literal_tokens, "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_cells":1792, "full_state_clock_equal":true, "stream_bytes_served":consumed,
        "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
