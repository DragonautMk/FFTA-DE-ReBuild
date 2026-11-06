//! EUR managed dispatch fragments; the original callback itself is not executed.
use super::*;
use gba_core::{capi::RT_API, rom_data::audio_extended_dispatch::*, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(m: &mut Machine, selector: u8) {
    m.bus.ewram[0x140..0x144].copy_from_slice(&0x0200_0300u32.to_le_bytes());
    m.bus.ewram[0x300] = selector;
    m.cpu.regs = [0; 16];
    m.cpu.regs[1] = 0x0200_0100;
    m.cpu.regs[13] = 0x0300_7d00;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = 0x0814_9e8a;
}

#[test]
#[ignore = "private exact EUR ROM, E130 dataset and TinyCC; Windows executable under Wine"]
fn european_audio_extended_dispatch_publication_and_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_AUDIO_EXTENDED_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Manifest =
        toml::from_str(&fs::read_to_string(old_dir.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (130, 129, 101)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (131, 130, 102)
    );
    let managed_root = root.join("resources");
    let created = preparation::prepare(&managed_root, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(created.action, "created");
    assert!(created
        .directory
        .parent()
        .unwrap()
        .ends_with("v1/e131-d130"));
    assert_eq!(
        preparation::prepare(&managed_root, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    verify(&created.directory, &source).unwrap();
    assert_eq!(fs::read_dir(&created.directory).unwrap().count(), 103);
    assert_eq!(&source.manifest.coverage[..101], old.coverage.as_slice());
    for (current, previous) in source.manifest.resources[..101].iter().zip(&old.resources) {
        assert_eq!(current, previous);
        assert_eq!(
            fs::read(created.directory.join(&current.path)).unwrap(),
            fs::read(old_dir.join(&previous.path)).unwrap()
        );
    }
    let model = &source.audio_extended_dispatch.as_ref().unwrap().decoded;
    let spans = model.source_spans().unwrap();
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    setup(&mut trace, 0);
    let mut visited = BTreeSet::new();
    while trace.cpu.regs[15] != 0x0814_9e9a {
        visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
        trace.step();
        assert!(visited.len() <= 8);
    }
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
    let object = root.join("dispatcher-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut interpreted = crate::make_machine(rom.clone(), None);
    let mut translated = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut interpreted, &created.directory).unwrap();
    attach_from_directory(&mut translated, &created.directory).unwrap();
    let owner = translated.bus.rom_resources().unwrap().clone();
    for selector in SELECTORS.into_iter().chain([8, 9, 12, 255]) {
        for machine in [&mut raw, &mut interpreted, &mut translated] {
            machine.load_state(&initial).unwrap();
            setup(machine, selector);
        }
        for machine in [&mut interpreted, &mut translated] {
            for (offset, bytes, _) in &spans {
                machine.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd);
            }
        }
        while raw.cpu.regs[15] != 0x0814_9e9a {
            raw.step();
        }
        while interpreted.cpu.regs[15] != 0x0814_9e9a {
            interpreted.step();
        }
        let mut count = 0;
        while translated.cpu.regs[15] != 0x0814_9e9a {
            table
                .get(translated.cpu.regs[15] | u32::from(translated.cpu.thumb()))
                .unwrap()(
                &RT_API,
                &mut translated as *mut Machine as *mut std::ffi::c_void,
            );
            count += 1;
            assert!(count <= 8);
        }
        for machine in [&mut interpreted, &mut translated] {
            for (offset, bytes, _) in &spans {
                machine.bus.rom[*offset as usize..*offset as usize + bytes.len()]
                    .copy_from_slice(bytes);
            }
        }
        assert_eq!(
            raw.save_state().unwrap(),
            interpreted.save_state().unwrap(),
            "interpreter selector {selector}"
        );
        assert_eq!(
            raw.save_state().unwrap(),
            translated.save_state().unwrap(),
            "native selector {selector}"
        );
    }
    for (offset, bytes, _) in &spans {
        translated.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd);
    }
    for start in [TABLE_OFFSET + 30, TABLE_OFFSET + 38] {
        let value = owner
            .audio_host_read::<4>(start as usize, &translated.bus.rom)
            .unwrap();
        assert_eq!(value, rom[start as usize..start as usize + 4]);
    }
    for (offset, bytes, _) in &spans {
        translated.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes);
    }
    let stats = owner.audio_extended_dispatch_stats().unwrap();
    assert_eq!(
        stats.selector_mask,
        SELECTORS.into_iter().fold(0u64, |v, n| v | 1 << n)
    );
    assert!(stats.source_bytes >= 40 && stats.host_source_bytes == 4);
    let before = translated.save_state().unwrap();
    let bytes = fs::read(created.directory.join(AUDIO_EXTENDED_DISPATCH_FILE)).unwrap();
    let manifest_bytes = fs::read(created.directory.join(MANIFEST_FILE)).unwrap();
    let mut altered = model.clone();
    altered.entries[0].callback ^= 2;
    let bad = encode_audio_extended_dispatch(&altered).unwrap();
    let mut bad_manifest = source.manifest.clone();
    bad_manifest.resources[101].sha256 = gba_core::ffta_rom::digest(&bad);
    fs::write(created.directory.join(AUDIO_EXTENDED_DISPATCH_FILE), bad).unwrap();
    fs::write(
        created.directory.join(MANIFEST_FILE),
        toml::to_string_pretty(&bad_manifest).unwrap(),
    )
    .unwrap();
    assert!(attach_from_directory(&mut translated, &created.directory).is_err());
    assert!(Arc::ptr_eq(translated.bus.rom_resources().unwrap(), &owner));
    assert_eq!(translated.save_state().unwrap(), before);
    fs::write(created.directory.join(AUDIO_EXTENDED_DISPATCH_FILE), bytes).unwrap();
    fs::write(created.directory.join(MANIFEST_FILE), manifest_bytes).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[131,130],"resources":102,"previous_unchanged":101,"prefix_cases":14,"raw_interpreter_and_translated_managed_full_state":true,"mixed_host_boundaries":2,"coherent_tamper_transaction":true,"callbacks_executed":false,"ordinary_producer":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM and E131 dataset; Windows executable under Wine"]
fn european_audio_extended_dispatch_guest_transfers_and_boundaries() {
    use gba_core::Bus;
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let report = PathBuf::from(std::env::var_os("FFTA_AUDIO_EXTENDED_TRANSFER_REPORT").unwrap());
    let source = Extraction::from_rom(&rom).unwrap();
    verify(&dataset, &source).unwrap();
    let model = &source.audio_extended_dispatch.as_ref().unwrap().decoded;
    let spans = model.source_spans().unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    attach_from_directory(&mut decoded, &dataset).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let poison = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()].fill(0xcd);
        }
        // Poison the separately owned slots 8/9 as well, so a mixed read must
        // compose both resource owners rather than silently using the ROM.
        let at = TABLE_OFFSET as usize + 32;
        m.bus.rom[at..at + 8].fill(0xcd);
    };
    let restore = |m: &mut Machine| {
        let start = TABLE_OFFSET as usize;
        m.bus.rom[start..start + 48].copy_from_slice(&rom[start..start + 48]);
    };
    poison(&mut decoded);
    let mut checked = 0;
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for boundary in [
            TABLE_OFFSET,
            TABLE_OFFSET + 32,
            TABLE_OFFSET + 40,
            TABLE_OFFSET + 48,
        ] {
            for offset in boundary - 3..boundary + 4 {
                for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                    assert_eq!(decoded.bus.read8(mirror + offset), rom[offset as usize]);
                    assert_eq!(
                        decoded.bus.read16(mirror + offset),
                        u16::from_le_bytes(
                            rom[offset as usize..offset as usize + 2]
                                .try_into()
                                .unwrap()
                        )
                    );
                    assert_eq!(
                        (RT_API.read32)(
                            &mut decoded as *mut Machine as *mut std::ffi::c_void,
                            mirror + offset
                        ),
                        u32::from_le_bytes(
                            rom[offset as usize..offset as usize + 4]
                                .try_into()
                                .unwrap()
                        )
                    );
                    checked += 1;
                }
            }
        }
    }
    decoded.bus.real_bios = false;
    let fallback = TABLE_OFFSET as usize + 48;
    decoded.bus.rom[fallback] ^= 0x5a;
    assert_eq!(
        decoded.bus.read8(0x0800_0000 + fallback as u32),
        rom[fallback] ^ 0x5a
    );
    decoded.bus.rom[fallback] = rom[fallback];
    restore(&mut decoded);

    for src in [TABLE_OFFSET + 28, TABLE_OFFSET + 36] {
        for m in [&mut raw, &mut decoded] {
            m.load_state(&initial).unwrap();
            m.bus.write32(0x0400_00d4, 0x0800_0000 + src);
            m.bus.write32(0x0400_00d8, 0x0203_2000);
            m.bus.write16(0x0400_00dc, 2);
        }
        poison(&mut decoded);
        let before_new = owner.audio_extended_dispatch_stats().unwrap().source_bytes;
        let before_old = owner.audio_sequences_stats().unwrap().source_bytes;
        raw.bus.write16(0x0400_00de, 0x8400);
        decoded.bus.write16(0x0400_00de, 0x8400);
        assert_eq!(
            owner.audio_extended_dispatch_stats().unwrap().source_bytes - before_new,
            4
        );
        assert_eq!(
            owner.audio_sequences_stats().unwrap().source_bytes - before_old,
            4
        );
        assert_eq!(
            &decoded.bus.ewram[0x32000..0x32008],
            &rom[src as usize..src as usize + 8]
        );
        restore(&mut decoded);
        assert_eq!(raw.save_state().unwrap(), decoded.save_state().unwrap());
    }
    for src in [TABLE_OFFSET + 28, TABLE_OFFSET + 36] {
        for m in [&mut raw, &mut decoded] {
            m.load_state(&initial).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + src, 0x0203_2400, 4]);
            m.cpu.regs[15] = 0x0200_0000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x0200_0000, 0xdf0b);
        }
        poison(&mut decoded);
        let before_new = owner.audio_extended_dispatch_stats().unwrap().source_bytes;
        let before_old = owner.audio_sequences_stats().unwrap().source_bytes;
        raw.step();
        decoded.step();
        assert_eq!(
            owner.audio_extended_dispatch_stats().unwrap().source_bytes - before_new,
            4
        );
        assert_eq!(
            owner.audio_sequences_stats().unwrap().source_bytes - before_old,
            4
        );
        assert_eq!(
            &decoded.bus.ewram[0x32400..0x32408],
            &rom[src as usize..src as usize + 8]
        );
        restore(&mut decoded);
        assert_eq!(raw.save_state().unwrap(), decoded.save_state().unwrap());
    }
    fs::write(report, serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"E131 audio fragment guest widths, mirrors, two-owner DMA3 and CpuSet boundaries","boundary_mirror_bios_cases":checked,"dma_pairs":2,"cpuset_pairs":2,"full_states_and_clocks_equal":true,"raw_fallback":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
