//! Passive source discovery from a retained normal world-map checkpoint.
use gba_core::rom_data::read_audit::{Capture, SiteCapture};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "private normal EUR world-map checkpoint/current dataset/native cache; Wine only"]
fn normal_world_frame_records_source_sites_and_ram() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let root = path("FFTA_WORLD_LOCATION_AUDIT_ROOT");
    assert!(!root.exists(), "preserve earlier source observations");
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let mut machine = crate::make_machine(rom, None);
    super::attach_from_directory(&mut machine, &path("FFTA_STATE_TEST_DATASET")).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    let first = machine.bus.frames;
    let before_registers = machine.cpu.regs;
    fs::write(root.join("before-ewram.bin"), &machine.bus.ewram).unwrap();
    fs::write(root.join("before-iwram.bin"), &machine.bus.iwram).unwrap();
    machine.bus.keys = 0x3ff;
    let addresses = Capture::start();
    let scope = SiteCapture::start_all();
    let pointer = &mut machine as *mut gba_core::Machine as *mut std::ffi::c_void;
    crate::run_frame_native(&mut machine, &table, pointer, crate::FRAME_STEP_GUARD);
    let sites = scope.finish();
    let reads = addresses.finish();
    assert_eq!(machine.bus.frames, first + 1);
    assert!(machine.bus.frame_ready);
    assert_eq!(
        sites
            .iter()
            .map(|site| site.read)
            .collect::<std::collections::BTreeSet<_>>(),
        reads
    );
    assert_eq!(
        gba_core::ffta_rom::digest(&machine.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    fs::write(root.join("after-ewram.bin"), &machine.bus.ewram).unwrap();
    fs::write(root.join("after-iwram.bin"), &machine.bus.iwram).unwrap();
    let scoped = sites
        .iter()
        .filter(|site| site.instruction.is_some())
        .count();
    let report = serde_json::json!({
        "task":"ROMDATA-01-WORLD-LOCATION-SELECTOR", "status":"PASS-passive-single-frame-capture",
        "frames":1, "first_guest_frame":first, "last_guest_frame":machine.bus.frames,
        "before_registers":before_registers, "after_registers":machine.cpu.regs,
        "reads":reads, "sites":sites, "scoped_sites":scoped,
        "scope":"new source discovery only, neutral normal input; no manual guest edits or equivalence claim",
        "site_limit":"native/host reads can have no instruction; nested accesses inherit interpreter/HLE scope, validate opcode before attribution",
        "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    });
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_WORLD_SOURCE frames=1 reads={} sites={} scoped={scoped} PASS",
        reads.len(),
        sites.len()
    );
}
