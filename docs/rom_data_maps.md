# European map and terrain sources

`ROMDATA-01-TERRAIN-MAP-SOURCE` closes the source classification of the normal
map loader for the authenticated 162-descriptor EUR block. Its static sources
already belong to `graphics.bg-descriptors` and `graphics.bg-packet-programs`.
This adds no decoder, resource, revision or runtime shortcut. Other map paths
remain unclassified where stated below. Offsets are EUR file offsets; addresses
with an `08`, `02`, `03`, `05` or `06` prefix are guest addresses.

## Original loader and static ownership

The setter at `0801B270` narrows its first argument to u16. Zero selects the
`00FF` sentinel; other arguments become `(argument - 1) & FFFF`. It stores this
descriptor index at `02007F10`. It also obtains descriptor selector field54/55
through `0801ED08`, storing that result at context+12. Neither this setter nor
the inspected loader introduces a global 162-record guest limit. The declared
source catalog is bounded independently by the existing descriptor contract.

The staged loader at `0801B688`, state6 at `0801BAB4`, uses root `087662A8`
and stride `58`. Its normal branch at `0801BAF0` constructs these arguments
for `0801AE88`, called at `0801BB36`:

- r0: descriptor field10, auxiliary patches, added to the descriptor root.
- r1: field04, paired planes, added to the same root.
- r2: field08, halfword planes, added to the same root.
- r3 and the fifth argument: 32 and 32, used by the original scene setup.

`0801AE88` calls the original plane reset at `0801C838`, then wrappers
`080215F4`, `080215E8`, `080215DC`. These call the already classified
`080205A8`, `0802045C`, `08020318`, respectively. Their sources are the existing
auxiliary, halfword and paired packet programs. There is no separate discovered
terrain archive behind this normal branch.

The source audit checks every one of the 162×3 links against the original
descriptor words, authenticated JSON program IDs, original encoded sources and
manifest ranges. It verifies exclusive ownership against all 49 e54 resources:

- 122 paired programs, 236091 source bytes, 162 links and 40 alias links.
- 117 halfword programs, 143317 source bytes, 162 links and 45 alias links.
- 113 auxiliary programs, 18819 source bytes, 162 links and 49 alias links.

