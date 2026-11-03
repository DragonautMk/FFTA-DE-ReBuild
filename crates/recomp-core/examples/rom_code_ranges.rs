//! Metadata-only static ROM block ranges; never emits instructions or ROM bytes.
//! Usage: cargo run -p recomp-core --example rom_code_ranges -- ROM [seeds.txt]
//! Seed lines are hexadecimal guest keys (bit 0 selects Thumb).
use recomp_core::analyze::{analyze, View};
use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: rom_code_ranges ROM [seeds.txt]".into());
    }
    let rom = fs::read(&args[0])?;
    let mut seeds = BTreeSet::new();
    if let Some(path) = args.get(1) {
        for line in fs::read_to_string(path)?.lines() {
            let key = u32::from_str_radix(line.trim().trim_start_matches("0x"), 16)?;
            let offset = (key & !1).checked_sub(0x0800_0000).ok_or("non-ROM seed")?;
            if offset as usize >= rom.len() {
                return Err("out-of-ROM seed".into());
            }
            seeds.insert(key);
        }
    }
    let analysis = analyze(
        &View {
            rom: &rom,
            ewram: None,
            iwram: None,
            bios: None,
        },
        &seeds.iter().copied().collect::<Vec<_>>(),
    );
    let mut out = io::BufWriter::new(io::stdout().lock());
    writeln!(
        out,
        "# rom_bytes={} rom_fnv64={:016x} seeds={}",
        rom.len(),
        recomp_core::fnv64(&rom),
        seeds.len()
    )?;
    writeln!(out, "start\tend\tmode\tinstructions\tretained_entry")?;
    for block in analysis.blocks {
        let last = block.instrs.last().ok_or("empty block")?;
        let start = block.addr.checked_sub(0x0800_0000).ok_or("non-ROM block")?;
        let end = last.addr.checked_add(last.size()).ok_or("block overflow")? - 0x0800_0000;
        if end as usize > rom.len() {
            return Err("out-of-ROM block".into());
        }
        writeln!(
            out,
            "{start}\t{end}\t{}\t{}\t{}",
            if block.thumb { "thumb" } else { "arm" },
            block.instrs.len(),
            u8::from(seeds.contains(&block.key()))
        )?;
    }
    Ok(())
}
