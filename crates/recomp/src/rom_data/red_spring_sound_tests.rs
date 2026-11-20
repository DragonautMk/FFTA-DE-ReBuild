//! EUR Red Spring timed sound schedules and original guest reads.
use super::*;
use gba_core::{capi::RT_API, rom_data::audio_directory as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, E186 predecessor and Wine; Linux/USA pending"]
fn european_red_spring_sounds_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOUND_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOUND_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (186, 185, 120));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(AUDIO_DIRECTORY_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (187, 186, 120));
    for prior in &old.resources {
        if prior.id == AUDIO_DIRECTORY_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.audio_directory.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == AUDIO_DIRECTORY_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == AUDIO_DIRECTORY_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 4);
    let schedules = data.red_spring_sounds.as_ref().unwrap();
    for (schedule, &(root_offset, offset, count)) in schedules.iter().zip(&model::RED_SPRING_SOUND_SPECS) {
        assert_eq!((schedule.root_offset, schedule.source_offset, schedule.rows.len()), (root_offset, offset, count));
        assert_eq!(schedule.root_pointer, 0x08000000 + offset);
        assert_eq!(schedule.rows.iter().flat_map(model::TimedSoundRow::source_bytes).collect::<Vec<_>>(), rom[offset as usize..offset as usize + count * 8]);
    }
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_audio_directory(&fs::read(prepared.directory.join(AUDIO_DIRECTORY_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.red_spring_sounds.as_mut().unwrap()[0].root_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.red_spring_sounds.as_mut().unwrap()[0].rows[0].sound_words[0] = 0x0fff; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.red_spring_sounds.as_mut().unwrap()[0].rows[0].tick ^= 1;
    let bad_bytes = encode_audio_directory(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == AUDIO_DIRECTORY_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.audio_directory = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let visited = [0x080e3688, 0x080e36aa, 0x080e36b2];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("red-spring-sound-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let spans = model::RED_SPRING_SOUND_SPECS.iter().flat_map(|&(root, offset, count)| [(root as usize, 4), (offset as usize, count * 8)]).collect::<Vec<_>>();
    for &(start, len) in &spans { for at in start..start + len { managed.bus.rom[at] = !rom[at]; } }
    for &(start, len) in &spans {
        for at in start..start + len { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, start + len] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
            }
        }
    }
    for &(start, len) in &spans { managed.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]); }
    let initial = raw.save_state().unwrap();
    for (index, schedule) in schedules.iter().enumerate() {
        for (row_index, row) in schedule.rows.iter().enumerate() {
            let address = 0x08000000 + schedule.source_offset + row_index as u32 * 8;
            for (mode, &pc) in visited.iter().enumerate() {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                    machine.cpu.regs[0] = address; machine.cpu.regs[1] = address; machine.cpu.regs[13] = 0x03007c00;
                    machine.cpu.regs[15] = pc; machine.cpu.cpsr = 0xbf;
                }
                for &(start, len) in &spans { for at in start..start + len { managed.bus.rom[at] = !rom[at]; } }
                let before = owner.audio_directory_stats().unwrap().red_spring_sound_bytes[index];
                raw.bus.frame_ready = false; raw.step();
                managed.bus.frame_ready = false;
                table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                let after = owner.audio_directory_stats().unwrap().red_spring_sound_bytes[index];
                assert_eq!(after - before, [2, 2, 1][mode]);
                if mode == 0 { assert_eq!(raw.cpu.regs[0], u32::from(row.tick)); }
                if mode == 1 { assert_eq!(raw.cpu.regs[2], u32::from(row.sound_words[0])); }
                if mode == 2 { assert_eq!(raw.cpu.regs[0], u32::from(row.channel)); }
                for &(start, len) in &spans { managed.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]); }
                assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
            }
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[187,186], "resources":120, "preserved_other_resources":119,
        "previous_audio_directory_preserved":true, "schedules":2, "rows":7, "new_source_bytes":64,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":21, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
