<a id="ffta-usa-productores-de-obj"></a>

# FFTA USA: OBJ producers

MEASURED by reading/disassembling the required local ROM; no ROM is included.
SHA256 and atomic checks: `crates/gba-core/src/ffta_rom.rs`.
Both functions are Thumb. The full-range hashes include their literals;
individual instruction checks cover two bytes.

<a id="abi-y-entrada"></a>

## ABI and entry

`0x080014C8`: r0 list, r1 origin X, r2 origin Y, r3 attribute mask.
On the entry stack: +0 tile offset, +4 priority, +8 palette,
+12 OAM buffer, +16 limit, +20 initial index. Returns r0 = final index;
if the limit has already been reached on entry, the early exit preserves the original r0.
`0x080016D4`: r0 list, r1 X, r2 Y, r3 tile offset; stack:
+0 priority (low byte only), +4 palette, +8 buffer, +12 limit, +16 index.
Returns r0 = final index, even when exiting immediately at the limit.

List: little-endian uint16 count, followed by six-byte records:
attr0 uint16 (signed 8-bit Y), attr1 uint16 (signed 9-bit X),
attr2 uint16. There is no padding between records. The dimension table is read
at `0x08149510 + 4*(4*shape+size)`, two uint16 values for width/height.

First function: mask 0x30000000 selects reflection about the X/Y origin;
in that case X=originX-offsetX-width, Y=originY-offsetY-height. Otherwise
it adds offsets. The original function narrows both results to int16 before
clipping. The second always adds and also narrows to int16.

<a id="comportamiento-observado"></a>

## Observed behavior

The literal game disassembly was removed from the public source distribution.
Only independently described behavior, hook addresses and verification hashes are retained.

Clipping: rejects when the origin and origin+dimension lie beyond the same
edge (X<0 or both X>239; Y<0 or both Y>159). It does not reject solely for X>239
without reading the dimension. Processing continues until descriptors or capacity run out.
It writes only the first six bytes of each OBJ, preserving the fourth halfword
that contains affine matrices. Attributes: tile addition modulo 1024, priority
addition modulo four, palette addition modulo 16; the other bits are preserved.

Direct Thumb BL callers read in the explored region:
14C8: 08001D80, 08001EA4, 08001FC0, 080020DC.
16D4: 08001DEE, 08001F0A, 08002026, 08002142.
NOT MEASURED: exhaustive coverage of indirect callers or code in RAM.

<a id="enganche"></a>

## Hook

Step a: interpret original instructions, one instruction per dispatch,
with the same cycle, bus and IRQ calculations. There is no binary copy of the game.
The native path includes a labeled guard in these functions' blocks,
including chained calls; dispatch also guards older libraries.
The product cache uses the `-ffta-spr1` suffix to rebuild those guards.
`FFTA_PATCH_SPR=0` retains the previous native execution. Extra=0 does not use a host list.
The project's own code allows the A/B check without modifying the ROM.