The 352 ranges total 398227 source bytes and 523794 post-wrapper body bytes.
The 14256 descriptor bytes reconstruct exactly. Existing raw/LZ77 wrappers,
ordered writes, dictionaries, inheritance and source extents remain those of
the [packet contract](rom_data_graphics.md#complete-descriptor-packet-catalogs).
There are 9/6/5 distinct parented programs but 9/6/9 parented descriptor links;
the auxiliary difference is sharing. Maximum chain length is two nodes.
All 162 optional field50 variant pointers are zero in this declared block.
Main graphics, base/alternate palettes and transfer programs retain their
existing separate owners; this classification does not change those resources.

## Live terrain and display state

At `0801AEDA`, the normal builder stores `02007CB0` at context+4 and sets
width and height at context+8/+9 to 16. The auxiliary program initializes a
512-byte grid of 256 little-endian cells. `0801DD68` computes a cell pointer as
`live_pointer + 2 * (u8(y) * live_width + u8(x))`. `0801DD88` reads its first
byte as height. `0801DDA8` can subtract one when the second byte has bit1 set
and the caller's low flag bit is clear, returning a signed 16-bit result.
`0801DE78` reads the second byte; `0801DE98` writes that byte at `0801DEB0`.
The [constructor-animation follow-up](#constructor-animation-cell-flag-seeds)
now ties flag02to a conditional slot gate and classifies its static seed
candidates; complete flag meanings and live writer reachability remain open.

Consequently, decoded static patches initialize terrain; they cannot replace
the mutable terrain grid or its current pointer/width. The sentinel loader
branch clears the 512-byte buffer and stores its pointer; it does not execute
the normal dimension-setting builder, so the normal 16×16 assignment must not
be generalized to every possible context state.

Paired programs initialize the two calculated 64×64 planes at
`020091A0..0200D1A0`, with the original live fill word at `02007F5E`.
Halfword programs use the next 8192 bytes at `0200D1A0`. Original producers
retain layout calculations, fills, patches, camera transformations and DMA.
`ffta_stage.rs` observes calculated planes and their VRAM crops. Movement
overlays at `02003CB0` and `02005CB0` are also live calculated data; the first
buffer is reused as packet decompression scratch. These EWRAM/VRAM buffers
are not additional static assets to export.

## Scope and remaining sources

`0801DFB4` uses the same live pointer and width with a ten-byte stride and
reads byte4. A static scan below offset370000 finds no direct Thumb BL or
aligned Thumb pointer literal to this accessor. This does not establish dead
code, a second static archive or a global two-byte layout. Its caller and
possible alternate context remain unclassified.

The selector path also exposes a distinct root `084872B0`, through the literal
at `0801EE24` and lookup `0801EDEC`. The separate source contract below now
bounds this map raster palette family. Its typed model and targeted runtime
migration now pass below; it is separate from the already migrated terrain seeds.

## Validation and limits

The new check is static only: exact EUR ROM identity, two authenticated dataset
files, descriptor reconstruction, 486 link/source formulas, all 352 source
hashes and exclusive owners, inheritance, plus 40 instruction halfwords,
11 literals and nine original BL edges. One hand-transcribed STRB fingerprint
initially used the wrong base-register encoding; correcting that private
expectation to the inspected `704A` makes the check pass. No product source or
existing decoder/oracle changed.

Private evidence is under `target/rom-data-terrain-maps/`: disassembly,
`verify_source_contract.py`, `source-contract.json` and
`descriptor-packet-links.json`. It contains no new guest execution or build.
Earlier packet source/original-loader, typed native/managed/lifecycle and
ordinary Wine results remain closed and are referenced in the graphics
contract; they were not replayed for this semantic classification. No new
end-to-end map-loader execution or full flag/caller semantics is claimed.
This classification left EUR v1/e54-d53 and USA e8-d7 unchanged. Later raster
publication is described below. New Linux, USA and native Windows checks remain
pending. No performance or space measurement was performed.

## Map raster palette source contract

`ROMDATA-01-MAP-RASTER-PALETTE-SOURCE` identifies an uncompressed EUR bank at
`4872B0..489BB0`, 10496 bytes with SHA-256
`6570159e474457326a39cc65bab5530c9467f74470d4c4a2a0f2cb2e19e13df3`.
Original functions `0801EAF0` and `0801EDEC` independently use this root;
the only aligned literal occurrences are `01EB2C` and `01EE24`.

The first relative u16 is `0040`, placing the first record after 32 little-endian
u16 directory entries. All 32 values point to distinct consecutive records;
all IDs0..31 also occur among the 324 field54/55 links of the 162 authenticated
BG descriptors. The final record's independently decoded length ends exactly
at the known alternate-palette root `489BB0`. These checks agree on the
declared 32-record projection. Neither original reader checks a global ID<32
condition, and the extraction bound must not become a new guest limit.

Each 326-byte record contains three u16 header words followed by 160 indexed
BGR555 color words. All current headers are `[160,160,0]`. Retain the complete
u16 words, including bit15, and the physical record identity even when contents
are equal. There are 32 distinct source records but 26 distinct color contents:
pairs0/16,1/17,3/19,5/21,7/23,15/31 match in content, not source address. There
is no compression, alignment gap or additional directory in this projection.
The 33 source ranges partition 64 directory bytes, 192 header bytes and 10240
color bytes without overlap with any of the current 49 managed resources.

At `0801EAF0`, the input index is narrowed to unsigned u16. An existing live
`00FF` selector at `02007F20+2` prevents the lookup. Otherwise it reads the
relative directory entry and adds six to obtain the color pointer. It stores
header0 at live+4; header4 at live+C; header2+header4-1 at live+E; the color
pointer at live+10; and `colors + 2*header0 - 320` at live+14. These are the
observed formulas, not a complete interpretation of camera/scroll metadata.
The update routine `0801EB80` uses live header0<=160 to choose the direct
color window; larger live values have a separate camera-dependent path.
Every canonical record follows the direct window path, but live mutations
and the larger path must keep their original behavior.

`0801EDEC` has a different raw-argument contract: it first checks the current
selector at `02007F10+12` against `00FF`, returns zero when disabled, and
substitutes that live selector only for raw argument `FFFF`. It then uses
signed16(argument)*2 to address the directory and returns `root + relative + 6`.
Other out-of-projection inputs retain their original raw memory behavior.
These two readers must not be conflated into a newly capped host lookup.

### Exact copy boundary and raster setup

Calls at `0801EB38`, `0801EC3C`, `0801EC80`, `0801EE58`, `0801EEF6` reach
`08145230`, whose original instructions are `SWI 0B; BX LR`. Control `00A1`
selects 161 halfword copies, 322 bytes, not 161 words. They copy to `02008390`
or defer the source pointer in live state. The original code then overwrites
destination halfword160 with halfword0. The extra source read is still part of
the original bus accesses and timing; it cannot be dropped as an optimization.

Each canonical source window therefore reads one halfword beyond its 160
colors. In records0..30 that word is the next record's header0 (`00A0`). The
last read is at `489BB0`, value `013C`, owned by the existing 314-byte
`graphics.bg-alternate-palettes` index span. It stays with that owner. The new
source projection ends at `489BB0`; it does not duplicate or steal these two
bytes. Future managed/native checks must cover a read spanning both owners.

`0801ECAC` copies the first RAM color to the palette shadow at `03003860` and
schedules channel2 with source `02008392`, destination `05000000` and input
control/count `20400001`. Original helper `08000D80` ORs `A2000000`, queuing
`A2400001`: one halfword, incrementing source, fixed destination, repeat and
HBlank timing. This is the source-level basis for calling the bank a raster
palette. It does not establish an executed DMA trace or a game-level sky name.
Guest scheduling, enable/disable, deferred copies, RAM edits and raster timing
remain original work; a static asset cannot replace these states.

### Inventory validation and remaining migration

The private static parser reconstructs every byte and verifies all 324 links,
33 range hashes, all 49 prior owners, the extra read ownership, 67 instruction
halfwords, 13 literals and ten BL edges. It passes on its first execution.
Header0-backed extents and directory/descriptor links provide the bound;
the next resource root is an independent cross-check. No build, decompression,
guest invocation or previous test replay was needed. Evidence under
`target/rom-data-map-raster-palettes/` includes `source-plan.json`,
`source-contract.json`, disassembly and `verify_source_contract.py`.

A strict lossless model and one reduced Wine original-reader/copy oracle now
pass below, followed by managed publication and targeted bus/native/lifecycle
consumption. Actual production CLI reuse and a positive ordinary Wine pair
now pass in the final section. Preserve the raw signed/unsigned reader
differences and the mixed owner boundary. The static inventory left EUR e54-d53
and USA e8-d7 unchanged; later publication advances EUR as described below.
New Linux, USA and native Windows checks remain pending. No performance or
space measurements were made.

## Typed raster palette model and original-reader oracle

`ROMDATA-02-MAP-RASTER-PALETTE-MODEL` adds the isolated
`rom_data::map_raster_palettes` module. `MapRasterPalettes` holds 32 physical
`RasterPalette` records and 162 pairs of descriptor program links. Each record
retains its source offset/ID, typed three-word `RasterHeader` and all 160 color
words. Index bytes derive from physical source offsets. There is no raw ROM
shadow, flattened image, compression or content-based merging of records.

Validation checks the declared contiguous index/record extents, ordered IDs,
header/color counts and descriptor targets. Actual BG descriptors are a checked
dependency of the derived links. Authentication reconstructs all 33 original
source spans and compares the exact EUR source, fields and hashes. Structural
acceptance of changed color/range words does not authorize modified datasets;
`verify_source` rejects them against the canonical ROM. The neighbor halfword
at `489BB0` remains outside `byte_at` and all new source spans.

The public arithmetic helpers describe the unsigned initializer and signed
pointer-getter index formulas, including the latter's live `00FF` gate and
`FFFF` substitution. They describe source addresses; they do not impose host
bounds on guest arguments. The only existing source edits register the module
and its new private test. That model stage left startup, managed files,
revision and runtime readers unchanged.

One affected Windows GNU frontend test build passes without diagnostics. Its
frozen executable SHA-256 is
`6434366e7fad4ada6ff0bf10bdfba86d5082671606cfd8358dd92aaed67315b0`.
One new exact Wine filter passes on its first execution, verifying:

- All independent source-plan fields, 33 spans/hashes, 5120 color words and
  324 descriptor links, exact JSON roundtrip and 15 shape/dependency/JSON/source
  guards. Every new source byte matches the independent static inventory.
- All 32 original initializers with immediate and deferred paths (64 calls),
  32 canonical pointer getters, 11 raw/sentinel getters, seven unsigned raw
  initializer prefixes and three disabled initializer calls. High argument
  bits, negative signed indices and raw indices outside the projection retain
  their original behavior.
- Two deferred updates, both explicit-source setter modes and one original
  queued DMA setup. The latter checks source, destination, channel flags and
  `A2400001` in IWRAM; it is not an executed raster DMA campaign.
- All 36 original `CpuSet` observations compare the entire 322-byte destination
  immediately after the SWI and before wrap. The final source halfword `013C`
  is observed at `489BB0`; original caller code then replaces destination
  word160 with word0. Next-header `00A0` reads are also observed.
- One synthetic original initializer preserves a high color bit and changed
  range words, including wrapped range metadata. All calls check the complete
  EWRAM and IWRAM outside declared stack scratch, restored SP and callee-saved
  registers; expected live metadata/shadow changes remain explicit.

The original loadsite observations plus checked HLE copies cover all 10496
owned source bytes. The 4753 instructions and 202 PCs are coverage counts,
not performance measurements or a full state/clock equivalence proof. The
component oracle does not validate native consumers, full camera/ABI behavior,
executed raster timing or ordinary gameplay. All 643 hash/mtime guards pass.
No older source/scene tests, fixtures or builds were repeated for context.

Evidence under `target/rom-data-map-raster-palettes/` includes
`windows-model-build.json`, the frozen `recomp-tests-model.exe`,
`windows-model-result.json`, `windows-model.log` and
`windows-model-europe/{map-raster-palettes,result}.json`. That stage left
managed/runtime/CLI/ordinary work pending and EUR e54-d53/USA e8-d7 unchanged.
The next section closes managed publication and targeted consumption.
No save/native ABI, performance, space, Android or package change is implied
by the source model.

## Managed raster palette publication and typed reads

`ROMDATA-03-MAP-RASTER-PALETTE-RUNTIME` publishes
`map-raster-palettes.json` / `graphics.map-raster-palettes` under EUR
v1/e55-d54. The generation has 51 files and 50 resources. All 49 previous
decoded files remain byte-identical and the old e54 generation is retained.
USA remains e8-d7. The resource authenticates its actual
`graphics.bg-descriptors` dependency and all 33 source ranges, 10496 bytes.
The complete EUR resource set is required independently of that semantic link.

Dataset identity is
`cfc4012ce77ff78dd5e77f2b2ff9cb247a62b94e05d0a2fb80e24759a4e321b1`;
the new file SHA-256 is
`d263f1a538fa9bd8c02225c4a6c41b6931b1c85f0e13d886f66a437e22abe9e8`.
These identify the accepted targeted-test generation and identical later
production CLI result. Bounded JSON parsing and source validation precede
activation of one immutable typed owner; no raw ROM shadow or per-read file
access is added.

The owner serves index/header/color bytes through existing CPU, DMA, BIOS HLE
and native ROM APIs. Its page selection includes 487..489. Each read overlays
only its owned bytes on the result from earlier owners. A read across `489BB0`
therefore combines new color bytes with the existing alternate-palette index.
The neighbor is neither duplicated nor taken from poisoned raw ROM. Unowned
neighbors retain the original raw behavior. Usage counters report aggregate
byte reads by source class and a distinct program mask; they do not measure
distinct bytes read during ordinary gameplay.

Original signed/unsigned index arithmetic, live windows and selectors, 322-byte
copies, extra source reads, destination wrap and DMA scheduling remain guest
work. No instruction or cycle is skipped. Save/native/cache identity and state
formats are unchanged; activation remains transactional and the shared owner
survives file loss, Load, Undo, backup restore and restart.

One affected Windows GNU core/frontend test build passes without diagnostics.
The frozen frontend SHA-256 is
`bd3e69fda059e58e4c1932122220fac2304de40cc0a7f3e22f17323a02cfef2a`;
the affected core tests compile without being executed. Two new exact Wine
filters pass on their first executions:

- Managed authentication/lifecycle checks pass 17 coherent or missing-file
  guards, actual descriptor dependency and complete-set checks, failed-write
  isolation, create/reuse/repair/move, all 49 prior file comparisons and e54
  retention. After files become unavailable, Load/Undo/backup/restart retain the
  same owner. With raw sources poisoned, reads cover exactly 64 index, 192
  header and 10240 color bytes across all 32 programs. A mixed word independently
  increments new and alternate-palette counters by two bytes each.
- 28 original/interpreter/compiled cases match complete serialized states and
  guest clocks. They cover immediate/deferred initialization, getters and raw
  prefixes, signed/unsigned arguments, sentinels, live windows, explicit source
  setters, original copies/wrap and queued DMA configuration. All 10496 new raw
  source bytes and the adjacent 314-byte old index are poisoned. Compiled API
  reads serve 15 index, 46 header and 24 color bytes from typed values.
- Additional width/mirror and real-BIOS **bus-mode** reads pass, including mixed
  owners and an unowned raw neighbor. Four original CpuSet/FastSet comparisons
  and one immediate DMA comparison preserve full states/clocks. The last DMA
  reads two bytes from each owner. This does not claim a complete real-BIOS
  execution, whole camera ABI or an executed HBlank raster campaign.

All 646 hash/mtime guards pass. The 638 original instructions, 204 PCs and 55
native dispatches are coverage counts only. The previous source-model oracle
and older runtime/scene proofs were not repeated. Evidence under
`target/rom-data-map-raster-palettes/` includes `windows-managed-build.json`,
`windows-managed-results.json`, `runtime-targeted-outcome.json`, logs and
`windows-managed-europe/{managed,native}-result.json`.

The production CLI and ordinary Wine criteria now pass below. New Linux,
USA, native Windows and new 32:9 checks remain pending.
No performance/space measurement, new gameplay fixture/navigation, Android,
package, agent commit or push was performed.

## Raster palette production CLI and positive ordinary boot

`ROMDATA-04-MAP-RASTER-PALETTE-CLI` and
`ROMDATA-05-MAP-RASTER-PALETTE-PAIR` close the declared EUR/Wine production and
ordinary-use criteria. One required Windows production build passes without
diagnostics; its frozen executable SHA-256 is
`43c9992fc68038796f9c21066d536dd23c568e44963dfa295d2b81f1beebdf77`.
The canonical production executable's exact hash and mtime are restored.
The existing Cargo workflow then invalidates its output assumption by touching
`main.rs`; that file's content is unchanged.

Actual CLI commands from an unrelated working directory create and reuse e55,
with all 53 managed files retaining their hashes and mtimes on reuse. Every
one of the 51 generation files matches the accepted targeted dataset. One
1500-frame cold-boot locator loads the previous native cache and uses the
existing `target/desktop-perf/state-replay/boot-regression/demo.input`, SHA-256
`9152ab80916c04cee8234e7cb17d0cc97fcb73ddfb7e1a1b55452327afb1b322`.
No initial state, cartridge save, PC/RAM override, new fixture or navigation
is supplied. This is a new-family use check, not a repeat of earlier accepted
boot proofs.

The locator is positive for all three source classes: 25440 bus reads,
50880 aggregate bytes = 2 index + 322 header + 50556 color bytes. The program
mask reports two touched source records; reading an extra next-record header
also marks that record. This is not evidence of two complete palette
initializations or 50880 distinct source bytes.

One Wine full pair then reuses that actual CLI generation, frozen targeted
test executable, old cache, input and exact 1500 frames, with widescreen0.
Every complete serialized guest state/clock, central/wide image, OAM and
dispatch result matches. Mixed, PSG, FIFO and bitwise HLE audio match with
sample counts `[3291096,3291096,395212,395212,3269758]`. The new-family counters
match the CLI locator exactly. The run mixes native and interpreter paths;
it does not establish an all-native or new 32:9 result. All 701 hash/mtime
guards pass.

Evidence under `target/rom-data-map-raster-palettes/` includes
`windows-build.json`, `windows-cli-{processes,no-write,summary}.json`,
`windows-cli-boot-raster1500.log`, `windows-scene-result.json` and
`windows-boot-raster-full-state-audio.log`. Source/model/managed/native checks
were not rerun; no new implementation change or test build was needed for
this production/ordinary closure. Other ordinary palette/camera contexts and
whole raster/camera ABI remain outside the declared scene. New Linux, USA,
native Windows and new 32:9 checks remain pending. No performance/space
measurement, package, Android, agent commit or push was performed.


## Constructor-animation cell flag seeds

The normal terrain loader and its BG ownership were already closed by the
24October source classification above; they are reused here. The new criterion
is the origin of cell flag02needed by the constructor-animation slot gate.
No separate terrain archive, decoder or resource is introduced.

A targeted scan of the exact16MiBEUR image finds no direct ThumbBL or aligned
Thumb-pointer encoding for the1DE98flag setter. This does not prove dead code,
absence of indirect callers or immutable live flags. The cell-pointer getter
1DD68has one directBL encoding at1D798; its bounded authenticated caller window
reads the flags byte and tests02, providing another consumer rather than a
new writer route.

The static seed source is already available in the authenticated e77
`bg-packet-programs.json`. A new semantic query applies its ordered auxiliary
u16writes to the512-byte grid, starting with the existing zero-fill or parent
image before child writes. This reuses the accepted packet representation and
source semantics; it does not rerun the old codec or original-loader oracle.
Among113auxiliary programs linked by162descriptors,25programs initialize cells
with flag02and44descriptor IDs refer to those seeds. Observed seed flag-byte
values are0,1,2,3,4,8and9. The live pointer/width and later writes remain guest
state. The inspected normal builder window1AF08..1AF6Creads height bytes to
calculate bounds and contains no flags store in that window.

The44descriptor IDs are source-backed candidates, not normal gameplay
reachability or a mapping to named battle locations. Their detailed cells,
programs, parents and seed hashes remain in private `typed-cell-seeds.json`.
The previously inspected81snapshots still cover only their two02-free maps;
that negative bound is not reopened. The [script-map follow-up](#script-map-selection-and-remaining-battle-reachability)
now links19literal instructions to eligible seeds. Reaching a suitable normal
battle remains a separate open criterion, without forcing map IDs or guest fields.

All24actual source/tool/old-receipt/current-dataset/model/output guards and four
bounded source-window hashes pass. No implementation, revision, build,
gameplay, Wine/Linux test, capture or checkpoint is added. Required artifact
completion finds zero eligible paths. A private closure-script quote typo was
corrected before it could execute; the premature cleanup attempt refused the
unit without a terminal summary, then completed after the summary existed.
No product or evidence result changed. Evidence is under
`target/rom-data-cell-flag-writers/`: targeted candidate JSONs,
`typed-cell-seeds.json`, `source-contract.json`, final summary, preservation and
cleanup receipts. Static disassembly is not Linux gameplay validation.

Positive constructor-animation ordinary use still needs all four phases and
all four lists through guest and host readers before the fullpair. Linux/USA/
native Windows/new32:9 and the full ROM TODO remain pending. No performance/
storage campaign, Android/package, assistant commit or push.


## Script map selection and remaining battle reachability

The actual source path now links the eligible seed IDs to map-selection
instructions. The normal state handler253C8has a guarded35-entry dispatch at
2540C; state10reaches2549C. It reads variable7throughCCF00from02002037, narrows
the result to8bits, calls1B270at254A8, creates the1B689loader and advances to
state11. The setter's existing unsigned-minus-one/zero-sentinel contract remains
unchanged. The2540Caddress is table data, not the handler's entry.

The authenticated script command directory binds opcode12, advance2, to Thumb
callback126831. A nonzero instruction byte1is written to variable7through
CCF18. Zero instead reads byte2at `70C8CC + 40 * U16(02002170)`, using0A104to
obtain the live index, and writes that value to variable7. Both paths set the
normal state byte0200F390to10. A51encoding direct-call census forCCF18selects
the two adjacent literal7arms for authentication; the other49candidate dataflows
are not classified here. This does not prove that all variable7writes have
been found.

A new semantic query of the accepted typed command directory and script archive
finds204opcode12instructions:56use the live default and148carry nonzero literals.
Nineteen literal instructions select ten raw map arguments whose declared BG
seeds contain cell02:31,33,34,73,75,89,91,109,115and129. Exact source/program/
bank/event-key references remain in private `script-map-links.json`. These are
conditional code-to-asset links, not proof that an event or battle is reachable
in ordinary play, nor a mapping to a named location. Archive event-key IDs and
the live active record index must not be equated without another source join.

Read-only fields from two already authenticated normal representatives provide
consistent default-path examples: one has active index35, map argument99and
loaded descriptor98; the other has197,85and84. Their source record byte2values
are99and85respectively. Both maps are ineligible, matching the already closed
retained-map bound. This does not attribute an executed opcode or replay the
81snapshot search.

All37actual source/tool/current-dataset/parser/normal-state/output guards and
nine bounded source-window hashes pass. No build, gameplay, Wine/Linux test,
capture, checkpoint, decoder, resource, ABI, cache or save change occurs. This
source linkage child is closed; normal eligible-battle selection remains
partial. No managed/native validation or disposable fixture copy is created.
Evidence: `target/rom-data-constructor-eligible-map-selection/`, including
`source-contract.json`, `script-map-links.json`, `retained-selector-contexts.json`,
targeted caller candidate JSONs, final summary and preservation receipt.

The [active-index follow-up](#active-index-producer-and-event-record-links)
authenticates the producer and conditional event joins. The full40-byte catalog
extent and an eligible quest available in normal play remain open. No guessed map ID, guest-memory edit or navigation is authorized by
these static links. Positive constructor-animation ordinary acceptance still
needs all four phases and all four lists through guest and host readers before
a fullpair. Existing source/runtime/scene/census,81map and five Move negative
bounds remain closed. Linux/USA/native Windows/new32:9 and the full ROM TODO
remain pending; no measurements, package, assistant commit or push.


## Active index producer and event record links

The EUR setter `08009CF0` writes the low16 bits of r0 to `02002170` and r1 to
`02002172`, without an index bound. An exact-image encoding census finds two
direct Thumb BLs, at9D2C and1273A8, and no aligned Thumb-pointer encoding for
this setter. This is not a census of all stores to either field or a proof
that indirect callers are absent.

The first caller belongs to9D04. When the supplied event row has nonzero u16
at+4, it selects a12-byte record at `71D1EC + 12 * low8(arg1)` and passes its
halfwords+2/+4 to the setter. Otherwise it passes the event row's halfword+2
and zero. A source path through63878 state3 and a helper condition reaches
63944: u16 at0200219E selects a70-byte record at713B3C, whose byte45 selects a
12-byte event row at71C760. The call at6397E supplies this row and its byte0
to9D04. These are conditional original code paths; no claim about an executed
menu action follows from static inspection.

The other setter caller is in script callback127388. Its little-endian u16
parameter is compared with event-row byte1 while scanning rows255 down to1.
The first matching row supplies halfword+2 and zero to the setter. Consequently,
duplicate keys select the highest matching row and row0 is excluded from this
scan. Event keys, event-row indices and active40-byte record indices are
separate selectors; this source join now connects them conditionally.

A new semantic query uses the caller's256-value u8 event-row domain, not an
assumed archive extent. It finds208 rows taking the direct-primary branch,
54 rows whose primary map field selects a declared cell02 seed, and44 rows
with both the direct-primary branch and a referenced default-map opcode12.
Detailed joins remain in private `event-map-leads.json`. Eight event-key groups
have duplicates. The maximum primary index442 is an observed reference, not
a count or upper bound on guest access. A separate helperD2760 returns300;
it is not evidence for the extent of the40-byte catalog. No complete catalog
schema, resource or decoder is introduced.

Read-only inspection of the already authenticated pub checkpoint finds selected
index0200219E=0 and event row0, whose direct primary selects raw map11 without
cell02. This is not evidence about missions subsequently visible in its menu.
The first private parser attempt stopped for a missing Python struct import
before producing output; the corrected read preserves the original state.

All42 source/tool/prior-proof/parser/state/output guards and23 bounded source
window hashes pass. This closes the producer and conditional-join source child;
normal eligible-battle reachability and full catalog extent remain partial.
Evidence is under `target/rom-data-active-encounter-records/`: source contract,
caller candidates, event-map leads, retained pub selector, final summary and
preservation receipt. No build, gameplay, Wine/Linux test, capture, checkpoint
or managed/native disposable copy is created. Static disassembly is not Linux
game validation.

The [normal pub-menu follow-up](#normal-pub-menu-identifies-an-eligible-quest)
now identifies an available eligible quest and confirms its detail selector.
Acceptance and actual battle entry remain open.
No forced index/map/RAM or further blind Move attempt follows from these links.
Positive constructor acceptance still requires all four phases and all four
lists through guest and host readers before a fullpair. Closed source/runtime/
CLI/scene/census/81-map/five-Move results remain closed. Linux, USA, native
Windows, new32:9 and the full ROM TODO remain pending; no performance/storage
campaign, Android/package, assistant commit or push occurs.


## Normal pub menu identifies an eligible quest

The missing menu endpoint is now retained through normal game input on exact
local copies of the authenticated EUR pub checkpoint. One620-frame Wine route
uses the existing input to open the mission list, ending at guest frame14969.
A second300-frame route presses Down four times and A, ending at15269 with the
White Flowers detail screen. Both use the existing frozen309ba31c observer and
accepted e77 dataset; there is no build. The former620-frame scene acceptance
is not replayed as acceptance: the new deliverable is its missing machine
endpoint and available-quest selectors.

The live list context at[0200F44C]=02024D5C has14 pointers and count byte+1B.
Original62126..6215E binds the pointer at+20+4*ordinal to the mission ID encoded
as byte0|((byte1&3)<<8). A new query of these14 live entries, joined to the
closed event/map source proof, finds two direct-primary/default-script/cell02
candidates: mission IDs58and72. The first is the fifth visible menu row.
After normal selection, the image visibly identifies White Flowers, while
[0200F448]=0202389C and its+1450 pointer selects02002268 with mission ID58.
Original60208..60228 and its literals authenticate this selector projection.
Its source link is event row58, primary index315, raw map69/descriptor68 with
cell02 seeds. The detail names Giza Plains as its destination. This establishes
an eligible quest present in the normal menu, not an accepted mission or an
executed battle/map load.

Exploratory031C/03D windows referring to03001028 were not linked to the live pub
menu; the observed IWRAM word is an executable encoding, so that candidate
context layout was rejected. It is not used to decode the mission list. The
selected source projections above are authenticated separately.

Both Wine workers pass and retain normal endpoint states/images/logs. Their
constructor counters stay zero in the pub, as expected; this is not a positive
animation or equivalence result. All130 actual input/dataset/output guards,
496 frozen source hashes and23 bounded source-window hashes pass. Originals,
source copies and70 accepted dataset files preserve hashes and mtimes. There
are no manual RAM/task/ROM edits or cartridge-save writes. The artifact helper
finds zero disposable paths and keeps the new normal checkpoints. Evidence:
`target/rom-data-eligible-quest-locator/`, especially source contract, final
summary, preservation and cleanup receipts, `windows-menu/` and
`windows-white-flowers-detail/`.

The [acceptance follow-up](#normal-white-flowers-acceptance-and-open-battle-entry)
now proves payment and active mission status. Actual battle entry remains open. No forced map/index or blind Move follows from
the source link. All four constructor phases and lists through guest and host
readers remain required before the ordinary fullpair. Old source/runtime/CLI/
scene/census/81-map/five-Move children stay closed. Linux, USA, native Windows,
new32:9 and full ROM_DATA_TODO remain pending. No performance/storage campaign,
Android/package, assistant commit or push occurs.


## Normal White Flowers acceptance and open battle entry

Normal input from an exact copy of the confirmed White Flowers detail completes
the loadout/default OK, fee/objective and conditions/Yes flow. The payment
endpoint shows300gil paid and funds reduced from7709to7409. A later quest-detail
screen explicitly shows Fee:Paid for White Flowers. This closes normal
acceptance of the eligible mission58; it does not prove a battle/map load.

One menu item was initially misread as Quit Pub. It is Quit Mission: that
selection opened the accepted-quest cancellation list, and A then opened its
detail with the A:Cancel label. No A was pressed on that cancellation detail.
The next route backs out with B, selects the actual fourth-row Leave and reaches
the world map. The paid mission remains active. Both the mistaken interpretation
and its correction are retained in the private step reasons and contract.

Twenty-seven bounded normal-input Wine captures,4840executed frames in total,
reuse the existing309ba31c observer and e77 dataset. These are navigation
checkpoints, not27equivalence acceptances. All workers pass; constructor table
use stays zero and no fullpair is run. Four cursor endpoints are identified by
the game's visible labels: Cyril, Lutia Pass, Nubswood and Sprohm. The unnamed
central patch returns the cursor to Cyril when selected and is excluded as a
new destination. Long cursor holds also scroll the camera; approximate pixel
estimates overshot several targets. Southern, eastern and northern viewport
bounds are retained, without inferring that Giza Plains is absent or unplaced.
Further blind cursor trials are not selected.

The source follow-up is incomplete. Seventeen bounded exploratory windows have
exact hashes, but92524is an initializer and stale task arrays do not prove the
active world dispatch. The031C/03D03001028 context is not linked to the live world
map. Approximate private font diagnostics locate English catalog165E8name
indices719Cyril,724Nubswood,725Giza Plains and729Lutia Pass; a3EE60list builder
uses a718base. These are discovery leads, not a verified placed-region/coordinate
join or a new decoder. They must not be used to force a location or guest ID.

All337 actual input/dataset/output guards,496 frozen source hashes and17 source
window hashes pass. Originals,70 dataset files and current executables remain
unchanged. All new snapshots derive only from normal keys on local copies;
there are no manual RAM/task/ROM edits or cartridge-save writes. No build or
runtime implementation changes occur. The partial navigation unit retains its
diagnostic checkpoints and existing executable; no artifact retirement is run.
Evidence: `target/rom-data-white-flowers-battle-navigation/`, including
`source-contract.json`, final summary, preservation receipt and27separate
step/input/log/state/image groups. The latest endpoint is the north-pan branch;
the Sprohm-labelled step20is a useful normal resume point.

Acceptance is closed, while actual Giza Plains battle entry, eligible actor/
cell state and positive constructor use remain open. The [world-selector follow-up](#world-selector-and-placed-region-coordinates)
now proves Giza Plains is placed and supplies its coordinates plus a retained
nearby cursor state. Battle entry remains open. All four phases and lists through
guest and host readers remain required before the ordinary fullpair. Closed
source/runtime/CLI/scenes/81-map/five-Move bounds stay closed. Linux, USA, native
Windows, new32:9 and full ROM_DATA_TODO remain pending; no performance/storage
campaign, Android/package, assistant commit or push occurs.


## World selector and placed region coordinates

A new feature-gated test records one neutral normal world frame through the
existing address/site capture API and writes the actual EWRAM/IWRAM to private
files. One warning-free Windows build uses a stable497-file e77 source copy;
one Wine run covers guest18829→18830. It records2232unique reads,2276sites and
323sites with interpreter/HLE scope. Native/host sites can have no instruction,
and nested accesses inherit scope: inspect the opcode before attribution.
This is source discovery, not a scene-equivalence or performance result.
The raw EWRAM matches the retained snapshot parser byte for byte.

Two inspection errors are corrected. The actual254B2call targets6B40; the
previous guessed7B40window was unrelated. Original6B40/6A58use the scheduler
count byte+2 and per-slot status bytes, not the word+4 used by the earlier
candidate query. More substantially, the literal low halfword2810/high0300
means03002810. Earlier probing at03001028read the wrong address, so the old
world-layout rejection did not follow from that probe. The actual normal
world parent at[03002810] is0200F3A4. Its save/view pointers both happen to be
02002C10in this snapshot; this is not a universal alias rule.

Original375F8uses the pointer at parent+067C, here08395CA8, and loops over30
nodes at20-byte stride. The first two bytes give unsigned tile X/Y; multiply
by8and add8to each for the centering target. The conditional hitbox is
Xtile*8−4..Xtile*8+20 and Ytile*8−8..Ytile*8+16. Conditional centering moves one
pixel per call. At37D6A..37D70the caller supplies view+6/+8 as cursor X/Y.
Original37A44subtracts view+Crepresenting vertical scroll when drawing cursor Y.
Thus screen Y and world Y must be distinguished.

Original385CC..38668uses save-base+B0+12*region index, checks byte1bit0 and reads
byte3as a one-based node selector. The coordinate source is node-root+
20*(selector−1). The existing name-catalog join is reused. Giza Plains at region
index7/catalog725has selector20and coordinate source395E24: tile13/36, world
center112/296. It is placed. The source join also agrees with the known Cyril,
Sprohm, Nubswood and Lutia Pass labels. These are coordinate/placement semantics,
not a complete schema for the20-byte node records. The other18bytes, complete
model/extraction and managed runtime for this newly identified family remain open.

A new read-only criterion examines only the retained southern endpoints:
step21has cursor128/287, step22has128/330, while the labelled Sprohm step20has
112/168. This does not replay the27navigation bounds. A short Left press from
step21can enter the Giza hitbox and allow the original centering logic to act;
verify the actual label before destination confirmation. No forced coordinate,
map ID or guest field is used.

All152 actual input/dataset/output guards,497 stable source hashes and32 bounded
window hashes pass. The artifact helper completes with zero disposable paths.
Evidence is in `target/rom-data-world-location-selector/`: source contract,
placed-node-coordinate join, retained south coordinates, final summary,
preservation/cleanup receipts and `windows-world/` read sites/raw RAM. The only
tracked code is the optional test module and its feature-gated declaration;
runtime/resource/schema/cache/save behavior is unchanged. No new navigation or
old source/runtime/CLI/scene acceptance is run. Actual battle entry and all-four-
phase/all-four-list constructor guest/host use remain open before a fullpair.
Linux, USA, native Windows, new32:9 and full ROM_DATA_TODO remain pending; no
performance/storage campaign, Android/package, assistant commit or push occurs.


## Normal Giza battle entry and positive constructor animation

The source-backed short Left movement from the retained southern world
checkpoint reaches the visible Giza Plains label. Normal A travels there and
opens White Flowers with22days remaining. A enters its deployment screen.
The live primary index is315, variable7is69 and the loaded descriptor is68.
The actual512-byte terrain grid has SHA-256
`e8427b5e31cda367749b505f234ab95b2ecd85c3d91b91a5677817082589e840`,
matching the existing packet56seed and all20flag02cells. This closes normal
battle/map reachability; earlier negative map/navigation bounds stay closed.

Normal deployment places Marsh and confirms his facing. The first START
attempts happened before facing confirmation and leave that state unchanged;
B returns to placement. After facing A, the actual “START To Battle” prompt
appears. START/Yes and the opening dialogue reach the normal Move menu.
Marsh has unitFD2, position(6,5), height2, actor flags0and animation0.
Move/A followed by Left/Down/Down puts the actual cursor at(5,7), a highlighted
reachable flag02water cell. A confirms the move without guest-field edits.

The passive observer becomes positive after75executed frames, ending at22724:
22table reads/88aggregate bytes,44threshold and44pointer bytes, all four
phases and all four OBJ lists through both guest and host readers. It retains
an exact normal checkpoint at22713and an11-frame suffix before first use.
This is positive discovery, not yet a complete scene-equivalence result.

The unit uses16normal-input Wine captures/3875executed frames total, with the
existing309ba31c observer and e77dataset; no new build. These captures are not
16equivalence acceptances. All221actual input/dataset/output guards and496
frozen source hashes pass. Originals, cartridge saves and dataset are unchanged;
all local checkpoints derive from normal keys on copies. The artifact helper
retires zero paths. Evidence is in `target/rom-data-giza-battle-entry/`, including
per-step states/images/results, map/actor context, positive observation, final
summary and preservation/cleanup receipts. The [subsequent actual CLI and fullpair](rom_data_obj.md#ordinary-constructor-animation-positive-giza-scene)
now pass on the11-frame suffix, closing the ordinary constructor case.
Linux, USA, native Windows, new32:9 and the full ROM TODO remain pending.
No performance measurements, Android/package, assistant commit or push occur.


## World node record source projection

The next source unit bounds the30-record projection at395CA8..395F00:
600bytes,20-byte stride, SHA-256
`ed9d29c7b96b37537f0b8ac8e95fc17c57605c1056427e8dcbc58775e12d2e76`.
The30count comes from the already proved375F8loop. It does not impose a global
index limit:36E90/36EB4and4DAD0narrow inputs to bytes without checking30, while
4DB80uses32-entry work arrays. Future typed reads must preserve raw fallback
outside the declared projection. The primary31488initializer and32BE6..32BEC
store08395CA8at[03002810]+067C. Four exact aligned root literals and20aligned
067Cconstant matches are inventoried; this is not every synthesized pointer.

Bytes0/1are unsigned tile coordinates. Four ordered link slots begin at
2/6/10/14: path ID byte, neighbor node byte, then two opaque bytes. The last two
bytes18/19are also opaque. Thus300bytes have source-backed field meaning and
300bytes remain opaque, all zero in this exact image. Preserve them losslessly;
zero contents and selected readers do not prove global padding/dead-byte rules.
The exact graph has120slots,90nonzero path links and30zero-path slots. Its45
path IDs each join two opposite directions. Neighbor0is valid, including the
active links from nodes1/2; it is distinct from a zero path ID.

Original4DAD0matches neighbor fields in slot order and returns the corresponding
path byte, orFFonly when no neighbor matches. It does not first test path!=0:
target0therefore returns0for22nodes through empty slots. Original46EB0first
accepts equal nodes, then requires nonzero path before matching neighbors.
Original4DB80copies all four neighbors and paths;4DC5Eskips zero paths. Its
conditional node8/27gates depend on02002E58bits1/2. A nonzero path ID conditionally
selects397018+(ID−1)*8and reads byte0for path-cost comparison. Those road records
and payloads are a separate unmodeled dependency; no complete pathfinder ABI or
road-catalog bounds are inferred here.

The600-byte projection is disjoint from all49746source spans of the69accepted
e77resource owners. Twenty-one new bounded windows, four reused closed windows,
44instruction pins and42actual input/output/source guards pass. A new query of
the retained neutral world frame finds35unique reads/42sites within these600B,
only coordinate fields at375F8and3864E/38662. It is not new edge ordinary-use
coverage. A diagnostic print initially treated structured instruction metadata
as an integer; the saved query was intact and inspected without recapture.

No build, game run, Wine/Linux test, fixture or implementation change occurs.
Evidence is in `target/rom-data-world-node-records/`: source contract, exact
field/graph analysis, retained site query, authenticated windows and final/
preservation/cleanup receipts. The helper retires zero paths. Next is the strict
lossless typed node/link/opaque model and shared source plan, then one focused
Wine original-reader oracle. Extraction/managed runtime/actual CLI/ordinary
checks remain open, as do Linux, USA, native Windows, new32:9 and the full ROM
TODO. No measurements, Android/package, assistant commit or push occurs.


## Typed world nodes and original-reader proof

`world_nodes::Catalog` authenticates the exact EUR image, six producer/reader
windows and the600-byte source hash. `WorldNodes` exposes30ordered `Node`
records, unsigned tile coordinates, four ordered `Link` slots with distinct
path/neighbor bytes, each opaque pair and an opaque tail. Its shared
`source_spans()` plan reconstructs one exact600-byte range. JSON denies unknown
fields and wrong fixed array lengths. Shape validation checks source identity,
node count and order; it does not impose inferred max45paths/max29neighbors or
opaque-zero rules. Source verification separately rejects content changes.

The typed coordinate helper scales by8without the centering offset. Path lookup
matches neighbors first and can return zero through an empty slot; adjacency
accepts equal declared nodes and otherwise requires nonzero path before a
neighbor match. Helpers return `None` for nodes outside the declared30-record
owner, preserving the requirement for raw fallback in future guest integration.
No host pathfinder or live placement algorithm replaces original guest code.

One warning-free Windows core-only test build, frozen as6c2ef15a.., and one
focused Wine filter pass23shape/JSON/source guards and566original-interpreter
calls. Independent private model/probe files confirm exact600-byte
reconstruction and every return/load-address sequence, including16low-byte
aliases and24outside-projection calls. All300semantic bytes are observed at
original load sites. The300opaque bytes are reconstructed/hash checked, not
claimed read. Callee registers, SP, RAM outside the original stack, VRAM, OAM
and palette remain checked. These are synthetic call inputs, not new ordinary
checkpoints, full ABI, native/state-clock or pathfinder acceptance.

All24actual input/output/executable guards and215retained core-source hashes
pass. Original ROM/cache/loader/states/inputs and canonical executable remain
unchanged; the artifact helper retires zero paths. The parallel frontend is not
built. Evidence is in `target/rom-data-world-node-model/`, including independent
inputs, `windows-model-europe/`, build/run results, source preimages and final/
preservation/cleanup receipts. The only production code addition is this
standalone model and its module declaration. Schema/extractor/decoder stay
1/77/76; managed extraction/runtime, actual CLI and ordinary coverage are next.
Linux, USA, native Windows, new32:9 and full ROM_DATA_TODO remain pending.
No gameplay, performance/storage campaign, Android/package, assistant commit or
push occurs; the previous source/coordinate/constructor proofs stay closed.


## Managed world nodes and typed guest reads

EUR e78-d77 publishes `world-nodes.json` as `gameplay.world-nodes`, resource
index69:71generation files and70resources. All69previous records and files are
exactly retained; USA stays e8-d7. The file contains the strict30-node model and
one authenticated600-byte source binding. The runtime serves coordinates, path
IDs, neighbors and opaque bytes directly from the decoded fields. Seven usage
fields report reads/bytes, the four field classes and distinct nodes used.

Typed reads compose with earlier owners and raw bytes at either boundary.
The owner only claims395CA8..395F00, leaving node30/31/255queries and live RAM
root overrides to the original guest. No Machine/CAPI/native-cache/save ABI,
placement/pathfinder algorithm or host timing replacement is added. Host
pathfinding is not introduced. Publication, parsing, coherent full-set checks,
source authentication and startup attachment share the same model.

One reduced Wine filter passes publication and interrupted-new-file refusal,
12attachment guards and two full-set guards. Failed attachment preserves the
previous owner and full serialized state. Fifteen original/interpreter/native
prefixes match complete state and guest clock with all600raw source bytes
poisoned after state load:124compiled instructions and32explicit compiled
source bytes. They cover coordinate/path/adjacency semantics, low-byte alias,
mirrored pointer, outside30/31and equal255queries, plus a live RAM root override.
This is not a full pathfinder ABI or ordinary-scene claim.

The same filter serves all600bytes and every field class/30nodes through the
bus, with byte/halfword/word widths, ROM mirrors, mixed raw edges and an earlier
typed owner. Unavailable files preserve the attached owner through Load and
slot Undo. DMA, CpuSet and CpuFastSet each match complete state and clock across
the typed source and following raw bytes. The model's earlier566-call source
oracle and the world/constructor scenes are not replayed.

Two initial Windows build attempts fail because Cargo reuses an old core
artifact. Its dependency record still names the older host-source-snapshot;
touching the new copy's crate entry does not invalidate it. Moving only that
specific core fingerprint into private diagnostics forces the selected core to
compile. The third build is warning-free and frozen as2cf1d882..; its artifact
record says `fresh=false`. No global Cargo clean or accepted fixture/executable
removal occurs. The stable502-file source copy preserves parallel frontend work.

All200actual preservation guards,502integration-source hashes and215retained
model-source hashes pass. The artifact helper retires only the disposable
interrupted publication copy, keeping the accepted offline dataset, executable,
source preimages and failed-build diagnostics. Dataset identity is
e949b04d.., manifestf4b3c8b2.. and world-node JSON4336cfcc... Evidence is under
`target/rom-data-world-node-runtime/`, including three build results,
`windows-runtime-result.json`, accepted `windows-managed-europe/offline dataset/`,
final summary and preservation/cleanup receipts. The [actual production CLI](#actual-world-node-cli-and-ordinary-coordinate-use)
now passes registration and ordinary coordinate use; positive path/neighbor and
the full ordinary pair remain open. Linux, USA, native Windows,
new32:9, road397018and full ROM_DATA_TODO remain pending. No new gameplay,
performance/storage campaign, Android/package, assistant commit or push occurs.


## Actual world node CLI and ordinary coordinate use

One warning-free Windows production build from the accepted502-file integration
copy is frozen as e0491cdc... The canonical executable's hash and mtime are
restored exactly. Three actual Wine CLI commands pass: prepare creates the
managed e78-d77generation, a second prepare reuses it without writes, and
`runc` loads the unchanged28859-block native cache with normal Giza world input.
All71generation files match the accepted runtime dataset, all70logged resource
IDs/hashes match the manifest, and all73managed paths preserve hashes/mtimes
through reuse and normal startup.

The normal16-frame prefix uses exact copies of the already retained Giza01
world state and02travel input, with no new navigation fixture or guest edits.
It covers guest19049→19065and consumes402world-node coordinate bytes/402reads,
22nodes. Path, neighbor and opaque usage stay zero in this selected bound.
This closes actual production registration and positive ordinary coordinate
consumption. The [later source-backed360-frame travel case](#ordinary-world-node-travel-and-caller-gates)
now closes positive path/neighbor use and the full ordinary pair. The runtime's complete
field coverage is targeted evidence, distinct from this ordinary observation.

The private wrapper initially expected `frames:16`; the CLI reports the
absolute guest endpoint19065. All three child processes had already exited0
and passed their post-run preservation guards. The final summary is corrected
from retained logs only, with no child rerun. The16-frame zero path/neighbor
bound is closed; do not repeat it unchanged. Inspect original callers or a
source-backed later ordinary state before selecting further gameplay.

All169actual input/output/dataset/executable guards and502frozen source hashes
pass. Required artifact completion retires zero paths. Private evidence is in
`target/rom-data-world-node-cli/`: exact normal copies, build/three CLI logs,
`windows-cli-summary.json`, no-write receipt and final/preservation/cleanup
receipts. Dataset identity remains e949b04d.., manifest f4b3c8b2.. and new JSON
4336cfcc... No new fullpair, source/model/runtime oracle replay, performance
measurement, Android/package, assistant commit or push occurs. Linux, USA,
native Windows, new32:9, road397018and the full ROM TODO remain pending.


## Ordinary world node travel and caller gates

A new source criterion follows the path/neighbor readers, without replaying
the source model, world audit or closed16-frame observation. The direct Thumb
BL query finds4DAD0at4672C/4676E/4706C,4DB80at4DB30and46EB0at46D08/46D6C/46E50.
Their aligned Thumb-pointer query is empty; this does not exclude synthesized
or indirect references. The4DB1Cwrapper has five direct caller candidates,
including3B012;47034has468AC/468DC. Ten bounded windows are authenticated.

At3AFEE..3B016, a nonzero view+1Aone-based destination differing from
[parent+7BB] selects4DB1C/4DB80, with output at parent+7C1. Two existing normal
snapshots provide a concrete later-travel candidate: Giza01has current node17
and selected20(destination19); the accepted360-frame endpoint has current/
target19, progressFFand phase3. Entity1also moves from node0to1. The selected
entity projection is parent+7B8+52*index: bytes3/4origin/target,7phase,8progress.
Original47034requires progress!=FFbefore its4DAD0call. These source and retained
RAM links are conditional; they are not an exact-frame/PC attribution claim.

A single existing-CLI360-frame observation on exact copies of Giza01/02input
now proves all three semantic field classes:6841reads/6841aggregate bytes,
4820coordinate/864path-ID/1157neighbor bytes and29nodes. Opaque use stays0.
The longer bound is justified by the already retained player/entity transition;
the first16zero path/neighbor bound remains closed, not rerun as acceptance.
No new checkpoint, navigation fixture or manual guest edit is introduced.

A five-line optional `FFTA_STATE_TEST_REQUIRE_WORLD_NODES` gate requires positive
coordinates, paths and neighbors in the common scene validator. Removing it
restores the previous validator byte for byte. A single warning-free Windows
test build from a stable502-file copy includes the guard; only the exact prior
test fingerprint is isolated to avoid Cargo's known old-snapshot dep-info trap.
The core and runtime implementations are unchanged, and no old proof is rerun.

One Wine fullpair over guest19049→19409passes every360complete serialized
states and guest clocks, classic/central/extended images, OAM, native/fallback
dispatch and all five audio streams. All seven world-node usage counters match
the actual CLI exactly. Dispatch counts are2006894/3620371; audio sample counts
are[790020,790020,95040,95040,789942]. This closes the ordinary EUR/Wine world-node
case. It does not claim all30nodes/maps/actors, opaque ordinary use, global
archive bounds or a complete pathfinder ABI. Targeted model/runtime evidence
for all600bytes remains distinct from ordinary29-node/three-field coverage.

All103actual ordinary-unit guards and502stable source hashes pass, as do the
27caller-source guards and169closed CLI guards. Artifact completion retires
zero paths. Evidence is in `target/rom-data-world-node-ordinary-callers/` and
`target/rom-data-world-node-travel-ordinary/`: caller contracts/retained gates,
CLI and fullpair results, validator preimage/guard, source copy, final summaries
and preservation/cleanup receipts. New test executable dabb1ac6.. and existing
CLI e0491cdc.. are retained. Linux, USA, native Windows, new32:9, road397018and
full ROM_DATA_TODO remain pending. No performance/storage campaign,
Android/package, assistant commit or push occurs; parallel DBG work is preserved.


## World road path source contract

The45path IDs selected by the already proved30-node/90-active-link projection
identify45eight-byte headers at397018..397180. This is a conditional projection,
not a global archive bound. Header byte0is an unsigned sample count; bytes1..3
remain opaque, and word+4is the exact encoded ROM pointer. The45distinct payloads
are contiguous at395F00..397018:2188signed XY pairs/4376bytes. Together the
headers and payloads give46nonoverlapping source spans/4736bytes,4601semantic
and135opaque bytes. The full contiguous source hash is `e0a6a0a205c1dba5a9b0681309a7261056072d51c8cf7a4c1ce4814369caa8ec`.
All135opaque bytes happen to be zero; no global padding/zero rule follows.

The closed primary producer stores08397018at parent+680. The absolute root has
three exact aligned literal matches:31560,4DD08and4DD34. Original4DC8C..96uses
(pathID−1)*8/header byte0as a path-cost comparison term. Original4DD0Ctakes an
output pointer plus direction, header index and sample step, narrowing the
last three inputs to bytes without a45-index guard. If step>=count it writes
zero32and skips pointer/sample loads. Otherwise direction0selects step, while
nonzero selects count−(step+1); the doubled reverse offset is narrowed to16bits.
The reader sign-extends each source byte into a halfword and packs Xlow16/Yhigh16,
writes the output word and returns the same output pointer.

Original4DD70computes unsigned low-byte current-node>target-node to choose
reverse order. Original4DD84returns their unsigned low-byte minimum. At
4708Cand47112..44, the caller queries the sample and adds its signed offsets to
that lower-index node's tile X/Y multiplied by8. These are sampled offsets,
not incremental per-step deltas; reverse playback does not negate them.
The46B64facing consumer queries current/next samples across at most five
successive differences, retaining byte-step wrapping. Its angle/helper ABI is
not claimed. The direct-reader query finds4DD0Ccalls at46B94/46BE2/4708C,
4DD70at4707Cand4DD84at47112, with no aligned Thumb pointers for these three
exact addresses; indirect/synthesized references remain outside that query.

Every selected sample word is nonzero in this image, matching the caller's
packed-zero arrival sentinel. This is not a model rule forbidding a zero point.
Observed counts32..85and signed extrema are descriptive, not guest caps.
Forty paths end exactly at the node displacement. Paths4/6/8/10/12end one pixel
short; preserve those source samples and do not normalize or force endpoints.

Six windows(two new/four reused),60instruction pins and17actual source/tool/
output guards pass. The4736bytes are disjoint from all49747source spans of the
70accepted e78owners. Their left neighbor is the existing world-node owner;
future mixed reads must retain it. Source bytes after397180remain outside the
new projection. Evidence is in `target/rom-data-world-road-paths/`: selected
header/point analysis, source contract/46-span plan, caller candidates, windows,
final summary and preservation/cleanup receipts. The artifact helper retires
zero paths. No build, game run, Wine/Linux test, fixture or implementation change
occurs; the completed world-node ordinary case is not replayed.

Next is the strict lossless road-header/point model and shared source plan,
then one focused Wine original-reader oracle. Extraction, managed consumption,
actual CLI, ordinary use, global catalog extent and135opaque-byte meaning stay
open. Linux, USA, native Windows, new32:9and the full ROM TODO remain pending;
no performance/storage campaign, Android/package, assistant commit or push occurs.


## Typed sampled world roads and original-reader proof

`world_roads::Catalog` now authenticates the exact EUR image, the six closed
producer/reader windows and the complete 4,736-byte projection hash.
`WorldRoads` retains 45 ordered headers, each full-byte sample count, three
opaque bytes, exact encoded pointer and ordered signed `Point` offsets. The
shared `source_spans()` plan reconstructs 46 spans: 45 sample payloads followed
by the 360-byte header table. `byte_at()` covers this owner only. JSON rejects
unknown fields, invalid byte ranges and wrong opaque-array lengths.

Shape validation checks the declared header identity/order and a coherent,
nonoverlapping partition of its source payload region. Authenticity is a
separate exact-source check. Structural count values span the full byte range;
there is no inferred 32..85 cap, opaque-zero requirement, zero-point prohibition
or endpoint correction. All five one-pixel-short endpoints remain exact.
`packed_point()` applies low-byte direction/index/step aliases, forward or
reverse sample order and signed halfword packing. It returns `Some(0)` when
step is at least count, and `None` for header indices outside the declared
projection, requiring raw fallback. The unsigned node-order helpers preserve
the original reverse flag and lower-node base. Guest movement, facing and
pathfinding remain original code; no runtime integration is included yet.

One warning-free core-only Windows build (`de9d1e11..`) and one focused Wine
filter pass 29 structural/JSON/source guards and 2,537 original-interpreter
calls. Independent private inputs cover every 2,188 forward sample, 135 reverse
queries, 180 out-of-range queries, six low-byte aliases, six raw header queries
outside the owner and 22 node-order helper calls. Exact output words, returned
output pointers and load sequences match, including pointer/sample-load skips
on exhaustion. All 4,601 semantic bytes are observed at original load sites;
135 opaque bytes are reconstructed/hash checked and remain unread. Callee
registers, SP/LR, RAM outside the original stack/output, VRAM, OAM and palette
are checked. These synthetic calls do not establish full ABI, native clock,
facing/pathfinder or ordinary gameplay acceptance.

The stable source copy contains 504 build files, including 218 core/build
inputs; all hashes and 32 actual preservation guards pass. Only the affected
old core-test fingerprint was isolated to prevent reuse through dep-info naming
an earlier source copy. No old source, canonical executable or global cache was
cleaned. The parallel Debug audit/frontend work was excluded from this build.
Artifact completion retires zero paths. Evidence is retained in
`target/rom-data-world-road-model/`: independent model/probes/source plan,
build/Wine logs, decoded model/result, source preimages and final/preservation/
cleanup receipts. No game fixture, checkpoint or cartridge data was changed.

Schema/extractor/decoder remain 1/78/77. Next are managed publication and typed
guest source reads, actual CLI and one positive ordinary case. The source/model
and already closed world-node cases require no replay. Global catalog extent,
opaque meaning, Linux, USA, native Windows, new 32:9 and the full ROM TODO
remain pending. No performance measurement, Android/package, assistant commit
or push occurs.


## Managed world roads and typed guest reads

EUR e79-d78 publishes `gameplay.world-roads`, `world-roads.json` and
`world-roads-v1-json` at resource index70. The generation contains 72 files and
71 resources; all 70 previous resource records and decoded files remain exact.
USA remains e8-d7. Schema1 and the Machine/C API, save-state and native-cache
formats are unchanged. Parsing, source authentication, coherent full-set
validation, publication/startup attachment and resource/usage logging share the
model's 46-span plan. Original movement, facing and pathfinding remain guest
algorithms.

The immutable runtime owner reconstructs header counts, exact encoded pointers,
signed sample bytes and opaque bytes from typed fields. Its seven usage fields
are bus_reads, source_bytes, count_bytes, pointer_bytes, sample_bytes,
opaque_bytes and paths_used. Width/mirror reads compose bytewise with previous
owners and raw source bytes. In particular, the byte immediately before the
sample region remains owned by world nodes; the right boundary at397180 is raw.
Header indices45..255 stay outside this owner and retain prior-owner/raw fallback. No opaque-zero, endpoint correction or source-reader
index clamp is introduced.

Two warning-free Windows test builds (accepted `52be2981..`) use a stable 506-file
source copy, excluding the parallel Debug audit/frontend work. Only the two
affected old core-library/recomp-test fingerprints are isolated to avoid
cross-copy dep-info reuse; no failed build, global clean or old-source change
occurs. The first reduced Wine filter passes new-file publication/interruption,
12 attach guards and two full-set guards, preserving the prior owner and full
machine state on failure. It then finds a real routing bug at native case3: global source-page filtering
omits page396, so three output RAM bytes leak the poisoned source. Adding the
road source-page gate fixes it. A second filter resumes only native/bus/lifecycle
against the already accepted dataset; publication/12attach/fullset are not
repeated. Twelve original/interpreter/native prefixes now match
complete state and guest clock with all4,736raw source bytes poisoned after
state load. The prefixes cover forward/reverse packing, a retained short
endpoint, exhaustion/pointer skip, low-byte aliases, outside-owner headers and
unsigned node-order helpers. They execute 67
compiled instructions and observe 44 explicit
compiled source bytes; this is correctness coverage, not a timing result.

The same filter checks all4,736typed bytes, all four field classes
(45count/180pointer/4,376sample/135opaque bytes) and all45paths. Byte/halfword/word
reads, ROM mirrors, adjacent old ownership, the right raw boundary and a prior
separate owner pass. Dataset-unavailable Load/slot-Undo retains the same owner.
DMA, CpuSet and CpuFastSet each match full state and guest clock across the new
source region and following raw bytes. No model/original-reader oracle, world-
node ordinary case, navigation or gameplay fixture is replayed.

All 221 actual input/output/old-dataset/executable guards, 506 integration
source hashes and 504 closed-model source preimages pass. Artifact completion
retires only the disposable interrupted publication; accepted offline data,
source preimages, executables, logs and receipts remain. Evidence is in
`target/rom-data-world-road-runtime/`: build/run results,
`windows-managed-europe/offline dataset/`, final/preservation/cleanup receipts.
Dataset identity is `14970175f018122fb4235dec3410d236d3f960ca2acc9018eedaf044ff768781`; manifest hash is
`282808e5580b4e5cae3ae5665401bb173f993bcd87fcac36861263dcc67a4230` and new JSON hash is
`f19748d8807a78027a7e595bf14bc4c93acaa4fb52afa956c98bdd469c93203d`.

Actual production CLI creation/reuse/startup and one positive ordinary case
remain open. Native prefixes establish selected routine coverage, not full
facing/pathfinder ABI, gameplay or global archive/index extent. Linux, USA,
native Windows, new32:9and the full ROM TODO remain pending. No new game or
checkpoint, performance/storage campaign, Android/package, assistant commit or
push occurs.


## Actual sampled-road CLI and ordinary consumption

The actual e79-d78 Windows production CLI now passes three Wine commands:
`rom-data prepare` creates the generation, a second prepare reuses it without
writes, and `runc` executes the normal retained Giza360 travel input. The build
is warning-free and frozen as `1a86d341..` from the accepted506-file runtime
source copy. The canonical executable's hash and mtime are restored exactly.
Only the affected production fingerprint is isolated; unchanged core artifacts
are reused. Preparation initially finds several historical production
fingerprints; the exact prior copied source path in dep-info selects only the
one relevant fingerprint. Existing normal copies are validated without
recreating fixtures; no failed build or global cache clean occurs.

All72generation files match the accepted runtime dataset. All71logged resource
IDs/hashes match the manifest, and all74managed files retain hashes/mtimes
through reuse and normal startup. The original28859-block
native cache loads. Exact normal state/input copies cover guest19049→19409,
with 2976road reads/5067source bytes:
885count, 2788encoded-pointer,
1394signed-sample and 0opaque bytes,
40paths used. Count/pointer/sample consumption is positive before
selecting the ordinary equivalence pair; opaque usage stays a separate targeted
proof. This is one normal travel bound, not all45path payloads/maps/entities,
full facing/pathfinder ABI or global catalog extent.

All180actual input/output/executable/dataset guards and506frozen
source hashes pass; artifact completion retires zero paths. Evidence is in
`target/rom-data-world-road-cli/`: exact normal copies, build and three process
logs, CLI summary/no-write receipt and final/preservation/cleanup receipts.
No new navigation or checkpoint, source/model/native/lifecycle/node acceptance
replay, guest/cart edit or performance measurement occurs. The next criterion
is one ordinary full pair with the same positive counts. Linux, USA, native
Windows, new32:9and the full ROM TODO remain pending; no Android/package,
assistant commit or push occurs.


## Ordinary sampled-road travel equivalence

The ordinary EUR world-road case is now closed: exact retained normal Giza01
state and02travel input, with the actual CLI e79-d78dataset, pass one Wine360
pair at guest19049→19409. All360complete states and guest clocks, classic/
central/wide images, OAM, native/fallback dispatch and five audio streams match
the raw-ROM machine. All seven road usage counts equal the actual production
CLI: 2976reads/5067bytes, 885count,
2788pointer, 1394sample, 0opaque,
40paths touched by any field. The path counter includes header-
only reads; it does not prove40sample payloads. Dispatch counts are
[2006894, 3620371]; audio sample counts are[790020, 790020, 95040, 95040, 789942]. These are
correctness counts, with no host performance measurement.

The validator adds one five-line optional
`FFTA_STATE_TEST_REQUIRE_WORLD_ROADS` guard requiring count/pointer/sample
consumption and reporting all seven statistics. Removing that exact block
reproduces the previous validator byte for byte, including prior family guards.
One warning-free Windows test build (`4c453f07..`) uses the accepted506-file
source copy with only this validator overlay; only the affected old test
fingerprint is isolated, leaving unchanged core artifacts and earlier source
copies intact. No source/model/native/lifecycle/CLI-registration or old world-
node acceptance is replayed.

All100actual guards, 506 pair-source hashes and180closed CLI guards
pass. Artifact completion retires zero paths. Evidence is retained in
`target/rom-data-world-road-travel-ordinary/`: validator preimage/block proof,
build/run results, source copy and final/preservation/cleanup receipts. The
same normal state/input and accepted actual CLI generation are retained by the
CLI unit. No new game/checkpoint/navigation fixture or manual guest/cart edit
occurs. Source/model/runtime/actual CLI and this ordinary case are closed.
This is one conditional travel scene; all45payloads/maps/entities, opaque
ordinary use, full facing/pathfinder ABI and global catalog extent remain
unproved. Linux, USA, native Windows, new32:9and the full ROM TODO remain pending.
No performance/storage campaign, Android/package, assistant commit or push
occurs; the parallel Debug work is preserved.


## European encounter headers and shared ownership

The declared EUR encounter-header catalog now has independent physical
boundary evidence. The indexing origin is70C8CC, but the first physical header
is70C8F4(index1), independently referenced at98E4andD2558. The next family begins
at710E04; its independent roots24540/C6D0Cand arithmetic use28-byte records.
The intervening17,680bytes contain exactly442ordered40-byte headers, indices
1..442. All442unit-template backlinks form one continuous136,800-byte chain of
2,85048-byte templates from6EA048to70B6A8. The physical partition follows these
independent root/type/backlink witnesses, rather than treating the earlier
observed maximum442or unrelated constant300as an array bound. No442guest-input
check is introduced; original callers narrow indices to16bits.

A new PC-relative-loader query reuses the27cached aligned70C8CCroot literals,
without repeating their ROM census. It identifies one candidate load for each.
Selected indirect consumers additionally establish a u16unit-template count at
+0and an encodedu32pointer at+4; the importerCE18Calso respects its live r2limit
and advances templates by48. Byte+2is the default map argument. The inspected
placement readers use bytes8/10as parameters, bytes12/14as search counts,
byte13as a random-selection range, and encoded pointers16/20/24/28for coordinate
lists; signed XY and first-XY accesses remain distinct. Byte32is compared with
live unit count,33is tested for nonzero,34is an original switch selector,36a
zero gate and38a script default operand. These17fields account for33bytes per
header. Bytes3/9/11/15/35/37/39remain opaque: no zero policy, enum restriction,
boolean canonicalization or null-pointer safety assumption is inferred.

Index0is outside that physical header span. Its last16bytes are the actual
coordinate payloads selected by header442:14bytes at70C8E4and two at70C8F2,
ending exactly at the first header. Decoding index0as a valid header would
misinterpret coordinate bytes as counts/pointers. Preserve raw/prior-owner
fallback for index0and indices443..65535, with their original low16aliases.
This does not prove global absence of other/synthesized/indirect consumers.

One header field is already owned: header29's pointer at70CD58..70CD5Cbelongs
to `gameplay.constructor-alternate-templates`, as do its192template bytes at
6ECA78..6ECB38. The new header plan therefore contains two disjoint fragments,
70C8F4..70CD58and70CD5C..710E04, totaling17,676new bytes(14,582semantic and
3,094opaque). Future managed/runtime integration must declare that dependency,
validate shared encoded-pointer coherence and skip those four bytes in the new
owner so the existing owner/statistics remain authoritative. All other header
bytes are disjoint from49,793spans of71accepted e79owners. Unit-template payload
schema/ownership and full coordinate-pool coverage remain separate work; the
computed contiguous pools are source leads, not migrated resources.

Forty source windows(39new/one reused),54instruction pins and68actual guards
pass. All442headers reconstruct exactly, with full header hash
`580d0702d55d148643c3d920282eb52f73afc2514e0f1f95564e0e0e0b569ee6`.
Evidence is under `target/rom-data-encounter-catalog-readers/`: root loader and
boundary candidates, field witnesses, header/template partition analysis,
reconstructable private records, source contract/two-span plan and final/
preservation/cleanup receipts. Artifact completion retires zero paths. No
build, Wine/Linux test, game run, checkpoint, decoder or runtime change occurs;
the previous road/node/constructor proofs stay closed.

Next is a strict lossless442-header model with the shared-pointer dependency,
then one focused Wine original-reader proof and managed/runtime integration.
Nested payloads, all indirect readers/full ABI/global archive coverage, Linux,
USA, native Windows, new32:9and the full ROM TODO remain pending. There is no
new navigation, guest/cart edit, performance/storage campaign, Android/package,
assistant commit or push.


## Typed encounter headers and original-reader proof

`encounter_headers::Catalog` authenticates the exact EUR image, six source-root/
reader windows and the full17,680-byte header hash. `EncounterHeaders` retains
442ordered `Header` values, indices1..442, with named u16count, u8map/placement/
comparison/gate/default fields, exact encoded u32template/position pointers and
seven opaque bytes. Strict JSON rejects unknown fields, wrong array lengths and
out-of-range integers. Shape validation checks the physical source, count and
order; it imposes no inferred count caps, pointer-validity/null rules, enum sets
or opaque-zero requirements. Byte33remains a full byte, even though the original
reader and typed helper test it for nonzero.

The shared two-span plan reconstructs17,676newly owned bytes. `byte_at()` returns
`None` for70CD58..70CD5Cso existing constructor-alternate ownership remains
untouched. Full header reconstruction and source verification include that
shared field. `validate_constructor_alternates()` checks its encoded-pointer
coherence against the already authenticated owner. A changed header29pointer
can leave both new spans unchanged; dependency/source authentication still
reject it. Index helpers preserve original low16aliases and return `None` for
0and443..65535, requiring prior-owner/raw fallback. Unit/coordinate payloads and
live import/placement/facing algorithms remain separate.

One warning-free core-only Windows build, frozen as5855af92.., and one focused
Wine filter pass29shape/JSON/content/dependency guards and7,523original probes.
Independent private model/source/probe files confirm exact17,680-byte
reconstruction and the two source-fragment hashes. There are7,072one-instruction
prefixes at actual original load sites across16fields, and451completeC0034gate
calls(442declared headers, six low16aliases and three outside indices). Their
actual effective addresses, widths, loaded values and gate returns match the
model. All14,586semantic source bytes are observed, including four shared bytes;
14,582belong to the new fragments. The3,094opaque bytes are reconstructed/hash
checked and remain unread. Registers/flags/PC for each load prefix, callee
registers/SP/LR for full gate calls, and RAM outside the original four-byte stack,
VRAM/OAM/palette are checked. Prefixes are not a full importer/caller ABI or
native/state-clock/ordinary gameplay proof.

All33actual input/output/executable guards and508frozen build-file hashes,
including221core inputs, pass. Only the affected old core-test fingerprint is
isolated to avoid copied-source dep-info reuse; no failed build, global clean
or old-source mutation occurs. The parallel Debug audit/coverage/frontend work
is excluded from this build. Artifact completion retires zero paths. Evidence
is under `target/rom-data-encounter-headers-model/`: independent model/probes/
source plan, build/Wine results, decoded model/result, source preimages and
final/preservation/cleanup receipts. No game, checkpoint, fixture or closed
road/node/constructor source/model/runtime acceptance is replayed.

Schema/extractor/decoder remain1/79/78. Next is managed publication and immutable
typed guest reads with the constructor-alternate dependency, shared-four-byte
skip and global source-page coverage70C..710, followed by actual CLI and one
positive ordinary case. Nested template/coordinate schema and global/all-caller/
full ABI coverage remain open, as do Linux, USA, native Windows, new32:9and the
full ROM TODO. No performance/storage campaign, Android/package, assistant
commit or push occurs.


## Managed encounter headers and typed guest reads

EUR e80-d79 now publishes `gameplay.encounter-headers`, `encounter-headers.json`
and `encounter-headers-v1-json` at resource index71. The generation contains
73files/72resources; all71prior resource records and decoded files remain exact.
The explicit dependency is `gameplay.constructor-alternate-templates`.
Shape, full model/source authentication and constructor-pointer coherence are
checked before attachment; schema1and USAe8-d7remain unchanged. Parsing,
publication, startup attachment and usage logging use the model's two-fragment
17,676-byte source plan. Guest import/placement/facing and nested payloads
remain original operations.

The immutable runtime owner rebuilds header fields from the decoded model and
excludes70CD58..70CD5Cfrom its byte projection. Those four bytes still come from
the existing constructor owner, with that owner's counters. The global ROM page
dispatch explicitly covers70C..710. Eight byte classes record unit count, map,
template pointer, placement parameter, placement count, position pointer,
control and opaque reads; a seven-word mask records442headers. Together with
bus_reads/source_bytes/headers_used there are11usage fields. Byte/halfword/word
reads compose with the shared old pointer and raw boundaries. Original low16
aliases and raw/prior-owner behavior for index0and443..65535remain intact.

One warning-free Windows integration build, frozen as `1c4135f5..`, and one
reduced Wine filter pass publication/new-file interruption,14attach guards and
three full-set guards with failed-owner/full-state retention. The guards include
shared-pointer changes whose new source fragments stay unchanged, missing
dependency declarations, counterfeit shared ownership, source reordering and
strict JSON failures. Twenty-six original/interpreter/native calls or prefixes
match complete state and guest clock with all17,680raw header bytes poisoned
after state load. They cover selected field loads, fullC0034returns, low16
aliases, outside indices, mirrors and the shared pointer. There are
31compiled instructions,
37explicit new compiled source bytes and
4explicit shared bytes from the old owner.
These counts are correctness evidence, not performance measurements.

The same filter verifies every17,676new byte and all four prior bytes, all442
headers, and class totals884/442/1764/884/1326/7072/2210/3094. Widths/mirrors,
reads crossing the shared hole and either raw edge, plus an older separate
owner pass. Dataset-unavailable Load and slot-Undo retain the same owner. DMA,
CpuSet and CpuFastSet each match full state and clock across both fragments,
the shared pointer and following raw bytes. This is selected native coverage,
not a full importer/caller ABI, payload or ordinary gameplay proof.

All206actual input/output/old-dataset/executable guards,510frozen integration
source hashes and508closed-model source preimages pass. The accepted source
copy excludes the parallel Debug/coverage/frontend work; only the two affected
old core-library/recomp-test fingerprints are isolated before the build. No
failed build, global clean or old-source modification occurs. Artifact
completion retires only the disposable interrupted publication; accepted offline
data, executable, logs and source preimages remain. Evidence is in
`target/rom-data-encounter-headers-runtime/`, including build/run results,
`windows-managed-europe/offline dataset/` and final/preservation/cleanup receipts.
Dataset identity is `a6f4094e6f3998cb308bd8db24a0bcecf1e06da246082987592be3e1420dfac9`, manifest hash
`b064dc975f0a7f49c60809768bbfcd10ff9b9c86d737c00b5aa6be35ba850b67` and header JSON hash
`626c910741e2c796908c40b03ced73a8c6bb9284df92e8df58e12dfa2ae368fc`.

Actual production CLI creation/reuse/startup and one positive ordinary pair
remain next. No new game/checkpoint/navigation fixture or old7523-probe model,
road/node/constructor source/runtime/ordinary acceptance is replayed. Nested
unit/coordinate payloads, global/all-caller/full ABI coverage, Linux, USA,
native Windows, new32:9and the full ROM TODO remain pending. No performance/
storage campaign, Android/package, assistant commit or push occurs.


## Actual encounter-header CLI and ordinary consumption

The actual e80-d79 Windows production CLI passes three Wine commands: prepare
creates the generation, a second prepare reuses it, and normal `runc` enters
White Flowers from the retained Giza menu. All73 generation files match the
accepted runtime dataset; all72 logged resource IDs/hashes match its manifest.
All75 managed files retain hashes and mtimes through reuse and game startup.
The header resource retains its mandatory constructor-alternate dependency and
two source fragments, leaving the existing four shared pointer bytes with their
prior owner. The original 28859 translated blocks load.

Exact normal state/input copies cover guest19409→20109 (700 frames), with
17 header reads/41 source bytes:
12 unit-count, 1 map,
4 unit-template-pointer,
0 placement-parameter,
1 placement-count,
20 position-pointer,
3 control and 0 opaque bytes;
1 headers are touched. Six semantic byte classes have
positive ordinary use before selecting the full equivalence pair. Placement-parameter and opaque-byte
coverage remain the separate targeted source/native/bus proof. Neither is read
in this bounded ordinary scene; no extra navigation is added to obtain usage. This single battle-entry
case does not establish all442 headers, nested payloads or full guest ABI.

One warning-free production build `9d0a772d..` uses the accepted510-file runtime
source copy, excluding parallel Debug/frontend work. Only the affected production
fingerprint is isolated; the canonical executable's hash and mtime are restored
exactly. All184 actual guards and510 frozen source hashes pass; artifact
completion retires zero paths. Evidence is retained under
`target/rom-data-encounter-headers-cli/`: the build, three process logs,
no-write and final/preservation/cleanup receipts, exact normal copies and CLI.
No new navigation/checkpoint, closed source/model/runtime/road-node proof replay
or performance measurement occurs. One ordinary full pair with the same counts
is next; Linux, USA, native Windows, new32:9 and the full ROM TODO stay pending.


## Ordinary encounter-header battle-entry equivalence

One Wine ordinary equivalence pair now closes the same retained normal700-frame
Giza menu→White Flowers deployment bound, guest19409→20109, using the actual
e80-d79 CLI dataset. Every complete machine state and guest clock matches the
original ROM path. Classic, central and wide images, OAM, native/fallback
dispatch and all five audio streams also match. Dispatch counts are
`[3316146, 8054486]` and audio sample counts are `[1536150, 1536150, 184800, 184800, 1536026]`; these are
correctness/coverage evidence, with no performance measurement.

All eleven resource usage fields equal the preceding production CLI exactly:
17 reads/41 source bytes,
12 count, 1 map,
4 template-pointer,
0 placement-parameter,
1 placement-count,
20 position-pointer,
3 control, 0 opaque,
1 header touched. The six observed semantic classes are positive.
Placement-parameter and opaque ordinary consumption are unobserved in this case;
the earlier targeted source/native/bus proofs remain separate. The initial CLI
seven-class criterion was recorded as partial, then this six-class bounded case
was explicitly selected; no extra navigation or counter forcing was performed.

An optional five-line `FFTA_STATE_TEST_REQUIRE_ENCOUNTER_HEADERS` guard enforces
positive use of the six observed classes and reports all eleven fields. Removing
only that guard reproduces the previous validator byte-for-byte, preserving all
older scene requirements. One warning-free510-source Windows test build
`43fcc614..`, with only the affected test fingerprint isolated, and one Wine
filter pass. Core sources are unchanged from the accepted runtime. All
103 ordinary guards/510 frozen pair-source hashes and the closed
184 CLI guards/510 production-source hashes remain exact;
artifact completion retires zero paths. Evidence is retained under
`target/rom-data-encounter-headers-battle-ordinary/`, including preimage/guard,
build, full-pair result and final/preservation/cleanup receipts.

This closes one ordinary battle-entry case, not all442 headers, placement/facing
ABI, nested unit/coordinate payloads or global archive coverage. No new fixture,
checkpoint, navigation, prior source/model/runtime/CLI/road-node acceptance
replay or performance measurement occurs. Linux, USA, native Windows, new32:9
and the remaining ROM TODO stay pending.


The nested encounter unit-template source contract is now closed for the
header-linked2,850-record physical partition, retaining the192 bytes already
owned by constructor alternates. Its strict model and managed/runtime/ordinary
migration remain next. [Layout, readers and shared ownership](rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership).


## European encounter coordinate lists and original readers

The four pointer roles of the authenticated442 encounter headers define691
non-null coordinate lists,2,342 XY pairs and exactly4,684 bytes at
`70B6A8..70C8F4`. SHA-256 is `f5b215d130be9fc5536627f67c0dbe39d9bdb6738325d1ea8d6c16642af4e3d4`. The list extents uniquely
partition the full interval in header/slot order: no holes, overlaps, aliases or
out-of-range references. This independent pointer/count coverage corroborates
the closed unit-template end and first encounter-header address; adjacency alone
does not establish the payload catalog. All49,797 source spans of73 accepted
e81 owners are disjoint, so the coordinate payload has no shared prior owner.

The691 planned source spans preserve each list's header index, slot and raw XY
pairs. Their roles are:

- Placement0:323 lists/1,636 pairs/3,272 bytes, pointer16 and count12.
- Random:34 lists/340 pairs/680 bytes, pointer20 and range13.
- Placement2:12 lists/44 pairs/88 bytes, pointer24 and count14.
- Default:322 lists/322 pairs/644 bytes, pointer28's first XY pair.

There are323 headers with payloads and119 with all four pointers null. Of1,077
null links,408 are random,430 placement2,119 placement0 and120 default. All null
non-default roles have zero count. Default nulls remain conditional original
pointer reads with unproved caller gates, not permission to return a safe zero
coordinate. All data is raw u8: placement search uses signed LDRSB atBD044/BD048,
while first-point selection, candidate comparisons, random placement and default
getters use unsigned loads. Signed and unsigned views must preserve every bit;
actual small positive coordinate values are not a new range restriction.

Placement0/2 mode selection and live loops remain guest code. The random path
uses original RNG02868: `seed*0x41C64E6D+0x3039` stored to IWRAM030034B0, with a
nonnegative15-bit return. The caller uses signed remainder146314, low-u8 index,
a u16 uniqueness mask and pointer20+2*index. Actual random lists each contain10
pairs; neither10 nor the16-bit mask becomes an invented global guest cap or a
replacement RNG/placement algorithm. Zero/outside/null inputs retain their
original control and prior/raw behavior until their callers are proved.

The final16 bytes of the raw index0-header prefix are now accounted for by
header442's14-byte placement0 list at70C8E4 and two-byte default at70C8F2. Index0
is still outside the physical header catalog. Full payload projection and future
`byte_at` ownership will include these bytes without inventing an index0 header.

Nine closed bounded reader windows are reused; three new windows cover only
the missing remainder helper/tail and RNG body. All41 instruction pins, list
projections, span ownership and38 actual preservation guards pass. No new build,
Wine/Linux runtime test, gameplay/checkpoint/navigation, global root census,
closed model/runtime/ordinary proof replay or performance measurement occurs.
Evidence is retained under `target/rom-data-encounter-coordinates/`:
`source-contract.json`, `coordinate-lists.json`, references/null references,
coverage analysis,691-span plan, window metadata and final/preservation/cleanup
receipts. The intermediate coverage analysis is distinct from the closed source
contract, and does not by itself claim execution coverage.

Next is a strict lossless691-list model with header-pointer/count coherence and
one focused Wine signed/unsigned-load/random-prefix proof. Managed publication,
typed guest reads, actual CLI and ordinary use remain open. The dependency is
`gameplay.encounter-headers`; schema1/e81-d80 and USAe8-d7 remain unchanged.
Linux, USA, native Windows, new32:9, full placement ABI/all-caller gates/global
archive and the full ROM TODO remain pending.


## Typed encounter coordinates and original-reader proof

`encounter_coordinates::{Point,CoordinateList,EncounterCoordinates,Catalog}`
now models all691 header-linked lists/2,342 pairs losslessly. Raw u8 X/Y fields
and the diagnostic signed view preserve the same physical bits. The fixed
catalog shape checks source/count/order/slot/contiguous extent; separate header
coherence checks every non-null pointer, slot, count and header index. Null links
are absent diagnostic lists, never a safe guest fallback. Coordinate values,
observed role lengths and RNG limits do not become new shape restrictions.

All4,684 source bytes and691 per-list spans reconstruct independently. Exact EUR
ROM identity, full source digest and the12 closed reader windows authenticate
the source separately from structural validity. Strict JSON rejects unknown
fields and wrong/missing/nonnumeric coordinate fields. High-bit byte edits remain
structurally coherent with header pointers/counts but fail source authenticity.
The original guest placement, signed/unsigned loads, RNG, remainder, mask and
live/null gates remain unchanged.

The first core-only Windows build catches a missing `Bus` trait import in the
new test, before any Wine execution. Only that import is corrected. The failed
516-file source copy/logs remain diagnostic preimages; the second516-file copy
contains just the fixed test. Only the affected core-test fingerprint is isolated.
The accepted build `2fe3e236..` is warning-free, with227 core inputs; no global
clean, older source mutation or frontend build occurs.

One focused Wine filter passes30 guards and8,044 actual original single-load
prefixes: unsigned X/Y across every pair, plus signed search X/Y for both
placement roles. Actual effective addresses, registers, flags, PC and all4,684
source bytes are checked. Twelve isolated synthetic prefixes demonstrate
unsigned/signed behavior for128/255/127/0 without authenticating or publishing
these edits. The source ROM is restored and verified exactly afterward.

The same filter passes102 bounded original random-selection prefixes across
all34 random lists and seeds0/1/u32::MAX. It executes the original RNG and
remainder calls, checks returned range/index, pointer+2*index, both loaded
coordinates, seed update, u16 uniqueness-mask update and live unit F6/F7 writes.
Only the expected seed/mask/eight-byte helper stack scratch and two destination
bytes may change. Callee registers/SP/retained prefix LR, outer RAM, VRAM, OAM,
palette and original ROM guards pass. The25,362 instructions are execution
coverage, not performance measurements. These are bounded prefixes with an empty
mask, not full placement ABI, repeated-selection/exhaustion behavior, all caller
gates or native state/clock equivalence.

All40 actual model-unit guards,516 accepted source hashes,516 failed-build
preimage hashes and38 closed source-contract guards remain exact. Artifact
completion retires zero paths. Evidence is retained under
`target/rom-data-encounter-coordinate-model/`: independent model/probes/coverage/
source plan, both source snapshots/build logs, accepted Wine result and
final/preservation/cleanup receipts. No old header/template/constructor/source/
ordinary acceptance replay, gameplay/navigation/checkpoint or metrics occurs.

Managed publication and typed guest reads are next: e82-d81/index73,
`encounter-coordinates.json`, explicit encounter-header dependency,691 spans and
pages70B..70C including both global/local routing. Existing template/header
owners border the coordinate source on both sides. The last16 bytes of the raw
index0-header prefix will be coordinate-owned, while index0 remains outside the
header model. Actual CLI and a positive ordinary pair follow. Current schema1/
e81-d80 and USAe8-d7 are unchanged. Linux, USA, native Windows, new32:9, global/
all-caller/full-ABI verification and the full ROM TODO stay open.


## Managed encounter coordinates and typed guest reads

EUR e82-d81 publishes `encounter-coordinates.json`, resource
`gameplay.encounter-coordinates` at index73. The generation contains75 files and
74 resources. All73 previous resource records/files remain exact; schema1 and
USAe8-d7 are unchanged. The encounter-header dependency is explicit, and strict
parsing, header pointer/count/role coherence, source binding and full source
comparison precede transactional activation.

The immutable coordinate owner supplies691 list spans/4,684 bytes, with four
role counters, X/Y counters and list/header masks. Both local and global routing
cover pages70B..70C. Bytewise composition retains the unit-template owner on the
left and header owner on the right, including mixed-width boundary reads. The
last16 bytes of the raw index0 prefix are coordinate-owned; index0 still does
not become a typed header. Original signed/unsigned loads, mode selection, RNG,
remainder, live mask/loop and null gates remain guest code. No Machine/C API,
native-cache or save-state ABI changes. The64MiB parser guard bounds malformed
input, without a storage or performance campaign.

Three warning-free Windows integration builds use518 stable source files each;
the accepted executable is `0f85fbf7..`. All three extend the accepted516-file
model copy with integration paths. Subsequent copies change only the new test
harness. Unrelated
Debug changes are excluded. Integration anchors are validated before writing;
only the affected core-library and test fingerprints are isolated. No global
clean, old source modification or closed model/source/header/
template/ordinary proof replay occurs.

The first reduced Wine filter passes interrupted publication,14 attach refusals
and three full-set guards, preserving the old owner and complete state on
activation failure. It then stops at a missing native table entry in the
synthetic harness. Inspection
corrects its BL-pair/ARM-aware builder to the established native emission
contract. The second, remaining-only filter identifies a separate case-boundary
problem: single-load and random-prefix blocks share one chained analysis, so
execution continues past a single-load stop. The final test-only fix builds two
case families and asserts that each analysis excludes its stop boundaries. All22
cases remain required; native chaining and guest/runtime code are unchanged.
A third Wine filter runs only the remaining native and bus/lifecycle stages,
reusing the same75 accepted dataset files without re-publication or attach-guard
replay. All first sources/executable/logs/native-object diagnostics are retained.
The22 original-ROM/interpreter/compiled-native cases now match
complete states and guest clocks after all4,684 raw coordinate bytes are poisoned
in the test machines. They comprise18 actual signed/unsigned loads across four
roles, mirrors, prior owners and an unrelated raw successor; one full index0
gate call; and three original RNG/remainder/coordinate-selection prefixes.
The full index0 case verifies the new ownership boundary; it does not reopen the
older header-model oracle. Native code explicitly consumes
21 new coordinate bytes,
2 prior template bytes and
13 prior header bytes.
156 compiled instructions are coverage only, not a full
placement ABI or random-exhaustion claim.

Every new source byte passes the bus proof with raw data poisoned:3,272 placement0,
680 random,88 placement2 and644 default bytes,2,342 X and2,342 Y bytes, all691
lists/323 headers. The proof covers widths, mirrors, all691 list-end crossings,
page boundaries and poisoned prior-owned bytes on both sides. An unrelated older
owner leaves coordinate statistics unchanged. Load, slot save/load and Undo
retain the immutable owner even while dataset files are unavailable.

DMA3 and CpuSet copy2,344 halfwords, while CpuFastSet copies1,184 words, all to
EWRAM10000. Transfers cross the complete coordinate block into the header owner
and match complete states/clocks. All253 actual guards,518 integration
source hashes,518 first-attempt and518 second-attempt source hashes and516 closed model source
preimages remain exact. Artifact completion retires only the interrupted dataset copy; accepted offline data,
source preimages, executable and logs remain available.

Evidence is retained under `target/rom-data-encounter-coordinate-runtime/`:
build/filter/native/bus results, static integration plan, final/preservation/
cleanup receipts, accepted dataset and source snapshot. Dataset identity is
`c3804c66ce41e2e292ee67785aada27bcc65d22af5dd646e2b02e4a714cbc0e3`, manifest SHA-256
`08946ac3f72e4460ace9473e5366c8fd452bf00355a720f25310ec3b9bd35061` and coordinate JSON SHA-256
`91a2222cad4575095fe73f8a8a5ddc3d0f89a68482e01a6f3c79ea8cf66807e7`.
Actual production CLI/create-reuse/no-write/startup and positive ordinary use
are next. Linux, USA, native Windows, new32:9, full placement ABI/all caller/null
gates/global archive and the remaining ROM TODO stay pending. No new ordinary
fixture, checkpoint, navigation or performance measurement occurs.


## Actual encounter coordinate CLI and ordinary consumption

The production e82-d81 Windows CLI passes three Wine commands: prepare creates
the generation, a second prepare reuses it, and normal `runc` enters White
Flowers from the retained Giza menu. All75 generation files match the accepted
runtime dataset; all74 logged resource IDs/hashes match the manifest. All77
managed files keep hashes and mtimes through reuse and normal startup. The
original28859 translated blocks load, with the header dependency and691
coordinate spans intact.

Exact normal state/input copies cover guest19409→20109 (700 frames). All ten
coordinate counters are `{"bus_reads": 20, "default_bytes": 0, "headers_used": 1, "lists_used": 1, "placement0_bytes": 20, "placement2_bytes": 0, "random_bytes": 0, "source_bytes": 20, "x_bytes": 10, "y_bytes": 10}`. Positive roles are
`placement0_bytes`; unobserved roles are `random_bytes, placement2_bytes, default_bytes`.
Placement0 and both X/Y have positive use before selecting one ordinary full
pair. Zero roles retain their separate targeted source/native/bus proofs; no
extra navigation is added to increase counters. This one battle-entry window
does not prove all691 lists,323 payload headers or every placement/RNG branch.

One warning-free production build `b467d4ad..` uses the accepted518-file runtime
source copy, excluding unrelated Debug changes. Only the affected production
fingerprint is isolated; the canonical executable's hash and mtime are restored
exactly. All186 actual guards and518 source hashes remain exact; artifact
completion retires zero paths. Evidence is retained in
`target/rom-data-encounter-coordinate-cli/`: exact copies, build and command logs,
no-write/final/preservation/cleanup receipts and the production executable.
The actual dataset is `target/rom-data-encounter-coordinate-cli/windows-cli/resources/8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632/v1/e82-d81/generation-gMH5wq`.

One ordinary full pair with identical ten counters is next. No new checkpoint,
fixture/navigation, closed source/model/runtime/header/template acceptance
replay or performance measurement occurs. Linux, USA, native Windows, new32:9,
full placement ABI/all caller gates/global archive and the full ROM TODO remain
pending.


## Ordinary encounter coordinate battle-entry equivalence

One Wine ordinary equivalence pair closes the retained700-frame Giza menu to
White Flowers deployment case, guest19409→20109, using the actual e82-d81 CLI
dataset. Every complete machine state and guest clock matches the original-ROM
path. Classic, central and wide images, OAM and all five audio streams also
match. Native/fallback dispatch counts are `[3316146, 8054486]`; audio sample counts
are `[1536150, 1536150, 184800, 184800, 1536026]`. Counts describe correctness and executed coverage;
no performance measurement or comparison is made.

All ten coordinate usage fields match the production CLI exactly:
`{"bus_reads": 20, "default_bytes": 0, "headers_used": 1, "lists_used": 1, "placement0_bytes": 20, "placement2_bytes": 0, "random_bytes": 0, "source_bytes": 20, "x_bytes": 10, "y_bytes": 10}`. Placement0 and both X/Y bytes are positive,
covering one coordinate list and one encounter header. Random, placement2 and
default coordinates remain unobserved in this window; their targeted source,
model and poisoned-source native/bus proofs stay distinct. This case does not
establish all691 lists, all323 payload headers, full placement/RNG execution,
all caller gates or global archive coverage.

An optional five-line `FFTA_STATE_TEST_REQUIRE_ENCOUNTER_COORDINATES` block
requires positive source reads, placement0, X, Y, list and header use, and reports
all ten fields. Removing only that block reproduces the prior validator
byte-for-byte, including every older guard. One warning-free518-source Windows
test build `de67e015..` and one Wine filter pass. Only the affected test fingerprint
is isolated; core code matches the accepted runtime. All104 ordinary guards,
518 frozen pair-source hashes and186 closed CLI guards/518 production
source hashes remain exact. Artifact completion retires zero paths.

Evidence is retained under
`target/rom-data-encounter-coordinate-battle-ordinary/`: validator preimage and
added block, frozen source copy, executable, build and full-pair logs/results,
final/preservation/cleanup receipts. The original retained checkpoint and input
are used unchanged. Linux, USA, native Windows, new32:9, full placement ABI,
all caller gates, global archive and the remaining ROM TODO stay pending.
