//! Feature-gated address inventory for bus and selected host read paths.
use gba_core::{
    rom_data::{
        read_audit::{Capture, Reader},
        Manifest,
    },
    Bus, Machine,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn probe_capture_and_instruction_fetch(rom: &[u8]) {
    let mut machine = crate::make_machine(rom.to_vec(), None);
    assert_eq!(
        u32::from_le_bytes(rom[..4].try_into().unwrap()) & 0xff00_0000,
        0xea00_0000,
        "actual original ARM entry branch"
    );
    assert_eq!(
        u16::from_le_bytes(rom[0xd0650..0xd0652].try_into().unwrap()),
        0xb500,
        "actual original Thumb push"
    );
    for cached in [false, true] {
        for thumb in [false, true] {
            machine.cpu.regs[15] = if thumb { 0x080d0650 } else { 0x08000000 };
            machine.cpu.regs[13] = 0x03007e00;
            machine.cpu.cpsr = if thumb { 0x3f } else { 0x1f };
            let capture = Capture::start();
            if cached {
                let mut cache = gba_core::exec::DecodeCache::default();
                gba_core::exec::step_cached(&mut machine.cpu, &mut machine.bus, &mut cache);
            } else {
                gba_core::exec::step(&mut machine.cpu, &mut machine.bus);
            }
            assert!(capture.finish().is_empty(), "instruction fetch is not data");
        }
    }
    let capture = Capture::start();
    assert_eq!(machine.bus.read8(0x08000000), rom[0]);
    assert_eq!(
        machine.bus.read16(0x08000000),
        u16::from_le_bytes(rom[..2].try_into().unwrap())
    );
    assert_eq!(
        machine.bus.read32(0x08000000),
        u32::from_le_bytes(rom[..4].try_into().unwrap())
    );
    assert_eq!(machine.bus.read8(0x0a000000), rom[0]);
    assert_eq!(machine.bus.read8(0x0c000000), rom[0]);
    let view = gba_core::mp2k::MemView {
        rom: &machine.bus.rom,
        ewram: &machine.bus.ewram,
        iwram: &machine.bus.iwram,
        resources: None,
    };
    assert_eq!(view.u8(0x08000000), Some(rom[0]));
    let reads = capture.finish();
    assert_eq!(reads.len(), 6);
    assert_eq!(
        reads
            .iter()
            .filter(|read| read.reader == Reader::Bus)
            .count(),
        5
    );
    assert_eq!(
        reads
            .iter()
            .filter(|read| read.reader == Reader::AudioValue)
            .count(),
        1
    );
    assert!(reads.iter().all(|read| read.physical_offset == 0));
    let capture = Capture::start();
    machine.bus.read8(0x08000000);
    drop(capture);
    machine.bus.read8(0x08000000);
    assert!(
        Capture::start().finish().is_empty(),
        "finish/drop reset activation"
    );
}

fn owned_ranges(manifest: &Manifest) -> Vec<(u32, u32)> {
    let mut ranges: Vec<_> = manifest
        .resources
        .iter()
        .flat_map(|r| &r.sources)
        .map(|source| {
            let end = source.offset.checked_add(source.length).unwrap();
            assert!(end <= manifest.source_rom_size);
            (source.offset, end)
        })
        .filter(|&(start, end)| start < end)
        .collect();
    ranges.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = merged.last_mut().filter(|last| start <= last.1) {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

fn clear_audio(machine: &mut Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.hle_tap.clear();
    machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
}

#[test]
#[ignore = "requires rom-data-read-audit and private EUR/native/state/input/dataset fixtures with fresh report path"]
fn bus_and_host_rom_reads_preserve_bounded_scene() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_ROM_READ_AUDIT_REPORT");
    assert!(!report.exists(), "preserve earlier inventory reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    probe_capture_and_instruction_fetch(&rom);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!(manifest.source_rom_sha256, gba_core::ffta_rom::digest(&rom));
    assert_eq!(manifest.source_rom_size as usize, rom.len());
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let original = Box::new(table);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut reference = crate::make_machine(rom.clone(), None);
    let mut observed = crate::make_machine(rom, None);
    for machine in [&mut reference, &mut observed] {
        super::attach_from_directory(machine, &dataset).unwrap();
        crate::save_state::load(machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        clear_audio(machine);
    }
    assert!(reference.save_state().unwrap() == observed.save_state().unwrap());
    let mut reads = BTreeSet::new();
    let first = reference.bus.frames;
    let frames = 12;
    let mut dispatch = [0u64; 2];
    let mut audio = [0u64; 5];
    for relative in 0..frames {
        let mut counts = [(0, 0); 2];
        for (index, (machine, table)) in [
            (&mut reference, original.as_ref()),
            (&mut observed, original.as_ref()),
        ]
        .into_iter()
        .enumerate()
        {
            let capture = (index == 1).then(Capture::start);
            machine.bus.keys = input.keys_at(relative);
            input.apply_mouse(&mut machine.bus, relative).unwrap();
            let pointer = machine as *mut Machine as *mut std::ffi::c_void;
            counts[index] =
                crate::run_frame_native(machine, table, pointer, crate::FRAME_STEP_GUARD);
            if let Some(capture) = capture {
                reads.extend(capture.finish());
            }
            assert_eq!(machine.bus.frames, first + relative + 1, "incomplete frame");
            assert!(machine.bus.frame_ready);
        }
        assert_eq!(counts[0], counts[1], "dispatch at {relative}");
        dispatch[0] += counts[0].0;
        dispatch[1] += counts[0].1;
        assert_eq!(
            reference.bus.framebuffer, observed.bus.framebuffer,
            "central at {relative}"
        );
        assert_eq!(
            reference.bus.wide_framebuffer, observed.bus.wide_framebuffer,
            "wide at {relative}"
        );
        assert_eq!(reference.bus.oam, observed.bus.oam, "OAM at {relative}");
        assert_eq!(
            reference.bus.audio_buf, observed.bus.audio_buf,
            "mixed at {relative}"
        );
        assert_eq!(
            reference.bus.psg_tap, observed.bus.psg_tap,
            "PSG at {relative}"
        );
        assert_eq!(
            reference.bus.fifo_tap, observed.bus.fifo_tap,
            "FIFO at {relative}"
        );
        assert_eq!(reference.bus.hle_tap.len(), observed.bus.hle_tap.len());
        assert!(
            reference
                .bus
                .hle_tap
                .iter()
                .zip(&observed.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {relative}"
        );
        assert!(
            reference.save_state().unwrap() == observed.save_state().unwrap(),
            "full state/clock at {relative}"
        );
        for (sum, length) in audio.iter_mut().zip([
            observed.bus.audio_buf.len(),
            observed.bus.psg_tap.len(),
            observed.bus.fifo_tap[0].len(),
            observed.bus.fifo_tap[1].len(),
            observed.bus.hle_tap.len(),
        ]) {
            *sum += length as u64;
        }
        clear_audio(&mut reference);
        clear_audio(&mut observed);
    }
    assert!(dispatch[0] > 0 && audio.iter().all(|&length| length > 0));
    assert!(
        reads.iter().any(|read| read.reader == Reader::Bus),
        "actual bus reads"
    );
    assert!(
        reads.iter().any(|read| read.reader == Reader::AudioValue),
        "actual host audio reads"
    );
    let owned = owned_ranges(&manifest);
    let mut image_bytes = BTreeSet::new();
    let mut unowned_bytes = BTreeSet::new();
    let mut unowned_reads = Vec::new();
    let mut readers = std::collections::BTreeMap::new();
    for read in &reads {
        let end = read
            .physical_offset
            .checked_add(u32::try_from(read.length).unwrap())
            .unwrap();
        assert!(
            end <= manifest.source_rom_size,
            "successful physical-image reads only"
        );
        *readers.entry(read.reader).or_insert(0usize) += 1;
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for off in read.physical_offset..end {
            image_bytes.insert(off);
            let index = owned.partition_point(|&(_, end)| end <= off);
            if !owned.get(index).is_some_and(|&(start, _)| start <= off) {
                unowned_bytes.insert(off);
                if let Some(last) = ranges.last_mut().filter(|last| last.1 == off) {
                    last.1 += 1;
                } else {
                    ranges.push((off, off + 1));
                }
            }
        }
        if !ranges.is_empty() {
            unowned_reads.push(serde_json::json!({"read":read,"unowned_ranges":ranges}));
        }
    }
    let value = serde_json::json!({
        "schema":1, "frames":frames, "first_guest_frame":first, "last_guest_frame":observed.bus.frames,
        "source_rom_sha256":manifest.source_rom_sha256, "resource_count":manifest.resources.len(),
        "extractor_revision":manifest.extractor_revision,"decoder_revision":manifest.decoder_revision,
        "merged_owned_ranges":owned.len(), "unique_reads":reads.len(), "reader_unique_requests":readers,
        "unique_image_bytes":image_bytes.len(), "unique_unowned_bytes":unowned_bytes.len(),
        "dispatch_coverage_only":dispatch,"audio_samples":audio,
        "per_frame_full_state_clock_buffers_dispatch_audio_equal":true,
        "instruction_fetch_and_capture_lifecycle_probes":"PASS",
        "reads":reads,"unowned_reads":unowned_reads,"performance_measurements":false,
        "scope":"resolved physical-ROM bus reads including interpreter data/BIOS/DMA/native APIs plus selected audio/sprite/table/keyboard host access points; interpreter instruction fetch suppressed",
        "blind_spots":["other scenes/threads", "unselected direct host readers", "typed HLE fast paths bypassing bus (already managed)", "folded native constants", "code/signature authentication"],
        "context_limit":"reader tag identifies instrumented access point only, not exact guest instruction or semantic consumer; unowned ranges are observations, not asset extents"
    });
    fs::write(&report, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    eprintln!("ROM_DATA_HOST_AUDIT frames={frames} requests={} image_bytes={} unowned_bytes={} transparency=PASS", reads.len(), image_bytes.len(), unowned_bytes.len());
}
