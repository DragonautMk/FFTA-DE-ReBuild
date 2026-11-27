//! Versioned decoded base resources. No filesystem I/O or guest state.
//! Only exact, independently verified profiles are eligible for extraction.
use crate::ffta_rom::{self, Version};
use serde::{Deserialize, Serialize};

mod runtime;
#[cfg(feature = "rom-data-read-audit")]
pub mod read_audit;
pub use runtime::{AlternatePaletteStats, BackgroundStats, GraphicsStats, ObservedLzRead, ObservedLzStats, PacketRead, PacketStats, PaletteRead, PaletteStats, ResourceStats, RuntimeResources, TransferStats, TextStats, TextCatalogStats, FontKeyboardStats, ObjListStats, ObjAnimationStats};
pub use runtime::{CompactObjStats, SecondaryCompactObjStats, TertiaryCompactObjStats};
pub use runtime::{LayeredObjStats, AudioExtendedDispatchStats, ConditionalItemParametersStats};
pub use runtime::IndexedRecords16Stats;
pub use runtime::{TilemapLz77Input, TilemapTransferDirectoryStats, TilemapType3Stats};
pub mod bg_descriptors;
pub mod bg_packets;
pub mod bg_packet_data;
pub mod bg_palettes;
pub mod bg_alternate_palettes;
pub mod bg_graphics;
pub mod bg_transfers;
pub mod text_short;
pub mod text_catalogs;
pub mod fonts;
pub mod name_keyboard;
pub mod obj_lists;
pub mod obj_animations;
pub mod compact_obj_animations;
pub mod compact_obj_animations_secondary;
pub mod compact_obj_animations_tertiary;
pub mod compact_obj_animations_quaternary;
pub mod compact_obj_animations_quinary;
pub mod compact_obj_animations_senary;
pub mod compact_obj_animations_septenary;
pub mod compact_obj_animations_octonary;
pub mod ui_context_templates;
pub mod compact_obj_catalog;
pub mod word_task_obj_animations;
pub mod layered_obj_animations;
pub mod obj_graphics;
pub mod ui_palettes;
pub mod ui_graphics;
pub mod ui_raw_transfers;
pub mod items;
pub mod item_links;
pub mod metadata16;
pub mod metadata52;
pub mod metadata4;
pub mod metadata2;
pub mod metadata28;
pub mod records4;
pub mod conditional_records8;
pub mod mission_records70;
pub mod selection_byte_tables;
pub mod event_records12;
pub mod parameter_records27;
pub mod world_records38;
pub mod indexed_graphics;
pub mod direct_indexed_graphics;
pub mod direct_indexed_dictionary;
pub mod context_callback_descriptor;
pub mod conditional_language_pair;
pub mod text_reader_registry;
pub mod conditional_substitution_words;
pub mod display_control_tables;
pub mod compact_primary_alias;
pub mod observed_lz;
pub mod constructor_template64;
pub mod observed_tilemap;
pub mod observed_tilemap_descriptor;
pub mod observed_tilemap_row;
pub mod dma_register_directories;
pub mod iwram_memory_helpers;
pub mod adjacent_offset_tables;
pub mod indexed_records16;
pub mod indexed_words_3959;
pub mod thumb_switch_tables;
pub mod thumb_pc_literals;
pub mod obj_derived_dimensions;
pub mod copied_halfword_lookups;
pub mod indexed_halfwords_39a5;
pub mod script_byte_lookup_3ad;
pub mod soil_scene_records;
pub mod soil_indirect_records;
pub mod soil_indexed_bytes;
pub mod soil_compact_record;
pub mod soil_scoped_values;
pub mod soil_remaining_rectangles;
pub mod firaga_copied_matrices;
pub mod firaga_compact_cells;
pub mod firaga_indexed_obj_bank;
pub mod firaga_compact_obj_bank;
pub mod firaga_adjacent_compact_obj_bank;
pub mod firaga_6c12c8_obj_bank;
pub mod firaga_transfer_program;
pub mod firaga_palette_rows;
pub mod firaga_stack_initialization;
pub mod firaga_queued_transfer;
pub mod white_flowers_signed_bytes;
pub mod white_flowers_isolated_values;
pub mod tilemap_transfer_directory;
pub mod tilemap_stream_bank;
mod tilemap_type3;
mod tilemap_type2;
pub mod compact_cell_bank;
mod background_effect_graphics;
mod localized_obj_animation_banks;
mod background_screen_banks;
mod physical_audio_banks;
mod physical_compact_obj_region;
mod screen_update_tables;
mod counted_effect_tables;
pub mod default48;
pub mod constructor_alternates;
pub mod metadata14;
pub mod metadata12;
pub mod global_pair_flags;
pub mod evaluation_parameters;
pub mod conditional_four_byte_lookup;
pub mod conditional_packed_lookup;
pub mod conditional_descriptor_banks;
pub mod conditional_descriptor_bank_zero;
pub mod conditional_property_masks;
pub mod progression_rewards;
pub mod progression_gates;
pub mod progression_selection_gates;
pub mod aggregate_flag_keys;
pub mod trade_parameters;
pub mod item_effect_parameters;
pub mod audio_directory;
mod audio_producers;
pub mod audio_tones;
pub mod audio_sequences;
pub mod audio_extended_dispatch;
pub mod conditional_item_parameters;
pub mod range_lookup;
pub mod tilemap_geometry;
pub mod audio_driver;
pub mod palette_effect_dispatch;
pub mod palette_programs;
pub mod blend_programs;
pub mod task_descriptors;
pub mod effect_event_directories;
pub mod script_commands;
pub mod script_archive;
pub mod script_text;
pub mod script_coordinates;
pub mod auxiliary_scripts;
pub mod map_raster_palettes;
pub mod ui_choice_lists;
pub mod unit_obj_initial_pose;
pub mod unit_obj_equipment;
pub mod unit_obj_normal_battle;
pub mod unit_obj_script_special_normal;
pub mod unit_obj_script_special_alternate;
pub mod unit_obj_script_override;
pub mod unit_obj_script_registry_special;
pub mod unit_obj_script_registry_generic;
pub mod script39_registry_poses;
pub mod script39_special_poses;
pub mod unit_obj_property_words;
pub mod unit_obj_ffff_reader_fields;
pub mod slot_job_lookup;
pub mod observed_task_parameters;
pub mod task_cycle_deltas;
pub mod world_nodes;
pub mod world_roads;
pub mod encounter_headers;
pub mod encounter_unit_templates;
pub mod encounter_coordinates;
pub mod unit_obj_byte_property_words;
pub mod unit_obj_alternate_battle;
pub mod color_mode_palettes;
mod custom_lz;
pub mod lz77;

pub const SCHEMA: u32 = 1;
pub const EXTRACTOR_REV: u32 = 362;
pub const DECODER_REV: u32 = 361;
const PRE_INDEXED_TILEMAP_REVISION: (u32, u32) = (302, 301);
const PRE_TILEMAP_STREAM_FILE_REVISION: (u32, u32) = (303, 302);
const PRE_TILEMAP_LZ77_FILE_REVISION: (u32, u32) = (304, 303);
const PRE_TILEMAP_TYPE3_REVISION: (u32, u32) = (305, 304);
const PRE_TILEMAP_TYPE2_REVISION: (u32, u32) = (306, 305);
const PRE_TILEMAP_DIRECT_REVISION: (u32, u32) = (307, 306);
const PRE_TILEMAP_ORIGIN_REVISION: (u32, u32) = (308, 307);
const PRE_PROJECTED_TILEMAP_DIRECTORIES_REVISION: (u32, u32) = (309, 308);
const PRE_COMPACT_CELL_BANK_REVISION: (u32, u32) = (310, 309);
const PRE_ADDITIONAL_UI_TABLE_REVISION: (u32, u32) = (311, 310);
const PRE_DIRECT_UI_LITERALS_REVISION: (u32, u32) = (312, 311);
const PRE_CUSTOM_DECODER_ROOTS_REVISION: (u32, u32) = (313, 312);
const PRE_LOCALIZED_TEXT_ROOTS_REVISION: (u32, u32) = (314, 313);
const PRE_INDEXED_READER_BANKS_REVISION: (u32, u32) = (315, 314);
const PRE_VRAM_LZ_BANK_REVISION: (u32, u32) = (316, 315);
const PRE_HALFWORD_COPY_REVISION: (u32, u32) = (317, 316);
const PRE_HALFWORD_PRELOAD_REVISION: (u32, u32) = (318, 317);
const PRE_VRAM_LOADER_REVISION: (u32, u32) = (319, 318);
const PRE_ROW_PAIR_REVISION: (u32, u32) = (320, 319);
const PRE_STATE_INITIALIZER_REVISION: (u32, u32) = (321, 320);
const PRE_SCROLL_REGISTER_REVISION: (u32, u32) = (322, 321);
const PRE_STARTUP_TASK_ROOTS_REVISION: (u32, u32) = (323, 322);
const PRE_STARTUP_DISPATCHER_REVISION: (u32, u32) = (324, 323);
const PRE_TITLE_CALLBACKS_REVISION: (u32, u32) = (325, 324);
const PRE_TITLE_FRONTEND_REVISION: (u32, u32) = (326, 325);
const PRE_TITLE_COLOR_MODE_REVISION: (u32, u32) = (327, 326);
const PRE_COLOR_MODE_PAIRS_REVISION: (u32, u32) = (328, 327);
const PRE_SCANLINE_BLEND_REVISION: (u32, u32) = (329, 328);
const PRE_INDEXED_VRAM_SELECTORS_REVISION: (u32, u32) = (330, 329);
const PRE_WORLD_MAP_LOOKUPS_REVISION: (u32, u32) = (331, 330);
const PRE_TILEMAP_RECTANGLES_REVISION: (u32, u32) = (332, 331);
const PRE_TIMED_SOUND_BANKS_REVISION: (u32, u32) = (333, 332);
const PRE_INITIALIZER_BANK_REVISION: (u32, u32) = (334, 333);
const PRE_INITIALIZER_VARIANTS_REVISION: (u32, u32) = (335, 334);
const PRE_INITIALIZER_PAIRS_REVISION: (u32, u32) = (336, 335);
const PRE_SIGNED_MAP_DELTAS_REVISION: (u32, u32) = (337, 336);
const PRE_WORLD_NODE_RENDERER_REVISION: (u32, u32) = (338, 337);
const PRE_WORLD_STATE_TASK_REVISION: (u32, u32) = (339, 338);
const PRE_STACK_BYTE_COPIES_REVISION: (u32, u32) = (340, 339);
const PRE_FIXED_BYTE_COPIES_REVISION: (u32, u32) = (341, 340);
const PRE_DIRECT_COPY_ARGUMENTS_REVISION: (u32, u32) = (342, 341);
const PRE_COPY_PRODUCER_LITERALS_REVISION: (u32, u32) = (343, 342);
const PRE_SHARED_SLOT_LITERALS_REVISION: (u32, u32) = (344, 343);
const PRE_HELPER_FIELD_DISPATCH_REVISION: (u32, u32) = (345, 344);
const PRE_CONTEXT_RECORD_LITERALS_REVISION: (u32, u32) = (346, 345);
const PRE_SCRATCH_RESTORE_LITERALS_REVISION: (u32, u32) = (347, 346);
const PRE_CACHED_OBJECT_REVISION: (u32, u32) = (348, 347);
const PRE_INDEXED_EXCHANGE_REVISION: (u32, u32) = (349, 348);
const PRE_FIVE_BYTE_LIST_REVISION: (u32, u32) = (350, 349);
const PRE_PALETTE_TASK_REVISION: (u32, u32) = (351, 350);
const PRE_RECOVERED_TILEMAP_LISTS_REVISION: (u32, u32) = (352, 351);
const PRE_BACKGROUND_VARIANT_STATE_REVISION: (u32, u32) = (353, 352);
const PRE_CPU_FAST_SET_LITERALS_REVISION: (u32, u32) = (354, 353);
const PRE_BACKGROUND_EFFECT_GRAPHICS_REVISION: (u32, u32) = (355, 354);
const PRE_LOCALIZED_OBJ_ANIMATION_BANKS_REVISION: (u32, u32) = (356, 355);
const PRE_BACKGROUND_SCREEN_BANKS_REVISION: (u32, u32) = (357, 356);
const PRE_PHYSICAL_AUDIO_BANKS_REVISION: (u32, u32) = (358, 357);
const PRE_PHYSICAL_COMPACT_OBJ_REGION_REVISION: (u32, u32) = (359, 358);
const PRE_SCREEN_UPDATE_TABLES_REVISION: (u32, u32) = (360, 359);
const PRE_COUNTED_EFFECT_TABLES_REVISION: (u32, u32) = (361, 360);
pub const METADATA16_ID: &str = "gameplay.linked-metadata16";
pub const METADATA16_FILE: &str = "linked-metadata16.json";
pub const METADATA16_FORMAT: &str = "linked-metadata16-v1-json";
pub const MAX_METADATA16_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const METADATA52_ID: &str = "gameplay.linked-metadata52";
pub const METADATA52_FILE: &str = "linked-metadata52.json";
pub const METADATA52_FORMAT: &str = "linked-metadata52-v1-json";
pub const MAX_METADATA52_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const METADATA4_ID: &str = "gameplay.linked-metadata4";
pub const METADATA4_FILE: &str = "linked-metadata4.json";
pub const METADATA4_FORMAT: &str = "linked-metadata4-v1-json";
pub const MAX_METADATA4_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const METADATA2_ID: &str = "gameplay.linked-metadata2";
pub const METADATA2_FILE: &str = "linked-metadata2.json";
pub const METADATA2_FORMAT: &str = "linked-metadata2-v1-json";
pub const MAX_METADATA2_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const DEFAULT48_ID: &str = "gameplay.default-constructor-template";
pub const DEFAULT48_FILE: &str = "default-constructor-template.json";
pub const DEFAULT48_FORMAT: &str = "default-constructor-template-v1-json";
pub const MAX_DEFAULT48_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONSTRUCTOR_ALTERNATES_ID: &str = "gameplay.constructor-alternate-templates";
pub const CONSTRUCTOR_ALTERNATES_FILE: &str = "constructor-alternate-templates.json";
pub const CONSTRUCTOR_ALTERNATES_FORMAT: &str = "constructor-alternate-templates-v1-json";
pub const MAX_CONSTRUCTOR_ALTERNATES_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const METADATA14_ID: &str = "gameplay.linked-metadata14";
pub const METADATA14_FILE: &str = "linked-metadata14.json";
pub const METADATA14_FORMAT: &str = "linked-metadata14-v1-json";
pub const MAX_METADATA14_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const METADATA12_ID: &str = "gameplay.linked-metadata12";
pub const METADATA12_FILE: &str = "linked-metadata12.json";
pub const METADATA12_FORMAT: &str = "linked-metadata12-v1-json";
pub const MAX_METADATA12_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const GLOBAL_PAIR_FLAGS_ID: &str = "gameplay.global-pair-flags";
pub const GLOBAL_PAIR_FLAGS_FILE: &str = "global-pair-flags.json";
pub const GLOBAL_PAIR_FLAGS_FORMAT: &str = "global-pair-flags-v1-json";
pub const MAX_GLOBAL_PAIR_FLAGS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const EVALUATION_PARAMETERS_ID: &str = "gameplay.evaluation-parameters";
pub const EVALUATION_PARAMETERS_FILE: &str = "evaluation-parameters.json";
pub const EVALUATION_PARAMETERS_FORMAT: &str = "evaluation-parameters-v1-json";
pub const MAX_EVALUATION_PARAMETERS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const PROGRESSION_REWARDS_ID: &str = "gameplay.progression-rewards";
pub const PROGRESSION_REWARDS_FILE: &str = "progression-rewards.json";
pub const PROGRESSION_REWARDS_FORMAT: &str = "progression-rewards-v1-json";
pub const MAX_PROGRESSION_REWARDS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const PROGRESSION_GATES_ID: &str = "gameplay.progression-gates";
pub const PROGRESSION_GATES_FILE: &str = "progression-gates.json";
pub const PROGRESSION_GATES_FORMAT: &str = "progression-gates-v1-json";
pub const MAX_PROGRESSION_GATES_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const PROGRESSION_SELECTION_GATES_ID: &str = "gameplay.progression-selection-gates";
pub const PROGRESSION_SELECTION_GATES_FILE: &str = "progression-selection-gates.json";
pub const PROGRESSION_SELECTION_GATES_FORMAT: &str = "progression-selection-gates-v1-json";
pub const MAX_PROGRESSION_SELECTION_GATES_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const AGGREGATE_FLAG_KEYS_ID: &str = "gameplay.aggregate-flag-keys";
pub const AGGREGATE_FLAG_KEYS_FILE: &str = "aggregate-flag-keys.json";
pub const AGGREGATE_FLAG_KEYS_FORMAT: &str = "aggregate-flag-keys-v1-json";
pub const MAX_AGGREGATE_FLAG_KEYS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const TRADE_PARAMETERS_ID: &str = "gameplay.trade-parameters";
pub const TRADE_PARAMETERS_FILE: &str = "trade-parameters.json";
pub const TRADE_PARAMETERS_FORMAT: &str = "trade-parameters-v1-json";
pub const MAX_TRADE_PARAMETERS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const ITEM_EFFECT_PARAMETERS_ID: &str = "gameplay.item-effect-parameters";
pub const ITEM_EFFECT_PARAMETERS_FILE: &str = "item-effect-parameters.json";
pub const ITEM_EFFECT_PARAMETERS_FORMAT: &str = "item-effect-parameters-v1-json";
pub const MAX_ITEM_EFFECT_PARAMETERS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const AUDIO_DIRECTORY_ID: &str = "audio.m4a-directory";
pub const AUDIO_DIRECTORY_FILE: &str = "audio-directory.json";
pub const AUDIO_DIRECTORY_FORMAT: &str = "audio-directory-v2-json";
pub const MAX_AUDIO_DIRECTORY_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const AUDIO_TONES_ID: &str = "audio.m4a-tones";
pub const AUDIO_TONES_FILE: &str = "audio-tones.json";
pub const AUDIO_TONES_FORMAT: &str = "audio-tones-v1-json";
pub const MAX_AUDIO_TONES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const AUDIO_SEQUENCES_ID: &str = "audio.m4a-sequences";
pub const AUDIO_SEQUENCES_FILE: &str = "audio-sequences.json";
pub const AUDIO_SEQUENCES_FORMAT: &str = "audio-sequences-v1-json";
pub const MAX_AUDIO_SEQUENCES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const AUDIO_DRIVER_ID: &str = "audio.m4a-driver-tables";
pub const AUDIO_DRIVER_FILE: &str = "audio-driver-tables.json";
pub const AUDIO_DRIVER_FORMAT: &str = "audio-driver-tables-v1-json";
pub const MAX_AUDIO_DRIVER_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const PALETTE_EFFECT_DISPATCH_ID: &str = "graphics.palette-effect-dispatch";
pub const PALETTE_EFFECT_DISPATCH_FILE: &str = "palette-effect-dispatch.json";
pub const PALETTE_EFFECT_DISPATCH_FORMAT: &str = "palette-effect-dispatch-v1-json";
pub const MAX_PALETTE_EFFECT_DISPATCH_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const PALETTE_PROGRAMS_ID: &str = "graphics.palette-programs";
pub const PALETTE_PROGRAMS_FILE: &str = "palette-programs.json";
pub const PALETTE_PROGRAMS_FORMAT: &str = "palette-programs-v1-json";
pub const MAX_PALETTE_PROGRAMS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const BLEND_PROGRAMS_ID: &str = "graphics.blend-programs";
pub const BLEND_PROGRAMS_FILE: &str = "blend-programs.json";
pub const BLEND_PROGRAMS_FORMAT: &str = "blend-programs-v1-json";
pub const MAX_BLEND_PROGRAMS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const TASK_DESCRIPTORS_ID: &str = "engine.task-descriptors";
pub const TASK_DESCRIPTORS_FILE: &str = "task-descriptors.json";
pub const TASK_DESCRIPTORS_FORMAT: &str = "task-descriptors-v1-json";
pub const MAX_TASK_DESCRIPTORS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const EFFECT_EVENT_DIRECTORIES_ID: &str = "engine.effect-event-directories";
pub const EFFECT_EVENT_DIRECTORIES_FILE: &str = "effect-event-directories.json";
pub const EFFECT_EVENT_DIRECTORIES_FORMAT: &str = "effect-event-directories-v1-json";
pub const MAX_EFFECT_EVENT_DIRECTORIES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const SCRIPT_COMMANDS_ID: &str = "engine.script-command-directory";
pub const SCRIPT_COMMANDS_FILE: &str = "script-command-directory.json";
pub const SCRIPT_COMMANDS_FORMAT: &str = "script-command-directory-v1-json";
pub const MAX_SCRIPT_COMMANDS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const SCRIPT_ARCHIVE_ID: &str = "engine.script-archive";
pub const SCRIPT_ARCHIVE_FILE: &str = "script-archive.json";
pub const SCRIPT_ARCHIVE_FORMAT: &str = "script-archive-v1-json";
pub const MAX_SCRIPT_ARCHIVE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const SCRIPT_TEXT_ID: &str = "text.script-archives";
pub const SCRIPT_TEXT_FILE: &str = "script-text.json";
pub const SCRIPT_TEXT_FORMAT: &str = "script-text-v1-json";
pub const MAX_SCRIPT_TEXT_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const SCRIPT_COORDINATES_ID: &str = "engine.script-coordinate-paths";
pub const SCRIPT_COORDINATES_FILE: &str = "script-coordinate-paths.json";
pub const SCRIPT_COORDINATES_FORMAT: &str = "script-coordinate-paths-v1-json";
pub const MAX_SCRIPT_COORDINATES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UI_CHOICE_LISTS_ID: &str = "engine.ui-choice-lists";
pub const UI_CHOICE_LISTS_FILE: &str = "ui-choice-lists.json";
pub const UI_CHOICE_LISTS_FORMAT: &str = "ui-choice-lists-v1-json";
pub const MAX_UI_CHOICE_LISTS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_INITIAL_POSE_ID: &str = "graphics.unit-obj-initial-pose";
pub const UNIT_OBJ_INITIAL_POSE_FILE: &str = "unit-obj-initial-pose.json";
pub const UNIT_OBJ_INITIAL_POSE_FORMAT: &str = "unit-obj-initial-pose-v1-json";
pub const MAX_UNIT_OBJ_INITIAL_POSE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const COLOR_MODE_PALETTES_ID: &str = "graphics.color-mode-palettes";
pub const COLOR_MODE_PALETTES_FILE: &str = "color-mode-palettes.json";
pub const COLOR_MODE_PALETTES_FORMAT: &str = "color-mode-palettes-v1-json";
pub const MAX_COLOR_MODE_PALETTES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_EQUIPMENT_ID: &str = "graphics.unit-obj-equipment";
pub const UNIT_OBJ_EQUIPMENT_FILE: &str = "unit-obj-equipment.json";
pub const UNIT_OBJ_EQUIPMENT_FORMAT: &str = "unit-obj-equipment-v1-json";
pub const MAX_UNIT_OBJ_EQUIPMENT_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_NORMAL_BATTLE_ID: &str = "graphics.unit-obj-normal-battle";
pub const UNIT_OBJ_NORMAL_BATTLE_FILE: &str = "unit-obj-normal-battle.json";
pub const UNIT_OBJ_NORMAL_BATTLE_FORMAT: &str = "unit-obj-normal-battle-v1-json";
pub const MAX_UNIT_OBJ_NORMAL_BATTLE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_ALTERNATE_BATTLE_ID: &str = "graphics.unit-obj-alternate-battle";
pub const UNIT_OBJ_ALTERNATE_BATTLE_FILE: &str = "unit-obj-alternate-battle.json";
pub const UNIT_OBJ_ALTERNATE_BATTLE_FORMAT: &str = "unit-obj-alternate-battle-v1-json";
pub const MAX_UNIT_OBJ_ALTERNATE_BATTLE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_ID: &str = "graphics.unit-obj-script-special-normal";
pub const UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE: &str = "unit-obj-script-special-normal.json";
pub const UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FORMAT: &str = "unit-obj-script-special-normal-v1-json";
pub const MAX_UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID: &str = "graphics.unit-obj-script-special-alternate";
pub const UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE: &str = "unit-obj-script-special-alternate.json";
pub const UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FORMAT: &str = "unit-obj-script-special-alternate-v1-json";
pub const MAX_UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_SCRIPT_OVERRIDE_ID: &str = "graphics.unit-obj-script-override";
pub const UNIT_OBJ_SCRIPT_OVERRIDE_FILE: &str = "unit-obj-script-override.json";
pub const UNIT_OBJ_SCRIPT_OVERRIDE_FORMAT: &str = "unit-obj-script-override-v1-json";
pub const MAX_UNIT_OBJ_SCRIPT_OVERRIDE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID: &str = "graphics.unit-obj-script-registry-special";
pub const UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE: &str = "unit-obj-script-registry-special.json";
pub const UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FORMAT: &str = "unit-obj-script-registry-special-v1-json";
pub const MAX_UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_ID: &str = "graphics.unit-obj-script-registry-generic";
pub const UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE: &str = "unit-obj-script-registry-generic.json";
pub const UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FORMAT: &str = "unit-obj-script-registry-generic-v1-json";
pub const MAX_UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_PROPERTY_WORDS_ID: &str = "graphics.unit-obj-property-words";
pub const UNIT_OBJ_PROPERTY_WORDS_FILE: &str = "unit-obj-property-words.json";
pub const UNIT_OBJ_PROPERTY_WORDS_FORMAT: &str = "unit-obj-property-words-v1-json";
pub const MAX_UNIT_OBJ_PROPERTY_WORDS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_BYTE_PROPERTY_WORDS_ID: &str = "graphics.unit-obj-byte-property-words";
pub const UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE: &str = "unit-obj-byte-property-words.json";
pub const UNIT_OBJ_BYTE_PROPERTY_WORDS_FORMAT: &str = "unit-obj-byte-property-words-v1-json";
pub const MAX_UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UNIT_OBJ_FFFF_READER_FIELDS_ID: &str = "graphics.unit-obj-ffff-reader-fields";
pub const UNIT_OBJ_FFFF_READER_FIELDS_FILE: &str = "unit-obj-ffff-reader-fields.json";
pub const UNIT_OBJ_FFFF_READER_FIELDS_FORMAT: &str = "unit-obj-ffff-reader-fields-v1-json";
pub const MAX_UNIT_OBJ_FFFF_READER_FIELDS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const SLOT_JOB_LOOKUP_ID: &str = "gameplay.slot-job-lookup";
pub const SLOT_JOB_LOOKUP_FILE: &str = "slot-job-lookup.json";
pub const SLOT_JOB_LOOKUP_FORMAT: &str = "slot-job-lookup-v1-json";
pub const MAX_SLOT_JOB_LOOKUP_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const OBSERVED_TASK_PARAMETERS_ID: &str = "gameplay.observed-task-parameters";
pub const OBSERVED_TASK_PARAMETERS_FILE: &str = "observed-task-parameters.json";
pub const OBSERVED_TASK_PARAMETERS_FORMAT: &str = "observed-task-parameters-v1-json";
pub const MAX_OBSERVED_TASK_PARAMETERS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const CONSTRUCTOR_OBJ_ANIMATION_ID: &str = "graphics.constructor-obj-animation";
pub const CONSTRUCTOR_OBJ_ANIMATION_FILE: &str = "constructor-obj-animation.json";
pub const CONSTRUCTOR_OBJ_ANIMATION_FORMAT: &str = "constructor-obj-animation-v1-json";
pub const MAX_CONSTRUCTOR_OBJ_ANIMATION_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const TASK_CYCLE_DELTAS_ID: &str = "gameplay.task-cycle-deltas";
pub const TASK_CYCLE_DELTAS_FILE: &str = "task-cycle-deltas.json";
pub const TASK_CYCLE_DELTAS_FORMAT: &str = "task-cycle-deltas-v1-json";
pub const MAX_TASK_CYCLE_DELTAS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const UI_RAW_TRANSFERS_ID: &str = "graphics.ui-raw-transfers";
pub const UI_RAW_TRANSFERS_FILE: &str = "ui-raw-transfers.json";
pub const UI_RAW_TRANSFERS_FORMAT: &str = "ui-raw-transfers-v1-json";
pub const MAX_UI_RAW_TRANSFERS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const DYNAMIC_RAW_FRAGMENTS_ID: &str = "graphics.dynamic-raw-fragments";
pub const DYNAMIC_RAW_FRAGMENTS_FILE: &str = "dynamic-raw-fragments.json";
pub const DYNAMIC_RAW_FRAGMENTS_FORMAT: &str = "dynamic-raw-fragments-v3-json";
pub const MAX_DYNAMIC_RAW_FRAGMENTS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const CONDITIONAL_FOUR_BYTE_LOOKUP_ID: &str = "gameplay.conditional-four-byte-lookup";
pub const CONDITIONAL_FOUR_BYTE_LOOKUP_FILE: &str = "conditional-four-byte-lookup.json";
pub const CONDITIONAL_FOUR_BYTE_LOOKUP_FORMAT: &str = "conditional-four-byte-lookup-v1-json";
pub const MAX_CONDITIONAL_FOUR_BYTE_LOOKUP_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONDITIONAL_PACKED_LOOKUP_ID: &str = "gameplay.conditional-packed-lookup";
pub const CONDITIONAL_PACKED_LOOKUP_FILE: &str = "conditional-packed-lookup.json";
pub const CONDITIONAL_PACKED_LOOKUP_FORMAT: &str = "conditional-packed-lookup-v1-json";
pub const MAX_CONDITIONAL_PACKED_LOOKUP_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONDITIONAL_DESCRIPTOR_BANKS_ID: &str = "gameplay.conditional-descriptor-banks";
pub const CONDITIONAL_DESCRIPTOR_BANKS_FILE: &str = "conditional-descriptor-banks.json";
pub const CONDITIONAL_DESCRIPTOR_BANKS_FORMAT: &str = "conditional-descriptor-banks-v1-json";
pub const MAX_CONDITIONAL_DESCRIPTOR_BANKS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONDITIONAL_DESCRIPTOR_BANK_ZERO_ID: &str = "gameplay.conditional-descriptor-bank-zero";
pub const CONDITIONAL_DESCRIPTOR_BANK_ZERO_FILE: &str = "conditional-descriptor-bank-zero.json";
pub const CONDITIONAL_DESCRIPTOR_BANK_ZERO_FORMAT: &str = "conditional-descriptor-bank-zero-v1-json";
pub const MAX_CONDITIONAL_DESCRIPTOR_BANK_ZERO_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONDITIONAL_PROPERTY_MASKS_ID: &str = "gameplay.conditional-property-masks";
pub const CONDITIONAL_PROPERTY_MASKS_FILE: &str = "conditional-property-masks.json";
pub const CONDITIONAL_PROPERTY_MASKS_FORMAT: &str = "conditional-property-masks-v1-json";
pub const MAX_CONDITIONAL_PROPERTY_MASKS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations";
pub const COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations.json";
pub const COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-v1-json";
pub const MAX_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const SECONDARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-secondary";
pub const SECONDARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-secondary.json";
pub const SECONDARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-secondary-v1-json";
pub const MAX_SECONDARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const TERTIARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-tertiary";
pub const TERTIARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-tertiary.json";
pub const TERTIARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-tertiary-v1-json";
pub const MAX_TERTIARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const QUATERNARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-quaternary";
pub const QUATERNARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-quaternary.json";
pub const QUATERNARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-quaternary-v1-json";
pub const MAX_QUATERNARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const QUINARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-quinary";
pub const QUINARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-quinary.json";
pub const QUINARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-quinary-v1-json";
pub const MAX_QUINARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const SENARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-senary";
pub const SENARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-senary.json";
pub const SENARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-senary-v1-json";
pub const MAX_SENARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const SEPTENARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-septenary";
pub const OCTONARY_COMPACT_OBJ_ANIMATIONS_ID: &str = "graphics.conditional-compact-obj-animations-octonary";
pub const SEPTENARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-septenary.json";
pub const OCTONARY_COMPACT_OBJ_ANIMATIONS_FILE: &str = "compact-obj-animations-octonary.json";
pub const SEPTENARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-septenary-v1-json";
pub const OCTONARY_COMPACT_OBJ_ANIMATIONS_FORMAT: &str = "compact-obj-animations-octonary-v1-json";
pub const MAX_SEPTENARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_OCTONARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const UI_CONTEXT_TEMPLATES_ID: &str = "ui.context-templates";
pub const UI_CONTEXT_TEMPLATES_FILE: &str = "ui-context-templates.json";
pub const UI_CONTEXT_TEMPLATES_FORMAT: &str = "ui-context-templates-v1-json";
pub const MAX_UI_CONTEXT_TEMPLATES_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const COMPACT_OBJ_CATALOG_ID: &str = "graphics.conditional-compact-obj-catalog";
pub const COMPACT_OBJ_CATALOG_FILE: &str = "compact-obj-catalog.json";
pub const COMPACT_OBJ_CATALOG_FORMAT: &str = "compact-obj-catalog-v15-json";
pub const MAX_COMPACT_OBJ_CATALOG_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONDITIONAL_ITEM_PARAMETERS_ID: &str = "gameplay.conditional-item-parameters";
pub const CONDITIONAL_ITEM_PARAMETERS_FILE: &str = "conditional-item-parameters.json";
pub const CONDITIONAL_ITEM_PARAMETERS_FORMAT: &str = "conditional-item-parameters-v1-json";
pub const MAX_CONDITIONAL_ITEM_PARAMETERS_FILE_BYTES: u64 = 16 * 1024;
pub const RANGE_LOOKUP_ID: &str = "engine.bounded-range-lookup";
pub const RANGE_LOOKUP_FILE: &str = "bounded-range-lookup.json";
pub const RANGE_LOOKUP_FORMAT: &str = "bounded-range-lookup-v1-json";
pub const MAX_RANGE_LOOKUP_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const TILEMAP_GEOMETRY_ID: &str = "graphics.tilemap-geometry";
pub const TILEMAP_GEOMETRY_FILE: &str = "tilemap-geometry.json";
pub const TILEMAP_GEOMETRY_FORMAT: &str = "tilemap-geometry-v1-json";
pub const MAX_TILEMAP_GEOMETRY_FILE_BYTES: u64 = 16 * 1024;
pub const SCRIPT39_REGISTRY_POSES_ID: &str = "graphics.script39-registry-poses";
pub const SCRIPT39_REGISTRY_POSES_FILE: &str = "script39-registry-poses.json";
pub const SCRIPT39_REGISTRY_POSES_FORMAT: &str = "script39-registry-poses-v1-json";
pub const MAX_SCRIPT39_REGISTRY_POSES_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const SCRIPT39_SPECIAL_POSES_ID: &str = "graphics.script39-special-poses";
pub const SCRIPT39_SPECIAL_POSES_FILE: &str = "script39-special-poses.json";
pub const SCRIPT39_SPECIAL_POSES_FORMAT: &str = "script39-special-poses-v1-json";
pub const MAX_SCRIPT39_SPECIAL_POSES_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const AUDIO_EXTENDED_DISPATCH_ID: &str = "audio.m4a-extended-dispatch";
pub const AUDIO_EXTENDED_DISPATCH_FILE: &str = "audio-extended-dispatch.json";
pub const AUDIO_EXTENDED_DISPATCH_FORMAT: &str = "audio-extended-dispatch-v1-json";
pub const MAX_AUDIO_EXTENDED_DISPATCH_FILE_BYTES: u64 = 16 * 1024;
pub const LAYERED_OBJ_ANIMATIONS_ID: &str = "graphics.layered-obj-animations";
pub const LAYERED_OBJ_ANIMATIONS_FILE: &str = "layered-obj-animations.json";
pub const LAYERED_OBJ_ANIMATIONS_FORMAT: &str = "layered-obj-animations-v1-json";
pub const MAX_LAYERED_OBJ_ANIMATIONS_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const INDEXED_GRAPHICS_ID: &str = "graphics.indexed-archives";
pub const INDEXED_GRAPHICS_FILE: &str = "indexed-graphics.json";
pub const INDEXED_GRAPHICS_FORMAT: &str = "indexed-graphics-v2-json";
pub const MAX_INDEXED_GRAPHICS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const DIRECT_INDEXED_GRAPHICS_ID: &str = "graphics.direct-indexed-archive";
pub const DIRECT_INDEXED_GRAPHICS_FILE: &str = "direct-indexed-archive.json";
pub const DIRECT_INDEXED_GRAPHICS_FORMAT: &str = "direct-indexed-archive-v1-json";
pub const MAX_DIRECT_INDEXED_GRAPHICS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const DIRECT_INDEXED_DICTIONARY_ID: &str = "graphics.direct-indexed-dictionary-archive";
pub const DIRECT_INDEXED_DICTIONARY_FILE: &str = "direct-indexed-dictionary-archive.json";
pub const DIRECT_INDEXED_DICTIONARY_FORMAT: &str = "direct-indexed-dictionary-archive-v1-json";
pub const MAX_DIRECT_INDEXED_DICTIONARY_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const CONTEXT_CALLBACK_DESCRIPTOR_ID: &str = "engine.context-callback-descriptor";
pub const CONTEXT_CALLBACK_DESCRIPTOR_FILE: &str = "context-callback-descriptor.json";
pub const CONTEXT_CALLBACK_DESCRIPTOR_FORMAT: &str = "context-callback-descriptor-v1-json";
pub const MAX_CONTEXT_CALLBACK_DESCRIPTOR_FILE_BYTES: u64 = 1024 * 1024;
pub const CONDITIONAL_LANGUAGE_PAIR_ID: &str = "engine.conditional-language-index-opaque";
pub const CONDITIONAL_LANGUAGE_PAIR_FILE: &str = "conditional-language-index-opaque.json";
pub const CONDITIONAL_LANGUAGE_PAIR_FORMAT: &str = "conditional-language-index-opaque-v1-json";
pub const MAX_CONDITIONAL_LANGUAGE_PAIR_FILE_BYTES: u64 = 1024 * 1024;
pub const TEXT_READER_REGISTRY_ID: &str = "engine.text-reader-registry";
pub const TEXT_READER_REGISTRY_FILE: &str = "text-reader-registry.json";
pub const TEXT_READER_REGISTRY_FORMAT: &str = "text-reader-registry-v1-json";
pub const MAX_TEXT_READER_REGISTRY_FILE_BYTES: u64 = 1024 * 1024;
pub const CONDITIONAL_SUBSTITUTION_WORDS_ID: &str = "text.conditional-substitution-words";
pub const CONDITIONAL_SUBSTITUTION_WORDS_FILE: &str = "conditional-substitution-words.json";
pub const CONDITIONAL_SUBSTITUTION_WORDS_FORMAT: &str = "conditional-substitution-words-v1-json";
pub const MAX_CONDITIONAL_SUBSTITUTION_WORDS_FILE_BYTES: u64 = 1024 * 1024;
pub const DISPLAY_CONTROL_TABLES_ID: &str = "graphics.display-control-tables";
pub const DISPLAY_CONTROL_TABLES_FILE: &str = "display-control-tables.json";
pub const DISPLAY_CONTROL_TABLES_FORMAT: &str = "display-control-tables-v1-json";
pub const MAX_DISPLAY_CONTROL_TABLES_FILE_BYTES: u64 = 1024 * 1024;
pub const COMPACT_PRIMARY_ALIAS_ID: &str = "graphics.compact-primary-alias";
pub const COMPACT_PRIMARY_ALIAS_FILE: &str = "compact-primary-alias.json";
pub const COMPACT_PRIMARY_ALIAS_FORMAT: &str = "compact-primary-alias-v1-json";
pub const MAX_COMPACT_PRIMARY_ALIAS_FILE_BYTES: u64 = 1024 * 1024;
pub const OBSERVED_LZ_ID: &str = "graphics.observed-lz-block";
pub const OBSERVED_LZ_FILE: &str = "observed-lz-block.json";
pub const OBSERVED_LZ_FORMAT: &str = "observed-lz-block-v2-json";
// The 25-stream packet includes both encoded and decoded byte arrays.
pub const MAX_OBSERVED_LZ_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const CONSTRUCTOR_TEMPLATE64_ID: &str = "engine.constructor-template64";
pub const CONSTRUCTOR_TEMPLATE64_FILE: &str = "constructor-template64.json";
pub const CONSTRUCTOR_TEMPLATE64_FORMAT: &str = "constructor-template64-v1-json";
pub const MAX_CONSTRUCTOR_TEMPLATE64_FILE_BYTES: u64 = 1024 * 1024;
pub const OBSERVED_TILEMAP_ID: &str = "graphics.observed-tilemap";
pub const OBSERVED_TILEMAP_FILE: &str = "observed-tilemap.json";
pub const OBSERVED_TILEMAP_FORMAT: &str = "observed-tilemap-v4-json";
pub const MAX_OBSERVED_TILEMAP_FILE_BYTES: u64 = 1024 * 1024;
pub const OBSERVED_TILEMAP_DESCRIPTOR_ID: &str = "graphics.observed-tilemap-descriptor";
pub const OBSERVED_TILEMAP_DESCRIPTOR_FILE: &str = "observed-tilemap-descriptor.json";
pub const OBSERVED_TILEMAP_DESCRIPTOR_FORMAT: &str = "observed-tilemap-descriptor-v4-json";
pub const MAX_OBSERVED_TILEMAP_DESCRIPTOR_FILE_BYTES: u64 = 1024 * 1024;
pub const OBSERVED_TILEMAP_ROW_ID: &str = "graphics.observed-tilemap-row";
pub const OBSERVED_TILEMAP_ROW_FILE: &str = "observed-tilemap-row.json";
pub const OBSERVED_TILEMAP_ROW_FORMAT: &str = "observed-tilemap-row-v5-json";
pub const MAX_OBSERVED_TILEMAP_ROW_FILE_BYTES: u64 = 1024 * 1024;
pub const DMA_REGISTER_DIRECTORIES_ID: &str = "engine.dma-register-directories";
pub const DMA_REGISTER_DIRECTORIES_FILE: &str = "dma-register-directories.json";
pub const DMA_REGISTER_DIRECTORIES_FORMAT: &str = "dma-register-directories-v1-json";
pub const MAX_DMA_REGISTER_DIRECTORIES_FILE_BYTES: u64 = 1024 * 1024;
pub const IWRAM_MEMORY_HELPERS_ID: &str = "engine.iwram-memory-helpers";
pub const ADJACENT_OFFSET_TABLES_ID: &str = "engine.adjacent-offset-tables";
pub const IWRAM_MEMORY_HELPERS_FILE: &str = "iwram-memory-helpers.json";
pub const ADJACENT_OFFSET_TABLES_FILE: &str = "adjacent-offset-tables.json";
pub const IWRAM_MEMORY_HELPERS_FORMAT: &str = "iwram-memory-helpers-v1-json";
pub const ADJACENT_OFFSET_TABLES_FORMAT: &str = "adjacent-offset-tables-v1-json";
pub const MAX_IWRAM_MEMORY_HELPERS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_ADJACENT_OFFSET_TABLES_FILE_BYTES: u64 = 1024 * 1024;
pub const INDEXED_RECORDS16_ID: &str = "gameplay.indexed-records16";
pub const INDEXED_WORDS_3959_ID: &str = "gameplay.indexed-words-3959a4";
pub const THUMB_SWITCH_TABLES_ID: &str = "engine.thumb-switch-tables";
pub const THUMB_PC_LITERALS_ID: &str = "engine.thumb-pc-literals";
pub const COPIED_HALFWORD_LOOKUPS_ID: &str = "engine.copied-halfword-lookups";
pub const INDEXED_HALFWORDS_39A5_ID: &str = "gameplay.indexed-halfwords-39a5c8";
pub const SCRIPT_BYTE_LOOKUP_3AD_ID: &str = "engine.script-byte-lookup-3ad16e";
pub const SOIL_SCENE_RECORDS_ID: &str = "engine.soil-scene-records";
pub const SOIL_INDIRECT_RECORDS_ID: &str = "engine.soil-indirect-records";
pub const SOIL_INDEXED_BYTES_ID: &str = "engine.soil-indexed-bytes";
pub const SOIL_COMPACT_RECORD_ID: &str = "graphics.soil-compact-record";
pub const SOIL_SCOPED_VALUES_ID: &str = "engine.soil-scoped-values";
pub const SOIL_REMAINING_RECTANGLES_ID: &str = "graphics.soil-remaining-rectangles";
pub const FIRAGA_COPIED_MATRICES_ID: &str = "graphics.firaga-copied-matrices";
pub const FIRAGA_COMPACT_CELLS_ID: &str = "graphics.firaga-compact-cells";
pub const FIRAGA_INDEXED_OBJ_BANK_ID: &str = "graphics.firaga-indexed-obj-bank";
pub const FIRAGA_COMPACT_OBJ_BANK_ID: &str = "graphics.firaga-compact-obj-bank";
pub const FIRAGA_ADJACENT_COMPACT_OBJ_BANK_ID: &str = "graphics.firaga-adjacent-compact-obj-bank";
pub const FIRAGA_6C12C8_OBJ_BANK_ID: &str = "graphics.firaga-6c12c8-obj-bank";
pub const FIRAGA_TRANSFER_PROGRAM_ID: &str = "graphics.firaga-transfer-program";
pub const FIRAGA_PALETTE_ROWS_ID: &str = "graphics.firaga-palette-rows";
pub const FIRAGA_STACK_INITIALIZATION_ID: &str = "engine.firaga-stack-initialization";
pub const FIRAGA_QUEUED_TRANSFER_ID: &str = "engine.firaga-queued-transfer";
pub const WHITE_FLOWERS_SIGNED_BYTES_ID: &str = "engine.white-flowers-signed-bytes";
pub const WHITE_FLOWERS_ISOLATED_VALUES_ID: &str = "engine.white-flowers-isolated-values";
pub const INDEXED_RECORDS16_FILE: &str = "indexed-records16.json";
pub const INDEXED_WORDS_3959_FILE: &str = "indexed-words-3959a4.json";
pub const THUMB_SWITCH_TABLES_FILE: &str = "thumb-switch-tables.json";
pub const THUMB_PC_LITERALS_FILE: &str = "thumb-pc-literals.json";
pub const COPIED_HALFWORD_LOOKUPS_FILE: &str = "copied-halfword-lookups.json";
pub const INDEXED_HALFWORDS_39A5_FILE: &str = "indexed-halfwords-39a5c8.json";
pub const SCRIPT_BYTE_LOOKUP_3AD_FILE: &str = "script-byte-lookup-3ad16e.json";
pub const SOIL_SCENE_RECORDS_FILE: &str = "soil-scene-records.json";
pub const SOIL_INDIRECT_RECORDS_FILE: &str = "soil-indirect-records.json";
pub const SOIL_INDEXED_BYTES_FILE: &str = "soil-indexed-bytes.json";
pub const SOIL_COMPACT_RECORD_FILE: &str = "soil-compact-record.json";
pub const SOIL_SCOPED_VALUES_FILE: &str = "soil-scoped-values.json";
pub const SOIL_REMAINING_RECTANGLES_FILE: &str = "soil-remaining-rectangles.json";
pub const FIRAGA_COPIED_MATRICES_FILE: &str = "firaga-copied-matrices.json";
pub const FIRAGA_COMPACT_CELLS_FILE: &str = "firaga-compact-cells.json";
pub const FIRAGA_INDEXED_OBJ_BANK_FILE: &str = "firaga-indexed-obj-bank.json";
pub const FIRAGA_COMPACT_OBJ_BANK_FILE: &str = "firaga-compact-obj-bank.json";
pub const FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FILE: &str = "firaga-adjacent-compact-obj-bank.json";
pub const FIRAGA_6C12C8_OBJ_BANK_FILE: &str = "firaga-6c12c8-obj-bank.json";
pub const FIRAGA_TRANSFER_PROGRAM_FILE: &str = "firaga-transfer-program.json";
pub const FIRAGA_PALETTE_ROWS_FILE: &str = "firaga-palette-rows.json";
pub const FIRAGA_STACK_INITIALIZATION_FILE: &str = "firaga-stack-initialization.json";
pub const FIRAGA_QUEUED_TRANSFER_FILE: &str = "firaga-queued-transfer.json";
pub const WHITE_FLOWERS_SIGNED_BYTES_FILE: &str = "white-flowers-signed-bytes.json";
pub const WHITE_FLOWERS_ISOLATED_VALUES_FILE: &str = "white-flowers-isolated-values.json";
pub const INDEXED_RECORDS16_FORMAT: &str = "indexed-records16-v1-json";
pub const INDEXED_WORDS_3959_FORMAT: &str = "indexed-words-3959a4-v1-json";
pub const THUMB_SWITCH_TABLES_FORMAT: &str = "thumb-switch-tables-v1-json";
pub const THUMB_PC_LITERALS_FORMAT: &str = "thumb-pc-literals-v1-json";
pub const COPIED_HALFWORD_LOOKUPS_FORMAT: &str = "copied-halfword-lookups-v1-json";
pub const INDEXED_HALFWORDS_39A5_FORMAT: &str = "indexed-halfwords-39a5c8-v1-json";
pub const SCRIPT_BYTE_LOOKUP_3AD_FORMAT: &str = "script-byte-lookup-3ad16e-v1-json";
pub const SOIL_SCENE_RECORDS_FORMAT: &str = "soil-scene-records-v1-json";
pub const SOIL_INDIRECT_RECORDS_FORMAT: &str = "soil-indirect-records-v1-json";
pub const SOIL_INDEXED_BYTES_FORMAT: &str = "soil-indexed-bytes-v1-json";
pub const SOIL_COMPACT_RECORD_FORMAT: &str = "soil-compact-record-v1-json";
pub const SOIL_SCOPED_VALUES_FORMAT: &str = "soil-scoped-values-v1-json";
pub const SOIL_REMAINING_RECTANGLES_FORMAT: &str = "soil-remaining-rectangles-v1-json";
pub const FIRAGA_COPIED_MATRICES_FORMAT: &str = "firaga-copied-matrices-v1-json";
pub const FIRAGA_COMPACT_CELLS_FORMAT: &str = "firaga-compact-cells-v1-json";
pub const FIRAGA_INDEXED_OBJ_BANK_FORMAT: &str = "firaga-indexed-obj-bank-v1-json";
pub const FIRAGA_COMPACT_OBJ_BANK_FORMAT: &str = "firaga-compact-obj-bank-v1-json";
pub const FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FORMAT: &str = "firaga-adjacent-compact-obj-bank-v1-json";
pub const FIRAGA_6C12C8_OBJ_BANK_FORMAT: &str = "firaga-6c12c8-obj-bank-v1-json";
pub const FIRAGA_TRANSFER_PROGRAM_FORMAT: &str = "firaga-transfer-program-v1-json";
pub const FIRAGA_PALETTE_ROWS_FORMAT: &str = "firaga-palette-rows-v1-json";
pub const FIRAGA_STACK_INITIALIZATION_FORMAT: &str = "firaga-stack-initialization-v1-json";
pub const FIRAGA_QUEUED_TRANSFER_FORMAT: &str = "firaga-queued-transfer-v1-json";
pub const WHITE_FLOWERS_SIGNED_BYTES_FORMAT: &str = "white-flowers-signed-bytes-v1-json";
pub const WHITE_FLOWERS_ISOLATED_VALUES_FORMAT: &str = "white-flowers-isolated-values-v1-json";
pub const MAX_INDEXED_RECORDS16_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_INDEXED_WORDS_3959_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_THUMB_SWITCH_TABLES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_THUMB_PC_LITERALS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_COPIED_HALFWORD_LOOKUPS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_INDEXED_HALFWORDS_39A5_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SCRIPT_BYTE_LOOKUP_3AD_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_SCENE_RECORDS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_INDIRECT_RECORDS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_INDEXED_BYTES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_COMPACT_RECORD_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_SCOPED_VALUES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SOIL_REMAINING_RECTANGLES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_COPIED_MATRICES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_COMPACT_CELLS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_INDEXED_OBJ_BANK_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_COMPACT_OBJ_BANK_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_6C12C8_OBJ_BANK_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_TRANSFER_PROGRAM_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_PALETTE_ROWS_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_STACK_INITIALIZATION_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FIRAGA_QUEUED_TRANSFER_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_WHITE_FLOWERS_SIGNED_BYTES_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_WHITE_FLOWERS_ISOLATED_VALUES_FILE_BYTES: u64 = 1024 * 1024;
pub const TILEMAP_TRANSFER_DIRECTORY_ID: &str = "graphics.tilemap-transfer-directory";
pub const TILEMAP_TRANSFER_DIRECTORY_FILE: &str = "tilemap-transfer-directory.json";
pub const TILEMAP_TRANSFER_DIRECTORY_FORMAT: &str = "tilemap-transfer-directory-v1-json";
pub const MAX_TILEMAP_TRANSFER_DIRECTORY_FILE_BYTES: u64 = 64 * 1024;
pub const INDEXED_TILEMAP_CONTROLS_ID: &str = "graphics.indexed-tilemap-controls";
pub const INDEXED_TILEMAP_CONTROLS_FILE: &str = "indexed-tilemap-controls.json";
pub const INDEXED_TILEMAP_CONTROLS_FORMAT: &str = "indexed-tilemap-controls-v1-json";
pub const MAX_INDEXED_TILEMAP_CONTROLS_FILE_BYTES: u64 = 16 * 1024;
pub const TILEMAP_STREAM_BANK_ID: &str = "graphics.tilemap-stream-bank";
pub const TILEMAP_STREAM_BANK_FILE: &str = "tilemap-stream-bank.bin";
pub const TILEMAP_STREAM_BANK_FORMAT: &str = "tilemap-stream-bank-v1-bin";
pub const MAX_TILEMAP_STREAM_BANK_FILE_BYTES: u64 =
    (tilemap_stream_bank::END - tilemap_stream_bank::START) as u64;
