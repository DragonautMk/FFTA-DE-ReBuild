//! Private original-reader probes for statically produced EUR tones and sample endpoints.
//! Partial note/kernel probes are explicit checkpoints, not complete mixer/scene equivalence.
use super::*;
use gba_core::Machine;
use serde_json::Value;
use std::collections::BTreeSet;
const PLAYER: u32 = 0x02010000;
const TRACK: u32 = 0x02010100;
const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
fn n(v: &Value, key: &str) -> u32 {
    v[key].as_u64().unwrap().try_into().unwrap()
}
fn put(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn reset(m: &mut Machine, pc: u32, thumb: bool) {
    m.bus.ewram.fill(0);
    m.bus.iwram.fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    put(&mut m.bus.iwram, 0x7ff0, 0x020084e0);
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = if thumb { 0xbf } else { 0x9f };
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(m: &mut Machine, end: u32, visited: &mut BTreeSet<u32>) -> usize {
    let mut count = 0;
    while m.cpu.regs[15] != end {
        assert!(count < 300, "pc{:08x}", m.cpu.regs[15]);
        visited.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    count
}
#[test]
#[ignore = "private EUR produced tone/map/PCM source readers; Wine only, Linux/USA pending"]
fn european_produced_tones_keymaps_and_original_pcm_endpoints() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TONE_SOURCE_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let data: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TONE_SOURCE_CASES").unwrap()).unwrap(),
    )
    .unwrap();
    let programs = data["programs"].as_array().unwrap();
    let maps = data["mapped"].as_array().unwrap();
    let waves = data["waves"].as_array().unwrap();
    assert_eq!((programs.len(), maps.len(), waves.len()), (390, 348, 277));
    let mut m = Machine::new(rom.clone());
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    let mut program_reads = BTreeSet::new();
    let mut map_reads = BTreeSet::new();
    let mut tail_reads = BTreeSet::new();
    let mut fixed = 0;
    for p in programs {
        reset(&mut m, 0x081483e4, true);
        m.cpu.regs[0] = PLAYER;
        m.cpu.regs[1] = TRACK;
        put(&mut m.bus.ewram, 0x10030, n(p, "bank"));
        put(
            &mut m.bus.ewram,
            0x10140,
            0x08000000 + n(p, "operand_offset"),
        );
        let mut expected = m.bus.ewram.clone();
        let t = &p["tone"];
        let mut bytes = vec![
            n(t, "type") as u8,
            n(t, "key") as u8,
            n(t, "length") as u8,
            n(t, "pan_or_sweep") as u8,
        ];
        bytes.extend(n(t, "word04").to_le_bytes());
        bytes.extend(n(t, "word08").to_le_bytes());
        assert_eq!(
            &rom[n(t, "source_offset") as usize..n(t, "source_offset") as usize + 12],
            bytes
        );
        expected[0x10124..0x10130].copy_from_slice(&bytes);
        put(
            &mut expected,
            0x10140,
            0x08000000 + n(p, "operand_offset") + 1,
        );
        for i in 0..12 {
            program_reads.insert(n(t, "source_offset") + i);
        }
        instructions += until(&mut m, RETURN, &mut visited);
        assert_eq!(
            m.bus.ewram.iter().zip(&expected).position(|(a, b)| a != b),
            None,
            "original BD {}:{}",
            n(p, "bank"),
            n(p, "program")
        );
        assert_eq!(m.cpu.regs[13], STACK);
        assert!(m.cpu.regs[4..12].iter().all(|v| *v == 0));
        assert!(m.bus.iwram[0x7c00..0x7d00].iter().all(|v| *v == 0x5a));
        assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|v| *v == 0x5a));
    }
    for p in maps {
        reset(&mut m, 0x081487cc, true);
        m.cpu.regs[0] = 0;
        m.cpu.regs[1] = PLAYER;
        m.cpu.regs[2] = TRACK;
        let tone = n(p, "parent_tone_offset") as usize;
        m.bus.ewram[0x10124..0x10130].copy_from_slice(&rom[tone..tone + 12]);
        m.bus.ewram[0x10105] = n(p, "key") as u8;
        put(
            &mut m.bus.ewram,
            0x10140,
            0x08000000 + n(p, "note_operand_offset"),
        );
        let before = m.bus.ewram.clone();
        let mut saw_map = false;
        while m.cpu.regs[15] != 0x08148868 {
            assert!(visited.len() < 2000);
            if m.cpu.regs[15] == 0x0814882e {
                let at = m.cpu.regs[1];
                assert_eq!(at, 0x08000000 + n(p, "map_offset"));
                map_reads.insert(at - 0x08000000);
                saw_map = true;
            }
            let pc = m.cpu.regs[15];
            visited.insert(pc);
            m.bus.frame_ready = false;
            m.step();
            instructions += 1;
            assert!(
                (0x081487cc..0x08148868).contains(&m.cpu.regs[15]) || m.cpu.regs[15] == 0x08148868,
                "note probe pc{:08x}",
                m.cpu.regs[15]
            );
        }
        assert!(saw_map);
        assert_eq!(m.cpu.regs[9], 0x08000000 + n(p, "tone_offset"));
        assert_eq!(m.cpu.regs[3], n(p, "key"));
        for (i, (a, b)) in m.bus.ewram.iter().zip(&before).enumerate() {
            if !(0x10104..0x10108).contains(&i) && !(0x10140..0x10144).contains(&i) {
                assert_eq!(a, b, "note read-only outside operands {i:x}");
            }
        }
    }
    for p in waves {
        let at = n(p, "source_offset");
        let size = n(p, "size");
        let last = at + 16 + size - 1;
        reset(&mut m, 0x081480c8, false);
        m.cpu.regs[2] = 1;
        m.cpu.regs[3] = 0x08000000 + last;
        m.cpu.regs[4] = PLAYER;
        m.cpu.regs[5] = 0x02020000;
        m.cpu.regs[8] = 1;
        let is_fixed = p["ctype_values"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_u64() == Some(8));
        m.bus.ewram[0x10001] = if is_fixed { 8 } else { 0 };
        let before = m.bus.ewram.clone();
        let end = if is_fixed { 0x08148144 } else { 0x081481e8 };
        instructions += until(&mut m, end, &mut visited);
        assert_eq!(m.cpu.regs[0], rom[last as usize] as i8 as i32 as u32);
        assert_eq!(m.cpu.regs[3], 0x08000000 + last + 1);
        if is_fixed {
            fixed += 1;
        } else {
            assert_eq!(m.cpu.regs[1], rom[last as usize + 1] as i8 as i32 as u32);
            tail_reads.insert(last + 1);
        }
        assert_eq!(m.bus.ewram, before, "kernel source read-only");
    }
    assert_eq!(
        (
            program_reads.len(),
            map_reads.len(),
            tail_reads.len(),
            fixed
        ),
        (4680, 347, 276, 1)
    );
    assert_eq!(m.bus.rom, rom);
    assert!(m.bus.rom_resources().is_none());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "produced_program_copies":390,"map_key_links":348,"distinct_map_bytes":347,"interpolating_endpoints":276,"fixed8_last_sample_probe":fixed,
        "program_bytes":program_reads.len(),"instructions_coverage":instructions,"pcs":visited.len(),"original_bd_full_words_ewram_sp_callee":true,
        "original_map_selection_partial_checkpoint":true,"original_signed_pcm_last_plus1_interpolating_endpoint":true,
        "note_kernel_probes":"explicit partial checkpoints; not full mixer/scene/ABI equivalence","all_pcm_payload_bytes_read":false,
        "managed":"EUR e41-d40 unchanged","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
