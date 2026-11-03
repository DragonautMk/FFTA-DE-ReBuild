//! Opt-in measurements and complete recorded-scene differential checks.
//! Fixtures and machine-specific results remain outside the repository.
use super::*;
use crate::{
    arm_audio_hle, load_native, make_machine, run_frame_native, InputScript, FRAME_STEP_GUARD,
};
use std::time::Instant;

#[derive(serde::Serialize)]
struct StartupObservation {
    cached: bool,
    source_io_ms: f64,
    source_decode_ms: f64,
    preparation_ms: f64,
    activation_ms: f64,
    total_after_source_read_ms: f64,
    resident_bytes: usize,
    generation: String,
}

#[test]
#[ignore = "requires a private ROM and a new FFTA_STATE_TEST_COST_ROOT output directory"]
fn measure_decoded_startup_costs() {
    let rom_path = std::env::var_os("FFTA_STATE_TEST_ROM").expect("ROM fixture");
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COST_ROOT").expect("cost output"));
    assert!(
        !root.exists(),
        "preserve earlier measurements; select a new output root"
    );
    fs::create_dir_all(&root).unwrap();
    let mut observations = Vec::new();
    for index in 0..7 {
        let dataset_root = root.join(format!("sample-{index}"));
        for cached in [false, true] {
            let io_start = Instant::now();
            let rom = fs::read(&rom_path).unwrap();
            let source_io_ms = io_start.elapsed().as_secs_f64() * 1000.0;
            // ROM allocation and machine construction belong to both paths.
            let mut machine = make_machine(rom, None);
            let start = Instant::now();
            let mut source = Extraction::from_rom(&machine.bus.rom).unwrap();
            let decoded_at = start.elapsed();
            let prepared =
                preparation::prepare(&dataset_root, &mut source, &mut |_| Ok(())).unwrap();
            let prepared_at = start.elapsed();
            assert_eq!(prepared.action, if cached { "reused" } else { "created" });
            attach_from_directory(&mut machine, &prepared.directory).unwrap();
            let activated_at = start.elapsed();
            let resources = machine.bus.rom_resources().unwrap();
            observations.push(StartupObservation {
                cached,
                source_io_ms,
                source_decode_ms: decoded_at.as_secs_f64() * 1000.0,
                preparation_ms: (prepared_at - decoded_at).as_secs_f64() * 1000.0,
                activation_ms: (activated_at - prepared_at).as_secs_f64() * 1000.0,
                total_after_source_read_ms: activated_at.as_secs_f64() * 1000.0,
                resident_bytes: resources.resident_bytes() + 2 * std::mem::size_of::<usize>(),
                generation: prepared
                    .directory
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            });
        }
    }
    #[derive(serde::Serialize)]
    struct Report {
        observations: Vec<StartupObservation>,
    }
    // Save every observation, including slower results; evaluate budgets separately.
    fs::write(
        root.join("startup.toml"),
        toml::to_string_pretty(&Report { observations }).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires private ROM/native/input fixtures and explicit scene length; optional state/save"]
fn decoded_base_matches_recorded_scene_at_every_frame() {
    crate::configure_headless_widescreen();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").expect("ROM fixture")).unwrap();
    let library = std::env::var_os("FFTA_STATE_TEST_NATIVE").expect("native fixture");
    let state = std::env::var_os("FFTA_STATE_TEST_STATE");
    let save = std::env::var_os("FFTA_STATE_TEST_SAVE");
    assert!(
        !(state.is_some() && save.is_some()),
        "choose a snapshot or cartridge save"
    );
    let input_path = std::env::var("FFTA_STATE_TEST_INPUT").expect("input fixture");
    let frames: u64 = std::env::var("FFTA_STATE_TEST_FRAMES")
        .expect("explicit frame count")
        .parse()
        .unwrap();
    assert!(frames > 0);
    let input = InputScript::load(&input_path).unwrap();
    let (_library, table) = load_native(Path::new(&library)).unwrap();
    let mut legacy = make_machine(rom.clone(), None);
    let mut decoded = make_machine(rom.clone(), None);
    let directory = tempfile::tempdir().unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let dataset = match std::env::var_os("FFTA_STATE_TEST_DATASET") {
        Some(path) => PathBuf::from(path),
        None => {
            let dataset = directory.path().join("decoded");
            extract(&dataset, &mut source).unwrap();
            dataset
        }
    };
    attach_from_directory(&mut decoded, &dataset).unwrap();
    for machine in [&mut legacy, &mut decoded] {
        if let Some(path) = &state {
            crate::save_state::load(machine, Path::new(path)).unwrap();
        }
        if let Some(path) = &save {
            machine.bus.load_save_data(&fs::read(path).unwrap());
        }
    }
    // Compare the unmodified snapshot before configuring audio observation.
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for machine in [&mut legacy, &mut decoded] {
        // Loading a snapshot can overwrite these host shadows. Arm identically
        // AFTER loading so a zero-length HLE tap cannot masquerade as coverage.
        arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        clear_audio(machine);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    let alternate_mode = std::env::var("FFTA_STATE_TEST_ALTERNATE_MODE")
        .ok()
        .map(|value| value.parse::<u8>().unwrap());
    if let Some(mode) = alternate_mode {
        assert!(matches!(mode, 1 | 2), "select an original alternate Color Mode");
        assert_eq!(decoded.bus.ewram[0x2fc2] & 3, mode, "loaded Color Mode");
    }
    let first = legacy.bus.frames;
    let mut dispatch = [0u64; 2];
    let mut audio = [0u64; 5];
    let mut source_mask = 0;
    let mut first_alternate_read = None;
    for relative in 0..frames {
        let mut counts = [(0, 0); 2];
        for (index, machine) in [&mut legacy, &mut decoded].into_iter().enumerate() {
            machine.bus.keys = input.keys_at(relative);
            input.apply_mouse(&mut machine.bus, relative).unwrap();
            let pointer = machine as *mut gba_core::Machine as *mut std::ffi::c_void;
            counts[index] = run_frame_native(machine, &table, pointer, FRAME_STEP_GUARD);
            assert_eq!(machine.bus.frames, first + relative + 1, "incomplete frame");
            assert!(machine.bus.frame_ready);
        }
        assert_eq!(counts[0], counts[1], "dispatch at {relative}");
        dispatch[0] += counts[0].0;
        dispatch[1] += counts[0].1;
        assert_eq!(
            legacy.bus.framebuffer, decoded.bus.framebuffer,
            "central pixels at {relative}"
        );
        assert_eq!(
            legacy.bus.wide_framebuffer, decoded.bus.wide_framebuffer,
            "wide pixels at {relative}"
        );
        assert_eq!(legacy.bus.oam, decoded.bus.oam, "OAM at {relative}");
        assert_eq!(
            legacy.bus.audio_buf, decoded.bus.audio_buf,
            "mixed audio at {relative}"
        );
        assert_eq!(legacy.bus.psg_tap, decoded.bus.psg_tap, "PSG at {relative}");
        assert_eq!(
            legacy.bus.fifo_tap, decoded.bus.fifo_tap,
            "FIFO at {relative}"
        );
        assert_eq!(legacy.bus.hle_tap.len(), decoded.bus.hle_tap.len());
        assert!(
            legacy
                .bus
                .hle_tap
                .iter()
                .zip(&decoded.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {relative}"
        );
        assert!(
            legacy.save_state().unwrap() == decoded.save_state().unwrap(),
            "full state at {relative}"
        );
        if let Some(mode) = alternate_mode {
            assert_eq!(decoded.bus.ewram[0x2fc2] & 3, mode, "Color Mode at {relative}");
            let alternate = decoded.bus.rom_resources().unwrap().alternate_palette_stats().unwrap();
            if alternate.decoded_literal_bytes > 0 && first_alternate_read.is_none() {
                let id = u16::from_le_bytes(decoded.bus.ewram[0x7f10..0x7f12].try_into().unwrap());
                first_alternate_read = Some((relative, decoded.bus.frames, id));
            }
        }
        for (sum, len) in audio.iter_mut().zip([
            decoded.bus.audio_buf.len(),
            decoded.bus.psg_tap.len(),
            decoded.bus.fifo_tap[0].len(),
            decoded.bus.fifo_tap[1].len(),
            decoded.bus.hle_tap.len(),
        ]) {
            *sum += len as u64;
        }
        let mask = decoded.bus.rom_resources().unwrap().stats().bus_source_mask;
        if mask != source_mask {
            eprintln!(
                "ROM_DATA_SOURCE first_frame={} clock={} mask={mask}",
                decoded.bus.frames, decoded.bus.clock
            );
            source_mask = mask;
        }
        if relative % 500 == 499 {
            eprintln!("ROM_DATA_SCENE_PROGRESS frames={}/{}", relative + 1, frames);
        }
        clear_audio(&mut legacy);
        clear_audio(&mut decoded);
    }
    assert!(dispatch[0] > 0, "native dispatch must execute");
    assert!(
        audio.iter().all(|&count| count > 0),
        "all audio streams need actual samples: {audio:?}"
    );
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_CONDITIONAL_LOOKUP").is_some() {
        let lookup = decoded.bus.rom_resources().unwrap().conditional_four_byte_lookup_stats()
            .expect("decoded EUR conditional lookup");
        assert!(lookup.bus_reads > 0 && lookup.first_field_bytes > 0 && lookup.second_field_bytes > 0,
            "the scene must consume both conditional lookup fields");
        eprintln!("ROM_DATA_CONDITIONAL_LOOKUP_SCENE frames={frames} bus_reads={} source_bytes={} first_field_bytes={} second_field_bytes={}",
            lookup.bus_reads, lookup.source_bytes, lookup.first_field_bytes, lookup.second_field_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_DESCRIPTOR_BANKS").is_some() {
        let banks = decoded.bus.rom_resources().unwrap().conditional_descriptor_banks_stats()
            .expect("decoded EUR conditional descriptor banks");
        assert!(banks.pointer_bytes > 0 && banks.field0_bytes > 0 && banks.field2_bytes > 0,
            "the scene must consume descriptor pointers and both selected halfword fields");
        eprintln!("ROM_DATA_DESCRIPTOR_BANKS_SCENE frames={frames} bus_reads={} source_bytes={} pointer_bytes={} field0_bytes={} field2_bytes={} field4_bytes={} kind_bytes={} byte7_bytes={}",
            banks.bus_reads, banks.source_bytes, banks.pointer_bytes, banks.field0_bytes, banks.field2_bytes, banks.field4_bytes, banks.kind_bytes, banks.byte7_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_DESCRIPTOR_BANKS_FIELD4").is_some() {
        let banks = decoded.bus.rom_resources().unwrap().conditional_descriptor_banks_stats()
            .expect("decoded EUR conditional descriptor banks");
        assert!(banks.pointer_bytes > 0 && banks.field4_bytes > 0,
            "the scene must consume descriptor pointers and field 4");
        eprintln!("ROM_DATA_DESCRIPTOR_BANKS_FIELD4_SCENE frames={frames} bus_reads={} source_bytes={} pointer_bytes={} field0_bytes={} field2_bytes={} field4_bytes={} kind_bytes={} byte7_bytes={}",
            banks.bus_reads, banks.source_bytes, banks.pointer_bytes, banks.field0_bytes, banks.field2_bytes, banks.field4_bytes, banks.kind_bytes, banks.byte7_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_DESCRIPTOR_BANK_ZERO").is_some() {
        let bank = decoded.bus.rom_resources().unwrap().conditional_descriptor_bank_zero_stats()
            .expect("decoded EUR conditional descriptor bank zero");
        assert!(bank.pointer_bytes > 0 && bank.field0_bytes > 0 && bank.field2_bytes > 0
            && bank.kind_bytes > 0 && bank.byte7_bytes > 0,
            "the scene must consume the bank-zero pointer and observed record fields");
        eprintln!("ROM_DATA_DESCRIPTOR_BANK_ZERO_SCENE frames={frames} bus_reads={} source_bytes={} pointer_bytes={} field0_bytes={} field2_bytes={} field4_bytes={} kind_bytes={} byte7_bytes={}",
            bank.bus_reads, bank.source_bytes, bank.pointer_bytes, bank.field0_bytes, bank.field2_bytes, bank.field4_bytes, bank.kind_bytes, bank.byte7_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_DESCRIPTOR_BANK_ZERO_FIELD4").is_some() {
        let bank = decoded.bus.rom_resources().unwrap().conditional_descriptor_bank_zero_stats()
            .expect("decoded EUR conditional descriptor bank zero");
        assert!(bank.pointer_bytes > 0 && bank.field4_bytes > 0,
            "the scene must consume the bank-zero pointer and field 4");
        eprintln!("ROM_DATA_DESCRIPTOR_BANK_ZERO_FIELD4_SCENE frames={frames} bus_reads={} source_bytes={} pointer_bytes={} field0_bytes={} field2_bytes={} field4_bytes={} kind_bytes={} byte7_bytes={}",
            bank.bus_reads, bank.source_bytes, bank.pointer_bytes, bank.field0_bytes, bank.field2_bytes, bank.field4_bytes, bank.kind_bytes, bank.byte7_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PROPERTY_MASKS").is_some() {
        let masks = decoded.bus.rom_resources().unwrap().conditional_property_masks_stats()
            .expect("decoded EUR conditional property masks");
        assert!(masks.bus_reads > 0 && masks.source_bytes > 0,
            "the scene must consume property-selected masks");
        eprintln!("ROM_DATA_PROPERTY_MASKS_SCENE frames={frames} bus_reads={} source_bytes={}",
            masks.bus_reads, masks.source_bytes);
    }
    let stats = decoded.bus.rom_resources().unwrap().stats();
    let bg = decoded
        .bus
        .rom_resources()
        .unwrap()
        .background_stats()
        .unwrap();
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_BG").is_some() {
        assert!(
            bg.bus_reads > 0,
            "this scene must exercise BG metadata reads"
        );
    }
    let palette = decoded
        .bus
        .rom_resources()
        .unwrap()
        .palette_stats()
        .unwrap();
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PALETTE").is_some() {
        assert!(
            palette.lz77_calls > 0 && palette.decoded_bytes > 0,
            "this scene must consume decoded palette banks"
        );
    }
    eprintln!("ROM_DATA_PALETTE_SCENE frames={frames} bus_reads={} source_bytes={} lz77_calls={} decoded_bytes={}", palette.bus_reads, palette.source_bytes, palette.lz77_calls, palette.decoded_bytes);
    if let Some(mode) = alternate_mode {
        let alternate = decoded.bus.rom_resources().unwrap().alternate_palette_stats().unwrap();
        assert!(alternate.table_bytes > 0 && alternate.decoded_literal_bytes > 0,
            "this scene must read alternate links and decoded literal banks");
        let (relative, absolute, id) = first_alternate_read.expect("an actual alternate scene");
        eprintln!("ROM_DATA_ALTERNATE_SCENE mode={mode} frames={frames} first_relative={relative} first_absolute={absolute} background_id={id} bus_reads={} source_bytes={} table_bytes={} decoded_literal_bytes={}", alternate.bus_reads, alternate.source_bytes, alternate.table_bytes, alternate.decoded_literal_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PACKET").is_some() {
        let packet = decoded.bus.rom_resources().unwrap().packet_stats().expect("decoded EUR packets");
        assert!(packet.typed_value_bytes + packet.typed_hle_bytes > 0,
            "this scene must consume decoded packet values");
        eprintln!("ROM_DATA_PACKET_SCENE frames={frames} bus_reads={} source_bytes={} typed_value_bytes={} lz77_calls={} decoded_body_bytes={} typed_hle_bytes={}", packet.bus_reads, packet.source_bytes, packet.typed_value_bytes, packet.lz77_calls, packet.decoded_body_bytes, packet.typed_hle_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_GRAPHICS").is_some() {
        let graphics = decoded.bus.rom_resources().unwrap().graphics_stats().expect("decoded EUR graphics");
        assert!(graphics.decoded_literal_bytes > 0, "this scene must consume decoded main graphics words");
        eprintln!("ROM_DATA_GRAPHICS_SCENE frames={frames} bus_reads={} source_bytes={} decoded_literal_bytes={}",
            graphics.bus_reads, graphics.source_bytes, graphics.decoded_literal_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_TRANSFER").is_some() {
        let transfer = decoded.bus.rom_resources().unwrap().transfer_stats().expect("decoded EUR transfers");
        assert!(transfer.control_bytes > 0 && transfer.tile_bytes > 0, "this scene must use typed transfer controls and tile words");
        eprintln!("ROM_DATA_TRANSFER_SCENE frames={frames} bus_reads={} source_bytes={} control_bytes={} tile_bytes={}",
            transfer.bus_reads, transfer.source_bytes, transfer.control_bytes, transfer.tile_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_TEXT_CATALOG").is_some() {
        let t = decoded.bus.rom_resources().unwrap().text_catalog_stats().expect("decoded EUR text catalogs");
        let complete_font = decoded.bus.rom_resources().unwrap().font_keyboard_stats();
        assert!(t.archive_index_bytes > 0 && t.decoded_program_bytes > 0
            && t.glyph_bytes + complete_font.map_or(0, |f| f.glyph_bytes) > 0,
            "this scene must consume typed archive programs and font words");
        eprintln!("ROM_DATA_TEXT_CATALOG_SCENE frames={frames} bus_reads={} source_bytes={} pointer_table_bytes={} string_bytes={} archive_index_bytes={} header_bytes={} encoded_control_bytes={} decoded_program_bytes={} glyph_bytes={} width_bytes={}",
            t.bus_reads,t.source_bytes,t.pointer_table_bytes,t.string_bytes,t.archive_index_bytes,t.header_bytes,t.encoded_control_bytes,t.decoded_program_bytes,t.glyph_bytes,t.width_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_COMPLETE_FONT").is_some() {
        let f = decoded.bus.rom_resources().unwrap().font_keyboard_stats().expect("decoded complete EUR font");
        assert!(f.glyph_bytes > 0 && f.width_bytes > 0, "this scene must consume complete-bank glyphs and metrics");
        eprintln!("ROM_DATA_FONT_KEYBOARD_SCENE frames={frames} bus_reads={} glyph_bytes={} width_bytes={} index_bytes={} program_bytes={} row_width_bytes={}",
            f.bus_reads,f.glyph_bytes,f.width_bytes,f.keyboard_index_bytes,f.keyboard_program_bytes,f.keyboard_row_width_bytes);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_OBJ_LISTS").is_some() {
        let l = decoded.bus.rom_resources().unwrap().obj_list_stats().expect("decoded EUR static OBJ lists");
        assert!(l.count_bytes > 0 && l.attribute_bytes > 0 && l.host_reads > 0,
            "this scene must consume file-derived OBJ counts/attributes in guest and host paths");
        eprintln!("ROM_DATA_OBJ_LIST_SCENE frames={frames} bus_reads={} source_bytes={} count_bytes={} attribute_bytes={} host_reads={}",
            l.bus_reads, l.source_bytes, l.count_bytes, l.attribute_bytes, l.host_reads);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_EXTRA_OBJ_LISTS").is_some() {
        let resources = decoded.bus.rom_resources().unwrap();
        let touched: Vec<_> = [0x394f34,0x3950fa,0x395102,0x399628,0x3adfc4,0x3adfd2,0x3adfda].into_iter()
            .filter(|&offset| resources.obj_list_source_used(offset) == Some((true,true))).collect();
        eprintln!("ROM_DATA_EXTRA_OBJ_SCENE frames={frames} guest_and_host_sources={touched:x?}");
        assert!(!touched.is_empty(), "this scene must consume a newly added OBJ list through guest and host readers");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_OBSERVED_OBJ_LISTS").is_some() {
        let resources = decoded.bus.rom_resources().unwrap();
        assert!([0x394f4c, 0x40d89e].into_iter().all(|at| resources.obj_list_source_used(at) == Some((true, true))), "this scene must consume both observed OBJ lists through guest and host readers");
        eprintln!("ROM_DATA_OBSERVED_OBJ_SCENE frames={frames} sources=00394f4c,0040d89e guest=true host=true");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_CONSTRUCTOR_ANIMATION").is_some() {
        let resources = decoded.bus.rom_resources().unwrap();
        let i = resources.constructor_obj_animation_stats().expect("decoded constructor animation");
        assert!(i.threshold_bytes > 0 && i.pointer_bytes > 0 && i.phases_used == 4, "ordinary scene must consume both fields and all four constructor animation phases");
        assert!(gba_core::rom_data::obj_lists::constructor_animation::LIST_ROOTS.into_iter().all(|(at, _)| resources.obj_list_source_used(at) == Some((true, true))), "ordinary scene must consume all four new lists through guest and host readers");
        eprintln!("ROM_DATA_CONSTRUCTOR_SCENE frames={frames} bus_reads={} source_bytes={} threshold_bytes={} pointer_bytes={} phases_used={} lists_guest_host=4",i.bus_reads,i.source_bytes,i.threshold_bytes,i.pointer_bytes,i.phases_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_EVENT_RECORDS12").is_some() {
        let i = decoded.bus.rom_resources().unwrap().event_records12_stats().expect("decoded EUR event records12");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && i.header_bytes+i.key_bytes+i.primary_bytes+i.alternate_bytes+i.parameter_bytes+i.byte07_bytes+i.flag_bytes+i.opaque_bytes == i.source_bytes, "ordinary scene must consume event-records12 storage");
        eprintln!("ROM_DATA_EVENT_RECORDS12_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} key_bytes={} primary_bytes={} alternate_bytes={} parameter_bytes={} byte07_bytes={} flag_bytes={} opaque_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.header_bytes,i.key_bytes,i.primary_bytes,i.alternate_bytes,i.parameter_bytes,i.byte07_bytes,i.flag_bytes,i.opaque_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_WORLD_RECORDS38").is_some() {
        let i = decoded.bus.rom_resources().unwrap().world_records38_stats().expect("decoded EUR world records38");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && i.byte01_bytes+i.value02_bytes+i.value04_bytes+i.threshold_bytes+i.opaque_bytes == i.source_bytes, "ordinary scene must consume world-records38 storage");
        eprintln!("ROM_DATA_WORLD_RECORDS38_SCENE frames={frames} bus_reads={} source_bytes={} byte01_bytes={} value02_bytes={} value04_bytes={} threshold_bytes={} opaque_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.byte01_bytes,i.value02_bytes,i.value04_bytes,i.threshold_bytes,i.opaque_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PARAMETER_RECORDS27").is_some() {
        let i = decoded.bus.rom_resources().unwrap().parameter_records27_stats().expect("decoded EUR sparse parameter records27");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && i.header_bytes+i.packed_bytes+i.value07_bytes+i.byte09_bytes+i.value0a_bytes+i.signed0c_bytes+i.flags0d_bytes+i.operands0e_bytes+i.operands10_bytes+i.value12_bytes+i.signed15_bytes+i.opaque_bytes == i.source_bytes, "ordinary scene must consume observed parameter-records27 storage");
        eprintln!("ROM_DATA_PARAMETER_RECORDS27_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} packed_bytes={} value07_bytes={} byte09_bytes={} value0a_bytes={} signed0c_bytes={} flags0d_bytes={} operands0e_bytes={} operands10_bytes={} value12_bytes={} signed15_bytes={} opaque_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.header_bytes,i.packed_bytes,i.value07_bytes,i.byte09_bytes,i.value0a_bytes,i.signed0c_bytes,i.flags0d_bytes,i.operands0e_bytes,i.operands10_bytes,i.value12_bytes,i.signed15_bytes,i.opaque_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_INDEXED_GRAPHICS").is_some() {
        let i = decoded.bus.rom_resources().unwrap().indexed_graphics_stats().expect("decoded EUR indexed graphics");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && (i.archives_used > 0 || i.groups_used > 0) && i.selector_bytes+i.archive_header_bytes+i.index_bytes+i.token_bytes+i.literal_bytes+i.dictionary_header_bytes+i.dictionary_bytes == i.source_bytes, "ordinary scene must consume indexed-graphics storage");
        eprintln!("ROM_DATA_INDEXED_GRAPHICS_SCENE frames={frames} bus_reads={} source_bytes={} selector_bytes={} archive_header_bytes={} index_bytes={} token_bytes={} literal_bytes={} dictionary_header_bytes={} dictionary_bytes={} archives_used={} programs_used={} groups_used={}",i.bus_reads,i.source_bytes,i.selector_bytes,i.archive_header_bytes,i.index_bytes,i.token_bytes,i.literal_bytes,i.dictionary_header_bytes,i.dictionary_bytes,i.archives_used,i.programs_used,i.groups_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SELECTION_BYTE_TABLES").is_some() {
        let i = decoded.bus.rom_resources().unwrap().selection_byte_tables_stats().expect("decoded EUR selection-byte tables");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.threshold_bytes + i.weight_bytes == i.source_bytes && i.threshold_positions_used + i.weight_positions_used > 0, "ordinary scene must consume selection-byte source");
        eprintln!("ROM_DATA_SELECTION_BYTE_TABLES_SCENE frames={frames} bus_reads={} source_bytes={} threshold_bytes={} weight_bytes={} threshold_positions_used={} weight_positions_used={}",i.bus_reads,i.source_bytes,i.threshold_bytes,i.weight_bytes,i.threshold_positions_used,i.weight_positions_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_MISSION_RECORDS70").is_some() {
        let i = decoded.bus.rom_resources().unwrap().mission_records70_stats().expect("decoded EUR mission records70");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && [i.word_bytes, i.packed_bytes, i.operand_bytes, i.inline_bytes, i.byte_list_bytes, i.scaled_bytes, i.scalar_bytes, i.mapped_bytes, i.flag_bytes, i.event_bytes].into_iter().any(|n| n > 0), "ordinary scene must consume a mission-records70 field");
        eprintln!("ROM_DATA_MISSION_RECORDS70_SCENE frames={frames} bus_reads={} source_bytes={} word_bytes={} packed_bytes={} operand_bytes={} inline_bytes={} byte_list_bytes={} scaled_bytes={} scalar_bytes={} mapped_bytes={} flag_bytes={} event_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.word_bytes,i.packed_bytes,i.operand_bytes,i.inline_bytes,i.byte_list_bytes,i.scaled_bytes,i.scalar_bytes,i.mapped_bytes,i.flag_bytes,i.event_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_CONDITIONAL_RECORDS8").is_some() {
        let i = decoded.bus.rom_resources().unwrap().conditional_records8_stats().expect("decoded EUR conditional records8");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && [i.gate_bytes, i.filter_bytes, i.first_key_bytes, i.first_operand_bytes, i.second_key_bytes, i.second_operand_bytes].into_iter().any(|n| n > 0), "ordinary scene must consume a conditional-records8 field");
        eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_SCENE frames={frames} bus_reads={} source_bytes={} gate_bytes={} filter_bytes={} first_key_bytes={} first_operand_bytes={} second_key_bytes={} second_operand_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.gate_bytes,i.filter_bytes,i.first_key_bytes,i.first_operand_bytes,i.second_key_bytes,i.second_operand_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_RECORDS4").is_some() {
        let i = decoded.bus.rom_resources().unwrap().records4_stats().expect("decoded EUR records4");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && [i.field00_bytes, i.field01_bytes, i.field02_bytes, i.field03_bytes].into_iter().any(|n| n > 0), "ordinary scene must consume a records4 field");
        eprintln!("ROM_DATA_RECORDS4_SCENE frames={frames} bus_reads={} source_bytes={} field00_bytes={} field01_bytes={} field02_bytes={} field03_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.field00_bytes,i.field01_bytes,i.field02_bytes,i.field03_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA28").is_some() {
        let i = decoded.bus.rom_resources().unwrap().metadata28_stats().expect("decoded EUR metadata28");
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && i.records_used > 0 && [i.halfword_bytes, i.byte_bytes, i.inline_bytes, i.flag_bytes].into_iter().any(|n| n > 0), "ordinary scene must consume an interpreted metadata28 field");
        eprintln!("ROM_DATA_METADATA28_SCENE frames={frames} bus_reads={} source_bytes={} halfword_bytes={} byte_bytes={} inline_bytes={} flag_bytes={} opaque_bytes={} records_used={}",i.bus_reads,i.source_bytes,i.halfword_bytes,i.byte_bytes,i.inline_bytes,i.flag_bytes,i.opaque_bytes,i.records_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ENCOUNTER_COORDINATES").is_some() {
        let i = decoded.bus.rom_resources().unwrap().encounter_coordinates_stats().expect("decoded EUR encounter coordinates");
        assert!([i.bus_reads, i.source_bytes, i.placement0_bytes, i.x_bytes, i.y_bytes, i.lists_used, i.headers_used].into_iter().all(|n| n > 0), "ordinary battle entry must consume placement0 X/Y coordinate data");
        eprintln!("ROM_DATA_ENCOUNTER_COORDINATES_SCENE frames={frames} bus_reads={} source_bytes={} placement0_bytes={} random_bytes={} placement2_bytes={} default_bytes={} x_bytes={} y_bytes={} lists_used={} headers_used={}",i.bus_reads,i.source_bytes,i.placement0_bytes,i.random_bytes,i.placement2_bytes,i.default_bytes,i.x_bytes,i.y_bytes,i.lists_used,i.headers_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ENCOUNTER_UNIT_TEMPLATES").is_some() {
        let i = decoded.bus.rom_resources().unwrap().encounter_unit_templates_stats().expect("decoded EUR encounter unit templates");
        assert!([i.selector_bytes, i.argument_bytes, i.equipment_bytes, i.auxiliary_bytes, i.ability_bit_bytes, i.optional_bytes, i.flag_bytes, i.control_bytes].into_iter().all(|n| n > 0), "ordinary battle entry must consume all eight observed semantic template classes");
        eprintln!("ROM_DATA_ENCOUNTER_UNIT_TEMPLATES_SCENE frames={frames} bus_reads={} source_bytes={} selector_bytes={} argument_bytes={} equipment_bytes={} auxiliary_bytes={} ability_bit_bytes={} optional_bytes={} flag_bytes={} control_bytes={} opaque_bytes={} records_used={} headers_used={}",i.bus_reads,i.source_bytes,i.selector_bytes,i.argument_bytes,i.equipment_bytes,i.auxiliary_bytes,i.ability_bit_bytes,i.optional_bytes,i.flag_bytes,i.control_bytes,i.opaque_bytes,i.records_used,i.headers_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ENCOUNTER_HEADERS").is_some() {
        let i = decoded.bus.rom_resources().unwrap().encounter_headers_stats().expect("decoded EUR encounter headers");
        assert!([i.unit_count_bytes, i.map_bytes, i.template_pointer_bytes, i.placement_count_bytes, i.position_pointer_bytes, i.control_bytes].into_iter().all(|n| n > 0), "ordinary battle-entry scene must consume all six observed semantic header classes");
        eprintln!("ROM_DATA_ENCOUNTER_HEADERS_SCENE frames={frames} bus_reads={} source_bytes={} unit_count_bytes={} map_bytes={} template_pointer_bytes={} placement_parameter_bytes={} placement_count_bytes={} position_pointer_bytes={} control_bytes={} opaque_bytes={} headers_used={}",i.bus_reads,i.source_bytes,i.unit_count_bytes,i.map_bytes,i.template_pointer_bytes,i.placement_parameter_bytes,i.placement_count_bytes,i.position_pointer_bytes,i.control_bytes,i.opaque_bytes,i.headers_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_WORLD_ROADS").is_some() {
        let i = decoded.bus.rom_resources().unwrap().world_roads_stats().expect("decoded EUR world roads");
        assert!(i.count_bytes > 0 && i.pointer_bytes > 0 && i.sample_bytes > 0, "ordinary scene must consume road counts, pointers and samples");
        eprintln!("ROM_DATA_WORLD_ROADS_SCENE frames={frames} bus_reads={} source_bytes={} count_bytes={} pointer_bytes={} sample_bytes={} opaque_bytes={} paths_used={}",i.bus_reads,i.source_bytes,i.count_bytes,i.pointer_bytes,i.sample_bytes,i.opaque_bytes,i.paths_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_WORLD_NODES").is_some() {
        let i = decoded.bus.rom_resources().unwrap().world_nodes_stats().expect("decoded EUR world nodes");
        assert!(i.coordinate_bytes > 0 && i.path_id_bytes > 0 && i.neighbor_bytes > 0, "ordinary scene must consume world-node coordinates, paths and neighbors");
        eprintln!("ROM_DATA_WORLD_NODES_SCENE frames={frames} bus_reads={} source_bytes={} coordinate_bytes={} path_id_bytes={} neighbor_bytes={} opaque_bytes={} nodes_used={}",i.bus_reads,i.source_bytes,i.coordinate_bytes,i.path_id_bytes,i.neighbor_bytes,i.opaque_bytes,i.nodes_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_WORD_TASK_OBJ").is_some() {
        let i = decoded.bus.rom_resources().unwrap().word_task_obj_stats().expect("decoded u16-task OBJ banks");
        assert!(i.index_bytes > 0 && i.count_bytes > 0 && i.attribute_bytes > 0 && i.sequence_header_bytes > 0 && i.frame_list_bytes > 0 && i.duration_bytes > 0 && i.host_reads > 0, "ordinary scene must consume new u16-task OBJ fields and host lists");
        eprintln!("ROM_DATA_WORD_TASK_OBJ_SCENE frames={frames} bus_reads={} source_bytes={} index_bytes={} count_bytes={} attribute_bytes={} sequence_header_bytes={} frame_list_bytes={} duration_bytes={} host_reads={}",i.bus_reads,i.source_bytes,i.index_bytes,i.count_bytes,i.attribute_bytes,i.sequence_header_bytes,i.frame_list_bytes,i.duration_bytes,i.host_reads);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_TASK_CYCLE_DELTAS").is_some() {
        let i = decoded.bus.rom_resources().unwrap().task_cycle_deltas_stats().expect("decoded signed task-cycle deltas");
        assert!(i.source_bytes > 0 && i.phases_used == 48, "ordinary scene must consume all48 signed cycle rows");
        eprintln!("ROM_DATA_TASK_CYCLE_SCENE frames={frames} bus_reads={} source_bytes={} phases_used={}",i.bus_reads,i.source_bytes,i.phases_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PHASE_ZERO_THRESHOLD").is_some() {
        let i = decoded.bus.rom_resources().unwrap().observed_task_parameters_stats().expect("decoded phase-zero threshold");
        assert!(i.phase_threshold_bytes > 0 && i.vertical_correction_bytes > 0 && i.phases_used == 3 && i.corrections_used == 4, "this scene must consume all three conditional thresholds and four correction rows");
        eprintln!("ROM_DATA_PHASE_ZERO_SCENE frames={frames} bus_reads={} source_bytes={} phase_threshold_bytes={} vertical_correction_bytes={} phases_used={} corrections_used={}",i.bus_reads,i.source_bytes,i.phase_threshold_bytes,i.vertical_correction_bytes,i.phases_used,i.corrections_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_OBSERVED_TASK_PARAMETERS").is_some() {
        let i = decoded.bus.rom_resources().unwrap().observed_task_parameters_stats().expect("decoded observed task parameters");
        assert!(i.phase_threshold_bytes > 0 && i.vertical_correction_bytes > 0 && i.phases_used == 2 && i.corrections_used == 4, "this scene must consume both observed task parameter kinds and all six rows");
        eprintln!("ROM_DATA_OBSERVED_TASK_SCENE frames={frames} bus_reads={} source_bytes={} phase_threshold_bytes={} vertical_correction_bytes={} phases_used={} corrections_used={}",i.bus_reads,i.source_bytes,i.phase_threshold_bytes,i.vertical_correction_bytes,i.phases_used,i.corrections_used);
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UI_PROPERTY_OBJ_LIST").is_some() {
        let used = decoded.bus.rom_resources().unwrap().obj_list_source_used(0x3950a4);
        assert_eq!(used, Some((true, true)), "this scene must consume the new UI property OBJ list through guest and host readers");
        eprintln!("ROM_DATA_UI_PROPERTY_OBJ_SCENE frames={frames} source=003950a4 guest=true host=true");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_OBJ_ANIMATION").is_some() {
        let a = decoded.bus.rom_resources().unwrap().obj_animation_stats().expect("decoded EUR OBJ animation banks");
        eprintln!("ROM_DATA_OBJ_ANIMATION_SCENE frames={frames} bus_reads={} source_bytes={} index_bytes={} count_bytes={} attribute_bytes={} sequence_header_bytes={} frame_list_bytes={} duration_bytes={} host_reads={}",
            a.bus_reads, a.source_bytes, a.index_bytes, a.count_bytes, a.attribute_bytes, a.sequence_header_bytes, a.frame_list_bytes, a.duration_bytes, a.host_reads);
        assert!(a.index_bytes > 0 && a.count_bytes > 0 && a.attribute_bytes > 0 && a.frame_list_bytes > 0 && a.duration_bytes > 0 && a.host_reads > 0,
            "this scene must consume typed OBJ animation indices/list fields/frame selectors/durations and host lists");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ZERO_COUNT_OBJ").is_some() {
        let a = decoded.bus.rom_resources().unwrap().obj_animation_stats().unwrap();
        eprintln!("ROM_DATA_ZERO_COUNT_SCENE frames={frames} selector_bytes={} host_reads={}",a.zero_count_selector_bytes,a.zero_count_host_reads);
        assert!(a.zero_count_selector_bytes > 0 && a.zero_count_host_reads > 0,
            "ordinary scene must use the newly rooted zero-count selectors in guest and host paths");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_LOCALIZED_OBJ").is_some() {
        let a = decoded.bus.rom_resources().unwrap().obj_animation_stats().unwrap();
        eprintln!("ROM_DATA_LOCALIZED_OBJ_SCENE frames={frames} selector_bytes={} source_bytes={} host_reads={}",
            a.localized_selector_bytes,a.localized_source_bytes,a.localized_host_reads);
        assert!(a.localized_selector_bytes>0 && a.localized_source_bytes>0 && a.localized_host_reads>0,
            "ordinary scene must use the new language-selected OBJ tables, bank and host lists");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_OBJ_GRAPHICS").is_some() {
        let g = decoded.bus.rom_resources().unwrap().obj_graphics_stats().expect("decoded EUR OBJ custom graphics");
        eprintln!("ROM_DATA_OBJ_GRAPHICS_SCENE frames={frames} bus_reads={} source_bytes={} decoded_literal_bytes={} selector_bytes={}",g.bus_reads,g.source_bytes,g.decoded_literal_bytes,g.selector_bytes);
        assert!(g.source_bytes > 0 && g.decoded_literal_bytes > 0 && g.selector_bytes > 0,
            "ordinary scene must consume custom OBJ bank literals and language selectors");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UI_PALETTES").is_some() {
        let p = decoded.bus.rom_resources().unwrap().ui_palette_stats().expect("decoded EUR raw UI palettes");
        eprintln!("ROM_DATA_UI_PALETTE_SCENE frames={frames} bus_reads={} source_bytes={}",p.bus_reads,p.source_bytes);
        assert!(p.source_bytes > 0,"ordinary scene must read new file-derived UI palette colors");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UI_GRAPHICS").is_some() {
        let owner = decoded.bus.rom_resources().unwrap();
        let g = owner.ui_graphics_stats().expect("decoded EUR custom UI graphics");
        eprintln!("ROM_DATA_UI_GRAPHICS_SCENE frames={frames} bus_reads={} source_bytes={} decoded_literal_bytes={} sources={:x?}",g.bus_reads,g.source_bytes,g.decoded_literal_bytes,owner.ui_graphics_used_sources());
        assert!(g.source_bytes>0 && g.decoded_literal_bytes>0,"ordinary scene must consume newly migrated custom UI literals");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA16").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata16_stats().expect("decoded EUR metadata16");
        eprintln!("ROM_DATA_METADATA16_SCENE frames={frames} bus_reads={} source_bytes={} alias_bytes={} display_bytes={} flag_bytes={} value_bytes={} opaque_bytes={}",i.bus_reads,i.source_bytes,i.alias_bytes,i.display_bytes,i.flag_bytes,i.value_bytes,i.opaque_bytes);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated metadata16fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA4").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata4_stats().expect("decoded EUR metadata4");
        eprintln!("ROM_DATA_METADATA4_SCENE frames={frames} bus_reads={} source_bytes={} selector_bytes={} value_bytes={}",i.bus_reads,i.source_bytes,i.selector_bytes,i.value_bytes);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated metadata4 fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA2").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata2_stats().expect("decoded EUR metadata2");
        eprintln!("ROM_DATA_METADATA2_SCENE frames={frames} bus_reads={} source_bytes={} key_bytes={} value_bytes={}",i.bus_reads,i.source_bytes,i.key_bytes,i.value_bytes);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated metadata2 fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_DEFAULT48").is_some() {
        let i=decoded.bus.rom_resources().unwrap().default48_stats().expect("decoded EUR default48");
        eprintln!("ROM_DATA_DEFAULT48_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} argument_bytes={} halfword_bytes={} flag_bytes={} tail_bytes={}",i.bus_reads,i.source_bytes,i.header_bytes,i.argument_bytes,i.halfword_bytes,i.flag_bytes,i.tail_bytes);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated default48 fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_CONSTRUCTOR_ALTERNATES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().constructor_alternates_stats().expect("decoded EUR constructor_alternates");
        eprintln!("ROM_DATA_CONSTRUCTOR_ALTERNATES_SCENE frames={frames} bus_reads={} source_bytes={} pointer_bytes={} record_mask={} header_bytes={} argument_bytes={} halfword_bytes={} flag_bytes={} tail_bytes={}",i.bus_reads,i.source_bytes,i.pointer_bytes,i.record_mask,i.header_bytes,i.argument_bytes,i.halfword_bytes,i.flag_bytes,i.tail_bytes);
        assert!(i.source_bytes>i.pointer_bytes && i.pointer_bytes>0 && i.record_mask>0,"ordinary scene must consume newly migrated constructor_alternates fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA14").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata14_stats().expect("decoded EUR metadata14");
        eprintln!("ROM_DATA_METADATA14_SCENE frames={frames} bus_reads={} source_bytes={} records_used={} byte_fields={} halfword_bytes={} packed_bytes={} tail_bytes={}",i.bus_reads,i.source_bytes,i.records_used(),i.byte_fields,i.halfword_bytes,i.packed_bytes,i.tail_bytes);
        assert!(i.source_bytes>0 && i.records_used()>0,"ordinary scene must consume newly migrated metadata14 fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA12").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata12_stats().expect("decoded EUR metadata12");
        eprintln!("ROM_DATA_METADATA12_SCENE frames={frames} bus_reads={} source_bytes={} records_used={} primary_bytes={} secondary_bytes={}",i.bus_reads,i.source_bytes,i.records_used(),i.primary_bytes,i.secondary_bytes);
        assert!(i.source_bytes>0 && i.records_used()>0,"ordinary scene must consume newly migrated metadata12 fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_GLOBAL_PAIR_FLAGS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().global_pair_flags_stats().expect("decoded EUR global_pair_flags");
        eprintln!("ROM_DATA_GLOBAL_PAIR_FLAGS_SCENE frames={frames} bus_reads={} source_bytes={} bytes_used={} byte_mask={}",i.bus_reads,i.source_bytes,i.bytes_used(),i.byte_mask);
        assert!(i.source_bytes>0 && i.bytes_used()>0,"ordinary scene must consume newly migrated global_pair_flags fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_EVALUATION_PARAMETERS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().evaluation_parameters_stats().expect("decoded EUR evaluation_parameters");
        eprintln!("ROM_DATA_EVALUATION_PARAMETERS_SCENE frames={frames} bus_reads={} source_bytes={} divisor_bytes={} weight_bytes={} bonus_bytes={} score_opaque_bytes={} threshold_bytes={} threshold_opaque_bytes={} threshold_mask={}",i.bus_reads,i.source_bytes,i.divisor_bytes,i.weight_bytes,i.bonus_bytes,i.score_opaque_bytes,i.threshold_bytes,i.threshold_opaque_bytes,i.threshold_mask);
        assert!(i.source_bytes>0 && i.divisor_bytes+i.weight_bytes+i.bonus_bytes+i.threshold_bytes>0,"ordinary scene must consume newly migrated evaluation_parameters fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PROGRESSION_REWARDS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().progression_rewards_stats().expect("decoded EUR progression_rewards");
        eprintln!("ROM_DATA_PROGRESSION_REWARDS_SCENE frames={frames} bus_reads={} source_bytes={} delta_bytes={} discount_bytes={} opaque_bytes={} record_mask_lo={} record_mask_hi={}",i.bus_reads,i.source_bytes,i.delta_bytes,i.discount_bytes,i.opaque_bytes,i.record_mask[0],i.record_mask[1]);
        assert!(i.source_bytes>0 && i.delta_bytes+i.discount_bytes>0,"ordinary scene must consume newly migrated progression_rewards fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PROGRESSION_GATES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().progression_gates_stats().expect("decoded EUR progression_gates");
        eprintln!("ROM_DATA_PROGRESSION_GATES_SCENE frames={frames} bus_reads={} source_bytes={} requirement_bytes={} return_bytes={} output_bytes={} unused_bytes={} record_mask={}",i.bus_reads,i.source_bytes,i.requirement_bytes,i.return_bytes,i.output_bytes,i.unused_bytes,i.record_mask);
        assert!(i.source_bytes>0 && i.requirement_bytes+i.return_bytes+i.output_bytes>0,"ordinary scene must consume newly migrated progression_gates fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PROGRESSION_SELECTION_GATES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().progression_selection_gates_stats().expect("decoded EUR selection gates");
        eprintln!("ROM_DATA_PROGRESSION_SELECTION_GATES_SCENE frames={frames} bus_reads={} source_bytes={} selector_bytes={} minimum01_bytes={} minimum02_bytes={} minimum04_bytes={} any_minima_bytes={} flag_bytes={} record_mask={}",i.bus_reads,i.source_bytes,i.selector_bytes,i.minimum01_bytes,i.minimum02_bytes,i.minimum04_bytes,i.any_minima_bytes,i.flag_bytes,i.record_mask);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated selection gates");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AGGREGATE_FLAG_KEYS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().aggregate_flag_keys_stats().expect("decoded EUR aggregate keys");
        eprintln!("ROM_DATA_AGGREGATE_FLAG_KEYS_SCENE frames={frames} bus_reads={} source_bytes={} key_bytes={} terminator_bytes={} entry_mask={:?}",i.bus_reads,i.source_bytes,i.key_bytes,i.terminator_bytes,i.entry_mask);
        assert!(i.key_bytes>0 && i.terminator_bytes>0,"ordinary scene must consume aggregate keys and terminal");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_TRADE_PARAMETERS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().trade_parameters_stats().expect("decoded EUR trade parameters");
        eprintln!("ROM_DATA_TRADE_PARAMETERS_SCENE frames={frames} bus_reads={} source_bytes={} opaque_bytes={} markup_low_bytes={} shared41_bytes={} discount_bytes={} record_mask={}",i.bus_reads,i.source_bytes,i.opaque_bytes,i.markup_low_bytes,i.shared41_bytes,i.discount_bytes,i.record_mask);
        assert!(i.source_bytes>0,"ordinary scene must consume newly owned trade parameters");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ITEM_EFFECT_PARAMETERS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().item_effect_parameters_stats().expect("decoded EUR item effect parameters");
        eprintln!("ROM_DATA_ITEM_EFFECT_PARAMETERS_SCENE frames={frames} bus_reads={} source_bytes={} coefficient_bytes={} shared05_bytes={} shared06_bytes={} record_mask={}",i.bus_reads,i.source_bytes,i.coefficient_bytes,i.shared05_bytes,i.shared06_bytes,i.record_mask);
        assert!(i.source_bytes>0,"ordinary scene must consume newly owned item effect parameters");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AUDIO_DIRECTORY").is_some() {
        let i=decoded.bus.rom_resources().unwrap().audio_directory_stats().expect("decoded EUR M4A directory");
        eprintln!("ROM_DATA_AUDIO_DIRECTORY_SCENE frames={frames} bus_reads={} source_bytes={} player_bytes={} directory_bytes={} header_bytes={} track_pointer_bytes={} player_mask={}",i.bus_reads,i.source_bytes,i.player_bytes,i.directory_bytes,i.header_bytes,i.track_pointer_bytes,i.player_mask);
        assert!(i.source_bytes>0,"ordinary scene must consume newly owned audio directory");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AUDIO_TONES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().audio_tones_stats().expect("decoded EUR M4A tones and samples");
        eprintln!("ROM_DATA_AUDIO_TONES_SCENE frames={frames} bus_reads={} source_bytes={} tone_bytes={} keymap_bytes={} wave_header_bytes={} pcm_bytes={} endpoint_bytes={} cgb_bytes={} host_reads={} host_source_bytes={} host_wave_header_bytes={} host_pcm_bytes={}",i.bus_reads,i.source_bytes,i.tone_bytes,i.keymap_bytes,i.wave_header_bytes,i.pcm_bytes,i.endpoint_bytes,i.cgb_bytes,i.host_reads,i.host_source_bytes,i.host_wave_header_bytes,i.host_pcm_bytes);
        assert!(i.source_bytes>0 && i.host_pcm_bytes>0,"ordinary scene must consume typed audio through guest and shadow sampler");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AUDIO_SEQUENCES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().audio_sequences_stats().expect("decoded EUR M4A sequence instructions");
        eprintln!("ROM_DATA_AUDIO_SEQUENCES_SCENE frames={frames} bus_reads={} source_bytes={} opcode_bytes={} operand_bytes={} clock_bytes={} command_bytes={} extended_bytes={} instructions_used={} clock_mask={} command_mask={} extended_mask={}",i.bus_reads,i.source_bytes,i.opcode_bytes,i.operand_bytes,i.clock_bytes,i.command_bytes,i.extended_bytes,i.instructions_used,i.clock_mask,i.command_mask,i.extended_mask);
        assert!(i.opcode_bytes>0 && i.operand_bytes>0 && i.clock_bytes>0,"ordinary scene must consume typed sequence opcodes, operands and clocks");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AUDIO_DRIVER").is_some() {
        let i=decoded.bus.rom_resources().unwrap().audio_driver_stats().expect("decoded EUR M4A driver tables");
        eprintln!("ROM_DATA_AUDIO_DRIVER_SCENE frames={frames} bus_reads={} source_bytes={} table_bytes={:?} entries_used={:?}",i.bus_reads,i.source_bytes,i.table_bytes,i.entries_used);
        assert!(i.table_bytes[0]>0 && i.table_bytes[1]>0,"ordinary scene must consume typed PCM key and base tables");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PALETTE_EFFECT_DISPATCH").is_some() {
        let i=decoded.bus.rom_resources().unwrap().palette_effect_dispatch_stats().expect("decoded EUR palette effect dispatch");
        eprintln!("ROM_DATA_PALETTE_EFFECT_DISPATCH_SCENE frames={frames} bus_reads={} source_bytes={} callback_bytes={} null_bytes={} slot_mask={}",i.bus_reads,i.source_bytes,i.callback_bytes,i.null_bytes,i.slot_mask);
        assert!(i.callback_bytes>0,"ordinary scene must consume typed palette effect callback slots");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_PALETTE_PROGRAMS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().palette_programs_stats().expect("decoded EUR palette programs");
        eprintln!("ROM_DATA_PALETTE_PROGRAMS_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} command_bytes={} parameter_bytes={} producer_bytes={} programs_used={} producer_tables_used={}",i.bus_reads,i.source_bytes,i.header_bytes,i.command_bytes,i.parameter_bytes,i.producer_bytes,i.programs_used,i.producer_tables_used);
        assert!(i.header_bytes>0 && i.command_bytes>0 && i.parameter_bytes>0,"ordinary scene must consume typed palette programs and parameters");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_BLEND_PROGRAMS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().blend_programs_stats().expect("decoded EUR blend programs");
        eprintln!("ROM_DATA_BLEND_PROGRAMS_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} command_bytes={} producer_bytes={} programs_used={} producer_tables_used={}",i.bus_reads,i.source_bytes,i.header_bytes,i.command_bytes,i.producer_bytes,i.programs_used,i.producer_tables_used);
        assert!(i.header_bytes>0 && i.command_bytes>0,"ordinary scene must consume typed blend programs");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_TASK_DESCRIPTORS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().task_descriptors_stats().expect("decoded EUR task descriptors");
        eprintln!("ROM_DATA_TASK_DESCRIPTORS_SCENE frames={frames} bus_reads={} source_bytes={} callback_bytes={} context_size_bytes={} metadata_bytes={} records_used={} catalogs_used={}",i.bus_reads,i.source_bytes,i.callback_bytes,i.context_size_bytes,i.metadata_bytes,i.records_used,i.catalogs_used);
        assert!(i.callback_bytes>0 && i.context_size_bytes>0,"ordinary scene must consume typed task descriptors");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_EFFECT_EVENT_DIRECTORIES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().effect_event_directories_stats().expect("decoded EUR effect/event directories");
        eprintln!("ROM_DATA_EFFECT_EVENT_DIRECTORIES_SCENE frames={frames} bus_reads={} source_bytes={} event_bytes={} selector_bytes={} dispatch_bytes={} event_records_used={} selectors_used={} dispatch_tables_used={}",i.bus_reads,i.source_bytes,i.event_bytes,i.selector_bytes,i.dispatch_bytes,i.event_records_used,i.selectors_used,i.dispatch_tables_used);
        assert!((i.event_bytes>0 || i.selector_bytes>0),"ordinary scene must consume typed event records or effect selectors");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SCRIPT_COMMANDS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().script_commands_stats().expect("decoded EUR script commands");
        eprintln!("ROM_DATA_SCRIPT_COMMANDS_SCENE frames={} bus_reads={} source_bytes={} advance_bytes={} callback_bytes={} commands_used={}",frames,i.bus_reads,i.source_bytes,i.advance_bytes,i.callback_bytes,i.commands_used);
        assert!(i.advance_bytes>0 && i.callback_bytes>0, "scene must consume typed script advances and callbacks");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SCRIPT_ARCHIVE").is_some() {
        let i=decoded.bus.rom_resources().unwrap().script_archive_stats().expect("decoded EUR script archive");
        eprintln!("ROM_DATA_SCRIPT_ARCHIVE_SCENE frames={} bus_reads={} source_bytes={} index_bytes={} opcode_bytes={} parameter_bytes={} key_bytes={} padding_bytes={} banks_used={} programs_used={} key_records_used={} padding_spans_used={}",frames,i.bus_reads,i.source_bytes,i.index_bytes,i.opcode_bytes,i.parameter_bytes,i.key_bytes,i.padding_bytes,i.banks_used,i.programs_used,i.key_records_used,i.padding_spans_used);
        assert!(i.opcode_bytes>0 && i.parameter_bytes>0, "scene must consume typed script opcodes and parameters");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SCRIPT_TEXT").is_some() {
        let i=decoded.bus.rom_resources().unwrap().script_text_stats().expect("decoded EUR script text");
        eprintln!("ROM_DATA_SCRIPT_TEXT_SCENE frames={} bus_reads={} source_bytes={} top_index_bytes={} variant_index_bytes={} bank_index_bytes={} header_bytes={} encoded_control_bytes={} glyph_bytes={} command_bytes={} operand_bytes={} terminator_bytes={} decoded_tail_bytes={} alignment_bytes={} decoded_program_bytes={} languages_used={} banks_used={} programs_used={} variant_records_used={} alignment_spans_used={}",frames,i.bus_reads,i.source_bytes,i.top_index_bytes,i.variant_index_bytes,i.bank_index_bytes,i.header_bytes,i.encoded_control_bytes,i.glyph_bytes,i.command_bytes,i.operand_bytes,i.terminator_bytes,i.decoded_tail_bytes,i.alignment_bytes,i.decoded_program_bytes,i.languages_used,i.banks_used,i.programs_used,i.variant_records_used,i.alignment_spans_used);
        assert!(i.header_bytes>0 && i.decoded_program_bytes>0, "scene must consume typed script text headers and program bytes");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SCRIPT_COORDINATES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().script_coordinates_stats().expect("decoded EUR coordinate paths");
        eprintln!("ROM_DATA_SCRIPT_COORDINATES_SCENE frames={} bus_reads={} source_bytes={} index_bytes={} count_bytes={} coordinate_bytes={} alignment_bytes={} paths_used={} alignment_spans_used={}",frames,i.bus_reads,i.source_bytes,i.index_bytes,i.count_bytes,i.coordinate_bytes,i.alignment_bytes,i.paths_used,i.alignment_spans_used);
        assert!(i.index_bytes>0 && i.count_bytes>0 && i.coordinate_bytes>0, "scene must consume typed coordinate directory, count and XY fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_AUXILIARY_SCRIPTS").is_some() {
        let resources=decoded.bus.rom_resources().unwrap();
        let c=resources.auxiliary_commands_stats().expect("decoded EUR auxiliary commands");
        let a=resources.auxiliary_archive_stats().expect("decoded EUR auxiliary archive");
        eprintln!("ROM_DATA_AUXILIARY_SCENE frames={} command_source_bytes={} archive_source_bytes={} advance_bytes={} callback_bytes={} index_bytes={} opcode_bytes={} parameter_bytes={} alignment_bytes={}",frames,c.source_bytes,a.source_bytes,c.advance_bytes,c.callback_bytes,a.index_bytes,a.opcode_bytes,a.parameter_bytes,a.alignment_bytes);
        assert!(c.source_bytes>0 && a.source_bytes>0,"scene must consume both typed auxiliary resources");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UI_CHOICE_LISTS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().ui_choice_lists_stats().expect("decoded EUR UI choice lists");
        eprintln!("ROM_DATA_UI_CHOICE_LISTS_SCENE frames={} bus_reads={} source_bytes={} identifier_bytes={} coordinate_bytes={} opaque_bytes={} navigation_bytes={} lists_used={}",frames,i.bus_reads,i.source_bytes,i.identifier_bytes,i.coordinate_bytes,i.opaque_bytes,i.navigation_bytes,i.lists_used);
        assert!(i.source_bytes>0,"scene must consume typed UI choice list fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UNIT_OBJ_EQUIPMENT").is_some() {
        let i=decoded.bus.rom_resources().unwrap().unit_obj_equipment_stats().expect("decoded EUR equipment OBJ");
        eprintln!("ROM_DATA_UNIT_OBJ_EQUIPMENT_SCENE frames={} bus_reads={} source_bytes={} directory_bytes={} index_bytes={} sequence_header_bytes={} frame_bytes={} object_bytes={} pixel_bytes={} sequences_used={}",frames,i.bus_reads,i.source_bytes,i.directory_bytes,i.index_bytes,i.sequence_header_bytes,i.frame_bytes,i.object_bytes,i.pixel_bytes,i.sequences_used);
        assert!(i.pixel_bytes>0 && i.sequences_used>0,"ordinary scene must consume newly decoded equipment OBJ pixels and sequences");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UNIT_OBJ_NORMAL_BATTLE").is_some() {
        let i=decoded.bus.rom_resources().unwrap().unit_obj_normal_battle_stats().expect("decoded EUR normal battle OBJ");
        eprintln!("ROM_DATA_UNIT_OBJ_NORMAL_BATTLE_SCENE frames={} bus_reads={} source_bytes={} directory_bytes={} index_bytes={} sequence_header_bytes={} frame_bytes={} object_bytes={} pixel_bytes={} sequences_used={}",frames,i.bus_reads,i.source_bytes,i.directory_bytes,i.index_bytes,i.sequence_header_bytes,i.frame_bytes,i.object_bytes,i.pixel_bytes,i.sequences_used);
        assert!(i.pixel_bytes>0,"ordinary scene must consume newly decoded normal battle OBJ pixels");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UNIT_OBJ_PROPERTY_WORDS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().unit_obj_property_words_stats().expect("decoded EUR actor property words");
        eprintln!("ROM_DATA_UNIT_OBJ_PROPERTY_WORDS_SCENE frames={} bus_reads={} source_bytes={} words_used={}",frames,i.bus_reads,i.source_bytes,i.words_used);
        assert!(i.source_bytes>0 && i.words_used>0,"ordinary scene must consume newly decoded actor property words");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UNIT_OBJ_BYTE_PROPERTY_WORDS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().unit_obj_byte_property_words_stats().expect("decoded EUR UI byte property words");
        eprintln!("ROM_DATA_UNIT_OBJ_BYTE_PROPERTY_WORDS_SCENE frames={} bus_reads={} source_bytes={} words_used={}",frames,i.bus_reads,i.source_bytes,i.words_used);
        assert!(i.source_bytes>0 && i.words_used>0,"ordinary scene must consume newly decoded UI byte property words");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_SLOT_JOB_LOOKUP").is_some() {
        let i=decoded.bus.rom_resources().unwrap().slot_job_lookup_stats().expect("decoded EUR slot-job lookup");
        eprintln!("ROM_DATA_SLOT_JOB_LOOKUP_SCENE frames={} bus_reads={} source_bytes={} slots_used={}",frames,i.bus_reads,i.source_bytes,i.slots_used);
        assert!(i.source_bytes>0 && i.slots_used>0,"ordinary scene must consume newly decoded slot-job references");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UNIT_OBJ_INITIAL_POSE").is_some() {
        let i=decoded.bus.rom_resources().unwrap().unit_obj_initial_pose_stats().expect("decoded EUR unit OBJ initial pose");
        eprintln!("ROM_DATA_UNIT_OBJ_INITIAL_POSE_SCENE frames={} bus_reads={} source_bytes={} directory_bytes={} index_bytes={} sequence_header_bytes={} frame_bytes={} object_bytes={} pixel_bytes={} sequences_used={}",frames,i.bus_reads,i.source_bytes,i.directory_bytes,i.index_bytes,i.sequence_header_bytes,i.frame_bytes,i.object_bytes,i.pixel_bytes,i.sequences_used);
        assert!(i.pixel_bytes>0,"scene must consume typed decoded unit OBJ pixels");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_COLOR_MODE_PALETTES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().color_mode_palettes_stats().expect("decoded EUR Color-Mode palettes");
        eprintln!("ROM_DATA_COLOR_MODE_PALETTES_SCENE frames={} bus_reads={} source_bytes={} unit_base_bytes={} companion_bytes={} supplemental_bytes={} shared_color_bytes={} special_unit_bytes={} selector_bytes={} fragments_used={}",frames,i.bus_reads,i.source_bytes,i.unit_base_bytes,i.companion_bytes,i.supplemental_bytes,i.shared_color_bytes,i.special_unit_bytes,i.selector_bytes,i.fragments_used);
        assert!(i.source_bytes>0 && i.unit_base_bytes+i.companion_bytes+i.supplemental_bytes+i.shared_color_bytes+i.special_unit_bytes>0,"ordinary scene must consume typed Color-Mode palette colors");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_MAP_RASTER_PALETTES").is_some() {
        let i=decoded.bus.rom_resources().unwrap().map_raster_palettes_stats().expect("decoded EUR map raster palettes");
        eprintln!("ROM_DATA_MAP_RASTER_PALETTES_SCENE frames={} bus_reads={} source_bytes={} index_bytes={} header_bytes={} color_bytes={} programs_used={}",frames,i.bus_reads,i.source_bytes,i.index_bytes,i.header_bytes,i.color_bytes,i.programs_used);
        assert!(i.source_bytes>0,"scene must consume typed map raster palette fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_METADATA52").is_some() {
        let i=decoded.bus.rom_resources().unwrap().metadata52_stats().expect("decoded EUR metadata52");
        eprintln!("ROM_DATA_METADATA52_SCENE frames={frames} bus_reads={} source_bytes={} header_bytes={} routing_bytes={} packed_bytes={} tail_bytes={} opaque_bytes={}",i.bus_reads,i.source_bytes,i.header_bytes,i.routing_bytes,i.packed_bytes,i.tail_bytes,i.opaque_bytes);
        assert!(i.source_bytes>0,"ordinary scene must consume newly migrated metadata52fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ITEM_LINKS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().item_link_stats().expect("decoded EUR item links");
        eprintln!("ROM_DATA_ITEM_LINK_SCENE frames={frames} bus_reads={} source_bytes={} count_bytes={} selector_bytes={} value_bytes={} opaque_bytes={}",i.bus_reads,i.source_bytes,i.count_bytes,i.selector_bytes,i.value_bytes,i.opaque_bytes);
        assert!(i.source_bytes>0 && i.count_bytes>0 && i.selector_bytes>0 && i.value_bytes>0,"ordinary scene must consume newly migrated item-link fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_ITEMS").is_some() {
        let i=decoded.bus.rom_resources().unwrap().item_stats().expect("decoded EUR item records");
        eprintln!("ROM_DATA_ITEM_SCENE frames={frames} bus_reads={} source_bytes={} halfword_bytes={} signed_bytes={} other_bytes={}",i.bus_reads,i.source_bytes,i.halfword_bytes,i.signed_bytes,i.other_bytes);
        assert!(i.source_bytes>0,"ordinary scene must read new file-derived item fields");
    }
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_UI_GRAPHICS_SELECTOR").is_some() {
        let g=decoded.bus.rom_resources().unwrap().ui_graphics_stats().expect("decoded EUR localized UI graphics");
        eprintln!("ROM_DATA_UI_GRAPHICS_SELECTOR_SCENE frames={frames} selector_bytes={} localized_source_bytes={} localized_literal_bytes={}",g.selector_bytes,g.localized_source_bytes,g.localized_literal_bytes);
        assert!(g.selector_bytes>0,"ordinary scene must read a new file-derived UI language pointer");
    }
    if let Ok(mask) = std::env::var("FFTA_STATE_TEST_REQUIRE_SOURCE_MASK") {
        let required: u8 = mask.parse().unwrap();
        assert_eq!(
            stats.bus_source_mask & required,
            required,
            "required source ranges did not execute"
        );
    }
    // A scene can use typed fields through guest bus reads without calling the
    // original OBJ-dimension host fast path. Family-specific requirements above
    // still verify consumption of the resource selected for this scene.
    assert!(
        stats.bus_reads + stats.host_rom_reads + stats.host_ram_reads > 0,
        "typed bus or host consumers must execute"
    );
    eprintln!("ROM_DATA_SCENE frames={frames} first={first} native={} fallback={} full_states={frames} audio={audio:?} bus_reads={} host_rom_reads={} host_ram_reads={} dynamic_ram_reads={} bus_source_mask={}", dispatch[0], dispatch[1], stats.bus_reads, stats.host_rom_reads, stats.host_ram_reads, stats.dynamic_ram_reads, stats.bus_source_mask);
    eprintln!(
        "ROM_DATA_BG_SCENE frames={frames} bus_reads={} projected_bytes={}",
        bg.bus_reads, bg.projected_bytes
    );
}

fn clear_audio(machine: &mut gba_core::Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.hle_tap.clear();
    machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
}

#[test]
#[ignore = "requires the exact private USA ROM; follows clean-boot demo input to the original copier"]
fn usa_copied_obj_initializer_from_clean_boot_uses_decoded_source() {
    crate::configure_headless_widescreen();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").expect("USA ROM fixture")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::SHA256);
    // Independently located USA literals and function entry. No EUR offset
    // arithmetic or copied guest instructions are used to construct this test.
    const ENTRY: u32 = 0x0808_f74c;
    const START: usize = 0xa38adc;
    const END: usize = 0xa3991c;
    const ALIAS: usize = 0xa398dc;
    const DEST: usize = 0x5f28;
    let word = |offset| u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap());
    assert_eq!(word(0x8f764), 0x0800_0000 + END as u32);
    assert_eq!(word(0x8f768), 0x0800_0000 + START as u32);
    assert_eq!(word(0x8f76c), 0x0300_0000 + DEST as u32);
    assert_eq!(DEST + ALIAS - START, 0x6d28);

    let mut source = Extraction::from_rom(&rom).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let dataset = directory.path().join("decoded");
    extract(&dataset, &mut source).unwrap();
    let mut decoded = make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &dataset).unwrap();
    let max_frames: u64 = std::env::var("FFTA_STATE_TEST_FRAMES")
        .unwrap_or_else(|_| "12000".into())
        .parse()
        .unwrap();
    assert!((1..=30000).contains(&max_frames));
    for frame in 0..max_frames {
        decoded.bus.keys = crate::demo_keys(frame);
        decoded.bus.frame_ready = false;
        for _ in 0..FRAME_STEP_GUARD {
            if decoded.cpu.regs[15] == ENTRY {
                assert!(decoded.cpu.thumb());
                let checkpoint = decoded.save_state().unwrap();
                let entered_frame = decoded.bus.frames;
                let entered_clock = decoded.bus.clock;
                let return_pc = decoded.cpu.regs[14] & !1;
                let before = decoded.bus.rom_resources().unwrap().stats();
                let mut legacy = make_machine(rom.clone(), None);
                legacy.load_state(&checkpoint).unwrap();
                assert_eq!(legacy.save_state().unwrap(), checkpoint);
                let mut count = 0;
                let mut read_pc = None;
                while decoded.cpu.regs[15] != return_pc || count == 0 {
                    assert!(count < 10000, "original initializer did not return");
                    let pc = decoded.cpu.regs[15];
                    let reads = decoded.bus.rom_resources().unwrap().stats().bus_reads;
                    legacy.step();
                    decoded.step();
                    assert!(
                        legacy.save_state().unwrap() == decoded.save_state().unwrap(),
                        "initializer full state at step {count}/PC {pc:08x}"
                    );
                    if decoded.bus.rom_resources().unwrap().stats().bus_reads > reads {
                        assert!(
                            read_pc.replace(pc).is_none(),
                            "unexpected multiple copy instructions"
                        );
                    }
                    count += 1;
                }
                assert_eq!(
                    &decoded.bus.iwram[DEST..DEST + END - START],
                    &rom[START..END]
                );
                assert_eq!(
                    &decoded.bus.iwram[0x6d28..0x6d58],
                    &source.dimensions.source_bytes()
                );
                let after = decoded.bus.rom_resources().unwrap().stats();
                assert_eq!(after.bus_reads - before.bus_reads, 24);
                assert_eq!(after.bus_source_mask & 2, 2);
                let read_pc = read_pc.expect("original copier must read the file-backed alias");

                // A separate provenance probe poisons only the old raw table
                // AFTER authenticated activation. Original code, other source
                // bytes and the caller's real checkpoint are left intact.
                let mut file_only = make_machine(rom.clone(), None);
                attach_from_directory(&mut file_only, &dataset).unwrap();
                file_only.load_state(&checkpoint).unwrap();
                file_only.bus.rom[ALIAS..ALIAS + 48].fill(0xa5);
                for _ in 0..count {
                    file_only.step();
                }
                assert_eq!(
                    &file_only.bus.iwram[DEST..DEST + END - START],
                    &rom[START..END]
                );
                assert_eq!(file_only.bus.rom_resources().unwrap().stats().bus_reads, 24);
                assert_eq!(file_only.cpu.regs[15], return_pc);
                // Restore the test poison only after the guest returned, so
                // snapshot ROM identity can be compared without masking state.
                file_only.bus.rom[ALIAS..ALIAS + 48].copy_from_slice(&rom[ALIAS..ALIAS + 48]);
                assert!(file_only.save_state().unwrap() == legacy.save_state().unwrap());
                eprintln!("ROM_DATA_USA_INITIALIZER frame={entered_frame} clock={entered_clock} entry={ENTRY:08x} return={return_pc:08x} read_pc={read_pc:08x} steps={count} copied_bytes={} table_bytes=48 decoded_reads=24 poisoned_rom_probe=true full_states={count}", END-START);
                return;
            }
            decoded.step();
            if decoded.bus.frame_ready {
                break;
            }
        }
        assert_eq!(decoded.bus.frames, frame + 1, "incomplete boot frame");
        clear_audio(&mut decoded);
        if frame % 500 == 499 {
            eprintln!(
                "ROM_DATA_USA_BOOT frames={} pc={:08x}",
                decoded.bus.frames, decoded.cpu.regs[15]
            );
        }
    }
    panic!("original USA initializer was not reached in {max_frames} clean-boot frames");
}
