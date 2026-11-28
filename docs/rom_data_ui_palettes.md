# Raw UI palette resources

EUR schema 1/extractor 20/decoder 19 adds fixed `ui-palettes.json`, resource
`graphics.ui-raw-palettes`, format `ui-raw-palettes-v1-json`. Its initial catalog
contained 21 raw banks/1328 full u16 color words, 21 disjoint source ranges/2656
bytes and 23 checked direct, derived or conditional references. E274-d273 adds
one 16-word physical projection, for 22 banks/1344 words and 2688 source bytes.
The original banks serve both BG and OBJ halves of the UI palette shadow. This is a
separate source catalog from the 162-descriptor BG base/alternate palettes and
is not a complete inventory of every game palette or indirect source.

## Source and consumer contract

Original callers copy into shadow RAM `03003860..03003c60`; its OBJ half starts
at `03003a60`. The apparent lack of direct ROM-to-hardware CpuSet callers does
not imply absent static palettes: these paths call dispatcher `08145c14`,
which branches through r3 to the callback from ROM pointer `08370eb4`.

The pointer value `03005ee9` selects the original Thumb memcpy at `03005ee8`.
Initializer sequence `080027ee..080027fe` uses independently checked literals
`2814/2810/2818` to copy ROM `d96838..d968e8` to IWRAM `03005e78`, 176 bytes,
through original CpuSet. The callback is at source `d968a8`, offset70 in this
block. Read-only parsing of a retained snapshot independently finds its exact
64-byte code prefix uniquely in ROM; the focused Wine oracle then executes
the original initializer. Copied code remains in ROM/IWRAM and is not a data
resource. The separate sprite-code copier was not used to infer this mapping.

Banks preserve exact little-endian color words, including bit15. Colors are
not quantized, converted to RGBA or normalized. Bank spans contain multiples
of16colors; explicit read/count/alignment/overlap guards reject malformed
inputs. A contained reference into `b8c238+c0` shares the existing bank owner.
Two original selectors at `0804fafe` and `0812f27e` read the low two bits of
live byte `02002fc2`: mode1 selects a second palette, mode2 a third, and0/3 the
base. Higher flag bits remain live. Their branch sources are independently
rooted; the data do not flatten them into one palette or invent a fallback.

## Managed and runtime behavior

All16files/15resources are authenticated before attachment; every file and ID
contributes to whole-set identity. Fixed paths and exact ROM provenance reject
coherently rehashed color/bit15/source changes, missing banks and traversal.
The14earlier decoded files remain byte-identical, e19/older generations stay
retained and USA remains e8-d7. The16MiB file bound and model cardinality/ROM
bounds protect parsing; no extracted-space minimization is introduced.

Immutable typed words serve exact ROM bytes through the shared CPU/DMA/native
and guest-BIOS bus paths. Each byte of a crossing read resolves its own bank.
Original selectors, callback instructions, copy clocks, shadow RAM writes,
live fades and palette publication remain active. The PPU reads dynamic palette
RAM, with no added host ROM palette consumer or decompression shortcut.
Nonserialized `bus_reads/source_bytes` and per-bank source-byte counts establish
ownership. Save/native identities and ABI stay source-equal; resource ownership
survives Load/Undo/backup/restart after data files become unavailable.

## Verification

One reduced EUR Wine original-source check passes strict deterministic JSON,
independent source hashes/order/extent guards and coherent bit15-edit rejection,
the176-byte initializer,27actual copy paths including10Color-Mode cases and
four unaligned/byte-tail callback cases. Complete shadow output and guards match.
6196instructions are coverage counts, not performance measurements. An initial
compile succeeded with an unused future native-helper warning; the later
integration uses that helper and builds warning-free, without repeating the
closed source oracle.

Two focused Wine integration checks pass publication interruption/create/reuse/
repair/move/e19retention, earlier-file equality, hash/path/source/coherent-edit
rejection with transactional attachment, and unavailable-copy Load/Undo/backup/
restart. The27actual interpreter/native paths match complete states/clocks with
all21raw spans altered:3528compiled source bytes and every bank's positive
counter prove typed use. All bytes/edges/widths/mirrors, real-BIOS bus,
CpuSet/FastSet and160typed DMA bytes pass. A live shadow edit reaches palette
RAM through DMA. A proposed old-source adjacency probe was corrected before
execution to the actual new-bank boundary40fd50; no source/managed oracle was
repeated for that test-only correction.

