#!/usr/bin/env python3
"""Cross-load synthetic desktop/Android snapshots within a validation unit."""
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
WORK = Path(os.environ['FFTA_VALIDATION_WORK'])
PROGRAM = r'''
use gba_core::Bus;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let work = std::env::var("FFTA_VALIDATION_WORK").unwrap();
    let mut rom = vec![0; 512];
    rom[..4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
    rom[256..265].copy_from_slice(b"FLASH_V12");
    let mut m = gba_core::Machine::new(rom);
    m.cpu.regs[3] = 0x12345678;
    m.bus.ewram[125] = 99;
    m.bus.write16(0x04000100, 0xff00);
    m.bus.write16(0x04000102, 0x0080);
    m.bus.write16(0x04000084, 0x0080);
    m.bus.write16(0x04000062, 0xf080);
    m.bus.write16(0x04000064, 0x8700);
    m.bus.tick(12345);
    let own = m.save_state().unwrap();
    if args[1] == "write" {
        std::fs::write(format!("{work}/{}.state", args[2]), own).unwrap();
        return;
    }
    let incoming = std::fs::read(format!("{work}/{}.state", args[2])).unwrap();
    m.load_state(&incoming).expect("cross-platform load");
    let roundtrip = m.save_state().unwrap();
    assert!(roundtrip == incoming, "cross-platform state changed during decode and re-encode");
    let mut legacy = incoming.clone();
    legacy[..8].copy_from_slice(b"FFTA-AN\0");
    m.load_state(&legacy).expect("legacy Android load");
    m.load_state(&std::fs::read(format!("{work}/pc.state")).unwrap()).unwrap();
    m.run_frame(1_000_000);
    m.run_frame(1_000_000);
    let video: Vec<u8> = m.bus.framebuffer.iter().flat_map(|v| v.to_le_bytes()).collect();
    let audio: Vec<u8> = m.bus.audio_buf.iter().flat_map(|v| v.to_le_bytes()).collect();
    let summary = format!("{:?} {:x} {} {} {} {} {} {}", m.cpu.regs, m.cpu.cpsr,
        m.bus.clock, m.bus.frames, gba_core::ffta_rom::digest(&m.bus.ewram),
        gba_core::ffta_rom::digest(&m.bus.oam), gba_core::ffta_rom::digest(&video),
        gba_core::ffta_rom::digest(&audio));
    std::fs::write(format!("{work}/{}.after", args[3]), summary).unwrap();
}
'''

for platform, core in [('pc', ROOT / 'crates/gba-core'), ('android', ROOT / 'AndroidPort/crates/gba-core')]:
    project = WORK / platform
    (project / 'src').mkdir(parents=True)
    (project / 'Cargo.toml').write_text(
        f'[package]\nname="ffta-cross-{platform}-{WORK.parent.name}"\nversion="0.1.0"\nedition="2021"\n'
        f'[workspace]\n[dependencies]\ngba-core={{path={json.dumps(str(core))}}}\n')
    (project / 'src/main.rs').write_text(PROGRAM)

def run(platform, *args):
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT / ('target' if platform == 'pc' else 'AndroidPort/target'))
    subprocess.run(['cargo', 'run', '--offline', '--manifest-path', str(WORK / platform / 'Cargo.toml'), '--', *args], env=env, check=True)

for platform in ('pc', 'android'):
    run(platform, 'write', platform)
run('pc', 'verify', 'android', 'pc')
run('android', 'verify', 'pc', 'android')
assert (WORK / 'pc.after').read_bytes() == (WORK / 'android.after').read_bytes(), 'CPU, RAM, video or audio replay differs'
print('Desktop and Android snapshots cross-load; synthetic CPU, RAM, video and audio replay match')
(WORK / 'compat-result.json').write_text(json.dumps({'status': 'PASS'}) + '\n')
