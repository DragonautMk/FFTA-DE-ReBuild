//! EUR conditional substitution-word publication and original reader prefixes.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::conditional_substitution_words::{
        Catalog, ConditionalSubstitutionWords, TABLE_END, TABLE_OFFSET,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};

fn configure(machine: &mut Machine, kind: u8, index: u32) -> u32 {
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = 0x0201_0000;
    machine.cpu.regs[1] = index;
    machine.cpu.regs[3] = 0x0200_1000;
    machine.cpu.regs[4] = index;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = match kind {
        0 => 0x0801_9172,
        1 => 0x0801_9424,
        _ => 0x0801_9e94,
    };
    machine.cpu.cpsr = 0xbf;
    match kind {
        0 => 0x0801_9182,
        1 => 0x0801_9430,
        _ => 0x0801_9ea6,
    }
}

#[test]
#[ignore = "exact EUR ROM, E169 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_conditional_substitution_words_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_WORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_WORDS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (169, 168));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (170, 169)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    assert_eq!(
        source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == CONDITIONAL_SUBSTITUTION_WORDS_ID)
            .unwrap()
            .dependencies,
        vec![TEXT_CATALOGS_ID.to_string()]
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.conditional_substitution_words.as_ref().unwrap();
    let model: ConditionalSubstitutionWords =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    assert_eq!(model.words.len(), 98);
    assert_eq!(model.words.iter().filter(|w| w.word == 0).count(), 93);
    assert_eq!(catalog.sources.len(), 99);
    let mut invalid = model.clone();
    invalid.words[1].index = invalid.words[0].index;
    assert!(invalid.validate().is_err());
    let mut overlap = old.resources.clone();
    let mut partial = catalog.sources[1].clone();
    partial.length = 1;
    overlap[0].sources.push(partial);
    assert!(Catalog::from_rom(&rom, &overlap).is_err());
    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    decoded
        .conditional_substitution_words
        .as_mut()
        .unwrap()
        .words[0]
        .word ^= 1;
    let changed = encode_conditional_substitution_words(
        decoded.conditional_substitution_words.as_ref().unwrap(),
    )
    .unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == CONDITIONAL_SUBSTITUTION_WORDS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.conditional_substitution_words_bytes = Some(changed);
    assert!(source.validate_files(decoded.files()).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let spans = model.source_spans().unwrap();
    let poison = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()].fill(0xcd);
        }
    };
    let restore = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()]
                .copy_from_slice(&rom[*at as usize..*at as usize + bytes.len()]);
        }
    };
    poison(&mut managed);
    let before = owner.conditional_substitution_words_stats().unwrap();
    for (at, bytes, _) in &spans {
        for (i, byte) in bytes.iter().enumerate() {
            assert_eq!(managed.bus.read8(0x0800_0000 + *at + i as u32), *byte);
        }
    }
    let after = owner.conditional_substitution_words_stats().unwrap();
    assert_eq!(
        (
            after.literal_bytes - before.literal_bytes,
            after.word_bytes - before.word_bytes
        ),
        (4, 392)
    );
    // Adjacent earlier owners must combine with new words on mixed-width reads.
    for at in [TABLE_OFFSET as usize, TABLE_END as usize] {
        managed.bus.rom[at..at + 4].fill(0xcd);
    }
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in [
        TABLE_OFFSET as usize + 3,
        TABLE_OFFSET as usize + 4,
        TABLE_OFFSET as usize + 27,
        TABLE_END as usize - 1,
    ] {
        assert_eq!(
            (RT_API.read16)(ptr, 0x0a00_0000 + at as u32),
            u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
        );
        assert_eq!(
            (RT_API.read32)(ptr, 0x0800_0000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    for at in [TABLE_OFFSET as usize, TABLE_END as usize] {
        managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
    }
    restore(&mut managed);
    let initial = raw.save_state().unwrap();
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&raw.bus.iwram),
        bios: None,
    };
    let blocks = [
        (0x0801_9172, 0x0801_9182),
        (0x0801_9424, 0x0801_9430),
        (0x0801_9e94, 0x0801_9ea6),
    ]
    .into_iter()
    .flat_map(|(start, end)| (start..end).step_by(2))
    .map(|addr| Block {
        addr,
        thumb: true,
        instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    })
    .collect();
    let object = root.join("conditional-substitution-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut comparisons = 0;
    for native in [false, true] {
        for kind in 0..3 {
            for index in [0u32, 1, 7, 10, 17, 32, 68, 172, 173, 255, 0x101] {
                let guest_index = if kind < 2 { 60 + index } else { index };
                raw.load_state(&initial).unwrap();
                managed.load_state(&initial).unwrap();
                let stop = configure(&mut raw, kind, guest_index);
                assert_eq!(configure(&mut managed, kind, guest_index), stop);
                poison(&mut managed);
                let before = owner.conditional_substitution_words_stats().unwrap();
                for _ in 0..32 {
                    if raw.cpu.regs[15] == stop {
                        break;
                    }
                    raw.bus.frame_ready = false;
                    raw.step();
                }
                assert_eq!(raw.cpu.regs[15], stop);
                for _ in 0..32 {
                    if managed.cpu.regs[15] == stop {
                        break;
                    }
                    managed.bus.frame_ready = false;
                    if native {
                        table.get(managed.cpu.regs[15] | 1).unwrap()(
                            &RT_API,
                            &mut managed as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        managed.step();
                    }
                }
                assert_eq!(managed.cpu.regs[15], stop);
                let effective = (guest_index & 255) as usize;
                let address = if kind < 2 {
                    0x371068usize + effective * 4
                } else {
                    TABLE_OFFSET as usize + effective * 4
                };
                let selected = model
                    .words
                    .iter()
                    .any(|w| TABLE_OFFSET as usize + usize::from(w.index) * 4 == address);
                let after = owner.conditional_substitution_words_stats().unwrap();
                assert_eq!(
                    after.word_bytes - before.word_bytes,
                    if selected { 4 } else { 0 },
                    "kind {kind}, index {index}"
                );
                assert_eq!(
                    after.literal_bytes - before.literal_bytes,
                    if kind == 2 { 4 } else { 0 }
                );
                restore(&mut managed);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "kind {kind}, index {index}, native {native}"
                );
                comparisons += 1;
            }
        }
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "source_bytes":396,
        "previous_resources_preserved":old.resources.len(), "created_reused_verified":true,
        "modified_word_rejected":true, "duplicate_index_rejected":true, "partial_overlap_rejected":true,
        "all_new_bytes_checked":true, "mixed_width_adjacent_owners_checked":true,
        "original_prefix_comparisons":comparisons, "full_state_clock_equal":true,
        "compiled_dispatch":"instruction blocks may fall back to interpretation", "managed_sources_poisoned":true,
        "guest_index_bound":"unchanged", "callback_invocation":"unproved", "ordinary_indices":"unproved",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR ROM, E170 dataset and TinyCC; Wine only"]
fn european_range_lookup_indirect_registry_reader_domain() {
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    const RANGE: usize = 0x37140c;
    const LEN: usize = 88;
    const REGISTRY: usize = 0x371068;
    assert_eq!(RANGE, REGISTRY + 233 * 4);
    assert_eq!(RANGE + LEN, REGISTRY + 255 * 4);
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = [(0x0801_9172, 0x0801_9182), (0x0801_9424, 0x0801_9430)]
        .into_iter()
        .flat_map(|(start, end)| (start..end).step_by(2))
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("registry-range-readers.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let mut cases = 0u32;
    let mut selected = 0u32;
    let mut boundary = 0u32;
    for native in [false, true] {
        for kind in [0u8, 1] {
            for high in [0u32, 0x100] {
                for index in 232u32..=255 {
                    for m in [&mut raw, &mut managed] {
                        m.load_state(&initial).unwrap();
                        m.bus.ffta_sprites.enabled = false;
                        configure(m, kind, high | index);
                    }
                    let stop = if kind == 0 { 0x0801_9182 } else { 0x0801_9430 };
                    for i in RANGE..RANGE + LEN {
                        managed.bus.rom[i] = rom[i] ^ 0xff;
                    }
                    let before = owner.range_lookup_stats().unwrap().source_bytes;
                    for _ in 0..32 {
                        if raw.cpu.regs[15] == stop {
                            break;
                        }
                        raw.bus.frame_ready = false;
                        raw.step();
                    }
                    assert_eq!(raw.cpu.regs[15], stop);
                    for _ in 0..32 {
                        if managed.cpu.regs[15] == stop {
                            break;
                        }
                        managed.bus.frame_ready = false;
                        if native {
                            table.get(managed.cpu.regs[15] | 1).unwrap()(
                                &RT_API,
                                &mut managed as *mut Machine as *mut std::ffi::c_void,
                            );
                        } else {
                            managed.step();
                        }
                    }
                    assert_eq!(managed.cpu.regs[15], stop);
                    let bytes = owner.range_lookup_stats().unwrap().source_bytes - before;
                    if (233..=254).contains(&index) {
                        assert_eq!(bytes, 4, "kind {kind}, index {index}, native {native}");
                        selected += 1;
                    } else {
                        assert_eq!(bytes, 0, "neighbor index {index}");
                        boundary += 1;
                    }
                    managed.bus.rom[RANGE..RANGE + LEN].copy_from_slice(&rom[RANGE..RANGE + LEN]);
                    assert_eq!(
                        raw.save_state().unwrap(),
                        managed.save_state().unwrap(),
                        "kind {kind}, index {}, native {native}",
                        high | index
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!((cases, selected, boundary), (192, 176, 16));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"two original byte-indexed text registry reader prefixes crossing the E170 bounded-range owner",
        "revision":[170,169], "range_owned_registry_indices":[233,254],
        "neighbor_indices":[232,255], "high_alias":256,
        "controlled_cases":cases, "range_owned_cases":selected, "neighbor_cases":boundary,
        "full_state_clock_equal":true, "all_88_range_source_bytes_poisoned":true,
        "interpreter_and_generated_block_dispatch":true,
        "limits":"Prefixes stop before callback invocation; the selected indices are conditional addresses, not proof of ordinary reachability or safe callback execution. Generated blocks may use interpreter fallback.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR ROM, E170 dataset and TinyCC; Wine only"]
fn european_text_registry_complete_byte_reader_domain() {
    use gba_core::rom_data::read_audit::{Reader, Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );

    const REGISTRY: usize = 0x371068;
    const LENGTH: usize = 256 * 4;
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = [(0x0801_9172, 0x0801_9182), (0x0801_9424, 0x0801_9430)]
        .into_iter()
        .flat_map(|(start, end)| (start..end).step_by(2))
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("complete-registry-readers.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for kind in [0u8, 1] {
            for high in [0u32, 0x100] {
                for index in 0u32..=255 {
                    for m in [&mut raw, &mut managed] {
                        m.load_state(&initial).unwrap();
                        m.bus.ffta_sprites.enabled = false;
                        configure(m, kind, high | index);
                    }
                    for i in REGISTRY..REGISTRY + LENGTH {
                        managed.bus.rom[i] = rom[i] ^ 0xff;
                    }
                    let stop = if kind == 0 { 0x0801_9182 } else { 0x0801_9430 };
                    for _ in 0..32 {
                        if raw.cpu.regs[15] == stop {
                            break;
                        }
                        raw.bus.frame_ready = false;
                        raw.step();
                    }
                    assert_eq!(raw.cpu.regs[15], stop);
                    let trace = TraceCapture::start(
                        TraceFilter {
                            readers: vec![Reader::Bus],
                            ranges: vec![(REGISTRY as u32, (REGISTRY + LENGTH) as u32)],
                        },
                        16,
                    )
                    .unwrap();
                    trace.frame(Some(0));
                    for _ in 0..32 {
                        if managed.cpu.regs[15] == stop {
                            break;
                        }
                        managed.bus.frame_ready = false;
                        if native {
                            table.get(managed.cpu.regs[15] | 1).unwrap()(
                                &RT_API,
                                &mut managed as *mut Machine as *mut std::ffi::c_void,
                            );
                        } else {
                            managed.step();
                        }
                    }
                    assert_eq!(managed.cpu.regs[15], stop);
                    let trace = trace.finish();
                    assert_eq!(trace.dropped_requests, 0);
                    assert!(
                        !trace.events.is_empty(),
                        "no table read: kind {kind}, index {index}, native {native}"
                    );
                    let expected = REGISTRY as u32 + 4 * index;
                    let mut byte_counts = [0u64; 4];
                    for event in trace.events {
                        assert_eq!(event.key.service, Service::Typed);
                        let offset = event.key.read.physical_offset;
                        for byte in offset..offset + event.key.read.length as u32 {
                            assert!(
                                (expected..expected + 4).contains(&byte),
                                "unexpected read {byte:x}"
                            );
                            byte_counts[(byte - expected) as usize] += event.requests;
                        }
                    }
                    assert_eq!(
                        byte_counts, [1; 4],
                        "kind {kind}, index {index}, native {native}"
                    );
                    managed.bus.rom[REGISTRY..REGISTRY + LENGTH]
                        .copy_from_slice(&rom[REGISTRY..REGISTRY + LENGTH]);
                    assert_eq!(
                        raw.save_state().unwrap(),
                        managed.save_state().unwrap(),
                        "kind {kind}, index {}, native {native}",
                        high | index
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 2048);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"two original registry reader prefixes over the entire byte index domain",
        "revision":[170,169], "indices":[0,255], "high_alias":256, "controlled_cases":cases,
        "full_state_clock_equal":true, "all_1024_table_bytes_inverted":true,
        "each_selected_word_typed_once":true, "trace_dropped_requests":0,
        "interpreter_and_generated_block_dispatch":true,
        "limits":"Prefixes stop before callback invocation. Values are conditional inputs, not ordinary reachability; generated blocks may fall back to interpretation.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR ROM, E170 dataset and TinyCC; Wine only"]
fn european_range_lookup_indirect_substitution_dispatcher_domain() {
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    const RANGE: usize = 0x37140c;
    const LEN: usize = 88;
    const SUBSTITUTION: usize = 0x371158;
    assert_eq!(RANGE, SUBSTITUTION + 173 * 4);
    assert_eq!(RANGE + LEN, SUBSTITUTION + 195 * 4);
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = [(0x0801_9e94, 0x0801_9ea6)]
        .into_iter()
        .flat_map(|(start, end)| (start..end).step_by(2))
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("substitution-range-reader.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let mut cases = 0u32;
    let mut selected = 0u32;
    let mut boundary = 0u32;
    for native in [false, true] {
        for kind in [2u8] {
            for high in [0u32, 0x100] {
                for index in 172u32..=195 {
                    for m in [&mut raw, &mut managed] {
                        m.load_state(&initial).unwrap();
                        m.bus.ffta_sprites.enabled = false;
                        configure(m, kind, high | index);
                    }
                    let stop = 0x0801_9ea6;
                    for i in RANGE..RANGE + LEN {
                        managed.bus.rom[i] = rom[i] ^ 0xff;
                    }
                    let before = owner.range_lookup_stats().unwrap().source_bytes;
                    for _ in 0..32 {
                        if raw.cpu.regs[15] == stop {
                            break;
                        }
                        raw.bus.frame_ready = false;
                        raw.step();
                    }
                    assert_eq!(raw.cpu.regs[15], stop);
                    for _ in 0..32 {
                        if managed.cpu.regs[15] == stop {
                            break;
                        }
                        managed.bus.frame_ready = false;
                        if native {
                            table.get(managed.cpu.regs[15] | 1).unwrap()(
                                &RT_API,
                                &mut managed as *mut Machine as *mut std::ffi::c_void,
                            );
                        } else {
                            managed.step();
                        }
                    }
                    assert_eq!(managed.cpu.regs[15], stop);
                    let bytes = owner.range_lookup_stats().unwrap().source_bytes - before;
                    if (173..=194).contains(&index) {
                        assert_eq!(bytes, 4, "kind {kind}, index {index}, native {native}");
                        selected += 1;
                    } else {
                        assert_eq!(bytes, 0, "neighbor index {index}");
                        boundary += 1;
                    }
                    managed.bus.rom[RANGE..RANGE + LEN].copy_from_slice(&rom[RANGE..RANGE + LEN]);
                    assert_eq!(
                        raw.save_state().unwrap(),
                        managed.save_state().unwrap(),
                        "kind {kind}, index {}, native {native}",
                        high | index
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!((cases, selected, boundary), (96, 88, 8));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"original byte-indexed substitution dispatcher prefix crossing the E170 bounded-range owner",
        "revision":[170,169], "range_owned_dispatcher_indices":[173,194],
        "neighbor_indices":[172,195], "high_alias":256,
        "controlled_cases":cases, "range_owned_cases":selected, "neighbor_cases":boundary,
        "full_state_clock_equal":true, "all_88_range_source_bytes_poisoned":true,
        "interpreter_and_generated_block_dispatch":true,
        "limits":"Prefixes stop before callback invocation; the selected indices are conditional addresses, not proof of ordinary reachability or safe callback execution. Generated blocks may use interpreter fallback.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