pub const TILEMAP_TRANSFER_LZ77_ID: &str = "graphics.tilemap-transfer-lz77";
pub const TILEMAP_TRANSFER_LZ77_FILE: &str = "tilemap-transfer-lz77.bin";
pub const TILEMAP_TRANSFER_LZ77_FORMAT: &str = "tilemap-transfer-lz77-v1-bin";
pub const MAX_TILEMAP_TRANSFER_LZ77_FILE_BYTES: u64 = 64 * 1024;
pub const TILEMAP_TYPE3_ID: &str = "graphics.tilemap-type3-transfers";
pub const TILEMAP_TYPE3_FILE: &str = "tilemap-type3-transfers.bin";
pub const TILEMAP_TYPE3_FORMAT: &str = "tilemap-type3-transfers-v1-bin";
pub const MAX_TILEMAP_TYPE3_FILE_BYTES: u64 = 512 * 1024;
pub const TILEMAP_TYPE2_ID: &str = "graphics.tilemap-type2-transfers";
pub const TILEMAP_TYPE2_FILE: &str = "tilemap-type2-transfers.bin";
pub const TILEMAP_TYPE2_FORMAT: &str = "tilemap-type2-transfers-v1-bin";
pub const MAX_TILEMAP_TYPE2_FILE_BYTES: u64 = 32 * 1024;
pub const TILEMAP_DIRECT_ID: &str = "graphics.tilemap-direct-transfers";
pub const TILEMAP_DIRECT_FILE: &str = "tilemap-direct-transfers.bin";
pub const TILEMAP_DIRECT_FORMAT: &str = "tilemap-direct-transfers-v1-bin";
pub const MAX_TILEMAP_DIRECT_FILE_BYTES: u64 = 16 * 1024;
pub const COMPACT_CELL_BANK_ID: &str = "graphics.compact-cell-bank";
pub const COMPACT_CELL_BANK_FILE: &str = "compact-cell-bank.bin";
pub const COMPACT_CELL_BANK_FORMAT: &str = "compact-cell-bank-v1-bin";
pub const MAX_COMPACT_CELL_BANK_FILE_BYTES: u64 = 32 * 1024;
pub const BACKGROUND_EFFECT_GRAPHICS_ID: &str = "graphics.background-effect-graphics";
pub const BACKGROUND_EFFECT_GRAPHICS_FILE: &str = "background-effect-graphics.bin";
pub const BACKGROUND_EFFECT_GRAPHICS_FORMAT: &str = "background-effect-graphics-v1-bin";
pub const MAX_BACKGROUND_EFFECT_GRAPHICS_FILE_BYTES: u64 = background_effect_graphics::PACKED_BYTES;
pub const LOCALIZED_OBJ_ANIMATION_BANKS_ID: &str = "graphics.localized-obj-animation-banks";
pub const LOCALIZED_OBJ_ANIMATION_BANKS_FILE: &str = "localized-obj-animation-banks.bin";
pub const LOCALIZED_OBJ_ANIMATION_BANKS_FORMAT: &str = "localized-obj-animation-banks-v1-bin";
pub const MAX_LOCALIZED_OBJ_ANIMATION_BANKS_FILE_BYTES: u64 =
    localized_obj_animation_banks::PACKED_BYTES;
pub const BACKGROUND_SCREEN_BANKS_ID: &str = "graphics.background-screen-banks";
pub const BACKGROUND_SCREEN_BANKS_FILE: &str = "background-screen-banks.bin";
pub const BACKGROUND_SCREEN_BANKS_FORMAT: &str = "background-screen-banks-v1-bin";
pub const MAX_BACKGROUND_SCREEN_BANKS_FILE_BYTES: u64 = background_screen_banks::PACKED_BYTES;
pub const PHYSICAL_AUDIO_BANKS_ID: &str = "audio.physical-tone-banks";
pub const PHYSICAL_AUDIO_BANKS_FILE: &str = "physical-tone-banks.bin";
pub const PHYSICAL_AUDIO_BANKS_FORMAT: &str = "physical-tone-banks-v1-bin";
pub const MAX_PHYSICAL_AUDIO_BANKS_FILE_BYTES: u64 = physical_audio_banks::PACKED_BYTES;
pub const PHYSICAL_COMPACT_OBJ_REGION_ID: &str = "graphics.physical-compact-obj-banks";
pub const PHYSICAL_COMPACT_OBJ_REGION_FILE: &str = "physical-compact-obj-banks.bin";
pub const PHYSICAL_COMPACT_OBJ_REGION_FORMAT: &str = "physical-compact-obj-banks-v1-bin";
pub const MAX_PHYSICAL_COMPACT_OBJ_REGION_FILE_BYTES: u64 = physical_compact_obj_region::PACKED_BYTES;
pub const SCREEN_UPDATE_TABLES_ID: &str = "graphics.screen-update-tables";
pub const SCREEN_UPDATE_TABLES_FILE: &str = "screen-update-tables.bin";
pub const SCREEN_UPDATE_TABLES_FORMAT: &str = "screen-update-tables-v1-bin";
pub const MAX_SCREEN_UPDATE_TABLES_FILE_BYTES: u64 = screen_update_tables::PACKED_BYTES;
pub const COUNTED_EFFECT_TABLES_ID: &str = "graphics.counted-effect-tables";
pub const COUNTED_EFFECT_TABLES_FILE: &str = "counted-effect-tables.bin";
pub const COUNTED_EFFECT_TABLES_FORMAT: &str = "counted-effect-tables-v1-bin";
pub const MAX_COUNTED_EFFECT_TABLES_FILE_BYTES: u64 = counted_effect_tables::PACKED_BYTES;
pub const WORLD_RECORDS38_ID: &str = "gameplay.world-records38";
pub const WORLD_RECORDS38_FILE: &str = "world-records38.json";
pub const WORLD_RECORDS38_FORMAT: &str = "world-records38-v1-json";
pub const MAX_WORLD_RECORDS38_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const PARAMETER_RECORDS27_ID: &str = "gameplay.parameter-records27";
pub const PARAMETER_RECORDS27_FILE: &str = "parameter-records27.json";
pub const PARAMETER_RECORDS27_FORMAT: &str = "parameter-records27-v1-json";
pub const MAX_PARAMETER_RECORDS27_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const EVENT_RECORDS12_ID: &str = "gameplay.event-records12";
pub const EVENT_RECORDS12_FILE: &str = "event-records12.json";
pub const EVENT_RECORDS12_FORMAT: &str = "event-records12-v1-json";
pub const MAX_EVENT_RECORDS12_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const SELECTION_BYTE_TABLES_ID: &str = "gameplay.selection-byte-tables";
pub const SELECTION_BYTE_TABLES_FILE: &str = "selection-byte-tables.json";
pub const SELECTION_BYTE_TABLES_FORMAT: &str = "selection-byte-tables-v1-json";
pub const MAX_SELECTION_BYTE_TABLES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MISSION_RECORDS70_ID: &str = "gameplay.mission-records70";
pub const MISSION_RECORDS70_FILE: &str = "mission-records70.json";
pub const MISSION_RECORDS70_FORMAT: &str = "mission-records70-v1-json";
pub const MAX_MISSION_RECORDS70_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const CONDITIONAL_RECORDS8_ID: &str = "gameplay.conditional-records8";
pub const CONDITIONAL_RECORDS8_FILE: &str = "conditional-records8.json";
pub const CONDITIONAL_RECORDS8_FORMAT: &str = "conditional-records8-v1-json";
pub const MAX_CONDITIONAL_RECORDS8_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const RECORDS4_ID: &str = "gameplay.linked-records4";
pub const RECORDS4_FILE: &str = "linked-records4.json";
pub const RECORDS4_FORMAT: &str = "linked-records4-v1-json";
pub const MAX_RECORDS4_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const METADATA28_ID: &str = "gameplay.linked-metadata28";
pub const METADATA28_FILE: &str = "linked-metadata28.json";
pub const METADATA28_FORMAT: &str = "linked-metadata28-v1-json";
pub const MAX_METADATA28_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const ENCOUNTER_COORDINATES_ID: &str = "gameplay.encounter-coordinates";
pub const ENCOUNTER_COORDINATES_FILE: &str = "encounter-coordinates.json";
pub const ENCOUNTER_COORDINATES_FORMAT: &str = "encounter-coordinates-v1-json";
pub const MAX_ENCOUNTER_COORDINATES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const ENCOUNTER_UNIT_TEMPLATES_ID: &str = "gameplay.encounter-unit-templates";
pub const ENCOUNTER_UNIT_TEMPLATES_FILE: &str = "encounter-unit-templates.json";
pub const ENCOUNTER_UNIT_TEMPLATES_FORMAT: &str = "encounter-unit-templates-v1-json";
pub const MAX_ENCOUNTER_UNIT_TEMPLATES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const ENCOUNTER_HEADERS_ID: &str = "gameplay.encounter-headers";
pub const ENCOUNTER_HEADERS_FILE: &str = "encounter-headers.json";
pub const ENCOUNTER_HEADERS_FORMAT: &str = "encounter-headers-v1-json";
pub const MAX_ENCOUNTER_HEADERS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const WORLD_ROADS_ID: &str = "gameplay.world-roads";
pub const WORLD_ROADS_FILE: &str = "world-roads.json";
pub const WORLD_ROADS_FORMAT: &str = "world-roads-v1-json";
pub const MAX_WORLD_ROADS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const WORLD_NODES_ID: &str = "gameplay.world-nodes";
pub const WORLD_NODES_FILE: &str = "world-nodes.json";
pub const WORLD_NODES_FORMAT: &str = "world-nodes-v1-json";
pub const MAX_WORLD_NODES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const WORD_TASK_OBJ_ANIMATIONS_ID: &str = "graphics.word-task-obj-animations";
pub const WORD_TASK_OBJ_ANIMATIONS_FILE: &str = "word-task-obj-animations.json";
pub const WORD_TASK_OBJ_ANIMATIONS_FORMAT: &str = "word-task-obj-animations-v1-json";
pub const MAX_WORD_TASK_OBJ_ANIMATIONS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAP_RASTER_PALETTES_ID: &str = "graphics.map-raster-palettes";
pub const MAP_RASTER_PALETTES_FILE: &str = "map-raster-palettes.json";
pub const MAP_RASTER_PALETTES_FORMAT: &str = "map-raster-palettes-v1-json";
pub const MAX_MAP_RASTER_PALETTES_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const AUXILIARY_COMMANDS_ID: &str = "engine.auxiliary-script-command-directory";
pub const AUXILIARY_COMMANDS_FILE: &str = "auxiliary-script-command-directory.json";
pub const AUXILIARY_COMMANDS_FORMAT: &str = "auxiliary-script-command-directory-v1-json";
pub const MAX_AUXILIARY_COMMANDS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const AUXILIARY_ARCHIVE_ID: &str = "engine.auxiliary-script-archive";
pub const AUXILIARY_ARCHIVE_FILE: &str = "auxiliary-script-archive.json";
pub const AUXILIARY_ARCHIVE_FORMAT: &str = "auxiliary-script-archive-v1-json";
pub const MAX_AUXILIARY_ARCHIVE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const ITEM_LINKS_ID: &str = "gameplay.item-links";
pub const ITEM_LINKS_FILE: &str = "item-links.json";
pub const ITEM_LINKS_FORMAT: &str = "item-links-v1-json";
pub const MAX_ITEM_LINKS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const ITEM_RECORDS_ID: &str = "gameplay.item-records";
pub const ITEM_RECORDS_FILE: &str = "items.json";
pub const ITEM_RECORDS_FORMAT: &str = "item-records-v1-json";
pub const MAX_ITEM_RECORDS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const UI_CUSTOM_GRAPHICS_ID: &str = "graphics.ui-custom-banks";
pub const UI_CUSTOM_GRAPHICS_FILE: &str = "ui-graphics.json";
pub const UI_CUSTOM_GRAPHICS_FORMAT: &str = "ui-custom-banks-v1-json";
pub const MAX_UI_CUSTOM_GRAPHICS_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const UI_RAW_PALETTES_ID: &str = "graphics.ui-raw-palettes";
pub const UI_RAW_PALETTES_FILE: &str = "ui-palettes.json";
pub const UI_RAW_PALETTES_FORMAT: &str = "ui-raw-palettes-v1-json";
pub const MAX_UI_RAW_PALETTES_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const OBJ_CUSTOM_GRAPHICS_ID: &str = "graphics.obj-custom-banks";
pub const OBJ_CUSTOM_GRAPHICS_FILE: &str = "obj-graphics.json";
pub const OBJ_CUSTOM_GRAPHICS_FORMAT: &str = "obj-custom-banks-v1-json";
pub const MAX_OBJ_CUSTOM_GRAPHICS_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const OBJ_ANIMATION_BANKS_ID: &str = "graphics.obj-animation-banks";
pub const OBJ_ANIMATION_BANKS_FILE: &str = "obj-animation-banks.json";
pub const OBJ_ANIMATION_BANKS_FORMAT: &str = "obj-animation-banks-v1-json";
pub const MAX_OBJ_ANIMATION_BANKS_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const OBJ_STATIC_LISTS_ID: &str = "graphics.obj-static-lists";
pub const OBJ_STATIC_LISTS_FILE: &str = "obj-static-lists.json";
pub const OBJ_STATIC_LISTS_FORMAT: &str = "obj-static-lists-v2-json";
pub const MAX_OBJ_STATIC_LISTS_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const OBJ_DIMENSIONS_ID: &str = "graphics.obj-dimensions";
pub const OBJ_DIMENSIONS_FILE: &str = "obj-dimensions.toml";
pub const BG_DESCRIPTORS_ID: &str = "graphics.bg-descriptors";
pub const BG_DESCRIPTORS_FILE: &str = "bg-descriptors.json";
pub const BG_DESCRIPTORS_FORMAT: &str = "bg-descriptors-json-v2";
pub const BG_BASE_PALETTES_ID: &str = "graphics.bg-base-palettes";
pub const BG_BASE_PALETTES_FILE: &str = "bg-base-palettes.json";
pub const BG_BASE_PALETTES_FORMAT: &str = "bg-base-palettes-v2-json";
pub const BG_ALTERNATE_PALETTES_ID: &str = "graphics.bg-alternate-palettes";
pub const BG_ALTERNATE_PALETTES_FILE: &str = "bg-alternate-palettes.json";
pub const BG_ALTERNATE_PALETTES_FORMAT: &str = "bg-alternate-palettes-v1-json";
pub const BG_PACKET_PROGRAMS_ID: &str = "graphics.bg-packet-programs";
pub const BG_PACKET_PROGRAMS_FILE: &str = "bg-packet-programs.json";
pub const BG_PACKET_PROGRAMS_FORMAT: &str = "bg-packet-programs-v1-json";
pub const BG_MAIN_GRAPHICS_ID: &str = "graphics.bg-main-graphics";
pub const BG_MAIN_GRAPHICS_FILE: &str = "bg-main-graphics.json";
pub const BG_MAIN_GRAPHICS_FORMAT: &str = "bg-main-graphics-v1-json";
pub const MAX_BG_MAIN_GRAPHICS_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const FONT_BANK_ID: &str = "font.complete-bank";
pub const FONT_BANK_FILE: &str = "font-bank.json";
pub const FONT_BANK_FORMAT: &str = "font-bank-v1-json";
pub const MAX_FONT_BANK_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const NAME_KEYBOARD_ID: &str = "text.name-keyboard";
pub const NAME_KEYBOARD_FILE: &str = "name-keyboard.json";
pub const NAME_KEYBOARD_FORMAT: &str = "name-keyboard-v1-json";
pub const MAX_NAME_KEYBOARD_FILE_BYTES: u64 = 1 * 1024 * 1024;
pub const TEXT_CATALOGS_ID: &str = "text.catalogs";
pub const TEXT_CATALOGS_FILE: &str = "text-catalogs.json";
pub const TEXT_CATALOGS_FORMAT: &str = "text-catalogs-v1-json";
pub const MAX_TEXT_CATALOGS_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const SHORT_TEXTS_ID: &str = "text.short-strings";
pub const SHORT_TEXTS_FILE: &str = "short-texts.json";
pub const SHORT_TEXTS_FORMAT: &str = "short-texts-v1-json";
pub const MAX_SHORT_TEXTS_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const BG_TRANSFER_PROGRAMS_ID: &str = "graphics.bg-transfer-programs";
pub const BG_TRANSFER_PROGRAMS_FILE: &str = "bg-transfer-programs.json";
pub const BG_TRANSFER_PROGRAMS_FORMAT: &str = "bg-transfer-programs-v1-json";
pub const MAX_BG_TRANSFER_PROGRAMS_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_BG_PACKET_PROGRAMS_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_BG_ALTERNATE_PALETTES_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_MANIFEST_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_BG_BASE_PALETTES_FILE_BYTES: u64 = 1024 * 1024;
pub const MANIFEST_FILE: &str = "manifest.toml";
pub const MAX_FILE_BYTES: u64 = 16 * 1024;
pub const MAX_BG_DESCRIPTORS_FILE_BYTES: u64 = 256 * 1024;
// Retain a finite aggregate input guard while allowing current and subsequent
// decoded EUR resources; individual file guards continue to apply separately.
pub const MAX_DATASET_BYTES: usize = 768 * 1024 * 1024;
pub const DIMENSIONS_SHA256: &str =
    "6ff886b23cde2726a4ceb9cba8c5569bbb4dc6a1185769547f78dfbcb7e2d991";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjSize {
    pub width: u16,
    pub height: u16,
}

/// Shape-major, size-minor indexed metadata. Shape 3 has no record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjDimensions {
    pub entries: [ObjSize; 12],
}

