# FFTA movement grid: measured background edge correction

MEASURED: ROM SHA256 required by the existing BG gate:
43FC8204C6DCEEE58828AEBC7AF0C72EB807E99F35AD641C8BB0A4FA8B6EDC19.
No ROM payload is added by this change.

## Identification

MEASURED: native runc with FFTA_OAM_LOG=1, FFTA_WIDE_DEBUG=nospr,
FFTA_BG_EDGE=wrap and FFTA_BG_DUMP_PREFIX isolated each active text layer.
The images, not register similarity alone, identify the blue cells in BG0
and a small foreground cell fragment in BG1.

| Layer | BGCNT | Map base | Character base | Pixels | Priority | HOFS | VOFS |
|---|---|---|---|---|---|---|---|
| BG0, blue cells | 0A0B | 06005000 | 06008000 | 256x256 | 3 | 8 | 88 |
| BG1, cell fragment | 0B0A | 06005800 | 06008000 | 256x256 | 2 | 8 | 88 |
| BG2, terrain | 4C03 | 06006000 | 06000000 | 512x256 | 3 | 200 | 56 |
| BG3, terrain | 4E02 | 06007000 | 06000000 | 512x256 | 2 | 200 | 56 |

Literal registers: target/ffta-movement/left-layers.log, frame 13916.
Visual evidence: left-bg0.png and left-bg1.png in the same directory.
That frame and the earlier frame 14156 have equal terrain scroll and are
NOT evidence of opposite camera extremes.

## Change and scope

The existing FFTA_PATCH_BG and ROM hash gate remains authoritative.
The existing transparent, clamp and wrap policy now includes BG0/BG1 only
when all four text backgrounds are enabled in mode 0, BG2/BG3 match the
paired terrain configuration, BG0/BG1 have the measured BGCNT values,
their masked scroll values match, and HOFS is in 8..15.
This conservatively avoids static UI at HOFS=0 and other layer layouts.
NOT MEASURED: exhaustive visual coverage of every menu and cloud scene.
ASSUMPTION: movement grids in other battles retain this signature.
There is no behavior change with extra=0.

## Reproduction

Run after cargo build -p recomp --release:

```powershell
$env:FFTA_PATCH_BG='1'
$env:FFTA_PATCH_SPR='0'
$env:FFTA_OAM_LOG='0'
$env:FFTA_WIDESCREEN='32:9'
$env:FFTA_WIDE_DEBUG='nospr'
$env:FFTA_BG_EDGE='transparent'
$env:FFTA_CAPTURE_BG_FROM='13916'
$env:FFTA_CAPTURE_BG_PREFIX='target/ffta-movement/new-transparent'
.\target\release\recomp.exe runc 'Final Fantasy Tactics Advance (USA).gba' --frames 15000 --input target/ffta-movement/movement-pan.input
python scripts/ffta_ppm_to_png.py target/ffta-movement
```

Use a new prefix for each run: captures refuse to overwrite existing files.
Use wrap or clamp instead of transparent for the alternative policies.
For interactive native play, clear capture/benchmark variables in a fresh
PowerShell, set PATCH_BG=1, PATCH_SPR=0, OAM_LOG=0, WIDESCREEN=32:9 and
BG_EDGE=transparent as above, then run:

```powershell
.\target\release\recomp.exe play 'Final Fantasy Tactics Advance (USA).gba'
```

The pan input extends ffta-batalla-1.input with left held during
13916..14315 and right held during 14376..14975.

MEASURED: left traversal minimum HOFS=152, VOFS=36 at frame 14007;
right traversal maximum HOFS=216, VOFS=64 at frame 14473.
NOT MEASURED: these being the global horizontal limits of the whole battle map.
The blue grid remains visible in both captures. Transparent removes the
blue repeat visible at the left in wrap; no blue repeat is visible at the
right in either capture, so that pair does not prove removal on the right.
Central 240x160 pixels: 0 changes for both pairs.
Margins: 2060 changed pixels left, 1018 right, including terrain effects.
NOT MEASURED: missing logical movement cells outside the hardware tilemap;
the edge policy cannot create tilemap contents that the game never wrote.

## Validation

MEASURED: native extra=0, SPR=0, LOG=0, BG=0 versus BG=1, 600 frames:

```text
frames: 600 hash: 727e9157e9bcb0a6 native_blocks: 2116712 fallback_steps: 2
frames: 600 hash: 727e9157e9bcb0a6 native_blocks: 2116712 fallback_steps: 2
```

This is a final framebuffer hash after a 600-frame boot run, not a full
frame-by-frame hash stream or a 600-frame movement-grid scene comparison.
The new synthetic layer test also compares all three policies at extra=0,
checks wide margins, and rejects static UI and non-terrain configurations.

