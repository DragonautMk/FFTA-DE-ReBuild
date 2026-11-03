//! One private Wine filter: canonical source/model and actual original wrapper.
use super::*;
use crate::Machine;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    archive_index: Option<usize>,
    source: Vec<u8>,
    start: u32,
    count: u32,
    output: Vec<u8>,
    status: u32,
    initialized: bool,
    model_accepts: bool,
}
fn original(m: &mut Machine, c: &Case) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    let source = if let Some(i) = c.archive_index {
        0x08000000 + ARCHIVES[i]
    } else {
        m.bus.ewram[0x10000..0x10000 + c.source.len()].copy_from_slice(&c.source);
        0x02010000
    };
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[0] = source;
    m.cpu.regs[1] = 0x02000100;
    m.cpu.regs[2] = c.start;
    m.cpu.regs[3] = c.count;
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.regs[15] = 0x0800537c;
    let preserved = m.cpu.regs[4..12].to_vec();
    let (mut ewram, mut iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    if c.initialized {
        ewram[0x100] = 0;
        iwram[0x2bb0..0x2bb4].copy_from_slice(&(c.output.len() as u32).to_le_bytes());
        iwram[0x2bb4..0x2bb8].copy_from_slice(&u32::MAX.to_le_bytes());
        iwram[0x2bb8..0x2bbc].copy_from_slice(&0x02000100u32.to_le_bytes());
    }
    ewram[0x100..0x100 + c.output.len()].copy_from_slice(&c.output);
    let mut instructions = 0;
    while m.cpu.regs[15] != 0x03007f00 {
        let pc = m.cpu.regs[15];
        assert!(
            (0x08005228..=0x0800542a).contains(&pc),
            "{} escaped {:08x}",
            c.label,
            pc
        );
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(
            instructions < 2_000_000,
            "{} PC{:08x}",
            c.label,
            m.cpu.regs[15]
        );
    }
    assert_eq!(m.cpu.regs[0], c.status, "{} status", c.label);
    assert_eq!(m.cpu.regs[13], 0x03007d00, "{} SP", c.label);
    assert_eq!(
        &m.cpu.regs[4..12],
        preserved,
        "{} callee registers",
        c.label
    );
    assert_eq!(
        m.bus.ewram, ewram,
        "{} complete EWRAM/output guards",
        c.label
    );
    // Original wrapper24B + entry44B stack writes are intentionally not restored.
    iwram[0x7cbc..0x7d00].copy_from_slice(&m.bus.iwram[0x7cbc..0x7d00]);
    assert_eq!(
        m.bus.iwram, iwram,
        "{} IWRAM/globals/outside stack",
        c.label
    );
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR indexed archive source/original wrapper proof; Wine only, Linux/USA pending"]
fn european_indexed_graphics_match_source_and_original_wrapper() {
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_GRAPHICS_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: IndexedGraphics =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 24);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 74907);
    for (at, bytes, _) in &spans {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        assert_eq!(data.byte_at(*at), bytes.first().copied());
        assert_eq!(
            data.byte_at(*at + bytes.len() as u32 - 1),
            bytes.last().copied()
        );
    }
    for at in [0, 0x71e57a, 0x71e57b, 0x71e590, 0x71e70c, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    let decoded: Vec<u8> = data
        .archives
        .iter()
        .flat_map(|a| a.programs.iter().flat_map(Program::decoded_bytes))
        .collect();
    assert_eq!(decoded.len(), 278336);
    assert_eq!(
        ffta_rom::digest(&decoded),
        "b380cbb85eb5606a94622362822e09d2d15761343e3b4a088a294a314a862559"
    );
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<IndexedGraphics>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for kind in 0..20 {
        let mut bad = data.clone();
        match kind {
            0 => {
                bad.groups.pop();
            }
            1 => bad.groups.swap(0, 1),
            2 => bad.groups[0].pointers[0] += 4,
            3 => {
                bad.archives.pop();
            }
            4 => bad.archives.swap(0, 1),
            5 => bad.archives[0].source_offset += 1,
            6 => {
                bad.archives[0].entry_offsets.pop();
            }
            7 => bad.archives[0].entry_offsets[0] += 1,
            8 => bad.archives[0].programs[0].index = 1,
            9 => bad.archives[0].programs[0].relative_offset += 1,
            10 => bad.archives[0].dictionary_relative_offset += 1,
            11 => bad.archives[0].dictionary_mode = 3,
            12 => bad.archives[0].dictionary_length_bits = 8,
            13 => bad.archives[0].fixed_decoded_bytes += 4,
            14 => {
                bad.archives[0].dictionary_bytes.pop();
            }
            15 => bad.archives[0].dictionary_bytes.push(0),
            16 => bad.archives[0].programs[0].decoded_words[0] ^= 1,
            17 => bad.archives[0].programs[0].packets[0] = Packet::Literal { bytes: vec![] },
            18 => {
                bad.archives[0].programs[0].packets[0] = Packet::Dictionary {
                    length: 2,
                    offset: 0,
                }
            }
            _ => {
                bad.archives[0].programs[0].packets[0] = Packet::Dictionary {
                    length: 35,
                    offset: 1024,
                }
            }
        }
        assert!(bad.validate().is_err(), "shape guard{kind}");
        guards += 1;
    }
    for kind in 0..8 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match kind {
            0 => bad["extra"] = json!(0),
            1 => bad["groups"][0]["extra"] = json!(0),
            2 => bad["archives"][0]["programs"][0]["extra"] = json!(0),
            3 => bad["archives"][0]["programs"][0]["packets"][0]["extra"] = json!(0),
            4 => bad["archives"][0]["dictionary_mode"] = json!(256),
            5 => bad["archives"][0]["entry_offsets"][0] = json!(-1),
            6 => bad["groups"][0]["pointers"] = json!([0, 1, 2, 3]),
            _ => {
                bad["archives"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("opaque_header");
            }
        }
        assert!(serde_json::from_value::<IndexedGraphics>(bad).is_err());
        guards += 1;
    }
    let mut changed = data.clone();
    changed.archives[0].opaque_header = [0, 255];
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[0x5228] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[ARCHIVES[0] as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let a = &data.archives[0];
    let source = a.source_bytes().unwrap();
    for len in [0, 7, 127, source.len() - 1] {
        assert!(Archive::from_source(&source[..len], a.source_offset).is_err());
        guards += 1;
    }
    assert!(Archive::from_source(&source, 0xffffff).is_err());
    guards += 1;
    let cases: Vec<Case> =
        serde_json::from_slice(&fs::read(root.join("independent-cases.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let (mut canonical, mut boundary, mut synthetic, mut instructions) = (0, 0, 0, 0);
    for c in &cases {
        if let Some(i) = c.archive_index {
            assert_eq!(
                data.archives[i].selected_bytes(c.start, c.count),
                if c.initialized {
                    Some(c.output.clone())
                } else {
                    None
                },
                "{} model selection",
                c.label
            );
            if c.label.starts_with("whole-") {
                canonical += 1;
            } else {
                boundary += 1;
            }
        } else {
            let parsed = Archive::from_source(&c.source, 0x10000);
            assert_eq!(parsed.is_ok(), c.model_accepts, "{} scoped parser", c.label);
            if let Ok(a) = parsed {
                assert_eq!(a.selected_bytes(c.start, c.count), Some(c.output.clone()));
            }
            synthetic += 1;
        }
        instructions += original(&mut m, c);
    }
    let mut selectors = 0;
    for group in &data.groups {
        for language in [0u8, 1, 2, 3, 4, 5, 255] {
            m.bus.ewram.fill(0x5a);
            m.bus.ewram[0x1f6e] = language;
            m.cpu.cpsr = 0xbf;
            m.cpu.regs[0] = 0x02001f6e;
            m.cpu.regs[1] = 0x08000000 + group.source_offset;
            m.cpu.regs[15] = 0x080196a0;
            for _ in 0..4 {
                m.bus.frame_ready = false;
                m.step();
                instructions += 1;
            }
            assert_eq!(m.cpu.regs[15], 0x080196a8);
            let view = group.encoded_view(language);
            assert_eq!(m.cpu.regs[0], view);
            let at = (view - 0x08000000) as usize;
            let pointer = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
            assert_eq!(m.cpu.regs[5], pointer);
            assert_eq!(
                group.select(language),
                if language < 5 {
                    Some(pointer - 0x08000000)
                } else {
                    None
                }
            );
            selectors += 1;
        }
    }
    assert_eq!(m.bus.rom, rom);
    let result = json!({"status":"PASS-source-model-original-wrapper","guards":guards,"archives":20,"programs":649,"source_spans":24,"source_bytes":74907,"decoded_bytes":278336,
        "whole_archive_cases":canonical,"boundary_cases":boundary,"synthetic_cases":synthetic,"raw_language_selector_prefix_cases":selectors,"instructions_coverage_only":instructions,
        "complete_output_and_guards_exact":true,"wrapper_status_globals_exact":true,"SP_R4_R11_exact":true,"all_RAM_outside_expected_output_globals_stack_exact":true,"final_ROM_exact":true,
        "scope":"Full0537C canonical calls and low16start-count boundaries; controlled mode0width5 synthetic streams/failures and rawlanguage prefix. Original stack scratch excluded from preservation comparison. No nativeclock/ordinary/liveupload/fullroutineABI universal claim; mode3-widerwidth support pending.",
        "schema_extractor_decoder_unchanged":[1,90,89],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_INDEXED_GRAPHICS_MODEL source74907 decoded278336 guards{guards} whole{canonical} boundary{boundary} synthetic{synthetic} selectors{selectors} PASS"
    );
}
