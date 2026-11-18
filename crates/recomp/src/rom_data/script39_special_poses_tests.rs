//! Conditional EUR opcode-39 special-unit source model, before publication.
use super::*;
use gba_core::rom_data::{
    script39_registry_poses::IndexTarget,
    script39_special_poses::{Catalog, Script39SpecialPoses, SOURCE_SHA256},
};
use gba_core::Bus;

#[test]
#[ignore = "exact EUR ROM and e163 dataset; Wine only, Linux/USA pending"]
fn european_script39_special_pose_source_model() {
    let root = PathBuf::from(std::env::var_os("FFTA_SCRIPT39_SPECIAL_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SCRIPT39_SPECIAL_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (163, 162)
    );
    let scripts: ScriptArchive =
        serde_json::from_slice(&fs::read(dataset.join(SCRIPT_ARCHIVE_FILE)).unwrap()).unwrap();
    let special: Metadata14 =
        serde_json::from_slice(&fs::read(dataset.join(METADATA14_FILE)).unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom, &manifest.resources, &scripts, &special).unwrap();
    let model = &catalog.decoded;
    assert_eq!(
        (
            model.producer_pairs.len(),
            model.cells.len(),
            model.sequences.len(),
            model.objects.len(),
            model.pixels.len()
        ),
        (285, 964, 109, 7, 87)
    );
    assert_eq!(catalog.sources.len(), 418);
    assert_eq!(
        catalog
            .sources
            .iter()
            .map(|s| u64::from(s.length))
            .sum::<u64>(),
        34026
    );
    for at in [0x37be00, 0x37be04] {
        let cell = model
            .cells
            .binary_search_by_key(&at, |c| c.source_offset)
            .unwrap();
        assert_eq!(model.cells[cell].target, IndexTarget::Null {});
    }
    let cell = model
        .cells
        .binary_search_by_key(&0x37be44, |c| c.source_offset)
        .unwrap();
    assert!(matches!(
        model.cells[cell].target,
        IndexTarget::Sequence { .. }
    ));
    assert!(model
        .cells
        .binary_search_by_key(&0x3714b0, |c| c.source_offset)
        .is_err());
    assert!(model
        .producer_pairs
        .iter()
        .flat_map(|p| &p.script_sources)
        .all(|&at| at != 0xb96a1e && at != 0xba0841));

    let mut joined = Vec::new();
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.len(), catalog.sources.len());
    for ((offset, data, role), source) in spans.into_iter().zip(&catalog.sources) {
        assert_eq!(
            (offset, data.len() as u32, role),
            (source.offset, source.length, source.role.as_str())
        );
        assert_eq!(data, rom[offset as usize..offset as usize + data.len()]);
        assert_eq!(gba_core::ffta_rom::digest(&data), source.sha256);
        joined.extend_from_slice(&data);
    }
    assert_eq!(gba_core::ffta_rom::digest(&joined), SOURCE_SHA256);
    let decoded: Script39SpecialPoses =
        serde_json::from_slice(&serde_json::to_vec(model).unwrap()).unwrap();
    assert_eq!(&decoded, model);
    decoded.validate_producers(&scripts).unwrap();
    let mut invalid = model.clone();
    invalid.cells.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut changed_scripts = scripts.clone();
    let witness = model.producer_pairs[0].script_sources[0];
    changed_scripts
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.source_offset == witness)
        .unwrap()
        .parameters[1] ^= 1;
    assert!(model.validate_producers(&changed_scripts).is_err());

    // Exercise the original low-selector lookup and actor join with a
    // nonmatching list node before the matching one. Inputs are controlled.
    const UNIT: u32 = 0x0200_2fc4;
    const OTHER_UNIT: u32 = 0x0202_1100;
    const MANAGER: u32 = 0x0202_4000;
    const LIST: u32 = 0x0202_4100;
    const FIRST: u32 = 0x0202_4200;
    const SECOND: u32 = 0x0202_4220;
    const OTHER_ACTOR: u32 = 0x0202_4400;
    const ACTOR: u32 = 0x0202_4600;
    const CONTEXT: u32 = 0x0202_6000;
    const WORK: u32 = 0x0202_6100;
    let mut machine = crate::make_machine(rom.clone(), None);
    machine.bus.ewram.fill(0);
    machine.bus.iwram.fill(0);
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.reg_ie = 0;
    machine.bus.ime = false;
    machine.bus.write8(UNIT + 4, 2);
    machine.bus.write8(UNIT + 5, 2);
    machine.bus.write32(0x0200_f4b0, MANAGER);
    machine.bus.write32(MANAGER + 0x10, LIST);
    machine.bus.write32(LIST + 4, FIRST);
    machine.bus.write32(FIRST, OTHER_ACTOR);
    machine.bus.write32(FIRST + 8, SECOND);
    machine.bus.write32(SECOND, ACTOR);
    machine.bus.write32(SECOND + 8, 0);
    machine.bus.write32(OTHER_ACTOR, OTHER_UNIT);
    machine.bus.write32(ACTOR, UNIT);
    machine.bus.write16(ACTOR + 0x34, 0);
    machine.bus.write16(CONTEXT, 0);
    machine.bus.write16(CONTEXT + 0x14, 2);
    machine.bus.write16(CONTEXT + 0x16, 1);
    machine.bus.write16(CONTEXT + 0x18, 1);
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[1] = CONTEXT;
    machine.cpu.regs[2] = WORK;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0812_a8c4;
    machine.cpu.cpsr = 0xbf;
    let (mut saw_unit, mut saw_manager_join, mut comparisons) = (false, false, 0);
    for _ in 0..2000 {
        match machine.cpu.regs[15] {
            0x0812_a8f0 => {
                assert_eq!(machine.cpu.regs[0], UNIT);
                saw_unit = true;
            }
            0x0809_cc78 => {
                assert_eq!((machine.cpu.regs[0], machine.cpu.regs[1]), (MANAGER, UNIT));
                saw_manager_join = true;
            }
            0x0809_e274 => comparisons += 1,
            0x0812_a902 => break,
            _ => {}
        }
        machine.step();
    }
    assert_eq!(
        machine.cpu.regs[15], 0x0812_a902,
        "task did not reach actor join"
    );
    assert!(saw_unit && saw_manager_join);
    assert_eq!(comparisons, 2);
    assert_eq!(machine.bus.read32(WORK + 4), ACTOR);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR opcode-39 conditional special-unit source model; e163 prior-owner subtraction",
        "producer_pairs":285, "new_index_cells":964, "new_sequences":109,
        "new_object_lists":7, "new_pixel_blocks":87, "source_ranges":418,
        "source_bytes":34026, "source_sha256":SOURCE_SHA256,
        "prior_high_registry_owner_preserved":true, "excluded_special_branch_instructions":2,
        "source_reconstruction_and_json_roundtrip":true, "controlled_unit_actor_join":true,
        "runtime_publication":"pending", "original_reader_validation":"pending",
        "ordinary_reachability":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