impl ObjDimensions {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 48 {
            return Err("OBJ dimensions require exactly 48 source bytes".into());
        }
        let result = Self {
            entries: std::array::from_fn(|i| ObjSize {
                width: u16::from_le_bytes([bytes[i * 4], bytes[i * 4 + 1]]),
                height: u16::from_le_bytes([bytes[i * 4 + 2], bytes[i * 4 + 3]]),
            }),
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Hardware OBJ extents; the actual base ordering is authenticated by
        // the source digest, independently of these structural constraints.
        if self
            .entries
            .iter()
            .any(|e| !matches!(e.width, 8 | 16 | 32 | 64) || !matches!(e.height, 8 | 16 | 32 | 64))
        {
            return Err("Invalid OBJ dimension (expected 8, 16, 32 or 64 pixels)".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> [u8; 48] {
        let mut out = [0; 48];
        for (i, e) in self.entries.iter().enumerate() {
            out[i * 4..i * 4 + 2].copy_from_slice(&e.width.to_le_bytes());
            out[i * 4 + 2..i * 4 + 4].copy_from_slice(&e.height.to_le_bytes());
        }
        out
    }

    pub fn get(&self, shape: u8, size: u8) -> Option<&ObjSize> {
        if shape >= 3 || size >= 4 {
            return None;
        }
        self.entries.get(shape as usize * 4 + size as usize)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRange {
    pub offset: u32,
    pub length: u32,
    pub sha256: String,
    pub role: String,
}

/// Store verified encoded ranges in their manifest order.
fn pack_encoded_sources(
    rom: &[u8],
    ranges: impl IntoIterator<Item = (std::ops::Range<u32>, &'static str)>,
) -> (Vec<u8>, Vec<SourceRange>) {
    let mut encoded = Vec::new();
    let mut sources = Vec::new();
    for (range, role) in ranges {
        let bytes = &rom[range.start as usize..range.end as usize];
        sources.push(SourceRange {
            offset: range.start,
            length: range.end - range.start,
            sha256: ffta_rom::digest(bytes),
            role: role.into(),
        });
        encoded.extend_from_slice(bytes);
    }
    (encoded, sources)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub path: String,
    pub format: String,
    pub sha256: String,
    pub dependencies: Vec<String>,
    pub sources: Vec<SourceRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub extractor_revision: u32,
    pub decoder_revision: u32,
    pub source_rom_sha256: String,
    pub source_rom_size: u32,
    pub profile: String,
    pub coverage: Vec<String>,
    pub resources: Vec<Resource>,
}

/// A source-authenticated extraction. This is not a mod loader.
pub struct Extraction {
    pub manifest: Manifest,
    pub dimensions: ObjDimensions,
    pub background: Option<bg_descriptors::Descriptors>,
    pub palettes: Option<bg_palettes::Catalog>,
    pub alternate_palettes: Option<bg_alternate_palettes::Catalog>,
    pub packet_programs: Option<bg_packet_data::Catalog>,
    pub main_graphics: Option<bg_graphics::Catalog>,
    pub transfers: Option<bg_transfers::Catalog>,
    pub short_texts: Option<text_short::Catalog>,
    pub font_bank: Option<fonts::Catalog>,
    pub name_keyboard: Option<name_keyboard::Catalog>,
    pub obj_lists: Option<obj_lists::Catalog>,
    pub obj_animations: Option<obj_animations::Catalog>,
    pub obj_graphics: Option<obj_graphics::Catalog>,
    pub ui_palettes: Option<ui_palettes::Catalog>,
    pub ui_graphics: Option<ui_graphics::Catalog>,
    pub items: Option<items::Catalog>,
    pub item_links: Option<item_links::Catalog>,
    pub metadata16: Option<metadata16::Catalog>,
    pub metadata52: Option<metadata52::Catalog>,
    pub metadata4: Option<metadata4::Catalog>,
    pub metadata2: Option<metadata2::Catalog>,
    pub default48: Option<default48::Catalog>,
    pub constructor_alternates: Option<constructor_alternates::Catalog>,
    pub metadata14: Option<metadata14::Catalog>,
    pub metadata12: Option<metadata12::Catalog>,
    pub global_pair_flags: Option<global_pair_flags::Catalog>,
    pub evaluation_parameters: Option<evaluation_parameters::Catalog>,
    pub progression_rewards: Option<progression_rewards::Catalog>,
    pub progression_gates: Option<progression_gates::Catalog>,
    pub progression_selection_gates: Option<progression_selection_gates::Catalog>,
    pub aggregate_flag_keys: Option<aggregate_flag_keys::Catalog>,
    pub trade_parameters: Option<trade_parameters::Catalog>,
    pub item_effect_parameters: Option<item_effect_parameters::Catalog>,
    pub audio_directory: Option<audio_directory::Catalog>,
    pub audio_tones: Option<audio_tones::Catalog>,
    pub audio_sequences: Option<audio_sequences::Catalog>,
    pub audio_driver: Option<audio_driver::Catalog>,
    pub palette_effect_dispatch: Option<palette_effect_dispatch::Catalog>,
    pub palette_programs: Option<palette_programs::Catalog>,
    pub blend_programs: Option<blend_programs::Catalog>,
    pub task_descriptors: Option<task_descriptors::Catalog>,
    pub effect_event_directories: Option<effect_event_directories::Catalog>,
    pub script_commands: Option<script_commands::Catalog>,
    pub script_archive: Option<script_archive::Catalog>,
    pub script_text: Option<script_text::Catalog>,
    pub script_coordinates: Option<script_coordinates::Catalog>,
    pub ui_choice_lists: Option<ui_choice_lists::Catalog>,
    pub unit_obj_initial_pose: Option<unit_obj_initial_pose::Catalog>,
    pub color_mode_palettes: Option<color_mode_palettes::Catalog>,
    pub unit_obj_equipment: Option<unit_obj_equipment::Catalog>,
    pub unit_obj_normal_battle: Option<unit_obj_normal_battle::Catalog>,
    pub unit_obj_alternate_battle: Option<unit_obj_alternate_battle::Catalog>,
    pub unit_obj_script_special_normal: Option<unit_obj_script_special_normal::Catalog>,
    pub unit_obj_script_special_alternate: Option<unit_obj_script_special_alternate::Catalog>,
    pub unit_obj_script_override: Option<unit_obj_script_override::Catalog>,
    pub unit_obj_script_registry_special: Option<unit_obj_script_registry_special::Catalog>,
    pub unit_obj_script_registry_generic: Option<unit_obj_script_registry_generic::Catalog>,
    pub unit_obj_property_words: Option<unit_obj_property_words::Catalog>,
    pub unit_obj_byte_property_words: Option<unit_obj_byte_property_words::Catalog>,
    pub unit_obj_ffff_reader_fields: Option<unit_obj_ffff_reader_fields::Catalog>,
    pub slot_job_lookup: Option<slot_job_lookup::Catalog>,
    pub observed_task_parameters: Option<observed_task_parameters::Catalog>,
    pub constructor_obj_animation: Option<obj_lists::constructor_animation::Catalog>,
    pub task_cycle_deltas: Option<task_cycle_deltas::Catalog>,
    pub word_task_obj_animations: Option<word_task_obj_animations::Catalog>,
    pub world_nodes: Option<world_nodes::Catalog>,
    pub world_roads: Option<world_roads::Catalog>,
    pub encounter_headers: Option<encounter_headers::Catalog>,
    pub encounter_unit_templates: Option<encounter_unit_templates::Catalog>,
    pub encounter_coordinates: Option<encounter_coordinates::Catalog>,
    pub metadata28: Option<metadata28::Catalog>,
    pub records4: Option<records4::Catalog>,
    pub conditional_records8: Option<conditional_records8::Catalog>,
    pub mission_records70: Option<mission_records70::Catalog>,
    pub selection_byte_tables: Option<selection_byte_tables::Catalog>,
    pub event_records12: Option<event_records12::Catalog>,
    pub parameter_records27: Option<parameter_records27::Catalog>,
    pub world_records38: Option<world_records38::Catalog>,
    pub indexed_graphics: Option<indexed_graphics::Catalog>,
    pub ui_raw_transfers: Option<ui_raw_transfers::Catalog>,
    pub dynamic_raw_fragments: Option<ui_raw_transfers::dynamic_fragments::Catalog>,
    pub conditional_four_byte_lookup: Option<conditional_four_byte_lookup::Catalog>,
    pub conditional_packed_lookup: Option<conditional_packed_lookup::Catalog>,
    pub conditional_descriptor_banks: Option<conditional_descriptor_banks::Catalog>,
    pub conditional_descriptor_bank_zero: Option<conditional_descriptor_bank_zero::Catalog>,
    pub conditional_property_masks: Option<conditional_property_masks::Catalog>,
    pub compact_obj_animations: Option<compact_obj_animations::Catalog>,
    pub secondary_compact_obj_animations: Option<compact_obj_animations_secondary::Catalog>,
    pub tertiary_compact_obj_animations: Option<compact_obj_animations_tertiary::Catalog>,
    pub quaternary_compact_obj_animations: Option<compact_obj_animations_quaternary::Catalog>,
    pub quinary_compact_obj_animations: Option<compact_obj_animations_quinary::Catalog>,
    pub senary_compact_obj_animations: Option<compact_obj_animations_senary::Catalog>,
    pub septenary_compact_obj_animations: Option<compact_obj_animations_septenary::Catalog>,
    pub octonary_compact_obj_animations: Option<compact_obj_animations_octonary::Catalog>,
    pub ui_context_templates: Option<ui_context_templates::Catalog>,
    pub compact_obj_catalog: Option<compact_obj_catalog::Catalog>,
    pub layered_obj_animations: Option<layered_obj_animations::Catalog>,
    pub audio_extended_dispatch: Option<audio_extended_dispatch::Catalog>,
    pub conditional_item_parameters: Option<conditional_item_parameters::Catalog>,
    pub range_lookup: Option<range_lookup::Catalog>,
    pub tilemap_geometry: Option<tilemap_geometry::Catalog>,
    pub script39_registry_poses: Option<script39_registry_poses::Catalog>,
    pub script39_special_poses: Option<script39_special_poses::Catalog>,
    pub direct_indexed_graphics: Option<direct_indexed_graphics::Catalog>,
    pub direct_indexed_dictionary: Option<direct_indexed_dictionary::Catalog>,
    pub context_callback_descriptor: Option<context_callback_descriptor::Catalog>,
    pub conditional_language_pair: Option<conditional_language_pair::Catalog>,
    pub text_reader_registry: Option<text_reader_registry::Catalog>,
    pub conditional_substitution_words: Option<conditional_substitution_words::Catalog>,
    pub display_control_tables: Option<display_control_tables::Catalog>,
    pub compact_primary_alias: Option<compact_primary_alias::Catalog>,
    pub observed_lz: Option<observed_lz::Catalog>,
    pub constructor_template64: Option<constructor_template64::Catalog>,
    pub observed_tilemap: Option<observed_tilemap::Catalog>,
    pub observed_tilemap_descriptor: Option<observed_tilemap_descriptor::Catalog>,
    pub observed_tilemap_row: Option<observed_tilemap_row::Catalog>,
    pub dma_register_directories: Option<dma_register_directories::Catalog>,
    pub iwram_memory_helpers: Option<iwram_memory_helpers::Catalog>,
    pub adjacent_offset_tables: Option<adjacent_offset_tables::Catalog>,
    pub indexed_records16: Option<indexed_records16::Catalog>,
    pub indexed_words_3959: Option<indexed_words_3959::Catalog>,
    pub thumb_switch_tables: Option<thumb_switch_tables::Catalog>,
    pub thumb_pc_literals: Option<thumb_pc_literals::Catalog>,
    pub copied_halfword_lookups: Option<copied_halfword_lookups::Catalog>,
    pub indexed_halfwords_39a5: Option<indexed_halfwords_39a5::Catalog>,
    pub script_byte_lookup_3ad: Option<script_byte_lookup_3ad::Catalog>,
    pub soil_scene_records: Option<soil_scene_records::Catalog>,
    pub soil_indirect_records: Option<soil_indirect_records::Catalog>,
    pub soil_indexed_bytes: Option<soil_indexed_bytes::Catalog>,
    pub soil_compact_record: Option<soil_compact_record::Catalog>,
    pub soil_scoped_values: Option<soil_scoped_values::Catalog>,
    pub soil_remaining_rectangles: Option<soil_remaining_rectangles::Catalog>,
    pub firaga_copied_matrices: Option<firaga_copied_matrices::Catalog>,
    pub firaga_compact_cells: Option<firaga_compact_cells::Catalog>,
    pub firaga_indexed_obj_bank: Option<firaga_indexed_obj_bank::Catalog>,
    pub firaga_compact_obj_bank: Option<firaga_compact_obj_bank::Catalog>,
    pub firaga_adjacent_compact_obj_bank: Option<firaga_adjacent_compact_obj_bank::Catalog>,
    pub firaga_6c12c8_obj_bank: Option<firaga_6c12c8_obj_bank::Catalog>,
    pub firaga_transfer_program: Option<firaga_transfer_program::Catalog>,
    pub firaga_palette_rows: Option<firaga_palette_rows::Catalog>,
    pub firaga_stack_initialization: Option<firaga_stack_initialization::Catalog>,
    pub firaga_queued_transfer: Option<firaga_queued_transfer::Catalog>,
    pub white_flowers_signed_bytes: Option<white_flowers_signed_bytes::Catalog>,
    pub white_flowers_isolated_values: Option<white_flowers_isolated_values::Catalog>,
    pub tilemap_transfer_directory: Option<tilemap_transfer_directory::Catalog>,
    pub indexed_tilemap_controls: Option<tilemap_stream_bank::IndexedTilemapControls>,
    pub tilemap_stream_bank: Option<Vec<u8>>,
    pub tilemap_transfer_lz77: Option<Vec<u8>>,
    pub tilemap_type3: Option<Vec<u8>>,
    pub tilemap_type2: Option<Vec<u8>>,
    pub tilemap_direct: Option<Vec<u8>>,
    pub compact_cell_bank: Option<Vec<u8>>,
    pub background_effect_graphics: Option<Vec<u8>>,
    pub localized_obj_animation_banks: Option<Vec<u8>>,
    pub background_screen_banks: Option<Vec<u8>>,
    pub physical_audio_banks: Option<Vec<u8>>,
    pub physical_compact_obj_region: Option<Vec<u8>>,
    pub screen_update_tables: Option<Vec<u8>>,
    pub counted_effect_tables: Option<Vec<u8>>,
    pub map_raster_palettes: Option<map_raster_palettes::Catalog>,
    pub auxiliary_scripts: Option<auxiliary_scripts::Catalog>,
    pub text_catalogs: Option<text_catalogs::Catalog>,
}

/// Typed values borrowed from the exact captured file bytes. The frontend
/// parses them once; source validation and runtime binding cover the whole set.
#[derive(Clone, Copy)]
pub struct BaseFiles<'a> {
    pub manifest: &'a Manifest,
    pub dimensions: (&'a ObjDimensions, &'a [u8]),
    pub background: Option<(&'a bg_descriptors::Descriptors, &'a [u8])>,
    pub palettes: Option<(&'a bg_palettes::Palettes, &'a [u8])>,
    pub alternate_palettes: Option<(&'a bg_alternate_palettes::Palettes, &'a [u8])>,
    pub packet_programs: Option<(&'a bg_packet_data::Programs, &'a [u8])>,
    pub main_graphics: Option<(&'a bg_graphics::Graphics, &'a [u8])>,
    pub transfers: Option<(&'a bg_transfers::Transfers, &'a [u8])>,
    pub short_texts: Option<(&'a text_short::ShortTexts, &'a [u8])>,
    pub font_bank: Option<(&'a fonts::FontBank, &'a [u8])>,
    pub name_keyboard: Option<(&'a name_keyboard::Keyboard, &'a [u8])>,
    pub obj_lists: Option<(&'a obj_lists::Lists, &'a [u8])>,
    pub obj_animations: Option<(&'a obj_animations::Banks, &'a [u8])>,
    pub obj_graphics: Option<(&'a obj_graphics::Graphics, &'a [u8])>,
    pub ui_palettes: Option<(&'a ui_palettes::Palettes, &'a [u8])>,
    pub ui_graphics: Option<(&'a ui_graphics::Graphics, &'a [u8])>,
    pub items: Option<(&'a items::Items, &'a [u8])>,
    pub item_links: Option<(&'a item_links::Links, &'a [u8])>,
    pub metadata16: Option<(&'a metadata16::Metadata16, &'a [u8])>,
    pub metadata52: Option<(&'a metadata52::Metadata52, &'a [u8])>,
    pub metadata4: Option<(&'a metadata4::Metadata4, &'a [u8])>,
    pub metadata2: Option<(&'a metadata2::Metadata2, &'a [u8])>,
    pub default48: Option<(&'a default48::Template, &'a [u8])>,
    pub constructor_alternates: Option<(&'a constructor_alternates::Alternates, &'a [u8])>,
    pub metadata14: Option<(&'a metadata14::Metadata14, &'a [u8])>,
    pub metadata12: Option<(&'a metadata12::Metadata12, &'a [u8])>,
    pub global_pair_flags: Option<(&'a global_pair_flags::GlobalPairFlags, &'a [u8])>,
    pub evaluation_parameters: Option<(&'a evaluation_parameters::EvaluationParameters, &'a [u8])>,
    pub progression_rewards: Option<(&'a progression_rewards::ProgressionRewards, &'a [u8])>,
    pub progression_gates: Option<(&'a progression_gates::ProgressionGates, &'a [u8])>,
    pub progression_selection_gates: Option<(&'a progression_selection_gates::ProgressionSelectionGates, &'a [u8])>,
    pub aggregate_flag_keys: Option<(&'a aggregate_flag_keys::AggregateFlagKeys, &'a [u8])>,
    pub trade_parameters: Option<(&'a trade_parameters::TradeParameters, &'a [u8])>,
    pub item_effect_parameters: Option<(&'a item_effect_parameters::ItemEffectParameters, &'a [u8])>,
    pub audio_directory: Option<(&'a audio_directory::AudioDirectory, &'a [u8])>,
    pub audio_tones: Option<(&'a audio_tones::AudioTones, &'a [u8])>,
    pub audio_sequences: Option<(&'a audio_sequences::AudioSequences, &'a [u8])>,
    pub audio_driver: Option<(&'a audio_driver::AudioDriver, &'a [u8])>,
    pub palette_effect_dispatch: Option<(&'a palette_effect_dispatch::PaletteEffectDispatch, &'a [u8])>,
    pub palette_programs: Option<(&'a palette_programs::PalettePrograms, &'a [u8])>,
    pub blend_programs: Option<(&'a blend_programs::BlendPrograms, &'a [u8])>,
    pub task_descriptors: Option<(&'a task_descriptors::TaskDescriptors, &'a [u8])>,
    pub effect_event_directories: Option<(&'a effect_event_directories::EffectEventDirectories, &'a [u8])>,
    pub script_commands: Option<(&'a script_commands::ScriptCommands, &'a [u8])>,
    pub script_archive: Option<(&'a script_archive::ScriptArchive, &'a [u8])>,
    pub script_text: Option<(&'a script_text::ScriptText, &'a [u8])>,
    pub script_coordinates: Option<(&'a script_coordinates::ScriptCoordinates, &'a [u8])>,
    pub ui_choice_lists: Option<(&'a ui_choice_lists::UiChoiceLists, &'a [u8])>,
    pub unit_obj_initial_pose: Option<(&'a unit_obj_initial_pose::UnitObjInitialPose, &'a [u8])>,
    pub color_mode_palettes: Option<(&'a color_mode_palettes::ColorModePalettes, &'a [u8])>,
    pub unit_obj_equipment: Option<(&'a unit_obj_equipment::UnitObjEquipment, &'a [u8])>,
    pub unit_obj_normal_battle: Option<(&'a unit_obj_normal_battle::UnitObjNormalBattle, &'a [u8])>,
    pub unit_obj_alternate_battle: Option<(&'a unit_obj_alternate_battle::UnitObjAlternateBattle, &'a [u8])>,
    pub unit_obj_script_special_normal: Option<(&'a unit_obj_script_special_normal::UnitObjScriptSpecialNormal, &'a [u8])>,
    pub unit_obj_script_special_alternate: Option<(&'a unit_obj_script_special_alternate::UnitObjScriptSpecialAlternate, &'a [u8])>,
    pub unit_obj_script_override: Option<(&'a unit_obj_script_override::UnitObjScriptOverride, &'a [u8])>,
    pub unit_obj_script_registry_special: Option<(&'a unit_obj_script_registry_special::UnitObjScriptRegistrySpecial, &'a [u8])>,
    pub unit_obj_script_registry_generic: Option<(&'a unit_obj_script_registry_generic::UnitObjScriptRegistryGeneric, &'a [u8])>,
    pub unit_obj_property_words: Option<(&'a unit_obj_property_words::UnitObjPropertyWords, &'a [u8])>,
    pub unit_obj_byte_property_words: Option<(&'a unit_obj_byte_property_words::UnitObjBytePropertyWords, &'a [u8])>,
    pub unit_obj_ffff_reader_fields: Option<(&'a unit_obj_ffff_reader_fields::UnitObjFfffReaderFields, &'a [u8])>,
    pub slot_job_lookup: Option<(&'a slot_job_lookup::SlotJobLookup, &'a [u8])>,
    pub observed_task_parameters: Option<(&'a observed_task_parameters::ObservedTaskParameters, &'a [u8])>,
    pub constructor_obj_animation: Option<(&'a obj_lists::constructor_animation::Animation, &'a [u8])>,
    pub task_cycle_deltas: Option<(&'a task_cycle_deltas::TaskCycleDeltas, &'a [u8])>,
    pub word_task_obj_animations: Option<(&'a word_task_obj_animations::WordTaskObjAnimations, &'a [u8])>,
    pub world_nodes: Option<(&'a world_nodes::WorldNodes, &'a [u8])>,
    pub world_roads: Option<(&'a world_roads::WorldRoads, &'a [u8])>,
    pub encounter_headers: Option<(&'a encounter_headers::EncounterHeaders, &'a [u8])>,
    pub encounter_unit_templates: Option<(&'a encounter_unit_templates::EncounterUnitTemplates, &'a [u8])>,
    pub encounter_coordinates: Option<(&'a encounter_coordinates::EncounterCoordinates, &'a [u8])>,
    pub metadata28: Option<(&'a metadata28::Metadata28, &'a [u8])>,
    pub records4: Option<(&'a records4::Records4, &'a [u8])>,
    pub conditional_records8: Option<(&'a conditional_records8::Records8, &'a [u8])>,
    pub mission_records70: Option<(&'a mission_records70::Records70, &'a [u8])>,
    pub selection_byte_tables: Option<(&'a selection_byte_tables::ByteTables, &'a [u8])>,
    pub event_records12: Option<(&'a event_records12::EventRecords12, &'a [u8])>,
    pub parameter_records27: Option<(&'a parameter_records27::ParameterRecords27, &'a [u8])>,
    pub world_records38: Option<(&'a world_records38::WorldRecords38, &'a [u8])>,
    pub indexed_graphics: Option<(&'a indexed_graphics::IndexedGraphics, &'a [u8])>,
    pub ui_raw_transfers: Option<(&'a ui_raw_transfers::RawTransfers, &'a [u8])>,
    pub dynamic_raw_fragments: Option<(&'a ui_raw_transfers::dynamic_fragments::Fragments, &'a [u8])>,
    pub conditional_four_byte_lookup: Option<(&'a conditional_four_byte_lookup::Lookup, &'a [u8])>,
    pub conditional_packed_lookup: Option<(&'a conditional_packed_lookup::Lookup, &'a [u8])>,
    pub conditional_descriptor_banks: Option<(&'a conditional_descriptor_banks::DescriptorBanks, &'a [u8])>,
    pub conditional_descriptor_bank_zero: Option<(&'a conditional_descriptor_bank_zero::BankZero, &'a [u8])>,
    pub conditional_property_masks: Option<(&'a conditional_property_masks::Masks, &'a [u8])>,
    pub compact_obj_animations: Option<(&'a compact_obj_animations::CompactAnimations, &'a [u8])>,
    pub secondary_compact_obj_animations: Option<(&'a compact_obj_animations_secondary::CompactAnimationsSecondary, &'a [u8])>,
    pub tertiary_compact_obj_animations: Option<(&'a compact_obj_animations_tertiary::CompactAnimationsTertiary, &'a [u8])>,
    pub quaternary_compact_obj_animations: Option<(&'a compact_obj_animations_quaternary::CompactAnimationsQuaternary, &'a [u8])>,
    pub quinary_compact_obj_animations: Option<(&'a compact_obj_animations_quinary::CompactAnimationsQuinary, &'a [u8])>,
    pub senary_compact_obj_animations: Option<(&'a compact_obj_animations_senary::CompactAnimationsSenary, &'a [u8])>,
    pub septenary_compact_obj_animations: Option<(&'a compact_obj_animations_septenary::CompactAnimationsSeptenary, &'a [u8])>,
    pub octonary_compact_obj_animations: Option<(&'a compact_obj_animations_octonary::CompactAnimationsOctonary, &'a [u8])>,
    pub ui_context_templates: Option<(&'a ui_context_templates::UiContextTemplates, &'a [u8])>,
    pub compact_obj_catalog: Option<(&'a compact_obj_catalog::CompactObjCatalog, &'a [u8])>,
    pub layered_obj_animations: Option<(&'a layered_obj_animations::LayeredObjAnimations, &'a [u8])>,
    pub audio_extended_dispatch: Option<(&'a audio_extended_dispatch::AudioExtendedDispatch, &'a [u8])>,
    pub conditional_item_parameters: Option<(&'a conditional_item_parameters::ConditionalItemParameters, &'a [u8])>,
    pub range_lookup: Option<(&'a range_lookup::RangeLookup, &'a [u8])>,
    pub tilemap_geometry: Option<(&'a tilemap_geometry::TilemapGeometry, &'a [u8])>,
    pub script39_registry_poses: Option<(&'a script39_registry_poses::Script39RegistryPoses, &'a [u8])>,
    pub script39_special_poses: Option<(&'a script39_special_poses::Script39SpecialPoses, &'a [u8])>,
    pub direct_indexed_graphics: Option<(&'a direct_indexed_graphics::DirectIndexedGraphics, &'a [u8])>,
    pub direct_indexed_dictionary: Option<(&'a direct_indexed_dictionary::DirectIndexedDictionary, &'a [u8])>,
    pub context_callback_descriptor: Option<(&'a context_callback_descriptor::ContextCallbackDescriptor, &'a [u8])>,
    pub conditional_language_pair: Option<(&'a conditional_language_pair::ConditionalLanguagePair, &'a [u8])>,
    pub text_reader_registry: Option<(&'a text_reader_registry::TextReaderRegistry, &'a [u8])>,
    pub conditional_substitution_words: Option<(&'a conditional_substitution_words::ConditionalSubstitutionWords, &'a [u8])>,
    pub display_control_tables: Option<(&'a display_control_tables::DisplayControlTables, &'a [u8])>,
    pub compact_primary_alias: Option<(&'a compact_primary_alias::CompactPrimaryAlias, &'a [u8])>,
    pub observed_lz: Option<(&'a observed_lz::ObservedLzBlock, &'a [u8])>,
    pub constructor_template64: Option<(&'a constructor_template64::ConstructorTemplate64, &'a [u8])>,
    pub observed_tilemap: Option<(&'a observed_tilemap::ObservedTilemap, &'a [u8])>,
    pub observed_tilemap_descriptor: Option<(&'a observed_tilemap_descriptor::ObservedTilemapDescriptor, &'a [u8])>,
    pub observed_tilemap_row: Option<(&'a observed_tilemap_row::ObservedTilemapRow, &'a [u8])>,
    pub dma_register_directories: Option<(&'a dma_register_directories::DmaRegisterDirectories, &'a [u8])>,
    pub iwram_memory_helpers: Option<(&'a iwram_memory_helpers::IwramMemoryHelpers, &'a [u8])>,
    pub adjacent_offset_tables: Option<(&'a adjacent_offset_tables::AdjacentOffsetTables, &'a [u8])>,
    pub indexed_records16: Option<(&'a indexed_records16::IndexedRecords16, &'a [u8])>,
    pub indexed_words_3959: Option<(&'a indexed_words_3959::IndexedWords3959, &'a [u8])>,
    pub thumb_switch_tables: Option<(&'a thumb_switch_tables::ThumbSwitchTables, &'a [u8])>,
    pub thumb_pc_literals: Option<(&'a thumb_pc_literals::ThumbPcLiterals, &'a [u8])>,
    pub copied_halfword_lookups: Option<(&'a copied_halfword_lookups::CopiedHalfwordLookups, &'a [u8])>,
    pub indexed_halfwords_39a5: Option<(&'a indexed_halfwords_39a5::IndexedHalfwords39a5, &'a [u8])>,
    pub script_byte_lookup_3ad: Option<(&'a script_byte_lookup_3ad::ScriptByteLookup3ad, &'a [u8])>,
    pub soil_scene_records: Option<(&'a soil_scene_records::SoilSceneRecords, &'a [u8])>,
    pub soil_indirect_records: Option<(&'a soil_indirect_records::SoilIndirectRecords, &'a [u8])>,
    pub soil_indexed_bytes: Option<(&'a soil_indexed_bytes::SoilIndexedBytes, &'a [u8])>,
    pub soil_compact_record: Option<(&'a soil_compact_record::SoilCompactRecord, &'a [u8])>,
    pub soil_scoped_values: Option<(&'a soil_scoped_values::SoilScopedValues, &'a [u8])>,
    pub soil_remaining_rectangles: Option<(&'a soil_remaining_rectangles::SoilRemainingRectangles, &'a [u8])>,
    pub firaga_copied_matrices: Option<(&'a firaga_copied_matrices::FiragaCopiedMatrices, &'a [u8])>,
    pub firaga_compact_cells: Option<(&'a firaga_compact_cells::FiragaCompactCells, &'a [u8])>,
    pub firaga_indexed_obj_bank: Option<(&'a firaga_indexed_obj_bank::FiragaIndexedObjBank, &'a [u8])>,
    pub firaga_compact_obj_bank: Option<(&'a firaga_compact_obj_bank::FiragaCompactObjBank, &'a [u8])>,
    pub firaga_adjacent_compact_obj_bank: Option<(&'a firaga_adjacent_compact_obj_bank::FiragaAdjacentCompactObjBank, &'a [u8])>,
    pub firaga_6c12c8_obj_bank: Option<(&'a firaga_6c12c8_obj_bank::Firaga6c12c8ObjBank, &'a [u8])>,
    pub firaga_transfer_program: Option<(&'a firaga_transfer_program::FiragaTransferProgram, &'a [u8])>,
    pub firaga_palette_rows: Option<(&'a firaga_palette_rows::FiragaPaletteRows, &'a [u8])>,
    pub firaga_stack_initialization: Option<(&'a firaga_stack_initialization::FiragaStackInitialization, &'a [u8])>,
    pub firaga_queued_transfer: Option<(&'a firaga_queued_transfer::FiragaQueuedTransfer, &'a [u8])>,
    pub white_flowers_signed_bytes: Option<(&'a white_flowers_signed_bytes::WhiteFlowersSignedBytes, &'a [u8])>,
    pub white_flowers_isolated_values: Option<(&'a white_flowers_isolated_values::WhiteFlowersIsolatedValues, &'a [u8])>,
    pub tilemap_transfer_directory: Option<(&'a tilemap_transfer_directory::TilemapTransferDirectory, &'a [u8])>,
    pub indexed_tilemap_controls: Option<(&'a tilemap_stream_bank::IndexedTilemapControls, &'a [u8])>,
    pub tilemap_stream_bank: Option<&'a [u8]>,
    pub tilemap_transfer_lz77: Option<&'a [u8]>,
    pub tilemap_type3: Option<&'a [u8]>,
    pub tilemap_type2: Option<&'a [u8]>,
    pub tilemap_direct: Option<&'a [u8]>,
    pub compact_cell_bank: Option<&'a [u8]>,
    pub background_effect_graphics: Option<&'a [u8]>,
    pub localized_obj_animation_banks: Option<&'a [u8]>,
    pub background_screen_banks: Option<&'a [u8]>,
    pub physical_audio_banks: Option<&'a [u8]>,
    pub physical_compact_obj_region: Option<&'a [u8]>,
    pub screen_update_tables: Option<&'a [u8]>,
    pub counted_effect_tables: Option<&'a [u8]>,
    pub map_raster_palettes: Option<(&'a map_raster_palettes::MapRasterPalettes, &'a [u8])>,
    pub auxiliary_commands: Option<(&'a auxiliary_scripts::AuxiliaryCommands, &'a [u8])>,
    pub auxiliary_archive: Option<(&'a auxiliary_scripts::AuxiliaryArchive, &'a [u8])>,
    pub text_catalogs: Option<(&'a text_catalogs::TextCatalogs, &'a [u8])>,
}

impl Extraction {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        Self::from_rom_with_revision(rom, (EXTRACTOR_REV, DECODER_REV))
    }

    /// Keep the recent EUR revisions readable for explicitly selected datasets.
    /// The supplied manifest selects the revision only. Its content and source
    /// identity are validated against the resulting extraction.
    pub fn from_rom_for_manifest(rom: &[u8], manifest: &Manifest) -> Result<Self, String> {
        let revision = match (manifest.extractor_revision, manifest.decoder_revision) {
            previous @ (PRE_INDEXED_TILEMAP_REVISION
            | PRE_TILEMAP_STREAM_FILE_REVISION | PRE_TILEMAP_LZ77_FILE_REVISION
            | PRE_TILEMAP_TYPE3_REVISION | PRE_TILEMAP_TYPE2_REVISION
            | PRE_TILEMAP_DIRECT_REVISION | PRE_TILEMAP_ORIGIN_REVISION
            | PRE_PROJECTED_TILEMAP_DIRECTORIES_REVISION | PRE_COMPACT_CELL_BANK_REVISION
            | PRE_ADDITIONAL_UI_TABLE_REVISION | PRE_DIRECT_UI_LITERALS_REVISION
            | PRE_CUSTOM_DECODER_ROOTS_REVISION | PRE_LOCALIZED_TEXT_ROOTS_REVISION
            | PRE_INDEXED_READER_BANKS_REVISION | PRE_VRAM_LZ_BANK_REVISION
            | PRE_HALFWORD_COPY_REVISION | PRE_HALFWORD_PRELOAD_REVISION
            | PRE_VRAM_LOADER_REVISION | PRE_ROW_PAIR_REVISION
            | PRE_STATE_INITIALIZER_REVISION | PRE_SCROLL_REGISTER_REVISION
            | PRE_STARTUP_TASK_ROOTS_REVISION | PRE_STARTUP_DISPATCHER_REVISION
            | PRE_TITLE_CALLBACKS_REVISION | PRE_TITLE_FRONTEND_REVISION
            | PRE_TITLE_COLOR_MODE_REVISION | PRE_COLOR_MODE_PAIRS_REVISION
            | PRE_SCANLINE_BLEND_REVISION | PRE_INDEXED_VRAM_SELECTORS_REVISION
            | PRE_WORLD_MAP_LOOKUPS_REVISION | PRE_TILEMAP_RECTANGLES_REVISION
            | PRE_TIMED_SOUND_BANKS_REVISION | PRE_INITIALIZER_BANK_REVISION
            | PRE_INITIALIZER_VARIANTS_REVISION | PRE_INITIALIZER_PAIRS_REVISION
            | PRE_SIGNED_MAP_DELTAS_REVISION | PRE_WORLD_NODE_RENDERER_REVISION
            | PRE_WORLD_STATE_TASK_REVISION | PRE_STACK_BYTE_COPIES_REVISION
            | PRE_FIXED_BYTE_COPIES_REVISION | PRE_DIRECT_COPY_ARGUMENTS_REVISION
            | PRE_COPY_PRODUCER_LITERALS_REVISION | PRE_SHARED_SLOT_LITERALS_REVISION
            | PRE_HELPER_FIELD_DISPATCH_REVISION | PRE_CONTEXT_RECORD_LITERALS_REVISION
            | PRE_SCRATCH_RESTORE_LITERALS_REVISION | PRE_CACHED_OBJECT_REVISION
            | PRE_INDEXED_EXCHANGE_REVISION | PRE_FIVE_BYTE_LIST_REVISION
            | PRE_PALETTE_TASK_REVISION | PRE_RECOVERED_TILEMAP_LISTS_REVISION
            | PRE_BACKGROUND_VARIANT_STATE_REVISION | PRE_CPU_FAST_SET_LITERALS_REVISION
            | PRE_BACKGROUND_EFFECT_GRAPHICS_REVISION
            | PRE_LOCALIZED_OBJ_ANIMATION_BANKS_REVISION
            | PRE_BACKGROUND_SCREEN_BANKS_REVISION
            | PRE_PHYSICAL_AUDIO_BANKS_REVISION
            | PRE_PHYSICAL_COMPACT_OBJ_REGION_REVISION
            | PRE_SCREEN_UPDATE_TABLES_REVISION
            | PRE_COUNTED_EFFECT_TABLES_REVISION) => previous,
            _ => (EXTRACTOR_REV, DECODER_REV),
        };
        Self::from_rom_with_revision(rom, revision)
    }

    fn from_rom_with_revision(rom: &[u8], revision: (u32, u32)) -> Result<Self, String> {
        let hash = ffta_rom::digest(rom);
        let (profile, ranges): (&str, &[(u32, &str)]) = match hash.as_str() {
            ffta_rom::SHA256 => ("usa", &[(0x149510, "direct"), (0xa398dc, "iwram-copy")]),
            ffta_rom::EUROPE_SHA256 => (
                "europe",
                &[(0x14ced8, "rom-table"), (0xd976e8, "iwram-copy")],
            ),
            _ => return Err("Decoded resources require a verified FFTA EUR or USA ROM".into()),
        };
        let catalog = bg_descriptors::Catalog::from_rom(rom)?;
        let palettes = bg_palettes::Catalog::from_descriptors(rom, &catalog)?;
        let alternates = bg_alternate_palettes::Catalog::from_rom(rom)?;
        let packet_programs = bg_packet_data::Catalog::from_rom(rom)?;
        let mut result = Self::from_ranges(rom, hash, profile, ranges, DIMENSIONS_SHA256)?;
        if profile == "europe" {
            result.manifest.resources[0].sources.extend(
                obj_derived_dimensions::source_ranges(rom, &result.dimensions)?
            );
        }
        result.background = Some(catalog.decoded);
        result.manifest.coverage.push(BG_DESCRIPTORS_ID.into());
        result.manifest.resources.push(Resource {
            id: BG_DESCRIPTORS_ID.into(),
            path: BG_DESCRIPTORS_FILE.into(),
            format: BG_DESCRIPTORS_FORMAT.into(),
            sha256: String::new(),
            dependencies: Vec::new(),
            sources: vec![catalog.source],
        });
        result.manifest.coverage.push(BG_BASE_PALETTES_ID.into());
        result.manifest.resources.push(Resource {
            id: BG_BASE_PALETTES_ID.into(),
            path: BG_BASE_PALETTES_FILE.into(),
            format: BG_BASE_PALETTES_FORMAT.into(),
            sha256: String::new(),
            dependencies: vec![BG_DESCRIPTORS_ID.into()],
            sources: palettes
                .sources
                .iter()
                .map(|source| source.range.clone())
                .collect(),
        });
        result.palettes = Some(palettes);
        result.manifest.coverage.push(BG_ALTERNATE_PALETTES_ID.into());
        result.manifest.resources.push(Resource {
            id: BG_ALTERNATE_PALETTES_ID.into(),
            path: BG_ALTERNATE_PALETTES_FILE.into(),
            format: BG_ALTERNATE_PALETTES_FORMAT.into(),
            sha256: String::new(),
            dependencies: vec![BG_DESCRIPTORS_ID.into()],
            sources: alternates.tables.iter().chain(&alternates.sources).cloned().collect(),
        });
        result.alternate_palettes = Some(alternates);
        result.manifest.coverage.push(BG_PACKET_PROGRAMS_ID.into());
        result.manifest.resources.push(Resource {
            id: BG_PACKET_PROGRAMS_ID.into(),
            path: BG_PACKET_PROGRAMS_FILE.into(),
            format: BG_PACKET_PROGRAMS_FORMAT.into(),
            sha256: String::new(),
            dependencies: vec![BG_DESCRIPTORS_ID.into()],
            sources: packet_programs.sources.clone(),
        });
        result.packet_programs = Some(packet_programs);
        // User priority is EUR. Preserve USA's existing dataset contract until
        // its graphics catalog and new managed path are independently verified.
        if profile == "europe" {
            let graphics = bg_graphics::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(BG_MAIN_GRAPHICS_ID.into());
            result.manifest.resources.push(Resource {
                id: BG_MAIN_GRAPHICS_ID.into(), path: BG_MAIN_GRAPHICS_FILE.into(),
                format: BG_MAIN_GRAPHICS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![BG_DESCRIPTORS_ID.into()], sources: graphics.sources.clone(),
            });
            result.main_graphics = Some(graphics);
            let transfers = bg_transfers::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(BG_TRANSFER_PROGRAMS_ID.into());
            result.manifest.resources.push(Resource {
                id: BG_TRANSFER_PROGRAMS_ID.into(), path: BG_TRANSFER_PROGRAMS_FILE.into(),
                format: BG_TRANSFER_PROGRAMS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![BG_DESCRIPTORS_ID.into()], sources: transfers.sources.clone(),
            });
            result.transfers = Some(transfers);
            let short_texts = text_short::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SHORT_TEXTS_ID.into());
            result.manifest.resources.push(Resource {
                id: SHORT_TEXTS_ID.into(), path: SHORT_TEXTS_FILE.into(),
                format: SHORT_TEXTS_FORMAT.into(), sha256: String::new(),
                dependencies: Vec::new(), sources: short_texts.sources.clone(),
            });
            result.short_texts = Some(short_texts);
            let texts = text_catalogs::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TEXT_CATALOGS_ID.into());
            result.manifest.resources.push(Resource {
                id: TEXT_CATALOGS_ID.into(), path: TEXT_CATALOGS_FILE.into(), format: TEXT_CATALOGS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![SHORT_TEXTS_ID.into()], sources: texts.sources.clone(),
            });
            result.text_catalogs = Some(texts);
            let font_bank = fonts::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(FONT_BANK_ID.into());
            result.manifest.resources.push(Resource {
                id: FONT_BANK_ID.into(), path: FONT_BANK_FILE.into(), format: FONT_BANK_FORMAT.into(),
                sha256: String::new(), dependencies: vec![TEXT_CATALOGS_ID.into()],
                sources: font_bank.sources.clone(),
            });
            result.font_bank = Some(font_bank);
            let name_keyboard = name_keyboard::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(NAME_KEYBOARD_ID.into());
            result.manifest.resources.push(Resource {
                id: NAME_KEYBOARD_ID.into(), path: NAME_KEYBOARD_FILE.into(), format: NAME_KEYBOARD_FORMAT.into(),
                sha256: String::new(), dependencies: vec![FONT_BANK_ID.into()],
                sources: name_keyboard.sources.clone(),
            });
            result.name_keyboard = Some(name_keyboard);
            let obj_lists = obj_lists::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(OBJ_STATIC_LISTS_ID.into());
            result.manifest.resources.push(Resource {
                id: OBJ_STATIC_LISTS_ID.into(), path: OBJ_STATIC_LISTS_FILE.into(), format: OBJ_STATIC_LISTS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: obj_lists.sources.clone(),
            });
            result.obj_lists = Some(obj_lists);
            let animations = obj_animations::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(OBJ_ANIMATION_BANKS_ID.into());
            result.manifest.resources.push(Resource {
                id: OBJ_ANIMATION_BANKS_ID.into(), path: OBJ_ANIMATION_BANKS_FILE.into(), format: OBJ_ANIMATION_BANKS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: animations.sources.clone(),
            });
            result.obj_animations = Some(animations);
            let graphics = obj_graphics::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(OBJ_CUSTOM_GRAPHICS_ID.into());
            result.manifest.resources.push(Resource {
                id: OBJ_CUSTOM_GRAPHICS_ID.into(), path: OBJ_CUSTOM_GRAPHICS_FILE.into(), format: OBJ_CUSTOM_GRAPHICS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: graphics.sources.clone(),
            });
            result.obj_graphics = Some(graphics);
            let palettes = ui_palettes::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UI_RAW_PALETTES_ID.into());
            result.manifest.resources.push(Resource {
                id: UI_RAW_PALETTES_ID.into(), path: UI_RAW_PALETTES_FILE.into(), format: UI_RAW_PALETTES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: palettes.sources.clone(),
            });
            result.ui_palettes = Some(palettes);
            let layout = if revision > PRE_DIRECT_UI_LITERALS_REVISION {
                ui_graphics::SourceLayout::DirectLiterals
            } else if revision > PRE_ADDITIONAL_UI_TABLE_REVISION {
                ui_graphics::SourceLayout::AdditionalTable
            } else {
                ui_graphics::SourceLayout::Legacy
            };
            let graphics = ui_graphics::Catalog::from_rom_with_layout(rom, layout)?;
            let mut dependencies = vec![OBJ_DIMENSIONS_ID.into(), OBJ_CUSTOM_GRAPHICS_ID.into()];
            if layout == ui_graphics::SourceLayout::DirectLiterals {
                dependencies.push(UNIT_OBJ_FFFF_READER_FIELDS_ID.into());
            }
            result.manifest.coverage.push(UI_CUSTOM_GRAPHICS_ID.into());
            result.manifest.resources.push(Resource {
                id: UI_CUSTOM_GRAPHICS_ID.into(), path: UI_CUSTOM_GRAPHICS_FILE.into(), format: UI_CUSTOM_GRAPHICS_FORMAT.into(),
                sha256: String::new(), dependencies, sources: graphics.sources.clone(),
            });
            result.ui_graphics = Some(graphics);
            let items = items::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(ITEM_RECORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: ITEM_RECORDS_ID.into(), path: ITEM_RECORDS_FILE.into(), format: ITEM_RECORDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: items.sources.clone(),
            });
            result.items = Some(items);
            let links = item_links::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(ITEM_LINKS_ID.into());
            result.manifest.resources.push(Resource {
                id: ITEM_LINKS_ID.into(), path: ITEM_LINKS_FILE.into(), format: ITEM_LINKS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ITEM_RECORDS_ID.into()], sources: links.sources.clone(),
            });
            result.item_links = Some(links);
            let metadata = metadata16::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(METADATA16_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA16_ID.into(), path: METADATA16_FILE.into(), format: METADATA16_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_DIMENSIONS_ID.into()], sources: metadata.sources.clone(),
            });
            result.metadata16 = Some(metadata);
            let metadata = metadata52::Catalog::from_rom(rom)?;
            metadata.decoded.validate_links(&result.item_links.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(METADATA52_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA52_ID.into(), path: METADATA52_FILE.into(), format: METADATA52_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ITEM_LINKS_ID.into()], sources: metadata.sources.clone(),
            });
            result.metadata52 = Some(metadata);
            let metadata = metadata4::Catalog::from_rom(rom)?;
            metadata.decoded.validate_metadata52(&result.metadata52.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(METADATA4_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA4_ID.into(), path: METADATA4_FILE.into(), format: METADATA4_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into()], sources: metadata.sources.clone(),
            });
            result.metadata4 = Some(metadata);
            let metadata = metadata2::Catalog::from_rom(rom)?;
            metadata.decoded.validate_metadata52(&result.metadata52.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(METADATA2_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA2_ID.into(), path: METADATA2_FILE.into(), format: METADATA2_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into()], sources: metadata.sources.clone(),
            });
            result.metadata2 = Some(metadata);
            let metadata = default48::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(DEFAULT48_ID.into());
            result.manifest.resources.push(Resource {
                id: DEFAULT48_ID.into(), path: DEFAULT48_FILE.into(), format: DEFAULT48_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.default48 = Some(metadata);
            let metadata = constructor_alternates::Catalog::from_rom(rom)?;
            metadata.decoded.validate_items(&result.items.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(CONSTRUCTOR_ALTERNATES_ID.into());
            result.manifest.resources.push(Resource {
                id: CONSTRUCTOR_ALTERNATES_ID.into(), path: CONSTRUCTOR_ALTERNATES_FILE.into(), format: CONSTRUCTOR_ALTERNATES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ITEM_RECORDS_ID.into()], sources: metadata.sources.clone(),
            });
            result.constructor_alternates = Some(metadata);
            let metadata = metadata14::Catalog::from_rom(rom)?;
            metadata.decoded.validate_metadata52(&result.metadata52.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(METADATA14_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA14_ID.into(), path: METADATA14_FILE.into(), format: METADATA14_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into()], sources: metadata.sources.clone(),
            });
            result.metadata14 = Some(metadata);
            let metadata = metadata12::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(METADATA12_ID.into());
            result.manifest.resources.push(Resource {
                id: METADATA12_ID.into(), path: METADATA12_FILE.into(), format: METADATA12_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.metadata12 = Some(metadata);
            let metadata = global_pair_flags::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(GLOBAL_PAIR_FLAGS_ID.into());
            result.manifest.resources.push(Resource {
                id: GLOBAL_PAIR_FLAGS_ID.into(), path: GLOBAL_PAIR_FLAGS_FILE.into(), format: GLOBAL_PAIR_FLAGS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.global_pair_flags = Some(metadata);
            let metadata = evaluation_parameters::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(EVALUATION_PARAMETERS_ID.into());
            result.manifest.resources.push(Resource {
                id: EVALUATION_PARAMETERS_ID.into(), path: EVALUATION_PARAMETERS_FILE.into(), format: EVALUATION_PARAMETERS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.evaluation_parameters = Some(metadata);
            let metadata = progression_rewards::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PROGRESSION_REWARDS_ID.into());
            result.manifest.resources.push(Resource {
                id: PROGRESSION_REWARDS_ID.into(), path: PROGRESSION_REWARDS_FILE.into(), format: PROGRESSION_REWARDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.progression_rewards = Some(metadata);
            let metadata = progression_gates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PROGRESSION_GATES_ID.into());
            result.manifest.resources.push(Resource {
                id: PROGRESSION_GATES_ID.into(), path: PROGRESSION_GATES_FILE.into(), format: PROGRESSION_GATES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.progression_gates = Some(metadata);
            let metadata = progression_selection_gates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PROGRESSION_SELECTION_GATES_ID.into());
            result.manifest.resources.push(Resource {
                id: PROGRESSION_SELECTION_GATES_ID.into(), path: PROGRESSION_SELECTION_GATES_FILE.into(), format: PROGRESSION_SELECTION_GATES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.progression_selection_gates = Some(metadata);
            let metadata = aggregate_flag_keys::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(AGGREGATE_FLAG_KEYS_ID.into());
            result.manifest.resources.push(Resource {
                id: AGGREGATE_FLAG_KEYS_ID.into(), path: AGGREGATE_FLAG_KEYS_FILE.into(), format: AGGREGATE_FLAG_KEYS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.aggregate_flag_keys = Some(metadata);
            let metadata = trade_parameters::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TRADE_PARAMETERS_ID.into());
            result.manifest.resources.push(Resource {
                id: TRADE_PARAMETERS_ID.into(), path: TRADE_PARAMETERS_FILE.into(), format: TRADE_PARAMETERS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.trade_parameters = Some(metadata);
            let metadata = item_effect_parameters::Catalog::from_rom(rom)?;
            metadata.decoded.validate_items(&result.items.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(ITEM_EFFECT_PARAMETERS_ID.into());
            result.manifest.resources.push(Resource {
                id: ITEM_EFFECT_PARAMETERS_ID.into(), path: ITEM_EFFECT_PARAMETERS_FILE.into(), format: ITEM_EFFECT_PARAMETERS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ITEM_RECORDS_ID.into()], sources: metadata.sources.clone(),
            });
            result.item_effect_parameters = Some(metadata);
            let metadata = audio_directory::Catalog::from_rom_with_timed_sounds(
                rom, revision > PRE_TIMED_SOUND_BANKS_REVISION
            )?;
            result.manifest.coverage.push(AUDIO_DIRECTORY_ID.into());
            result.manifest.resources.push(Resource {
                id: AUDIO_DIRECTORY_ID.into(), path: AUDIO_DIRECTORY_FILE.into(), format: AUDIO_DIRECTORY_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.audio_directory = Some(metadata);
            let metadata = audio_tones::Catalog::from_rom(rom)?;
            metadata.decoded.validate_directory(&result.audio_directory.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(AUDIO_TONES_ID.into());
            result.manifest.resources.push(Resource {
                id: AUDIO_TONES_ID.into(), path: AUDIO_TONES_FILE.into(), format: AUDIO_TONES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![AUDIO_DIRECTORY_ID.into()], sources: metadata.sources.clone(),
            });
            result.audio_tones = Some(metadata);
            let metadata = audio_sequences::Catalog::from_rom(rom)?;
            metadata.decoded.validate_owners(&result.audio_directory.as_ref().unwrap().decoded,&result.audio_tones.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(AUDIO_SEQUENCES_ID.into());
            result.manifest.resources.push(Resource {
                id: AUDIO_SEQUENCES_ID.into(), path: AUDIO_SEQUENCES_FILE.into(), format: AUDIO_SEQUENCES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![AUDIO_DIRECTORY_ID.into(),AUDIO_TONES_ID.into()], sources: metadata.sources.clone(),
            });
            result.audio_sequences = Some(metadata);
            let metadata = audio_driver::Catalog::from_rom(rom)?;
            metadata.decoded.validate_sequence_owner(&result.audio_sequences.as_ref().unwrap().decoded)?;
            result.manifest.coverage.push(AUDIO_DRIVER_ID.into());
            result.manifest.resources.push(Resource {
                id: AUDIO_DRIVER_ID.into(), path: AUDIO_DRIVER_FILE.into(), format: AUDIO_DRIVER_FORMAT.into(),
                sha256: String::new(), dependencies: vec![AUDIO_SEQUENCES_ID.into()], sources: metadata.sources.clone(),
            });
            result.audio_driver = Some(metadata);
            let metadata = palette_effect_dispatch::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PALETTE_EFFECT_DISPATCH_ID.into());
            result.manifest.resources.push(Resource {
                id: PALETTE_EFFECT_DISPATCH_ID.into(), path: PALETTE_EFFECT_DISPATCH_FILE.into(), format: PALETTE_EFFECT_DISPATCH_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.palette_effect_dispatch = Some(metadata);
            let metadata = palette_programs::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PALETTE_PROGRAMS_ID.into());
            result.manifest.resources.push(Resource {
                id: PALETTE_PROGRAMS_ID.into(), path: PALETTE_PROGRAMS_FILE.into(), format: PALETTE_PROGRAMS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.palette_programs = Some(metadata);
            let metadata = blend_programs::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(BLEND_PROGRAMS_ID.into());
            result.manifest.resources.push(Resource {
                id: BLEND_PROGRAMS_ID.into(), path: BLEND_PROGRAMS_FILE.into(), format: BLEND_PROGRAMS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.blend_programs = Some(metadata);
            let metadata = task_descriptors::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TASK_DESCRIPTORS_ID.into());
            result.manifest.resources.push(Resource {
                id: TASK_DESCRIPTORS_ID.into(), path: TASK_DESCRIPTORS_FILE.into(), format: TASK_DESCRIPTORS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.task_descriptors = Some(metadata);
            let metadata = effect_event_directories::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(EFFECT_EVENT_DIRECTORIES_ID.into());
            result.manifest.resources.push(Resource {
                id: EFFECT_EVENT_DIRECTORIES_ID.into(), path: EFFECT_EVENT_DIRECTORIES_FILE.into(), format: EFFECT_EVENT_DIRECTORIES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![TASK_DESCRIPTORS_ID.into()], sources: metadata.sources.clone(),
            });
            result.effect_event_directories = Some(metadata);
            let metadata = script_commands::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SCRIPT_COMMANDS_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT_COMMANDS_ID.into(), path: SCRIPT_COMMANDS_FILE.into(), format: SCRIPT_COMMANDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.script_commands = Some(metadata);
            let metadata = script_archive::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SCRIPT_ARCHIVE_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT_ARCHIVE_ID.into(), path: SCRIPT_ARCHIVE_FILE.into(), format: SCRIPT_ARCHIVE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![SCRIPT_COMMANDS_ID.into(), EFFECT_EVENT_DIRECTORIES_ID.into()], sources: metadata.sources.clone(),
            });
            result.script_archive = Some(metadata);
            let metadata = script_text::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SCRIPT_TEXT_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT_TEXT_ID.into(), path: SCRIPT_TEXT_FILE.into(), format: SCRIPT_TEXT_FORMAT.into(),
                sha256: String::new(), dependencies: vec![SCRIPT_ARCHIVE_ID.into(), FONT_BANK_ID.into()], sources: metadata.sources.clone(),
            });
            result.script_text = Some(metadata);
            let metadata = script_coordinates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SCRIPT_COORDINATES_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT_COORDINATES_ID.into(), path: SCRIPT_COORDINATES_FILE.into(), format: SCRIPT_COORDINATES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![SCRIPT_ARCHIVE_ID.into(), SCRIPT_COMMANDS_ID.into()], sources: metadata.sources.clone(),
            });
            result.script_coordinates = Some(metadata);
            let metadata = auxiliary_scripts::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(AUXILIARY_COMMANDS_ID.into());
            result.manifest.resources.push(Resource {
                id: AUXILIARY_COMMANDS_ID.into(), path: AUXILIARY_COMMANDS_FILE.into(), format: AUXILIARY_COMMANDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: vec![metadata.sources[1].clone()],
            });
            result.manifest.coverage.push(AUXILIARY_ARCHIVE_ID.into());
            result.manifest.resources.push(Resource {
                id: AUXILIARY_ARCHIVE_ID.into(), path: AUXILIARY_ARCHIVE_FILE.into(), format: AUXILIARY_ARCHIVE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![AUXILIARY_COMMANDS_ID.into()], sources: vec![metadata.sources[0].clone()],
            });
            result.auxiliary_scripts = Some(metadata);
            let metadata = map_raster_palettes::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(MAP_RASTER_PALETTES_ID.into());
            result.manifest.resources.push(Resource {
                id: MAP_RASTER_PALETTES_ID.into(), path: MAP_RASTER_PALETTES_FILE.into(), format: MAP_RASTER_PALETTES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![BG_DESCRIPTORS_ID.into()], sources: metadata.sources.clone(),
            });
            result.map_raster_palettes = Some(metadata);
            let metadata = ui_choice_lists::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UI_CHOICE_LISTS_ID.into());
            result.manifest.resources.push(Resource {
                id: UI_CHOICE_LISTS_ID.into(), path: UI_CHOICE_LISTS_FILE.into(), format: UI_CHOICE_LISTS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.ui_choice_lists = Some(metadata);
            let metadata = unit_obj_initial_pose::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_INITIAL_POSE_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_INITIAL_POSE_ID.into(), path: UNIT_OBJ_INITIAL_POSE_FILE.into(), format: UNIT_OBJ_INITIAL_POSE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), METADATA14_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_initial_pose = Some(metadata);
            let metadata = color_mode_palettes::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(COLOR_MODE_PALETTES_ID.into());
            result.manifest.resources.push(Resource {
                id: COLOR_MODE_PALETTES_ID.into(), path: COLOR_MODE_PALETTES_FILE.into(), format: COLOR_MODE_PALETTES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA14_ID.into()], sources: metadata.sources.clone(),
            });
            result.color_mode_palettes = Some(metadata);
            let metadata = unit_obj_equipment::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_EQUIPMENT_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_EQUIPMENT_ID.into(), path: UNIT_OBJ_EQUIPMENT_FILE.into(), format: UNIT_OBJ_EQUIPMENT_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ITEM_RECORDS_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_equipment = Some(metadata);
            let metadata = unit_obj_normal_battle::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_NORMAL_BATTLE_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_NORMAL_BATTLE_ID.into(), path: UNIT_OBJ_NORMAL_BATTLE_FILE.into(), format: UNIT_OBJ_NORMAL_BATTLE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), METADATA14_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_EQUIPMENT_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_normal_battle = Some(metadata);
            let metadata = unit_obj_alternate_battle::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_ALTERNATE_BATTLE_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_ALTERNATE_BATTLE_ID.into(), path: UNIT_OBJ_ALTERNATE_BATTLE_FILE.into(), format: UNIT_OBJ_ALTERNATE_BATTLE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), METADATA14_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_EQUIPMENT_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_alternate_battle = Some(metadata);
            let metadata = unit_obj_script_special_normal::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_ID.into(), path: UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE.into(), format: UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA14_ID.into(), SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_script_special_normal = Some(metadata);
            let metadata = unit_obj_script_special_alternate::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into(), path: UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE.into(), format: UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA14_ID.into(), SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_script_special_alternate = Some(metadata);
            let metadata = unit_obj_script_override::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_SCRIPT_OVERRIDE_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_SCRIPT_OVERRIDE_ID.into(), path: UNIT_OBJ_SCRIPT_OVERRIDE_FILE.into(), format: UNIT_OBJ_SCRIPT_OVERRIDE_FORMAT.into(),
                sha256: String::new(), dependencies: vec![SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_script_override = Some(metadata);
            let metadata = unit_obj_script_registry_special::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID.into(), path: UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE.into(), format: UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA14_ID.into(), SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_EQUIPMENT_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into(), UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_ID.into(), UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_script_registry_special = Some(metadata);
            let metadata = unit_obj_script_registry_generic::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_ID.into(), path: UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE.into(), format: UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_EQUIPMENT_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into(), UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into(), UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_script_registry_generic = Some(metadata);
            let metadata = unit_obj_property_words::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_PROPERTY_WORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_PROPERTY_WORDS_ID.into(), path: UNIT_OBJ_PROPERTY_WORDS_FILE.into(), format: UNIT_OBJ_PROPERTY_WORDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), METADATA14_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_property_words = Some(metadata);
            let metadata = unit_obj_byte_property_words::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_BYTE_PROPERTY_WORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_BYTE_PROPERTY_WORDS_ID.into(), path: UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE.into(), format: UNIT_OBJ_BYTE_PROPERTY_WORDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![UNIT_OBJ_PROPERTY_WORDS_ID.into(), COLOR_MODE_PALETTES_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_byte_property_words = Some(metadata);
            let metadata = unit_obj_ffff_reader_fields::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UNIT_OBJ_FFFF_READER_FIELDS_ID.into());
            result.manifest.resources.push(Resource {
                id: UNIT_OBJ_FFFF_READER_FIELDS_ID.into(), path: UNIT_OBJ_FFFF_READER_FIELDS_FILE.into(), format: UNIT_OBJ_FFFF_READER_FIELDS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into(), SCRIPT_ARCHIVE_ID.into()], sources: metadata.sources.clone(),
            });
            result.unit_obj_ffff_reader_fields = Some(metadata);
            let metadata = slot_job_lookup::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SLOT_JOB_LOOKUP_ID.into());
            result.manifest.resources.push(Resource {
                id: SLOT_JOB_LOOKUP_ID.into(), path: SLOT_JOB_LOOKUP_FILE.into(), format: SLOT_JOB_LOOKUP_FORMAT.into(),
                sha256: String::new(), dependencies: vec![METADATA52_ID.into()], sources: metadata.sources.clone(),
            });
            result.slot_job_lookup = Some(metadata);
            let metadata = observed_task_parameters::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(OBSERVED_TASK_PARAMETERS_ID.into());
            result.manifest.resources.push(Resource {
                id: OBSERVED_TASK_PARAMETERS_ID.into(), path: OBSERVED_TASK_PARAMETERS_FILE.into(), format: OBSERVED_TASK_PARAMETERS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.observed_task_parameters = Some(metadata);
            let mut metadata = obj_lists::constructor_animation::Catalog::from_rom(rom)?;
            metadata.sources.retain(|s| s.role == "constructor-obj-animation");
            result.manifest.coverage.push(CONSTRUCTOR_OBJ_ANIMATION_ID.into());
            result.manifest.resources.push(Resource {
                id: CONSTRUCTOR_OBJ_ANIMATION_ID.into(), path: CONSTRUCTOR_OBJ_ANIMATION_FILE.into(), format: CONSTRUCTOR_OBJ_ANIMATION_FORMAT.into(),
                sha256: String::new(), dependencies: vec![OBJ_STATIC_LISTS_ID.into()], sources: metadata.sources.clone(),
            });
            result.constructor_obj_animation = Some(metadata);
            let metadata = task_cycle_deltas::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TASK_CYCLE_DELTAS_ID.into());
            result.manifest.resources.push(Resource {
                id: TASK_CYCLE_DELTAS_ID.into(), path: TASK_CYCLE_DELTAS_FILE.into(), format: TASK_CYCLE_DELTAS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.task_cycle_deltas = Some(metadata);
            let metadata = word_task_obj_animations::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(WORD_TASK_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {
                id: WORD_TASK_OBJ_ANIMATIONS_ID.into(), path: WORD_TASK_OBJ_ANIMATIONS_FILE.into(), format: WORD_TASK_OBJ_ANIMATIONS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.word_task_obj_animations = Some(metadata);
            let metadata = world_nodes::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(WORLD_NODES_ID.into());
            result.manifest.resources.push(Resource {
                id: WORLD_NODES_ID.into(), path: WORLD_NODES_FILE.into(), format: WORLD_NODES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.world_nodes = Some(metadata);
            let metadata = world_roads::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(WORLD_ROADS_ID.into());
            result.manifest.resources.push(Resource {
                id: WORLD_ROADS_ID.into(), path: WORLD_ROADS_FILE.into(), format: WORLD_ROADS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.world_roads = Some(metadata);
            let metadata = encounter_headers::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(ENCOUNTER_HEADERS_ID.into());
            result.manifest.resources.push(Resource {
                id: ENCOUNTER_HEADERS_ID.into(), path: ENCOUNTER_HEADERS_FILE.into(), format: ENCOUNTER_HEADERS_FORMAT.into(),
                sha256: String::new(), dependencies: vec![CONSTRUCTOR_ALTERNATES_ID.into()], sources: metadata.sources.clone(),
            });
            result.encounter_headers = Some(metadata);
            let metadata = encounter_unit_templates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(ENCOUNTER_UNIT_TEMPLATES_ID.into());
            result.manifest.resources.push(Resource {
                id: ENCOUNTER_UNIT_TEMPLATES_ID.into(), path: ENCOUNTER_UNIT_TEMPLATES_FILE.into(), format: ENCOUNTER_UNIT_TEMPLATES_FORMAT.into(),
                sha256: String::new(), dependencies: vec![ENCOUNTER_HEADERS_ID.into(), CONSTRUCTOR_ALTERNATES_ID.into(), ITEM_RECORDS_ID.into(), METADATA14_ID.into(), METADATA52_ID.into()], sources: metadata.sources.clone(),
            });
            result.encounter_unit_templates = Some(metadata);
            let metadata = encounter_coordinates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(ENCOUNTER_COORDINATES_ID.into());
            result.manifest.resources.push(Resource {id:ENCOUNTER_COORDINATES_ID.into(),path:ENCOUNTER_COORDINATES_FILE.into(),format:ENCOUNTER_COORDINATES_FORMAT.into(),sha256:String::new(),dependencies:vec![ENCOUNTER_HEADERS_ID.into()],sources:metadata.sources.clone()});
            result.encounter_coordinates = Some(metadata);
            let metadata = metadata28::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(METADATA28_ID.into());
            result.manifest.resources.push(Resource {id:METADATA28_ID.into(),path:METADATA28_FILE.into(),format:METADATA28_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.metadata28 = Some(metadata);
            let metadata = records4::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(RECORDS4_ID.into());
            result.manifest.resources.push(Resource {id:RECORDS4_ID.into(),path:RECORDS4_FILE.into(),format:RECORDS4_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.records4 = Some(metadata);
            let metadata = conditional_records8::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_RECORDS8_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_RECORDS8_ID.into(),path:CONDITIONAL_RECORDS8_FILE.into(),format:CONDITIONAL_RECORDS8_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_records8 = Some(metadata);
            let metadata = mission_records70::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(MISSION_RECORDS70_ID.into());
            result.manifest.resources.push(Resource {id:MISSION_RECORDS70_ID.into(),path:MISSION_RECORDS70_FILE.into(),format:MISSION_RECORDS70_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.mission_records70 = Some(metadata);
            let metadata = selection_byte_tables::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SELECTION_BYTE_TABLES_ID.into());
            result.manifest.resources.push(Resource {id:SELECTION_BYTE_TABLES_ID.into(),path:SELECTION_BYTE_TABLES_FILE.into(),format:SELECTION_BYTE_TABLES_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.selection_byte_tables = Some(metadata);
            let metadata = event_records12::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(EVENT_RECORDS12_ID.into());
            result.manifest.resources.push(Resource {id:EVENT_RECORDS12_ID.into(),path:EVENT_RECORDS12_FILE.into(),format:EVENT_RECORDS12_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.event_records12 = Some(metadata);
            let metadata = parameter_records27::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(PARAMETER_RECORDS27_ID.into());
            result.manifest.resources.push(Resource {id:PARAMETER_RECORDS27_ID.into(),path:PARAMETER_RECORDS27_FILE.into(),format:PARAMETER_RECORDS27_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.parameter_records27 = Some(metadata);
            let metadata = world_records38::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(WORLD_RECORDS38_ID.into());
            result.manifest.resources.push(Resource {id:WORLD_RECORDS38_ID.into(),path:WORLD_RECORDS38_FILE.into(),format:WORLD_RECORDS38_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.world_records38 = Some(metadata);
            let indexed_layout = if revision > PRE_WORLD_MAP_LOOKUPS_REVISION {
                indexed_graphics::SourceLayout::WorldMapLookups
            } else if revision > PRE_INDEXED_VRAM_SELECTORS_REVISION {
                indexed_graphics::SourceLayout::VramSelectors
            } else if revision > PRE_INDEXED_READER_BANKS_REVISION {
                indexed_graphics::SourceLayout::ReaderBanks
            } else {
                indexed_graphics::SourceLayout::Legacy
            };
            let metadata = indexed_graphics::Catalog::from_rom_with_layout(rom, indexed_layout)?;
            result.manifest.coverage.push(INDEXED_GRAPHICS_ID.into());
            result.manifest.resources.push(Resource {id:INDEXED_GRAPHICS_ID.into(),path:INDEXED_GRAPHICS_FILE.into(),format:INDEXED_GRAPHICS_FORMAT.into(),sha256:String::new(),dependencies:vec![UNIT_OBJ_FFFF_READER_FIELDS_ID.into()],sources:metadata.sources.clone()});
            result.indexed_graphics = Some(metadata);
            let copy_layout = if revision > PRE_SIGNED_MAP_DELTAS_REVISION {
                ui_raw_transfers::CopyLayout::SignedMapDeltas
            } else if revision > PRE_TILEMAP_RECTANGLES_REVISION {
                ui_raw_transfers::CopyLayout::TilemapRectangles
            } else if revision > PRE_SCANLINE_BLEND_REVISION {
                ui_raw_transfers::CopyLayout::ScanlineBlend
            } else if revision > PRE_COLOR_MODE_PAIRS_REVISION {
                ui_raw_transfers::CopyLayout::ColorModePairs
            } else if revision > PRE_SCROLL_REGISTER_REVISION {
                ui_raw_transfers::CopyLayout::ScrollRegisters
            } else if revision > PRE_STATE_INITIALIZER_REVISION {
                ui_raw_transfers::CopyLayout::StateInitializers
            } else if revision > PRE_ROW_PAIR_REVISION {
                ui_raw_transfers::CopyLayout::RowPairs
            } else if revision > PRE_VRAM_LOADER_REVISION {
                ui_raw_transfers::CopyLayout::VramLoaders
            } else if revision > PRE_HALFWORD_PRELOAD_REVISION {
                ui_raw_transfers::CopyLayout::Preloads
            } else if revision > PRE_HALFWORD_COPY_REVISION {
                ui_raw_transfers::CopyLayout::Biased
            } else {
                ui_raw_transfers::CopyLayout::Legacy
            };
            let metadata = ui_raw_transfers::Catalog::from_rom_with_copy_layout(rom, copy_layout)?;
            result.manifest.coverage.push(UI_RAW_TRANSFERS_ID.into());
            result.manifest.resources.push(Resource {id:UI_RAW_TRANSFERS_ID.into(),path:UI_RAW_TRANSFERS_FILE.into(),format:UI_RAW_TRANSFERS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.ui_raw_transfers = Some(metadata);
            let metadata = ui_raw_transfers::dynamic_fragments::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(DYNAMIC_RAW_FRAGMENTS_ID.into());
            result.manifest.resources.push(Resource {id:DYNAMIC_RAW_FRAGMENTS_ID.into(),path:DYNAMIC_RAW_FRAGMENTS_FILE.into(),format:DYNAMIC_RAW_FRAGMENTS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.dynamic_raw_fragments = Some(metadata);
            let metadata = conditional_four_byte_lookup::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_FOUR_BYTE_LOOKUP_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_FOUR_BYTE_LOOKUP_ID.into(),path:CONDITIONAL_FOUR_BYTE_LOOKUP_FILE.into(),format:CONDITIONAL_FOUR_BYTE_LOOKUP_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_four_byte_lookup = Some(metadata);
            let metadata = conditional_packed_lookup::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_PACKED_LOOKUP_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_PACKED_LOOKUP_ID.into(),path:CONDITIONAL_PACKED_LOOKUP_FILE.into(),format:CONDITIONAL_PACKED_LOOKUP_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_packed_lookup = Some(metadata);
            let metadata = conditional_descriptor_banks::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_DESCRIPTOR_BANKS_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_DESCRIPTOR_BANKS_ID.into(),path:CONDITIONAL_DESCRIPTOR_BANKS_FILE.into(),format:CONDITIONAL_DESCRIPTOR_BANKS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_descriptor_banks = Some(metadata);
            let metadata = conditional_descriptor_bank_zero::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_DESCRIPTOR_BANK_ZERO_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_DESCRIPTOR_BANK_ZERO_ID.into(),path:CONDITIONAL_DESCRIPTOR_BANK_ZERO_FILE.into(),format:CONDITIONAL_DESCRIPTOR_BANK_ZERO_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_descriptor_bank_zero = Some(metadata);
            let metadata = conditional_property_masks::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_PROPERTY_MASKS_ID.into());
            result.manifest.resources.push(Resource {id:CONDITIONAL_PROPERTY_MASKS_ID.into(),path:CONDITIONAL_PROPERTY_MASKS_FILE.into(),format:CONDITIONAL_PROPERTY_MASKS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.conditional_property_masks = Some(metadata);
            let metadata = compact_obj_animations::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:COMPACT_OBJ_ANIMATIONS_ID.into(),path:COMPACT_OBJ_ANIMATIONS_FILE.into(),format:COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_secondary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SECONDARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:SECONDARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:SECONDARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:SECONDARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.secondary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_tertiary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TERTIARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:TERTIARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:TERTIARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:TERTIARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.tertiary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_quaternary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(QUATERNARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:QUATERNARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:QUATERNARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:QUATERNARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.quaternary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_quinary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(QUINARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:QUINARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:QUINARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:QUINARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.quinary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_senary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SENARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:SENARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:SENARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:SENARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.senary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_septenary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(SEPTENARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:SEPTENARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:SEPTENARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:SEPTENARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.septenary_compact_obj_animations = Some(metadata);
            let metadata = compact_obj_animations_octonary::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(OCTONARY_COMPACT_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource {id:OCTONARY_COMPACT_OBJ_ANIMATIONS_ID.into(),path:OCTONARY_COMPACT_OBJ_ANIMATIONS_FILE.into(),format:OCTONARY_COMPACT_OBJ_ANIMATIONS_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.octonary_compact_obj_animations = Some(metadata);
            let metadata = ui_context_templates::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(UI_CONTEXT_TEMPLATES_ID.into());
            result.manifest.resources.push(Resource {id:UI_CONTEXT_TEMPLATES_ID.into(),path:UI_CONTEXT_TEMPLATES_FILE.into(),format:UI_CONTEXT_TEMPLATES_FORMAT.into(),sha256:String::new(),dependencies:vec![],sources:metadata.sources.clone()});
            result.ui_context_templates = Some(metadata);
            let metadata = compact_obj_catalog::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(COMPACT_OBJ_CATALOG_ID.into());
            result.manifest.resources.push(Resource {id:COMPACT_OBJ_CATALOG_ID.into(),path:COMPACT_OBJ_CATALOG_FILE.into(),format:COMPACT_OBJ_CATALOG_FORMAT.into(),sha256:String::new(),dependencies:vec![OBJ_STATIC_LISTS_ID.into()],sources:metadata.sources.clone()});
            result.compact_obj_catalog = Some(metadata);
            let metadata = layered_obj_animations::Catalog::from_rom_with_color_mode(
                rom,
                revision > PRE_TITLE_COLOR_MODE_REVISION,
            )?;
            result.manifest.coverage.push(LAYERED_OBJ_ANIMATIONS_ID.into());
            result.manifest.resources.push(Resource { id: LAYERED_OBJ_ANIMATIONS_ID.into(), path: LAYERED_OBJ_ANIMATIONS_FILE.into(), format: LAYERED_OBJ_ANIMATIONS_FORMAT.into(), sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone() });
            result.layered_obj_animations = Some(metadata);
            let metadata = audio_extended_dispatch::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(AUDIO_EXTENDED_DISPATCH_ID.into());
            result.manifest.resources.push(Resource { id: AUDIO_EXTENDED_DISPATCH_ID.into(), path: AUDIO_EXTENDED_DISPATCH_FILE.into(), format: AUDIO_EXTENDED_DISPATCH_FORMAT.into(), sha256: String::new(), dependencies: vec![AUDIO_SEQUENCES_ID.into()], sources: metadata.sources.clone() });
            result.audio_extended_dispatch = Some(metadata);
            let metadata = conditional_item_parameters::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(CONDITIONAL_ITEM_PARAMETERS_ID.into());
            result.manifest.resources.push(Resource { id: CONDITIONAL_ITEM_PARAMETERS_ID.into(), path: CONDITIONAL_ITEM_PARAMETERS_FILE.into(), format: CONDITIONAL_ITEM_PARAMETERS_FORMAT.into(), sha256: String::new(), dependencies: vec![ITEM_RECORDS_ID.into()], sources: metadata.sources.clone() });
            result.conditional_item_parameters = Some(metadata);
            let metadata = range_lookup::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(RANGE_LOOKUP_ID.into());
            result.manifest.resources.push(Resource { id: RANGE_LOOKUP_ID.into(), path: RANGE_LOOKUP_FILE.into(), format: RANGE_LOOKUP_FORMAT.into(), sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone() });
            result.range_lookup = Some(metadata);
            let metadata = tilemap_geometry::Catalog::from_rom(rom)?;
            result.manifest.coverage.push(TILEMAP_GEOMETRY_ID.into());
            result.manifest.resources.push(Resource { id: TILEMAP_GEOMETRY_ID.into(), path: TILEMAP_GEOMETRY_FILE.into(), format: TILEMAP_GEOMETRY_FORMAT.into(), sha256: String::new(), dependencies: vec![], sources: metadata.sources.clone() });
            result.tilemap_geometry = Some(metadata);
            let metadata = script39_registry_poses::Catalog::from_rom(
                rom, &result.manifest.resources,
                &result.script_archive.as_ref().ok_or("Missing opcode-39 script archive")?.decoded,
                &result.metadata52.as_ref().ok_or("Missing opcode-39 job metadata")?.decoded,
            )?;
            result.manifest.coverage.push(SCRIPT39_REGISTRY_POSES_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT39_REGISTRY_POSES_ID.into(), path: SCRIPT39_REGISTRY_POSES_FILE.into(),
                format: SCRIPT39_REGISTRY_POSES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![METADATA52_ID.into(), SCRIPT_ARCHIVE_ID.into(), UNIT_OBJ_INITIAL_POSE_ID.into(),
                    UNIT_OBJ_EQUIPMENT_ID.into(), UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into(),
                    UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into(), UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID.into(),
                    UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_ID.into()], sources: metadata.sources.clone(),
            });
            result.script39_registry_poses = Some(metadata);
            let metadata = script39_special_poses::Catalog::from_rom(
                rom, &result.manifest.resources,
                &result.script_archive.as_ref().ok_or("Missing opcode-39 script archive")?.decoded,
                &result.metadata14.as_ref().ok_or("Missing opcode-39 special-unit metadata")?.decoded,
            )?;
            result.manifest.coverage.push(SCRIPT39_SPECIAL_POSES_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT39_SPECIAL_POSES_ID.into(), path: SCRIPT39_SPECIAL_POSES_FILE.into(),
                format: SCRIPT39_SPECIAL_POSES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![METADATA14_ID.into(), SCRIPT_ARCHIVE_ID.into(),
                    UNIT_OBJ_INITIAL_POSE_ID.into(), UNIT_OBJ_EQUIPMENT_ID.into(),
                    UNIT_OBJ_NORMAL_BATTLE_ID.into(), UNIT_OBJ_ALTERNATE_BATTLE_ID.into(),
                    UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_ID.into(), UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_ID.into(),
                    UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_ID.into(), SCRIPT39_REGISTRY_POSES_ID.into()],
                sources: metadata.sources.clone(),
            });
            result.script39_special_poses = Some(metadata);
            let metadata = direct_indexed_graphics::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(DIRECT_INDEXED_GRAPHICS_ID.into());
            result.manifest.resources.push(Resource {
                id: DIRECT_INDEXED_GRAPHICS_ID.into(), path: DIRECT_INDEXED_GRAPHICS_FILE.into(),
                format: DIRECT_INDEXED_GRAPHICS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.direct_indexed_graphics = Some(metadata);
            let metadata = direct_indexed_dictionary::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(DIRECT_INDEXED_DICTIONARY_ID.into());
            result.manifest.resources.push(Resource {
                id: DIRECT_INDEXED_DICTIONARY_ID.into(), path: DIRECT_INDEXED_DICTIONARY_FILE.into(),
                format: DIRECT_INDEXED_DICTIONARY_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.direct_indexed_dictionary = Some(metadata);
            let metadata = context_callback_descriptor::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(CONTEXT_CALLBACK_DESCRIPTOR_ID.into());
            result.manifest.resources.push(Resource {
                id: CONTEXT_CALLBACK_DESCRIPTOR_ID.into(), path: CONTEXT_CALLBACK_DESCRIPTOR_FILE.into(),
                format: CONTEXT_CALLBACK_DESCRIPTOR_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.context_callback_descriptor = Some(metadata);
            let metadata = conditional_language_pair::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(CONDITIONAL_LANGUAGE_PAIR_ID.into());
            result.manifest.resources.push(Resource {
                id: CONDITIONAL_LANGUAGE_PAIR_ID.into(), path: CONDITIONAL_LANGUAGE_PAIR_FILE.into(),
                format: CONDITIONAL_LANGUAGE_PAIR_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.conditional_language_pair = Some(metadata);
            let metadata = text_reader_registry::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(TEXT_READER_REGISTRY_ID.into());
            result.manifest.resources.push(Resource {
                id: TEXT_READER_REGISTRY_ID.into(), path: TEXT_READER_REGISTRY_FILE.into(),
                format: TEXT_READER_REGISTRY_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.text_reader_registry = Some(metadata);
            let metadata = conditional_substitution_words::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(CONDITIONAL_SUBSTITUTION_WORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: CONDITIONAL_SUBSTITUTION_WORDS_ID.into(), path: CONDITIONAL_SUBSTITUTION_WORDS_FILE.into(),
                format: CONDITIONAL_SUBSTITUTION_WORDS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![TEXT_CATALOGS_ID.into()], sources: metadata.sources.clone(),
            });
            result.conditional_substitution_words = Some(metadata);
            let metadata = display_control_tables::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(DISPLAY_CONTROL_TABLES_ID.into());
            result.manifest.resources.push(Resource {
                id: DISPLAY_CONTROL_TABLES_ID.into(), path: DISPLAY_CONTROL_TABLES_FILE.into(),
                format: DISPLAY_CONTROL_TABLES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.display_control_tables = Some(metadata);
            let metadata = compact_primary_alias::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(COMPACT_PRIMARY_ALIAS_ID.into());
            result.manifest.resources.push(Resource {
                id: COMPACT_PRIMARY_ALIAS_ID.into(), path: COMPACT_PRIMARY_ALIAS_FILE.into(),
                format: COMPACT_PRIMARY_ALIAS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![TASK_DESCRIPTORS_ID.into(), COMPACT_OBJ_CATALOG_ID.into()], sources: metadata.sources.clone(),
            });
            result.compact_primary_alias = Some(metadata);
            let metadata = observed_lz::Catalog::from_rom_with_vram_bank(
                rom,
                &result.manifest.resources,
                revision > PRE_VRAM_LZ_BANK_REVISION,
            )?;
            result.manifest.coverage.push(OBSERVED_LZ_ID.into());
            result.manifest.resources.push(Resource {
                id: OBSERVED_LZ_ID.into(), path: OBSERVED_LZ_FILE.into(),
                format: OBSERVED_LZ_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.observed_lz = Some(metadata);
            let metadata = constructor_template64::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(CONSTRUCTOR_TEMPLATE64_ID.into());
            result.manifest.resources.push(Resource {
                id: CONSTRUCTOR_TEMPLATE64_ID.into(), path: CONSTRUCTOR_TEMPLATE64_FILE.into(),
                format: CONSTRUCTOR_TEMPLATE64_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.constructor_template64 = Some(metadata);
            let metadata = observed_tilemap::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(OBSERVED_TILEMAP_ID.into());
            result.manifest.resources.push(Resource {
                id: OBSERVED_TILEMAP_ID.into(), path: OBSERVED_TILEMAP_FILE.into(),
                format: OBSERVED_TILEMAP_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.observed_tilemap = Some(metadata);
            let metadata = observed_tilemap_descriptor::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(OBSERVED_TILEMAP_DESCRIPTOR_ID.into());
            result.manifest.resources.push(Resource {
                id: OBSERVED_TILEMAP_DESCRIPTOR_ID.into(), path: OBSERVED_TILEMAP_DESCRIPTOR_FILE.into(),
                format: OBSERVED_TILEMAP_DESCRIPTOR_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBSERVED_TILEMAP_ID.into(), OBSERVED_LZ_ID.into()], sources: metadata.sources.clone(),
            });
            result.observed_tilemap_descriptor = Some(metadata);
            let metadata = observed_tilemap_row::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(OBSERVED_TILEMAP_ROW_ID.into());
            result.manifest.resources.push(Resource {
                id: OBSERVED_TILEMAP_ROW_ID.into(), path: OBSERVED_TILEMAP_ROW_FILE.into(),
                format: OBSERVED_TILEMAP_ROW_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBSERVED_TILEMAP_DESCRIPTOR_ID.into(), OBSERVED_TILEMAP_ID.into()], sources: metadata.sources.clone(),
            });
            result.observed_tilemap_row = Some(metadata);
            let metadata = dma_register_directories::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(DMA_REGISTER_DIRECTORIES_ID.into());
            result.manifest.resources.push(Resource {
                id: DMA_REGISTER_DIRECTORIES_ID.into(), path: DMA_REGISTER_DIRECTORIES_FILE.into(),
                format: DMA_REGISTER_DIRECTORIES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.dma_register_directories = Some(metadata);
            let metadata = iwram_memory_helpers::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(IWRAM_MEMORY_HELPERS_ID.into());
            result.manifest.resources.push(Resource {
                id: IWRAM_MEMORY_HELPERS_ID.into(), path: IWRAM_MEMORY_HELPERS_FILE.into(),
                format: IWRAM_MEMORY_HELPERS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.iwram_memory_helpers = Some(metadata);
            let indexed_exchange = revision > PRE_INDEXED_EXCHANGE_REVISION;
            let five_byte_list = revision > PRE_FIVE_BYTE_LIST_REVISION;
            let layout = if five_byte_list {
                adjacent_offset_tables::SourceLayout::FiveByteList
            } else if indexed_exchange {
                adjacent_offset_tables::SourceLayout::IndexedExchange
            } else {
                adjacent_offset_tables::SourceLayout::Legacy
            };
            let metadata = adjacent_offset_tables::Catalog::from_rom_with_layout(
                rom, &result.manifest.resources, layout,
            )?;
            result.manifest.coverage.push(ADJACENT_OFFSET_TABLES_ID.into());
            result.manifest.resources.push(Resource {
                id: ADJACENT_OFFSET_TABLES_ID.into(), path: ADJACENT_OFFSET_TABLES_FILE.into(),
                format: ADJACENT_OFFSET_TABLES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.adjacent_offset_tables = Some(metadata);
            let metadata = indexed_records16::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(INDEXED_RECORDS16_ID.into());
            result.manifest.resources.push(Resource {
                id: INDEXED_RECORDS16_ID.into(), path: INDEXED_RECORDS16_FILE.into(),
                format: INDEXED_RECORDS16_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.indexed_records16 = Some(metadata);
            let metadata = indexed_words_3959::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(INDEXED_WORDS_3959_ID.into());
            result.manifest.resources.push(Resource {
                id: INDEXED_WORDS_3959_ID.into(), path: INDEXED_WORDS_3959_FILE.into(),
                format: INDEXED_WORDS_3959_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.indexed_words_3959 = Some(metadata);
            let title_frontend = revision > PRE_TITLE_FRONTEND_REVISION;
            let world_node_renderer = revision > PRE_WORLD_NODE_RENDERER_REVISION;
            let helper_field_dispatch = revision > PRE_HELPER_FIELD_DISPATCH_REVISION;
            let cached_object = revision > PRE_CACHED_OBJECT_REVISION;
            let background_variant_state = revision > PRE_BACKGROUND_VARIANT_STATE_REVISION;
            let layout = if background_variant_state {
                thumb_switch_tables::SourceLayout::BackgroundVariantState
            } else if five_byte_list {
                thumb_switch_tables::SourceLayout::FiveByteList
            } else if indexed_exchange {
                thumb_switch_tables::SourceLayout::IndexedExchange
            } else if cached_object {
                thumb_switch_tables::SourceLayout::CachedObject
            } else if helper_field_dispatch {
                thumb_switch_tables::SourceLayout::HelperFieldDispatch
            } else if world_node_renderer {
                thumb_switch_tables::SourceLayout::WorldNodeRenderer
            } else if title_frontend {
                thumb_switch_tables::SourceLayout::TitleFrontend
            } else {
                thumb_switch_tables::SourceLayout::Legacy
            };
            let metadata = thumb_switch_tables::Catalog::from_rom_with_layout(
                rom, &result.manifest.resources, layout,
            )?;
            result.manifest.coverage.push(THUMB_SWITCH_TABLES_ID.into());
            result.manifest.resources.push(Resource {
                id: THUMB_SWITCH_TABLES_ID.into(), path: THUMB_SWITCH_TABLES_FILE.into(),
                format: THUMB_SWITCH_TABLES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.thumb_switch_tables = Some(metadata);
            let layout = if revision > PRE_CPU_FAST_SET_LITERALS_REVISION {
                thumb_pc_literals::SourceLayout::CpuFastSet
            } else if background_variant_state {
                thumb_pc_literals::SourceLayout::BackgroundVariantState
            } else if revision > PRE_PALETTE_TASK_REVISION {
                thumb_pc_literals::SourceLayout::PaletteTask
            } else if five_byte_list {
                thumb_pc_literals::SourceLayout::FiveByteList
            } else if indexed_exchange {
                thumb_pc_literals::SourceLayout::IndexedExchange
            } else if cached_object {
                thumb_pc_literals::SourceLayout::CachedObject
            } else if helper_field_dispatch {
                thumb_pc_literals::SourceLayout::HelperFieldDispatch
            } else if revision > PRE_WORLD_STATE_TASK_REVISION {
                thumb_pc_literals::SourceLayout::WorldStateTask
            } else if world_node_renderer {
                thumb_pc_literals::SourceLayout::WorldNodeRenderer
            } else if title_frontend {
                thumb_pc_literals::SourceLayout::TitleFrontend
            } else if revision > PRE_TITLE_CALLBACKS_REVISION {
                thumb_pc_literals::SourceLayout::TitleTaskCallbacks
            } else if revision > PRE_STARTUP_DISPATCHER_REVISION {
                thumb_pc_literals::SourceLayout::StartupTaskDispatcher
            } else if revision > PRE_STARTUP_TASK_ROOTS_REVISION {
                thumb_pc_literals::SourceLayout::StartupTaskRoots
            } else if revision > PRE_LOCALIZED_TEXT_ROOTS_REVISION {
                thumb_pc_literals::SourceLayout::LocalizedTextRoots
            } else if revision > PRE_CUSTOM_DECODER_ROOTS_REVISION {
                thumb_pc_literals::SourceLayout::CustomDecoderRoots
            } else {
                thumb_pc_literals::SourceLayout::Legacy
            };
            let metadata = thumb_pc_literals::Catalog::from_rom_with_layout(rom, &result.manifest.resources, layout)?;
            result.manifest.coverage.push(THUMB_PC_LITERALS_ID.into());
            result.manifest.resources.push(Resource {
                id: THUMB_PC_LITERALS_ID.into(), path: THUMB_PC_LITERALS_FILE.into(),
                format: THUMB_PC_LITERALS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.thumb_pc_literals = Some(metadata);
            let fixed_bytes = if revision > PRE_SCRATCH_RESTORE_LITERALS_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::WithScratchRestoreLiterals)
            } else if revision > PRE_CONTEXT_RECORD_LITERALS_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::WithContextRecordLiterals)
            } else if revision > PRE_SHARED_SLOT_LITERALS_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::WithSharedSlotLiterals)
            } else if revision > PRE_COPY_PRODUCER_LITERALS_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::WithProducerLiterals)
            } else if revision > PRE_DIRECT_COPY_ARGUMENTS_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::WithDirectArguments)
            } else if revision > PRE_FIXED_BYTE_COPIES_REVISION {
                Some(copied_halfword_lookups::fixed_byte_copies::Layout::FixedSize)
            } else {
                None
            };
            let metadata = copied_halfword_lookups::Catalog::from_rom_with_initializers(
                rom, &result.manifest.resources, revision > PRE_INITIALIZER_BANK_REVISION,
                revision > PRE_INITIALIZER_VARIANTS_REVISION, revision > PRE_INITIALIZER_PAIRS_REVISION,
                revision > PRE_STACK_BYTE_COPIES_REVISION, fixed_bytes,
            )?;
            result.manifest.coverage.push(COPIED_HALFWORD_LOOKUPS_ID.into());
            result.manifest.resources.push(Resource {
                id: COPIED_HALFWORD_LOOKUPS_ID.into(), path: COPIED_HALFWORD_LOOKUPS_FILE.into(),
                format: COPIED_HALFWORD_LOOKUPS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.copied_halfword_lookups = Some(metadata);
            let metadata = indexed_halfwords_39a5::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(INDEXED_HALFWORDS_39A5_ID.into());
            result.manifest.resources.push(Resource {
                id: INDEXED_HALFWORDS_39A5_ID.into(), path: INDEXED_HALFWORDS_39A5_FILE.into(),
                format: INDEXED_HALFWORDS_39A5_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.indexed_halfwords_39a5 = Some(metadata);
            let metadata = script_byte_lookup_3ad::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SCRIPT_BYTE_LOOKUP_3AD_ID.into());
            result.manifest.resources.push(Resource {
                id: SCRIPT_BYTE_LOOKUP_3AD_ID.into(), path: SCRIPT_BYTE_LOOKUP_3AD_FILE.into(),
                format: SCRIPT_BYTE_LOOKUP_3AD_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.script_byte_lookup_3ad = Some(metadata);
            let metadata = soil_scene_records::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_SCENE_RECORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_SCENE_RECORDS_ID.into(), path: SOIL_SCENE_RECORDS_FILE.into(),
                format: SOIL_SCENE_RECORDS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_scene_records = Some(metadata);
            let metadata = soil_indirect_records::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_INDIRECT_RECORDS_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_INDIRECT_RECORDS_ID.into(), path: SOIL_INDIRECT_RECORDS_FILE.into(),
                format: SOIL_INDIRECT_RECORDS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_indirect_records = Some(metadata);
            let metadata = soil_indexed_bytes::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_INDEXED_BYTES_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_INDEXED_BYTES_ID.into(), path: SOIL_INDEXED_BYTES_FILE.into(),
                format: SOIL_INDEXED_BYTES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_indexed_bytes = Some(metadata);
            let metadata = soil_compact_record::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_COMPACT_RECORD_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_COMPACT_RECORD_ID.into(), path: SOIL_COMPACT_RECORD_FILE.into(),
                format: SOIL_COMPACT_RECORD_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into(), COMPACT_OBJ_CATALOG_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_compact_record = Some(metadata);
            let metadata = soil_scoped_values::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_SCOPED_VALUES_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_SCOPED_VALUES_ID.into(), path: SOIL_SCOPED_VALUES_FILE.into(),
                format: SOIL_SCOPED_VALUES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_scoped_values = Some(metadata);
            let metadata = soil_remaining_rectangles::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(SOIL_REMAINING_RECTANGLES_ID.into());
            result.manifest.resources.push(Resource {
                id: SOIL_REMAINING_RECTANGLES_ID.into(), path: SOIL_REMAINING_RECTANGLES_FILE.into(),
                format: SOIL_REMAINING_RECTANGLES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.soil_remaining_rectangles = Some(metadata);
            let metadata = firaga_copied_matrices::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_COPIED_MATRICES_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_COPIED_MATRICES_ID.into(), path: FIRAGA_COPIED_MATRICES_FILE.into(),
                format: FIRAGA_COPIED_MATRICES_FORMAT.into(), sha256: String::new(),
                dependencies: Vec::new(), sources: metadata.sources.clone(),
            });
            result.firaga_copied_matrices = Some(metadata);
            let metadata = firaga_compact_cells::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_COMPACT_CELLS_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_COMPACT_CELLS_ID.into(), path: FIRAGA_COMPACT_CELLS_FILE.into(),
                format: FIRAGA_COMPACT_CELLS_FORMAT.into(), sha256: String::new(),
                dependencies: Vec::new(), sources: metadata.sources.clone(),
            });
            result.firaga_compact_cells = Some(metadata);
            let metadata = firaga_indexed_obj_bank::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_INDEXED_OBJ_BANK_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_INDEXED_OBJ_BANK_ID.into(), path: FIRAGA_INDEXED_OBJ_BANK_FILE.into(),
                format: FIRAGA_INDEXED_OBJ_BANK_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBJ_STATIC_LISTS_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_indexed_obj_bank = Some(metadata);
            let metadata = firaga_compact_obj_bank::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_COMPACT_OBJ_BANK_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_COMPACT_OBJ_BANK_ID.into(), path: FIRAGA_COMPACT_OBJ_BANK_FILE.into(),
                format: FIRAGA_COMPACT_OBJ_BANK_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBJ_STATIC_LISTS_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_compact_obj_bank = Some(metadata);
            let metadata = firaga_adjacent_compact_obj_bank::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_ADJACENT_COMPACT_OBJ_BANK_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_ADJACENT_COMPACT_OBJ_BANK_ID.into(), path: FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FILE.into(),
                format: FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBJ_STATIC_LISTS_ID.into(), FIRAGA_COMPACT_OBJ_BANK_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_adjacent_compact_obj_bank = Some(metadata);
            let metadata = firaga_6c12c8_obj_bank::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_6C12C8_OBJ_BANK_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_6C12C8_OBJ_BANK_ID.into(), path: FIRAGA_6C12C8_OBJ_BANK_FILE.into(),
                format: FIRAGA_6C12C8_OBJ_BANK_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBJ_STATIC_LISTS_ID.into(), THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_6c12c8_obj_bank = Some(metadata);
            let metadata = firaga_transfer_program::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_TRANSFER_PROGRAM_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_TRANSFER_PROGRAM_ID.into(), path: FIRAGA_TRANSFER_PROGRAM_FILE.into(),
                format: FIRAGA_TRANSFER_PROGRAM_FORMAT.into(), sha256: String::new(),
                dependencies: vec![OBSERVED_LZ_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_transfer_program = Some(metadata);
            let metadata = firaga_palette_rows::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_PALETTE_ROWS_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_PALETTE_ROWS_ID.into(), path: FIRAGA_PALETTE_ROWS_FILE.into(),
                format: FIRAGA_PALETTE_ROWS_FORMAT.into(), sha256: String::new(),
                dependencies: vec![FIRAGA_TRANSFER_PROGRAM_ID.into()], sources: metadata.sources.clone(),
            });
            result.firaga_palette_rows = Some(metadata);
            let metadata = firaga_stack_initialization::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_STACK_INITIALIZATION_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_STACK_INITIALIZATION_ID.into(), path: FIRAGA_STACK_INITIALIZATION_FILE.into(),
                format: FIRAGA_STACK_INITIALIZATION_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.firaga_stack_initialization = Some(metadata);
            let metadata = firaga_queued_transfer::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(FIRAGA_QUEUED_TRANSFER_ID.into());
            result.manifest.resources.push(Resource {
                id: FIRAGA_QUEUED_TRANSFER_ID.into(), path: FIRAGA_QUEUED_TRANSFER_FILE.into(),
                format: FIRAGA_QUEUED_TRANSFER_FORMAT.into(), sha256: String::new(),
                dependencies: vec![], sources: metadata.sources.clone(),
            });
            result.firaga_queued_transfer = Some(metadata);
            let metadata = white_flowers_signed_bytes::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(WHITE_FLOWERS_SIGNED_BYTES_ID.into());
            result.manifest.resources.push(Resource {
                id: WHITE_FLOWERS_SIGNED_BYTES_ID.into(), path: WHITE_FLOWERS_SIGNED_BYTES_FILE.into(),
                format: WHITE_FLOWERS_SIGNED_BYTES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.white_flowers_signed_bytes = Some(metadata);
            let metadata = white_flowers_isolated_values::Catalog::from_rom(rom, &result.manifest.resources)?;
            result.manifest.coverage.push(WHITE_FLOWERS_ISOLATED_VALUES_ID.into());
            result.manifest.resources.push(Resource {
                id: WHITE_FLOWERS_ISOLATED_VALUES_ID.into(), path: WHITE_FLOWERS_ISOLATED_VALUES_FILE.into(),
                format: WHITE_FLOWERS_ISOLATED_VALUES_FORMAT.into(), sha256: String::new(),
                dependencies: vec![THUMB_PC_LITERALS_ID.into()], sources: metadata.sources.clone(),
            });
            result.white_flowers_isolated_values = Some(metadata);
            let metadata = tilemap_transfer_directory::Catalog::from_verified_rom(
                rom,
                &result.manifest.resources,
            )?;
            result.manifest.coverage.push(TILEMAP_TRANSFER_DIRECTORY_ID.into());
            result.manifest.resources.push(Resource {
                id: TILEMAP_TRANSFER_DIRECTORY_ID.into(),
                path: TILEMAP_TRANSFER_DIRECTORY_FILE.into(),
                format: TILEMAP_TRANSFER_DIRECTORY_FORMAT.into(),
                sha256: String::new(),
                dependencies: [
                    THUMB_PC_LITERALS_ID,
                    SOIL_SCOPED_VALUES_ID,
                    OBSERVED_TILEMAP_ROW_ID,
                    OBSERVED_TILEMAP_DESCRIPTOR_ID,
                ].into_iter().map(str::to_owned).collect(),
                sources: metadata.sources.clone(),
            });
            let streams = if revision != PRE_INDEXED_TILEMAP_REVISION {
                let streams = tilemap_stream_bank::from_verified_rom(rom)?;
                let controls = tilemap_stream_bank::IndexedTilemapControls::from_verified_rom(rom, &streams)?;
                let prior = script39_registry_poses::PriorSources::new(&result.manifest.resources)?;
                let sources = controls.uncovered_fields(&prior)?.iter()
                    .map(|field| field.source_range())
                    .collect();
                result.manifest.coverage.push(INDEXED_TILEMAP_CONTROLS_ID.into());
                result.manifest.resources.push(Resource {
                    id: INDEXED_TILEMAP_CONTROLS_ID.into(),
                    path: INDEXED_TILEMAP_CONTROLS_FILE.into(),
                    format: INDEXED_TILEMAP_CONTROLS_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [
                        OBSERVED_TILEMAP_ID,
                        OBSERVED_TILEMAP_ROW_ID,
                        OBSERVED_TILEMAP_DESCRIPTOR_ID,
                    ].into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.indexed_tilemap_controls = Some(controls);
                if revision != PRE_TILEMAP_STREAM_FILE_REVISION {
                    let ranges = tilemap_stream_bank::uncovered_ranges(&streams, &prior)?;
                    let (encoded, sources) = pack_encoded_sources(rom, ranges);
                    result.manifest.coverage.push(TILEMAP_STREAM_BANK_ID.into());
                    result.manifest.resources.push(Resource {
                        id: TILEMAP_STREAM_BANK_ID.into(),
                        path: TILEMAP_STREAM_BANK_FILE.into(),
                        format: TILEMAP_STREAM_BANK_FORMAT.into(),
                        sha256: String::new(),
                        dependencies: vec![OBSERVED_TILEMAP_ID.into()],
                        sources,
                    });
                    result.tilemap_stream_bank = Some(encoded);
                }
                Some(streams)
            } else {
                None
            };
            if revision >= PRE_TILEMAP_TYPE3_REVISION {
                let prior = script39_registry_poses::PriorSources::new(&result.manifest.resources)?;
                let ranges = metadata.decoded.uncovered_lz77_inputs(rom, &prior)?;
                let (encoded, sources) = pack_encoded_sources(
                    rom, ranges.into_iter().map(|range| (range, "lz77-input")),
                );
                result.manifest.coverage.push(TILEMAP_TRANSFER_LZ77_ID.into());
                result.manifest.resources.push(Resource {
                    id: TILEMAP_TRANSFER_LZ77_ID.into(),
                    path: TILEMAP_TRANSFER_LZ77_FILE.into(),
                    format: TILEMAP_TRANSFER_LZ77_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: vec![TILEMAP_TRANSFER_DIRECTORY_ID.into(), OBSERVED_LZ_ID.into()],
                    sources,
                });
                result.tilemap_transfer_lz77 = Some(encoded);
            }
            let recovered_tilemap_lists = revision > PRE_RECOVERED_TILEMAP_LISTS_REVISION;
            if revision >= PRE_TILEMAP_TYPE2_REVISION {
                let ranges = tilemap_type3::uncovered_ranges(
                    rom, &result.manifest.resources, recovered_tilemap_lists,
                )?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(TILEMAP_TYPE3_ID.into());
                result.manifest.resources.push(Resource {
                    id: TILEMAP_TYPE3_ID.into(),
                    path: TILEMAP_TYPE3_FILE.into(),
                    format: TILEMAP_TYPE3_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [
                        OBSERVED_TILEMAP_ROW_ID, OBSERVED_TILEMAP_DESCRIPTOR_ID, OBSERVED_LZ_ID,
                        FIRAGA_TRANSFER_PROGRAM_ID, COMPACT_PRIMARY_ALIAS_ID,
                        TILEMAP_TRANSFER_DIRECTORY_ID, TILEMAP_TRANSFER_LZ77_ID,
                    ].into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.tilemap_type3 = Some(encoded);
            }
            if revision >= PRE_TILEMAP_DIRECT_REVISION {
                let firaga = &result.firaga_transfer_program.as_ref().unwrap().decoded;
                let ranges = tilemap_type2::uncovered_ranges(rom, firaga, &result.manifest.resources)?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(TILEMAP_TYPE2_ID.into());
                result.manifest.resources.push(Resource {
                    id: TILEMAP_TYPE2_ID.into(),
                    path: TILEMAP_TYPE2_FILE.into(),
                    format: TILEMAP_TYPE2_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [
                        OBSERVED_TILEMAP_ID, OBSERVED_TILEMAP_ROW_ID,
                        FIRAGA_TRANSFER_PROGRAM_ID, FIRAGA_PALETTE_ROWS_ID,
                    ].into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.tilemap_type2 = Some(encoded);
            }
            if revision >= PRE_TILEMAP_ORIGIN_REVISION {
                let streams = streams.as_deref().ok_or("Missing direct tilemap streams")?;
                let mut dependencies = vec![
                    OBSERVED_TILEMAP_ROW_ID, OBSERVED_TILEMAP_DESCRIPTOR_ID,
                    COMPACT_PRIMARY_ALIAS_ID, INDEXED_TILEMAP_CONTROLS_ID, TILEMAP_STREAM_BANK_ID,
                ];
                let origin_controls = if revision > PRE_TILEMAP_ORIGIN_REVISION {
                    dependencies.push(THUMB_PC_LITERALS_ID);
                    Some(result.indexed_tilemap_controls.as_ref().ok_or("Missing tilemap origin controls")?)
                } else {
                    None
                };
                let mut ranges = tilemap_stream_bank::uncovered_direct_transfer_ranges(
                    rom, streams, &result.manifest.resources, origin_controls,
                )?;
                if revision > PRE_PROJECTED_TILEMAP_DIRECTORIES_REVISION {
                    ranges.extend(tilemap_transfer_directory::uncovered_projected_controls(
                        rom, &result.manifest.resources, recovered_tilemap_lists,
                    )?);
                    ranges.sort_unstable_by_key(|(span, _)| span.start);
                    if ranges.windows(2).any(|pair| pair[0].0.end > pair[1].0.start) {
                        return Err("Projected tilemap controls overlap the direct transfer sources".into());
                    }
                    dependencies.extend([
                        TILEMAP_TRANSFER_DIRECTORY_ID, SOIL_SCOPED_VALUES_ID, FIRAGA_TRANSFER_PROGRAM_ID,
                    ]);
                }
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(TILEMAP_DIRECT_ID.into());
                result.manifest.resources.push(Resource {
                    id: TILEMAP_DIRECT_ID.into(),
                    path: TILEMAP_DIRECT_FILE.into(),
                    format: TILEMAP_DIRECT_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: dependencies.into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.tilemap_direct = Some(encoded);
            }
            if revision > PRE_COMPACT_CELL_BANK_REVISION {
                let ranges = compact_cell_bank::uncovered_ranges(rom, &result.manifest.resources)?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(COMPACT_CELL_BANK_ID.into());
                result.manifest.resources.push(Resource {
                    id: COMPACT_CELL_BANK_ID.into(),
                    path: COMPACT_CELL_BANK_FILE.into(),
                    format: COMPACT_CELL_BANK_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: Vec::new(),
                    sources,
                });
                result.compact_cell_bank = Some(encoded);
            }
            if revision > PRE_BACKGROUND_EFFECT_GRAPHICS_REVISION {
                let ranges = background_effect_graphics::source_ranges(
                    rom, &result.manifest.resources,
                )?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(BACKGROUND_EFFECT_GRAPHICS_ID.into());
                result.manifest.resources.push(Resource {
                    id: BACKGROUND_EFFECT_GRAPHICS_ID.into(),
                    path: BACKGROUND_EFFECT_GRAPHICS_FILE.into(),
                    format: BACKGROUND_EFFECT_GRAPHICS_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: Vec::new(),
                    sources,
                });
                result.background_effect_graphics = Some(encoded);
            }
            if revision > PRE_LOCALIZED_OBJ_ANIMATION_BANKS_REVISION {
                let ranges = localized_obj_animation_banks::source_ranges(
                    rom, &result.manifest.resources,
                )?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(LOCALIZED_OBJ_ANIMATION_BANKS_ID.into());
                result.manifest.resources.push(Resource {
                    id: LOCALIZED_OBJ_ANIMATION_BANKS_ID.into(),
                    path: LOCALIZED_OBJ_ANIMATION_BANKS_FILE.into(),
                    format: LOCALIZED_OBJ_ANIMATION_BANKS_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [
                        OBJ_STATIC_LISTS_ID,
                        FIRAGA_INDEXED_OBJ_BANK_ID,
                        THUMB_PC_LITERALS_ID,
                        UI_CUSTOM_GRAPHICS_ID,
                        INDEXED_GRAPHICS_ID,
                    ].into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.localized_obj_animation_banks = Some(encoded);
            }
            if revision > PRE_BACKGROUND_SCREEN_BANKS_REVISION {
                let ranges = background_screen_banks::source_ranges(rom, &result.manifest.resources)?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(BACKGROUND_SCREEN_BANKS_ID.into());
                result.manifest.resources.push(Resource {
                    id: BACKGROUND_SCREEN_BANKS_ID.into(),
                    path: BACKGROUND_SCREEN_BANKS_FILE.into(),
                    format: BACKGROUND_SCREEN_BANKS_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: Vec::new(),
                    sources,
                });
                result.background_screen_banks = Some(encoded);
            }
            if revision > PRE_PHYSICAL_AUDIO_BANKS_REVISION {
                let ranges = physical_audio_banks::uncovered_ranges(rom, &result.manifest.resources)?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(PHYSICAL_AUDIO_BANKS_ID.into());
                result.manifest.resources.push(Resource {
                    id: PHYSICAL_AUDIO_BANKS_ID.into(),
                    path: PHYSICAL_AUDIO_BANKS_FILE.into(),
                    format: PHYSICAL_AUDIO_BANKS_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [AUDIO_DIRECTORY_ID, AUDIO_TONES_ID]
                        .into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.physical_audio_banks = Some(encoded);
            }
            if revision > PRE_PHYSICAL_COMPACT_OBJ_REGION_REVISION {
                let ranges = physical_compact_obj_region::uncovered_ranges(
                    rom, &result.manifest.resources,
                )?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(PHYSICAL_COMPACT_OBJ_REGION_ID.into());
                result.manifest.resources.push(Resource {
                    id: PHYSICAL_COMPACT_OBJ_REGION_ID.into(),
                    path: PHYSICAL_COMPACT_OBJ_REGION_FILE.into(),
                    format: PHYSICAL_COMPACT_OBJ_REGION_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: [
                        OBJ_STATIC_LISTS_ID,
                        COMPACT_OBJ_ANIMATIONS_ID,
                        SECONDARY_COMPACT_OBJ_ANIMATIONS_ID,
                        TERTIARY_COMPACT_OBJ_ANIMATIONS_ID,
                        QUATERNARY_COMPACT_OBJ_ANIMATIONS_ID,
                        QUINARY_COMPACT_OBJ_ANIMATIONS_ID,
                        SENARY_COMPACT_OBJ_ANIMATIONS_ID,
                        SEPTENARY_COMPACT_OBJ_ANIMATIONS_ID,
                        OCTONARY_COMPACT_OBJ_ANIMATIONS_ID,
                        COMPACT_OBJ_CATALOG_ID,
                        FIRAGA_COMPACT_OBJ_BANK_ID,
                        FIRAGA_ADJACENT_COMPACT_OBJ_BANK_ID,
                        FIRAGA_6C12C8_OBJ_BANK_ID,
                        THUMB_PC_LITERALS_ID,
                    ].into_iter().map(str::to_owned).collect(),
                    sources,
                });
                result.physical_compact_obj_region = Some(encoded);
            }
            if revision > PRE_SCREEN_UPDATE_TABLES_REVISION {
                let (encoded, sources) = pack_encoded_sources(rom, screen_update_tables::SOURCES);
                result.manifest.coverage.push(SCREEN_UPDATE_TABLES_ID.into());
                result.manifest.resources.push(Resource {
                    id: SCREEN_UPDATE_TABLES_ID.into(),
                    path: SCREEN_UPDATE_TABLES_FILE.into(),
                    format: SCREEN_UPDATE_TABLES_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: vec![COMPACT_OBJ_CATALOG_ID.into()],
                    sources,
                });
                result.screen_update_tables = Some(encoded);
            }
            if revision > PRE_COUNTED_EFFECT_TABLES_REVISION {
                let ranges = counted_effect_tables::source_ranges(rom)?;
                let (encoded, sources) = pack_encoded_sources(rom, ranges);
                result.manifest.coverage.push(COUNTED_EFFECT_TABLES_ID.into());
                result.manifest.resources.push(Resource {
                    id: COUNTED_EFFECT_TABLES_ID.into(),
                    path: COUNTED_EFFECT_TABLES_FILE.into(),
                    format: COUNTED_EFFECT_TABLES_FORMAT.into(),
                    sha256: String::new(),
                    dependencies: Vec::new(),
                    sources,
                });
                result.counted_effect_tables = Some(encoded);
            }
            result.tilemap_transfer_directory = Some(metadata);
            (result.manifest.extractor_revision, result.manifest.decoder_revision) = revision;
        } else {
            result.manifest.extractor_revision = 8;
            result.manifest.decoder_revision = 7;
        }
        Ok(result)
    }

    fn from_ranges(
        rom: &[u8],
        hash: String,
        profile: &str,
        ranges: &[(u32, &str)],
        expected: &str,
    ) -> Result<Self, String> {
        let mut sources = Vec::new();
        let mut dimensions = None;
        for &(offset, role) in ranges {
            let start = offset as usize;
            let bytes = rom
                .get(start..start.checked_add(48).ok_or("Source range overflow")?)
                .ok_or("Truncated OBJ dimensions source range")?;
            let sha256 = ffta_rom::digest(bytes);
            if sha256 != expected {
                return Err(format!("OBJ dimensions source mismatch at {offset:#x}"));
            }
            let decoded = ObjDimensions::decode(bytes)?;
            if dimensions.as_ref().is_some_and(|d| d != &decoded) {
                return Err("OBJ dimensions source aliases disagree".into());
            }
            dimensions = Some(decoded);
            sources.push(SourceRange {
                offset,
                length: 48,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self {
            dimensions: dimensions.ok_or("Missing OBJ dimensions source")?,
            background: None,
            palettes: None,
            alternate_palettes: None,
            packet_programs: None,
            main_graphics: None,
            transfers: None,
            short_texts: None,
            text_catalogs: None,
            name_keyboard: None,
            obj_lists: None,
            obj_animations: None,
            obj_graphics: None,
            ui_palettes: None,
            ui_graphics: None,
            items: None,
            item_links: None,
            metadata16: None,
            metadata52: None,
            metadata4: None,
            metadata2: None,
            default48: None,
            constructor_alternates: None,
            metadata14: None,
            metadata12: None,
            global_pair_flags: None,
            evaluation_parameters: None,
            progression_rewards: None,
            progression_gates: None,
            progression_selection_gates: None,
            aggregate_flag_keys: None,
            trade_parameters: None,
            item_effect_parameters: None,
            audio_directory: None,
            audio_tones: None,
            audio_sequences: None,
            audio_driver: None,
            palette_effect_dispatch: None,
            palette_programs: None,
            blend_programs: None,
            task_descriptors: None,
            effect_event_directories: None,
            script_commands: None,
            script_archive: None,
            script_text: None,
            script_coordinates: None,
            ui_choice_lists: None,
            unit_obj_initial_pose: None,
            color_mode_palettes: None,
            unit_obj_equipment: None,
            unit_obj_normal_battle: None,
            unit_obj_alternate_battle: None,
            unit_obj_script_special_normal: None,
            unit_obj_script_special_alternate: None,
            unit_obj_script_override: None,
            unit_obj_script_registry_special: None,
            unit_obj_script_registry_generic: None,
            unit_obj_property_words: None,
            unit_obj_byte_property_words: None,
            unit_obj_ffff_reader_fields: None,
            slot_job_lookup: None,
            observed_task_parameters: None,
            constructor_obj_animation: None,
            task_cycle_deltas: None,
            word_task_obj_animations: None,
            world_nodes: None,
            world_roads: None,
            encounter_headers: None,
            encounter_unit_templates: None,
            encounter_coordinates: None,
            metadata28: None,
            records4: None,
            conditional_records8: None,
            mission_records70: None,
            selection_byte_tables: None,
            event_records12: None,
            parameter_records27: None,
            world_records38: None,
            indexed_graphics: None,
            ui_raw_transfers: None,
            dynamic_raw_fragments: None,
            conditional_four_byte_lookup: None,
            conditional_packed_lookup: None,
            conditional_descriptor_banks: None,
            conditional_descriptor_bank_zero: None,
            conditional_property_masks: None,
            compact_obj_animations: None,
            secondary_compact_obj_animations: None,
            tertiary_compact_obj_animations: None,
            quaternary_compact_obj_animations: None,
            quinary_compact_obj_animations: None,
            senary_compact_obj_animations: None,
            septenary_compact_obj_animations: None,
            octonary_compact_obj_animations: None,
            ui_context_templates: None,
            compact_obj_catalog: None,
            layered_obj_animations: None,
            audio_extended_dispatch: None,
            conditional_item_parameters: None,
            range_lookup: None,
            tilemap_geometry: None,
            script39_registry_poses: None,
            script39_special_poses: None,
            direct_indexed_graphics: None,
            direct_indexed_dictionary: None,
            context_callback_descriptor: None,
            conditional_language_pair: None,
            text_reader_registry: None,
            conditional_substitution_words: None,
            display_control_tables: None,
            compact_primary_alias: None,
            observed_lz: None,
            constructor_template64: None,
            observed_tilemap: None,
            observed_tilemap_descriptor: None,
            observed_tilemap_row: None,
            dma_register_directories: None,
            iwram_memory_helpers: None,
            adjacent_offset_tables: None,
            indexed_records16: None,
            indexed_words_3959: None,
            thumb_switch_tables: None,
            thumb_pc_literals: None,
            copied_halfword_lookups: None,
            indexed_halfwords_39a5: None,
            script_byte_lookup_3ad: None,
            soil_scene_records: None,
            soil_indirect_records: None,
            soil_indexed_bytes: None,
            soil_compact_record: None,
            soil_scoped_values: None,
            soil_remaining_rectangles: None,
            firaga_copied_matrices: None,
            firaga_compact_cells: None,
            firaga_indexed_obj_bank: None,
            firaga_compact_obj_bank: None,
            firaga_adjacent_compact_obj_bank: None,
            firaga_6c12c8_obj_bank: None,
            firaga_transfer_program: None,
            firaga_palette_rows: None,
            firaga_stack_initialization: None,
            firaga_queued_transfer: None,
            white_flowers_signed_bytes: None,
            white_flowers_isolated_values: None,
            tilemap_transfer_directory: None,
            indexed_tilemap_controls: None,
            tilemap_stream_bank: None,
            tilemap_transfer_lz77: None,
            tilemap_type3: None,
            tilemap_type2: None,
            tilemap_direct: None,
            compact_cell_bank: None,
            background_effect_graphics: None,
            localized_obj_animation_banks: None,
            background_screen_banks: None,
            physical_audio_banks: None,
            physical_compact_obj_region: None,
            screen_update_tables: None,
            counted_effect_tables: None,
            map_raster_palettes: None,
            auxiliary_scripts: None,
            font_bank: None,
            manifest: Manifest {
                schema: SCHEMA,
                extractor_revision: EXTRACTOR_REV,
                decoder_revision: DECODER_REV,
                source_rom_sha256: hash,
                source_rom_size: u32::try_from(rom.len()).map_err(|_| "ROM is too large")?,
                profile: profile.into(),
                coverage: vec![OBJ_DIMENSIONS_ID.into()],
                resources: vec![Resource {
                    id: OBJ_DIMENSIONS_ID.into(),
                    path: OBJ_DIMENSIONS_FILE.into(),
                    format: "obj-dimensions-v1".into(),
                    // File bytes are defined by the frontend serializer.
                    sha256: String::new(),
                    dependencies: Vec::new(),
                    sources,
                }],
            },
        })
    }

    pub fn set_file_hashes(&mut self, files: &[&[u8]]) -> Result<(), String> {
        if self.manifest.resources.len()
            != self.resource_count()
            || files.len() != self.manifest.resources.len()
        {
            return Err("Resource set does not match extraction".into());
        }
        for (resource, bytes) in self.manifest.resources.iter_mut().zip(files) {
            resource.sha256 = ffta_rom::digest(bytes);
        }
        Ok(())
    }

    /// The expected manifest comes from the selected source, never the files.
    /// A self-consistent edited resource is still an invalid *base* dataset.
    pub fn validate_files(&self, files: BaseFiles<'_>) -> Result<(), String> {
        if self.palettes.is_some() && self.background.is_none()
            || self.alternate_palettes.is_some() && self.palettes.is_none()
            || self.packet_programs.is_some() && self.alternate_palettes.is_none()
            || self.main_graphics.is_some() && self.packet_programs.is_none()
            || self.transfers.is_some() && self.main_graphics.is_none()
            || self.short_texts.is_some() && self.transfers.is_none()
            || self.text_catalogs.is_some() && self.short_texts.is_none()
            || self.font_bank.is_some() && self.text_catalogs.is_none()
            || self.name_keyboard.is_some() && self.font_bank.is_none()
            || self.obj_lists.is_some() && self.name_keyboard.is_none()
            || self.obj_animations.is_some() && self.obj_lists.is_none()
            || self.obj_graphics.is_some() && self.obj_animations.is_none()
            || self.ui_palettes.is_some() && self.obj_graphics.is_none()
            || self.ui_graphics.is_some() && self.ui_palettes.is_none()
            || self.items.is_some() && self.ui_graphics.is_none()
            || self.item_links.is_some() && self.items.is_none()
            || self.metadata16.is_some() && self.item_links.is_none()
            || self.metadata52.is_some() && self.metadata16.is_none()
            || self.metadata4.is_some() && self.metadata52.is_none()
            || self.metadata2.is_some() && self.metadata52.is_none()
            || self.default48.is_some() && self.metadata2.is_none()
            || self.constructor_alternates.is_some() && self.default48.is_none()
            || self.metadata14.is_some() && self.constructor_alternates.is_none()
            || self.metadata12.is_some() && self.metadata14.is_none()
            || self.global_pair_flags.is_some() && self.metadata12.is_none()
            || self.evaluation_parameters.is_some() && self.global_pair_flags.is_none()
            || self.progression_rewards.is_some() && self.evaluation_parameters.is_none()
            || self.progression_gates.is_some() && self.progression_rewards.is_none()
            || self.progression_selection_gates.is_some() && self.progression_gates.is_none()
            || self.aggregate_flag_keys.is_some() && self.progression_selection_gates.is_none()
            || self.trade_parameters.is_some() && self.aggregate_flag_keys.is_none()
            || self.item_effect_parameters.is_some() && self.trade_parameters.is_none()
            || self.audio_directory.is_some() && self.item_effect_parameters.is_none()
            || self.audio_tones.is_some() && self.audio_directory.is_none()
            || self.audio_sequences.is_some() && self.audio_tones.is_none()
            || self.audio_driver.is_some() && self.audio_sequences.is_none()
            || self.palette_effect_dispatch.is_some() && self.audio_driver.is_none()
            || self.palette_programs.is_some() && self.palette_effect_dispatch.is_none()
            || self.blend_programs.is_some() && self.palette_programs.is_none()
            || self.task_descriptors.is_some() && self.blend_programs.is_none()
            || self.effect_event_directories.is_some() && self.task_descriptors.is_none()
            || self.script_commands.is_some() && self.effect_event_directories.is_none()
            || self.script_archive.is_some() && self.script_commands.is_none()
            || self.script_text.is_some() && self.script_archive.is_none()
            || self.script_coordinates.is_some() && self.script_archive.is_none()
            || self.ui_choice_lists.is_some() && self.map_raster_palettes.is_none()
            || self.unit_obj_initial_pose.is_some() && (self.ui_choice_lists.is_none() || self.metadata52.is_none() || self.metadata14.is_none())
            || self.color_mode_palettes.is_some() && (self.unit_obj_initial_pose.is_none() || self.metadata14.is_none())
            || self.unit_obj_equipment.is_some() && (self.color_mode_palettes.is_none() || self.items.is_none())
            || self.unit_obj_normal_battle.is_some() && (self.unit_obj_equipment.is_none() || self.unit_obj_initial_pose.is_none() || self.metadata52.is_none() || self.metadata14.is_none())
            || self.unit_obj_alternate_battle.is_some() && (self.unit_obj_normal_battle.is_none() || self.unit_obj_equipment.is_none() || self.unit_obj_initial_pose.is_none() || self.metadata52.is_none() || self.metadata14.is_none())
            || self.unit_obj_script_special_normal.is_some() && (self.unit_obj_alternate_battle.is_none() || self.script_archive.is_none() || self.metadata14.is_none() || self.unit_obj_initial_pose.is_none() || self.unit_obj_normal_battle.is_none())
            || self.unit_obj_script_special_alternate.is_some() && (self.unit_obj_alternate_battle.is_none() || self.script_archive.is_none() || self.metadata14.is_none() || self.unit_obj_initial_pose.is_none() || self.unit_obj_normal_battle.is_none())
            || self.unit_obj_script_override.is_some() && (self.unit_obj_alternate_battle.is_none() || self.script_archive.is_none() || self.unit_obj_normal_battle.is_none())
            || self.unit_obj_script_registry_special.is_some() && (self.metadata14.is_none() || self.script_archive.is_none() || self.unit_obj_initial_pose.is_none() || self.unit_obj_equipment.is_none() || self.unit_obj_normal_battle.is_none() || self.unit_obj_alternate_battle.is_none() || self.unit_obj_script_special_normal.is_none() || self.unit_obj_script_special_alternate.is_none())
            || self.unit_obj_script_registry_generic.is_some() && (self.metadata52.is_none() || self.script_archive.is_none() || self.unit_obj_initial_pose.is_none() || self.unit_obj_equipment.is_none() || self.unit_obj_normal_battle.is_none() || self.unit_obj_alternate_battle.is_none() || self.unit_obj_script_special_alternate.is_none() || self.unit_obj_script_registry_special.is_none())
            || self.unit_obj_property_words.is_some() && (self.metadata52.is_none() || self.metadata14.is_none())
            || self.unit_obj_byte_property_words.is_some() && (self.unit_obj_property_words.is_none() || self.color_mode_palettes.is_none())
            || self.unit_obj_ffff_reader_fields.is_some() && (self.metadata52.is_none() || self.script_archive.is_none())
            || self.slot_job_lookup.is_some() && self.metadata52.is_none()
            || self.slot_job_lookup.is_some() != self.observed_task_parameters.is_some()
            || self.observed_task_parameters.is_some() != self.constructor_obj_animation.is_some()
            || self.constructor_obj_animation.is_some() != self.task_cycle_deltas.is_some()
            || self.task_cycle_deltas.is_some() != self.word_task_obj_animations.is_some()
            || self.word_task_obj_animations.is_some() != self.world_nodes.is_some()
            || self.world_nodes.is_some() != self.world_roads.is_some()
            || self.world_roads.is_some() != self.encounter_headers.is_some()
            || self.encounter_headers.is_some() != self.encounter_unit_templates.is_some()
            || self.encounter_unit_templates.is_some() != self.encounter_coordinates.is_some()
            || self.encounter_coordinates.is_some() != self.metadata28.is_some()
            || self.encounter_coordinates.is_some() != self.records4.is_some()
            || self.records4.is_some() != self.conditional_records8.is_some()
            || self.conditional_records8.is_some() != self.mission_records70.is_some()
            || self.mission_records70.is_some() != self.selection_byte_tables.is_some()
            || self.selection_byte_tables.is_some() != self.event_records12.is_some()
            || self.event_records12.is_some() != self.parameter_records27.is_some()
            || self.parameter_records27.is_some() != self.world_records38.is_some()
            || self.world_records38.is_some() != self.indexed_graphics.is_some()
            || self.indexed_graphics.is_some() != self.ui_raw_transfers.is_some()
            || self.ui_raw_transfers.is_some() != self.dynamic_raw_fragments.is_some()
            || self.dynamic_raw_fragments.is_some() != self.conditional_four_byte_lookup.is_some()
            || self.conditional_four_byte_lookup.is_some() != self.conditional_packed_lookup.is_some()
            || self.conditional_packed_lookup.is_some() != self.conditional_descriptor_banks.is_some()
            || self.conditional_descriptor_banks.is_some() != self.conditional_descriptor_bank_zero.is_some()
            || self.conditional_descriptor_bank_zero.is_some() != self.conditional_property_masks.is_some()
            || self.conditional_property_masks.is_some() != self.compact_obj_animations.is_some()
            || self.compact_obj_animations.is_some() != self.secondary_compact_obj_animations.is_some()
            || self.secondary_compact_obj_animations.is_some() != self.tertiary_compact_obj_animations.is_some()
            || self.tertiary_compact_obj_animations.is_some() != self.quaternary_compact_obj_animations.is_some()
            || self.quaternary_compact_obj_animations.is_some() != self.quinary_compact_obj_animations.is_some()
            || self.quinary_compact_obj_animations.is_some() != self.senary_compact_obj_animations.is_some()
            || self.senary_compact_obj_animations.is_some() != self.septenary_compact_obj_animations.is_some()
            || self.septenary_compact_obj_animations.is_some() != self.octonary_compact_obj_animations.is_some()
            || self.octonary_compact_obj_animations.is_some() != self.ui_context_templates.is_some()
            || self.ui_context_templates.is_some() != self.compact_obj_catalog.is_some()
            || self.compact_obj_catalog.is_some() != self.layered_obj_animations.is_some()
            || self.layered_obj_animations.is_some() != self.audio_extended_dispatch.is_some()
            || self.audio_extended_dispatch.is_some() != self.conditional_item_parameters.is_some()
            || self.conditional_item_parameters.is_some() != self.range_lookup.is_some()
            || self.range_lookup.is_some() != self.tilemap_geometry.is_some()
            || self.tilemap_geometry.is_some() != self.script39_registry_poses.is_some()
            || self.script39_registry_poses.is_some() != self.script39_special_poses.is_some()
            || self.script39_special_poses.is_some() != self.direct_indexed_graphics.is_some()
            || self.direct_indexed_graphics.is_some() != self.direct_indexed_dictionary.is_some()
            || self.direct_indexed_dictionary.is_some() != self.context_callback_descriptor.is_some()
            || self.context_callback_descriptor.is_some() != self.conditional_language_pair.is_some()
            || self.conditional_language_pair.is_some() != self.text_reader_registry.is_some()
            || self.text_reader_registry.is_some() != self.conditional_substitution_words.is_some()
            || self.conditional_substitution_words.is_some() != self.display_control_tables.is_some()
            || self.display_control_tables.is_some() != self.compact_primary_alias.is_some()
            || self.compact_primary_alias.is_some() != self.observed_lz.is_some()
            || self.observed_lz.is_some() != self.constructor_template64.is_some()
            || self.constructor_template64.is_some() != self.observed_tilemap.is_some()
            || self.observed_tilemap.is_some() != self.observed_tilemap_descriptor.is_some()
            || self.observed_tilemap_descriptor.is_some() != self.observed_tilemap_row.is_some()
            || self.observed_tilemap_row.is_some() != self.dma_register_directories.is_some()
            || self.dma_register_directories.is_some() != self.iwram_memory_helpers.is_some()
            || self.iwram_memory_helpers.is_some() != self.adjacent_offset_tables.is_some()
            || self.adjacent_offset_tables.is_some() != self.indexed_records16.is_some()
            || self.indexed_records16.is_some() != self.indexed_words_3959.is_some()
            || self.indexed_words_3959.is_some() != self.thumb_switch_tables.is_some()
            || self.thumb_switch_tables.is_some() != self.thumb_pc_literals.is_some()
            || self.thumb_pc_literals.is_some() != self.copied_halfword_lookups.is_some()
            || self.copied_halfword_lookups.is_some() != self.indexed_halfwords_39a5.is_some()
            || self.indexed_halfwords_39a5.is_some() != self.script_byte_lookup_3ad.is_some()
            || self.script_byte_lookup_3ad.is_some() != self.soil_scene_records.is_some()
            || self.soil_scene_records.is_some() != self.soil_indirect_records.is_some()
            || self.soil_indirect_records.is_some() != self.soil_indexed_bytes.is_some()
            || self.soil_indexed_bytes.is_some() != self.soil_compact_record.is_some()
            || self.soil_compact_record.is_some() != self.soil_scoped_values.is_some()
            || self.soil_scoped_values.is_some() != self.soil_remaining_rectangles.is_some()
            || self.soil_remaining_rectangles.is_some() != self.firaga_copied_matrices.is_some()
            || self.firaga_copied_matrices.is_some() != self.firaga_compact_cells.is_some()
            || self.firaga_compact_cells.is_some() != self.firaga_indexed_obj_bank.is_some()
            || self.firaga_indexed_obj_bank.is_some() != self.firaga_compact_obj_bank.is_some()
            || self.firaga_compact_obj_bank.is_some() != self.firaga_adjacent_compact_obj_bank.is_some()
            || self.firaga_adjacent_compact_obj_bank.is_some() != self.firaga_6c12c8_obj_bank.is_some()
            || self.firaga_6c12c8_obj_bank.is_some() != self.firaga_transfer_program.is_some()
            || self.firaga_transfer_program.is_some() != self.firaga_palette_rows.is_some()
            || self.firaga_palette_rows.is_some() != self.firaga_stack_initialization.is_some()
            || self.firaga_stack_initialization.is_some() != self.firaga_queued_transfer.is_some()
            || self.firaga_queued_transfer.is_some() != self.white_flowers_signed_bytes.is_some()
            || self.white_flowers_signed_bytes.is_some() != self.white_flowers_isolated_values.is_some()
            || self.white_flowers_isolated_values.is_some() != self.tilemap_transfer_directory.is_some()
            || self.indexed_tilemap_controls.is_some() && self.tilemap_transfer_directory.is_none()
            || self.tilemap_stream_bank.is_some() && self.indexed_tilemap_controls.is_none()
            || self.tilemap_transfer_lz77.is_some() && self.tilemap_transfer_directory.is_none()
            || self.tilemap_type3.is_some() && self.tilemap_transfer_lz77.is_none()
            || self.tilemap_type2.is_some() && self.tilemap_type3.is_none()
            || self.tilemap_direct.is_some() && self.tilemap_stream_bank.is_none()
            || self.constructor_obj_animation.is_some() && self.obj_lists.is_none()
            || self.map_raster_palettes.is_some() && self.auxiliary_scripts.is_none()
            || self.auxiliary_scripts.is_some() && self.script_coordinates.is_none()
            || self.manifest.resources.len() != self.resource_count()
        {
            return Err("Extraction resource set is inconsistent".into());
        }
        let (dimensions, file_bytes) = files.dimensions;
        if file_bytes.len() as u64 > MAX_FILE_BYTES {
            return Err("OBJ dimensions file exceeds size limit".into());
        }
        let mut total = file_bytes.len();
        let mut expected = self.manifest.clone();
        expected.resources[0].sha256 = ffta_rom::digest(file_bytes);
        match (&self.background, files.background) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_DESCRIPTORS_FILE_BYTES {
                    return Err("Background descriptor file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[1].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected background descriptor resource".into()),
        }
        match (&self.palettes, files.palettes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_BASE_PALETTES_FILE_BYTES {
                    return Err("Base palette file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[2].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected base palette resource".into()),
        }
        match (&self.alternate_palettes, files.alternate_palettes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_ALTERNATE_PALETTES_FILE_BYTES {
                    return Err("Alternate palette file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[3].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected alternate palette resource".into()),
        }
        match (&self.packet_programs, files.packet_programs) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_PACKET_PROGRAMS_FILE_BYTES {
                    return Err("Packet program file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[4].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected packet program resource".into()),
        }
        match (&self.main_graphics, files.main_graphics) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_MAIN_GRAPHICS_FILE_BYTES {
                    return Err("Main graphics file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[5].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected main graphics resource".into()),
        }
        match (&self.transfers, files.transfers) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BG_TRANSFER_PROGRAMS_FILE_BYTES {
                    return Err("Transfer program file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[6].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected transfer program resource".into()),
        }
        match (&self.short_texts, files.short_texts) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SHORT_TEXTS_FILE_BYTES {
                    return Err("Short-text file exceeds size limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources[7].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected short-text resource".into()),
        }
        match (&self.text_catalogs, files.text_catalogs) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TEXT_CATALOGS_FILE_BYTES { return Err("Text catalog file exceeds size limit".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[8].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected text catalog resource".into()),
        }
        match (&self.font_bank, files.font_bank) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FONT_BANK_FILE_BYTES { return Err("font-bank file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[9].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected font-bank resource".into()),
        }
        match (&self.name_keyboard, files.name_keyboard) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_NAME_KEYBOARD_FILE_BYTES { return Err("name-keyboard file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[10].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected name-keyboard resource".into()),
        }
        match (&self.obj_lists, files.obj_lists) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBJ_STATIC_LISTS_FILE_BYTES { return Err("OBJ static-list file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[11].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected OBJ static-list resource".into()),
        }
        match (&self.obj_animations, files.obj_animations) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBJ_ANIMATION_BANKS_FILE_BYTES { return Err("OBJ animation file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[12].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected OBJ animation resource".into()),
        }
        match (&self.obj_graphics, files.obj_graphics) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBJ_CUSTOM_GRAPHICS_FILE_BYTES { return Err("OBJ custom graphics file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[13].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected OBJ custom graphics resource".into()),
        }
        match (&self.ui_palettes, files.ui_palettes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UI_RAW_PALETTES_FILE_BYTES { return Err("UI raw palette file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[14].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected UI raw palette resource".into()),
        }
        match (&self.ui_graphics, files.ui_graphics) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UI_CUSTOM_GRAPHICS_FILE_BYTES { return Err("UI custom graphics file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[15].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected UI custom graphics resource".into()),
        }
        match (&self.items, files.items) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ITEM_RECORDS_FILE_BYTES { return Err("Item file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[16].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected item resource".into()),
        }
        match (&self.item_links, files.item_links) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ITEM_LINKS_FILE_BYTES { return Err("Item-link file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[17].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected item-link resource".into()),
        }
        match (&self.metadata16, files.metadata16) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA16_FILE_BYTES { return Err("Metadata16 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[18].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata16 resource".into()),
        }
        match (&self.metadata52, files.metadata52) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA52_FILE_BYTES { return Err("Metadata52 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[19].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata52 resource".into()),
        }
        match (&self.metadata4, files.metadata4) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA4_FILE_BYTES { return Err("Metadata4 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[20].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata4 resource".into()),
        }
        match (&self.metadata2, files.metadata2) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA2_FILE_BYTES { return Err("Metadata2 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[21].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata2 resource".into()),
        }
        match (&self.default48, files.default48) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_DEFAULT48_FILE_BYTES { return Err("Default48 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[22].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected default48 resource".into()),
        }
        match (&self.constructor_alternates, files.constructor_alternates) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONSTRUCTOR_ALTERNATES_FILE_BYTES { return Err("ConstructorAlternates file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[23].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected constructor_alternates resource".into()),
        }
        match (&self.metadata14, files.metadata14) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA14_FILE_BYTES { return Err("Metadata14 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[24].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata14 resource".into()),
        }
        match (&self.metadata12, files.metadata12) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_METADATA12_FILE_BYTES { return Err("Metadata12 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[25].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected metadata12 resource".into()),
        }
        match (&self.global_pair_flags, files.global_pair_flags) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_GLOBAL_PAIR_FLAGS_FILE_BYTES { return Err("GlobalPairFlags file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[26].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected global_pair_flags resource".into()),
        }
        match (&self.evaluation_parameters, files.evaluation_parameters) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_EVALUATION_PARAMETERS_FILE_BYTES { return Err("EvaluationParameters file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[27].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected evaluation_parameters resource".into()),
        }
        match (&self.progression_rewards, files.progression_rewards) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_PROGRESSION_REWARDS_FILE_BYTES { return Err("ProgressionRewards file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[28].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected progression_rewards resource".into()),
        }
        match (&self.progression_gates, files.progression_gates) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_PROGRESSION_GATES_FILE_BYTES { return Err("ProgressionGates file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[29].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected progression_gates resource".into()),
        }
        match (&self.progression_selection_gates, files.progression_selection_gates) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_PROGRESSION_SELECTION_GATES_FILE_BYTES { return Err("ProgressionSelectionGates file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[30].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected progression_selection_gates resource".into()),
        }
        match (&self.aggregate_flag_keys, files.aggregate_flag_keys) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AGGREGATE_FLAG_KEYS_FILE_BYTES { return Err("AggregateFlagKeys file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[31].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected aggregate_flag_keys resource".into()),
        }
        match (&self.trade_parameters, files.trade_parameters) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TRADE_PARAMETERS_FILE_BYTES { return Err("TradeParameters file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[32].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected trade_parameters resource".into()),
        }
        match (&self.item_effect_parameters, files.item_effect_parameters) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ITEM_EFFECT_PARAMETERS_FILE_BYTES { return Err("ItemEffectParameters file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[33].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected item_effect_parameters resource".into()),
        }
        match (&self.audio_directory, files.audio_directory) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUDIO_DIRECTORY_FILE_BYTES { return Err("AudioDirectory file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[34].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected audio_directory resource".into()),
        }
        match (&self.audio_tones, files.audio_tones) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUDIO_TONES_FILE_BYTES { return Err("AudioTones file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[35].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected audio_tones resource".into()),
        }
        match (&self.audio_sequences, files.audio_sequences) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUDIO_SEQUENCES_FILE_BYTES { return Err("AudioSequences file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[36].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected audio_sequences resource".into()),
        }
        match (&self.audio_driver, files.audio_driver) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUDIO_DRIVER_FILE_BYTES { return Err("AudioDriver file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[37].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected audio_driver resource".into()),
        }
        match (&self.palette_effect_dispatch, files.palette_effect_dispatch) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_PALETTE_EFFECT_DISPATCH_FILE_BYTES { return Err("PaletteEffectDispatch file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[38].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected palette_effect_dispatch resource".into()),
        }
        match (&self.palette_programs, files.palette_programs) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_PALETTE_PROGRAMS_FILE_BYTES { return Err("PalettePrograms file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[39].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected palette_programs resource".into()),
        }
        match (&self.blend_programs, files.blend_programs) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_BLEND_PROGRAMS_FILE_BYTES { return Err("BlendPrograms file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[40].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected blend_programs resource".into()),
        }
        match (&self.task_descriptors, files.task_descriptors) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TASK_DESCRIPTORS_FILE_BYTES { return Err("TaskDescriptors file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[41].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected task_descriptors resource".into()),
        }
        match (&self.effect_event_directories, files.effect_event_directories) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_EFFECT_EVENT_DIRECTORIES_FILE_BYTES { return Err("EffectEventDirectories file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[42].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected effect_event_directories resource".into()),
        }
        match (&self.script_commands, files.script_commands) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT_COMMANDS_FILE_BYTES { return Err("ScriptCommands file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[43].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected script_commands resource".into()),
        }
        match (&self.script_archive, files.script_archive) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT_ARCHIVE_FILE_BYTES { return Err("ScriptArchive file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[44].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected script_archive resource".into()),
        }
        match (&self.script_text, files.script_text) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT_TEXT_FILE_BYTES { return Err("ScriptText file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[45].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected script_text resource".into()),
        }
        match (&self.script_coordinates, files.script_coordinates) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT_COORDINATES_FILE_BYTES { return Err("ScriptCoordinates file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[46].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected script_coordinates resource".into()),
        }
        match (&self.ui_choice_lists, files.ui_choice_lists) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UI_CHOICE_LISTS_FILE_BYTES { return Err("UiChoiceLists file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[50].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected ui_choice_lists resource".into()),
        }
        match (&self.unit_obj_initial_pose, files.unit_obj_initial_pose) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_INITIAL_POSE_FILE_BYTES { return Err("UnitObjInitialPose file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[51].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_initial_pose resource".into()),
        }
        match (&self.color_mode_palettes, files.color_mode_palettes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_COLOR_MODE_PALETTES_FILE_BYTES { return Err("ColorModePalettes file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[52].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected color_mode_palettes resource".into()),
        }
        match (&self.unit_obj_equipment, files.unit_obj_equipment) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_EQUIPMENT_FILE_BYTES { return Err("UnitObjEquipment file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[53].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_equipment resource".into()),
        }
        match (&self.unit_obj_normal_battle, files.unit_obj_normal_battle) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_NORMAL_BATTLE_FILE_BYTES { return Err("UnitObjNormalBattle file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[54].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_normal_battle resource".into()),
        }
        match (&self.unit_obj_alternate_battle, files.unit_obj_alternate_battle) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_ALTERNATE_BATTLE_FILE_BYTES { return Err("UnitObjAlternateBattle file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[55].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_alternate_battle resource".into()),
        }
        match (&self.unit_obj_script_special_normal, files.unit_obj_script_special_normal) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE_BYTES { return Err("UnitObjScriptSpecialNormal file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[56].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_script_special_normal resource".into()),
        }
        match (&self.unit_obj_script_special_alternate, files.unit_obj_script_special_alternate) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE_BYTES { return Err("UnitObjScriptSpecialAlternate file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[57].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_script_special_alternate resource".into()),
        }
        match (&self.unit_obj_script_override, files.unit_obj_script_override) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_SCRIPT_OVERRIDE_FILE_BYTES { return Err("UnitObjScriptOverride file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[58].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_script_override resource".into()),
        }
        match (&self.unit_obj_script_registry_special, files.unit_obj_script_registry_special) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE_BYTES { return Err("UnitObjScriptRegistrySpecial file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[59].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_script_registry_special resource".into()),
        }
        match (&self.unit_obj_script_registry_generic, files.unit_obj_script_registry_generic) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE_BYTES { return Err("UnitObjScriptRegistryGeneric file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[60].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_script_registry_generic resource".into()),
        }
        match (&self.unit_obj_property_words, files.unit_obj_property_words) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_PROPERTY_WORDS_FILE_BYTES { return Err("UnitObjPropertyWords file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[61].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_property_words resource".into()),
        }
        match (&self.unit_obj_byte_property_words, files.unit_obj_byte_property_words) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE_BYTES { return Err("UnitObjBytePropertyWords file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[62].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_byte_property_words resource".into()),
        }
        match (&self.unit_obj_ffff_reader_fields, files.unit_obj_ffff_reader_fields) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_UNIT_OBJ_FFFF_READER_FIELDS_FILE_BYTES { return Err("UnitObjFfffReaderFields file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[63].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected unit_obj_ffff_reader_fields resource".into()),
        }
        match (&self.slot_job_lookup, files.slot_job_lookup) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SLOT_JOB_LOOKUP_FILE_BYTES { return Err("SlotJobLookup file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[64].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected slot_job_lookup resource".into()),
        }
        match (&self.observed_task_parameters, files.observed_task_parameters) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBSERVED_TASK_PARAMETERS_FILE_BYTES { return Err("ObservedTaskParameters file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[65].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected observed_task_parameters resource".into()),
        }
        match (&self.constructor_obj_animation, files.constructor_obj_animation) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONSTRUCTOR_OBJ_ANIMATION_FILE_BYTES { return Err("ConstructorObjAnimation file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[66].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected constructor_obj_animation resource".into()),
        }
        match (&self.task_cycle_deltas, files.task_cycle_deltas) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TASK_CYCLE_DELTAS_FILE_BYTES { return Err("TaskCycleDeltas file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[67].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected task_cycle_deltas resource".into()),
        }
        match (&self.word_task_obj_animations, files.word_task_obj_animations) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_WORD_TASK_OBJ_ANIMATIONS_FILE_BYTES { return Err("WordTaskObjAnimations file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[68].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected word_task_obj_animations resource".into()),
        }
        match (&self.world_nodes, files.world_nodes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_WORLD_NODES_FILE_BYTES { return Err("WorldNodes file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[69].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected world_nodes resource".into()),
        }
        match (&self.world_roads, files.world_roads) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_WORLD_ROADS_FILE_BYTES { return Err("WorldRoads file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[70].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected world_roads resource".into()),
        }
        match (&self.encounter_headers, files.encounter_headers) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ENCOUNTER_HEADERS_FILE_BYTES { return Err("EncounterHeaders file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[71].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected encounter_headers resource".into()),
        }
        match (&self.encounter_unit_templates, files.encounter_unit_templates) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ENCOUNTER_UNIT_TEMPLATES_FILE_BYTES { return Err("EncounterUnitTemplates file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[72].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected encounter_unit_templates resource".into()),
        }
        match (&self.encounter_coordinates, files.encounter_coordinates) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_ENCOUNTER_COORDINATES_FILE_BYTES {return Err("Encounter coordinates file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[73].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected encounter coordinates".into()),
        }
        match (&self.metadata28, files.metadata28) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_METADATA28_FILE_BYTES {return Err("Metadata28 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[74].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected metadata28".into()),
        }
        match (&self.records4, files.records4) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_RECORDS4_FILE_BYTES {return Err("Records4 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[75].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected records4".into()),
        }
        match (&self.conditional_records8, files.conditional_records8) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_RECORDS8_FILE_BYTES {return Err("ConditionalRecords8 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[76].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_records8".into()),
        }
        match (&self.mission_records70, files.mission_records70) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_MISSION_RECORDS70_FILE_BYTES {return Err("MissionRecords70 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[77].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected mission_records70".into()),
        }
        match (&self.selection_byte_tables, files.selection_byte_tables) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_SELECTION_BYTE_TABLES_FILE_BYTES {return Err("SelectionByteTables file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[78].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected selection_byte_tables".into()),
        }
        match (&self.event_records12, files.event_records12) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_EVENT_RECORDS12_FILE_BYTES {return Err("EventRecords12 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[79].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected event_records12".into()),
        }
        match (&self.parameter_records27, files.parameter_records27) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_PARAMETER_RECORDS27_FILE_BYTES {return Err("ParameterRecords27 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[80].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected parameter_records27".into()),
        }
        match (&self.world_records38, files.world_records38) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_WORLD_RECORDS38_FILE_BYTES {return Err("WorldRecords38 file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[81].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected world_records38".into()),
        }
        match (&self.indexed_graphics, files.indexed_graphics) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_INDEXED_GRAPHICS_FILE_BYTES {return Err("IndexedGraphics file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[82].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected indexed_graphics".into()),
        }
        match (&self.ui_raw_transfers, files.ui_raw_transfers) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_UI_RAW_TRANSFERS_FILE_BYTES {return Err("RawTransfers file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[83].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected ui_raw_transfers".into()),
        }
        match (&self.dynamic_raw_fragments, files.dynamic_raw_fragments) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_DYNAMIC_RAW_FRAGMENTS_FILE_BYTES {return Err("Dynamic fragments file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[84].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected dynamic_raw_fragments".into()),
        }
        match (&self.conditional_four_byte_lookup, files.conditional_four_byte_lookup) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_FOUR_BYTE_LOOKUP_FILE_BYTES {return Err("Conditional lookup file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[85].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_four_byte_lookup".into()),
        }
        match (&self.conditional_packed_lookup, files.conditional_packed_lookup) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_PACKED_LOOKUP_FILE_BYTES {return Err("Conditional packed lookup file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[86].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_packed_lookup".into()),
        }
        match (&self.conditional_descriptor_banks, files.conditional_descriptor_banks) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_DESCRIPTOR_BANKS_FILE_BYTES {return Err("Conditional descriptor banks file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[87].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_descriptor_banks".into()),
        }
        match (&self.conditional_descriptor_bank_zero, files.conditional_descriptor_bank_zero) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_DESCRIPTOR_BANK_ZERO_FILE_BYTES {return Err("Conditional descriptor bank zero file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[88].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_descriptor_bank_zero".into()),
        }
        match (&self.conditional_property_masks, files.conditional_property_masks) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_CONDITIONAL_PROPERTY_MASKS_FILE_BYTES {return Err("Conditional property masks file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[89].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected conditional_property_masks".into()),
        }
        match (&self.compact_obj_animations, files.compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[90].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected compact_obj_animations".into()),
        }
        match (&self.secondary_compact_obj_animations, files.secondary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_SECONDARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Secondary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[91].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected secondary_compact_obj_animations".into()),
        }
        match (&self.tertiary_compact_obj_animations, files.tertiary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_TERTIARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Tertiary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[92].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected tertiary_compact_obj_animations".into()),
        }
        match (&self.quaternary_compact_obj_animations, files.quaternary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_QUATERNARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Quaternary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[93].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected quaternary_compact_obj_animations".into()),
        }
        match (&self.quinary_compact_obj_animations, files.quinary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_QUINARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Quinary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[94].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected quinary_compact_obj_animations".into()),
        }
        match (&self.senary_compact_obj_animations, files.senary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_SENARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Senary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[95].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected senary_compact_obj_animations".into()),
        }
        match (&self.septenary_compact_obj_animations, files.septenary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_SEPTENARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Septenary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[96].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected septenary_compact_obj_animations".into()),
        }
        match (&self.octonary_compact_obj_animations, files.octonary_compact_obj_animations) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_OCTONARY_COMPACT_OBJ_ANIMATIONS_FILE_BYTES {return Err("Octonary compact OBJ animations file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[97].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected octonary_compact_obj_animations".into()),
        }
        match (&self.ui_context_templates, files.ui_context_templates) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_UI_CONTEXT_TEMPLATES_FILE_BYTES {return Err("UI context templates file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[98].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected ui_context_templates".into()),
        }
        match (&self.compact_obj_catalog, files.compact_obj_catalog) {
            (Some(_),Some((_,bytes)))=>{if bytes.len() as u64>MAX_COMPACT_OBJ_CATALOG_FILE_BYTES {return Err("Compact OBJ catalog file exceeds input guard".into());}total=total.saturating_add(bytes.len());expected.resources[99].sha256=ffta_rom::digest(bytes);}
            (None,None)=>{},_=>return Err("Missing or unexpected compact_obj_catalog".into()),
        }
        match (&self.layered_obj_animations, files.layered_obj_animations) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_LAYERED_OBJ_ANIMATIONS_FILE_BYTES { return Err("Layered OBJ animations file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[100].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected layered_obj_animations".into()),
        }
        match (&self.audio_extended_dispatch, files.audio_extended_dispatch) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUDIO_EXTENDED_DISPATCH_FILE_BYTES { return Err("M4A extended dispatch file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[101].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected audio_extended_dispatch".into()),
        }
        match (&self.conditional_item_parameters, files.conditional_item_parameters) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONDITIONAL_ITEM_PARAMETERS_FILE_BYTES { return Err("Conditional item parameters file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[102].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected conditional_item_parameters".into()),
        }
        match (&self.range_lookup, files.range_lookup) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_RANGE_LOOKUP_FILE_BYTES { return Err("Range lookup file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[103].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected range_lookup".into()),
        }
        match (&self.tilemap_geometry, files.tilemap_geometry) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TILEMAP_GEOMETRY_FILE_BYTES { return Err("Tilemap geometry file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[104].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected tilemap_geometry".into()),
        }
        match (&self.script39_registry_poses, files.script39_registry_poses) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT39_REGISTRY_POSES_FILE_BYTES { return Err("Opcode-39 registry pose file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[105].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected script39_registry_poses".into()),
        }
        match (&self.script39_special_poses, files.script39_special_poses) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT39_SPECIAL_POSES_FILE_BYTES { return Err("Opcode-39 special pose file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[106].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected script39_special_poses".into()),
        }
        match (&self.direct_indexed_graphics, files.direct_indexed_graphics) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_DIRECT_INDEXED_GRAPHICS_FILE_BYTES { return Err("Direct indexed graphics file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[107].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected direct_indexed_graphics".into()),
        }
        match (&self.direct_indexed_dictionary, files.direct_indexed_dictionary) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_DIRECT_INDEXED_DICTIONARY_FILE_BYTES { return Err("Direct indexed dictionary file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[108].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected direct_indexed_dictionary".into()),
        }
        match (&self.context_callback_descriptor, files.context_callback_descriptor) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONTEXT_CALLBACK_DESCRIPTOR_FILE_BYTES { return Err("Context callback descriptor file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[109].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected context_callback_descriptor".into()),
        }
        match (&self.conditional_language_pair, files.conditional_language_pair) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONDITIONAL_LANGUAGE_PAIR_FILE_BYTES { return Err("Conditional language pair file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[110].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected conditional_language_pair".into()),
        }
        match (&self.text_reader_registry, files.text_reader_registry) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TEXT_READER_REGISTRY_FILE_BYTES { return Err("Text-reader registry file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[111].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected text_reader_registry".into()),
        }
        match (&self.conditional_substitution_words, files.conditional_substitution_words) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONDITIONAL_SUBSTITUTION_WORDS_FILE_BYTES { return Err("Conditional substitution words file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[112].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected conditional_substitution_words".into()),
        }
        match (&self.display_control_tables, files.display_control_tables) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_DISPLAY_CONTROL_TABLES_FILE_BYTES { return Err("Display-control tables file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[113].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected display_control_tables".into()),
        }
        match (&self.compact_primary_alias, files.compact_primary_alias) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_COMPACT_PRIMARY_ALIAS_FILE_BYTES { return Err("Compact primary alias file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[114].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected compact_primary_alias".into()),
        }
        match (&self.observed_lz, files.observed_lz) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBSERVED_LZ_FILE_BYTES { return Err("Observed LZ file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[115].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected observed_lz".into()),
        }
        match (&self.constructor_template64, files.constructor_template64) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_CONSTRUCTOR_TEMPLATE64_FILE_BYTES { return Err("Constructor template file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[116].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected constructor_template64".into()),
        }
        match (&self.observed_tilemap, files.observed_tilemap) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBSERVED_TILEMAP_FILE_BYTES { return Err("Observed tilemap file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[117].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected observed_tilemap".into()),
        }
        match (&self.observed_tilemap_descriptor, files.observed_tilemap_descriptor) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBSERVED_TILEMAP_DESCRIPTOR_FILE_BYTES { return Err("Observed tilemap descriptor file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[118].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected observed_tilemap_descriptor".into()),
        }
        match (&self.observed_tilemap_row, files.observed_tilemap_row) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_OBSERVED_TILEMAP_ROW_FILE_BYTES { return Err("Observed tilemap row file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[119].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected observed_tilemap_row".into()),
        }
        match (&self.dma_register_directories, files.dma_register_directories) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_DMA_REGISTER_DIRECTORIES_FILE_BYTES { return Err("DMA register directories file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[120].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected dma_register_directories".into()),
        }
        match (&self.iwram_memory_helpers, files.iwram_memory_helpers) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_IWRAM_MEMORY_HELPERS_FILE_BYTES { return Err("IWRAM memory helpers file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[121].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected iwram_memory_helpers".into()),
        }
        match (&self.adjacent_offset_tables, files.adjacent_offset_tables) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_ADJACENT_OFFSET_TABLES_FILE_BYTES { return Err("Adjacent offset tables file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[122].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected adjacent_offset_tables".into()),
        }
        match (&self.indexed_records16, files.indexed_records16) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_INDEXED_RECORDS16_FILE_BYTES { return Err("Indexed records file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[123].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected indexed_records16".into()),
        }
        match (&self.indexed_words_3959, files.indexed_words_3959) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_INDEXED_WORDS_3959_FILE_BYTES { return Err("Indexed words file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[124].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected indexed_words_3959".into()),
        }
        match (&self.thumb_switch_tables, files.thumb_switch_tables) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_THUMB_SWITCH_TABLES_FILE_BYTES { return Err("Thumb switch tables file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[125].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected thumb_switch_tables".into()),
        }
        match (&self.thumb_pc_literals, files.thumb_pc_literals) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_THUMB_PC_LITERALS_FILE_BYTES { return Err("Thumb PC literals file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[126].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected thumb_pc_literals".into()),
        }
        match (&self.copied_halfword_lookups, files.copied_halfword_lookups) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_COPIED_HALFWORD_LOOKUPS_FILE_BYTES { return Err("Copied halfword lookups file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[127].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected copied_halfword_lookups".into()),
        }
        match (&self.indexed_halfwords_39a5, files.indexed_halfwords_39a5) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_INDEXED_HALFWORDS_39A5_FILE_BYTES { return Err("Indexed halfwords 39a5c8 file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[128].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected indexed_halfwords_39a5".into()),
        }
        match (&self.script_byte_lookup_3ad, files.script_byte_lookup_3ad) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SCRIPT_BYTE_LOOKUP_3AD_FILE_BYTES { return Err("Script byte lookup 3ad16e file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[129].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected script_byte_lookup_3ad".into()),
        }
        match (&self.soil_scene_records, files.soil_scene_records) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_SCENE_RECORDS_FILE_BYTES { return Err("Soil scene records file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[130].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_scene_records".into()),
        }
        match (&self.soil_indirect_records, files.soil_indirect_records) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_INDIRECT_RECORDS_FILE_BYTES { return Err("Soil indirect records file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[131].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_indirect_records".into()),
        }
        match (&self.soil_indexed_bytes, files.soil_indexed_bytes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_INDEXED_BYTES_FILE_BYTES { return Err("Soil indexed bytes file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[132].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_indexed_bytes".into()),
        }
        match (&self.soil_compact_record, files.soil_compact_record) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_COMPACT_RECORD_FILE_BYTES { return Err("Soil compact record file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[133].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_compact_record".into()),
        }
        match (&self.soil_scoped_values, files.soil_scoped_values) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_SCOPED_VALUES_FILE_BYTES { return Err("Soil scoped values file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[134].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_scoped_values".into()),
        }
        match (&self.soil_remaining_rectangles, files.soil_remaining_rectangles) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_SOIL_REMAINING_RECTANGLES_FILE_BYTES { return Err("Soil remaining rectangles file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[135].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected soil_remaining_rectangles".into()),
        }
        match (&self.firaga_copied_matrices, files.firaga_copied_matrices) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_COPIED_MATRICES_FILE_BYTES { return Err("Firaga copied matrices file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[136].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_copied_matrices".into()),
        }
        match (&self.firaga_compact_cells, files.firaga_compact_cells) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_COMPACT_CELLS_FILE_BYTES { return Err("Firaga compact cells file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[137].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_compact_cells".into()),
        }
        match (&self.firaga_indexed_obj_bank, files.firaga_indexed_obj_bank) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_INDEXED_OBJ_BANK_FILE_BYTES { return Err("Firaga indexed OBJ bank file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[138].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_indexed_obj_bank".into()),
        }
        match (&self.firaga_compact_obj_bank, files.firaga_compact_obj_bank) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_COMPACT_OBJ_BANK_FILE_BYTES { return Err("Firaga compact OBJ bank file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[139].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_compact_obj_bank".into()),
        }
        match (&self.firaga_adjacent_compact_obj_bank, files.firaga_adjacent_compact_obj_bank) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_ADJACENT_COMPACT_OBJ_BANK_FILE_BYTES { return Err("Adjacent Firaga compact OBJ bank file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[140].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_adjacent_compact_obj_bank".into()),
        }
        match (&self.firaga_6c12c8_obj_bank, files.firaga_6c12c8_obj_bank) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_6C12C8_OBJ_BANK_FILE_BYTES { return Err("Firaga 6c12c8 OBJ bank file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[141].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_6c12c8_obj_bank".into()),
        }
        match (&self.firaga_transfer_program, files.firaga_transfer_program) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_TRANSFER_PROGRAM_FILE_BYTES { return Err("Firaga transfer program file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[142].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_transfer_program".into()),
        }
        match (&self.firaga_palette_rows, files.firaga_palette_rows) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_PALETTE_ROWS_FILE_BYTES { return Err("Firaga palette rows file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[143].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_palette_rows".into()),
        }
        match (&self.firaga_stack_initialization, files.firaga_stack_initialization) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_STACK_INITIALIZATION_FILE_BYTES { return Err("Firaga stack initialization file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[144].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_stack_initialization".into()),
        }
        match (&self.firaga_queued_transfer, files.firaga_queued_transfer) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_FIRAGA_QUEUED_TRANSFER_FILE_BYTES { return Err("Firaga queued transfer file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[145].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected firaga_queued_transfer".into()),
        }
        match (&self.white_flowers_signed_bytes, files.white_flowers_signed_bytes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_WHITE_FLOWERS_SIGNED_BYTES_FILE_BYTES { return Err("White Flowers signed-byte file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[146].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected white_flowers_signed_bytes".into()),
        }
        match (&self.white_flowers_isolated_values, files.white_flowers_isolated_values) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_WHITE_FLOWERS_ISOLATED_VALUES_FILE_BYTES { return Err("White Flowers isolated-value file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[147].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {},
            _ => return Err("Missing or unexpected white_flowers_isolated_values".into()),
        }
        match (&self.tilemap_transfer_directory, files.tilemap_transfer_directory) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_TILEMAP_TRANSFER_DIRECTORY_FILE_BYTES {
                    return Err("Tilemap transfer directory exceeds input limit".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_TRANSFER_DIRECTORY_ID)
                    .ok_or("Missing tilemap transfer directory in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected tilemap transfer directory".into()),
        }
        match (&self.indexed_tilemap_controls, files.indexed_tilemap_controls) {
            (Some(controls), Some((decoded, bytes))) => {
                if bytes.len() as u64 > MAX_INDEXED_TILEMAP_CONTROLS_FILE_BYTES {
                    return Err("Indexed tilemap controls exceed input limit".into());
                }
                if decoded != controls {
                    return Err("Decoded indexed tilemap controls differ from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == INDEXED_TILEMAP_CONTROLS_ID)
                    .ok_or("Missing indexed tilemap controls in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected indexed tilemap controls".into()),
        }
        match (&self.tilemap_stream_bank, files.tilemap_stream_bank) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_TILEMAP_STREAM_BANK_FILE_BYTES {
                    return Err("Tilemap stream bank exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Tilemap stream bank differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_STREAM_BANK_ID)
                    .ok_or("Missing tilemap stream bank in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected tilemap stream bank".into()),
        }
        match (&self.tilemap_transfer_lz77, files.tilemap_transfer_lz77) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_TILEMAP_TRANSFER_LZ77_FILE_BYTES {
                    return Err("Tilemap transfer LZ77 file exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Tilemap transfer LZ77 file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_TRANSFER_LZ77_ID)
                    .ok_or("Missing tilemap transfer LZ77 resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected tilemap transfer LZ77 file".into()),
        }
        match (&self.tilemap_type3, files.tilemap_type3) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_TILEMAP_TYPE3_FILE_BYTES {
                    return Err("Type-3 tilemap transfer file exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Type-3 tilemap transfer file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_TYPE3_ID)
                    .ok_or("Missing type-3 tilemap transfer resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected type-3 tilemap transfer file".into()),
        }
        match (&self.tilemap_type2, files.tilemap_type2) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_TILEMAP_TYPE2_FILE_BYTES {
                    return Err("Type-2 tilemap transfer file exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Type-2 tilemap transfer file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_TYPE2_ID)
                    .ok_or("Missing type-2 tilemap transfer resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected type-2 tilemap transfer file".into()),
        }
        match (&self.tilemap_direct, files.tilemap_direct) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_TILEMAP_DIRECT_FILE_BYTES {
                    return Err("Direct tilemap transfer file exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Direct tilemap transfer file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == TILEMAP_DIRECT_ID)
                    .ok_or("Missing direct tilemap transfer resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected direct tilemap transfer file".into()),
        }
        match (&self.compact_cell_bank, files.compact_cell_bank) {
            (Some(encoded), Some(bytes)) => {
                if bytes.len() as u64 > MAX_COMPACT_CELL_BANK_FILE_BYTES {
                    return Err("Compact-cell bank file exceeds input limit".into());
                }
                if bytes != encoded.as_slice() {
                    return Err("Compact-cell bank file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == COMPACT_CELL_BANK_ID)
                    .ok_or("Missing compact-cell bank resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected compact-cell bank file".into()),
        }
        match (&self.background_effect_graphics, files.background_effect_graphics) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Background effect graphics file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == BACKGROUND_EFFECT_GRAPHICS_ID)
                    .ok_or("Missing background effect graphics resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected background effect graphics file".into()),
        }
        match (&self.localized_obj_animation_banks, files.localized_obj_animation_banks) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Localized OBJ animation file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == LOCALIZED_OBJ_ANIMATION_BANKS_ID)
                    .ok_or("Missing localized OBJ animation resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected localized OBJ animation file".into()),
        }
        match (&self.background_screen_banks, files.background_screen_banks) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Background screen bank file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == BACKGROUND_SCREEN_BANKS_ID)
                    .ok_or("Missing background screen bank resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected background screen bank file".into()),
        }
        match (&self.physical_audio_banks, files.physical_audio_banks) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Physical tone bank file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == PHYSICAL_AUDIO_BANKS_ID)
                    .ok_or("Missing physical tone bank resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected physical tone bank file".into()),
        }
        match (&self.physical_compact_obj_region, files.physical_compact_obj_region) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Physical compact OBJ file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == PHYSICAL_COMPACT_OBJ_REGION_ID)
                    .ok_or("Missing physical compact OBJ resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected physical compact OBJ file".into()),
        }
        match (&self.screen_update_tables, files.screen_update_tables) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Screen update table file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == SCREEN_UPDATE_TABLES_ID)
                    .ok_or("Missing screen update table resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected screen update table file".into()),
        }
        match (&self.counted_effect_tables, files.counted_effect_tables) {
            (Some(encoded), Some(bytes)) => {
                if bytes != encoded.as_slice() {
                    return Err("Counted effect table file differs from authenticated source".into());
                }
                total = total.saturating_add(bytes.len());
                expected.resources.iter_mut()
                    .find(|resource| resource.id == COUNTED_EFFECT_TABLES_ID)
                    .ok_or("Missing counted effect table resource in expected manifest")?
                    .sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected counted effect table file".into()),
        }
        match (&self.map_raster_palettes, files.map_raster_palettes) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_MAP_RASTER_PALETTES_FILE_BYTES { return Err("MapRasterPalettes file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[49].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected map_raster_palettes resource".into()),
        }
        match (&self.auxiliary_scripts, files.auxiliary_commands) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUXILIARY_COMMANDS_FILE_BYTES { return Err("AuxiliaryCommands file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[47].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected auxiliary_commands resource".into()),
        }
        match (&self.auxiliary_scripts, files.auxiliary_archive) {
            (Some(_), Some((_, bytes))) => {
                if bytes.len() as u64 > MAX_AUXILIARY_ARCHIVE_FILE_BYTES { return Err("AuxiliaryArchive file exceeds input guard".into()); }
                total = total.saturating_add(bytes.len());
                expected.resources[48].sha256 = ffta_rom::digest(bytes);
            }
            (None, None) => {}
            _ => return Err("Missing or unexpected auxiliary_archive resource".into()),
        }

        if total > MAX_DATASET_BYTES {
            return Err("Resource files exceed total size limit".into());
        }
        if files.manifest != &expected {
            return Err(
                "Resource manifest does not match the source/profile/schema or file hash".into(),
            );
        }
        dimensions.validate()?;
        if dimensions != &self.dimensions
            || ffta_rom::digest(&dimensions.source_bytes())
                != self.manifest.resources[0].sources[0].sha256
        {
            return Err("Decoded OBJ dimensions do not reproduce the selected ROM source".into());
        }
        if let (Some(expected), Some((decoded, _))) = (&self.background, files.background) {
            let source = &self.manifest.resources[1].sources[0];
            decoded.validate(source.offset, self.manifest.source_rom_size as usize)?;
            if decoded != expected || ffta_rom::digest(&decoded.source_bytes()?) != source.sha256 {
                return Err(
                    "Decoded background descriptors do not reproduce the selected ROM source"
                        .into(),
                );
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.palettes, files.palettes) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err(
                    "Decoded base palettes do not reproduce the selected ROM source".into(),
                );
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.alternate_palettes, files.alternate_palettes) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err("Decoded alternate palettes do not reproduce the selected ROM source".into());
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.packet_programs, files.packet_programs) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err("Decoded packet programs do not reproduce the selected ROM source".into());
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.main_graphics, files.main_graphics) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err("Decoded main graphics do not reproduce the selected ROM source".into());
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.transfers, files.transfers) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err("Decoded transfer programs do not reproduce the selected ROM source".into());
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.short_texts, files.short_texts) {
            decoded.validate()?;
            if decoded != &expected.decoded {
                return Err("Decoded short texts do not reproduce the selected ROM source".into());
            }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.text_catalogs, files.text_catalogs) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded text catalogs do not reproduce selected ROM source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.font_bank, files.font_bank) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded font-bank differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.name_keyboard, files.name_keyboard) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded name-keyboard differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.obj_lists, files.obj_lists) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded OBJ static lists differ from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.obj_animations, files.obj_animations) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded OBJ animation banks differ from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.obj_graphics, files.obj_graphics) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded OBJ custom graphics differ from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.ui_palettes, files.ui_palettes) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded UI raw palettes differ from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.ui_graphics, files.ui_graphics) {
            decoded.validate()?;
            decoded.validate_shared_obj(files.obj_graphics.ok_or("Missing shared OBJ source owner")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded UI custom banks differ from authenticated source".into()); }
        }
        if let (Some(expected),Some((decoded,_))) = (&self.items, files.items) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded items differ from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.item_links, files.item_links) {
            decoded.validate_items(files.items.ok_or("Missing item-link item owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded item links differ from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata16, files.metadata16) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded metadata16 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata52, files.metadata52) {
            decoded.validate_links(files.item_links.ok_or("Missing metadata52 link owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded metadata52 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata4, files.metadata4) {
            decoded.validate_metadata52(files.metadata52.ok_or("Missing metadata4 owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded metadata4 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata2, files.metadata2) {
            decoded.validate_metadata52(files.metadata52.ok_or("Missing metadata2 owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded metadata2 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.default48, files.default48) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded default48 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.constructor_alternates, files.constructor_alternates) {
            decoded.validate_items(files.items.ok_or("Missing constructor-alternate item owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded constructor_alternates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata14, files.metadata14) {
            decoded.validate_metadata52(files.metadata52.ok_or("Missing metadata14 job owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded metadata14 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.metadata12, files.metadata12) {
            if decoded != &expected.decoded {return Err("Decoded metadata12 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.global_pair_flags, files.global_pair_flags) {
            if decoded != &expected.decoded {return Err("Decoded global_pair_flags differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.evaluation_parameters, files.evaluation_parameters) {
            if decoded != &expected.decoded {return Err("Decoded evaluation_parameters differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.progression_rewards, files.progression_rewards) {
            if decoded != &expected.decoded {return Err("Decoded progression_rewards differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.progression_gates, files.progression_gates) {
            if decoded != &expected.decoded {return Err("Decoded progression_gates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.progression_selection_gates, files.progression_selection_gates) {
            if decoded != &expected.decoded {return Err("Decoded progression_selection_gates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.aggregate_flag_keys, files.aggregate_flag_keys) {
            if decoded != &expected.decoded {return Err("Decoded aggregate_flag_keys differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.trade_parameters, files.trade_parameters) {
            if decoded != &expected.decoded {return Err("Decoded trade_parameters differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.item_effect_parameters, files.item_effect_parameters) {
            decoded.validate_items(files.items.ok_or("Missing item-effect item owner")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded item_effect_parameters differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.audio_directory, files.audio_directory) {
            if decoded != &expected.decoded {return Err("Decoded audio_directory differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.audio_tones, files.audio_tones) {
            decoded.validate_directory(files.audio_directory.ok_or("Missing actual audio-directory dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded audio_tones differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.audio_sequences, files.audio_sequences) {
            decoded.validate_owners(files.audio_directory.ok_or("Missing actual audio-directory dependency")?.0,files.audio_tones.ok_or("Missing actual audio-tone dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded audio_sequences differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.audio_driver, files.audio_driver) {
            decoded.validate_sequence_owner(files.audio_sequences.ok_or("Missing actual audio-sequence dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded audio_driver differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.palette_effect_dispatch, files.palette_effect_dispatch) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded palette_effect_dispatch differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.palette_programs, files.palette_programs) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded palette_programs differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.blend_programs, files.blend_programs) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded blend_programs differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.task_descriptors, files.task_descriptors) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded task_descriptors differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.effect_event_directories, files.effect_event_directories) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded effect_event_directories differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.script_commands, files.script_commands) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded script_commands differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.script_archive, files.script_archive) {
            let commands=files.script_commands.ok_or("Missing script archive command dependency")?.0;
            let effects=files.effect_event_directories.ok_or("Missing script archive effect dependency")?.0;
            decoded.validate_dependencies(commands,effects)?;
            if decoded != &expected.decoded {return Err("Decoded script_archive differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.script_text, files.script_text) {
            let keys=files.script_archive.ok_or("Missing script text key dependency")?.0;
            let font=files.font_bank.ok_or("Missing script text font dependency")?.0;
            decoded.validate_dependencies(keys,font)?;
            if decoded != &expected.decoded {return Err("Decoded script_text differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.script_coordinates, files.script_coordinates) {
            let archive=files.script_archive.ok_or("Missing coordinate archive dependency")?.0;
            let commands=files.script_commands.ok_or("Missing coordinate command dependency")?.0;
            decoded.validate_dependencies(archive,commands)?;
            if decoded != &expected.decoded {return Err("Decoded script_coordinates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.ui_choice_lists, files.ui_choice_lists) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded ui_choice_lists differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_initial_pose, files.unit_obj_initial_pose) {
            let jobs=files.metadata52.ok_or("Missing unit OBJ job dependency")?.0;
            let special=files.metadata14.ok_or("Missing unit OBJ special dependency")?.0;
            decoded.validate_producers(jobs,special)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_initial_pose differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.color_mode_palettes, files.color_mode_palettes) {
            let special=files.metadata14.ok_or("Missing Color-Mode palette metadata14 dependency")?.0;
            runtime::color_mode_palettes::validate_dependencies(decoded,special)?;
            if decoded != &expected.decoded {return Err("Decoded color_mode_palettes differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_equipment, files.unit_obj_equipment) {
            let items=files.items.ok_or("Missing equipment OBJ item dependency")?.0;
            decoded.validate_producers(items)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_equipment differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_normal_battle, files.unit_obj_normal_battle) {
            let jobs=files.metadata52.ok_or("Missing normal battle job dependency")?.0;
            let special=files.metadata14.ok_or("Missing normal battle special dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing normal battle initial dependency")?.0;
            let equipment=files.unit_obj_equipment.ok_or("Missing normal battle equipment dependency")?.0;
            decoded.validate_producers(jobs,special)?;
            decoded.owned_spans(initial,equipment)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_normal_battle differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_alternate_battle, files.unit_obj_alternate_battle) {
            let jobs=files.metadata52.ok_or("Missing alternate battle job dependency")?.0;
            let special=files.metadata14.ok_or("Missing alternate battle special dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing alternate battle initial dependency")?.0;
            let equipment=files.unit_obj_equipment.ok_or("Missing alternate battle equipment dependency")?.0;
            let normal=files.unit_obj_normal_battle.ok_or("Missing alternate battle normal dependency")?.0;
            decoded.validate_producers(jobs,special)?;
            decoded.owned_spans(initial,equipment,normal)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_alternate_battle differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_script_special_normal, files.unit_obj_script_special_normal) {
            let special=files.metadata14.ok_or("Missing script-special normal special dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing script-special normal initial dependency")?.0;
            let normal=files.unit_obj_normal_battle.ok_or("Missing script-special normal normal dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing script-special normal archive dependency")?.0;
            decoded.validate_producers(scripts,special)?;
            decoded.owned_spans(initial,normal)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_script_special_normal differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_script_special_alternate, files.unit_obj_script_special_alternate) {
            let special=files.metadata14.ok_or("Missing script-special alternate special dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing script-special alternate initial dependency")?.0;
            let normal=files.unit_obj_normal_battle.ok_or("Missing script-special alternate normal dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing script-special alternate archive dependency")?.0;
            decoded.validate_producers(scripts,special)?;
            let alternate=files.unit_obj_alternate_battle.ok_or("Missing script-special alternate alternate dependency")?.0;
            decoded.owned_spans(initial,normal,alternate)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_script_special_alternate differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_script_override, files.unit_obj_script_override) {
            let normal=files.unit_obj_normal_battle.ok_or("Missing script override normal dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing script override archive dependency")?.0;
            decoded.validate_producers(scripts)?;
            let alternate=files.unit_obj_alternate_battle.ok_or("Missing script override alternate dependency")?.0;
            decoded.owned_spans(normal,alternate)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_script_override differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_script_registry_special, files.unit_obj_script_registry_special) {
            let special=files.metadata14.ok_or("Missing registry-special script metadata14 dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing registry-special script script_archive dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing registry-special script unit_obj_initial_pose dependency")?.0;
            let equipment=files.unit_obj_equipment.ok_or("Missing registry-special script unit_obj_equipment dependency")?.0;
            let normal=files.unit_obj_normal_battle.ok_or("Missing registry-special script unit_obj_normal_battle dependency")?.0;
            let alternate=files.unit_obj_alternate_battle.ok_or("Missing registry-special script unit_obj_alternate_battle dependency")?.0;
            let low_normal=files.unit_obj_script_special_normal.ok_or("Missing registry-special script unit_obj_script_special_normal dependency")?.0;
            let low_alternate=files.unit_obj_script_special_alternate.ok_or("Missing registry-special script unit_obj_script_special_alternate dependency")?.0;
            decoded.validate_producers(scripts,special)?;
            decoded.owned_spans(initial,equipment,normal,alternate,low_normal,low_alternate)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_script_registry_special differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_script_registry_generic, files.unit_obj_script_registry_generic) {
            let jobs=files.metadata52.ok_or("Missing registry-generic script metadata52 dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing registry-generic script script_archive dependency")?.0;
            let initial=files.unit_obj_initial_pose.ok_or("Missing registry-generic script unit_obj_initial_pose dependency")?.0;
            let equipment=files.unit_obj_equipment.ok_or("Missing registry-generic script unit_obj_equipment dependency")?.0;
            let normal=files.unit_obj_normal_battle.ok_or("Missing registry-generic script unit_obj_normal_battle dependency")?.0;
            let alternate=files.unit_obj_alternate_battle.ok_or("Missing registry-generic script unit_obj_alternate_battle dependency")?.0;
            let low_alternate=files.unit_obj_script_special_alternate.ok_or("Missing registry-generic script unit_obj_script_special_alternate dependency")?.0;
            let known_special=files.unit_obj_script_registry_special.ok_or("Missing registry-generic script unit_obj_script_registry_special dependency")?.0;
            decoded.validate_producers(scripts,jobs)?;
            decoded.owned_spans(initial,equipment,normal,alternate,low_alternate,known_special)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_script_registry_generic differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_property_words, files.unit_obj_property_words) {
            let jobs=files.metadata52.ok_or("Missing actor propertyword metadata52 dependency")?.0;
            let special=files.metadata14.ok_or("Missing actor propertyword metadata14 dependency")?.0;
            decoded.validate_producers(jobs,special)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_property_words differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_byte_property_words, files.unit_obj_byte_property_words) {
            let properties=files.unit_obj_property_words.ok_or("Missing UI byte propertyword actor-property dependency")?.0;
            let palettes=files.color_mode_palettes.ok_or("Missing UI byte propertyword palette dependency")?.0;
            decoded.validate_shared(properties,palettes)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_byte_property_words differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.unit_obj_ffff_reader_fields, files.unit_obj_ffff_reader_fields) {
            let jobs=files.metadata52.ok_or("Missing FFFF reader metadata52 dependency")?.0;
            let scripts=files.script_archive.ok_or("Missing FFFF reader script archive dependency")?.0;
            decoded.validate_dependencies(scripts,jobs)?;
            if decoded != &expected.decoded {return Err("Decoded unit_obj_ffff_reader_fields differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.slot_job_lookup, files.slot_job_lookup) {
            let jobs=files.metadata52.ok_or("Missing slot-job metadata52 dependency")?.0;
            decoded.validate_dependencies(jobs)?;
            if decoded != &expected.decoded {return Err("Decoded slot_job_lookup differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.observed_task_parameters, files.observed_task_parameters) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded observed_task_parameters differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.constructor_obj_animation, files.constructor_obj_animation) {
            decoded.validate_lists(files.obj_lists.ok_or("Missing constructor animation OBJ-list dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded constructor_obj_animation differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.task_cycle_deltas, files.task_cycle_deltas) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded task_cycle_deltas differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.word_task_obj_animations, files.word_task_obj_animations) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded word_task_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.world_nodes, files.world_nodes) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded world_nodes differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.world_roads, files.world_roads) {
            decoded.validate()?;
            if decoded != &expected.decoded {return Err("Decoded world_roads differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.encounter_headers, files.encounter_headers) {
            decoded.validate_constructor_alternates(files.constructor_alternates.ok_or("Missing encounter constructor dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded encounter_headers differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_))) = (&self.encounter_unit_templates, files.encounter_unit_templates) {
            decoded.validate_constructor_alternates(files.constructor_alternates.ok_or("Missing encounter constructor dependency")?.0)?;
            decoded.validate_headers(files.encounter_headers.ok_or("Missing encounter header dependency")?.0)?;
            decoded.validate_references(files.items.ok_or("Missing encounter items dependency")?.0, files.metadata14.ok_or("Missing encounter descriptor dependency")?.0, files.metadata52.ok_or("Missing encounter job dependency")?.0)?;
            if decoded != &expected.decoded {return Err("Decoded encounter_unit_templates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.encounter_coordinates,files.encounter_coordinates) {
            decoded.validate_headers(files.encounter_headers.ok_or("Missing coordinate header dependency")?.0)?;
            if decoded!=&expected.decoded {return Err("Decoded coordinates differ from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.metadata28,files.metadata28) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded metadata28 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.records4,files.records4) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded records4 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_records8,files.conditional_records8) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_records8 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.mission_records70,files.mission_records70) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded mission_records70 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.selection_byte_tables,files.selection_byte_tables) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded selection_byte_tables differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.event_records12,files.event_records12) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded event_records12 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.parameter_records27,files.parameter_records27) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded parameter_records27 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.world_records38,files.world_records38) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded world_records38 differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.indexed_graphics,files.indexed_graphics) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded indexed_graphics differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.ui_raw_transfers,files.ui_raw_transfers) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded ui_raw_transfers differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.dynamic_raw_fragments,files.dynamic_raw_fragments) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded dynamic_raw_fragments differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_four_byte_lookup,files.conditional_four_byte_lookup) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_four_byte_lookup differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_packed_lookup,files.conditional_packed_lookup) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_packed_lookup differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_descriptor_banks,files.conditional_descriptor_banks) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_descriptor_banks differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_descriptor_bank_zero,files.conditional_descriptor_bank_zero) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_descriptor_bank_zero differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.conditional_property_masks,files.conditional_property_masks) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded conditional_property_masks differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.compact_obj_animations,files.compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.secondary_compact_obj_animations,files.secondary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded secondary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.tertiary_compact_obj_animations,files.tertiary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded tertiary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.quaternary_compact_obj_animations,files.quaternary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded quaternary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.quinary_compact_obj_animations,files.quinary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded quinary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.senary_compact_obj_animations,files.senary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded senary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.septenary_compact_obj_animations,files.septenary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded septenary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.octonary_compact_obj_animations,files.octonary_compact_obj_animations) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded octonary_compact_obj_animations differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.ui_context_templates,files.ui_context_templates) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded ui_context_templates differs from authenticated source".into());}
        }
        if let (Some(expected),Some((decoded,_)))=(&self.compact_obj_catalog,files.compact_obj_catalog) {
            decoded.validate()?;
            if decoded!=&expected.decoded {return Err("Decoded compact_obj_catalog differs from authenticated source".into());}
        }
        if let (Some(expected), Some((decoded, _))) = (&self.layered_obj_animations, files.layered_obj_animations) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded layered_obj_animations differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.audio_extended_dispatch, files.audio_extended_dispatch) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded audio_extended_dispatch differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.conditional_item_parameters, files.conditional_item_parameters) {
            decoded.validate_items(files.items.ok_or("Missing conditional item parameter owner")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded conditional_item_parameters differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.range_lookup, files.range_lookup) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded range_lookup differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.tilemap_geometry, files.tilemap_geometry) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded tilemap_geometry differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.script39_registry_poses, files.script39_registry_poses) {
            decoded.validate_producers(files.script_archive.ok_or("Missing opcode-39 script archive dependency")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded script39_registry_poses differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.script39_special_poses, files.script39_special_poses) {
            decoded.validate_producers(files.script_archive.ok_or("Missing opcode-39 script archive dependency")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded script39_special_poses differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.direct_indexed_graphics, files.direct_indexed_graphics) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded direct_indexed_graphics differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.direct_indexed_dictionary, files.direct_indexed_dictionary) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded direct_indexed_dictionary differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.context_callback_descriptor, files.context_callback_descriptor) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded context_callback_descriptor differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.conditional_language_pair, files.conditional_language_pair) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded conditional_language_pair differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.text_reader_registry, files.text_reader_registry) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded text_reader_registry differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.conditional_substitution_words, files.conditional_substitution_words) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded conditional_substitution_words differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.display_control_tables, files.display_control_tables) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded display_control_tables differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.compact_primary_alias, files.compact_primary_alias) {
            let tasks = files.task_descriptors.ok_or("Missing task descriptor dependency")?.0;
            let compact = files.compact_obj_catalog.ok_or("Missing compact catalog dependency")?.0;
            decoded.validate_dependencies(tasks, compact)?;
            if decoded != &expected.decoded { return Err("Decoded compact_primary_alias differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.observed_lz, files.observed_lz) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded observed_lz differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.constructor_template64, files.constructor_template64) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded constructor_template64 differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.observed_tilemap, files.observed_tilemap) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded observed_tilemap differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.observed_tilemap_descriptor, files.observed_tilemap_descriptor) {
            decoded.validate_dependencies(files.observed_tilemap.ok_or("Missing tilemap dependency")?.0)?;
            decoded.validate_lz_dependencies(files.observed_lz.ok_or("Missing observed LZ dependency")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded observed_tilemap_descriptor differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.observed_tilemap_row, files.observed_tilemap_row) {
            decoded.validate_dependencies(files.observed_tilemap_descriptor.ok_or("Missing tilemap dependency")?.0)?;
            decoded.validate_tilemap_dependency(files.observed_tilemap.ok_or("Missing rectangle list dependency")?.0)?;
            if decoded != &expected.decoded { return Err("Decoded observed_tilemap_row differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.dma_register_directories, files.dma_register_directories) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded dma_register_directories differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.iwram_memory_helpers, files.iwram_memory_helpers) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded iwram_memory_helpers differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.adjacent_offset_tables, files.adjacent_offset_tables) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded adjacent_offset_tables differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.indexed_records16, files.indexed_records16) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded indexed_records16 differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.indexed_words_3959, files.indexed_words_3959) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded indexed_words_3959 differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.thumb_switch_tables, files.thumb_switch_tables) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded thumb_switch_tables differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.thumb_pc_literals, files.thumb_pc_literals) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded thumb_pc_literals differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.copied_halfword_lookups, files.copied_halfword_lookups) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded copied_halfword_lookups differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.indexed_halfwords_39a5, files.indexed_halfwords_39a5) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded indexed_halfwords_39a5 differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.script_byte_lookup_3ad, files.script_byte_lookup_3ad) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded script_byte_lookup_3ad differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_scene_records, files.soil_scene_records) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_scene_records differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_indirect_records, files.soil_indirect_records) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_indirect_records differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_indexed_bytes, files.soil_indexed_bytes) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_indexed_bytes differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_compact_record, files.soil_compact_record) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_compact_record differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_scoped_values, files.soil_scoped_values) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_scoped_values differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.soil_remaining_rectangles, files.soil_remaining_rectangles) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded soil_remaining_rectangles differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_copied_matrices, files.firaga_copied_matrices) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_copied_matrices differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_compact_cells, files.firaga_compact_cells) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_compact_cells differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_indexed_obj_bank, files.firaga_indexed_obj_bank) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_indexed_obj_bank differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_compact_obj_bank, files.firaga_compact_obj_bank) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_compact_obj_bank differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_adjacent_compact_obj_bank, files.firaga_adjacent_compact_obj_bank) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_adjacent_compact_obj_bank differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_6c12c8_obj_bank, files.firaga_6c12c8_obj_bank) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_6c12c8_obj_bank differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_transfer_program, files.firaga_transfer_program) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_transfer_program differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_palette_rows, files.firaga_palette_rows) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_palette_rows differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_stack_initialization, files.firaga_stack_initialization) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_stack_initialization differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.firaga_queued_transfer, files.firaga_queued_transfer) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded firaga_queued_transfer differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.white_flowers_signed_bytes, files.white_flowers_signed_bytes) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded white_flowers_signed_bytes differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.white_flowers_isolated_values, files.white_flowers_isolated_values) {
            decoded.validate()?;
            if decoded != &expected.decoded { return Err("Decoded white_flowers_isolated_values differs from authenticated source".into()); }
        }
        if let (Some(expected), Some((decoded, _))) = (&self.tilemap_transfer_directory, files.tilemap_transfer_directory) {
            if decoded != &expected.decoded {
                return Err("Decoded tilemap transfer directory differs from authenticated source".into());
            }
        }
        if let (Some(expected),Some((decoded,_))) = (&self.map_raster_palettes, files.map_raster_palettes) {
            let descriptors=files.background.ok_or("Missing raster palette descriptor dependency")?.0;
            decoded.validate_dependencies(descriptors)?;
            if decoded != &expected.decoded {return Err("Decoded map_raster_palettes differs from authenticated source".into());}
        }
        if let Some(expected) = &self.auxiliary_scripts {
            let commands=files.auxiliary_commands.ok_or("Missing auxiliary command dependency")?.0;
            let archive=files.auxiliary_archive.ok_or("Missing auxiliary archive resource")?.0;
            commands.validate()?;
            archive.validate_dependencies(commands)?;
            if commands != &expected.commands || archive != &expected.decoded {return Err("Decoded auxiliary scripts differ from authenticated source".into());}
        }
        Ok(())
    }

    fn resource_count(&self) -> usize {
        1 + usize::from(self.background.is_some()) + usize::from(self.palettes.is_some())
            + usize::from(self.alternate_palettes.is_some())
            + usize::from(self.packet_programs.is_some())
            + usize::from(self.main_graphics.is_some())
            + usize::from(self.transfers.is_some())
            + usize::from(self.short_texts.is_some())
            + usize::from(self.text_catalogs.is_some())
            + usize::from(self.name_keyboard.is_some())
            + usize::from(self.font_bank.is_some())
            + usize::from(self.obj_lists.is_some())
            + usize::from(self.obj_animations.is_some())
            + usize::from(self.obj_graphics.is_some())
            + usize::from(self.ui_palettes.is_some())
            + usize::from(self.ui_graphics.is_some())
            + usize::from(self.items.is_some())
            + usize::from(self.item_links.is_some())
            + usize::from(self.metadata16.is_some())
            + usize::from(self.metadata52.is_some())
            + usize::from(self.metadata4.is_some())
            + usize::from(self.metadata2.is_some())
            + usize::from(self.default48.is_some())
            + usize::from(self.constructor_alternates.is_some())
            + usize::from(self.metadata14.is_some())
            + usize::from(self.metadata12.is_some())
            + usize::from(self.global_pair_flags.is_some())
            + usize::from(self.evaluation_parameters.is_some())
            + usize::from(self.progression_rewards.is_some())
            + usize::from(self.progression_gates.is_some())
            + usize::from(self.progression_selection_gates.is_some())
            + usize::from(self.aggregate_flag_keys.is_some())
            + usize::from(self.trade_parameters.is_some())
            + usize::from(self.item_effect_parameters.is_some())
            + usize::from(self.audio_directory.is_some())
            + usize::from(self.audio_tones.is_some())
            + usize::from(self.audio_sequences.is_some())
            + usize::from(self.audio_driver.is_some())
            + usize::from(self.palette_effect_dispatch.is_some())
            + usize::from(self.palette_programs.is_some())
            + usize::from(self.blend_programs.is_some())
            + usize::from(self.task_descriptors.is_some())
            + usize::from(self.effect_event_directories.is_some())
            + usize::from(self.script_commands.is_some())
            + usize::from(self.script_archive.is_some())
            + usize::from(self.script_text.is_some())
            + usize::from(self.script_coordinates.is_some())
            + usize::from(self.ui_choice_lists.is_some())
            + usize::from(self.unit_obj_initial_pose.is_some())
            + usize::from(self.color_mode_palettes.is_some())
            + usize::from(self.unit_obj_equipment.is_some())
            + usize::from(self.unit_obj_normal_battle.is_some())
            + usize::from(self.unit_obj_alternate_battle.is_some())
            + usize::from(self.unit_obj_script_special_normal.is_some())
            + usize::from(self.unit_obj_script_special_alternate.is_some())
            + usize::from(self.unit_obj_script_override.is_some())
            + usize::from(self.unit_obj_script_registry_special.is_some())
            + usize::from(self.unit_obj_script_registry_generic.is_some())
            + usize::from(self.unit_obj_property_words.is_some())
            + usize::from(self.unit_obj_byte_property_words.is_some())
            + usize::from(self.unit_obj_ffff_reader_fields.is_some())
            + usize::from(self.slot_job_lookup.is_some())
            + usize::from(self.observed_task_parameters.is_some())
            + usize::from(self.constructor_obj_animation.is_some())
            + usize::from(self.task_cycle_deltas.is_some())
            + usize::from(self.word_task_obj_animations.is_some())
            + usize::from(self.world_nodes.is_some())
            + usize::from(self.world_roads.is_some())
            + usize::from(self.encounter_headers.is_some())
            + usize::from(self.encounter_unit_templates.is_some())
            + usize::from(self.encounter_coordinates.is_some())
            + usize::from(self.metadata28.is_some())
            + usize::from(self.records4.is_some())
            + usize::from(self.conditional_records8.is_some())
            + usize::from(self.mission_records70.is_some())
            + usize::from(self.selection_byte_tables.is_some())
            + usize::from(self.event_records12.is_some())
            + usize::from(self.parameter_records27.is_some())
            + usize::from(self.world_records38.is_some())
            + usize::from(self.indexed_graphics.is_some())
            + usize::from(self.ui_raw_transfers.is_some())
            + usize::from(self.dynamic_raw_fragments.is_some())
            + usize::from(self.conditional_four_byte_lookup.is_some())
            + usize::from(self.conditional_packed_lookup.is_some())
            + usize::from(self.conditional_descriptor_banks.is_some())
            + usize::from(self.conditional_descriptor_bank_zero.is_some())
            + usize::from(self.conditional_property_masks.is_some())
            + usize::from(self.compact_obj_animations.is_some())
            + usize::from(self.secondary_compact_obj_animations.is_some())
            + usize::from(self.tertiary_compact_obj_animations.is_some())
            + usize::from(self.quaternary_compact_obj_animations.is_some())
            + usize::from(self.quinary_compact_obj_animations.is_some())
            + usize::from(self.senary_compact_obj_animations.is_some())
            + usize::from(self.septenary_compact_obj_animations.is_some())
            + usize::from(self.octonary_compact_obj_animations.is_some())
            + usize::from(self.ui_context_templates.is_some())
            + usize::from(self.compact_obj_catalog.is_some())
            + usize::from(self.layered_obj_animations.is_some())
            + usize::from(self.audio_extended_dispatch.is_some())
            + usize::from(self.conditional_item_parameters.is_some())
            + usize::from(self.range_lookup.is_some())
            + usize::from(self.tilemap_geometry.is_some())
            + usize::from(self.script39_registry_poses.is_some())
            + usize::from(self.script39_special_poses.is_some())
            + usize::from(self.direct_indexed_graphics.is_some())
            + usize::from(self.direct_indexed_dictionary.is_some())
            + usize::from(self.context_callback_descriptor.is_some())
            + usize::from(self.conditional_language_pair.is_some())
            + usize::from(self.text_reader_registry.is_some())
            + usize::from(self.conditional_substitution_words.is_some())
            + usize::from(self.display_control_tables.is_some())
            + usize::from(self.compact_primary_alias.is_some())
            + usize::from(self.observed_lz.is_some())
            + usize::from(self.constructor_template64.is_some())
            + usize::from(self.observed_tilemap.is_some())
            + usize::from(self.observed_tilemap_descriptor.is_some())
            + usize::from(self.observed_tilemap_row.is_some())
            + usize::from(self.dma_register_directories.is_some())
            + usize::from(self.iwram_memory_helpers.is_some())
            + usize::from(self.adjacent_offset_tables.is_some())
            + usize::from(self.indexed_records16.is_some())
            + usize::from(self.indexed_words_3959.is_some())
            + usize::from(self.thumb_switch_tables.is_some())
            + usize::from(self.thumb_pc_literals.is_some())
            + usize::from(self.copied_halfword_lookups.is_some())
            + usize::from(self.indexed_halfwords_39a5.is_some())
            + usize::from(self.script_byte_lookup_3ad.is_some())
            + usize::from(self.soil_scene_records.is_some())
            + usize::from(self.soil_indirect_records.is_some())
            + usize::from(self.soil_indexed_bytes.is_some())
            + usize::from(self.soil_compact_record.is_some())
            + usize::from(self.soil_scoped_values.is_some())
            + usize::from(self.soil_remaining_rectangles.is_some())
            + usize::from(self.firaga_copied_matrices.is_some())
            + usize::from(self.firaga_compact_cells.is_some())
            + usize::from(self.firaga_indexed_obj_bank.is_some())
            + usize::from(self.firaga_compact_obj_bank.is_some())
            + usize::from(self.firaga_adjacent_compact_obj_bank.is_some())
            + usize::from(self.firaga_6c12c8_obj_bank.is_some())
            + usize::from(self.firaga_transfer_program.is_some())
            + usize::from(self.firaga_palette_rows.is_some())
            + usize::from(self.firaga_stack_initialization.is_some())
            + usize::from(self.firaga_queued_transfer.is_some())
            + usize::from(self.white_flowers_signed_bytes.is_some())
            + usize::from(self.white_flowers_isolated_values.is_some())
            + usize::from(self.tilemap_transfer_directory.is_some())
            + usize::from(self.indexed_tilemap_controls.is_some())
            + usize::from(self.tilemap_stream_bank.is_some())
            + usize::from(self.tilemap_transfer_lz77.is_some())
            + usize::from(self.tilemap_type3.is_some())
            + usize::from(self.tilemap_type2.is_some())
            + usize::from(self.tilemap_direct.is_some())
            + usize::from(self.compact_cell_bank.is_some())
            + usize::from(self.background_effect_graphics.is_some())
            + usize::from(self.localized_obj_animation_banks.is_some())
            + usize::from(self.background_screen_banks.is_some())
            + usize::from(self.physical_audio_banks.is_some())
            + usize::from(self.physical_compact_obj_region.is_some())
            + usize::from(self.screen_update_tables.is_some())
            + usize::from(self.counted_effect_tables.is_some())
            + usize::from(self.map_raster_palettes.is_some())
            + 2 * usize::from(self.auxiliary_scripts.is_some())
    }

    pub fn version(&self) -> Version {
        if self.manifest.profile == "usa" {
            Version::Usa
        } else {
            Version::Europe
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> [u8; 48] {
        let mut bytes = [0; 48];
        for i in 0..12 {
            bytes[i * 4..i * 4 + 2].copy_from_slice(&(8u16 << (i % 4)).to_le_bytes());
            bytes[i * 4 + 2..i * 4 + 4].copy_from_slice(&(8u16 << ((i + 1) % 4)).to_le_bytes());
        }
        bytes
    }

    #[test]
    fn dimensions_roundtrip_and_invalid_inputs() {
        let bytes = fixture();
        let decoded = ObjDimensions::decode(&bytes).unwrap();
        assert_eq!(decoded.source_bytes(), bytes);
        assert_eq!(decoded.get(2, 3), Some(&decoded.entries[11]));
        assert!(decoded.get(3, 0).is_none());
        assert!(decoded.get(0, 4).is_none());
        assert!(decoded.get(255, 255).is_none());
        for len in 0..48 {
            assert!(ObjDimensions::decode(&bytes[..len]).is_err());
        }
        assert!(ObjDimensions::decode(&[0; 49]).is_err());
        let mut invalid = bytes;
        invalid[0..2].copy_from_slice(&256u16.to_le_bytes());
        assert!(ObjDimensions::decode(&invalid).is_err());
        assert!(ObjDimensions::decode(&[0; 48]).is_err());
    }

    #[test]
    fn independent_aliases_bounds_and_base_authentication() {
        let bytes = fixture();
        let hash = ffta_rom::digest(&bytes);
        let mut rom = bytes.repeat(2);
        let ranges = [(0, "direct"), (48, "iwram-copy")];
        let extraction =
            Extraction::from_ranges(&rom, "synthetic".into(), "test", &ranges, &hash).unwrap();
        let mut manifest = extraction.manifest.clone();
        manifest.resources[0].sha256 = ffta_rom::digest(b"file");
        extraction
            .validate_files(BaseFiles {
                manifest: &manifest,
                dimensions: (&extraction.dimensions, b"file"),
                background: None,
                palettes: None,
                alternate_palettes: None,
                packet_programs: None,
                main_graphics: None,
                transfers: None,
            short_texts: None,
            text_catalogs: None,
            name_keyboard: None,
            obj_lists: None,
            obj_animations: None,
            obj_graphics: None,
            ui_palettes: None,
            ui_graphics: None,
            items: None,
            item_links: None,
            metadata16: None,
            metadata52: None,
            metadata4: None,
            metadata2: None,
            default48: None,
            constructor_alternates: None,
            metadata14: None,
            metadata12: None,
            global_pair_flags: None,
            evaluation_parameters: None,
            progression_rewards: None,
            progression_gates: None,
            progression_selection_gates: None,
            aggregate_flag_keys: None,
            trade_parameters: None,
            item_effect_parameters: None,
            audio_directory: None,
            audio_tones: None,
            audio_sequences: None,
            audio_driver: None,
            palette_effect_dispatch: None,
            palette_programs: None,
            blend_programs: None,
            task_descriptors: None,
            effect_event_directories: None,
            script_commands: None,
            script_archive: None,
            script_text: None,
            script_coordinates: None,
            ui_choice_lists: None,
            unit_obj_initial_pose: None,
            color_mode_palettes: None,
            unit_obj_equipment: None,
            unit_obj_normal_battle: None,
            unit_obj_alternate_battle: None,
            unit_obj_script_special_normal: None,
            unit_obj_script_special_alternate: None,
            unit_obj_script_override: None,
            unit_obj_script_registry_special: None,
            unit_obj_script_registry_generic: None,
            unit_obj_property_words: None,
            unit_obj_byte_property_words: None,
            unit_obj_ffff_reader_fields: None,
            slot_job_lookup: None,
            observed_task_parameters: None,
            constructor_obj_animation: None,
            task_cycle_deltas: None,
            word_task_obj_animations: None,
            world_nodes: None,
            world_roads: None,
            encounter_headers: None,
            encounter_unit_templates: None,
            encounter_coordinates: None,
            metadata28: None,
            records4: None,
            conditional_records8: None,
            mission_records70: None,
            selection_byte_tables: None,
            event_records12: None,
            parameter_records27: None,
            world_records38: None,
            indexed_graphics: None,
            ui_raw_transfers: None,
            dynamic_raw_fragments: None,
            conditional_four_byte_lookup: None,
            conditional_packed_lookup: None,
            conditional_descriptor_banks: None,
            conditional_descriptor_bank_zero: None,
            conditional_property_masks: None,
            compact_obj_animations: None,
            secondary_compact_obj_animations: None,
            tertiary_compact_obj_animations: None,
            quaternary_compact_obj_animations: None,
            quinary_compact_obj_animations: None,
            senary_compact_obj_animations: None,
            septenary_compact_obj_animations: None,
            octonary_compact_obj_animations: None,
            ui_context_templates: None,
            compact_obj_catalog: None,
            layered_obj_animations: None,
            audio_extended_dispatch: None,
            conditional_item_parameters: None,
            range_lookup: None,
            tilemap_geometry: None,
            script39_registry_poses: None,
            script39_special_poses: None,
            direct_indexed_graphics: None,
            direct_indexed_dictionary: None,
            context_callback_descriptor: None,
            conditional_language_pair: None,
            text_reader_registry: None,
            conditional_substitution_words: None,
            display_control_tables: None,
            compact_primary_alias: None,
            observed_lz: None,
            constructor_template64: None,
            observed_tilemap: None,
            observed_tilemap_descriptor: None,
            observed_tilemap_row: None,
            dma_register_directories: None,
            iwram_memory_helpers: None,
            adjacent_offset_tables: None,
            indexed_records16: None,
            indexed_words_3959: None,
            thumb_switch_tables: None,
            thumb_pc_literals: None,
            copied_halfword_lookups: None,
            indexed_halfwords_39a5: None,
            script_byte_lookup_3ad: None,
            soil_scene_records: None,
            soil_indirect_records: None,
            soil_indexed_bytes: None,
            soil_compact_record: None,
            soil_scoped_values: None,
            soil_remaining_rectangles: None,
            firaga_copied_matrices: None,
            firaga_compact_cells: None,
            firaga_indexed_obj_bank: None,
            firaga_compact_obj_bank: None,
            firaga_adjacent_compact_obj_bank: None,
            firaga_6c12c8_obj_bank: None,
            firaga_transfer_program: None,
            firaga_palette_rows: None,
            firaga_stack_initialization: None,
            firaga_queued_transfer: None,
            white_flowers_signed_bytes: None,
            white_flowers_isolated_values: None,
            tilemap_transfer_directory: None,
            indexed_tilemap_controls: None,
            tilemap_stream_bank: None,
            tilemap_transfer_lz77: None,
            tilemap_type3: None,
            tilemap_type2: None,
            tilemap_direct: None,
            compact_cell_bank: None,
            background_effect_graphics: None,
            localized_obj_animation_banks: None,
            background_screen_banks: None,
            physical_audio_banks: None,
            physical_compact_obj_region: None,
            screen_update_tables: None,
            counted_effect_tables: None,
            map_raster_palettes: None,
            auxiliary_commands: None,
            auxiliary_archive: None,
            font_bank: None,
            })
            .unwrap();
        let mut edited = extraction.dimensions.clone();
        edited.entries[0].width = 64;
        assert!(extraction
            .validate_files(BaseFiles {
                manifest: &manifest,
                dimensions: (&edited, b"file"),
                background: None,
                palettes: None,
                alternate_palettes: None,
                packet_programs: None,
                main_graphics: None,
                transfers: None,
            short_texts: None,
            text_catalogs: None,
            name_keyboard: None,
            obj_lists: None,
            obj_animations: None,
            obj_graphics: None,
            ui_palettes: None,
            ui_graphics: None,
            items: None,
            item_links: None,
            metadata16: None,
            metadata52: None,
            metadata4: None,
            metadata2: None,
            default48: None,
            constructor_alternates: None,
            metadata14: None,
            metadata12: None,
            global_pair_flags: None,
            evaluation_parameters: None,
            progression_rewards: None,
            progression_gates: None,
            progression_selection_gates: None,
            aggregate_flag_keys: None,
            trade_parameters: None,
            item_effect_parameters: None,
            audio_directory: None,
            audio_tones: None,
            audio_sequences: None,
            audio_driver: None,
            palette_effect_dispatch: None,
            palette_programs: None,
            blend_programs: None,
            task_descriptors: None,
            effect_event_directories: None,
            script_commands: None,
            script_archive: None,
            script_text: None,
            script_coordinates: None,
            ui_choice_lists: None,
            unit_obj_initial_pose: None,
            color_mode_palettes: None,
            unit_obj_equipment: None,
            unit_obj_normal_battle: None,
            unit_obj_alternate_battle: None,
            unit_obj_script_special_normal: None,
            unit_obj_script_special_alternate: None,
            unit_obj_script_override: None,
            unit_obj_script_registry_special: None,
            unit_obj_script_registry_generic: None,
            unit_obj_property_words: None,
            unit_obj_byte_property_words: None,
            unit_obj_ffff_reader_fields: None,
            slot_job_lookup: None,
            observed_task_parameters: None,
            constructor_obj_animation: None,
            task_cycle_deltas: None,
            word_task_obj_animations: None,
            world_nodes: None,
            world_roads: None,
            encounter_headers: None,
            encounter_unit_templates: None,
            encounter_coordinates: None,
            metadata28: None,
            records4: None,
            conditional_records8: None,
            mission_records70: None,
            selection_byte_tables: None,
            event_records12: None,
            parameter_records27: None,
            world_records38: None,
            indexed_graphics: None,
            ui_raw_transfers: None,
            dynamic_raw_fragments: None,
            conditional_four_byte_lookup: None,
            conditional_packed_lookup: None,
            conditional_descriptor_banks: None,
            conditional_descriptor_bank_zero: None,
            conditional_property_masks: None,
            compact_obj_animations: None,
            secondary_compact_obj_animations: None,
            tertiary_compact_obj_animations: None,
            quaternary_compact_obj_animations: None,
            quinary_compact_obj_animations: None,
            senary_compact_obj_animations: None,
            septenary_compact_obj_animations: None,
            octonary_compact_obj_animations: None,
            ui_context_templates: None,
            compact_obj_catalog: None,
            layered_obj_animations: None,
            audio_extended_dispatch: None,
            conditional_item_parameters: None,
            range_lookup: None,
            tilemap_geometry: None,
            script39_registry_poses: None,
            script39_special_poses: None,
            direct_indexed_graphics: None,
            direct_indexed_dictionary: None,
            context_callback_descriptor: None,
            conditional_language_pair: None,
            text_reader_registry: None,
            conditional_substitution_words: None,
            display_control_tables: None,
            compact_primary_alias: None,
            observed_lz: None,
            constructor_template64: None,
            observed_tilemap: None,
            observed_tilemap_descriptor: None,
            observed_tilemap_row: None,
            dma_register_directories: None,
            iwram_memory_helpers: None,
            adjacent_offset_tables: None,
            indexed_records16: None,
            indexed_words_3959: None,
            thumb_switch_tables: None,
            thumb_pc_literals: None,
            copied_halfword_lookups: None,
            indexed_halfwords_39a5: None,
            script_byte_lookup_3ad: None,
            soil_scene_records: None,
            soil_indirect_records: None,
            soil_indexed_bytes: None,
            soil_compact_record: None,
            soil_scoped_values: None,
            soil_remaining_rectangles: None,
            firaga_copied_matrices: None,
            firaga_compact_cells: None,
            firaga_indexed_obj_bank: None,
            firaga_compact_obj_bank: None,
            firaga_adjacent_compact_obj_bank: None,
            firaga_6c12c8_obj_bank: None,
            firaga_transfer_program: None,
            firaga_palette_rows: None,
            firaga_stack_initialization: None,
            firaga_queued_transfer: None,
            white_flowers_signed_bytes: None,
            white_flowers_isolated_values: None,
            tilemap_transfer_directory: None,
            indexed_tilemap_controls: None,
            tilemap_stream_bank: None,
            tilemap_transfer_lz77: None,
            tilemap_type3: None,
            tilemap_type2: None,
            tilemap_direct: None,
            compact_cell_bank: None,
            background_effect_graphics: None,
            localized_obj_animation_banks: None,
            background_screen_banks: None,
            physical_audio_banks: None,
            physical_compact_obj_region: None,
            screen_update_tables: None,
            counted_effect_tables: None,
            map_raster_palettes: None,
            auxiliary_commands: None,
            auxiliary_archive: None,
            font_bank: None,
            })
            .is_err());
        for mutate in [0, 1, 2, 3, 4, 5, 6, 7] {
            let mut bad = manifest.clone();
            match mutate {
                0 => bad.schema += 1,
                1 => bad.decoder_revision += 1,
                2 => bad.extractor_revision += 1,
                3 => bad.source_rom_sha256 = "edited".into(),
                4 => bad.resources[0].path = "../escape".into(),
                5 => bad.resources.push(bad.resources[0].clone()),
                6 => bad.resources[0].sources[0].offset += 1,
                _ => bad.coverage.push("unverified".into()),
            }
            assert!(extraction
                .validate_files(BaseFiles {
                    manifest: &bad,
                    dimensions: (&extraction.dimensions, b"file"),
                    background: None,
                    palettes: None,
                    alternate_palettes: None,
                    packet_programs: None,
                    main_graphics: None,
                    transfers: None,
            short_texts: None,
            text_catalogs: None,
            name_keyboard: None,
            obj_lists: None,
            obj_animations: None,
            obj_graphics: None,
            ui_palettes: None,
            ui_graphics: None,
            items: None,
            item_links: None,
            metadata16: None,
            metadata52: None,
            metadata4: None,
            metadata2: None,
            default48: None,
            constructor_alternates: None,
            metadata14: None,
            metadata12: None,
            global_pair_flags: None,
            evaluation_parameters: None,
            progression_rewards: None,
            progression_gates: None,
            progression_selection_gates: None,
            aggregate_flag_keys: None,
            trade_parameters: None,
            item_effect_parameters: None,
            audio_directory: None,
            audio_tones: None,
            audio_sequences: None,
            audio_driver: None,
            palette_effect_dispatch: None,
            palette_programs: None,
            blend_programs: None,
            task_descriptors: None,
            effect_event_directories: None,
            script_commands: None,
            script_archive: None,
            script_text: None,
            script_coordinates: None,
            ui_choice_lists: None,
            unit_obj_initial_pose: None,
            color_mode_palettes: None,
            unit_obj_equipment: None,
            unit_obj_normal_battle: None,
            unit_obj_alternate_battle: None,
            unit_obj_script_special_normal: None,
            unit_obj_script_special_alternate: None,
            unit_obj_script_override: None,
            unit_obj_script_registry_special: None,
            unit_obj_script_registry_generic: None,
            unit_obj_property_words: None,
            unit_obj_byte_property_words: None,
            unit_obj_ffff_reader_fields: None,
            slot_job_lookup: None,
            observed_task_parameters: None,
            constructor_obj_animation: None,
            task_cycle_deltas: None,
            word_task_obj_animations: None,
            world_nodes: None,
            world_roads: None,
            encounter_headers: None,
            encounter_unit_templates: None,
            encounter_coordinates: None,
            metadata28: None,
            records4: None,
            conditional_records8: None,
            mission_records70: None,
            selection_byte_tables: None,
            event_records12: None,
            parameter_records27: None,
            world_records38: None,
            indexed_graphics: None,
            ui_raw_transfers: None,
            dynamic_raw_fragments: None,
            conditional_four_byte_lookup: None,
            conditional_packed_lookup: None,
            conditional_descriptor_banks: None,
            conditional_descriptor_bank_zero: None,
            conditional_property_masks: None,
            compact_obj_animations: None,
            secondary_compact_obj_animations: None,
            tertiary_compact_obj_animations: None,
            quaternary_compact_obj_animations: None,
            quinary_compact_obj_animations: None,
            senary_compact_obj_animations: None,
            septenary_compact_obj_animations: None,
            octonary_compact_obj_animations: None,
            ui_context_templates: None,
            compact_obj_catalog: None,
            layered_obj_animations: None,
            audio_extended_dispatch: None,
            conditional_item_parameters: None,
            range_lookup: None,
            tilemap_geometry: None,
            script39_registry_poses: None,
            script39_special_poses: None,
            direct_indexed_graphics: None,
            direct_indexed_dictionary: None,
            context_callback_descriptor: None,
            conditional_language_pair: None,
            text_reader_registry: None,
            conditional_substitution_words: None,
            display_control_tables: None,
            compact_primary_alias: None,
            observed_lz: None,
            constructor_template64: None,
            observed_tilemap: None,
            observed_tilemap_descriptor: None,
            observed_tilemap_row: None,
            dma_register_directories: None,
            iwram_memory_helpers: None,
            adjacent_offset_tables: None,
            indexed_records16: None,
            indexed_words_3959: None,
            thumb_switch_tables: None,
            thumb_pc_literals: None,
            copied_halfword_lookups: None,
            indexed_halfwords_39a5: None,
            script_byte_lookup_3ad: None,
            soil_scene_records: None,
            soil_indirect_records: None,
            soil_indexed_bytes: None,
            soil_compact_record: None,
            soil_scoped_values: None,
            soil_remaining_rectangles: None,
            firaga_copied_matrices: None,
            firaga_compact_cells: None,
            firaga_indexed_obj_bank: None,
            firaga_compact_obj_bank: None,
            firaga_adjacent_compact_obj_bank: None,
            firaga_6c12c8_obj_bank: None,
            firaga_transfer_program: None,
            firaga_palette_rows: None,
            firaga_stack_initialization: None,
            firaga_queued_transfer: None,
            white_flowers_signed_bytes: None,
            white_flowers_isolated_values: None,
            tilemap_transfer_directory: None,
            indexed_tilemap_controls: None,
            tilemap_stream_bank: None,
            tilemap_transfer_lz77: None,
            tilemap_type3: None,
            tilemap_type2: None,
            tilemap_direct: None,
            compact_cell_bank: None,
            background_effect_graphics: None,
            localized_obj_animation_banks: None,
            background_screen_banks: None,
            physical_audio_banks: None,
            physical_compact_obj_region: None,
            screen_update_tables: None,
            counted_effect_tables: None,
            map_raster_palettes: None,
            auxiliary_commands: None,
            auxiliary_archive: None,
            font_bank: None,
                })
                .is_err());
        }
        assert!(extraction
            .validate_files(BaseFiles {
                manifest: &manifest,
                dimensions: (&extraction.dimensions, b"other"),
                background: None,
                palettes: None,
                alternate_palettes: None,
                packet_programs: None,
                main_graphics: None,
                transfers: None,
            short_texts: None,
            text_catalogs: None,
            name_keyboard: None,
            obj_lists: None,
            obj_animations: None,
            obj_graphics: None,
            ui_palettes: None,
            ui_graphics: None,
            items: None,
            item_links: None,
            metadata16: None,
            metadata52: None,
            metadata4: None,
            metadata2: None,
            default48: None,
            constructor_alternates: None,
            metadata14: None,
            metadata12: None,
            global_pair_flags: None,
            evaluation_parameters: None,
            progression_rewards: None,
            progression_gates: None,
            progression_selection_gates: None,
            aggregate_flag_keys: None,
            trade_parameters: None,
            item_effect_parameters: None,
            audio_directory: None,
            audio_tones: None,
            audio_sequences: None,
            audio_driver: None,
            palette_effect_dispatch: None,
            palette_programs: None,
            blend_programs: None,
            task_descriptors: None,
            effect_event_directories: None,
            script_commands: None,
            script_archive: None,
            script_text: None,
            script_coordinates: None,
            ui_choice_lists: None,
            unit_obj_initial_pose: None,
            color_mode_palettes: None,
            unit_obj_equipment: None,
            unit_obj_normal_battle: None,
            unit_obj_alternate_battle: None,
            unit_obj_script_special_normal: None,
            unit_obj_script_special_alternate: None,
            unit_obj_script_override: None,
            unit_obj_script_registry_special: None,
            unit_obj_script_registry_generic: None,
            unit_obj_property_words: None,
            unit_obj_byte_property_words: None,
            unit_obj_ffff_reader_fields: None,
            slot_job_lookup: None,
            observed_task_parameters: None,
            constructor_obj_animation: None,
            task_cycle_deltas: None,
            word_task_obj_animations: None,
            world_nodes: None,
            world_roads: None,
            encounter_headers: None,
            encounter_unit_templates: None,
            encounter_coordinates: None,
            metadata28: None,
            records4: None,
            conditional_records8: None,
            mission_records70: None,
            selection_byte_tables: None,
            event_records12: None,
            parameter_records27: None,
            world_records38: None,
            indexed_graphics: None,
            ui_raw_transfers: None,
            dynamic_raw_fragments: None,
            conditional_four_byte_lookup: None,
            conditional_packed_lookup: None,
            conditional_descriptor_banks: None,
            conditional_descriptor_bank_zero: None,
            conditional_property_masks: None,
            compact_obj_animations: None,
            secondary_compact_obj_animations: None,
            tertiary_compact_obj_animations: None,
            quaternary_compact_obj_animations: None,
            quinary_compact_obj_animations: None,
            senary_compact_obj_animations: None,
            septenary_compact_obj_animations: None,
            octonary_compact_obj_animations: None,
            ui_context_templates: None,
            compact_obj_catalog: None,
            layered_obj_animations: None,
            audio_extended_dispatch: None,
            conditional_item_parameters: None,
            range_lookup: None,
            tilemap_geometry: None,
            script39_registry_poses: None,
            script39_special_poses: None,
            direct_indexed_graphics: None,
            direct_indexed_dictionary: None,
            context_callback_descriptor: None,
            conditional_language_pair: None,
            text_reader_registry: None,
            conditional_substitution_words: None,
            display_control_tables: None,
            compact_primary_alias: None,
            observed_lz: None,
            constructor_template64: None,
            observed_tilemap: None,
            observed_tilemap_descriptor: None,
            observed_tilemap_row: None,
            dma_register_directories: None,
            iwram_memory_helpers: None,
            adjacent_offset_tables: None,
            indexed_records16: None,
            indexed_words_3959: None,
            thumb_switch_tables: None,
            thumb_pc_literals: None,
            copied_halfword_lookups: None,
            indexed_halfwords_39a5: None,
            script_byte_lookup_3ad: None,
            soil_scene_records: None,
            soil_indirect_records: None,
            soil_indexed_bytes: None,
            soil_compact_record: None,
            soil_scoped_values: None,
            soil_remaining_rectangles: None,
            firaga_copied_matrices: None,
            firaga_compact_cells: None,
            firaga_indexed_obj_bank: None,
            firaga_compact_obj_bank: None,
            firaga_adjacent_compact_obj_bank: None,
            firaga_6c12c8_obj_bank: None,
            firaga_transfer_program: None,
            firaga_palette_rows: None,
            firaga_stack_initialization: None,
            firaga_queued_transfer: None,
            white_flowers_signed_bytes: None,
            white_flowers_isolated_values: None,
            tilemap_transfer_directory: None,
            indexed_tilemap_controls: None,
            tilemap_stream_bank: None,
            tilemap_transfer_lz77: None,
            tilemap_type3: None,
            tilemap_type2: None,
            tilemap_direct: None,
            compact_cell_bank: None,
            background_effect_graphics: None,
            localized_obj_animation_banks: None,
            background_screen_banks: None,
            physical_audio_banks: None,
            physical_compact_obj_region: None,
            screen_update_tables: None,
            counted_effect_tables: None,
            map_raster_palettes: None,
            auxiliary_commands: None,
            auxiliary_archive: None,
            font_bank: None,
            })
            .is_err());
        rom[48] ^= 1;
        assert!(Extraction::from_ranges(&rom, "test".into(), "test", &ranges, &hash).is_err());
        assert!(
            Extraction::from_ranges(&bytes[..47], "test".into(), "test", &ranges, &hash).is_err()
        );
        assert!(
            Extraction::from_ranges(&rom, "test".into(), "test", &[(u32::MAX, "bad")], &hash)
                .is_err()
        );
        assert!(Extraction::from_rom(&rom).is_err());
    }

    #[test]
    #[ignore = "requires the user's exact USA and EUR ROMs; no game data is bundled"]
    fn regional_extractions_are_independent_and_roundtrip() {
        for (env, version, ranges) in [
            ("FFTA_TEST_ROM_USA", Version::Usa, 2),
            ("FFTA_TEST_ROM_EUR", Version::Europe, 4),
        ] {
            let rom = std::fs::read(std::env::var(env).unwrap()).unwrap();
            let extraction = Extraction::from_rom(&rom).unwrap();
            assert_eq!(extraction.version(), version);
            assert_eq!(extraction.manifest.resources[0].sources.len(), ranges);
            for source in &extraction.manifest.resources[0].sources[..2] {
                assert_eq!(
                    &rom[source.offset as usize..source.offset as usize + 48],
                    &extraction.dimensions.source_bytes()
                );
            }
            if version == Version::Europe {
                let projections = obj_derived_dimensions::projections(&extraction.dimensions);
                for (source, bytes) in extraction.manifest.resources[0].sources[2..].iter().zip(projections) {
                    assert_eq!(&rom[source.offset as usize..source.offset as usize + 48], &bytes);
                }
            }
            let mut changed = rom;
            changed[0] ^= 1;
            assert!(Extraction::from_rom(&changed).is_err());
        }
    }
}
