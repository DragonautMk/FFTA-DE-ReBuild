# Raw UI palette resources

EUR schema 1/extractor 20/decoder 19 adds fixed `ui-palettes.json`, resource
`graphics.ui-raw-palettes`, format `ui-raw-palettes-v1-json`. The declared catalog
contains 21 raw banks/1328 full u16 color words, 21 disjoint source ranges/2656
bytes and 23 checked direct, derived or conditional references. These colors
serve both BG and OBJ halves of the original UI palette shadow. This is a
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