One240-frame ordinary clean boot on Wine matches every complete state/clock,
central/wide image, OAM, native dispatch and audio stream, with33new palette
reads/130source bytes. The existing neutral input is reused. Three actual
Windows CLI processes create e20, reuse without writes and replay from unrelated
cwd with the old native cache; the final language-screen image is inspected.
Windows test/production builds are warning-free, the canonical executable is
restored and protected ROM/e19data/cache/loader/input are unchanged.

Evidence stays private under ignored `target/rom-data-obj-palettes/`:
`ui-raw-palette-inventory.json`, `windows-original-result.json`,
`windows-original-europe/result.json`, `windows-managed-results.json`,
`windows-native-results.json`, `windows-managed-europe/*-result.json`,
`windows-cli-summary.json`, `windows-language-scene-result.json`,
`windows-boot-ui-palettes-full-state-audio.log` and `windows-build.json`.
Broader indirect palette roots, other ordinary scenes, GUI/archives, native
Windows and new Linux/USA verification remain pending. No performance/startup/
RSS/space campaign, AndroidPort work, commit or push is part of this change.

## E359 caller-bounded palette projection

The EUR bank `40fd70..40fd90` contains 16 original halfwords selected by the
reader at `080138ec`; the caller at `08013918` bounds its argument to 1–4,
while the reader uses the low two selector bits. Other callers retain original
indexing and fallback. Wine readers, a 606-frame altered-source Firaga scene and
CLI pass for E274-d273. The address-only sample does not prove the caller path;
other scenes, Linux, USA and native Windows remain open.
[Evidence](rom_data_runtime.md#e359-firaga-palette-projection-runtime).

## E637 European B8 screen palette bank

The palette bank occupies `b8b9f8..b8c778`. Existing UI palettes and the
two timeline buffers cover 1,984 of its 3,456 bytes. Three original
loads account for the two remaining physical gaps.

The calls at `08143c4e` and `0814434e` load `b8be58` and `b8c038`
respectively. Both use the original byte-copy dispatcher `08145c14`
and callback `03005ee9` to copy 480 bytes to offset `8bd8` in the
selected screen context. The caller at `08144a8a` passes `b8c378`
to the original staged palette loader `0814b6f0`. It writes 256
halfwords to slots 256 through 511 of the table at `03003e68`,
with a 12-byte record stride.

Wine `z_zmh871` attests 117 original halfwords and the three reader
literals. The e370 ownership audit finds 1,484 unowned bytes across
five spans, including 736 color words and three literal words.
All 15 input guards pass and Wine drains. Result SHA-256
`5695b945d14d51c6465aee720ca62e9c9fefed8ebfdac84134eb902135e18c56`.

## E638 European B8 screen palette runtime

Dataset `e371-d370` publishes all 1,484 E637 bytes in
`screen-palette-bank.bin` as `graphics.screen-palette-bank`.
The manifest identifies each original address and source role.
`graphics.ui-raw-palettes` and `graphics.layered-obj-animations`
supply the established spans of the same physical bank.

Wine `upvkln7r` verifies both original 480-byte copies and the
256-color loop, with raw backing altered across the palette bank
and the three source literals. The original instructions consume
all 1,484 new bytes. All destination guards, the ten other bytes
per staged record, 480 mixed and mirrored boundary reads, complete
guest state and clock match. The staged path stops at the end of
the color loop before it schedules updates.

Windows GNU build `ci_n7_ch`, e370 attachment, invalid candidate
rejection and CLI creation, verification and reuse pass. All 171
earlier resources are byte-identical. All 17 Wine input guards pass
and the server drains. Stage `dxp4zz3_` retains 173 files. Result SHA-256
`d0db93ada4cdde68ce2223190b80f3ad1faceda69c1b7bd5e408ace06ec288d6`.

The recovery checklist tracks ordinary screen triggers, scheduled
updates, translated readers, Linux, USA and native Windows.
[Recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).
