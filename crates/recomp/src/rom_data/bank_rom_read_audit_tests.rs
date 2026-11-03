//! New Bank bus/host inventory, reusing the accepted native-only input bound.
use gba_core::{
    rom_data::read_audit::{Capture, SiteCapture},
    Bus, Machine,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn clear_audio(m: &mut Machine) {
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
    m.bus.hle_tap.clear();
}

fn probe_all_mode(rom: &[u8]) {
    // Synthetic diagnostic probes, separate from the untouched normal scene.
    let mut m = crate::make_machine(rom.to_vec(), None);
    for mode in 0..3 {
        m.cpu.regs[15] = 0x08000000;
        m.cpu.cpsr = 0x1f;
        let addresses = Capture::start();
        let sites = match mode {
            0 => SiteCapture::start([]),
            1 => SiteCapture::start([0x371034]),
            _ => SiteCapture::start_all(),
        };
        assert_eq!(m.bus.read8(0x08371034), rom[0x371034]);
        assert_eq!(m.bus.read8(0x08371038), rom[0x371038]);
        gba_core::exec::step(&mut m.cpu, &mut m.bus);
        let sites = sites.finish();
        let addresses = addresses.finish();
        assert_eq!(addresses.len(), 2, "ROM instruction fetch stays excluded");
        assert_eq!(sites.len(), mode);
        assert!(sites.iter().all(|site| site.instruction.is_none()));
        if mode == 1 {
            assert_eq!(sites.first().unwrap().read.physical_offset, 0x371034);
        } else if mode == 2 {
            assert_eq!(
                sites.iter().map(|site| site.read).collect::<BTreeSet<_>>(),
                addresses
            );
        }
    }
    let sites = SiteCapture::start_all();
    m.bus.read8(0x08371034);
    drop(sites);
    assert!(SiteCapture::start_all().finish().is_empty());
}

#[test]
#[ignore = "private EUR Bank bus-host source inventory; Wine only, no old fullpair replay"]
fn european_bank_bus_host_inventory_with_execution_scope() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let root = path("FFTA_BANK_READ_AUDIT_ROOT");
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    probe_all_mode(&rom);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest_bytes = fs::read(dataset.join("manifest.toml")).unwrap();
    let manifest: gba_core::rom_data::Manifest =
        toml::from_str(std::str::from_utf8(&manifest_bytes).unwrap()).unwrap();
    assert_eq!(
        (
            manifest.extractor_revision,
            manifest.decoder_revision,
            manifest.resources.len()
        ),
        (76, 75, 68)
    );
    let old: serde_json::Value =
        serde_json::from_slice(&fs::read(path("FFTA_BANK_NATIVE_REPORT")).unwrap()).unwrap();
    assert_eq!(old["frames"], 12);
    assert_eq!(
        old["per_frame_full_state_clock_buffers_dispatch_audio_equal"],
        true
    );
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut m = crate::make_machine(rom, None);
    super::attach_from_directory(&mut m, &dataset).unwrap();
    crate::save_state::load(&mut m, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut m, None);
    m.bus.tap_channels = true;
    clear_audio(&mut m);
    let first = m.bus.frames;
    assert_eq!(serde_json::json!(first), old["first_guest_frame"]);
    let capture = Capture::start();
    let scope = SiteCapture::start_all();
    let mut dispatch = [0u64; 2];
    let mut audio = [0u64; 5];
    for relative in 0..12 {
        m.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut m.bus, relative).unwrap();
        let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
        let (native, fallback) =
            crate::run_frame_native(&mut m, &table, pointer, crate::FRAME_STEP_GUARD);
        dispatch[0] += native;
        dispatch[1] += fallback;
        assert_eq!(m.bus.frames, first + relative + 1);
        assert!(m.bus.frame_ready);
        for (sum, length) in audio.iter_mut().zip([
            m.bus.audio_buf.len(),
            m.bus.psg_tap.len(),
            m.bus.fifo_tap[0].len(),
            m.bus.fifo_tap[1].len(),
            m.bus.hle_tap.len(),
        ]) {
            *sum += length as u64;
        }
        clear_audio(&mut m);
    }
    let sites = scope.finish();
    let reads = capture.finish();
    let projection: BTreeSet<_> = sites.iter().map(|site| site.read).collect();
    assert_eq!(
        projection, reads,
        "all-mode site projection exactly preserves existing address-only keys"
    );
    assert_eq!(serde_json::json!(dispatch), old["dispatch_coverage_only"]);
    assert_eq!(serde_json::json!(audio), old["audio_samples"]);
    assert!(audio.iter().all(|&count| count > 0));
    let cycle = m
        .bus
        .rom_resources()
        .unwrap()
        .task_cycle_deltas_stats()
        .unwrap();
    assert!(cycle.source_bytes > 0 && cycle.phases_used > 0);
    let with_scope = sites
        .iter()
        .filter(|site| site.instruction.is_some())
        .count();
    assert!(with_scope > 0 && with_scope < sites.len());
    let report = serde_json::json!({
        "task":"ROMDATA-01-BANK-BUS-HOST-RESIDUE","schema":1,"frames":12,
        "first_guest_frame":first,"last_guest_frame":m.bus.frames,"final_guest_clock":m.bus.clock,
        "source_rom_sha256":manifest.source_rom_sha256,"manifest_sha256":gba_core::ffta_rom::digest(&manifest_bytes),
        "extractor_revision":76,"decoder_revision":75,"resource_count":68,
        "all_mode_filter_fetch_projection_drop_probes":"PASS",
        "unique_reads":reads.len(),"site_reads":sites.len(),"reads_with_instruction_scope":with_scope,
        "reads_outside_instruction_scope":sites.len()-with_scope,"reads":reads,"sites":sites,
        "dispatch_coverage_only":dispatch,"audio_samples":audio,"retained_old_Bank12_dispatch_sample_counts_exact":true,
        "typed_cycle_reads":cycle.bus_reads,"typed_cycle_source_bytes":cycle.source_bytes,"typed_cycle_phases":cycle.phases_used,
        "scope":"one current decoded Bank12 source inventory only; old fullpair and source classifications reused, not replayed",
        "context_limit":"interpreter/HLE scope may include nested DMA accesses, validate opcode/address/width before operand attribution; native/host outside scope staysNone",
        "blind_spots":["other scenes/threads","unselected direct host readers","typed HLE paths bypassing bus","folded native constants","code/signature authentication"],
        "Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    });
    fs::write(
        root.join("reads-and-sites.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_BANK_READS frames=12 reads={} sites={} scoped={with_scope} outside={} projection=exact retained_counts=exact PASS",reads.len(),sites.len(),sites.len()-with_scope);
}