Command: cargo test -p gba-core -p recomp-core -p recomp --lib --bins

```text
test result: ok. 79 passed; 0 failed; 1 ignored
test result: ok. 6 passed; 0 failed; 0 ignored
test result: ok. 3 passed; 0 failed; 1 ignored
```

## Performance method

Native headless CPU measurements, not desktop GPU presentation.
FFTA_BENCH_FROM=13680 times only the next 600 frames through frame 14280.
The original input recording has no input changes in this interval.
All runs warm up with the sprite patch disabled, then restore the requested
SPR setting at the timer boundary. Normal play is unaffected by this option.
The starting guest-state hash covers EWRAM, IWRAM, IO and OAM; it is not a
complete machine snapshot. Identical hashes establish equality of those
scene data, not equality of every CPU and peripheral field.
The LOG case loads the diagnostic native DLL built with FFTA_OAM_LOG=1,
so native instruction observation and writes are included in its cost.
Logging output is redirected to a local file and included in the timing.

MEASURED: sequential first-pass results, milliseconds per frame:

| Setting | Width | LOG | SPR | ms/frame |
|---|---|---|---|---|
| Hardware width | 240 | 0 | 0 | 4.779085 |
| Wide baseline | 568 | 0 | 0 | 6.996778 |
| Wide logging | 568 | 1 | 0 | 7.402068 |
| Wide sprites | 568 | 0 | 1 | 6.909055 |
| Wide sprites, repeat | 568 | 0 | 1 | 7.029506 |
| Wide baseline, repeat | 568 | 0 | 0 | 7.126323 |

All starting scene hashes:
732d9b77fea13f28a39446d11194261ab438ecb88fdfbeb616b228b98d5767e0.
All final central framebuffer FNV hashes: 155fee01de0bef17.
First-pass increments: widescreen +2.217693 ms/frame;
logging +0.405290 ms/frame; sprite flag -0.087723 ms/frame.
Do not interpret the small negative sprite difference as an optimization.
No GPU/window cost, interaction between LOG and SPR, or statistical
confidence interval was measured.

Important literal verification from the sprite repeat:

```text
FFTA_PATCH_VERIFY frames=14280 extra=164 sprite_enabled=true calls=4200
FFTA_HOST publications=0 objects=0
FFTA_PIPELINE decoded=3000 no_input=600 mismatches=0 active=false
```

MEASURED: pending sprite lists existed with 28 objects in each alternating
shadow buffer, but no host list reached the PPU in this run. Consequently
NOT MEASURED: performance with an active, populated host list, and the strong
slowdown reported by the user. This task does not change or repair that
sprite publication path. Widescreen is the largest measured increment
among the settings exercised here; that is not a conclusion about the
unmeasured active-host or desktop presentation cases.
The two repeat runs also have identical OAM SHA256 and wide framebuffer
SHA256, respectively:
dcbe8daa45954dd89014b0dfe69d096a77598c05e97435b29b8f30a96d0d93fd
b59e41c41fad1e8ea47917252b58cb71bf941fc4a533ae986a8d016baa970b0b.

To reproduce a measured window, use the original recording and:

```powershell
$env:FFTA_PATCH_BG='1'
$env:FFTA_BG_EDGE='transparent'
$env:FFTA_WIDESCREEN='32:9'
$env:FFTA_OAM_LOG='0'
$env:FFTA_PATCH_SPR='1'
$env:FFTA_PATCH_VERIFY='1'
$env:FFTA_BENCH_FROM='13680'
$env:FFTA_BENCH_LIBRARY='target/ffta-movement/native-normal.dll'
.\target\release\recomp.exe runc 'Final Fantasy Tactics Advance (USA).gba' --frames 14280 --input ffta-batalla-1.input
```

The normal DLL is a local copy of the user's already compiled DLL.
For the LOG case use out/diag-source.dll, locally built from an identical
temporary ROM copy under ignored target/ with FFTA_OAM_LOG=1 and --ram.
Neither generated DLL nor ROM copy is intended for repository distribution.

NOT MEASURED: isolated cost of each host-list operation.
MEASURED by source inspection: render_sprites_extent creates an expanded Vec
per scanline and searches the host list per OAM slot; publish clones the
accepted host list and allocates comparison bytes per object; pending lists
are recreated on frame changes. Proposed follow-up, not implemented:
reuse list capacity, compare attributes without a Vec, and prepare ordered
sprite data once per frame/publication instead of scanning per scanline.
The installer and character sprite implementation are not changed by this task.
