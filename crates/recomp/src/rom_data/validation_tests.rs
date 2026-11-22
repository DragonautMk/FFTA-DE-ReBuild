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
    let poison_observed_lz = std::env::var_os("FFTA_STATE_TEST_POISON_OBSERVED_LZ").is_some();
    let poison_soil_lz = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_LZ").is_some();
    let poison_firaga_lz = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_LZ").is_some();
    let poison_soil_literals = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_LITERALS").is_some();
    let poison_soil_146_literals = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_146_LITERALS").is_some();
    let poison_firaga_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_LITERALS").is_some();
    let poison_firaga_2d2e_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_2D2E_LITERALS").is_some();
    let poison_firaga_eb83_ebc_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_EB83_EBC_LITERALS").is_some();
    let poison_firaga_code_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_CODE_LITERALS").is_some();
    let poison_firaga_mixed_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_MIXED_LITERALS").is_some();
    let poison_firaga_residual_literals = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_RESIDUAL_LITERALS").is_some();
    let poison_constructor64 = std::env::var_os("FFTA_STATE_TEST_POISON_CONSTRUCTOR64").is_some();
    let poison_observed_tilemap = std::env::var_os("FFTA_STATE_TEST_POISON_OBSERVED_TILEMAP").is_some();
    let poison_tilemap_rectangles = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_RECTANGLES").is_some();
    let poison_tilemap_alternates = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_ALTERNATES").is_some();
    let poison_tilemap_roots = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_ROOTS").is_some();
    let poison_tilemap_list_row = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_LIST_ROW").is_some();
    let poison_compact_records = std::env::var_os("FFTA_STATE_TEST_POISON_COMPACT_RECORDS").is_some();
    let poison_red_spring_sounds = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_SOUNDS").is_some();
    let poison_iwram_obj_callbacks = std::env::var_os("FFTA_STATE_TEST_POISON_IWRAM_OBJ_CALLBACKS").is_some();
    let poison_extra_iwram_obj_callbacks = std::env::var_os("FFTA_STATE_TEST_POISON_EXTRA_IWRAM_OBJ_CALLBACKS").is_some();
    let poison_obj_layout_offsets = std::env::var_os("FFTA_STATE_TEST_POISON_OBJ_LAYOUT_OFFSETS").is_some();
    let poison_dma_directories = std::env::var_os("FFTA_STATE_TEST_POISON_DMA_DIRECTORIES").is_some();
    let poison_iwram_helpers = std::env::var_os("FFTA_STATE_TEST_POISON_IWRAM_HELPERS").is_some();
    let poison_records4_callbacks = std::env::var_os("FFTA_STATE_TEST_POISON_RECORDS4_CALLBACKS").is_some();
    let poison_action_callbacks = std::env::var_os("FFTA_STATE_TEST_POISON_ACTION_CALLBACKS").is_some();
    let poison_offset_tables = std::env::var_os("FFTA_STATE_TEST_POISON_OFFSET_TABLES").is_some();
    let poison_tilemap_descriptor = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_DESCRIPTOR").is_some();
    let poison_extra_tilemap_descriptors = std::env::var_os("FFTA_STATE_TEST_POISON_EXTRA_TILEMAP_DESCRIPTORS").is_some();
    let poison_transfer_records = std::env::var_os("FFTA_STATE_TEST_POISON_TRANSFER_RECORDS").is_some();
    let poison_tilemap_row = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_ROW").is_some();
    let poison_tilemap_extra_rows = std::env::var_os("FFTA_STATE_TEST_POISON_TILEMAP_EXTRA_ROWS").is_some();
    let poison_blue_screw_chain = std::env::var_os("FFTA_STATE_TEST_POISON_BLUE_SCREW_CHAIN").is_some();
    let poison_additional_rectangles = std::env::var_os("FFTA_STATE_TEST_POISON_ADDITIONAL_RECTANGLES").is_some();
    let poison_relative_graphics = std::env::var_os("FFTA_STATE_TEST_POISON_RELATIVE_GRAPHICS").is_some();
    let poison_color_indexed = std::env::var_os("FFTA_STATE_TEST_POISON_COLOR_INDEXED").is_some();
    let poison_eight_row_lookup = std::env::var_os("FFTA_STATE_TEST_POISON_EIGHT_ROW_LOOKUP").is_some();
    let poison_task_signed_lookup = std::env::var_os("FFTA_STATE_TEST_POISON_TASK_SIGNED_LOOKUP").is_some();
    let poison_battle_obj = std::env::var_os("FFTA_STATE_TEST_POISON_BATTLE_OBJ").is_some();
    let poison_battle_obj_support = std::env::var_os("FFTA_STATE_TEST_POISON_BATTLE_OBJ_SUPPORT").is_some();
    let poison_indexed_records16 = std::env::var_os("FFTA_STATE_TEST_POISON_INDEXED_RECORDS16").is_some();
    let poison_indexed_words_3959 = std::env::var_os("FFTA_STATE_TEST_POISON_INDEXED_WORDS_3959").is_some();
    let poison_soil_e3838 = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_E3838").is_some();
    let poison_soil_sprite_lists = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_SPRITE_LISTS").is_some();
    let poison_firaga_sprite_lists = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_SPRITE_LISTS").is_some();
    let poison_soil_compact = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_COMPACT").is_some();
    let poison_soil_a83c2_switch = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_A83C2_SWITCH").is_some();
    let poison_soil_remaining_switches = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_REMAINING_SWITCHES").is_some();
    let poison_thumb_switch_tables = std::env::var_os("FFTA_STATE_TEST_POISON_THUMB_SWITCH_TABLES").is_some();
    let poison_obj_derived_dimensions = std::env::var_os("FFTA_STATE_TEST_POISON_OBJ_DERIVED_DIMENSIONS").is_some();
    let poison_copied_halfword_lookups = std::env::var_os("FFTA_STATE_TEST_POISON_COPIED_HALFWORD_LOOKUPS").is_some();
    let poison_soil_copied_halfwords = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_COPIED_HALFWORDS").is_some();
    let poison_soil_transfer = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_TRANSFER").is_some();
    let poison_soil_other_transfer = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_OTHER_TRANSFER").is_some();
    let poison_soil_scene_records = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_SCENE_RECORDS").is_some();
    let poison_soil_indirect_records = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_INDIRECT_RECORDS").is_some();
    let poison_soil_indexed_bytes = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_INDEXED_BYTES").is_some();
    let poison_soil_compact_record = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_COMPACT_RECORD").is_some();
    let poison_soil_scoped_values = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_SCOPED_VALUES").is_some();
    let poison_soil_remaining_rectangles = std::env::var_os("FFTA_STATE_TEST_POISON_SOIL_REMAINING_RECTANGLES").is_some();
    let poison_firaga_copied_matrices = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_COPIED_MATRICES").is_some();
    let poison_firaga_compact_cells = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_COMPACT_CELLS").is_some();
    let poison_firaga_indexed_obj = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_INDEXED_OBJ").is_some();
    let poison_firaga_compact_obj = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_COMPACT_OBJ").is_some();
    let poison_firaga_adjacent_compact_obj = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_ADJACENT_COMPACT_OBJ").is_some();
    let poison_firaga_6c12c8_obj = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_6C12C8_OBJ").is_some();
    let poison_firaga_transfer = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_TRANSFER").is_some();
    let poison_firaga_palette_rows = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_PALETTE_ROWS").is_some();
    let poison_firaga_stack = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_STACK").is_some();
    let poison_firaga_queued = std::env::var_os("FFTA_STATE_TEST_POISON_FIRAGA_QUEUED").is_some();
    let poison_final_copy = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_FINAL_COPY").is_some();
    let poison_indexed_halfwords_39a5 = std::env::var_os("FFTA_STATE_TEST_POISON_INDEXED_HALFWORDS_39A5").is_some();
    let poison_script_byte_lookup_3ad = std::env::var_os("FFTA_STATE_TEST_POISON_SCRIPT_BYTE_LOOKUP_3AD").is_some();
    let poison_additional_thumb_switches = std::env::var_os("FFTA_STATE_TEST_POISON_ADDITIONAL_THUMB_SWITCHES").is_some();
    let poison_thumb_pc_literals = std::env::var_os("FFTA_STATE_TEST_POISON_THUMB_PC_LITERALS").is_some();
    let poison_additional_thumb_literals = std::env::var_os("FFTA_STATE_TEST_POISON_ADDITIONAL_THUMB_LITERALS").is_some();
    let poison_observed_thumb_literals = std::env::var_os("FFTA_STATE_TEST_POISON_OBSERVED_THUMB_LITERALS").is_some();
    let poison_home_anchored_thumb_literals = std::env::var_os("FFTA_STATE_TEST_POISON_HOME_ANCHORED_THUMB_LITERALS").is_some();
    let poison_home_residual_code_support = std::env::var_os("FFTA_STATE_TEST_POISON_HOME_RESIDUAL_CODE_SUPPORT").is_some();
    let poison_red_spring_literals = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_LITERALS").is_some();
    let poison_red_spring_switches = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_SWITCHES").is_some();
    let poison_final_switches = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_FINAL_SWITCHES").is_some();
    let poison_red_spring_anchored = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_ANCHORED").is_some();
    let poison_post_indexed_observed = std::env::var_os("FFTA_STATE_TEST_POISON_POST_INDEXED_OBSERVED").is_some();
    let poison_post_indexed_anchored = std::env::var_os("FFTA_STATE_TEST_POISON_POST_INDEXED_ANCHORED").is_some();
    let poison_bank_observed = std::env::var_os("FFTA_STATE_TEST_POISON_BANK_OBSERVED").is_some();
    let poison_bank_switches = std::env::var_os("FFTA_STATE_TEST_POISON_BANK_SWITCHES").is_some();
    let poison_blue_screw = std::env::var_os("FFTA_STATE_TEST_POISON_BLUE_SCREW_SOURCES").is_some();
    let battle_obj_spans = if poison_battle_obj {
        source.obj_animations.as_ref().unwrap().decoded.banks.iter()
            .find(|bank| bank.object_root == gba_core::rom_data::obj_animations::BATTLE_OBJECT_ROOT)
            .expect("battle OBJ bank").spans().into_iter()
            .map(|(start, len, _)| (start as usize, len)).collect::<Vec<_>>()
    } else { Vec::new() };
    let color_indexed_span = if poison_color_indexed {
        let archive = &source.indexed_graphics.as_ref().unwrap().decoded.color_archives.as_ref().unwrap()[0];
        Some((archive.source_offset as usize, archive.source_bytes().unwrap().len()))
    } else { None };
    let relative_graphics_spans = if poison_relative_graphics {
        let archive = source.obj_graphics.as_ref().unwrap().decoded.relative_archive.as_ref().unwrap();
        std::iter::once((archive.source_offset as usize, archive.source_bytes().unwrap().len()))
            .chain([69usize, 71, 73].into_iter().map(|i| {
                let bank = &archive.banks[i]; (bank.source_offset as usize, bank.encoded_source.len())
            })).collect::<Vec<_>>()
    } else { Vec::new() };
    #[cfg(feature = "rom-data-read-audit")]
    let compact_watch_root = std::env::var("FFTA_STATE_TEST_COMPACT_WATCH_ROOT").ok()
        .map(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).expect("compact watch root"))
        .or_else(|| std::env::var_os("FFTA_STATE_TEST_REQUIRE_COMPACT_6BAC28").map(|_| 0x6bac28));
    #[cfg(feature = "rom-data-read-audit")]
    let compact_watch_span = compact_watch_root.map(|root| {
        let spec = gba_core::rom_data::compact_obj_catalog::BANK_SPECS.iter().find(|spec| spec.root == root).expect("known compact watch root");
        *spec.spans.first().expect("compact watch source span")
    });
    #[cfg(feature = "rom-data-read-audit")]
    let mut compact_watch_hit = None;
    #[cfg(feature = "rom-data-read-audit")]
    let scene_audit_path = std::env::var_os("FFTA_STATE_TEST_ROM_AUDIT_REPORT").map(PathBuf::from);
    #[cfg(feature = "rom-data-read-audit")]
    let poison_red_spring = std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_BANK").is_some();
    #[cfg(feature = "rom-data-read-audit")]
    let mut red_spring_reads = std::collections::BTreeSet::new();
    #[cfg(feature = "rom-data-read-audit")]
    if let Some(path) = &scene_audit_path {
        assert!(!path.exists(), "preserve earlier scene audit reports");
    }
    #[cfg(feature = "rom-data-read-audit")]
    let mut scene_reads = std::collections::BTreeSet::new();
    #[cfg(not(feature = "rom-data-read-audit"))]
    assert!(std::env::var_os("FFTA_STATE_TEST_REQUIRE_COMPACT_6BAC28").is_none() && std::env::var_os("FFTA_STATE_TEST_COMPACT_WATCH_ROOT").is_none() && std::env::var_os("FFTA_STATE_TEST_ROM_AUDIT_REPORT").is_none() && std::env::var_os("FFTA_STATE_TEST_POISON_RED_SPRING_BANK").is_none(), "scene source audit requires rom-data-read-audit");
    for relative in 0..frames {
        let mut counts = [(0, 0); 2];
        for (index, machine) in [&mut legacy, &mut decoded].into_iter().enumerate() {
            machine.bus.keys = input.keys_at(relative);
            input.apply_mouse(&mut machine.bus, relative).unwrap();
            if index == 1 && poison_additional_rectangles {
                for (start, len) in additional_rectangle_test_spans() {
                    for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_relative_graphics {
                for &(start, len) in &relative_graphics_spans {
                    for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 {
                if let Some((start, len)) = color_indexed_span {
                    for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_eight_row_lookup {
                for at in 0x371024..0x371034 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_task_signed_lookup {
                for at in 0x371020..0x371024 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_battle_obj {
                for &(start, len) in &battle_obj_spans {
                    for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_battle_obj_support {
                for start in gba_core::rom_data::obj_lists::IWRAM_ADDITIONAL_CALLBACK_ROOTS {
                    for at in start as usize..start as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
                for at in 0x398df4..0x398dfc { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_indexed_records16 {
                let start = gba_core::rom_data::indexed_records16::SOURCE_OFFSET as usize;
                let end = gba_core::rom_data::indexed_records16::END_OFFSET as usize;
                for at in start..end { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_remaining_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[23..].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_a83c2_switch {
                let spec = gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[22];
                for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_compact {
                for (start, end) in gba_core::rom_data::compact_obj_catalog::SOIL_EVIDENCE_METADATA_RANGES {
                    for at in start as usize..end as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_sprite_lists {
                for (offset, count, _) in gba_core::rom_data::obj_lists::FIRAGA_SPRITE_LISTS {
                    for at in offset as usize..offset as usize + 2 + count * 6 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_sprite_lists {
                for (offset, count, _) in gba_core::rom_data::obj_lists::SOIL_EVIDENCE_LISTS {
                    for at in offset as usize..offset as usize + 2 + count * 6 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_e3838 {
                use gba_core::rom_data::indexed_words_3959::{SOIL_E3838_BASE, SOIL_E3838_WORDS};
                for (id, _) in SOIL_E3838_WORDS {
                    let start = (SOIL_E3838_BASE + u32::from(id) * 4) as usize;
                    for at in start..start + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_indexed_words_3959 {
                let start = gba_core::rom_data::indexed_words_3959::SOURCE_OFFSET as usize;
                let end = gba_core::rom_data::indexed_words_3959::END_OFFSET as usize;
                for at in start..end { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_thumb_switch_tables {
                for spec in gba_core::rom_data::thumb_switch_tables::TABLES {
                    for at in spec.root as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_additional_thumb_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[..4].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_obj_derived_dimensions {
                for start in gba_core::rom_data::obj_derived_dimensions::TABLE_OFFSETS {
                    for at in start as usize..start as usize + 48 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_final_copy {
                let start = gba_core::rom_data::copied_halfword_lookups::TABLE_OFFSETS[2] as usize;
                for at in start..start + 8 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_copied_halfwords {
                let start = gba_core::rom_data::copied_halfword_lookups::SOIL_EVIDENCE_OFFSET as usize;
                for at in start..start + 8 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_scene_records {
                use gba_core::rom_data::soil_scene_records::{RANGE_OFFSET, SEQUENCE_OFFSET, SEQUENCE_COUNT, RECORD_BYTES};
                for at in RANGE_OFFSET as usize..SEQUENCE_OFFSET as usize + SEQUENCE_COUNT * RECORD_BYTES { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_indirect_records {
                use gba_core::rom_data::soil_indirect_records::{OFFSETS, RECORD_BYTES};
                for start in OFFSETS { for at in start as usize..start as usize + RECORD_BYTES { machine.bus.rom[at] = !rom[at]; } }
            }
            if index == 1 && poison_soil_indexed_bytes {
                use gba_core::rom_data::soil_indexed_bytes::{OFFSET, COUNT};
                for at in OFFSET as usize..OFFSET as usize + COUNT { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_soil_compact_record {
                use gba_core::rom_data::soil_compact_record::{RECORD_OFFSET, RECORD_BYTES, POINTER_OFFSET, POINTER_BYTES};
                for (start, len) in [(RECORD_OFFSET, RECORD_BYTES), (POINTER_OFFSET, POINTER_BYTES)] { for at in start as usize..start as usize + len { machine.bus.rom[at] = !rom[at]; } }
            }
            if index == 1 && poison_soil_scoped_values {
                for (start, len) in gba_core::rom_data::soil_scoped_values::SPANS { for at in start as usize..start as usize + len { machine.bus.rom[at] = !rom[at]; } }
            }
            if index == 1 && poison_firaga_copied_matrices {
                use gba_core::rom_data::firaga_copied_matrices::{LITERALS,MATRICES};
                for (offset,_) in LITERALS { for at in offset as usize..offset as usize+4 { machine.bus.rom[at]=!rom[at]; } }
                for (offset,width,height) in MATRICES { for at in offset as usize..offset as usize+2*usize::from(width)*usize::from(height) { machine.bus.rom[at]=!rom[at]; } }
            }
            if index == 1 && poison_firaga_compact_cells {
                use gba_core::rom_data::firaga_compact_cells::{CALLER_LITERAL,TABLE_CELL,SOURCE_OFFSET,FIRST_END,SEQUENCE_OFFSET,SEQUENCE_END};
                for (start,end) in [(CALLER_LITERAL,CALLER_LITERAL+4),(TABLE_CELL,TABLE_CELL+4),(SOURCE_OFFSET,FIRST_END),(SEQUENCE_OFFSET,SEQUENCE_END)] {
                    for at in start as usize..end as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_indexed_obj {
                use gba_core::rom_data::{firaga_indexed_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::DESCRIPTOR_OFFSET,12),(bank::SEQUENCE_ROOT,82),(bank::OBJECT_ROOT,80),(obj_lists::FIRAGA_EMPTY_LIST.0,2)] {
                    for at in start as usize..start as usize+len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_compact_obj {
                use gba_core::rom_data::{firaga_compact_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::CALLER_LITERAL,4),(bank::DESCRIPTOR_ROOT,4),(bank::SELECTED_POINTER,4),(bank::BANK_ROOT,24),(bank::SEQUENCE_ROOT,24)] {
                    for at in start as usize..start as usize+len { machine.bus.rom[at] = !rom[at]; }
                }
                for (start,_,_) in obj_lists::FIRAGA_COMPACT_EMPTY_LISTS { for at in start as usize..start as usize+2 { machine.bus.rom[at]=!rom[at]; } }
            }
            if index == 1 && poison_firaga_adjacent_compact_obj {
                use gba_core::rom_data::{firaga_adjacent_compact_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::COUNT_LITERAL,4),(bank::COUNT_BYTE,1),(bank::FIRST_FIELDS,20),(bank::SECOND_DESCRIPTOR,24),(bank::BANK_ROOT,26),(bank::SEQUENCE_DIRECTORY,2),(bank::FIRST_SEQUENCE,36),(obj_lists::FIRAGA_ADJACENT_EMPTY_LIST.0,2)] {
                    for at in start as usize..start as usize+len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_6c12c8_obj {
                use gba_core::rom_data::{firaga_6c12c8_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::COUNT_BYTE,1),(bank::DESCRIPTOR_POINTER,4),(bank::DESCRIPTOR,24),(bank::BANK_ROOT,24),(bank::SEQUENCE_ROOT,30),(obj_lists::FIRAGA_6C12C8_EMPTY_LIST.0,2)] {
                    for at in start as usize..start as usize+len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_transfer {
                use gba_core::rom_data::firaga_transfer_program as program;
                for (start,len) in program::SOURCE_OFFSETS.into_iter().zip(program::SOURCE_LENGTHS) {
                    for at in start as usize..start as usize+len {machine.bus.rom[at]=!rom[at];}
                }
            }
            if index == 1 && poison_firaga_palette_rows {
                for start in gba_core::rom_data::firaga_palette_rows::ROW_OFFSETS {
                    for at in start as usize..start as usize+8 {machine.bus.rom[at]=!rom[at];}
                }
            }
            if index == 1 && poison_firaga_stack {
                use gba_core::rom_data::firaga_stack_initialization as source;
                for (start,len) in source::SOURCE_OFFSETS.into_iter().zip(source::SOURCE_LENGTHS) {
                    for at in start as usize..start as usize+len {machine.bus.rom[at]=!rom[at];}
                }
            }
            if index == 1 && poison_firaga_queued {
                use gba_core::rom_data::firaga_queued_transfer as source;
                for (start,len) in source::SOURCE_OFFSETS.into_iter().zip(source::SOURCE_LENGTHS) {
                    for at in start as usize..start as usize+len {machine.bus.rom[at]=!rom[at];}
                }
            }
            if index == 1 && poison_soil_remaining_rectangles {
                use gba_core::rom_data::soil_remaining_rectangles::{ALT_LITERAL,RECTS};
                for at in ALT_LITERAL as usize..ALT_LITERAL as usize+4 { machine.bus.rom[at]=!rom[at]; }
                for (offset,height,width) in RECTS { for at in offset as usize..offset as usize+2+2*usize::from(height)*usize::from(width) { machine.bus.rom[at]=!rom[at]; } }
            }
            if index == 1 && poison_soil_other_transfer {
                use gba_core::rom_data::{observed_tilemap as tilemap, observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for (start, len) in [(tilemap::SOIL_RECTANGLE_ENTRIES_OFFSET, 16), (tilemap::SOIL_RECTANGLE_LIST_OFFSET, 8),
                    (tilemap::SOIL_MODE7_OFFSET, 10), (descriptor::SOIL_MODE7_DESCRIPTOR_OFFSET, 8)] {
                    for at in start as usize..start as usize + len { machine.bus.rom[at] = !rom[at]; }
                }
                for &(start, height, width, _) in &tilemap::SOIL_RECTANGLE_SPECS {
                    for at in start as usize..start as usize + 2 + usize::from(height) * usize::from(width) * 2 { machine.bus.rom[at] = !rom[at]; }
                }
                for &(start, _, _, _) in &row::SOIL_OTHER_ROW_SPECS {
                    for at in start as usize..start as usize + 12 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_transfer {
                use gba_core::rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for &(start, _, wrapper, _) in &descriptor::SOIL_TRANSFER_SPECS {
                    for (at, len) in [(start as usize, 8), (wrapper as usize, 4)] {
                        for offset in at..at + len { machine.bus.rom[offset] = !rom[offset]; }
                    }
                }
                for &(start, _, _) in &row::SOIL_TRANSFER_ROW_SPECS {
                    for at in start as usize..start as usize + 12 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_copied_halfword_lookups {
                use gba_core::rom_data::copied_halfword_lookups::{ADDITIONAL_ROOTS, TABLE_OFFSETS};
                for start in TABLE_OFFSETS[..2].iter().copied() { for at in start as usize..start as usize + 8 { machine.bus.rom[at] = !rom[at]; } }
                for start in ADDITIONAL_ROOTS { for at in start as usize..start as usize + 4 { machine.bus.rom[at] = !rom[at]; } }
            }
            if index == 1 && poison_indexed_halfwords_39a5 {
                use gba_core::rom_data::indexed_halfwords_39a5::{SOURCE_BYTES, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_BYTES { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_script_byte_lookup_3ad {
                use gba_core::rom_data::script_byte_lookup_3ad::{ADDITIONAL_ROOT, SOURCE_BYTES, SOURCE_OFFSET};
                for at in SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + SOURCE_BYTES { machine.bus.rom[at] = !rom[at]; }
                for at in ADDITIONAL_ROOT as usize..ADDITIONAL_ROOT as usize + 4 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_thumb_pc_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::LITERALS {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_additional_thumb_literals {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::ADDITIONAL {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_observed_thumb_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_home_anchored_thumb_literals {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::HOME_ANCHORED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_home_residual_code_support {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::HOME_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[4..6].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_red_spring_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::RED_SPRING_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_red_spring_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[6..16].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_final_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[16..18].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_bank_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[18..21].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_blue_screw {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::BLUE_SCREW_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[21..22].iter().copied() {
                    for at in spec.start() as usize..spec.end() as usize { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_bank_observed {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::BANK_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_post_indexed_anchored {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::POST_INDEXED_ANCHORED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_post_indexed_observed {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::POST_INDEXED_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_red_spring_anchored {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::RED_SPRING_ANCHORED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_blue_screw_chain {
                for (start, len) in blue_screw_chain_test_spans() {
                    for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_146_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::SOIL_146_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_2d2e_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_2D2E_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_eb83_ebc_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_EB83_EBC_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_code_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_CODE_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_mixed_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_MIXED_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_residual_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_RESIDUAL_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::SOIL_EVIDENCE_OBSERVED {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_firaga_lz {
                for spec in &gba_core::rom_data::observed_lz::ADDITIONAL_SPECS[16..19] {
                    for at in spec.offset as usize..spec.offset as usize + spec.encoded_length { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_soil_lz {
                for spec in &gba_core::rom_data::observed_lz::ADDITIONAL_SPECS[13..16] {
                    for at in spec.offset as usize..spec.offset as usize + spec.encoded_length { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_observed_lz {
                for (start, len) in observed_lz_test_spans() {
                    for at in start..start+len { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_constructor64 {
                use gba_core::rom_data::constructor_template64::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_observed_tilemap {
                use gba_core::rom_data::observed_tilemap::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_tilemap_rectangles {
                for &(offset, height, width, _, _) in &gba_core::rom_data::observed_tilemap::RECT_SPECS {
                    let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
                    for at in start..start + length { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_tilemap_alternates {
                use gba_core::rom_data::observed_tilemap::{ALTERNATE_RECT_SPECS, ALTERNATE_SELECTOR_LENGTH, ALTERNATE_SELECTOR_OFFSET};
                for &(offset, height, width, _, _) in &ALTERNATE_RECT_SPECS {
                    let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
                    for at in start..start + length { machine.bus.rom[at] = !rom[at]; }
                }
                let start = ALTERNATE_SELECTOR_OFFSET as usize;
                for at in start..start + ALTERNATE_SELECTOR_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_tilemap_roots {
                use gba_core::rom_data::observed_tilemap::{RECTANGLE_CHAIN_OFFSET, RECTANGLE_CHAIN_LENGTH, RECTANGLE_TABLE_OFFSET, RECTANGLE_TABLE_LENGTH};
                for (offset, length) in [(RECTANGLE_CHAIN_OFFSET, RECTANGLE_CHAIN_LENGTH), (RECTANGLE_TABLE_OFFSET, RECTANGLE_TABLE_LENGTH)] {
                    for at in offset as usize..offset as usize + length { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_tilemap_list_row {
                use gba_core::rom_data::observed_tilemap_row::{LIST_ROW_OFFSET, SOURCE_LENGTH};
                for at in LIST_ROW_OFFSET as usize..LIST_ROW_OFFSET as usize + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_compact_records {
                use gba_core::rom_data::compact_obj_catalog::{RED_SPRING_RECORD_OFFSET, RED_SPRING_RECORD_LENGTH};
                let start = RED_SPRING_RECORD_OFFSET as usize;
                for at in start..start + RED_SPRING_RECORD_LENGTH * 3 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_red_spring_sounds {
                for &(root, source, count) in &gba_core::rom_data::audio_directory::RED_SPRING_SOUND_SPECS {
                    for (start, len) in [(root as usize, 4), (source as usize, count * 8)] {
                        for at in start..start + len { machine.bus.rom[at] = !rom[at]; }
                    }
                }
            }
            if index == 1 && poison_iwram_obj_callbacks {
                use gba_core::rom_data::obj_lists::{IWRAM_DRAW_CALLBACKS_COUNT, IWRAM_DRAW_CALLBACKS_ROOT};
                let start = IWRAM_DRAW_CALLBACKS_ROOT as usize;
                for at in start..start + IWRAM_DRAW_CALLBACKS_COUNT * 4 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_extra_iwram_obj_callbacks {
                for offset in gba_core::rom_data::obj_lists::IWRAM_EXTRA_CALLBACK_ROOTS {
                    for at in offset as usize..offset as usize + 4 { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_records4_callbacks {
                use gba_core::rom_data::records4::{CALLBACK_ENDS, CALLBACK_ROOTS};
                for at in CALLBACK_ROOTS[0] as usize..CALLBACK_ENDS[2] as usize { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_action_callbacks {
                use gba_core::rom_data::records4::{ACTION_END, ACTION_ROOT};
                for at in ACTION_ROOT as usize..ACTION_END as usize { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_offset_tables {
                use gba_core::rom_data::adjacent_offset_tables::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_obj_layout_offsets {
                use gba_core::rom_data::obj_lists::{LAYOUT_OFFSETS_LENGTH, LAYOUT_OFFSETS_ROOT};
                let start = LAYOUT_OFFSETS_ROOT as usize;
                for at in start..start + LAYOUT_OFFSETS_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_dma_directories {
                use gba_core::rom_data::dma_register_directories::{DIRECTORY_LENGTH, WRITE_ROOT};
                let start = WRITE_ROOT as usize;
                for at in start..start + DIRECTORY_LENGTH * 2 { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_iwram_helpers {
                use gba_core::rom_data::iwram_memory_helpers::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_tilemap_descriptor {
                use gba_core::rom_data::observed_tilemap_descriptor::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_extra_tilemap_descriptors {
                use gba_core::rom_data::observed_tilemap_descriptor::{ADDITIONAL_SPECS, SOURCE_LENGTH, WRAPPER_LENGTH};
                for &(offset, _, wrapper, _) in &ADDITIONAL_SPECS {
                    for at in offset as usize..offset as usize + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
                    for at in wrapper as usize..wrapper as usize + WRAPPER_LENGTH { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_transfer_records {
                use gba_core::rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for &(offset, _, wrapper, _) in &descriptor::TRANSFER_SPECS {
                    for at in offset as usize..offset as usize + descriptor::SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
                    for at in wrapper as usize..wrapper as usize + descriptor::WRAPPER_LENGTH { machine.bus.rom[at] = !rom[at]; }
                }
                for &(offset, _, _) in &row::TRANSFER_SPECS {
                    for at in offset as usize..offset as usize + row::SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
                }
            }
            if index == 1 && poison_tilemap_row {
                use gba_core::rom_data::observed_tilemap_row::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
            }
            if index == 1 && poison_tilemap_extra_rows {
                use gba_core::rom_data::observed_tilemap_row::{ADDITIONAL_SPECS, SOURCE_LENGTH};
                for &(offset, _, _) in &ADDITIONAL_SPECS {
                    let start = offset as usize;
                    for at in start..start + SOURCE_LENGTH { machine.bus.rom[at] = !rom[at]; }
                }
            }
            #[cfg(feature = "rom-data-read-audit")]
            if index == 1 && poison_red_spring {
                for &(start, end) in super::compact_redspring_tests::NEW_SPANS {
                    for at in start..end { machine.bus.rom[at] = !rom[at]; }
                }
            }
            let pointer = machine as *mut gba_core::Machine as *mut std::ffi::c_void;
            #[cfg(feature = "rom-data-read-audit")]
            let compact_watch = if index == 1 && compact_watch_hit.is_none() && compact_watch_span.is_some() {
                let (start, end) = compact_watch_span.unwrap();
                Some(gba_core::debug_memory::Capture::start(gba_core::debug_memory::Watch {
                    start: 0x08000000 + start, end: 0x08000000 + end, write: false, width: None, value: None,
                }).unwrap())
            } else { None };
            #[cfg(feature = "rom-data-read-audit")]
            let scene_audit = (index == 1 && (scene_audit_path.is_some() || poison_red_spring))
                .then(gba_core::rom_data::read_audit::Capture::start);
            counts[index] = run_frame_native(machine, &table, pointer, FRAME_STEP_GUARD);
            #[cfg(feature = "rom-data-read-audit")]
            if let Some(capture) = scene_audit {
                let reads = capture.finish();
                if poison_red_spring {
                    red_spring_reads.extend(reads.iter().copied().filter(|read| {
                        let start = read.physical_offset as usize;
                        let end = start.saturating_add(read.length);
                        super::compact_redspring_tests::NEW_SPANS.iter().any(|&(a, z)| start < z && end > a)
                    }));
                }
                if scene_audit_path.is_some() { scene_reads.extend(reads); }
            }
            #[cfg(feature = "rom-data-read-audit")]
            if let Some(watch) = compact_watch {
                if let Some(hit) = watch.hit() { compact_watch_hit = Some((machine.bus.frames, hit)); }
            }
            #[cfg(feature = "rom-data-read-audit")]
            if index == 1 && poison_red_spring {
                // Snapshot identity hashes the ROM; restore only after the frame's
                // guest and host readers have used the altered backing bytes.
                for &(start, end) in super::compact_redspring_tests::NEW_SPANS {
                    machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                }
            }
            if index == 1 && poison_soil_146_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::SOIL_146_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_2d2e_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_2D2E_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_eb83_ebc_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_EB83_EBC_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_code_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_CODE_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_mixed_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_MIXED_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_residual_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::FIRAGA_RESIDUAL_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_soil_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::SOIL_EVIDENCE_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_firaga_lz {
                for spec in &gba_core::rom_data::observed_lz::ADDITIONAL_SPECS[16..19] {
                    let start = spec.offset as usize;
                    machine.bus.rom[start..start + spec.encoded_length].copy_from_slice(&rom[start..start + spec.encoded_length]);
                }
            }
            if index == 1 && poison_soil_lz {
                for spec in &gba_core::rom_data::observed_lz::ADDITIONAL_SPECS[13..16] {
                    let start = spec.offset as usize;
                    machine.bus.rom[start..start + spec.encoded_length].copy_from_slice(&rom[start..start + spec.encoded_length]);
                }
            }
            if index == 1 && poison_observed_lz {
                for (start, len) in observed_lz_test_spans() {
                    machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_blue_screw_chain {
                for (start, len) in blue_screw_chain_test_spans() {
                    machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                }
            }
            if index == 1 && poison_additional_rectangles {
                for (start, len) in additional_rectangle_test_spans() {
                    machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                }
            }
            if index == 1 && poison_relative_graphics {
                for &(start, len) in &relative_graphics_spans {
                    machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                }
            }
            if index == 1 {
                if let Some((start, len)) = color_indexed_span {
                    machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                }
            }
            if index == 1 && poison_eight_row_lookup {
                machine.bus.rom[0x371024..0x371034].copy_from_slice(&rom[0x371024..0x371034]);
            }
            if index == 1 && poison_task_signed_lookup {
                machine.bus.rom[0x371020..0x371024].copy_from_slice(&rom[0x371020..0x371024]);
            }
            if index == 1 && poison_battle_obj {
                for &(start, len) in &battle_obj_spans {
                    machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                }
            }
            if index == 1 && poison_battle_obj_support {
                for start in gba_core::rom_data::obj_lists::IWRAM_ADDITIONAL_CALLBACK_ROOTS {
                    let start = start as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
                machine.bus.rom[0x398df4..0x398dfc].copy_from_slice(&rom[0x398df4..0x398dfc]);
            }
            if index == 1 && poison_indexed_records16 {
                let start = gba_core::rom_data::indexed_records16::SOURCE_OFFSET as usize;
                let end = gba_core::rom_data::indexed_records16::END_OFFSET as usize;
                machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            }
            if index == 1 && poison_soil_remaining_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[23..].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize]
                        .copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_soil_a83c2_switch {
                let spec = gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[22];
                machine.bus.rom[spec.start() as usize..spec.end() as usize]
                    .copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
            }
            if index == 1 && poison_soil_compact {
                for (start, end) in gba_core::rom_data::compact_obj_catalog::SOIL_EVIDENCE_METADATA_RANGES {
                    let (start, end) = (start as usize, end as usize);
                    machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                }
            }
            if index == 1 && poison_firaga_sprite_lists {
                for (offset, count, _) in gba_core::rom_data::obj_lists::FIRAGA_SPRITE_LISTS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 2 + count * 6].copy_from_slice(&rom[start..start + 2 + count * 6]);
                }
            }
            if index == 1 && poison_soil_sprite_lists {
                for (offset, count, _) in gba_core::rom_data::obj_lists::SOIL_EVIDENCE_LISTS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 2 + count * 6].copy_from_slice(&rom[start..start + 2 + count * 6]);
                }
            }
            if index == 1 && poison_soil_e3838 {
                use gba_core::rom_data::indexed_words_3959::{SOIL_E3838_BASE, SOIL_E3838_WORDS};
                for (id, _) in SOIL_E3838_WORDS {
                    let start = (SOIL_E3838_BASE + u32::from(id) * 4) as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_indexed_words_3959 {
                let start = gba_core::rom_data::indexed_words_3959::SOURCE_OFFSET as usize;
                let end = gba_core::rom_data::indexed_words_3959::END_OFFSET as usize;
                machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            }
            if index == 1 && poison_thumb_switch_tables {
                for spec in gba_core::rom_data::thumb_switch_tables::TABLES {
                    let start = spec.root as usize; let end = spec.end() as usize;
                    machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                }
            }
            if index == 1 && poison_additional_thumb_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[..4].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_obj_derived_dimensions {
                for start in gba_core::rom_data::obj_derived_dimensions::TABLE_OFFSETS {
                    let start = start as usize;
                    machine.bus.rom[start..start + 48].copy_from_slice(&rom[start..start + 48]);
                }
            }
            if index == 1 && poison_final_copy {
                let start = gba_core::rom_data::copied_halfword_lookups::TABLE_OFFSETS[2] as usize;
                machine.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
            }
            if index == 1 && poison_soil_copied_halfwords {
                let start = gba_core::rom_data::copied_halfword_lookups::SOIL_EVIDENCE_OFFSET as usize;
                machine.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
            }
            if index == 1 && poison_soil_scene_records {
                use gba_core::rom_data::soil_scene_records::{RANGE_OFFSET, SEQUENCE_OFFSET, SEQUENCE_COUNT, RECORD_BYTES};
                let start = RANGE_OFFSET as usize; let end = SEQUENCE_OFFSET as usize + SEQUENCE_COUNT * RECORD_BYTES;
                machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            }
            if index == 1 && poison_soil_indirect_records {
                use gba_core::rom_data::soil_indirect_records::{OFFSETS, RECORD_BYTES};
                for start in OFFSETS { let at = start as usize; machine.bus.rom[at..at + RECORD_BYTES].copy_from_slice(&rom[at..at + RECORD_BYTES]); }
            }
            if index == 1 && poison_soil_indexed_bytes {
                use gba_core::rom_data::soil_indexed_bytes::{OFFSET, COUNT};
                let at = OFFSET as usize; machine.bus.rom[at..at + COUNT].copy_from_slice(&rom[at..at + COUNT]);
            }
            if index == 1 && poison_soil_compact_record {
                use gba_core::rom_data::soil_compact_record::{RECORD_OFFSET, RECORD_BYTES, POINTER_OFFSET, POINTER_BYTES};
                for (start, len) in [(RECORD_OFFSET, RECORD_BYTES), (POINTER_OFFSET, POINTER_BYTES)] { let at = start as usize; machine.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]); }
            }
            if index == 1 && poison_soil_scoped_values {
                for (start, len) in gba_core::rom_data::soil_scoped_values::SPANS { let at=start as usize; machine.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]); }
            }
            if index == 1 && poison_firaga_copied_matrices {
                use gba_core::rom_data::firaga_copied_matrices::{LITERALS,MATRICES};
                for (offset,_) in LITERALS { let at=offset as usize; machine.bus.rom[at..at+4].copy_from_slice(&rom[at..at+4]); }
                for (offset,width,height) in MATRICES { let at=offset as usize; let len=2*usize::from(width)*usize::from(height); machine.bus.rom[at..at+len].copy_from_slice(&rom[at..at+len]); }
            }
            if index == 1 && poison_firaga_compact_cells {
                use gba_core::rom_data::firaga_compact_cells::{CALLER_LITERAL,TABLE_CELL,SOURCE_OFFSET,FIRST_END,SEQUENCE_OFFSET,SEQUENCE_END};
                for (start,end) in [(CALLER_LITERAL,CALLER_LITERAL+4),(TABLE_CELL,TABLE_CELL+4),(SOURCE_OFFSET,FIRST_END),(SEQUENCE_OFFSET,SEQUENCE_END)] {
                    let start=start as usize; let end=end as usize; machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                }
            }
            if index == 1 && poison_firaga_indexed_obj {
                use gba_core::rom_data::{firaga_indexed_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::DESCRIPTOR_OFFSET,12),(bank::SEQUENCE_ROOT,82),(bank::OBJECT_ROOT,80),(obj_lists::FIRAGA_EMPTY_LIST.0,2)] {
                    let start=start as usize; machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_firaga_compact_obj {
                use gba_core::rom_data::{firaga_compact_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::CALLER_LITERAL,4),(bank::DESCRIPTOR_ROOT,4),(bank::SELECTED_POINTER,4),(bank::BANK_ROOT,24),(bank::SEQUENCE_ROOT,24)] {
                    let start=start as usize; machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
                for (start,_,_) in obj_lists::FIRAGA_COMPACT_EMPTY_LISTS {let start=start as usize;machine.bus.rom[start..start+2].copy_from_slice(&rom[start..start+2]);}
            }
            if index == 1 && poison_firaga_adjacent_compact_obj {
                use gba_core::rom_data::{firaga_adjacent_compact_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::COUNT_LITERAL,4),(bank::COUNT_BYTE,1),(bank::FIRST_FIELDS,20),(bank::SECOND_DESCRIPTOR,24),(bank::BANK_ROOT,26),(bank::SEQUENCE_DIRECTORY,2),(bank::FIRST_SEQUENCE,36),(obj_lists::FIRAGA_ADJACENT_EMPTY_LIST.0,2)] {
                    let start=start as usize; machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_firaga_6c12c8_obj {
                use gba_core::rom_data::{firaga_6c12c8_obj_bank as bank,obj_lists};
                for (start,len) in [(bank::COUNT_BYTE,1),(bank::DESCRIPTOR_POINTER,4),(bank::DESCRIPTOR,24),(bank::BANK_ROOT,24),(bank::SEQUENCE_ROOT,30),(obj_lists::FIRAGA_6C12C8_EMPTY_LIST.0,2)] {
                    let start=start as usize; machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_firaga_transfer {
                use gba_core::rom_data::firaga_transfer_program as program;
                for (start,len) in program::SOURCE_OFFSETS.into_iter().zip(program::SOURCE_LENGTHS) {
                    let start=start as usize;machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_firaga_palette_rows {
                for start in gba_core::rom_data::firaga_palette_rows::ROW_OFFSETS {
                    let start=start as usize;machine.bus.rom[start..start+8].copy_from_slice(&rom[start..start+8]);
                }
            }
            if index == 1 && poison_firaga_stack {
                use gba_core::rom_data::firaga_stack_initialization as source;
                for (start,len) in source::SOURCE_OFFSETS.into_iter().zip(source::SOURCE_LENGTHS) {
                    let start=start as usize;machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_firaga_queued {
                use gba_core::rom_data::firaga_queued_transfer as source;
                for (start,len) in source::SOURCE_OFFSETS.into_iter().zip(source::SOURCE_LENGTHS) {
                    let start=start as usize;machine.bus.rom[start..start+len].copy_from_slice(&rom[start..start+len]);
                }
            }
            if index == 1 && poison_soil_remaining_rectangles {
                use gba_core::rom_data::soil_remaining_rectangles::{ALT_LITERAL,RECTS};
                let at=ALT_LITERAL as usize; machine.bus.rom[at..at+4].copy_from_slice(&rom[at..at+4]);
                for (offset,height,width) in RECTS { let at=offset as usize; let len=2+2*usize::from(height)*usize::from(width); machine.bus.rom[at..at+len].copy_from_slice(&rom[at..at+len]); }
            }
            if index == 1 && poison_soil_other_transfer {
                use gba_core::rom_data::{observed_tilemap as tilemap, observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for (start, len) in [(tilemap::SOIL_RECTANGLE_ENTRIES_OFFSET, 16), (tilemap::SOIL_RECTANGLE_LIST_OFFSET, 8),
                    (tilemap::SOIL_MODE7_OFFSET, 10), (descriptor::SOIL_MODE7_DESCRIPTOR_OFFSET, 8)] {
                    let at = start as usize; machine.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
                }
                for &(start, height, width, _) in &tilemap::SOIL_RECTANGLE_SPECS {
                    let at = start as usize; let len = 2 + usize::from(height) * usize::from(width) * 2;
                    machine.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
                }
                for &(start, _, _, _) in &row::SOIL_OTHER_ROW_SPECS {
                    let at = start as usize; machine.bus.rom[at..at + 12].copy_from_slice(&rom[at..at + 12]);
                }
            }
            if index == 1 && poison_soil_transfer {
                use gba_core::rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for &(start, _, wrapper, _) in &descriptor::SOIL_TRANSFER_SPECS {
                    for (at, len) in [(start as usize, 8), (wrapper as usize, 4)] {
                        machine.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
                    }
                }
                for &(start, _, _) in &row::SOIL_TRANSFER_ROW_SPECS {
                    let at = start as usize; machine.bus.rom[at..at + 12].copy_from_slice(&rom[at..at + 12]);
                }
            }
            if index == 1 && poison_copied_halfword_lookups {
                use gba_core::rom_data::copied_halfword_lookups::{ADDITIONAL_ROOTS, TABLE_OFFSETS};
                for start in TABLE_OFFSETS[..2].iter().copied() { let start = start as usize; machine.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]); }
                for start in ADDITIONAL_ROOTS { let start = start as usize; machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]); }
            }
            if index == 1 && poison_indexed_halfwords_39a5 {
                use gba_core::rom_data::indexed_halfwords_39a5::{SOURCE_BYTES, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_BYTES].copy_from_slice(&rom[start..start + SOURCE_BYTES]);
            }
            if index == 1 && poison_script_byte_lookup_3ad {
                use gba_core::rom_data::script_byte_lookup_3ad::{ADDITIONAL_ROOT, SOURCE_BYTES, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_BYTES].copy_from_slice(&rom[start..start + SOURCE_BYTES]);
                let root = ADDITIONAL_ROOT as usize;
                machine.bus.rom[root..root + 4].copy_from_slice(&rom[root..root + 4]);
            }
            if index == 1 && poison_thumb_pc_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::LITERALS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_additional_thumb_literals {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::ADDITIONAL {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_observed_thumb_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_home_anchored_thumb_literals {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::HOME_ANCHORED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_home_residual_code_support {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::HOME_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[4..6].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_red_spring_literals {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::RED_SPRING_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_red_spring_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[6..16].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_final_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[16..18].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_bank_switches {
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[18..21].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_blue_screw {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::BLUE_SCREW_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
                for spec in gba_core::rom_data::thumb_switch_tables::ADDITIONAL_TABLES[21..22].iter().copied() {
                    machine.bus.rom[spec.start() as usize..spec.end() as usize].copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
                }
            }
            if index == 1 && poison_bank_observed {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::BANK_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_post_indexed_anchored {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::POST_INDEXED_ANCHORED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_post_indexed_observed {
                for (offset, _, _) in gba_core::rom_data::thumb_pc_literals::POST_INDEXED_OBSERVED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_red_spring_anchored {
                for (offset, _, _, _, _) in gba_core::rom_data::thumb_pc_literals::RED_SPRING_ANCHORED {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_constructor64 {
                use gba_core::rom_data::constructor_template64::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_observed_tilemap {
                use gba_core::rom_data::observed_tilemap::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_tilemap_rectangles {
                for &(offset, height, width, _, _) in &gba_core::rom_data::observed_tilemap::RECT_SPECS {
                    let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
                    machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
                }
            }
            if index == 1 && poison_tilemap_alternates {
                use gba_core::rom_data::observed_tilemap::{ALTERNATE_RECT_SPECS, ALTERNATE_SELECTOR_LENGTH, ALTERNATE_SELECTOR_OFFSET};
                for &(offset, height, width, _, _) in &ALTERNATE_RECT_SPECS {
                    let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
                    machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
                }
                let start = ALTERNATE_SELECTOR_OFFSET as usize;
                machine.bus.rom[start..start + ALTERNATE_SELECTOR_LENGTH].copy_from_slice(&rom[start..start + ALTERNATE_SELECTOR_LENGTH]);
            }
            if index == 1 && poison_tilemap_roots {
                use gba_core::rom_data::observed_tilemap::{RECTANGLE_CHAIN_OFFSET, RECTANGLE_CHAIN_LENGTH, RECTANGLE_TABLE_OFFSET, RECTANGLE_TABLE_LENGTH};
                for (offset, length) in [(RECTANGLE_CHAIN_OFFSET, RECTANGLE_CHAIN_LENGTH), (RECTANGLE_TABLE_OFFSET, RECTANGLE_TABLE_LENGTH)] {
                    let start = offset as usize;
                    machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
                }
            }
            if index == 1 && poison_tilemap_list_row {
                use gba_core::rom_data::observed_tilemap_row::{LIST_ROW_OFFSET, SOURCE_LENGTH};
                let start = LIST_ROW_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_compact_records {
                use gba_core::rom_data::compact_obj_catalog::{RED_SPRING_RECORD_OFFSET, RED_SPRING_RECORD_LENGTH};
                let start = RED_SPRING_RECORD_OFFSET as usize;
                machine.bus.rom[start..start + RED_SPRING_RECORD_LENGTH * 3].copy_from_slice(&rom[start..start + RED_SPRING_RECORD_LENGTH * 3]);
            }
            if index == 1 && poison_red_spring_sounds {
                for &(root, source, count) in &gba_core::rom_data::audio_directory::RED_SPRING_SOUND_SPECS {
                    for (start, len) in [(root as usize, 4), (source as usize, count * 8)] {
                        machine.bus.rom[start..start + len].copy_from_slice(&rom[start..start + len]);
                    }
                }
            }
            if index == 1 && poison_iwram_obj_callbacks {
                use gba_core::rom_data::obj_lists::{IWRAM_DRAW_CALLBACKS_COUNT, IWRAM_DRAW_CALLBACKS_ROOT};
                let start = IWRAM_DRAW_CALLBACKS_ROOT as usize;
                machine.bus.rom[start..start + IWRAM_DRAW_CALLBACKS_COUNT * 4]
                    .copy_from_slice(&rom[start..start + IWRAM_DRAW_CALLBACKS_COUNT * 4]);
            }
            if index == 1 && poison_extra_iwram_obj_callbacks {
                for offset in gba_core::rom_data::obj_lists::IWRAM_EXTRA_CALLBACK_ROOTS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
                }
            }
            if index == 1 && poison_records4_callbacks {
                use gba_core::rom_data::records4::{CALLBACK_ENDS, CALLBACK_ROOTS};
                let start = CALLBACK_ROOTS[0] as usize; let end = CALLBACK_ENDS[2] as usize;
                machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            }
            if index == 1 && poison_action_callbacks {
                use gba_core::rom_data::records4::{ACTION_END, ACTION_ROOT};
                let start = ACTION_ROOT as usize; let end = ACTION_END as usize;
                machine.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            }
            if index == 1 && poison_offset_tables {
                use gba_core::rom_data::adjacent_offset_tables::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_obj_layout_offsets {
                use gba_core::rom_data::obj_lists::{LAYOUT_OFFSETS_LENGTH, LAYOUT_OFFSETS_ROOT};
                let start = LAYOUT_OFFSETS_ROOT as usize;
                machine.bus.rom[start..start + LAYOUT_OFFSETS_LENGTH].copy_from_slice(&rom[start..start + LAYOUT_OFFSETS_LENGTH]);
            }
            if index == 1 && poison_dma_directories {
                use gba_core::rom_data::dma_register_directories::{DIRECTORY_LENGTH, WRITE_ROOT};
                let start = WRITE_ROOT as usize;
                machine.bus.rom[start..start + DIRECTORY_LENGTH * 2].copy_from_slice(&rom[start..start + DIRECTORY_LENGTH * 2]);
            }
            if index == 1 && poison_iwram_helpers {
                use gba_core::rom_data::iwram_memory_helpers::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_tilemap_descriptor {
                use gba_core::rom_data::observed_tilemap_descriptor::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_extra_tilemap_descriptors {
                use gba_core::rom_data::observed_tilemap_descriptor::{ADDITIONAL_SPECS, SOURCE_LENGTH, WRAPPER_LENGTH};
                for &(offset, _, wrapper, _) in &ADDITIONAL_SPECS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
                    let start = wrapper as usize;
                    machine.bus.rom[start..start + WRAPPER_LENGTH].copy_from_slice(&rom[start..start + WRAPPER_LENGTH]);
                }
            }
            if index == 1 && poison_transfer_records {
                use gba_core::rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
                for &(offset, _, wrapper, _) in &descriptor::TRANSFER_SPECS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + descriptor::SOURCE_LENGTH].copy_from_slice(&rom[start..start + descriptor::SOURCE_LENGTH]);
                    let start = wrapper as usize;
                    machine.bus.rom[start..start + descriptor::WRAPPER_LENGTH].copy_from_slice(&rom[start..start + descriptor::WRAPPER_LENGTH]);
                }
                for &(offset, _, _) in &row::TRANSFER_SPECS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + row::SOURCE_LENGTH].copy_from_slice(&rom[start..start + row::SOURCE_LENGTH]);
                }
            }
            if index == 1 && poison_tilemap_row {
                use gba_core::rom_data::observed_tilemap_row::{SOURCE_LENGTH, SOURCE_OFFSET};
                let start = SOURCE_OFFSET as usize;
                machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
            }
            if index == 1 && poison_tilemap_extra_rows {
                use gba_core::rom_data::observed_tilemap_row::{ADDITIONAL_SPECS, SOURCE_LENGTH};
                for &(offset, _, _) in &ADDITIONAL_SPECS {
                    let start = offset as usize;
                    machine.bus.rom[start..start + SOURCE_LENGTH].copy_from_slice(&rom[start..start + SOURCE_LENGTH]);
                }
            }
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
    if poison_constructor64 {
        let stats = decoded.bus.rom_resources().unwrap().constructor_template64_stats().expect("constructor template owner");
        assert!(stats.source_bytes > 0, "scene must consume the managed constructor template");
        eprintln!("ROM_DATA_CONSTRUCTOR64_POISON bus_reads={} source_bytes={}", stats.bus_reads, stats.source_bytes);
    }
    if poison_observed_tilemap {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_stats().expect("observed tilemap owner");
        assert!(stats.source_bytes > 0, "scene must consume the managed tilemap");
        eprintln!("ROM_DATA_OBSERVED_TILEMAP_POISON bus_reads={} source_bytes={}", stats.bus_reads, stats.source_bytes);
    }
    if poison_tilemap_rectangles {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_stats().expect("observed tilemap owner");
        assert!(stats.rectangle_bytes[0] >= 61 && stats.rectangle_bytes[1] >= 25,
            "scene must consume both managed tilemap rectangles");
        eprintln!("ROM_DATA_TILEMAP_RECTANGLES_POISON rectangle_bytes={:?}", stats.rectangle_bytes);
    }
    if poison_tilemap_alternates {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_stats().expect("observed tilemap owner");
        assert!(stats.alternate_rectangle_bytes[0] >= 2 && stats.alternate_selector_bytes > 0,
            "scene must consume the normal alternate rectangle and selector");
        eprintln!("ROM_DATA_TILEMAP_ALTERNATES_POISON rectangle_bytes={:?} selector_bytes={}", stats.alternate_rectangle_bytes, stats.alternate_selector_bytes);
    }
    if poison_tilemap_roots {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_stats().expect("observed tilemap owner");
        assert!(stats.root_bytes[0] >= 13 && stats.root_bytes[1] >= 4,
            "scene must consume both managed tilemap roots");
        eprintln!("ROM_DATA_TILEMAP_ROOTS_POISON root_bytes={:?}", stats.root_bytes);
    }
    if poison_tilemap_list_row {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_row_stats().expect("observed tilemap row owner");
        assert!(stats.list_source_bytes >= 6, "scene must consume the managed rectangle list row");
        eprintln!("ROM_DATA_TILEMAP_LIST_ROW_POISON source_bytes={}", stats.list_source_bytes);
    }
    if poison_compact_records {
        let stats = decoded.bus.rom_resources().unwrap().compact_obj_catalog_stats().expect("compact catalog owner");
        assert!(stats.red_spring_record_bytes.iter().all(|&bytes| bytes >= 20),
            "scene must consume each managed Red Spring compact record");
        eprintln!("ROM_DATA_COMPACT_RECORDS_POISON source_bytes={:?}", stats.red_spring_record_bytes);
    }
    if poison_red_spring_sounds {
        let stats = decoded.bus.rom_resources().unwrap().audio_directory_stats().expect("audio directory owner");
        assert!(stats.red_spring_sound_bytes[0] >= 14 && stats.red_spring_sound_bytes[1] >= 35,
            "scene must consume both sound tables");
        assert!(stats.red_spring_root_bytes.iter().all(|&bytes| bytes >= 4),
            "scene must consume both sound table pointers");
        eprintln!("ROM_DATA_RED_SPRING_SOUNDS_POISON table_bytes={:?} root_bytes={:?}",
            stats.red_spring_sound_bytes, stats.red_spring_root_bytes);
    }
    if poison_iwram_obj_callbacks {
        let stats = decoded.bus.rom_resources().unwrap().obj_list_stats().expect("OBJ list owner");
        assert!(stats.iwram_callback_bytes.iter().all(|&bytes| bytes >= 4),
            "scene must consume all five IWRAM callback words");
        eprintln!("ROM_DATA_IWRAM_OBJ_CALLBACKS_POISON source_bytes={:?}", stats.iwram_callback_bytes);
    }
    if poison_extra_iwram_obj_callbacks {
        let stats = decoded.bus.rom_resources().unwrap().obj_list_stats().expect("OBJ list owner");
        assert!(stats.iwram_extra_callback_bytes.iter().all(|&bytes| bytes >= 4),
            "scene must consume both extra IWRAM callback words");
        eprintln!("ROM_DATA_EXTRA_IWRAM_OBJ_CALLBACKS_POISON source_bytes={:?}", stats.iwram_extra_callback_bytes);
    }
    if poison_records4_callbacks {
        let stats = decoded.bus.rom_resources().unwrap().records4_stats().expect("records4 owner");
        assert!(stats.callback_bytes.iter().all(|&bytes| bytes >= 4),
            "scene must consume all three records4 callback tables");
        eprintln!("ROM_DATA_RECORDS4_CALLBACKS_POISON source_bytes={:?}", stats.callback_bytes);
    }
    if poison_action_callbacks {
        let stats = decoded.bus.rom_resources().unwrap().records4_stats().expect("records4 owner");
        assert!(stats.action_field_bytes.iter().all(|&bytes| bytes > 0),
            "scene must consume all three action-callback fields");
        eprintln!("ROM_DATA_ACTION_CALLBACKS_POISON source_bytes={:?}", stats.action_field_bytes);
    }
    if poison_offset_tables {
        let stats = decoded.bus.rom_resources().unwrap().adjacent_offset_tables_stats().expect("offset table owner");
        assert!([0usize, 1, 2, 4].iter().all(|&index| stats.table_bytes[index] > 0),
            "scene must consume the four observed offset tables");
        eprintln!("ROM_DATA_OFFSET_TABLES_POISON source_bytes={:?}", stats.table_bytes);
    }
    if poison_obj_layout_offsets {
        let stats = decoded.bus.rom_resources().unwrap().obj_list_stats().expect("OBJ list owner");
        assert!(stats.layout_offset_bytes[1] >= 2, "scene must consume the observed OBJ byte group");
        eprintln!("ROM_DATA_OBJ_LAYOUT_OFFSETS_POISON source_bytes={:?}", stats.layout_offset_bytes);
    }
    if poison_dma_directories {
        let stats = decoded.bus.rom_resources().unwrap().dma_register_directories_stats().expect("DMA directory owner");
        assert!(stats.directory_bytes.iter().all(|&bytes| bytes >= 12),
            "scene must consume both DMA directories");
        eprintln!("ROM_DATA_DMA_DIRECTORIES_POISON source_bytes={:?}", stats.directory_bytes);
    }
    if poison_iwram_helpers {
        let stats = decoded.bus.rom_resources().unwrap().iwram_memory_helpers_stats().expect("IWRAM helper owner");
        assert!(stats.helper_bytes.iter().all(|&bytes| bytes >= 4),
            "scene must consume all three IWRAM helper pointers");
        eprintln!("ROM_DATA_IWRAM_HELPERS_POISON source_bytes={:?}", stats.helper_bytes);
    }
    if poison_tilemap_descriptor {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_descriptor_stats().expect("tilemap descriptor owner");
        assert!(stats.source_bytes > 0, "scene must consume the managed tilemap descriptor");
        eprintln!("ROM_DATA_TILEMAP_DESCRIPTOR_POISON bus_reads={} source_bytes={}", stats.bus_reads, stats.source_bytes);
    }
    if poison_extra_tilemap_descriptors {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_descriptor_stats().expect("tilemap descriptor owner");
        assert!(stats.descriptor_bytes >= 16 && stats.wrapper_bytes >= 8, "scene must consume the managed descriptors and LZ wrappers");
        eprintln!("ROM_DATA_EXTRA_TILEMAP_DESCRIPTORS_POISON bus_reads={} descriptor_bytes={} wrapper_bytes={}", stats.bus_reads, stats.descriptor_bytes, stats.wrapper_bytes);
    }
    if poison_transfer_records {
        let resources = decoded.bus.rom_resources().unwrap();
        let descriptor = resources.observed_tilemap_descriptor_stats().expect("transfer descriptor owner");
        let row = resources.observed_tilemap_row_stats().expect("transfer row owner");
        assert!(descriptor.transfer_descriptor_bytes >= 16 && descriptor.transfer_wrapper_bytes >= 6 && row.transfer_source_bytes >= 36,
            "scene must consume the selected transfer rows, descriptors and wrappers");
        eprintln!("ROM_DATA_TRANSFER_RECORDS_POISON row_bytes={} descriptor_bytes={} wrapper_bytes={}",
            row.transfer_source_bytes, descriptor.transfer_descriptor_bytes, descriptor.transfer_wrapper_bytes);
    }
    if poison_tilemap_row {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_row_stats().expect("tilemap row owner");
        assert!(stats.source_bytes >= 12, "scene must consume the managed tilemap row");
        eprintln!("ROM_DATA_TILEMAP_ROW_POISON bus_reads={} source_bytes={}", stats.bus_reads, stats.source_bytes);
    }
    if poison_tilemap_extra_rows {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_row_stats().expect("tilemap row owner");
        assert!(stats.source_bytes >= 48, "scene must consume the managed tilemap rows");
        eprintln!("ROM_DATA_TILEMAP_EXTRA_ROWS_POISON bus_reads={} source_bytes={}", stats.bus_reads, stats.source_bytes);
    }
    if poison_blue_screw_chain {
        let resources = decoded.bus.rom_resources().unwrap();
        let rows = resources.observed_tilemap_row_stats().expect("tilemap row owner");
        let descriptors = resources.observed_tilemap_descriptor_stats().expect("tilemap descriptor owner");
        let tilemap = resources.observed_tilemap_stats().expect("tilemap owner");
        let lz = resources.observed_lz_stats().expect("LZ owner");
        assert!(rows.interleaved_lz_row_bytes >= 12 && rows.direct_tilemap_row_bytes >= 12,
            "scene must consume both selected rows: {rows:?}");
        assert!(descriptors.interleaved_lz_descriptor_bytes >= 5
            && descriptors.additional_tilemap_descriptor_bytes >= 7
            && descriptors.interleaved_lz_wrapper_bytes >= 2,
            "scene must consume both selected descriptors and the LZ wrapper: {descriptors:?}");
        assert!(tilemap.additional_stream_bytes[0] >= 100, "scene must consume the selected tilemap: {tilemap:?}");
        assert!(lz.interleaved_hle_calls_by_block[0] > 0, "scene must consume the selected decoded LZ block: {lz:?}");
        eprintln!("ROM_DATA_BLUE_SCREW_CHAIN_POISON rows={rows:?} descriptors={descriptors:?} tilemap={tilemap:?} lz={lz:?}");
    }
    if poison_additional_rectangles {
        let stats = decoded.bus.rom_resources().unwrap().observed_tilemap_stats().expect("tilemap owner");
        assert!(stats.additional_rectangle_pointer_bytes >= 4 && stats.additional_rectangle_bytes[0] >= 25,
            "scene must consume the observed additional pointer and rectangle: {stats:?}");
        eprintln!("ROM_DATA_ADDITIONAL_RECTANGLES_POISON pointers={} rectangles={:?}",
            stats.additional_rectangle_pointer_bytes, stats.additional_rectangle_bytes);
    }
    if poison_relative_graphics {
        let stats = decoded.bus.rom_resources().unwrap().obj_graphics_stats().expect("custom graphics owner");
        let expected = (1u64 << (69 - 64)) | (1u64 << (71 - 64)) | (1u64 << (73 - 64));
        assert!(stats.relative_directory_bytes > 0 && stats.relative_bank_bytes > 0
            && stats.relative_literal_bytes > 0 && stats.relative_banks_used[1] & expected == expected,
            "home scene must consume all three observed relative streams and directory: {stats:?}");
        eprintln!("ROM_DATA_RELATIVE_GRAPHICS_POISON directory_bytes={} bank_bytes={} literal_bytes={} banks={:?}",
            stats.relative_directory_bytes, stats.relative_bank_bytes, stats.relative_literal_bytes, stats.relative_banks_used);
    }
    if poison_color_indexed {
        let stats = decoded.bus.rom_resources().unwrap().indexed_graphics_stats().expect("indexed graphics owner");
        let expected = (1u64 << (125 - 64)) | (1u64 << (127 - 64));
        assert!(stats.color_source_bytes > 0 && stats.color_archives_mask & 1 != 0
            && stats.color_programs_mask[1] & expected == expected
            && stats.color_programs_mask[2] & (1 << (129 - 128)) != 0,
            "home scene must consume the observed color archive programs: {stats:?}");
        eprintln!("ROM_DATA_COLOR_INDEXED_POISON source_bytes={} archives={} programs={:?}",
            stats.color_source_bytes, stats.color_archives_mask, stats.color_programs_mask);
    }
    if poison_eight_row_lookup {
        let stats = decoded.bus.rom_resources().unwrap().tilemap_geometry_stats().expect("tilemap geometry owner");
        assert!(stats.eight_row_lookup_bytes > 0 && stats.eight_row_lookup_rows == 4,
            "home scene must consume all observed eight-row lookup rows: {stats:?}");
        eprintln!("ROM_DATA_EIGHT_ROW_LOOKUP_POISON source_bytes={} rows={}",
            stats.eight_row_lookup_bytes, stats.eight_row_lookup_rows);
    }
    if poison_task_signed_lookup {
        let stats = decoded.bus.rom_resources().unwrap().observed_task_parameters_stats().expect("task parameter owner");
        assert!(stats.signed_lookup_bytes > 0 && stats.signed_lookup_indices_mask & 6 == 6,
            "home scene must consume the two observed signed lookup indices: {stats:?}");
        eprintln!("ROM_DATA_TASK_SIGNED_LOOKUP_POISON source_bytes={} indices={}",
            stats.signed_lookup_bytes, stats.signed_lookup_indices_mask);
    }
    if poison_battle_obj {
        let stats = decoded.bus.rom_resources().unwrap().obj_animation_stats().expect("OBJ animation owner");
        assert!(stats.battle_source_bytes > 0, "battle must consume the selected OBJ bank: {stats:?}");
        eprintln!("ROM_DATA_BATTLE_OBJ_POISON source_bytes={} host_reads={}",
            stats.battle_source_bytes, stats.battle_host_reads);
    }
    if poison_battle_obj_support {
        let stats = decoded.bus.rom_resources().unwrap().obj_list_stats().expect("OBJ list owner");
        assert!(stats.actor_lookup_extension_bytes > 0
            && stats.iwram_additional_callback_bytes[3] > 0
            && stats.iwram_additional_callback_bytes[4] > 0,
            "battle must consume the observed lookup entries and callback words: {stats:?}");
        eprintln!("ROM_DATA_BATTLE_OBJ_SUPPORT_POISON lookup_bytes={} callback_bytes={:?}",
            stats.actor_lookup_extension_bytes, stats.iwram_additional_callback_bytes);
    }
    if poison_indexed_records16 {
        let stats = decoded.bus.rom_resources().unwrap().indexed_records16_stats().expect("indexed records owner");
        assert!(stats.source_bytes > 0, "battle must consume indexed record fields: {stats:?}");
        eprintln!("ROM_DATA_INDEXED_RECORDS16_POISON source_bytes={} fields={} records={:?}",
            stats.source_bytes, stats.fields_mask, stats.record_mask);
    }
    if poison_soil_remaining_switches {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        let mask = ((1u64 << 9) - 1) << 23;
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & mask == mask,
            "Soil Evidence scene must consume every selected switch: {stats:?}");
        eprintln!("ROM_DATA_SOIL_REMAINING_SWITCHES_POISON source_bytes={} switch_mask={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_soil_a83c2_switch {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & (1 << 22) != 0,
            "Soil Evidence scene must consume the selected switch: {stats:?}");
        eprintln!("ROM_DATA_SOIL_A83C2_SWITCH_POISON source_bytes={} switch_mask={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_soil_compact {
        let stats = decoded.bus.rom_resources().unwrap().compact_obj_catalog_stats().expect("compact catalog owner");
        assert_eq!(stats.soil_metadata_mask, [(1u64 << 44) - 1; 2],
            "Soil Evidence scene must consume both complete selected metadata spans");
        assert!(stats.soil_metadata_bytes.iter().all(|&count| count > 0));
        eprintln!("ROM_DATA_SOIL_COMPACT_POISON source_bytes={:?} masks={:?}", stats.soil_metadata_bytes, stats.soil_metadata_mask);
    }
    if poison_firaga_sprite_lists {
        let owner = decoded.bus.rom_resources().unwrap();
        let mut guest = 0;
        let mut host = 0;
        for (offset, _, _) in gba_core::rom_data::obj_lists::FIRAGA_SPRITE_LISTS {
            let (used_guest, used_host) = owner.obj_list_source_used(offset).expect("Firaga OBJ list owner");
            guest += usize::from(used_guest);
            host += usize::from(used_host);
        }
        assert_eq!(host, gba_core::rom_data::obj_lists::FIRAGA_SPRITE_LISTS.len(),
            "Firaga scene must consume each selected list on the host");
        eprintln!("ROM_DATA_FIRAGA_SPRITE_LISTS_POISON guest_lists={} host_lists={}", guest, host);
    }
    if poison_soil_sprite_lists {
        let owner = decoded.bus.rom_resources().unwrap();
        let mut guest = 0;
        let mut host = 0;
        for (offset, _, _) in gba_core::rom_data::obj_lists::SOIL_EVIDENCE_LISTS {
            let (used_guest, used_host) = owner.obj_list_source_used(offset).expect("Soil Evidence OBJ list owner");
            guest += usize::from(used_guest);
            host += usize::from(used_host);
        }
        assert_eq!(host, gba_core::rom_data::obj_lists::SOIL_EVIDENCE_LISTS.len(),
            "Soil Evidence scene must consume each selected list on the host");
        eprintln!("ROM_DATA_SOIL_SPRITE_LISTS_POISON guest_lists={} host_lists={}", guest, host);
    }
    if poison_soil_e3838 {
        let stats = decoded.bus.rom_resources().unwrap().indexed_words_3959_stats().expect("indexed words owner");
        assert!(stats.soil_e3838_source_bytes > 0 && stats.soil_e3838_record_mask == (1u64 << 30) - 1,
            "Soil Evidence scene must consume all sampled indexed words: {stats:?}");
        eprintln!("ROM_DATA_SOIL_E3838_POISON source_bytes={} records={}",
            stats.soil_e3838_source_bytes, stats.soil_e3838_record_mask);
    }
    if poison_indexed_words_3959 {
        let stats = decoded.bus.rom_resources().unwrap().indexed_words_3959_stats().expect("indexed words owner");
        assert!(stats.source_bytes > 0, "battle must consume indexed word fields: {stats:?}");
        eprintln!("ROM_DATA_INDEXED_WORDS_3959_POISON source_bytes={} fields={} records={:?}",
            stats.source_bytes, stats.fields_mask, stats.record_mask);
    }
    if poison_thumb_switch_tables {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.source_bytes > 0 && stats.roots_mask != 0 && stats.destinations_mask != 0,
            "battle must consume Thumb switch roots and destinations: {stats:?}");
        eprintln!("ROM_DATA_THUMB_SWITCH_TABLES_POISON source_bytes={} roots={} destinations={}",
            stats.source_bytes, stats.roots_mask, stats.destinations_mask);
    }
    if poison_additional_thumb_switches {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & 0xf == 0xf,
            "battle must consume all four additional Thumb switch tables: {stats:?}");
        eprintln!("ROM_DATA_ADDITIONAL_THUMB_SWITCHES_POISON source_bytes={} tables={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_final_copy {
        let stats = decoded.bus.rom_resources().unwrap().copied_halfword_lookups_stats().expect("copied halfword owner");
        assert!(stats.table_bytes > 0 && stats.table_mask & 4 == 4,
            "Red Spring scene must consume the final copy source: {stats:?}");
        eprintln!("ROM_DATA_RED_SPRING_FINAL_COPY_POISON table_bytes={} tables={}",
            stats.table_bytes, stats.table_mask);
    }
    if poison_firaga_copied_matrices {
        let stats=decoded.bus.rom_resources().unwrap().firaga_copied_matrices_stats().expect("Firaga copied matrix owner");
        assert!(stats.matrix_bytes.iter().all(|&n|n>0),"scene must consume both matrices: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_COPIED_MATRICES_POISON matrices={:?} literals={:?}",stats.matrix_bytes,stats.literal_bytes);
    }
    if poison_firaga_compact_cells {
        let stats=decoded.bus.rom_resources().unwrap().firaga_compact_cells_stats().expect("Firaga compact cell owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0),"scene must consume all compact sources: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_COMPACT_CELLS_POISON source_bytes={:?}",stats.source_bytes);
    }
    if poison_firaga_indexed_obj {
        let owner=decoded.bus.rom_resources().unwrap();
        let stats=owner.firaga_indexed_obj_bank_stats().expect("Firaga indexed OBJ owner");
        let empty=owner.obj_list_source_used(gba_core::rom_data::obj_lists::FIRAGA_EMPTY_LIST.0).expect("Firaga empty list owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0) && empty.0,"scene must consume indexed sources and empty list: {stats:?} {empty:?}");
        eprintln!("ROM_DATA_FIRAGA_INDEXED_OBJ_POISON source_bytes={:?} empty_guest={} empty_host={}",stats.source_bytes,empty.0,empty.1);
    }
    if poison_firaga_compact_obj {
        let owner=decoded.bus.rom_resources().unwrap();
        let stats=owner.firaga_compact_obj_bank_stats().expect("Firaga compact OBJ owner");
        let empty=gba_core::rom_data::obj_lists::FIRAGA_COMPACT_EMPTY_LISTS.map(|(at,_,_)|owner.obj_list_source_used(at).expect("Firaga empty list owner"));
        assert!(stats.source_bytes.iter().all(|&n|n>0) && empty.iter().all(|entry|entry.0),"scene must consume compact sources and empty lists: {stats:?} {empty:?}");
        eprintln!("ROM_DATA_FIRAGA_COMPACT_OBJ_POISON source_bytes={:?} empty={:?}",stats.source_bytes,empty);
    }
    if poison_firaga_adjacent_compact_obj {
        let owner=decoded.bus.rom_resources().unwrap();
        let stats=owner.firaga_adjacent_compact_obj_bank_stats().expect("Adjacent Firaga compact OBJ owner");
        let empty=owner.obj_list_source_used(gba_core::rom_data::obj_lists::FIRAGA_ADJACENT_EMPTY_LIST.0).expect("Adjacent Firaga empty list owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0) && empty.0,"scene must consume adjacent compact sources and empty list: {stats:?} {empty:?}");
        eprintln!("ROM_DATA_FIRAGA_ADJACENT_COMPACT_OBJ_POISON source_bytes={:?} empty_guest={} empty_host={}",stats.source_bytes,empty.0,empty.1);
    }
    if poison_firaga_6c12c8_obj {
        let owner=decoded.bus.rom_resources().unwrap();
        let stats=owner.firaga_6c12c8_obj_bank_stats().expect("Firaga 6c12c8 OBJ owner");
        let empty=owner.obj_list_source_used(gba_core::rom_data::obj_lists::FIRAGA_6C12C8_EMPTY_LIST.0).expect("Firaga 6c12c8 empty list owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0) && empty.0,"scene must consume Firaga 6c12c8 sources and empty list: {stats:?} {empty:?}");
        eprintln!("ROM_DATA_FIRAGA_6C12C8_OBJ_POISON source_bytes={:?} empty_guest={} empty_host={}",stats.source_bytes,empty.0,empty.1);
    }
    if poison_firaga_transfer {
        let stats=decoded.bus.rom_resources().unwrap().firaga_transfer_program_stats().expect("Firaga transfer owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0),"scene must consume each selected transfer source: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_TRANSFER_POISON source_bytes={:?}",stats.source_bytes);
    }
    if poison_firaga_palette_rows {
        let stats=decoded.bus.rom_resources().unwrap().firaga_palette_rows_stats().expect("Firaga palette rows owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0),"scene must consume both selected palette row prefixes: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_PALETTE_ROWS_POISON source_bytes={:?}",stats.source_bytes);
    }
    if poison_firaga_stack {
        let stats=decoded.bus.rom_resources().unwrap().firaga_stack_initialization_stats().expect("Firaga stack owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0),"scene must consume each selected stack source: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_STACK_POISON source_bytes={:?}",stats.source_bytes);
    }
    if poison_firaga_queued {
        let stats=decoded.bus.rom_resources().unwrap().firaga_queued_transfer_stats().expect("Firaga queued transfer owner");
        assert!(stats.source_bytes.iter().all(|&n|n>0),"scene must consume each selected queued source: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_QUEUED_POISON source_bytes={:?}",stats.source_bytes);
    }
    if poison_soil_remaining_rectangles {
        let stats=decoded.bus.rom_resources().unwrap().soil_remaining_rectangles_stats().expect("Soil remaining rectangle owner");
        assert!(stats.rectangle_bytes.iter().all(|&n|n>0),"scene must consume both rectangles: {stats:?}");
        eprintln!("ROM_DATA_SOIL_REMAINING_RECTANGLES_POISON rectangles={:?} literal_bytes={}",stats.rectangle_bytes,stats.literal_bytes);
    }
    if poison_soil_scoped_values {
        let stats = decoded.bus.rom_resources().unwrap().soil_scoped_values_stats().expect("Soil scoped values owner");
        assert!(stats.source_bytes > 0 && stats.values_mask & 0x3f == 0x3f, "scene must consume all six witnessed values: {stats:?}");
        eprintln!("ROM_DATA_SOIL_SCOPED_VALUES_POISON source_bytes={} values={}", stats.source_bytes, stats.values_mask);
    }
    if poison_soil_compact_record {
        let stats = decoded.bus.rom_resources().unwrap().soil_compact_record_stats().expect("Soil compact record owner");
        assert!(stats.record_bytes > 0 && stats.pointer_bytes > 0, "scene must consume the record and pointer: {stats:?}");
        eprintln!("ROM_DATA_SOIL_COMPACT_RECORD_POISON record_bytes={} pointer_bytes={}", stats.record_bytes, stats.pointer_bytes);
    }
    if poison_soil_indexed_bytes {
        let stats = decoded.bus.rom_resources().unwrap().soil_indexed_bytes_stats().expect("Soil indexed byte owner");
        assert!(stats.source_bytes > 0 && stats.values_mask & 0x3f == 0x3f, "scene must consume all six witnessed bytes: {stats:?}");
        eprintln!("ROM_DATA_SOIL_INDEXED_BYTES_POISON source_bytes={} values={}", stats.source_bytes, stats.values_mask);
    }
    if poison_soil_indirect_records {
        let stats = decoded.bus.rom_resources().unwrap().soil_indirect_records_stats().expect("Soil indirect record owner");
        assert!(stats.source_bytes > 0 && stats.record_mask & 0x1f == 0x1f, "scene must consume all five witnessed records: {stats:?}");
        eprintln!("ROM_DATA_SOIL_INDIRECT_RECORDS_POISON source_bytes={} records={}", stats.source_bytes, stats.record_mask);
    }
    if poison_soil_scene_records {
        let stats = decoded.bus.rom_resources().unwrap().soil_scene_records_stats().expect("Soil scene record owner");
        assert!(stats.range_bytes > 0 && stats.sequence_bytes > 0 && stats.range_mask & 0x15 == 0x15
            && stats.sequence_mask & 0x3f == 0x3f, "scene must consume all witnessed record groups: {stats:?}");
        eprintln!("ROM_DATA_SOIL_SCENE_RECORDS_POISON range_bytes={} sequence_bytes={} ranges={} sequence={}",
            stats.range_bytes, stats.sequence_bytes, stats.range_mask, stats.sequence_mask);
    }
    if poison_soil_other_transfer {
        let owner = decoded.bus.rom_resources().unwrap();
        let rows = owner.observed_tilemap_row_stats().expect("Soil list and mode-7 row owner");
        let descriptor = owner.observed_tilemap_descriptor_stats().expect("Soil mode-7 descriptor owner");
        let tilemap = owner.observed_tilemap_stats().expect("Soil list and mode-7 source owner");
        assert_eq!(rows.soil_other_row_mask & 3, 3, "scene must consume both selected rows: {rows:?}");
        assert!(descriptor.soil_mode7_descriptor_bytes > 0 && tilemap.soil_list_bytes > 0 && tilemap.soil_entry_bytes > 0
            && tilemap.soil_rectangle_bytes.iter().all(|&n| n > 0) && tilemap.soil_mode7_header_bytes > 0
            && tilemap.soil_mode7_run_bytes > 0, "scene must consume all selected source groups: {descriptor:?} {tilemap:?}");
        eprintln!("ROM_DATA_SOIL_OTHER_TRANSFER_POISON rows={} descriptor={} list={} entries={} rectangles={:?} header={} run={}",
            rows.soil_other_row_mask, descriptor.soil_mode7_descriptor_bytes, tilemap.soil_list_bytes, tilemap.soil_entry_bytes,
            tilemap.soil_rectangle_bytes, tilemap.soil_mode7_header_bytes, tilemap.soil_mode7_run_bytes);
    }
    if poison_soil_transfer {
        let owner = decoded.bus.rom_resources().unwrap();
        let rows = owner.observed_tilemap_row_stats().expect("Soil transfer row owner");
        let descriptors = owner.observed_tilemap_descriptor_stats().expect("Soil transfer descriptor owner");
        assert_eq!(rows.soil_row_mask & 0xf, 0xf, "scene must consume four selected rows: {rows:?}");
        assert_eq!(descriptors.soil_descriptor_mask & 0xf, 0xf, "scene must consume four selected descriptors: {descriptors:?}");
        assert_eq!(descriptors.soil_wrapper_mask & 0xd, 0xd, "scene must consume three new wrappers: {descriptors:?}");
        eprintln!("ROM_DATA_SOIL_TRANSFER_POISON rows={} descriptors={} wrappers={}", rows.soil_row_mask, descriptors.soil_descriptor_mask, descriptors.soil_wrapper_mask);
    }
    if poison_soil_copied_halfwords {
        let stats = decoded.bus.rom_resources().unwrap().copied_halfword_lookups_stats().expect("copied halfword owner");
        assert!(stats.table_bytes > 0 && stats.table_mask & 8 != 0 && stats.soil_source_mask == 0xff,
            "Soil Evidence scene must consume all eight selected source bytes: {stats:?}");
        eprintln!("ROM_DATA_SOIL_COPIED_HALFWORDS_POISON table_bytes={} source_mask={}", stats.table_bytes, stats.soil_source_mask);
    }
    if poison_copied_halfword_lookups {
        let stats = decoded.bus.rom_resources().unwrap().copied_halfword_lookups_stats().expect("copied halfword owner");
        assert!(stats.table_bytes > 0 && stats.table_mask & 1 != 0,
            "battle must consume the first copied halfword table: {stats:?}");
        eprintln!("ROM_DATA_COPIED_HALFWORD_LOOKUPS_POISON table_bytes={} root_bytes={} tables={} roots={}",
            stats.table_bytes, stats.root_bytes, stats.table_mask, stats.root_mask);
    }
    if poison_indexed_halfwords_39a5 {
        let stats = decoded.bus.rom_resources().unwrap().indexed_halfwords_39a5_stats().expect("indexed halfword owner");
        assert!(stats.source_bytes > 0 && stats.values_mask != 0,
            "battle must consume an indexed halfword: {stats:?}");
        eprintln!("ROM_DATA_INDEXED_HALFWORDS_39A5_POISON source_bytes={} values={}", stats.source_bytes, stats.values_mask);
    }
    if poison_script_byte_lookup_3ad {
        let stats = decoded.bus.rom_resources().unwrap().script_byte_lookup_3ad_stats().expect("script byte lookup owner");
        assert!(stats.table_bytes > 0 && stats.values_mask & 2 != 0,
            "battle must consume the observed script lookup entry: {stats:?}");
        eprintln!("ROM_DATA_SCRIPT_BYTE_LOOKUP_3AD_POISON table_bytes={} root_bytes={} values={}", stats.table_bytes, stats.root_bytes, stats.values_mask);
    }
    if poison_obj_derived_dimensions {
        let stats = decoded.bus.rom_resources().unwrap().stats();
        assert!(stats.derived_source_bytes > 0 && stats.derived_table_mask == 3 && stats.derived_rows_mask.iter().all(|&m| m != 0),
            "battle must consume both derived OBJ dimension tables: {stats:?}");
        eprintln!("ROM_DATA_OBJ_DERIVED_DIMENSIONS_POISON source_bytes={} tables={} rows={:?}",
            stats.derived_source_bytes, stats.derived_table_mask, stats.derived_rows_mask);
    }
    if poison_thumb_pc_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.source_bytes > 0 && stats.literals_mask.iter().any(|&mask| mask != 0),
            "battle must consume Thumb PC literals: {stats:?}");
        eprintln!("ROM_DATA_THUMB_PC_LITERALS_POISON source_bytes={} literals={:?}",
            stats.source_bytes, stats.literals_mask);
    }
    if poison_additional_thumb_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_mask.iter().any(|&mask| mask != 0),
            "battle must consume additional Thumb literals: {stats:?}");
        eprintln!("ROM_DATA_ADDITIONAL_THUMB_LITERALS_POISON source_bytes={} literals={:?}",
            stats.additional_source_bytes, stats.additional_mask);
    }
    if poison_observed_thumb_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.observed_source_bytes > 0 && stats.observed_mask.iter().any(|&mask| mask != 0),
            "battle must consume observed Thumb literals: {stats:?}");
        eprintln!("ROM_DATA_OBSERVED_THUMB_LITERALS_POISON source_bytes={} literals={:?}",
            stats.observed_source_bytes, stats.observed_mask);
    }
    if poison_home_anchored_thumb_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.home_anchored_bytes > 0 && stats.home_anchored_mask == (1u64 << 25) - 1,
            "home scene must consume the sampled anchored literals: {stats:?}");
        eprintln!("ROM_DATA_HOME_ANCHORED_THUMB_LITERALS_POISON source_bytes={} literals={}",
            stats.home_anchored_bytes, stats.home_anchored_mask);
    }
    if poison_home_residual_code_support {
        let owner = decoded.bus.rom_resources().unwrap();
        let literals = owner.thumb_pc_literals_stats().expect("Thumb literal owner");
        let switches = owner.thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(literals.home_observed_bytes > 0 && literals.home_observed_mask == (1u64 << 40) - 1,
            "home scene must consume the scoped literals: {literals:?}");
        assert!(switches.additional_source_bytes > 0 && switches.additional_destinations_mask & 0x30 == 0x30,
            "home scene must consume both scoped switch tables: {switches:?}");
        eprintln!("ROM_DATA_HOME_RESIDUAL_CODE_SUPPORT_POISON literal_bytes={} literals={} switch_bytes={} switches={}",
            literals.home_observed_bytes, literals.home_observed_mask, switches.additional_source_bytes, switches.additional_destinations_mask);
    }
    if poison_red_spring_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.red_spring_observed_bytes > 0 && stats.red_spring_observed_mask == [u64::MAX, u64::MAX, u64::MAX, 0xf],
            "Red Spring scene must consume all sampled literal words: {stats:?}");
        eprintln!("ROM_DATA_RED_SPRING_LITERALS_POISON source_bytes={} literals={:?}",
            stats.red_spring_observed_bytes, stats.red_spring_observed_mask);
    }
    if poison_red_spring_switches {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & 0xffc0 == 0xffc0,
            "Red Spring scene must consume each scoped switch table: {stats:?}");
        eprintln!("ROM_DATA_RED_SPRING_SWITCHES_POISON source_bytes={} tables={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_final_switches {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & 0x30000 == 0x30000,
            "Red Spring scene must consume both final switch tables: {stats:?}");
        eprintln!("ROM_DATA_RED_SPRING_FINAL_SWITCHES_POISON source_bytes={} tables={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_bank_switches {
        let stats = decoded.bus.rom_resources().unwrap().thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(stats.additional_source_bytes > 0 && stats.additional_destinations_mask & 0x1c0000 == 0x1c0000,
            "Bank scene must consume the three switch tables: {stats:?}");
        eprintln!("ROM_DATA_BANK_SWITCHES_POISON source_bytes={} tables={}",
            stats.additional_source_bytes, stats.additional_destinations_mask);
    }
    if poison_blue_screw {
        let owner = decoded.bus.rom_resources().unwrap();
        let literals = owner.thumb_pc_literals_stats().expect("Thumb literal owner");
        let switches = owner.thumb_switch_tables_stats().expect("Thumb switch owner");
        assert!(literals.blue_screw_observed_bytes > 0 && literals.blue_screw_observed_mask == 7,
            "Blue Screw scene must consume all three literal words: {literals:?}");
        assert!(switches.additional_source_bytes > 0 && switches.additional_destinations_mask & (1 << 21) != 0,
            "Blue Screw scene must consume the switch array: {switches:?}");
        eprintln!("ROM_DATA_BLUE_SCREW_SOURCES_POISON literal_bytes={} literal_mask={} switch_bytes={} switch_mask={}",
            literals.blue_screw_observed_bytes, literals.blue_screw_observed_mask,
            switches.additional_source_bytes, switches.additional_destinations_mask);
    }
    if poison_bank_observed {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.bank_observed_bytes > 0 && stats.bank_observed_mask == [u64::MAX, (1u64 << 13) - 1],
            "Bank scene must consume all observed literal words: {stats:?}");
        eprintln!("ROM_DATA_BANK_OBSERVED_POISON source_bytes={} literals={:?}",
            stats.bank_observed_bytes, stats.bank_observed_mask);
    }
    if poison_post_indexed_anchored {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.post_indexed_anchored_bytes > 0 && stats.post_indexed_anchored_mask == (1u64 << 18) - 1,
            "Post-indexed scene must consume all anchored literal words: {stats:?}");
        eprintln!("ROM_DATA_POST_INDEXED_ANCHORED_POISON source_bytes={} literals={}",
            stats.post_indexed_anchored_bytes, stats.post_indexed_anchored_mask);
    }
    if poison_post_indexed_observed {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.post_indexed_observed_bytes > 0 && stats.post_indexed_observed_mask == (1u64 << 9) - 1,
            "Post-indexed scene must consume all observed literal words: {stats:?}");
        eprintln!("ROM_DATA_POST_INDEXED_OBSERVED_POISON source_bytes={} literals={}",
            stats.post_indexed_observed_bytes, stats.post_indexed_observed_mask);
    }
    if poison_red_spring_anchored {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.red_spring_anchored_bytes > 0 && stats.red_spring_anchored_mask == (1u64 << 22) - 1,
            "Red Spring scene must consume all anchored literal words: {stats:?}");
        eprintln!("ROM_DATA_RED_SPRING_ANCHORED_POISON source_bytes={} literals={}",
            stats.red_spring_anchored_bytes, stats.red_spring_anchored_mask);
    }
    if poison_soil_146_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.soil_146_observed_bytes > 0 && stats.soil_146_observed_mask == (1u64 << 13) - 1,
            "Soil Evidence scene must consume all selected 146xxx words: {stats:?}");
        eprintln!("ROM_DATA_SOIL_146_LITERALS_POISON source_bytes={} literals={}",
            stats.soil_146_observed_bytes, stats.soil_146_observed_mask);
    }
    if poison_firaga_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_observed_bytes > 0 && stats.firaga_observed_mask == (1u64 << 7) - 1,
            "Firaga scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_observed_bytes, stats.firaga_observed_mask);
    }
    if poison_firaga_2d2e_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_2d2e_observed_bytes > 0 && stats.firaga_2d2e_observed_mask == (1u64 << 41) - 1,
            "Firaga 2d/2e scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_2D2E_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_2d2e_observed_bytes, stats.firaga_2d2e_observed_mask);
    }
    if poison_firaga_eb83_ebc_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_eb83_ebc_observed_bytes > 0 && stats.firaga_eb83_ebc_observed_mask == (1u64 << 10) - 1,
            "Firaga eb83/ebc scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_EB83_EBC_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_eb83_ebc_observed_bytes, stats.firaga_eb83_ebc_observed_mask);
    }
    if poison_firaga_code_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_code_observed_bytes > 0 && stats.firaga_code_observed_mask == (1u64 << 14) - 1,
            "Firaga code scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_CODE_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_code_observed_bytes, stats.firaga_code_observed_mask);
    }
    if poison_firaga_mixed_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_mixed_observed_bytes > 0 && stats.firaga_mixed_observed_mask == (1u64 << 57) - 1,
            "Firaga mixed scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_MIXED_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_mixed_observed_bytes, stats.firaga_mixed_observed_mask);
    }
    if poison_firaga_residual_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.firaga_residual_observed_bytes > 0 && stats.firaga_residual_observed_mask == (1u64 << 52) - 1,
            "Firaga residual scene must consume all selected literals: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_RESIDUAL_LITERALS_POISON source_bytes={} literals={}",
            stats.firaga_residual_observed_bytes, stats.firaga_residual_observed_mask);
    }
    if poison_soil_literals {
        let stats = decoded.bus.rom_resources().unwrap().thumb_pc_literals_stats().expect("Thumb literal owner");
        assert!(stats.soil_evidence_observed_bytes > 0 && stats.soil_evidence_observed_mask == [u64::MAX, u64::MAX, 0x7f],
            "Soil Evidence scene must consume all sampled literal words: {stats:?}");
        eprintln!("ROM_DATA_SOIL_LITERALS_POISON source_bytes={} literals={:?}",
            stats.soil_evidence_observed_bytes, stats.soil_evidence_observed_mask);
    }
    if poison_firaga_lz {
        let stats = decoded.bus.rom_resources().unwrap().observed_lz_stats().expect("observed LZ owner");
        assert!((0..3).all(|i| stats.firaga_source_bytes_by_block[i] > 0 || stats.firaga_hle_calls_by_block[i] > 0),
            "Firaga scene must consume all three LZ sources: {stats:?}");
        eprintln!("ROM_DATA_FIRAGA_LZ_POISON source_bytes={:?} hle_calls={:?}",
            stats.firaga_source_bytes_by_block, stats.firaga_hle_calls_by_block);
    }
    if poison_soil_lz {
        let stats = decoded.bus.rom_resources().unwrap().observed_lz_stats().expect("observed LZ owner");
        assert!((0..3).all(|i| stats.soil_source_bytes_by_block[i] > 0 || stats.soil_hle_calls_by_block[i] > 0),
            "Soil Evidence scene must consume all three LZ sources: {stats:?}");
        eprintln!("ROM_DATA_SOIL_LZ_POISON source_bytes={:?} hle_calls={:?}",
            stats.soil_source_bytes_by_block, stats.soil_hle_calls_by_block);
    }
    if poison_observed_lz {
        let stats = decoded.bus.rom_resources().unwrap().observed_lz_stats().expect("observed LZ owner");
        assert!(stats.hle_calls_by_block.iter().all(|&calls| calls > 0),
            "scene must consume each observed decoded LZ block: {:?}", stats.hle_calls_by_block);
        eprintln!("ROM_DATA_OBSERVED_LZ_POISON encoded_bytes={} hle_calls={} by_block={:?}",
            observed_lz_test_spans().map(|(_,len)| len).sum::<usize>(), stats.hle_calls, stats.hle_calls_by_block);
    }
    #[cfg(feature = "rom-data-read-audit")]
    if poison_red_spring {
        use gba_core::rom_data::read_audit::Reader;
        let bus = red_spring_reads.iter().filter(|read| read.reader == Reader::Bus).count();
        let typed_host = red_spring_reads.iter().filter(|read| read.reader == Reader::SpriteList).count();
        let raw_host = red_spring_reads.iter().filter(|read| read.reader == Reader::SpritePeek).count();
        assert!(bus > 0 && typed_host > 0 && raw_host == 0, "new bank bytes must be consumed through managed bus and sprite lists");
        eprintln!("ROM_DATA_RED_SPRING_POISON new_bytes=634 bus_requests={bus} typed_sprite_requests={typed_host} raw_sprite_requests={raw_host}");
    }
    #[cfg(feature = "rom-data-read-audit")]
    if let Some(root) = compact_watch_root {
        let (frame, hit) = compact_watch_hit.expect("scene must read represented compact bank bytes");
        let stats = decoded.bus.rom_resources().unwrap().compact_obj_catalog_stats().unwrap();
        assert!(stats.source_bytes > 0, "decoded compact owner must serve source bytes");
        eprintln!("ROM_DATA_COMPACT_BANK_WITNESS root={root:08x} frame={frame} address={:08x} width={} catalog_source_bytes={}", hit.address, hit.width, stats.source_bytes);
    }
    assert!(
        audio.iter().all(|&count| count > 0),
        "all audio streams need actual samples: {audio:?}"
    );
    if std::env::var_os("FFTA_STATE_TEST_REQUIRE_CONDITIONAL_ITEM_PARAMETERS").is_some() {
        let parameters = decoded.bus.rom_resources().unwrap().conditional_item_parameters_stats()
            .expect("decoded EUR conditional item parameters");
        assert!(parameters.bus_reads > 0 && parameters.byte01 > 0 && parameters.value02_bytes > 0
            && parameters.record_mask != 0,
            "the scene must consume item-derived parameter bytes and halfwords");
        eprintln!("ROM_DATA_CONDITIONAL_ITEM_PARAMETERS_SCENE frames={frames} bus_reads={} source_bytes={} byte00={} byte01={} value02_bytes={} record_mask={}",
            parameters.bus_reads, parameters.source_bytes, parameters.byte00, parameters.byte01, parameters.value02_bytes, parameters.record_mask);
    }
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
        eprintln!("ROM_DATA_TEXT_CATALOG_SCENE frames={frames} bus_reads={} source_bytes={} pointer_table_bytes={} string_bytes={} archive_index_bytes={} header_bytes={} encoded_control_bytes={} decoded_program_bytes={} glyph_bytes={} width_bytes={} substitution_callback_bytes={}",
            t.bus_reads,t.source_bytes,t.pointer_table_bytes,t.string_bytes,t.archive_index_bytes,t.header_bytes,t.encoded_control_bytes,t.decoded_program_bytes,t.glyph_bytes,t.width_bytes,t.substitution_callback_bytes);
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
        assert!(i.bus_reads > 0 && i.source_bytes > 0 && (i.archives_used > 0 || i.groups_used > 0 || i.context_bytes > 0) && i.selector_bytes+i.archive_header_bytes+i.index_bytes+i.token_bytes+i.literal_bytes+i.dictionary_header_bytes+i.dictionary_bytes+i.context_bytes == i.source_bytes, "ordinary scene must consume indexed-graphics storage");
        eprintln!("ROM_DATA_INDEXED_GRAPHICS_SCENE frames={frames} bus_reads={} source_bytes={} selector_bytes={} archive_header_bytes={} index_bytes={} token_bytes={} literal_bytes={} dictionary_header_bytes={} dictionary_bytes={} context_bytes={} archives_used={} programs_used={} groups_used={}",i.bus_reads,i.source_bytes,i.selector_bytes,i.archive_header_bytes,i.index_bytes,i.token_bytes,i.literal_bytes,i.dictionary_header_bytes,i.dictionary_bytes,i.context_bytes,i.archives_used,i.programs_used,i.groups_used);
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
    #[cfg(feature = "rom-data-read-audit")]
    if let Some(path) = scene_audit_path {
        write_scene_rom_audit(&path, &source.manifest, first, frames, &scene_reads);
    }
    eprintln!("ROM_DATA_SCENE frames={frames} first={first} native={} fallback={} full_states={frames} audio={audio:?} bus_reads={} host_rom_reads={} host_ram_reads={} dynamic_ram_reads={} bus_source_mask={}", dispatch[0], dispatch[1], stats.bus_reads, stats.host_rom_reads, stats.host_ram_reads, stats.dynamic_ram_reads, stats.bus_source_mask);
    eprintln!(
        "ROM_DATA_BG_SCENE frames={frames} bus_reads={} projected_bytes={}",
        bg.bus_reads, bg.projected_bytes
    );
}

#[cfg(feature = "rom-data-read-audit")]
fn write_scene_rom_audit(
    path: &Path,
    manifest: &Manifest,
    first: u64,
    frames: u64,
    reads: &std::collections::BTreeSet<gba_core::rom_data::read_audit::Read>,
) {
    use gba_core::rom_data::read_audit::Reader;
    use std::collections::{BTreeMap, BTreeSet};

    assert!(reads.iter().any(|read| read.reader == Reader::Bus));
    assert!(reads.iter().any(|read| read.reader == Reader::AudioValue));
    let mut ranges: Vec<_> = manifest.resources.iter().flat_map(|r| &r.sources)
        .map(|s| (s.offset, s.offset.checked_add(s.length).unwrap())).collect();
    ranges.sort_unstable();
    let mut owned: Vec<(u32, u32)> = Vec::new();
    for (start, end) in ranges {
        assert!(start < end && end <= manifest.source_rom_size);
        if let Some(last) = owned.last_mut().filter(|last| start <= last.1) {
            last.1 = last.1.max(end);
        } else {
            owned.push((start, end));
        }
    }
    let mut image_bytes = BTreeSet::new();
    let mut unowned_bytes = BTreeSet::new();
    let mut unowned_reads = Vec::new();
    let mut readers = BTreeMap::new();
    for read in reads {
        let end = read.physical_offset.checked_add(u32::try_from(read.length).unwrap()).unwrap();
        assert!(end <= manifest.source_rom_size);
        *readers.entry(read.reader).or_insert(0usize) += 1;
        let mut missing: Vec<(u32, u32)> = Vec::new();
        for offset in read.physical_offset..end {
            image_bytes.insert(offset);
            let index = owned.partition_point(|&(_, end)| end <= offset);
            if !owned.get(index).is_some_and(|&(start, _)| start <= offset) {
                unowned_bytes.insert(offset);
                if let Some(last) = missing.last_mut().filter(|last| last.1 == offset) {
                    last.1 += 1;
                } else {
                    missing.push((offset, offset + 1));
                }
            }
        }
        if !missing.is_empty() {
            unowned_reads.push(serde_json::json!({"read": read, "unowned_ranges": missing}));
        }
    }
    let report = serde_json::json!({
        "status": "PASS", "scope": "one recorded raw/managed scene; address-only selected readers",
        "first_guest_frame": first, "last_guest_frame": first + frames, "frames": frames,
        "extractor_revision": manifest.extractor_revision, "decoder_revision": manifest.decoder_revision,
        "resource_count": manifest.resources.len(), "source_rom_sha256": manifest.source_rom_sha256,
        "unique_reads": reads.len(), "reader_unique_requests": readers,
        "unique_image_bytes": image_bytes.len(), "unique_unowned_bytes": unowned_bytes.len(),
        "reads": reads, "unowned_reads": unowned_reads,
        "per_frame_state_clock_video_oam_dispatch_audio_equal": true,
        "limits": "Address-only selected readers; unowned bytes can be code. Interpreter fetches, folded constants, unselected host readers, other scenes and instruction attribution are outside this report.",
        "performance_measurements": false
    });
    fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}

fn clear_audio(machine: &mut gba_core::Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.hle_tap.clear();
    machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
}

fn additional_rectangle_test_spans() -> impl Iterator<Item = (usize, usize)> {
    use gba_core::rom_data::observed_tilemap::{ADDITIONAL_RECTANGLE_POINTER_COUNT, ADDITIONAL_RECTANGLE_POINTER_OFFSET, ADDITIONAL_RECTANGLE_SPECS};
    std::iter::once((ADDITIONAL_RECTANGLE_POINTER_OFFSET as usize, ADDITIONAL_RECTANGLE_POINTER_COUNT * 4))
        .chain(ADDITIONAL_RECTANGLE_SPECS.iter().map(|&(offset, _)| (offset as usize, 26)))
}

// Only the first pair was observed in the retained Blue Screw scene.
fn blue_screw_chain_test_spans() -> [(usize, usize); 7] {
    use gba_core::rom_data::{observed_lz, observed_tilemap as tilemap,
        observed_tilemap_descriptor as descriptor, observed_tilemap_row as row};
    let (lz_row, lz_target, _) = row::INTERLEAVED_LZ_ROW_SPECS[0];
    let (direct_row, direct_target, _) = row::DIRECT_ROW_SPECS[0];
    let (lz_descriptor, _, wrapper, _) = descriptor::INTERLEAVED_LZ_SPECS[0];
    let lz = observed_lz::ADDITIONAL_SPECS[6];
    let (tilemap_offset, tilemap_length, _, _) = tilemap::ADDITIONAL_STREAM_SPECS[0];
    assert_eq!(lz_target, 0x0800_0000 + lz_descriptor);
    assert_eq!(direct_target, 0x0800_0000 + descriptor::SOURCE_OFFSET + descriptor::SOURCE_LENGTH as u32);
    assert_eq!(lz.offset, wrapper + descriptor::WRAPPER_LENGTH as u32);
    [
        (lz_row as usize, row::SOURCE_LENGTH),
        (direct_row as usize, row::SOURCE_LENGTH),
        (lz_descriptor as usize, descriptor::SOURCE_LENGTH),
        ((direct_target - 0x0800_0000) as usize, descriptor::SOURCE_LENGTH),
        (wrapper as usize, descriptor::WRAPPER_LENGTH),
        (lz.offset as usize, lz.encoded_length),
        (tilemap_offset as usize, tilemap_length),
    ]
}

fn observed_lz_test_spans() -> impl Iterator<Item = (usize, usize)> {
    use gba_core::rom_data::observed_lz::{OFFSET, ENCODED_LENGTH, ADDITIONAL_SPECS};
    std::iter::once((OFFSET as usize, ENCODED_LENGTH))
        .chain(ADDITIONAL_SPECS[..13].iter().map(|s| (s.offset as usize, s.encoded_length)))
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
