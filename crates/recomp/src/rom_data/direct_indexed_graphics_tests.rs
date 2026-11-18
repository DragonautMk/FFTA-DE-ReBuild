//! EUR direct indexed-archive publication and original caller/decoder proof.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::direct_indexed_graphics::{
        DirectIndexedGraphics, ARCHIVE_LENGTH, ARCHIVE_OFFSET, CALLER_LITERAL, DECODED_SHA256,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, e164 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_direct_indexed_archive_publication_and_reader() {
    const START: u32 = 0x0802_3c98;
    const STOP: u32 = 0x0802_3ca8;
    const DESTINATION: usize = 0x3cb0;
    let root = PathBuf::from(std::env::var_os("FFTA_DIRECT_INDEXED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_DIRECT_INDEXED_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (164, 163));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (167, 166)
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
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 3);
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
    let catalog = source.direct_indexed_graphics.as_ref().unwrap();
    let model = &catalog.decoded;
    let entry = source
        .manifest
        .resources
        .iter()
        .find(|r| r.id == DIRECT_INDEXED_GRAPHICS_ID)
        .unwrap();
    assert_eq!(entry.sources, catalog.sources);
    assert_eq!(
        (model.archive.programs.len(), model.decoded_bytes().len()),
        (4, 256)
    );
    assert_eq!(
        gba_core::ffta_rom::digest(&model.decoded_bytes()),
        DECODED_SHA256
    );
    let decoded: DirectIndexedGraphics =
        serde_json::from_slice(&serde_json::to_vec(model).unwrap()).unwrap();
    assert_eq!(&decoded, model);
    let mut invalid = decoded.clone();
    invalid.archive.programs[0].decoded_words[0] ^= 1;
    assert!(invalid.validate().is_err());
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 292);
    for (at, bytes, _) in &spans {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
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
    let before = owner.direct_indexed_graphics_stats().unwrap();
    for (at, bytes, _) in &spans {
        for (i, expected) in bytes.iter().enumerate() {
            assert_eq!(managed.bus.read8(0x0800_0000 + *at + i as u32), *expected);
        }
    }
    let after = owner.direct_indexed_graphics_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 292);
    assert_eq!(after.literal_bytes - before.literal_bytes, 4);
    assert_eq!(after.header_bytes - before.header_bytes, 8);
    assert_eq!(after.index_bytes - before.index_bytes, 16);
    assert_eq!(after.token_bytes - before.token_bytes, 4);
    assert_eq!(
        after.decoded_payload_bytes - before.decoded_payload_bytes,
        256
    );
    assert_eq!(
        after.dictionary_header_bytes - before.dictionary_header_bytes,
        4
    );
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [
        CALLER_LITERAL as usize,
        ARCHIVE_OFFSET as usize,
        ARCHIVE_OFFSET as usize + ARCHIVE_LENGTH,
    ] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!(
                (RT_API.read16)(ptr, 0x0a00_0000 + at as u32),
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            );
            assert_eq!(
                (RT_API.read32)(ptr, 0x0800_0000 + at as u32),
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            );
        }
    }
    restore(&mut managed);
    let configure = |m: &mut Machine| {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0x3c);
        m.bus.ffta_sprites.enabled = false;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.frame_ready = false;
        m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        m.cpu.regs[13] = 0x0300_7d00;
        m.cpu.regs[14] = 0x0800_0301;
        m.cpu.regs[15] = START;
        m.cpu.cpsr = 0xbf;
    };
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    configure(&mut trace);
    let mut visited = BTreeSet::new();
    for _ in 0..200000 {
        if trace.cpu.regs[15] == STOP {
            break;
        }
        visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
        trace.bus.frame_ready = false;
        trace.step();
    }
    assert_eq!(trace.cpu.regs[15], STOP, "original direct archive caller");
    assert_eq!(
        &trace.bus.ewram[DESTINATION..DESTINATION + 256],
        model.decoded_bytes()
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&trace.bus.iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let first = if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            };
            Block {
                addr: address,
                thumb,
                instrs: vec![first],
            }
        })
        .collect();
    let object = root.join("direct-indexed-native.o");
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
        let mut raw = crate::make_machine(rom.clone(), None);
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        configure(&mut raw);
        configure(&mut managed);
        poison(&mut managed);
        let before = owner.direct_indexed_graphics_stats().unwrap();
        for _ in 0..200000 {
            if raw.cpu.regs[15] == STOP {
                break;
            }
            raw.bus.frame_ready = false;
            raw.step();
        }
        assert_eq!(raw.cpu.regs[15], STOP);
        let mut steps = 0;
        while managed.cpu.regs[15] != STOP {
            managed.bus.frame_ready = false;
            if native {
                table
                    .get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb()))
                    .expect("compiled archive instruction")(
                    &RT_API,
                    &mut managed as *mut Machine as *mut std::ffi::c_void,
                );
            } else {
                managed.step();
            }
            steps += 1;
            assert!(
                steps < 200000,
                "managed archive PC {:08x}",
                managed.cpu.regs[15]
            );
        }
        let after = owner.direct_indexed_graphics_stats().unwrap();
        assert!(after.decoded_payload_bytes - before.decoded_payload_bytes >= 256);
        assert_eq!(
            &managed.bus.ewram[DESTINATION..DESTINATION + 256],
            model.decoded_bytes()
        );
        restore(&mut managed);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "compiled {native}"
        );
        comparisons += 1;
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR direct indexed archive publication and original caller/decoder",
        "revision":[167,166], "previous_resources_preserved":old.resources.len(),
        "source_bytes":292, "decoded_bytes":256, "decoded_sha256":DECODED_SHA256,
        "source_rom_poisoned":true, "boundaries_and_json_guards":true,
        "original_caller_comparisons":comparisons, "full_state_clock_equal":true,
        "compiled_dispatch":"instruction blocks may use interpreter fallback", "ordinary_reachability":"pending",
        "neighbor_archive":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
