<a id="parche-1-bordes-del-terreno"></a>

# Patch 1: terrain edges

Only PPU reads change; ROM, VRAM and game instructions are not altered.
`FFTA_PATCH_BG` is enabled unless its value is `0`. Required SHA256:
`43FC8204C6DCEEE58828AEBC7AF0C72EB807E99F35AD641C8BB0A4FA8B6EDC19`.
A different ROM retains the previous behavior. There are no function hooks.
With extra=0 the original policy always applies.

Identification: video mode 0, active BG0/BG1 or BG2/BG3 pair,
exact BGxCNT values 4C03/4E02 and matching HOFS/VOFS for both backgrounds.
This requires 64x32 maps, character data at 06000000 and maps at 06006000/06007000.
ASSUMPTION: this signature semantically identifies terrain; it has not been
visually validated in every menu or scene. Other, nonmatching registers are unchanged.

`FFTA_BG_EDGE=transparent` (default) makes the area outside the tile plane
transparent; `clamp` repeats the plane's edge pixel; `wrap` retains the
previous behavior. The plane does not necessarily match the logical map's
boundaries: this patch does not generate columns the game has not written.
Sprite clipping remains pending for Patch 2.

<a id="comprobación-visual-pendiente"></a>

## Pending visual check

The current headless PPM output is 240x160; it was not used to make claims about
the margins. Run from the repository root in PowerShell, once for each mode:

```powershell
$env:FFTA_OAM_LOG='0'
$env:FFTA_PATCH_BG='1'
$env:FFTA_WIDESCREEN='32:9'
$env:FFTA_BG_EDGE='wrap' # repeat with 'transparent' and 'clamp'
.\target\release\recomp.exe play 'Final Fantasy Tactics Advance (USA).gba' --status --gamedb gamedb.sqlite
```

Enter the same battle and move the camera to its four extremes. Capture the
window manually with the Windows screenshot tool, keeping the same zoom in all
three runs. List of twelve captures (legacy filenames retained):

| Mode | Left | Right | Top | Bottom |
| --- | --- | --- | --- | --- |
| wrap | bg-wrap-izquierda.png | bg-wrap-derecha.png | bg-wrap-arriba.png | bg-wrap-abajo.png |
| transparent | bg-transparent-izquierda.png | bg-transparent-derecha.png | bg-transparent-arriba.png | bg-transparent-abajo.png |
| clamp | bg-clamp-izquierda.png | bg-clamp-derecha.png | bg-clamp-arriba.png | bg-clamp-abajo.png |

NOT MEASURED: visual disappearance of mirroring in battle. Transparent and clamp
prevent circular reads outside the plane in the synthetic test.

<a id="validación-realizada"></a>

## Validation performed

`cargo test -p gba-core --lib`: 76 passed; 0 failed.
Project tests: extra=0 preserves pixels with all three policies; left/right
boundaries at 1536 px; exclusion of nonmatching backgrounds and invalid ROMs.
`cargo build -p recomp --release`: successful build.
Two native runs with `FFTA_PATCH_BG=0/1`, `FFTA_OAM_LOG=0`,
`FFTA_WIDESCREEN=0`, `FFTA_BG_EDGE=transparent`:

```powershell
.\target\release\recomp.exe runc 'Final Fantasy Tactics Advance (USA).gba' --frames 600
```

Both: `frames: 600 hash: 727e9157e9bcb0a6 native_blocks: 2116712 fallback_steps: 2`.
This is a boot test without battle input; it does not establish visual
correctness in battle. `runc` does not configure widescreen from the environment
variable and uses extra=0 by default in these processes.
