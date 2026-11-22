//! EUR Firaga queued transfer controls: authenticated publication and original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::firaga_queued_transfer as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, pc: u32, register: Option<(usize, u32)>) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    if let Some((r, value)) = register {
        machine.cpu.regs[r] = value;
    }
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}
fn poison(machine: &mut Machine, rom: &[u8], spans: &[(u32, Vec<u8>, &'static str)], enable: bool) {
    for (offset, bytes, _) in spans {
        for at in *offset as usize..*offset as usize + bytes.len() {
            machine.bus.rom[at] = if enable { !rom[at] } else { rom[at] };
        }
    }
}
#[test]
#[ignore = "exact EUR ROM and E260 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_queued_transfer_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_QUEUED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_QUEUED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (260, 259, 145)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (261, 260, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior, "prior resource {}", prior.id);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let data = &source.firaga_queued_transfer.as_ref().unwrap().decoded;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 3);
    assert_eq!(spans.iter().map(|(_, b, _)| b.len()).sum::<usize>(), 264);
    for (offset, bytes, _) in &spans {
        assert_eq!(
            *bytes,
            rom[*offset as usize..*offset as usize + bytes.len()]
        );
    }
    assert_eq!(
        parse_firaga_queued_transfer(
            &fs::read(prepared.directory.join(FIRAGA_QUEUED_TRANSFER_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut forged = data.clone();
    forged.halfwords[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_firaga_queued_transfer(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed_manifest = dataset.manifest.clone();
    changed_manifest
        .resources
        .iter_mut()
        .find(|r| r.id == FIRAGA_QUEUED_TRANSFER_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &changed_manifest;
    files.firaga_queued_transfer = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    raw.bus.iwram[0x5e78..0x5f28].copy_from_slice(&rom[0xd96838..0xd968e8]);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    poison(&mut managed, &rom, &spans, true);
    for (offset, bytes, _) in &spans {
        for at in *offset..*offset + bytes.len() as u32 {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]);
            }
        }
    }
    assert!(owner
        .firaga_queued_transfer_stats()
        .unwrap()
        .source_bytes
        .iter()
        .all(|&n| n > 0));
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for (offset, bytes, _) in &spans {
        let start = *offset as usize;
        for edge in [start, start + bytes.len()] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for (offset, bytes, _) in &spans {
        let start = *offset as usize;
        for edge in [start, start + bytes.len()] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    if changed.contains(&(at + n)) {
                        rom[at + n] ^ 0x5a
                    } else {
                        rom[at + n]
                    }
                };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                    assert_eq!(
                        (RT_API.read16)(ptr, mirror + at as u32),
                        u32::from(u16::from_le_bytes([expected(0), expected(1)])),
                        "read16 {at:x}"
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]),
                        "read32 {at:x}"
                    );
                }
            }
        }
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    poison(&mut managed, &rom, &spans, false);
    let cases: [(u32, Option<(usize, u32)>); 2] = [(0xac950, None), (0xac954, None)];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut reader_cases = 0;
    for group in 0..2 {
        let addresses: BTreeSet<_> = cases
            .iter()
            .filter(|(pc, _)| (pc / 2) % 2 == group)
            .map(|(pc, _)| 0x0800_0000 + pc)
            .collect();
        if addresses.is_empty() {
            continue;
        }
        let blocks = addresses
            .iter()
            .map(|&addr| Block {
                addr,
                thumb: true,
                instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
            })
            .collect();
        let object = root.join(format!("firaga-queued-native-{group}.o"));
        recomp_core::tcc_memory::build(
            &Analysis { blocks },
            &view,
            &recomp_core::tcc_memory::library_path().unwrap(),
            &object,
            &mut |_| {},
        )
        .unwrap();
        let (_library, table) = crate::load_native(&object).unwrap();
        for native in [false, true] {
            for &(pc, register) in cases.iter().filter(|(pc, _)| (pc / 2) % 2 == group) {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, pc, register);
                }
                poison(&mut managed, &rom, &spans, true);
                let before = owner.firaga_queued_transfer_stats().unwrap().bus_reads;
                raw.bus.frame_ready = false;
                raw.step();
                managed.bus.frame_ready = false;
                if native {
                    table.get((0x0800_0000 + pc) | 1).unwrap()(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
                assert!(
                    owner.firaga_queued_transfer_stats().unwrap().bus_reads > before,
                    "reader {pc:x}"
                );
                poison(&mut managed, &rom, &spans, false);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "reader {pc:x} native={native}"
                );
                reader_cases += 1;
            }
        }
    }
    assert_eq!(reader_cases, 4);
    // Execute the original producer prefix, queue helper and DMA executor.
    let node = 0x0203_0000u32;
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        setup(machine, 0xac94e, None);
        machine.cpu.regs[0] = node;
        machine.cpu.regs[6] = 5;
    }
    poison(&mut managed, &rom, &spans, true);
    let before = owner.firaga_queued_transfer_stats().unwrap().source_bytes;
    for machine in [&mut raw, &mut managed] {
        let mut steps = 0;
        while machine.cpu.regs[15] != 0x080a_c95c {
            assert!(
                steps < 10000,
                "producer escaped at {:08x}",
                machine.cpu.regs[15]
            );
            machine.bus.frame_ready = false;
            machine.step();
            steps += 1;
        }
        assert_eq!(machine.bus.read32(0x0300_2bcc), node);
        assert_eq!(machine.bus.read32(node + 4), 0x0840_e844);
        assert_eq!(machine.bus.read32(node + 8), 0x0601_3700);
        assert_eq!(machine.bus.read32(node + 12), 0x8000_0080);
        setup(machine, 0x800, None);
        machine.cpu.regs[13] = 0x0300_7d00;
        machine.cpu.regs[14] = 0x0300_7f01;
        let mut steps = 0;
        while machine.cpu.regs[15] != 0x0300_7f00 {
            assert!(
                steps < 100000,
                "queue escaped at {:08x}",
                machine.cpu.regs[15]
            );
            machine.bus.frame_ready = false;
            machine.step();
            steps += 1;
        }
        assert_eq!(
            &machine.bus.vram[0x13700..0x13800],
            &rom[0x40e844..0x40e944]
        );
    }
    let after = owner.firaga_queued_transfer_stats().unwrap().source_bytes;
    assert!(
        after.iter().zip(before).all(|(a, b)| *a > b),
        "producer and DMA must consume all source groups"
    );
    poison(&mut managed, &rom, &spans, false);
    assert_eq!(
        raw.save_state().unwrap(),
        managed.save_state().unwrap(),
        "producer and queued DMA"
    );
    let queue_cases = 1;
    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let offset = model::SOURCE_OFFSETS[2];
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 8);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + offset, 0x0203_2400, 8]);
                machine.cpu.regs[15] = 0x0200_0000;
                machine.cpu.cpsr = 0xbf;
                machine
                    .bus
                    .write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        poison(&mut dma_managed, &rom, &spans, true);
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .firaga_queued_transfer_stats()
            .unwrap()
            .bus_reads;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .firaga_queued_transfer_stats()
                .unwrap()
                .bus_reads
                > before
        );
        poison(&mut dma_managed, &rom, &spans, false);
        assert_eq!(
            dma_raw.save_state().unwrap(),
            dma_managed.save_state().unwrap(),
            "transfer {kind}"
        );
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[261,260],"resources":146,"source_bytes":264,"prior_resources_unchanged":145,
        "mixed_boundaries_and_mirrors":true,"original_interpreter_generated_reader_cases":reader_cases,"original_producer_queue_dma_cases":queue_cases,
        "dma_cpuset_cpufastset_cases":3,"full_state_clock_equal":true,"coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true,"Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
