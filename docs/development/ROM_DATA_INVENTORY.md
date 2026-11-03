# ROM data inventory and contract

Status: ROMDATA-01, 24 October 2026. This is an inventory of verified entry
points and explicit unknowns, not a claim to have located every asset. Exact
EUR and USA images are identified by `ffta_rom::{EUROPE_SHA256,SHA256}`.
Offsets below are file offsets unless prefixed with a guest memory region.
Do not apply a regional offset to another image or infer language coverage
from a shared rendering format.

The [overall ROM coverage map](ROM_COVERAGE.md) provides an offset atlas and
reproducible metadata-only JSON/HTML following the committed EUR extractor.
Dataset generation and historical ordinary-proof review are independent; new
resources appear with review pending, without freezing regeneration. It
separates speculative code, authenticated data, typed runtime availability,
ordinary positive resource use, shared references and unclassified regions.
Overlapping layers are not additive; ordinary distinct-read-byte coverage is
explicitly unavailable rather than inferred from aggregate read counters.

## First migration unit: graphics.obj-dimensions

The first family is the static OBJ dimension metadata, not sprite pixel data
or animation. Twelve records in shape-major, size-minor order contain two
little-endian u16 values, width and height in pixels. The range is 48 bytes,
uncompressed, with no interior pointers or dependencies. Invalid shape 3 is
outside the resource; guest memory beyond this table must keep its old behavior.

The USA draw routines at 0x080014c8 and 0x080016d4 read the table at
0x149510. The USA copied RAM sprite/visibility routines use a second table at
0xa398dc, copied to IWRAM 0x03006d28. EUR draw routines at 0x08001500 and
0x0800170c use the direct ROM table at 0x14ced8; their host decoder's old IWRAM
selection was corrected against the original instructions during ROMDATA-04.
EUR also contains an identical table at 0xd976e8 in the copied-code source area,
with the same relative source/RAM offset as the copied code. Both ranges are
mapped. The protected EUR boot-to-battle replay now observes actual alias reads
at frame 5400 (primary first read 541), with every full guest/RAM state matching
legacy reads through 8476 frames; see the
[validation protocol/results](ROM_DATA_VALIDATION.md). This is per-frame source
coverage, not attribution to a particular DMA/PC. An independent USA clean-boot
trace now reaches the original copier at 0x0808f74c, frame 791. Its own literals
at 0x8f764/68/6c specify source 0xa38adc..0xa3991c and IWRAM 0x03005f28:
3648 bytes, including the 48-byte alias at relative 0xe00. Original CpuSet SWI
at 0x0814186c performs 24 decoded halfword reads; all 13 initializer steps match
legacy full states. Independently inspected EUR entry 0x080924e8 and literals
at 0x92500/04/08 specify source 0xd968e8..0xd97728 and the same destination;
the EUR replay above does not attribute execution to that entry. These blobs
include executable code, which remains in ROM; only 48 metadata bytes migrate.
Existing consumers and original execution are described in
[sprite routines](../ffta_sprite_routines.md) and implemented in
`ffta_sprites::{decode,decode_battle,visibility_result,ffta_sprite_interpret}`.
Both exact local regional images have been read independently: all four
48-byte ranges have SHA-256
`6ff886b23cde2726a4ceb9cba8c5569bbb4dc6a1185769547f78dfbcb7e2d991`.
Both copied aliases are checked independently, not assumed equal to the primary.
Exact-byte occurrence searches find two copies per region. Extractor revision 2
records the newly discovered EUR alias; schema/decoder remain 1.
The format is language independent; this does not validate localized assets.

Decoded files retain indexed dimensions rather than a rendered picture.
Host readers must preserve any genuinely changed RAM dimensions; a static
resource cannot replace dynamic RAM without checking the consumer contract.
CPU/DMA/interpreter/native source reads and typed host consumers now use the
decoded files with --rom-data. No original instruction or decompression is
skipped. [Runtime evidence](../rom_data_runtime.md) distinguishes conditional
guest reads from ordinary in-view boot/host use; broader gates remain 05.

## Other families and migration matrix

Each entry names the known decoder/runtime reader, static versus dynamic
boundary, regional evidence, remaining extraction work and residual ROM use.
BG descriptor metadata and base palettes now have managed extraction and
direct consumption as detailed below. The bounded generic LZ77/packet decoders
remain groundwork for other linked graphics families.

- **BG tiles, palettes and image graphics.** `ppu.rs` reads hardware VRAM
  (4/8-bpp indexed tiles and bitmap modes) and palette RAM (BGR555). Those are
  dynamic destinations, not ROM asset boundaries. CPU copies, DMA and BIOS
  HLE feed these regions. `hle.rs` implements LZ77, Huffman and run-length
  decoding, but support for a codec does not locate a game's compressed blocks.
  New 1200-frame clean boots independently observe four LZ77 sources per region:
  USA 56f4a8/5704cc/56fefc/56f52c and EUR 76c64c/76d670/76d0a0/76c6d0.
  Consumed lengths 125/133/1481/2507 decode to 160/456/1996/2960 bytes, matching
  actual IWRAM 3860/EWRAM 3cb0 post-SWI outputs on Linux and Wine. Original callers
  independently reference 88-byte descriptors at USA 569104/EUR 7662a8; record 0
  field 0c resolves the first source. Fields 04/08/10 in record 0 point to outer
  wrappers containing the observed LZ77 streams. Typed halfword and indexed-pair
  packets now match actual regional writer instructions on Linux/Wine, including
  staged/chunked application. Literal pair syntax retains a destination guard
  after a contradictory original-code probe. The contiguous 162-record descriptor
  block is now source-authenticated/typed, with full metadata roundtrip and
  original conditional-selector checks for all records on both targets. Relative
  sources, optional transfer triples/variant source and tail bytes are retained.
  All field 04/08/10 packet links now resolve 352 verified source programs per region,
  with wrapper/raw/LZ77/inheritance/dictionary extents and original-loader output
  equivalence for every 162 records. EUR main field 00 graphics now has 50 verified
  custom-LZ sources and direct typed file consumption. EUR transfer programs
  have 42 typed controls/34 shared raw banks, eight-file managed publication and
  original tick/DMA evidence and direct CPU/DMA/native source consumption. All 162
  variant pointers are absent in this declared EUR block. Other catalog bounds,
  new USA checks and language completeness remain unverified;
  this is not a complete graphics catalog.
  [Decoder contract, exact evidence and remaining work](../rom_data_graphics.md).
  Managed descriptor extraction/preparation: implemented, v1/e6-d5; bounded
  offline LZ77/packet decoders also tested. Authenticated BG descriptor reads
  now consume a file-derived session projection through CPU/DMA/native APIs,
  with independent counters, real-file regional provenance and native boot
  parity. Graphics payloads still reach VRAM/palette through original producers.
  The complete base-palette links now identify 127 unique sources per region
  (125 LZ77, 2 raw), matching source/decoded hashes across regions and original
  producers for all 162 records on Linux/Wine. Preserve all u16 bits, including
  bit 15. Alternate environment streams now have a separate verified catalog:
  157 valid keys in each of two regional tables, 314 streams/256 B outputs per region,
  custom LZ/RLE with BE size header. All source/output hashes agree and original
  producers match on Linux/Wine. Five FF descriptor keys are unavailable;
  caller preconditions remain unknown. Typed production decoding now matches
  retained outputs and new original synthetic/negative cases on Linux/Wine.
  Full banks, original links/keys and consumed source metadata are preserved;
  malformed input/overruns are rejected. Managed extraction/preparation now
  publishes and authenticates the fifth fixed file under e7-d6, retaining e6
  and all original source metadata. Actual
  runtime consumption of alternates now serves literal source bytes from
  decoded banks through CPU/DMA/native reads, with exact typed links/control
  metadata. Poisoned-source original/native and lifecycle checks pass; declared
  EUR LCD B/TV pub source-use scenes and native GUIs now pass on Linux/Wine.
  Broader scenes and global FF caller preconditions remain separate. Base words
  now have typed readable extraction/preparation and direct 04 consumption:
  decoded banks for palette HLE, exact source views for CPU/DMA/
  native/guest BIOS reads, with original writes/cycles and state parity. Needed:
  indexed pixels, palette identity/indices, transparency, tile order and all
  writes/timing, not only screenshot similarity.
- **Terrain maps and movement planes.** `ffta_stage.rs` observes DMA and
  verifies two calculated 64x64 tile planes in EWRAM, then checks their VRAM
  crop. `ffta_movement.rs` observes a separate dynamic movement plane.
  `ffta_rom::map_producers_verified` independently pins producer code
  0x1af08..0x1b5d0 for each region; this is code, not a map archive. New EUR
  static dataflow ties staged loader 1B688/state 6→1AE88 to the existing
  root 7662A8/162 descriptors and all 486 field 04/08/10 links into 352 packet sources.
  Extractor/typed decoder/runtime already use `graphics.bg-descriptors` and
  `graphics.bg-packet-programs`; no duplicate terrain resource is needed.
  Auxiliary patches initialize the normal 16×16 height/flag grid 02007CB0,
  assigned to live context 02007F10+4/width +8/height +9. Original accessors read
  heights and mutate flags, so static seeds do not replace live cells or
  calculated EWRAM/VRAM/movement planes. Forty instruction/11 literal/nine BL
  fingerprints and exact descriptor/source/ownership checks pass without
  guest execution. Other context paths, ten-byte accessor 1DFB4 and complete
  flag semantics remain unclassified. Separate raster palette root 4872B0 now
  has a bounded source contract: 32 records/324 selector links/33 ranges/10496 B,
  64 index/192 header/10240 color B, no overlap with the 49 prior owners. The
  original 161-halfword copy reads a neighboring header or existing alternate-palette owner at
  489BB0, then overwrites the extra destination word; preserve all bus/timing
  behavior without duplicating that source. Strict lossless typed model and
  one Wine original-reader/copy oracle now pass all 10496 B/324 links/15 guards,
  64 initializers/32 getters/11 raw getters/7 raw prefixes/3 disabled/36 full 322 B copies,
  deferred/explicit-source/queued DMA and synthetic bit 15/range words. Whole
  RAM outside stack/SP/callee checks pass; 4753 instructions/202 PC are coverage only.
  Managed e55-d54 now publishes 51 files/50 resources/all 49 prior decoded/e54 retained,
  actual BG descriptor dependency and one immutable typed owner. Two new Wine
  filters pass 17 guards/lifecycle/all 10496 B and 28 original-interpreter-native
  complete-state-clock paths, compiled [15 index/46 header/24 color] B, original copies/
  wrap/queued DMA/raw neighbors and mixed alternate owner. Widths/mirrors/BIOS
  bus-mode/SWI/DMA checks pass; 646 guards. Actual production CLI create/reuse-no-write
  53 managed files/all 51 generation equality/old cache and one 1500-frame genuine
  cold boot Wine full-state-clock-image-OAM-dispatch-audio pair now pass. Ordinary
  50880 aggregate B = [2 index/322 header/50556 color], mask 2 source records include
  neighbor-header touches; not two full palettes or distinct B. 701 guards pass.
  Other contexts/new Linux-USA-native Windows/new 32:9 remain pending.
  [Source and ordinary contracts](../rom_data_maps.md).
- **UI choice lists.** [The bounded EUR contract](../rom_data_ui.md) identifies
  twelve immutable lists/36 entries from 16 direct widget 8BBF0 callers plus a six-
  entry 397A9C template used by two live filters. Thirteen ranges/42 entries/336 B
  are disjoint 50 owners; all 18 root literals/93 instruction fingerprints/static
  byte reconstruction pass. Eight u8 fields retain ID/X/Y/opaque and four raw
  navigation values. Live 0200F250 filter/copy/overrides and unmasked flagged
  coordinate addressing stay original; no guest count/target guard. Strict lossless
  model/shared source plan and one focused Wine original copy/getter/template
  oracle now pass 17 guards/all 336 original-loadsite B/18 copy prefixes/44 getters/
  43 idle calls/64 filter prefixes/2 raw-coordinate prefixes/703 hash-mtime guards;
  warning-free Windows test build. No full widget/native/full-state-clock claim.
  Managed e56-d55 now publishes 52 files/51 resources/all 50 prior decoded/e55 retained,
  no asset dependency/full set coherence/USA e8-d7 unchanged. Two reduced Wine checks
  pass 17 guards/lifecycle/all 336 typed B/13 lists and 29 original-interpreter-native
  complete-state-clock paths/compiled [92 ID/183 coordinate/92 opaque/368 nav] aggregate B,
  widths/mirrors/BIOS-SWI-DMA/old OBJ-raw-new/706 guards. Original RAM filters/input/
  cursors/counts remain live; no guest caps. Actual CLI create/reuse 52 generation/
  54 managed files no-write/old cache and one 320-frame Party full-state-clock-image-OAM-
  dispatch-audio pair now pass 32 aggregate B = [4 ID/8 coordinate/4 opaque/16 nav]/1 list/
  769 guards. Two negative battle contexts remain closed/no pairs; other 12 lists'
  ordinary use pending, no whole widget/new 32:9 claim. Existing fixtures were
  reused. New Linux/USA/native Windows checks are pending.

- **OBJ graphics, descriptor lists and animation.** `ffta_sprites.rs` decodes
  a signed/unsigned count followed by packed six-byte OBJ records, reads tile,
  priority/palette/flip fields and observes partial OAM DMA. Lists can live in
  ROM or RAM; animation, owner state, matrices and OAM are dynamic. Profiles
  and copied routine source ranges are independently hash gated per region in
  `ffta_rom.rs`; these are not a catalog of all lists or frames. A bounded EUR
  direct-literal catalog now identifies 107 references and 66 static uncompressed
  lists/160 objects/1092 source bytes. Typed signed coordinates and every packed
  attribute field reproduce both original EUR draws in 1056 Wine calls, including
  whole OAM buffers/matrices/guards. Managed e14-d13 preparation and typed guest/
  native/host consumption pass focused Wine publication/source/native/lifecycle
  checks and one 397-frame ordinary scene/actual CLI pair; see
  [the OBJ contract](../rom_data_obj.md). Other indirect/table roots, frame
  timing, sprite graphics/palettes and cross-resource links still need discovery.
  A further independently located EUR UI family has ten indexed OBJ/sequence
  bank pairs, 167 lists/284 objects/95 sequences/184 frames, 282 consumed source
  ranges. Relative indices and u16 durations match original constructors/
  scheduler body on Wine; e15-d14 managed and typed native/host/lifecycle checks
  pass with all raw sources altered. Three empty sequence direct-root domains
  are classified below; their indirect/live scheduling remains unverified. One
  160-frame ordinary menu/actual CLI pair proves positive index/list/frame/
  duration and host consumption with full-state/audio parity. Conservative
  control-flow discovery also locates sixteen additional references/seven new direct lists,
  now migrated under e16-d15: current static totals are 123 references/73 lists/
  173 objects/1184 source bytes. All original 66 records and other decoded files
  remain unchanged. Three focused Wine integration checks and one positive
  240-frame boot language-screen/actual CLI pair pass with new-list guest/host
  coverage. Other switch/indirect callers remain unclassified. New Linux/USA
  verification pending. EUR e17-d16 now adds an independently rooted eleventh
  pair b4d8e8/b4f4e8: one list/two objects/five zero-count holds, eight new
  source ranges/58 bytes. Explicit selectors and duration-source aliases retain
  the original nonrepeat contract and reuse typed header/index owners. All
  ten prior bank records remain unchanged; focused Wine original/read/native/
  host/lifecycle checks and one positive 240-frame Party/actual CLI pair pass.
  Constructor-fed sources at 38008/88328 are now resolved: e18-d17 adds five
  language-selected pairs from two five-entry pointer tables and one old-pair
  reference. The new 149 lists/574 objects/144 frames/166 ranges/4984 B retain original selector loads
  and duration 256/counter-wrap semantics; current 16-pair catalog has 317 lists/
  105 sequences/328 frames. All 11 prior bank records/other files stay unchanged.
  Focused Wine source/original/native/host/lifecycle and a positive 400-frame
  “Engage!” ordinary scene/actual CLI pair pass. The three older empty-record
  direct-root domains are classified in
  [the OBJ contract](../rom_data_obj.md#earlier-empty-sequence-caller-domains):
  twelve calls select 0; eighteen initializer mode producers select 0..3 and
  two updates select 2/3, excluding the empty indices. The unproved hold
  extension is rejected; indirect/live mutation remains unverified. All thirty
  root literals/47 instruction pins/50 owners pass static checks. Six possible
  selector bytes stay raw; duration aliases reuse existing indices 8/16/4. No
  resource/model/revision change or guest cap follows. Current e19-d18 adds 39
  custom OBJ-related word banks
  and six five-entry selector tables (13 direct/nine bank roots plus six
  canonical indexed paths/thirty roots), 45 disjoint ranges/62674 source B and
  131872 decoded B. Source-authenticated literals/pointers feed original
  decoder/copy paths with live scratch transforms, partial uploads and RAM
  tails retained. Fifteenth obj-graphics.json leaves all earlier files intact;
  Wine source/managed/native/lifecycle and 400-frame Engage parity/use pass.
  Other graphics/palettes and new Linux/USA verification remain open.
- **Normal unit OBJ initial pose.** The original
  2D394→233A8→CF144→22788 path supplies fullword animation 0/part 0. Existing
  metadata52 property 4/metadata14 halfword 04 project 121 IDs through 394834,
  first pointer cells and 121 counted sequences/477 operation 1 frames. Six
  OAM lists/31 objects at A50020+relative offset determine 333 raw 4bpp requests,
  135168 B/4224 decoded indexed tiles at 898A40+relative offset. 00B14 is a RAM
  DMA queue constructor, not a codec; 00800 executes real DMA3. The strict
  typed model/source plan owns 415 spans/146358 B, disjoint from all 51 e56 owners, with
  actual existing producer dependencies and opaque bytes preserved. Static
  94 instruction/13 BL/6 literal fingerprints and one new Wine source-model
  oracle pass 121 original reset paths/one constructor prefix/477 scheduler-DMA
  uploads/all pixel B/16 shape-size probes/11 guards. 1797 reserved/OAM/tail B remain
  preserved but unread in those source probes. EUR e57-d56 now publishes 53 files/
  52 resources/all 51 prior decoded/resource records/e56 retained/USA e8-d7 with
  actual metadata52/14 dependencies. Two new Wine filters pass 17 guards/lifecycle/
  all 146358 typed B/30 original-interpreter-compiled complete-state-clock paths,
  raw ID/animation/live RAM frame, original scheduler/DMA and 415 edges/mirrors/BIOS/
  SWI-DMA/new-raw-old OBJ merging. Actual production CLI create/reuse-no-write 55 files/
  all 53 generation files and one existing Party-unit 320-frame full-state-clock-image-OAM-
  dispatch-all audio pair pass 10962 aggregate B/3520 pixel B/2 sequences; other 119 ordinary
  sequences remain targeted-only. Job 0/1 values 65535 unresolved;
  no guest cap/full sprite/action catalog, other poses/parts/IDs/palette claim.
  [Contract, typed model and limits](../rom_data_unit_obj.md).
- **Normal-mode actor battle poses (source/model).** A finite original-selector
  projection over 121 normal metadata52/14 IDs plus 42 includes 29 body/8 extra-part
  bases, 10980 logical keys/8374 physical cells/5293 nulls, 2581 sequences/17894 frames,
  150 OBJ lists/676 objects and 3216 raw 4bpp requests. Byte-identical initial/equipment
  shares 146358/6984 B retain prior ownership; new 6021 spans/1455474 B are disjoint
  from all other 54 current owners. One warning-free Windows build/one new exact
  Wine source filter passes: 12 guards/16474 constructor prefixes/all three parts/
  5293 null+2451 sequence resets/100 commands/2858 scheduler-DMA/all 1063200 new pixel B.
  No old-owner original-reader replay or every actor/action reachability claim.
  Alternate 247 cross-product is rejected because animation 104 aliases directory
  394834; no invented extent/count cap. Dynamic/script/alternate pairing and
  actor pairing remain open. EUR e60-d59 publishes/consumes typed sources with
  actual metadata52/14/initial/equipment dependencies; 56 files/55 resources preserve
  all 54 old records/e59/USA e8. Two reduced Wine filters pass: the first covers
  17 guards/lifecycle/all 1455474 new typed B; the second covers 32
  original-interpreter-compiled complete-state-clock
  paths, all parts/null/raw IDs/commands 0-2-3-6-7-8/6021 edges/mirrors/BIOS/SWI-DMA/
  mixed old owners/raw. Actual CLI create/no-write reuse 56 generation/58 managed files
  and one existing F09 Wait 3530-frame full-state-clock-image-OAM-dispatch-all audio pair pass
  842236 aggregate B/815360 pixel B/five sequences, matching CLI. Source/model/managed/
  native/CLI/ordinary declared children closed; other 2446 new ordinary sequences
  targeted-only/alternate-script-live pairing/Linux-USA-native Windows pending.
  [Contract and typed model](../rom_data_unit_obj.md#normal-mode-battle-poses-source-projection-and-typed-model).
- **Alternate battle selector views (static source contract).** 65 alternate IDs/
  5850 logical keys resolve 5682 physical cells: 4709 nulls/953 counted-sequence cells
  naming 706 sequences/2771 frames, plus 20 explicit known-index-root views. All 20
  aliases have initial-pose directory/index witnesses; the one 4 B directory/index
  self-overlap is sprite 247/animation 104→sprite 0. No sequence-count filter,
  dropped binding or adjacency extent repairs the rejected blanket interpretation.
  30 OBJ lists/108 objects/1203 pixel requests/387008 B give 3321 new spans/410086 B after
  exact 6232 initial+52604 normal-battle+22 equipment shared B. All 55 e60 owner source
  hashes/overlap checks pass; no other conflict or mixed typed-item ownership.
  Tagged source model/one original Wine filter now pass 15 guards/all 5850 bindings/
  10544 constructor prefixes including 40 shared-target aliases/4428 null+606 proper
  resets/five new commands/1079 scheduler-DMA/all 342112 new pixel B/whole VRAM. No fake
  frames or old original acceptance replay. EUR e61-d60 now publishes/consumes
  3321 new spans with five actual dependencies; 57 files/56 resources/all 55 old records/
  e60 retained/USA e8 unchanged. Two new Wine filters pass: the first covers 20
  managed guards/all 410086 typed B/606 new sequences; the second covers 34
  original-interpreter-compiled full-state-clock
  paths/all 3321 edges/BIOS-SWI-DMA/mixed raw-old owner composition. StrictNull rejects
  extra fields while valid JSON stays identical. Source/model/managed/native
  and actual CLI closed: 57 generation/59 managed no-write reuse/all 57 accepted files
  equal/one-frame old cache attachment passes with zero new alternate use. Positive
  ordinary/live reachability pending; 81 existing EUR snapshots have no current
  new alternate-task lead, no new fixture/navigation/full pair.
  [Contract and shared target views](../rom_data_unit_obj.md#alternate-battle-selectors-and-explicit-shared-target-views).
- **Script 49 actor binding (live selector contract).** Decimal 49 (0x31), callback
  126CC0, has 167 five-byte instructions/18 actor selectors/13 bases. 151 selectors
  resolve live unit identity in twelve+twenty-four 264 B records; 16 resolve the live
  script registry/fallback, none encodes an immediate sprite. Context lookup
  matches unit pointers and uses current actor+34 sprite; group base is low 16 of
  u16 group×4, facing signed actor+1F, body flag 0 and same-base extra parts. Nearby 31
  only patches byte 01 in new-unit branch; existing units retain it, and 32 only
  repositions. No static blanket sprite join or new ownership is justified.
  Exact existing scripts plus 67 instruction/nine literal/eighteen BL pins pass;
  no build/guest run. Selector classification closed, resolved query census/
  sprite pairing still pending. e61/USA e8 unchanged.
  [Live selector contract](../rom_data_unit_obj.md#script-animation-selectors-require-live-actor-bindings).
  New test-only functional Wine 6000-frame census on retained snowstate/input resolves six
  command 49 sources with selector 16/group 2 to sprite 46/animation 8/parts 0-1-2,
  eighteen real constructor queries. Body is old normal ownership; two null
  cells 37C760/37C764 (8 B) are unowned, no new sequence/pixels. Original 49/body
  fallback steps and retained native constructor preserved. First all-native
  preflight failed before guest frames, corrected rebuild/actual census pass.
  No fixed selector→sprite claim; stable null-cell source contract and other 161
  instructions remain open, no new model/runtime/generation.
  [Observed query evidence](../rom_data_unit_obj.md#source-linked-script-query-observation).
- **Normal special-unit script poses (conditional source contract).** Exact
  CBD24 unit-byte 4 selector match/CBAD4 special predicate/CF144 logical metadata14
  normal field and original 9B824 assignment prove a conditional source join with
  override 20 clear/current same-unit normal sprite/facing 0..3. All 151 low script 49
  instructions give 20 selector/base references, 18 physical normal-sprite/base
  pairs/12 sprites/108 queries/76 nulls/32 sequences/241 frames/11 OBJ lists/64 objects/
  95 pixel requests/35968 B. No blanket all-sprite product or failed row removal.
  The audit of all 56 e61 owners finds 4780 initial+30586 normal shared B only, no mixed item/
  internal-role/extent conflict. New 33 spans/6436 B = 240 index/28 header/920 frame/5248 pixel;
  60 new cells/53 nulls/seven new sequences/46 frames (all op 1)/17 new pixel requests. Two
  observed 37C760/64 null cells belong to this ROM-derived plan, not private state.
  27 instruction/one literal/three BL fingerprints pass; no build/guest run.
  Source contract closed, model/original Wine/runtime/CLI/ordinary still pending;
  alternate/override/high 16/current-state cases remain separate, e61/USA e8 stable.
  [Conditional source projection](../rom_data_unit_obj.md#conditional-normal-script-poses-for-special-unit-selectors).
- **Equipment OBJ poses (source model).** Original 9BA84→CE1EC item properties
  3/9 and 9C07A→22788 select 252 eligible item records/61 sprites 124..184, normal
  bases 0/4/8 and parts 0/1. Explicit action 12/13 overrides add 185/bases 0/4;
  action 16/17 selects 126/base 16 except item 45→124. Two physical orientations
  give 748 cells/86 null/258 sequences/3439 frames. Only 1911 operation 1 frames reference
  OAM/pixels; 609 op 6/888 op 8/31 op 0 retain command words/tails. Twelve OBJ lists/
  25 objects yield 280 raw 4bpp requests/37440 B/1170 decoded indexed tiles. Exact 593
  spans/110666 B (248 directory/2992 index/69812 sequence/174 OAM/37440 pixels) are disjoint
  from all 53 e58 resource sources. 73 instruction/4 BL/4 literal/two dispatch static checks
  and one new Wine source oracle pass 1496 constructor-prefix aliases/86 null+
  258 sequence resets/six command cases/280 scheduler-DMA/all pixel B/10 guards.
  EUR e59-d58 publishes and consumes typed fields/55 files/54 resources with actual
  item dependency/all 53 old decoded/resource records/e58 retained/USA e8-d7. Two
  new Wine filters pass 17 guards/lifecycle/all 110666 typed B and 26 original-interpreter-
  compiled complete-state-clock paths, nulls/commands/live RAM frame/593 edges/
  mirrors/BIOS/SWI-DMA/new-raw-old owner composition. Actual CLI create/reuse-no-
  write 55 generation/57 managed files passes; Sheep 1000/Fight 700 equipment-negative
  locators have no full pairs. Authorized copied EUR Polly→Hildegard confirmed Fight 500
  single ordinary full pair now passes all state-clock-central-wide-OAM-dispatch-
  audio; equipment 4477 aggregate B/1408 pixel B/two sequences exactly equal actual e61 CLI.
  The other 256 sequences remain targeted-only for ordinary use. No live cap/adjacency extent or actor-body/
  script pairing claim; new Linux/USA/native Windows pending.
  [Equipment contract and model](../rom_data_unit_obj.md#equipment-poses-source-model).
- **Color-Mode unit/UI palettes.** Five original helpers CF49C/CF4E4/CF51C/
  CF5A0/CF5DC and 70 direct Thumb calls prove 30 spans/5750 B = 2873 full u16 colors+
  four u8 slice selectors, disjoint from all 52 e57 owners. Complete 512 B base blocks,
  exact supplemental fragments, 12 B shared colors and 9 special-unit selectors
  follow actual reads/copies, not adjacency; metadata14 supplies 51 active
  projections across 93 records. Preserve low 2 mode/low 8 argument/mode 3 default/
  companion mode 1 alias and all raw holes. Strict source model/shared plan and
  corrected reduced Wine filter pass 165 original pointers/99 copies/30 staged
  readers/54 special copies/12 slice prefixes/all 5750 B/eight guards. First oracle
  had a destination-register seed failure; nearby original MOV/LDR decoding
  fixes the test, no model difference. EUR e58-d57 now publishes 54 files/53 resources
  with metadata14 dependency/all 52 prior decoded/resource records/e57 retained/
  USA e8-d7. Immutable typed colors/selectors/raw-gap/prior-owner composition
  pass two new Wine filters: 17 guards/lifecycle/all 5750 B and 28 original-interpreter-
  compiled complete-state-clock paths, low 8/mode 3/live RAM palette/30 edges/mirrors/
  BIOS-SWI-DMA. Mixed 2 new+20 raw+4 old UI B preserves exact 2/4 charges. Actual CLI
  create/no-write reuse 56 managed/all 54 generation files and one positive ordinary
  Lutia 1700-frame full-state-clock-image-OAM-dispatch-all audio pair pass 1102 aggregate
  color B = [512 unit/512 companion/78 supplemental]/four fragments. Party 320/boot 240
  new family-negative locators have no full pairs. Remaining 26 ordinary fragments/
  other modes/shared-special-selector/whole allocator-scheduler remain pending;
  source/model/managed/native proofs stay closed.
  [Contract and validation](../rom_data_color_palettes.md).
- **Raw UI palettes.** EUR e20-d19 adds 21 source-authenticated raw banks/
  1328 full-u16 words/2656 B/23 direct-derived-Color-Mode references for both BG/OBJ
  palette-shadow halves. Original dispatcher 145c14→copied IWRAM 03005ee8 is
  independently initialized from 176 ROM bytes; actual selectors/copy paths,
  native poisoned-source/full-state/clock/bus/DMA and immutable lifecycle pass
  Wine, followed by a positive 240-frame ordinary boot/CLI pair. This distinct
  sixteenth file leaves all 14 earlier decoded files unchanged. Original live
  fades/palette publication remain; indirect sources and new Linux/USA checks
  stay open. [Contract](../rom_data_ui_palettes.md).
- **Additional custom UI BG/OBJ graphics.** EUR e21-d20 adds nine raw custom
  banks/ten checked literal original decode/upload paths, 57038 source B/
  84160 decoded B with no earlier-resource overlap. Return-length/count propagation
  resolves callers omitted by the old constant-count OBJ filter. Eight sources
  feed BG and one OBJ (two paths); 43bc28 uploads 544 of 2944 decoded bytes. Original
  source/native/lifecycle and one positive 320-frame ordinary Clan/CLI Wine
  pair pass. Seventeenth file ui-graphics.json retains all 15 earlier decoded
  files/e20; original work/clock/VRAM remain live. Current e22-d21 extends this
  same file to 16 banks/two language selector groups/two shared OBJ links,
  26 ranges/71474 B/119168 decoded B. Seven new banks and ten pointer literals add
  17 ranges/14436 B; original code jump tables/language/fallbacks and 5056 OBJ/2688 BG
  split remain. Reduced Wine original/native/lifecycle pass every new bank;
  all 9 old UI records/other 15 decoded files/e21 retained. One 1200-frame English intro
  full-state/image/audio/CLI pair positively consumes 4 selector B/2347 new-source B/
  1124 literal B. Other ordinary languages/indirect sources
  and new Linux/USA checks remain open. [Contract](../rom_data_ui_graphics.md).
- **Text and fonts.** `ffta_dialogue.rs` classifies already rendered layers
  and portraits, not strings, font encodings or localized text archives.
  `ffta_mouse.rs` identifies a name-entry row-count table via independently
  checked regional literals: EUR 0x3ad2fe (three pages, six bytes each), USA
  0x3a8320 (two pages). That table describes navigation widths, not character
  codes or text resource pointers. A bounded EUR short-string catalog now has
  five independently located selector tables with 107 entries each: 535 strings,
  478 compact/57 extended, 5468 source bytes; only its 65 referenced packed 4bpp glyph
  slots/metrics are extracted. Original language/copy/font/width readers match
  every pointer/output/guard in two reduced Wine checks (1331 calls). Ninth EUR
  managed file under 1/e11-d10 authenticates all 670 ranges; publication/reuse/
  repair/move/transactional activation and actual CLI pass Wine. Runtime now
  consumes typed indices/codes/glyph words/metrics through fixed CPU/DMA/native
  pages and separate counters, preserving original rendering and guest work.
  Five altered-source original/native chains (20 calls/full states/clock equal),
  source/SWI/DMA and unavailable-copy lifecycle pass reduced Wine. A broad offline
  model now covers all 9 pointer selectors/45 tables/16100 strings
  and 29 relative selectors/145 archives/17035 programs from the declared reader
  block, with 128 referenced glyphs and 33581 disjoint source ranges. Two reduced
  Wine checks match independent inventories and original pointer boundaries/
  archive roots/300 script outputs/63 new font slots. Dynamic commands retain
  operands; the controlled oracle supplies valid one-based context 1, not a
  gameplay claim. Broad managed EUR 1/e12-d11 now adds a tenth file with 33581
  source ranges, complete-set authentication and atomic prepare/reuse/repair/
  move/transactional activation plus actual CLI verified on Wine. e11/eight
  prior decoded files and USA e8 remain unchanged. Broad runtime is still
  directly consumed via typed pointer/string/index/header/program/font fields,
  including decoded custom-LZ literals and all 8 source counters. Fifteen
  original/native chains with 33581 raw sources altered match 90 full states/clocks;
  synthetic encoded-literal alteration and unavailable-copy lifecycle pass Wine.
  Earlier overlapping subset is authenticated without duplicate ownership.
  One retained 397-frame pub transition matches full states/images/audio with
  positive raw-program/font use; actual CLI reuse/old-cache parity passes Wine.
  The auxiliary archive at 4f95c4 now provides three typed name-keyboard programs,
  18 navigation widths and 180 derived cells. Its last referenced glyph 3174 ends
  exactly at 4f885c, bounding the complete 3175-slot packed font/metric bank.
  Both files join managed EUR 1/e13-d12 and direct CPU/DMA/native typed consumption;
  the complete bank takes over earlier overlapping font sources. Source/original,
  managed provenance/repair/move, native altered-source/full-state/clock,
  lifecycle and one 397-frame ordinary font transition/actual CLI pass on Wine.
  Keyboard ordinary scenes, other catalogs/control semantics/Unicode and new
  Linux/USA checks remain open. See the
  [text/font contract](../rom_data_text.md); five selector IDs are verified for
  this declared subset, not the complete localized text catalogs.
- **Gameplay tables.** Unit structures at EWRAM 0x020223a4, stride 0x90,
  are live state, not a static unit database; see
  [character metrics](../ffta_character_metric.md). Battle/shop mouse readers
  validate transient tasks/inventory. A bounded original shop/property chain now
  roots EUR item table 6e0bd0..6e3ad0: 376 records/32 bytes/12032 B, 19 property field
  selectors, independent 1..361/362..375 enumeration bounds and following table
  root. Raw halfword/byte/signed fields and live 02002030 modifier branches are
  distinguished; all 32 record bytes roundtrip and no e22 resource overlaps.
  Current e23-d22 publishes eighteenth items.json/resource gameplay.item-records
  and directly consumes typed fields via original CPU/DMA/native logic, retaining
  all 16 previous decoded files/e22. Reduced Wine original 14371+240 isolated-negative
  and managed/native/lifecycle checks pass; 86 native paths with altered source
  match complete states/clocks. One positive 320-frame Unit full-state/image/audio/
  CLI Wine pair uses 40 typed signed-field bytes. New Linux/USA checks remain
  pending. Following 20-byte table/item semantics/job/ability/law/AI catalogs still need contracts;
  live inventory/modifiers and secondary reads remain guest state/code/ROM.
  The following item-linked table now has a standalone typed source contract:
  225 records/20 bytes/4500 B/count 0..9/opaque/nine pairs, 472 active entries/eight FF,
  all 376 item owners checked, 220 referenced rows and independent next root 6e4c64.
  Wine 2244 original cases/4 synthetic queries plus 8 actual FF predecessor gates pass.
  All unused bytes remain exact. EUR e24-d23 now adds item-links.json with parsed
  item-owner dependency and typed CPU/DMA/native count/selector/value/opaque
  views. All 17 old decoded files/e23 are retained; reduced Wine managed/lifecycle/
  62 native paths with 4500 raw bytes altered match full states/clocks, including
  a typed crossing read of both adjacent owners. One positive 560-frame ordinary
  headgear inventory full-state/image/audio/CLI Wine pair consumes 574 new bytes
  (287 count/205 selector/82 value); first slots-only locator is negative, not rerun.
  New Linux/USA remain pending.
  The next table, 6e4c64..6e5464, has a lossless typed contract: 128 records
  of 16 bytes / 2048 bytes, a u16 alias plus 14 raw bytes, 23 checked LDR roots
  and an independently rooted following 52-byte table. One Wine source oracle
  passes 2581 original display/nonzero-alias/live-pair sum/choice/flag cases and
  19 synthetic edges, including original LCG state. EUR e25-d24 adds its fixed
  linked-metadata16.json resource: 20 files / 19 resources, retaining all 18
  previous decoded files and e24 generations. Two focused Wine integration
  checks pass 42 native paths with all new raw bytes altered and equal complete
  states/clocks, including adjacent typed ownership. One positive 520-frame
  Clan Mission Item full-state/image/audio/actual CLI pair reads 6 new display
  bytes. Other ordinary fields, broader field names and alias-zero caller
  preconditions remain unproved; new Linux/USA/native Windows checks are pending.
  A further source-only metadata52 catalog bounds 6e5464..6e6bf4: 116 records
  of 52 bytes / 6032 bytes, eight checked LDR roots, original 0..115 enumeration
  and independently rooted next table. Its 48 unsigned packed properties retain
  caller-alternate FF routing and nonreturning invalid/cyclic behavior. A new
  Wine source oracle passes 6369 real returning calls / 4 bounded loops and
  294 synthetic high-bit/redirect calls / 2 bounded loops, plus raw live-index
  tail readers and strict storage/source guards. Subsequent e26-d25 now adds
  linked-metadata52.json / gameplay.linked-metadata52, 21 files / 20 resources,
  retaining all 19 previous decoded files and e25 generations. Parsed normal
  link-owner guards and two Wine managed/lifecycle/native checks pass 75 paths
  with all 6032 new raw bytes altered and equal complete states/clocks, all-byte
  bus/SWI/DMA and adjacent metadata16/metadata52 typed crossing. One positive
  540-frame Change Jobs full-state/image/audio/actual CLI pair passes with
  664 new typed bytes; ordinary tail/opaque and new Linux/USA/native Windows
  checks remain unverified.
  The following four-byte source table now has a separate lossless contract:
  20 rows / 80 bytes at 6e6bf4..6e6c44, two selector/value pairs each, 26 active
  pairs and parsed metadata52 cross-owner indices. One focused Wine oracle
  passes 320 real / 24 isolated synthetic cases across both original count,
  contiguous gate, caller selection and live-count threshold prefixes. Counted
  nonzero slots and leading pairs stay distinct; unsigned values, signed
  comparison, tally wrapping and deeper live scans are preserved. Subsequent
  e27-d26 adds linked-metadata4.json / gameplay.linked-metadata4, 22 files /
  21 resources, all 20 previous decoded files/e26 retained. Parsed owner bounds,
  two Wine managed/lifecycle/native checks with 56 prefixes/all 80 raw bytes
  altered/equal full states-clocks and adjacent typed crossing pass. One positive
  540-frame Change Jobs full-state/image/audio/actual CLI pair reads 32 selector /
  6 value bytes. New Linux/USA/native Windows remain pending.
  The following metadata2 table owns 44 key/value records / 88 bytes at
  6e6c44..6e6c9c, including real row 43. Original direct u8 indexing reads all 44;
  inverse u8 lookup scans only 43 and returns first stored key / miss 0. Parsed
  metadata52 key owners, independent next 48 B source, strict lossless source/JSON
  guards and one Wine 348 real+13 synthetic oracle pass. Live byte 7 wrapper prefix
  retains its frame; global caller bounds/semantic value names remain unproved.
  EUR e28-d27 adds linked-metadata2.json/gameplay.linked-metadata2, 23 files/
  22 resources; all 21 prior decoded files/e27 retained. Two focused Wine managed/
  lifecycle/native checks pass 21 paths with 88 raw B altered/equal full states-clock,
  typed boundary/DMA/SWI. One positive 1840-frame Pub mission full-state/image/audio/
  actual CLI pair reads 4 value/0 key B; inverse key use stays targeted-only.
  Negative Change Jobs/Rumors/Missions locator evidence remains closed.
  Default48 owns 6e6c9c..6e6ccc/48 B from exact constructor copy length,
  with structural byte/halfword fields and original signed-ID/selector/argument/
  OR flags patches. Mode 1/2 and four alternate IDs 20..23 stay distinct. One Wine
  oracle passes 271 real+30 synthetic constructor prefixes after original IWRAM
  copy initialization; allocation/live-unit tails and four alternate 192 B/ROM
  pointer/following 240 B remain unowned. EUR e29-d28 adds fixed default-constructor-
  template.json/resource gameplay.default-constructor-template, 24 files/23 resources,
  all 22 previous decoded files/e28 retained. Two focused Wine managed/native/lifecycle
  checks pass 16 paths/all 48 raw B altered/full states-clocks equal/IWRAM helper and
  eight alternate/rejected paths with 0 new reads. One 397-frame Pub full-state/image/audio/
  actual CLI pair reads 96 new B across every structural field category.
  Static next contract separately roots four alternate 48 B records/192 B at
  6eca78..6ecb38 and pointer 70cd58..70cd5c/4 B, 196 B total/no e29 overlap. Original
  gated ID 20..23 copy and five equipment-slot reads per record/parsed 376 item owners
  are checked; byte 01 meaning/neighboring headers remain unclaimed. Strict typed 4-record/
  pointer catalog and one Wine 24 real+8 synthetic original-copy/live-patch oracle
  pass source/JSON/fixed array/order/ID/item-owner/stack/outer RAM guards. EUR e30-d29
  adds constructor-alternate-templates.json/gameplay.constructor-alternate-templates,
  25 files/24 resources/all 23 previous decoded files/e29 retained. Two focused Wine
  managed/lifecycle/native checks pass 14 paths/both raw source spans (196 B total) altered/equal
  full states-clocks, native pointer/all 4 records mask 15 and 5 zero-new-use paths.
  One 3600-frame Lutia deployment full-state/image/audio/actual CLI pair reads 45 B (4 pointer,
  41 record 20 B/mask 1); remaining records/full-copy behavior targeted-only. Closed
  source 24+8/default 271+30/native 16/397 proofs not repeated.
  Next static 14 B contract: 93 records/1302 B physical 6e6f6c..6e7482/logical IDs 2..94,
  original address bias 6e6f50/predicate>1, independent next 12 B source 6e7484 and
  unowned 2 B gap. Exact reconstruction/no e30 overlap/parsed field-range checks pass;
  byte 02 live-job link supported, byte 01 semantic owner unproved. Strict lossless
  model and one Wine 504 real+13 synthetic getter/live-palette/initializer-prefix
  oracle pass exact source/JSON/bias/owner/stack/unit/palette/outer RAM guards.
  EUR e31-d30 adds linked-metadata14.json/gameplay.linked-metadata14,
  26 files/25 resources/all 24 prior decoded files/e30 retained. Two focused Wine
  managed/native/lifecycle checks pass 37 paths/raw 1302 B altered/full-state-clock
  equality; normal 0/1 no new source. One 397-frame Pub full-state/image/audio/actual CLI
  pair reads 27 B/two records/all four field categories. Separate 6e6dbc 4 B root/
  12 B root contracts, global bounds and new Linux/USA/native Windows remain open.
  Two distinct 12 B indexed banks 6e7484..6e78d4/92 rows and 6e78d4..6e7994/16 rows
  now have a strict lossless source model: 108 rows/1296 B/5184 two-bit pairs.
  Shared 135a1c reader uses the full 32-bit row input/signed-wrapping slot/u8 mode; caller loops
  slots 0..43, all 48 stored pairs retained. Next 6e7994 global root excluded.
  One reduced Wine source oracle passes 1640 real/10 raw-outside-wrapping/384
  synthetic cases plus exact LDRB/address/source/JSON/callee/stack/RAM guards.
  EUR e32-d31 now adds linked-metadata12.json/gameplay.linked-metadata12,
  27 files/26 resources/all 25 earlier decoded files/e31 retained. Two focused Wine
  managed/native/lifecycle checks pass 49 paths/raw 1296 B altered/full-state-clock
  equality/both banks positive/three outside 0 new B and bank-boundary/SWI/DMA.
  One 2200-frame Echo Grass confirmation full-state/image/audio/actual CLI pair is positive.
  Global bounds/flag semantics/global bank and new Linux/USA remain pending.
  The separately rooted global pair source 6e7994 has a minimum 44 pairs/11 B
  source contract from original 1377b4 RAM-mask loop 0..43/mode 0. Three bytes
  before independently rooted 6e79a2 remain unowned; no 12 B row stride inferred.
  Strict fixed 11×4 groups and one Wine 102 real/10 raw-wrap/91 synthetic oracle
  pass wrapper/both bits/live-mask/early exit/address/callee/stack/RAM/JSON/source
  guards. EUR e33-d32 now adds global-pair-flags.json/gameplay.global-pair-flags,
  28 files/27 resources/all 26 prior decoded files/e32 retained. Two focused Wine
  managed/native/lifecycle checks pass 28 paths/raw 11 B altered/full-state-clock
  equality/new typed use/prior-typed and raw-tail SWI/DMA crossings. Actual CLI
  creation/no-write reuse/old cache/cwd pass; 2200 Echo Grass 0 global B is negative,
  no full pair. Positive ordinary global use/full pair/high mode/new Linux/USA
  remain pending, bounded descriptor/live-mask gates guide future selection.
  Evaluation parameters have a separate 38 B source contract: 7 divisor-weight
  terms/2 conditional pairs at 6e79a2..6e79b4 and 5 class 1..5 threshold records at
  6e79c6..6e79da. First unsigned u8 quotient/remaining 6 signed32/wrap/low 8 bonus
  predicates/original LCG/signed low 16%256 thresholds retain original code.
  Strict model and one Wine 132 actual-source/52 mutated-source oracle pass,
  including 12 live-score prefixes and original RNG/class 0 skip. 14 B score gap/4 B row 0
  unowned; no 32/24 B gap inference/foreign owner. EUR e34-d33 adds evaluation-
  parameters.json/gameplay.evaluation-parameters, 29 files/28 resources/all 27 prior
  decoded files/e33 retained. Two focused Wine managed/native/lifecycle checks
  pass 27 prefixes/raw 38 B altered/full-state-clock equality/class mask 31/class 0 zero/
  SWI-DMA/raw gap checks. One 1840-frame Pub mission full-state/image/audio/actual CLI
  pair reads 15 score B; later 400-frame Yes/fee CLI/Wine full pair reads 1 threshold
  class 1/mask 1, classes 2..5 remain targeted-only. Whole tails/
  semantic labels/new Linux/USA remain pending.
  Next static 10 B contract roots 6e79da/*10/0 default and normal index 0..99 from
  04B3E0..04B416, 100 records/1000 B through 6e7dc2. 12 zero B before independent 6e7dce
  remain unowned; candidate 101 rows rejected. Eight u8 deltas feed original live
  level/progress/flags through 04870c; byte 08 feeds live item price discount,
  byte 09 structurally unused. Unsigned/signed/u8 wrap/min-price/zero behavior and
  actual parsed Items base are distinguished; no stored foreign key inferred.
  Exact roundtrip/no e34 overlap/roots/stride/fields pass. Strict source model and
  one Windows/Wine oracle pass 362 real/68 synthetic publisher/delta/full-price
  cases with actual parsed Items and complete EWRAM/SP/callee guards. Source
  work closed. EUR e35-d34 adds 30th file/29th resource; all 28 old decoded files/
  e34 retained. Two focused Wine managed/native/lifecycle checks pass 27 original/
  interpreter/native paths, raw 1000 B altered/full-state-clock equality, typed
  boundaries/mirrors/BIOS/SWI/DMA/coexistence. Actual CLI no-write reuse and
  one 600-frame Shop full-state/image/audio pair read 7 discount B/record 4; ordinary
  deltas targeted-only, new Linux/USA remain pending.
  Next static progression-gates roots 6e7dce/stride 12/index 0..63, 768 B to 6e80ce;
  independent next root D23E6→D24A8. Byte 00 unused/halfword 01 return/byte 03 output/
  eight live unsigned minima and original u64 mask order preserved structurally;
  row 63 zero return still marks bit 63. Roundtrip/no e35 overlap/opcodes and strict
  source model/one Wine full 0487c8 oracle 80 real+25 synthetic pass all 64 rows/eight
  live unsigned minima/mask order/zero-success/704 read bytes/full RAM/SP/callee
  guards. EUR e36-d35 adds 31st file/30th resource/all 29 prior decoded/e35 retained;
  two focused Wine managed-native-lifecycle checks pass 19 original/typed/native
  full-state-clock paths/raw 768 B altered/two-page typed bus/zero-success/full mask/
  SWI-DMA/gap/coexistence. Actual CLI no-write/old cache and one 1000-frame Wine normal
  dispatch completion full-state/image/audio pair pass 209 requirements/8 delta B;
  return/output ordinary targeted-only. Separate 400-frame Yes/fee prefix closes
  class 1 threshold ordinary use; downstream receipt-FK/new Linux/USA remain pending.
  Next selection gates own 64x16/1024 B 6e80de..6e84de from D2390 physical 1..64,
  not row 0. Hard unsigned minima 01/02/04, ANY active 06..0d and flag 0e preserve
  physical first match, zero-condition store 0 and exhausted 64 no-store. Actual
  row 52 sentinel/53..64 storage distinguished; independent next root 6e84e0 leaves 2 B.
  Static contract/roundtrip/no e36 overlap and strict source-only model pass; one
  Wine full D2390/reset-count-flag oracle 138 real+80 synthetic checks 831 actual field B/
  all 1024 B roundtrip/ANY/unsigned/zero-terminal/no-store/full RAM-SP-callee.
  EUR e37-d36 now adds 32nd file/31st resource, all 30 prior decoded/e36 retained.
  Two focused Wine managed/native/lifecycle checks pass 18 full D2390 paths/raw 1024 B
  altered/full-state-clock, all 6 fields/64 record mask/bus-BIOS/16 B row 0+2 B gap raw/
  SWI-DMA/old gate 2-raw 16-new 2. Actual CLI create/no-write reuse/old cache and one
  1000-frame Wine ordinary dispatch completion full-state-image-audio pair pass 277 new B/
  all 6 fields/first 26 records; 400 Yes-fee 0 B closed negative/no full pair. New Linux-USA/
  all 64 ordinary/semantic display remain pending.
  Next aggregate key table owns 6e9a5c..6e9cb6/602 B (300 u16 slots+ffff),
  D22FC global and 13FB1C context counters share exact source, next root 29AF4 at end.
  Strict source-only model/one Wine 32 real+26 synthetic oracle pass both complete
  counters/all 602 B/key+2ff u16 wrap/duplicates/early stop/flag 36 OR linked 017f/
  context isolation/full RAM readonly-SP-callee guards. Other indexed UI reader roots
  static-only. EUR e38-d37 now adds 33rd file/32nd resource/all 31 prior decoded/e37
  retained. Two focused Wine managed-native-lifecycle checks pass 16 full global/
  context paths/raw 602 B altered/full states-clocks/key-terminal-300 slot mask/
  bus-BIOS/SWI-DMA/typed 2 raw 2 terminal crossing. Actual CLI create/reuse-no-write/
  old cache and one 1000-frame Wine ordinary dispatch full-state-image-audio pair pass
  1204 source B (1200 key/4 terminal)/all 300 slot mask; counts include rereads, both
  ordinary entrypoints not inferred. New Linux-USA/whole other UI readers pending.
  [Item contract/evidence](../rom_data_gameplay.md).
- **Audio samples, instruments, songs and sequencing.** `engine.rs` detects
  driver families; `mp2k.rs`, `gax.rs` and `rdrv.rs` shadow live voices while
  leaving original mixing/sequencing running. `MemView` resolves ROM/RAM
  ranges, and each driver has its own sample/loop/instrument contract. Driver
  detection is not an exhaustive instrument or song index. Exact EUR now has
  [original M4A root contracts](../rom_data_audio.md): 3 player configurations/
  36 B, 640 directory entries/5120 B, 455 unique headers/10080 B including 186 aliases
  to a zero-track header, 1610 track roots and four bank roots. Independent
  lossless decoding/static readers/live retained-state pointers pass. Strict
  source model/457 ranges/15236 B and focused Wine original-reader 649 cases+one
  raw lookup+six synthetic variants pass; 13477 distinct original-read B, 1759 B
  retained-only. EUR e41-d40 managed audio directory publication/typed CPU-DMA-
  native consumption/lifecycle pass two focused Wine checks/44 full-state-clock
  paths; production CLI and one 400-frame ordinary Wine pair pass 179 source B
  (36 directory/79 header/64 track-pointer), three songs/headers and 0 player reads. There
  is no local 640 index guard or generic 128 tone bound. New producer inventory
  follows 1610 track roots/177047 bytecode B to 390 bank/program pairs, 429 tones,
  347 keymap B and 277 PCM waves/1931528 nominal payload B. One focused Wine check
  confirms 390 BD copies/348 mapped links/276 extra signed interpolation endpoints
  plus one fixed 8 nominal endpoint. Source plan 736 ranges/1941759 new B excludes 4 B
  e41 alias. Shared bounded parser/strict tone-keymap-signed PCM-CGB model now pass
  one Wine decoder/independent-hash/alias-dependency check and 2 original CGB copy
  checkpoints. e42-d41 now publishes audio-tones.json/37 files/36 resources with
  actual directory alias dependency; all 35 prior decoded/e41 retained. Two reduced
  Wine checks pass 26 guards/lifecycle/all 1941759 B/25 guest-native full-state-clock
  checkpoints/736 edges-SWI-DMA. Actual MP2K hook/tick reads typed header/PCM values
  for all 277 waves; bit-exact nonzero shadow samples/complete host state-clock pass.
  Production CLI create/reuse-no-write 39 managed files/old cache and one 400-frame ordinary
  Wine full-state-image-audio pair pass 516956 guest B/3093208 host B (1056 header/
  3092152 PCM), CGB ordinary 0. CLI default shadow off explicit; differential arms it
  after load. Partial probes are not full mixer ABI.
  New strict sequence source model identifies 105253 physical instructions with
  single ownership over 805 ranges/177047 B and 1610 directory roots. Canonical branch/
  pattern IDs and typed explicit/inherited operands preserve dynamic sequencing.
  49 clock B/36 original command words 144 B/CD8-9 dispatch 8 B add 3 spans/201 B: 808 sources/
  177248 B disjoint from all 36 prior owners. One Wine model/source check passes
  all fields/hashes/27 guards/404 new original-reader checkpoints. e43-d42 now publishes
  sequences: 38 files/37 resources/all 36 prior/e42 retained/actual directory+tone bank
  dependencies. Two Wine checks pass 25 guards/lifecycle/all 177248 B/all instruction-
  table masks/35 original-native full-state-clock checkpoints/808 edges-SWI-DMA/
  live RAM running override/raw fallback/tone-sequence guest+generic-host boundary.
  Production CLI create/reuse-no-write 40 files/old cache and one 400-frame ordinary Wine
  full-state-image-audio pair pass 819 B (366 opcode/262 operand/191 clock), 185 source
  records including lookahead; 0 command/extended/generic-host. Seven further driver
  tables 370b20..370d04/484 B have typed source model: 180 PCM packed keys/12 u32 bases/
  12 u16 sample counts/132 CGB packed keys/12 i16 bases/60 noise/16 volume. One Wine check
  passes 1138 new original reader probes, all 484 B/source-JSON and 15 guards; sample
  selector 0/13..15 and volume 16/64/255 aliases remain original/no added bounds.
  Eight targeted gain returns confirm balanced route levels 16/23/31 and only
  single-side cap 15; global ordinary 16-domain remains unproved. e44-d43 publishes/
  consumes 39 files/38 resources/all 37 prior/e43 retained with actual sequence boundary
  owner. Two reduced Wine checks pass 19 guards/lifecycle/all 484 B/424 entry bits/
  28 original-interpreter-native full-state-clock paths/mixed Thumb-ARM/adjacent
  aliases/boundaries-BIOS-SWI-DMA/generic-host. Production CLI create/reuse-no-write
  41 files/old cache and one 400-frame ordinary Wine full-state-clock-image-audio pair pass
  756 aggregate byte reads/67 entries from PCM-CGB key-base tables; 0 sample count/noise/
  volume/generic-host ordinary reads remain explicit. Bank gaps, other
  wrapper roots/LUTs, language variants and references stay open.
  Offline catalog: implemented for this initial
  EUR directory/produced tone-keymap-PCM-CGB-sequence-driver scope; managed extractor: e44-d43; runtime:
  guest mixer/DMA and shadow samplers. A PCM export alone loses sequencing,
  pitch, envelopes, loops and original guest timing.
- **Executable code and opaque data.** Native labels and copied IWRAM
  instruction ranges are code, explicitly excluded from decoded assets.
  Unknown regions stay in the original ROM. Do not label a sweep of plausible
  compression headers as a verified asset catalog. Code/data interleaving,
  embedded pointers/constants and opaque regions prevent ROM-free startup.

## Version 1 base dataset contract

The managed set contains `graphics.obj-dimensions`, `graphics.bg-descriptors`,
`graphics.bg-base-palettes`, `graphics.bg-alternate-palettes` and
`graphics.bg-packet-programs`. IDs remain semantic: OBJ index `shape * 4 + size`,
BG record index within the authenticated 162-record block; source offsets are
provenance, not resource IDs. All five families have direct EUR consumption;
USA packets stay verified-only pending consumer validation. Whole-set activation
authenticates every file. Other linked payloads/programs remain ROM.

- `manifest.toml`: schema revision, extractor revision, decoder revision,
  exact source ROM SHA-256/size and regional profile; a coverage list naming
  only the decoded families; resource ID, relative file name, typed format,
  decoded-file SHA-256, dependency IDs and all original source ranges/hashes.
- `obj-dimensions.toml`: twelve typed width/height records in index order.
  The file hash covers its exact bytes. The decoder verifies dimensions,
  length, source ranges and that canonical little-endian encoding reproduces
  the source table. No absolute paths, timestamps, ROM bytes or host pointers
  appear in the manifest. Generated assets remain private local files.
- `bg-descriptors.json`: all 162 typed metadata records with exact source
  roundtrip, validated relative links/transfer triples and retained tail bytes.
  It does not contain linked graphics/program payloads. Its source range is
  independently authenticated for each profile; file hashes cannot authorize
  edited base semantics. All five resource files contribute to complete dataset identity.
- `bg-base-palettes.json`: all 162 record-indexed base palettes as full u16 banks,
  with source aliases preserved in 162 manifest ranges and a dependency on BG
  descriptor IDs. Known source codecs raw tag 0 and LZ77 tag 10 are bounded and exact extents
  exclude padding. Preserve bit 15 and bank/index order; alternate environment
  programs use the separate verified alternate family. Exact consumed source bytes are
  retained beside banks for compatibility reads; both forms must agree.
  Decoded banks feed verified LZ77 HLE while raw CpuSet uses file-derived reads.
- `bg-alternate-palettes.json`: two 157-key catalogs of eight 16-word u16 banks,
  162 descriptor keys including five unavailable FF, original relative u16 links
  and exact consumed custom-codec metadata. Two 314 B link tables plus 314 streams
  retain their own manifest ranges/hashes and a dependency on descriptors.
  All fields and file hashes are authenticated; current activation verifies this
  family and its guest source reads now consume decoded literal bank bytes.
- `bg-packet-programs.json`: 352 unique typed programs/486 descriptor links,
  parent graph, exact encoded and post-wrapper body metadata plus ordered writes.
  352 source ranges depend on BG metadata. Every field/file hash is authenticated,
  with source-equal e8 publication/repair/retention verified for EUR/Wine. Current
  EUR runtime consumption now retains typed values for source literals and
  predecoded HLE body writes, preserving original dynamic fill and guest timing.
  USA stays verified-only. [Contract](../rom_data_graphics.md#direct-european-packet-consumption).
- EUR `bg-main-graphics.json`: 50 typed graphics resources/162 links, exact encoded
  metadata, destination offsets and full little-endian words. Fixed source views
  serve original CPU/DMA/native readers; literal bytes come from decoded words.
- EUR `bg-transfer-programs.json`: 42 controls/34 shared raw tile banks/97 active
  links and 162 absent variants, 76 authenticated source ranges. Current e10-d9
  publication validates the whole set and includes every file hash in identity;
  this transfer family now serves typed control/tile bytes through existing
  CPU/DMA/native readers without replacing live animation state or guest timing.
- Schema and decoder identity are separate from source identity and compiled
  code identity. For a base dataset, matching its own declared hashes is
  insufficient: verify the expected source ranges and reconstructed source
  digest against the selected exact image. User edits belong to a future mod
  layer; do not silently accept them as base data.
- Keep fixed names and bounded input: 256 KiB manifest (including 162 base and 316 alternate
  provenance entries), 16 KiB OBJ, 256 KiB BG descriptors, 1 MiB base palettes,
  4 MiB alternate palettes, 64 MiB packet programs and 64 MiB total. These guard malformed input, not extracted-data space targets.
  Reject unsupported schema/decoder revisions, duplicate/missing resources,
  bad hashes, unknown fields, truncated/out-of-range data and path escapes.
  Publication must validate all files before making a directory visible.
- Automatic path: portable `Data/resources/<rom-sha256>/v1/e8-d7/`;
  installed uses local application data `FFTARecomp/resources`. Immutable
  `generation-*` directories contain six files; an atomic relative `current`
  file selects one verified generation. Revisions have separate paths and an OS
  lock coordinates writers. `rom-data prepare` and opt-in `play --rom-data`
  implement reuse/recovery, retaining e1/e2/e3/e4/e5/e6/e7 generations; direct OBJ, BG
  metadata and base-palette consumption is opt-in. Other linked payloads/programs
  remain ROM.
  Normal enablement awaits 05.
  Native caches and saves remain separate.
- Decoded base data are immutable host resources, shared for a session and
  rebuilt/rebound on load/Undo/clean-boot restore. Do not serialize their
  pointers or duplicate files in `.ffstate`. Existing ROM/BIOS identity remains
  valid only after strict base equivalence is proven. Modified data require
  a separate content identity and compatibility policy before enabling mods.

## Gates before normal runtime use

ROMDATA-02 must first validate synthetic malformed inputs and both local
regional images. ROMDATA-03 needs preparation/reuse/recovery/move tests.
ROMDATA-04 must trace every consumer, demonstrate decoded-file provenance
and make fallback visible. A host-only migration does not close guest reads.
ROMDATA-05 needs deterministic state/image/audio equality, lifecycle checks
and measured startup/memory/frame cost against the existing in-memory ROM.
No speedup, removal of the ROM dependency, complete graphics migration or mod
support follows from this first family.

### Alternate palette mode naming and caller scope

The original EUR Options UI and unchanged-frame snapshot reads establish
LCD A=0, LCD B=1 (second alternate catalog), TV=2 (first). Only Save changes Yes
commits the candidate. Four direct Thumb callers per region and the original
setter/confirmation are inventoried; no local descriptor-FF guard was found.
Global FF reachability is unresolved. See the [bounded caller contract](../rom_data_graphics.md#alternate-palette-callers-and-color-mode).
Worldmap/Party mode changes alone produced 0 alternate reads and cannot count as
actual alternate-palette scene coverage.
The subsequent Cyril-pub BG 62 transition does consume both catalogs: declared
Linux/Wine LCD B/TV full-state/audio and GUI checks pass. See the [scene results](ROM_DATA_VALIDATION.md#alternate-palette-e7-scene-and-gui-acceptance).

### Descriptor-linked packet programs

The declared field 04/08/10 catalog has 122 paired/117 halfword/113 auxiliary unique
sources per exact region: 352 disjoint ranges, 398227 encoded bytes, 523794 decoded
body bytes. Four-byte wrapper 01/11 selects raw/LZ77 body; upper 24 bits are a parent
descriptor ID or FFFFFF initial fill. All aliases/content/parent graphs match
EUR/USA, no cycles, maximum 2 nodes. Paired fill is live EWRAM 02007f5e; other two
families fill zero. Preserve this initialization and ordered writes.
Four Linux/Wine original-loader cases each pass 486 wrapper invocations with
complete destination/guards/SP checks. The core's new referenced_extent bounds
raw dictionary reads separately from stream termination. See the [complete
catalog contract](../rom_data_graphics.md#complete-descriptor-packet-catalogs).
The sixth managed e8 file authenticates these programs and now serves their
actual runtime source/body reads for EUR/Wine. Field 00 and transfer programs
have separate verified EUR contracts below; broader catalogs remain open.

### Typed packet resource model: EUR priority

The bounded offline packet model now retains 352 unique programs and 486 descriptor
aliases, ordered u16/u32 writes, original parent graph and exact encoded/body
metadata. Source authentication rejects coherent base edits. Linux/Wine EUR
checks reuse all retained original bodies; readable JSON is byte-identical.
The new model is verified for EUR only; USA remains pending by user instruction.
See the [typed contract](../rom_data_graphics.md#typed-descriptor-packet-programs).
The file is published/verified in the e8-d7 managed dataset and consumed by the
EUR CPU/DMA/native/HLE paths. Three focused Wine checks and one 397-frame actual
native transition pass with positive typed consumption and complete-state parity.
New Linux/USA checks remain pending. See the [consumer contract](../rom_data_graphics.md#direct-european-packet-consumption).

### Main graphics field00: verified EUR offline catalog

The 162 descriptor links name 50 disjoint sources: 23 tag 20, 27 tag 22; exact consumed
sizes total 621923 bytes and decoded outputs 835232 bytes. Staged producer 0801b688
phase 1 selects custom BE-size LZ/RLE 080054a0, the same original decoder used by
alternate palettes. These are not BIOS Huffman streams. Tag 20's upper 24 bits
declare output bytes; tag 22's upper 24 bits are a byte destination offset followed
by a nested 20 header and the same custom stream. Inner/outer sizes agree.
All outputs/offsets are whole 32-byte tiles and fit the original 0x4fe0-byte
graphics destination. Phase 1 writes the compact output to EWRAM 020091a0;
phases 2/3 queue DMA32 transfers of each half to target+offset(+half for phase 3).
Tag 22 does not supply or replace the live destination prefix.
The normal producer at 0801b2cc reads field 00 at 0801b310..324 but branches for
12/02/10 and a copy case; it alone does not prove how 20/22 are prepared.
The separate palette-copy indirect pointer at ROM 370eb4 selects IWRAM 03005ee9;
08145c14 is only the bx r3 trampoline. It is not an independently located main
graphics decoder.

ROMDATA-01-BG-MAIN-GRAPHICS / ROMDATA-02-BG-GRAPHICS-DATA are complete for
this declared EUR catalog. The bounded typed model retains little-endian words,
offsets, exact compressed metadata and all alias links; canonical IDs follow
first descriptor occurrence. Validation rejects truncations, invalid references,
overruns, overlap, wrong IDs, size mismatch and coherent source edits.
Nine focused Wine checks pass, including 50 original staged decoded outputs with
exact live source cursors, full destination/guards/SP, and phase 2/3 DMA layout for
one 20 and one 22 source: 54 calls/11142126 instructions. Readable JSON roundtrip and
base authentication pass. Evidence: target/rom-data-bg-main-graphics/
windows-europe/result.json, windows-results.json and bounded disassemblies.
Managed EUR e9 publication and direct file-backed runtime consumption are now
complete for the declared Wine scope: retained words/metadata/literal maps,
fixed 769..895 source pages, original guest instructions/clock and separate
positive-use counters. Four poisoned-source original/native cases and owner
lifecycle pass, followed by one 397-frame full-state/image/audio pub transition
and actual CLI parity/no-write reuse. New Linux/USA verification remains pending.
[Current consumer contract](../rom_data_graphics.md#direct-european-main-graphics-consumption).


### EUR descriptor transfer and variant programs

The five transfer slots have 83/6/5/3/0 active links across 162 descriptors:
97 links, 42 control sources and 34 shared raw tile banks. All 162 variant links are
zero in this verified block; no absent pointer is dereferenced or treated as
an invented fallback. 76 disjoint source ranges total 968 control/199296 tile bytes
and do not overlap existing e9 resource ranges. Controls store u16 transfer byte
length, u8 frame count plus retained byte 3, and ordered u16 duration/source-tile
pairs. Source offsets are tiles of 32 bytes. Frame/tick/DMA progress is live RAM,
not extracted state. Tile banks retain full little-endian u32 words and exact
maximum referenced extents across shared aliases. Independent stable IDs follow
first descriptor/slot occurrence; typed links preserve VRAM destinations.

Original creator 08021f10, updater 08021bd8 and dispatcher 08021d10 confirm the
contract in four reduced EUR/Wine animation cases, using retained slot 5 only for
original IWRAM helper initialization. Two loops plus boundary match frame
countdowns/order and complete VRAM after every DMA chunk, including 34 partial
chunks with the original 1024-byte limit: 634 calls/80766 instructions. Five focused
Wine checks pass (3 core/1 JSON/1 private), source/JSON authentication and all 97 links
match the independent static classifier. New Linux/USA checks remain pending.
Managed transfer publication is now complete for EUR/Wine under 1/e10-d9,
with the eighth fixed JSON file and all 76 ranges. Four focused Wine checks and
three actual CLI commands pass authentication/publication/reuse/repair/move/
retention and transactional activation. Direct control/tile source consumption
now passes reduced EUR/Wine provenance/lifecycle and declared 397-frame scene/CLI
proof: all 76 raw spans poisoned in two original/native cases, full states/clocks
equal after 60 calls, 66 compiled control B/9984 native-DMA tile B. The actual pub scene
uses 1203 control B/21504 tile B with every state/image/audio sample equal. New Linux/
USA and broader scene/GUI/archive checks remain pending.
[Contract and evidence](../rom_data_graphics.md#european-background-transfer-programs).


### EUR adjacent lookup and five trade parameter records

ROMDATA-01-GAMEPLAY-ADJACENT-HALFWORD-LOOKUP-ROOTS is partial: 16 literal references
root 6e9cb6, next independent 6e9d8a, but the natural 106 halfword domain is unproved.
Fixed 64/67 and bounded lists 16..20/21,22,4,23 do not establish all UI producers.
No model or ownership is inferred from values or neighbor extent.

ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE96-ROOTS is complete for the declared static
contract: selector D0744 explicitly roots five 96 B records, IDs 2..6, 480 B at
6e85bc..6e879c. Independent next root and biased price source agree. D2728's 30
live RAM rows bound the internal count 0..30, not a 30-record ROM bank. Strict
source model retains 64 opaque B/u16 markup 40-41/30 discount B 42-5f per record;
count 0 aliases markup byte 41. D26AC retains original separate 70 B-source D1F30
property 36/zero skip/live flags/unsigned divide/wrap arithmetic. Windows 73640
warning-free/Wine 52757 passes 499 real+160 edited-source+15 separate injected operand
cases; all 160 price-field B observed/320 opaque B roundtrip-only/full RAM-SP-callee
and strict source/JSON guards pass. e38-d37 is unchanged; managed/runtime/
ordinary and new Linux-USA-native Windows remain pending. [Contract](../rom_data_gameplay.md#trade-parameters-source-contract).


ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-TRADE-PARAMETERS-CONSUMPTION
are complete for declared EUR/Wine: e39-d38/34 files/33 resources/all 32 prior/e38
retained/USA e8-d7. Windows 21694 warning-free; Wine 20164 two checks pass managed/
native/lifecycle/strict source bindings and 28 full paths/raw 480 B altered/full-state-
clock parity, all 480 B/mask 31/field edges/raw gap 222/SWI-DMA/prior owner coexistence.
Compiled opaque/markup-low/shared 41/discount 0-5-10-10, no opaque original price
read inferred. Source 499+160+15 closed; CLI/ordinary/new Linux-USA-native Windows
pending. [Contract](../rom_data_gameplay.md#managed-and-direct-trade-parameters).


ROMDATA-05-GAMEPLAY-TRADE-PARAMETERS-SCENE-CLI is complete for declared EUR/Wine:
production 46740 warning-free/canonical restored; actual CLI 17191 create/reuse-no-write/
old cache/unrelated cwd with 36 managed file hash-mtime guards. First 400-frame existing pre-Yes
fee acceptance locator uses 14 source B (7 markup-low/7 shared byte 41), mask 1 physical ID 2;
rereads included. One Wine 89539 full 400-frame pair matches every state-clock-image-OAM-
dispatch/mixed-PSG-FIFO-HLE audio sample, same source counts. Final Pub list/7109 funds
image inspected. Source 499+160+15/native 28/managed/lifecycle/e34 threshold 400/e38
proofs remain closed. New Linux-USA-native Windows/ordinary discount tail-opaque-
all 5 records/other callers remain pending. [Evidence](../rom_data_gameplay.md#ordinary-trade-parameter-fee-acceptance).


### EUR fourteen packed item effect parameter records

ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE7-ROOTS is complete for the declared static
contract: 11 literal readers/14×7 B/98 B 6e879c..6e87fe, existing item IDs 362..375
subtract 362 and independent next root agree; no e39 overlap. Nine 5-bit
coefficients, 8-bit cap and 3 opaque bits roundtrip and match split-byte getters.
CFE28 property/index low 8 and CFDA8 full u16 item-minus 362 cap index remain distinct;
live current/quantity/cap clamp stays original. UI cap>99 branches and arbitrary
indices remain unverified/raw. Source model/oracle is now complete: Windows 94173
warning-free/Wine 94185 passes 540 real+4 explicit raw-cap aliases+166 synthetic cases,
all 98 B/read-only full RAM/SP-callee/strict JSON-bitwidth-item-owner-source guards.
Managed/runtime/ordinary/new Linux-USA-native Windows pending; e39-d38 unchanged.
[Contract](../rom_data_gameplay.md#item-effect-parameter-static-contract).


ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-ITEM-EFFECT-PARAMETERS-CONSUMPTION
are complete for declared EUR/Wine: e40-d39/35 files/34 resources/all 33 prior/e39
retained/USA e8-d7, actual parsed item owner dependency. Windows 93110 warning-free/
Wine 21177 two checks pass 50 full original-interpreter-native paths/raw 98 B altered/
complete state-clock equality, all 98 B/14 record mask 16383/7 B record-field edges/BIOS/
SWI-DMA/adjacent trade 2-item effect 2 crossing/publication-retention-lifecycle/strict
source guards. Compiled coefficient/shared 05/shared 06 bytes 36/37/23. Source 540+4+
166 not repeated; CLI/ordinary/new Linux-USA-native Windows/full UI remain pending.
Existing DS22 populated bank proof is USA-only, not EUR evidence.
[Contract](../rom_data_gameplay.md#managed-and-direct-item-effect-parameters).


ROMDATA-05-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SCENE-CLI is complete for declared
EUR/Wine: production 4620 warning-free/canonical restored; CLI 59404 create-reuse-no-write
37 managed file hash-mtime guards/50707 old cache from unrelated cwd. New ordinary guest
input from retained EUR IN13 reaches Cyril Bank/List/owned Goblin Lv4/Play. Existing
pre-Yes Cyril state supplies measured 120,216/place 18; earlier wrong-town/guessed
long hold navigation trials are closed, not rerun. One Wine 5779 full 400-frame pair matches
every state-clock-image-OAM-dispatch/mixed-PSG-FIFO-HLE audio sample, positive 3 B
(1 coefficient/1 shared byte 05/1 shared byte 06)/mask 1 record 0. Ordinary mode is mixed native/
interpreted; source/native/earlier ordinary proofs stay closed. Final CLI separate UI
panels absent as previously known, no food-count claim. Other 14 item/20 monster/
cap>99 UI/new Linux-USA-native Windows checks pending.
[Evidence](../rom_data_gameplay.md#ordinary-item-effect-parameters-in-the-european-monster-bank).

### Reclassified palette-effect sources (24 October 2026)

Roots 370eb0/b94690 formerly queued as FFTA audio are not audio resources.
370eb0 is a copied-IWRAM zero-clear/copy/fill helper vector and stays executable
support metadata. b94690 is palette-effect dispatch: state 1/read u16 kind/stride 4,
fixed constructor kinds 1/2/4/8/16, 17 word/68 B projection including 12 null holes.
No live RAM max 16 guard/full table extent is inferred. All 38 e44 owners are disjoint;
static 7 literal/5 producer/opcode/hash checks pass without guest execution.
[Source contract](../rom_data_graphics.md#palette-effect-dispatch-source-contract).
Model/source Wine/publication/runtime/ordinary remain pending; audio migration
of these false leads is closed. Other unselected audio/graphics/families stay open.
Typed 17 slot/68 B palette dispatch model now retains canonical u16 kinds/guest Thumb
code offsets/nulls with strict shape/source authentication. Windows 71666 warning-
free/Wine 22360 one focused check passes 31 original prefixes/all 68 B/12 guards/
low 16 live RAM/raw outside/state gates/predicted producer stores. 1216 protected inputs
unchanged; no full callback/native/state clock/ordinary claim. Managed e44 unchanged;
publication/runtime/ordinary/new Linux-USA-native Windows remain pending.
e45-d44 managed palette dispatch now published/consumed: 40 files/39 resources/all 38
prior/e44 retained/USA e8-d7. Windows 67564 warning-free/Wine 46138 two reduced checks
pass 12 guards/publication-retention-lifecycle/all 68 B/17 slot mask/10 original-native
full-state-clock prefixes/24 callback-4 null compiled B/live kind/raw aliases/BIOS-SWI-
DMA/boundaries. No full callback ABI; actual CLI/ordinary remains pending.
Actual e45 production CLI create/reuse-no-write 42 files/old cache now pass Wine 56076;
neutral boot 240 closed 0 use/no full pair. New normal English-confirm 1000 CLI 60304/
one Wine 35467 full-state-clock-image-OAM-dispatch-audio pair pass 5944 aggregate
callback-byte reads/kind 1 only (mask 2)/null 0. Other callbacks targeted-only; no
RAM/state payload/save edits. Source 31/native 10/old family proofs not repeated.
Palette parameter/program producers and other families/new Linux-USA-native Windows
remain open; no performance/space measurements.

### Palette-program producers: partial inventory (24 October 2026)

Actual palette task 080d7fac/public 080d8214-080d8234 has 308 direct BL candidates;
285 exact argument expressions root 179 programs. 3 fixed-count 8 B list sources add 5,
2 explicit 3 word stack copies add 1; saved-R4 bridge reuses a known root. Combined 185
programs/1000 commands, 16135 candidate program B/124 ranges plus 88 producer B, all 16223
disjoint from 39 e45 owners. Six-byte header/count u8 and 6 B command/u16 relative offset
contracts distinguish direct/queued colors and minimum 1 counted-color sequences.
20 indexed producers now have explicit task-ID/message/constant/counted-loop
projections: 97 cells/388 B add 45 programs, total 230 programs/1135 commands/18221 program B
plus 476 producer B = 18697 B/165 ranges, disjoint from 39 e45 owners. All 185 earlier records
retained; mixed 39bcc4 owns 2 pointers only. These do not imply global RAM guards
or a complete table extent. Source Wine/model/publication/runtime/ordinary pending, e45 unchanged/
no new guest/build/tests or old repeat.
[Contracts](../rom_data_graphics.md#palette-program-producer-inventory).


### Palette programs: source model verified (24 October 2026)

Strict typed 230 programs/1135 commands/25 producer tables reconstruct 165 ranges/
18697 B; canonical IDs/raw fields/list delays-reserved/null slot preserved.
Windows warning-free/one Wine 96989 check passes all 18221 program B and 18681 distinct
all B at original Thumb readers; 16 reserved list B are preserved/unread. 97 indexed
cells/8 list rows/2 stack tables, 6 wrap-clamp copy inputs/3 min-count-period/1 raw direction
probe and 16 shape-JSON-source guards pass. Earlier 1645 artifacts/new test exe unchanged.
This closes the source oracle/model child, not full allocator/callback/native/
state clock/ordinary. e45-d44 unchanged; managed publication/runtime/ordinary next.
[Source model](../rom_data_graphics.md#palette-program-source-model-and-original-reader-validation).


### Palette programs: managed runtime (24 October 2026)

EUR e46-d45 publishes 41 files/40 resources, all 39 prior decoded/e45 retained/USA e8-d7.
165 bindings/18697 B serve typed header/command/parameter/producer fields; no guest
sequencer/index/save/native ABI change. One Wine lifecycle check passes 12 guards/
publication-create-reuse-repair-move/missing file-Load-Undo-backup-Restart/all 18697 B/
230 programs/25 tables. Native-only check passes 24 original-interpreter-native full
state-clock prefixes, generated API [16 header/52 command/66 parameter/12 producer] B,
raw command 65535/color offset 65534/index 2/widths-mirrors-BIOS-SWI-DMA. Two test boundary
failures retained and fixed via prefix guard respecting native chaining/events;
production emitter unchanged/source-managed checks not rerun. Actual CLI create/
reuse-no-write 43 files/old cache pass. English-confirm 1000 new program-use 0/closed
negative/no full pair. 10 protected worldmap-bank snapshots have no new palette task.
Actual ordinary EUR Montblanc Richiamo/Pecorelle CLI and one 1000-frame Wine
full-state-clock-classic 240x160-image-OAM-dispatch-audio pair pass 170 source B/
4 programs (32 header/90 command/48 parameter), 0 producer. Generic typed bus-or-host
consumption guard corrected; selected palette assertion unchanged. Other 226
programs/producer tables retain targeted-only coverage. New Linux-USA-native Windows
and new 32:9 scene checks remain open.
[Consumption](../rom_data_graphics.md#managed-palette-programs-and-typed-guest-reads).


### Blend programs: bounded producer inventory (24 October 2026)

The neighboring 080d77c4 task/080d79ac-080d79d0 constructors control BLDCNT,
BLDALPHA and BLDY through the compact PPU shadow; they are not e46 palette roots.
All 248 retained direct calls are classified: 235 constant/2 conditional/6 indexed/
1 inline/1 mutable RAM template/3 dynamic RAM. Current CFG-block validation rejects
the false 080e2ec8 code root; that caller copies 10 B from 0839a5f0 then overrides its
RAM duration. Six bounded projections supply 22 pointer cells; no global RAM guard
or complete physical table extent is inferred. Inline 0/1 variants follow explicit
message-ID branches; primary and secondary descriptor constructors are distinct.

160 programs/345 commands own 2340 B (960 header/1380 command), plus 88 producer B:
2428 B/158 ranges, disjoint from all 40 e46 owners/no repeated logical byte. Header 6 B is
u16 control/coefficients/count, command 4 B is u16 duration/u8 first/u8 second. Static
independent reconstruction/all hashes/44 consumer opcodes/DMA register mapping
pass. 135 alpha/20 brighten/5 darken; 5 instant ffff commands. Typed model/source Wine/
publication/runtime/ordinary remain pending; no new guest run/build/test or
performance campaign. EUR e46-d45/USA e8-d7 remain. [Source contract](../rom_data_graphics.md#blend-program-producer-inventory).


### Blend programs: typed source model (24 October 2026)

Strict lossless `rom_data::blend_programs` reconstructs all 160 programs/345 commands/
6 tables/22 pointers/2428 B/158 ranges, preserving packed reserved fields and raw
u16 duration/u8 targets/canonical references. Exact EUR identity/110 reader-producer
fingerprints, strict JSON/order/count/bitwidth/reference/overlap/source guards.
Windows 12341 warning-free; one Wine 94935 source check passes 160 headers/345 commands/
22 cells/5 instant chains/template copy-RAM override/2 packed header/6 duration/3 delay/
index 65535 raw prefixes. 2376 source B observed; 52 nonalpha timed second fields remain
preserved/hash checked only for these initial-mode probes. 17 shape/3 source-edit/
2 JSON/2 ROM identity guards pass; 52943 instructions/264 PCs are coverage, no timing.
Managed/runtime/native/full clock-ABI/CLI/ordinary/new Linux-USA-native Windows stay
pending; e46-d45 unchanged. [Model and scope](../rom_data_graphics.md#typed-blend-program-source-model).


### Blend programs: managed runtime (24 October 2026)

EUR e47-d46 publishes 42 files/41 resources/all 40 prior decoded/e46 retained/USA e8-d7.
158 bindings/2428 B serve typed header/command/producer fields on 13 pages; no guest
sequencer/index/save/native ABI change. Reduced Wine checks pass 12 guards/
publication-retention-lifecycle/all 2428 B/160 programs/6 tables and 20 original-native
complete-state/clock prefixes. Compiled reads [34 header/70 command/16 producer] B;
source modes/wait-instant-delay/template copy/live 2→1 mode/raw command 65535/index 3/
previous palette index 5/width-mirror-BIOS-SWI-DMA/new 2 raw 2/new 2 old 2 pass.
1034 instructions/295 PCs/93 dispatches are coverage only. Initial compile argument/
import fixes retained; corrected source-set coherence separately passes a new
focused Wine negative guard, without closed native/lifecycle repeats. No whole
callback/allocator/scheduler ABI. Actual CLI create/reuse-no-write 44 managed files/
old cache and one 1000-frame Wine ordinary EUR Pecorelle full-state-clock-classic 240x160-
image-OAM-dispatch-audio pair now pass 18 aggregate B (10 header/8 command), 1 program,
0 producer; other 159 programs/tables targeted-only. New Linux-USA-native Windows/
new 32:9 scene checks remain pending. [Consumption](../rom_data_graphics.md#managed-blend-programs-and-typed-guest-reads).

## Bounded European task descriptor sources

Three adjacent catalogs contain 341 primary/20 auxiliary/51 script records at 3997b0/
39a258/39a2f8: 412 records/3296 B/3 spans, disjoint from 41 e47 owners. Original constructors
080d7018/080d7328 use different ID formulas; nullable Thumb callbacks/context size/
raw metadata are distinct. Independent graphics source 39a490 and directory kind 0/
ID 413 reference at 3aca2c establish the suffix endpoint 39a488; no global live ID cap
or whole callback ABI. 53 BL candidates retained; unresolved CFGs are not proof of
absence. Static independent roundtrip/hash/59 halfword/12 word contract checks pass.
Strict typed model now passes one reduced Wine source oracle: all 3296 B reconstructed/
3156 B observed, 140 suffix metadata B preserved-unread, 415 main/59 secondary/346 halfword/
346 byte/18 low16/9 branch prefixes and 23 shape-source-JSON-identity guards. Warning-free
Windows build; 7191 instructions/39 PCs coverage only. e48-d47 managed runtime now passes
13 guards/lifecycle/all 3296 typed B/412 records/3 catalogs and 29 original/native full-state-
clock prefixes, generated API 44 callback/22 context/9 metadata B, raw/aliases/old blend
owner/boundaries-BIOS-SWI-DMA. 43 files/42 resources/all 41 prior decoded/e47 retained;
USA e8-d7 unchanged. Actual production CLI create/reuse-no-write 45 files/old cache and one
1000-frame Wine full-state-clock-classic-image-OAM-dispatch-all-audio pair now pass 28 aggregate
source B/4 records/2 catalogs (16 callback/8 context/4 metadata). Other 408 records/third catalog
targeted-only/new 32:9 pending. Source/model/runtime/CLI/ordinary children closed;
no whole allocator/callback/scheduler ABI or new Linux/USA claim. [Contract and scope](../rom_data_tasks.md). Linux/USA remain pending; no new
performance/space measurements.

## European effect/event producer directories

The bounded source contract covers 75 event 16 B rows/1200 B, 51 selector 4 B rows/204 B and
22+6 even Thumb same-state dispatch words/112 B: 1516 B/four spans disjoint from 42 e48 owners.
Original increment/compare 74 and independent 6 B next catalog 3acebc bound events; separate
u3 lookup 3ad240 bounds selectors. No count from shape or 51 task descriptors. 19 event+
51 selector logical links resolve to engine.task-descriptors. Original relative
jumps/u16 index/q0 underflow/secondary ID wrap/default branches stay live. Source-model
strict JSON/92 halfword/8 word guards/independent reconstruction and one Wine oracle pass:
77 header/77 delay/53 selector/43 param/32 dispatch/19 frontdoor-low16-increment prefixes,
1106 B observed/300 event raw/110 selector param B preserved-unread, 27 guards. 2582 instructions/
70 PC coverage only; no full interpreter/native/state clock/ordinary/reachability claim.
Managed e49-d48 integration now passes 13 guards/lifecycle/all 1516 typed B and 29 original-
interpreter-native full-state-clock prefixes/[24 event/6 selector/24 dispatch] API B,
raw/defaults/even state/previous font 11 B/same-page palette 4 B/BIOS-SWI-DMA. 44 files/43 resources
retain all 42 prior decoded/e48; real task descriptor dependency. Actual CLI create/
reuse-no-write 46 managed files/old cache passes. Pecorelle 1000/school cold boot 3000/suspend boot-to-battle 8476
locators read zero new bytes: closed negative/no event-required full pairs. New normal
EUR boot/name/snowball/street/bedroom progression yields 19 genuine snapshots;
one 1800-frame book locator/full pair from 39420 on actual e52 generation passes
all full-state-clock-image-OAM-dispatch-audio with 5 aggregate B = 1 selector+4 dispatch,
one selector/one table, 470 hash-mtime guards. Ordinary event rows 0/other 50 selectors/
other dispatch table targeted-only, entry 124e1c reachability unknown.
USA e8-d7 unchanged/Linux-USA-native Windows/new 32:9 pending/no performance.
[Contract and limits](../rom_data_events.md#positive-ordinary-book-transition).

## European script command directory

The next independent EUR source projection contains 115 six-byte commands,
690 bytes at 3acebc..3ad16e. Three original literal roots and byte-assembled
callback/advance readers are verified. Two independent byte-lookup roots at
1263c4/126468 bound the end; an initial 116-row candidate ending at 3ad174 was
rejected before the source plan/model because it included that distinct lookup.
All 115 callbacks are unique non-null Thumb BX entries; no asset dependency.
The projection is disjoint from all 43 e49 owners. Original u8 opcode arithmetic
has no new cap, and signed stream cursors/u16 advance wrap/relative targets/
return 0x110 loop remain live. Strict model/hash roundtrip/106 halfword and six
word fingerprints and one corrected Wine oracle pass all 690 observed bytes,
117 callback/117 advance/115 BX/four signed-cursor/ten relative-target/five loop
prefixes and 19 guards. First Wine failure was an ADDS expected-register mistake
in the test, corrected without a production/model change. 2905 instructions/
67 PCs are coverage only. Managed e50-d49 now publishes 45 files/44 resources/all 43
prior decoded/e49 retained/USA e8-d7 unchanged. Two reduced Wine checks pass 13 guards/
lifecycle/all 690 B and 26 original-interpreter-native full-state-clock prefixes with
14 advance/28 callback API B. Raw 115/255/signed cursor/wrap/relative-targets/BL-BX/
return loops/adjacent previous owners 4+4 B/mixed previous 2 new 2/new 2 raw 2/BIOS-SWI-DMA
agree. Actual CLI create/reuse-no-write 47 managed files/old cache and one 8476-frame Wine
EUR suspend boot-to-battle full-state-clock-image-OAM-dispatch-audio pair pass
42 B = 14 advance/28 callback/7 commands. Other 108 commands targeted-only; new 32:9/Linux-
USA-native Windows pending.
[Contract, evidence and limits](../rom_data_scripts.md).

## European relative script archive and event keys

Bounded static inventory identifies 115464 B/two disjoint ranges: 114528 B archive at
B946EC..BB064C and 936 B/234 keys at D68C2C..D68FD4. Independent localized-text and
coordinate-index roots bound both ends. Key domains/first relative offsets agree
with 62 top banks/233 nested slots (248+932 index B). 213 unique targets/20 repeated links
include 5 header alias links and 2 FF fill links. 207 nonempty physical command segments
contain 35048 storage instructions/110 opcodes/78212 parameter B; 45 FF fill spans/88 B remain
readable. 493 command 5a links target 49 existing effect selectors; actual dependencies
are engine.script-command-directory and engine.effect-event-directories. Key byte 2
selects one of five unmigrated localized text archives; byte 3 raw is preserved.
Independent reconstruction/234 keys/233 nonnull target formulas/128 halfword/5 word fingerprints/
all 115464 B/disjoint 44 owners pass without guest execution. First builder's empty-
program assertion exposed two fill targets; corrected before plan written. Not linear
CFG/whole argument semantics/reachability/new guest caps. Later model/runtime
results below cover separate criteria; inventory left e50-d49/USA e8-d7 unchanged. [Contract](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract).

The whole 115464 B archive/key model now has strict lossless Program/Header/Fill
references and complete physical reconstruction, no ROM shadow/alias copies. Actual
command/effect dependencies validate lengths and 493 selector links. Windows 91283
warning-free/one Wine 85436 first-pass verifies 238 key address/234 two-level/234 text bank/
35054 opcode/2397 relative-target/493 selector param prefixes, 42333 owned B observed,
73131 B preserved-unread and 40 guards. 224382 instructions/62 PC coverage only; not whole
interpreter/native/full-state-clock/ordinary. Model leaves e50-d49/USA e8-d7 unchanged,
source-model scope only; managed/runtime results follow. [Model evidence](../rom_data_scripts.md#typed-relative-archive-model-and-reduced-wine-source-oracle).


Managed e51-d50 adds script-archive.json/engine.script-archive, 46 files/45 resources,
all 44 prior decoded files/e50 retained, USA e8-d7 unchanged. Real command/effect
asset dependencies and two typed source bindings/115464 bytes authenticate before
activation. Two reduced Wine checks pass 13 guards/lifecycle/all bytes and 28 original/
interpreter/native complete-state/clock prefixes. Compiled API categories read
37 index/4 opcode/7 parameter/12 key/1 fill bytes, preserving raw IDs/unaligned rotation/
header-fill views/previous palette owner/boundaries/BIOS-SWI-DMA. Production CLI
create/reuse-no-write 48 managed files/old cache passes; retained 8476-frame EUR
locator reads 27 B = 8 index/7 opcode/10 parameter/2 key, one bank/program/key, no fill.
One Wine 8476-frame pair from the actual CLI generation passes all full states/
clocks/central-wide images/OAM/dispatch/mixed-PSG-FIFO-bitwise-HLE audio, same
positive archive use and 366 hash-mtime guards. Other 61 banks/206 programs/233 keys/all 45 fills
retain targeted coverage. No new 32:9/whole ABI claim; new Linux-USA-native Windows
pending. [Runtime evidence](../rom_data_scripts.md#managed-relative-archive-and-typed-guest-reads).


## European script-selected localized text

Five language roots BB064C/C076F0/C5FBF0/CB7C48/D10644, selected by 08009ab0,
independently bound 1803744 B through the next root/D68C2C script-key root. 63 top
entries per language agree with key-bank maximum 62/minimum offset FC; first word
is a shared late bank 0, not a length. Eleven repeated normal links per language
and 26 root-relative bank 61 variants resolve to 77 physical banks per language.
Live byte division by 10 selects the variant; a distinct remainder helper adds
24 times the remainder to the u16 message. Full-u16 and low-byte special-message
branches, raw key IDs, fallback language/key and live state remain original.

385 banks/41395 u16 index entries including 385 sentinels name 41010 programs. Existing
Program/custom LZ grammar gives 9813 compressed extended/5709 raw extended/25488 raw
compact programs, 212011 commands/11 forms, 147 glyph IDs 1..3112 already in the font
asset; actual dependencies script-archive/font.complete-bank. The whole source
partition is 1260 top+520 variant+82790 bank-index+1684407 program+34767 alignment B.
34314 gaps are 1..3 B; four mixed gaps reject a uniform-fill assumption before any
model/runtime/Wine. Exact bytes remain explicit, without inferred semantics.

Independent static whole-source reconstruction/hash/15195 key-variant formulas/
349 halfword/13 word guards/actual dependencies/glyphs/disjoint 45 e51 owners pass.
No guest execution/build/test or prior proof repeated. Typed model/Wine/runtime/
CLI/ordinary pending; EUR e51-d50/USA e8-d7 unchanged. [Contract and evidence](../rom_data_text.md#script-selected-localized-text-source-contract).


The strict whole-source script_text model now shares physical banks through
typed top/variant references and reconstructs all relative/u16 index words,
Program codec bytes, sentinels and exact slack without alias payload copies.
Actual key/font dependencies validate. First Wine strict-JSON failure exposed
unit-variant extra-field acceptance; corrected to empty struct, kept assertion
and all first-attempt evidence. Corrected Windows 55123 warning-free/single new
Wine 34745 passes all 1803744 B/independent fields/hashes and 22 shape/6 source-edit/
3 dependency/12 JSON-shape/2 identity guards. Original 7 language/1170 key-bank/
1280 live quotient/256 live remainder/90 raw-special/1170 u16 index/837 header/
837 source-vs-typed output cases pass, including 249 compressed bodies/tails.
2116900 instructions/834 PC coverage only; not native full-state-clock/ordinary/whole
ABI. Source model closed, publication/runtime/CLI/ordinary remain pending;
e51-d50/USA e8-d7 unchanged, new Linux/USA/native Windows pending. [Model evidence](../rom_data_text.md#typed-script-selected-text-model-and-reduced-wine-source-oracle).


Managed e52-d51 now publishes script-text.json/text.script-archives, 47 files/
46 resources/all 45 prior decoded/e51 retained/USA e8-d7, with actual key/font deps.
Five typed bindings 1803744 B reconstruct all index/header/control/decoded program/
slack fields and merge previous owners/raw bytes on BB0/D68. Shared literal map
keeps old categories; one affected core Wine check/new managed-native filters
pass 13 guards/lifecycle/all bytes/all masks and 35 original/interpreter/compiled
full-state-clock prefixes, including raw selectors/wrap/sentinels/width-mirror-
real BIOS-CpuSet-FastSet-DMA. Direct tail literals 0 is exact, generated tails remain
preserved. Real old 0/1/2 source 163 B also pass. 379 instructions/65 PCs/71 dispatches
coverage only. Production CLI create/reuse-no-write 49 managed files/old cache passes;
8476 battle locator is negative 0 text/no full pair, distinct school 3000 positive
reads 289 B/one language/bank/two programs. One Wine 3000 pair from CLI generation passes every full state/clock/image/OAM/
dispatch/mixed-PSG-FIFO-bitwise-HLE sample, same source use/468 guards;
other records/variants/slack targeted-only, new Linux-USA-native Windows pending.
[Runtime/CLI evidence](../rom_data_text.md#script-text-production-cli-and-ordinary-school-dialog).

## European script coordinate paths

Independent reader128914/literal128994and successor readerA1F8/literalA234 bound
D68FD4..D69BC4:3056B, a self-delimiting840B/210-slot relative directory,210u16
counts420B,856XYpairs1712B and42FFalignment spans84B. All physical storage is
covered once; these are no global live index/count/coordinate caps. Original
low16index/low8destination/buffer+80count-low8return and live map-height query
1DD88remain distinct.68primary-script links reach56paths:66opcode44low8/ignored
byte3 and2opcode45u16. Actual archive/command dependencies are checked; source
is disjoint from all46e52owners and shares only theD68page with earlier owners.

The strict typed model reconstructs/hashes all3056B. One warning-free Windows
build and one Wine source oracle pass24guards/allindependent fields/194halfword/
5word fingerprints,212original path copies (all210lists/two changed live map
cells),13raw-index/destination and13producer prefixes.2972sourceB observed;
84alignment Bpreserved-unread.40116instructions/134PCs coverage only, no native/
fullstate-clock/wholemovement-task/allocator/ordinary claim. Model source stage
closed; managed publication/direct runtime/CLI/ordinary remain pending, e52-d51/
USAe8-d7 unchanged. No new fixture/navigation/Linux-USA-nativeWindows/performance
check. [Source/model contract](../rom_data_scripts.md#european-script-coordinate-paths-source-contract).

Managed e53-d52now publishes48files47resources/all46prior decoded/e52retained,
USAe8-d7. Actual archive/command links authenticate before one shared typed
index/count/XY/alignment owner activates; D68merges text/key/coordinate fields,
rawD69BC4outside retained, guest height/control/ABI unchanged. Reduced Wine
managed/native checks pass14guards/lifecycle/all3056B/210paths856points42alignment
and24original-interpreter-compiled fullstate-clock cases/four full copies/live
height/allfour classes[28,85,76,2]B/mixed owners/edges-BIOS-SWI-DMA. New-test array/
observer errors corrected with evidence retained; successful checks reused.
ProductionCLIcreate/reuse-no-write50files/oldcache/all48generation equality and
one6000-frame genuine EUR ordinary pair pass every fullstate-clock-image-OAM-
dispatch-audio with28aggregateB=4index14count10XY/one path,0alignment;581guards.
Other209paths/42alignment targeted-only; wholemovement/allocatorABI/new32:9/
Linux-USA-nativeWindows unverified, no performance measurement. [Runtime evidence](../rom_data_scripts.md#coordinate-path-production-cli-and-ordinary-scene).


## European auxiliary script archive source projection

Independent readerA1F8/rootD69BC4 uses u8action/slot and two levels of relative
u16 offsets, separate from the primary u32 archive. The declared4098B projection
throughD6ABC6 has8banks/442slots/126distinct targets/30interior entries,
133physical programs/913instructions/2275parameterB/900indexB and5internalFFgaps
10B. Root walks alone miss69physical instructions. All489relative branch links
resolve to instruction boundaries; structural908/913coverage is not ordinary
reachability, and the five remaining instructions stay preserved.

Separate17x6Bcommand directory370F94..370FFA supplies advanceu16/Thumbcallbacks;
its successor370FFC is independently consumed. Its two zero-gapB and the last
archiveFF2B remain unowned. GraphicsbaseD6B4C8 is a ceiling, not archive end;
2306B afterD6ABC6 are excluded. Reassembly/hash/all4200B/723halfword4word/static
link formulas/all47e53owner disjointness pass. No build/guest test was run for
source inventory. Strictmodel/oneWineoracle are verified below; managed/runtime/
CLI/positiveordinary remain pending; no guest action-slot-opcode caps, primary-archive dependency or
wholeABI claim. e53-d52/USAe8-d7 unchanged, newLinux-USA-nativeWindows pending.
[Contract](../rom_data_scripts.md#european-auxiliary-script-archive-source-contract).


The strict auxiliary directory/archive model now reconstructs all4200B without
a ROMshadow; typed program+instruction entries preserve30interior links and133
physical programs. All nested JSON/source identities/own-callback+advance
 dependencies are checked. Warning-free Windows builds and the final new Wine
source filter pass40guards/442lookups/913dispatch+oneRAMopcode3/80advance/
489relative suffixes/10wrap/15raw/11wrapper probes.2887distinct canonicalB are
observed;1297other params+10gaps+6zero-rowB remain preserved/hash-checked unread.
First raw expectation fails on odd LDRH; corrected test-only observer, then a
specific32-bit high-rotation case added. First failure/earlier narrower passed
retry retained, unchanged model/core reused; only frontend/filter affected.
31839instructions177PCs coverageonly; no wholepredicates/interpreter/native/
fullstate-clock/ordinary claim. e53-d52/USAe8-d7 unchanged. [Model evidence](../rom_data_scripts.md#typed-auxiliary-archive-and-reduced-wine-source-oracle).


Managed EUR e54-d53now publishes both auxiliary directory/archive resources:
50files49resources/all47prior decoded/e53retained/USAe8-d7. One shared immutable
owner serves4200B after actualarchive→owndirectory authentication, with typed
index/callback/advance/opcode/parameter/gaps and no ROMshadow/hostinterpreter/
guestcap/save-nativeABI change. Shared370audio/D69coordinates and raw370FFA+/
D6ABC6+ remain correct. One warning-free affected Windows build/two exact new
Wine filters firstPASS26guards/lifecycle/all4200B and44original-interpreter-
compiled complete-state-clock prefixes/all6classes[6,65,18,19,15,2]B/unaligned
u32LDRH/interiorentries/signedcursor/rawopcodes/widths-mirrors-BIOS-SWI-DMA.
624instructions92PC52dispatchescoverageonly; whole predicates/interpreter/
callback ABI/ordinary unclaimed for that targeted scope. ActualproductionCLI
create/reuse-no-write52managedfiles/oldcache/all50file equality and one6000frame
positive ordinary Wine fullstate-clock-image-OAM-dispatch-audio pair nowpass:
96directoryB=32advance64callback/6commands,82archiveB=16index20opcode46parameter/
2banks2programs,0alignment/640guards. Other11commands6banks131programs/5alignment
spans targeted-only, newLinux-USA-nativeWindows/32:9pending without performance/
space work. No newfixture/navigation or oldproof repeat.
[Runtime and ordinary](../rom_data_scripts.md#auxiliary-script-production-cli-and-positive-ordinary-scene).


## European conditional normal script-special pose model

The151low-selector script references/20logical metadata14producer witnesses
retain108physical keys/12roots/32sequences241frames/11OBJ64objects/95pixel requests.
Strict lossless model reconstructs41802B and independently validates overlapping
initial/normal shared views before subtracting their union; only33spans6436B
have new ownership. One warning-free affected Windows frontend test build and
single reduced new Wine source oracle pass18guards/120constructor prefixes/
53null7proper resets/17scheduler-realDMA3 uploads/all5248newpixelB/wholeVRAM.
No oldcommands2/6/8or source/scene/census oracle repeated. Conditional source/
model closed; managed/runtime/CLI/ordinary and alternative/override/highregistry/
current-state coverage pending. e61/USAe8unchanged; newLinux-USA-nativeWindows-
32:9pending, no performance/storage measurements.
[Model and scope](../rom_data_unit_obj.md#typed-normal-script-special-pose-model).


The conditional normal script-special owner now publishes/consumes EUR e62-d61:
58files57resources/fouractual metadata14-scriptarchive-initial-normal dependencies,
all56prior decoded files/resources/coverage and e61 retained, USAe8 unchanged.
One warning-free Windows core/frontend build(core compiledonly) and two new Wine
filters firstPASS29guards/lifecycle/all6436newtypedB/seven sequences and23original-
interpreter-compiled fullstate-clock paths/33edges-mirrors-BIOS-SWI-DMA. Shared
initial/normal owners stay intact under41802graphB poisoning; new-role bytes
0dir240index28header920frame0OBJ5248pixel, nativeAPI0/36/6/43/0/1. Mixed2new4normal
transfers and2new2rawAPI preserve exact charges. No oldcommands/source/census/
scene repeat. ActualproductionCLI58generation60managed/all58matchaccepted/no-write
reuse and oneframeoldcacheattachment pass; zero newuse/smokeonly. Ordinarysource/
newpixel and newLinux-USA-nativeWindows-32:9pending, without measurements. [Managed runtime](../rom_data_unit_obj.md#conditional-script-special-normal-managed-runtime).


New actuale62ordinary locators distinguish typednull-source use from pixels:
Snow6000reads48newindexB/zero sequences-pixels; retainedRIVERstart618reads0newB.
No fullpair/guardbuild/oldcensus repeat. Newseven-sequence static81snapshot scan
finds no matching task; program159contains a producer but actualtrigger remains
unproven. Ordinarynewsequence/pixel/equivalence staysopen, original/cart inputs
preserved and no performance/otherplatform work. [Evidence limits](../rom_data_unit_obj.md#script-special-ordinary-source-locators-and-remaining-pixel-gate).


The alternate script-special branch now has a separate source projection over the
same 151 references/20 low-selector witnesses, metadata14 half 06 and original
current alternate assignment with override 20 clear. 84 cells/68 null/14 sequences
64 frames, 9048B total; new 20 spans 816B = 216 index 8 header 80 frame 512 pixel,
remaining 8142 alternate + 38 initial + 52 normal. Offline model/warning-free Windows
build/single Wine filter pass 19 guards, 108 constructor prefixes/52 null + 2 proper
resets/two real DMA uploads and 512 pixel B/full VRAM exact; initial sandbox Wine
startup before test retained, same binary retry PASS. e62-d61 runtime unchanged.
Ordinary new pixels/high16/override/current state remain explicit.
[Contract](../rom_data_unit_obj.md#conditional-alternate-script-special-source-and-typed-model).


EUR e63-d62 publishes/consumes the alternate script-special model: 59 files
58 resources, actual metadata14/script archive/initial/normal/alternate, 57 previous
data/records/coverage unchanged/e62 retained/USA e8. New owner serves only 20 spans
816B; 31 guards/lifecycle/all 816 typed B and 24 Wine original-interpreter-compiled
full state/clock paths/all 20 boundaries/BIOS-SWI-DMA/mixed owners/raw pass. Real
CLI/ordinary/high16/override/current actor and new Linux/USA/native Windows/32:9
remain to verify, no performance/space measurements/new fixture.
[Contract and evidence](../rom_data_unit_obj.md#conditional-script-special-alternate-managed-runtime).


EUR e63-d62 production CLI now verified on Wine: create/reuse 59 generation 61 managed,
all 59 match accepted/no-write, canonical exactly restored/old-cache one-frame PASS.
Zero new alternate-script use in the single frame, no ordinary/pixel/full-scene proof.
Source/model/managed/native/CLI closed; ordinary/high16/override/current actor and
new Linux/USA/native Windows/32:9 remain open. No performance measurement.
[CLI and limits](../rom_data_unit_obj.md#conditional-script-special-alternate-actual-production-cli).


Targeted read of 81 EUR snapshots for the two new alternate-script sequences
188/32-33: no matching enabled/dormant task, inputs unchanged. Lead closed negative,
no retry/guest/full pair; ordinary new pixels/equivalence still open while independent
override/high-registry source contracts continue.


Script override 42/230 has a separate source contract: 167 script refs/13 groups
with original forced current assignment/flag 20 set/facing 0..3, 26 mode-base
witnesses/156 cells/11722B. Shared 6252 normal + 4420 alternate; new 40 spans
1050B = 384 index 24 header 360 frame 26 OBJ 256 pixel, six sequences 18 frames
including two new op 2 argument copies. Offline model/warning-free Windows build/
single new Wine filter first PASS 19 guards/192 constructor aliases/90 null + 6
proper resets/two op 2 argument copies/two DMA uploads 256 pixel B/full VRAM.
e63/USA e8/runtime unchanged; no invented static high-registry identity.
[Contract and limits](../rom_data_unit_obj.md#conditional-forced42-and230script-pose-source-model).


EUR e64-d63 now publishes/consumes script-override 42/230: 60 files 59 resources,
three deps script archive/normal battle/alternate battle/all 58 previous/e63/USA e8
unchanged. Owner 40 spans 1050B; 31 guards/lifecycle/all typed B and 27 Wine
original-interpreter-compiled full state/clock paths/both new op 2/new OBJ/40 edges/
BIOS-SWI-DMA/mixed owners/raw pass. Production CLI/ordinary/high registry-current
actor/new Linux-USA-native Windows/32:9 remain pending, no performance/space
measurement or new fixture.
[Contract and evidence](../rom_data_unit_obj.md#forced-script-override-managed-runtime).


EUR e64-d63 production CLI now verified on Wine: create/reuse 60 generation
62 managed, all 60 match accepted/no-write/canonical exactly restored/one-frame old
cache PASS. Zero new override use in smoke, no ordinary pixel/full-scene claim;
previous e63 and USA e8 retained. Ordinary/high registry-current actor continue,
without performance measurements.
[CLI and limits](../rom_data_unit_obj.md#forced-script-override-actual-production-cli).


Exact six-target override ordinary lead: 81 authenticated EUR snapshots, no enabled/
dormant task matching 42/32-33-40-41 or 230/32-33, inputs unchanged. Lead closed
negative, no guest/build/retry/full pair; ordinary new pixels/equivalence still open,
high registry/current actor sources remain independent.


High-registry known-special script 49 has a new conditional source contract over
the 16 actual references/seven pairs and 93 logical metadata14 unit rows with
original current assignment/override 20 clear. 1302 witnesses/2220 logical keys
1908 cells/373842B; 358566B shared from 6 owners, new 272 spans 15276B (2392 index
84 header 1920 frame 10880 pixel), 21 sequences 96 op 1 frames/37 pixel requests.
All 59 e64 owners and 41 halfword 4 literal 8 BL pins pass, no target root/mixed
item/conflict/variable extent. Lossless model and first targeted Wine test pass
26 guards/1196 constructor aliases/567 null + 21 resets/37 DMA 10880B/full VRAM,
six previous views independently validated/consistent physical aliases. Corrected
warning-free frontend build; EUR e65-d64 publishes 61 files 60 resources with
metadata14/archive and 6 actual previous views. Two targeted Wine filters first PASS
33 attach + 10 dependency guards/lifecycle/all 15276 typed B 21 sequences and
25 original-interpreter-compiled full state-clock paths/272 edges/BIOS-SWI-DMA/
mixed old-raw-new. Real Wine CLI prepare/reuse 61 generation 63 managed/all 61
accepted/no-write/one-frame old cache attach first PASS (all new counters 0/smoke
only); ordinary new pixels/equivalence pending, USA e8 unchanged. New readonly
locator 81 snapshots/21 new sequences finds 0 active or dormant tasks; lead closed/
no guest-build-fixture-full pair or retry unchanged. Registry identity/generic job/
out of catalog/redirect/current changed remain separate. No measurement or new fixture.
[Contract](../rom_data_unit_obj.md#registry-selectors-and-known-special-unit-source-projection).


High-registry generic unit 0/1 has a separate contract for 16 high refs/seven pairs
and metadata52 route 0 with concrete field per mode. CCC48 reads job+5/alternate+7,
properties 4-5 skip RAM property 0 override; CF getters zero-extend u16. All 116 rows
classified: 100 both concrete/10 normal only/2 both FFFF/4 route FF, 110 normal
100 alternate → 2940 witnesses 6720 refs; initial 110 joint assumption corrected
before targets. 83 halfword 6 literal-dispatch 11 BL pins and all 60 e65 owners
authenticated before targets. 107 sprites 535 pairs 3210 keys/cells 2420 null/
777 proper sequences 5093 frames/45 OBJ lists 231 objects/1895 pixel requests 677280B;
12 initial index-root targets 48..59 with exact-field witnesses. Full 796992B/shared
697168B, new 659 spans 99824B (5544 index 504 header 11760 frame 82016 pixel),
1386 new cells 1260 null/126 new sequences 588 op 1/268 pixel requests. No new OBJ/
mixed item/role/extent conflict. Lossless model and first targeted Wine pass 34 guards/
2772 new-cell aliases/1260 null 126 proper resets/268 DMA 82016B/full VRAM/12 initial
root views; six previous views independently validated. First warning-free Windows
frontend build; EUR e66-d65 publishes 62 files 61 resources/metadata52-archive +
6 previous views. First two targeted Wine filters pass 36 attach + 10 dependency
guards/lifecycle/all 99824 typed B/126 new sequences and 25 original-interpreter-
compiled full state-clock paths/659 edges/BIOS-SWI-DMA/mixed old-raw-new. Real Wine
CLI create/reuse 62 generation 64 managed/all 62 accepted/no-write/one-frame old
cache attach first PASS (all new counters 0/smoke only); ordinary new pixels/
equivalence pending/USA e8 unchanged. New readonly locator 81 snapshots/126 new
sequences finds 0 active/dormant body tasks without current-pointer/end-of-frame
filter; lead closed/no guest-build-fixture-full pair or retry unchanged. FFFF/
redirect/out of catalog/changed actor/global identity and other platforms separate;
no old proof replay/fixture/navigation/measurements.
[Contract](../rom_data_unit_obj.md#registry-selectors-and-generic-direct-job-source-projection).

The high-registry contract also covers current jobs 80/81/82/85 route FF with known
alternate job route 0/concrete field: 110 normal 100 alternate → 11760 witnesses
26880 refs. The primary source is FF routing, sprite field comes from alternate row;
98 halfword 6 literal 11 BL pins verify CC35A/unchanged r4/return to same property.
All 116 rows and 928 actual classifications retained (840 concrete 56 FFFF 32 loop).
All 61 current resources/sources authenticated before targets: each of 2940 direct
witnesses has four preimages, same 3210 cells/777 sequences/12 root views/full
796992B/seven owners. Zero new bytes or resources; e66-d65 unchanged, metadata52/e66
Wine proofs reused without build/guest/fixture/navigation. Ordinary/global identity/
FFFF/nonreturning/out of catalog/changed actor and Linux/USA/native Windows/new 32:9
remain open.
[Redirect and limits](../rom_data_unit_obj.md#registry-generic-jobs-redirected-to-a-known-alternate-job).

Known FFFF results now have a separate reader classification: 196 direct + 784
redirected witnesses/2240 refs, 71 reader pins/three code blocks/two literals.
Property read 083B4C12 returns DC86, directory 083D4830 returns 14AE3F1E;
60 queries/30 words remain in unmapped region 14. CPU aligns/rotates 16, MemMap
uses livebios_open: no ROM sequence/constant null or universal nonreturn proven.
The two ROM reads total 6B without typed owner; raw fallback retained, 6B migration/
ordinary reachability still open. No invented ID cap/pointer fix/fake counted asset
or build/guest/fixture/navigation/measurements, e66/USA e8 unchanged.
[Reads and limits](../rom_data_unit_obj.md#known-generic-job-ffff-results-and-raw-bus-reads).

Actor property reader 221C4 now has a distinct source contract: 394C14+2*sprite,
u16→9AD0E/r8→constructor r3→allocation request/task+16. 398 normal/alternate
job-special-override witnesses derive 182 queries before targets; 98 halfword 15 BL/
four code blocks/one literal and 61 owners/files/sources pass. Four new disjoint
ranges 364B, SHA ad328c8ebccfc43091dc878c06dab33b22521d7b617d4b11232e4a532ffc2208;
139 words 16/43 words 24, all u16 bits retained, raw gaps. No inferred catalog 248/
ID cap, flag bitfield or VRAM unit. Source child and lossless model/Wine oracle now
closed: 20 guards/all 182 getters 364B/3 raw fullword/four new-task prefixes/two
reuse/four u16 setters/whole RAM guards pass. Existing tasks retain their own +16.
First warning-free Windows frontend build, 82 build 83 Wine guards 11 compiled inputs
unchanged; no full allocator/native-clock/ordinary claim for 02. EUR e67-d66 now
publishes 63 files 62 resources/metadata52-14 dependencies/all 61 previous-e66 retained.
First two Wine filters pass 17 attach + 4 full-set guards/lifecycle/364 typed B/
182 mask records and 16 original-interpreter-compiled full state-clock paths with
364 raw B altered. Four edges/width-mirrors/real BIOS bus/SWI-DMA/mixed new-old-raw
pass; 84 build/86 Wine guards 20 compiled inputs unchanged. Production CLI create/
reuse 63 generation/65 managed/all 63 accepted/no-write/one-frame old cache attach
first PASS (all new counters 0/smoke only), 154 active guards unchanged. New ordinary
F09 Wait 3530 CLI/full pair first PASS 10 reads 20 aggregate B/two words: every frame
state-clock-central-wide-OAM-dispatch-all audio equal; 155 active guards 21 compiled
inputs unchanged, other 180 words ordinary targeted only. Optional positive guard,
old normal scene child remains closed/no other proof repeated. No full allocator/
new 32:9 claim, USA e8 unchanged. FFFF raw 6B/other inputs/changed actor/global identity
separate, no repeat of old proof/fixture/navigation/measurements.
[Source and limits](../rom_data_unit_obj.md#actor-property-words-from-known-sprite-assignments).


The contract for additional OBJ property getter callers now also covers the full
byte input window of the two UI LDRBs: 256 queries chosen before targets, not a
catalog extension or fullword getter limit. Builder and four missing paths authenticated
with 680 halfword/25 BL/21 literal-control-flow pins. The two u16 paths update the
property while retaining allocation; known concrete assignments reuse the 182 words.
All 62 owners/files/sources verified: 184 shared words (182 properties + two words from
the four palette offsets), 72 new words/144B in 394CF4/2, 394CFE/2, 394D0C/128,
394E08/12, SHA 9d1239cc29ec40185c1605aed84b4cf74cedbc21cc217c001efb8348407253f9.
New complement with source contract and lossless model/Wine oracle closed: 14 guards,
512 UI prefixes/all 512B, eight skips, four high-u16 comparisons, four byte writers
and six setter prefixes pass on first attempt. RAM outside destinations and VRAM/OAM/
palette verified; 109 build 110 Wine guards/13 compiled inputs unchanged. The 184 shared
words come from the two dependencies without duplicate copies. Managed-runtime/CLI/
ordinary pending, e67-d66 and USA e8-d7 unchanged. FFFF raw 6B/unknown/changed actor/
global identity/computed indirect remain separate; no old proof replay/full UI-native
clock/fixture/navigation/measurement.
[Details](../rom_data_unit_obj.md#additional-property-getter-callers-and-the-ui-byte-input-window).


The OBJ property byte-input complement is now managed and consumed in EUR e68-d67:
64 files/63 resources, all 62 previous records/63 files and e67 retained. Only 72 new
words/144B; 184 shared words remain with the propertywords and colorpalettes owners,
real dependencies. Two new Wine filters pass 17 attach + four dependency checks/
lifecycle/all 144B and 72 records, plus original/interpreter/compiled full state-clock
equivalence on getter/UI prefix/mixed/fullword/live RAM. Four edges/width-mirrors/
real BIOS bus/SWI-DMA pass: API 2 new 2 property/2 new 2 palette/2 new 2 raw, CpuSet
2 new 4 property/FastSet 28 new 4 property/DMA 4 new 2 palette. No source/model/old
scene replay or measurement; production CLI/ordinary remain pending, USA e8 unchanged.
[Consumption and limits](../rom_data_unit_obj.md#managed-ui-byte-property-words-and-typed-runtime-reads).


The e68-d67 byte-property complement production CLI now passes first Wine prepare/
reuse: 64 generation 66 managed files/63 resources/two dependencies/all 64 equal to
dataset 03, no writes on reuse from different cwd. First warning-free Windows build/
frozen 0c0f5082.. and canonical hash+mtime restored; main content unchanged. One
authentic F09 frame loads previous native cache and uses 1 read/2B/1 new word, positive
attachment check; ordinary full scene still open. 182 active guards/23 compiled inputs
unchanged; source/model/runtime/old scenes closed without replay. No fixture/navigation/
private payload/measurement, other platforms to verify.
[CLI](../rom_data_unit_obj.md#ui-byte-property-words-in-the-actual-production-cli).


The declared scene for the new byte-property complement is now closed: a Wine full pair
F09 Wait 3530 passes every frame full state-clock/central-wide/OAM/dispatch/all audio,
476 reads/952 aggregate B/4 new positive words. Other 68 words retain targeted reader
proof. One new optional guard/report only, first warning-free Windows build, 183 active
guards/24 compiled inputs unchanged. One-frame CLI already positive, so no duplicate
CLI 3530; old normal/e67 property scene children and source/model/runtime/CLI closed
without replay. No full UI/allocator/all-word ordinary/new 32:9 or other platform
claimed; no fixture/navigation/private payload/performance-space measurement.
[Scene](../rom_data_unit_obj.md#ordinary-ui-byte-property-use-and-scene-equivalence).


The UI list selected by property==16, `3950A4`, now has a source contract: 48 halfword/
4 literal/4 BL and old UI/reader proofs verify four A4/B2 branches, 1F6C→1500 and stride
6. All 63 owners/sources authenticated before payload. A header 2 yields 14 new B/two
objects, SHA 1f9cfc55a363c69dcf8422d47641705427d4291a3801ff1c7646bfb8bc5daa64;
B2/14B already typed are an exact alias. Three additional references project 126 refs/
74 lists 175 objects 1198B in the same owner `graphics.obj-static-lists`, without a new
family/format. Source closed, model/publication/runtime/CLI/ordinary A4 pending;
e68-d67/USA e8-d7 unchanged. No old CFG campaign/oracle/scenes repeated, build/guest/
fixture/navigation or measurement.
[Details](../rom_data_obj.md#property-selected-ui-list-source-extension).


**ROMDATA-02-03-UI-PROPERTY-OBJ-LIST-EXTENSION completed EUR (2026-10-26).**
e69-d68 integrates three references in the same Catalog/List/Object/owner/format,
126 refs/74 lists 175 objects 1198B; new A4 only 14B/two objects, B2 shared. Same
64 files/63 resources and size dependency; all 73 previous records, 62 other files/
records and e68 unchanged. Corrected warning-free Windows build (first error only an
unqualified test constant). First two Wine filters pass: roundtrip/eight guards/
publication-repair-move management/lifecycle; two original-interpreter-compiled
draws full state-clock/709 instructions/4 dispatch/36 compiled B, two host
publications, edges-width-mirror-BIOS and mixed API 2 new 2 B2/DMA-CpuSet 14 new
2 B2/FastSet 4 prior 14 new 14 B2. Runtime handlers unchanged. 77 build 79 Wine
guards/29 compiled inputs exact. Real CLI 04 and ordinary A4 guest-host 05 still to
do; USA e8-d7/Linux/native Windows/new 32:9 pending. No old evidence/scene replay,
fixture/navigation or performance-space measurement.
[Integration and limits](../rom_data_obj.md#managed-ui-property-list-extension-and-typed-consumption).


**ROMDATA-04-UI-PROPERTY-OBJ-LIST-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen 17488009..; canonical restored
hash+mtime, main content unchanged. First Wine prepare/reuse/old-cache one frame
from /tmp pass 64 generation 66 managed/63 resources/74 spans 1198B/all 64 equal to
dataset 03, reuse without writes. 148 active CLI guards/29 compiled inputs exact.
Ordinary A4 scene equivalence still open; no old oracle/scene repeated, fixture/
navigation/private payload or performance-space measurement. Other platforms remain
to do.
[CLI and limits](../rom_data_obj.md#ui-property-lists-in-the-actual-production-cli).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-LOCATORS: negative observations closed, ordinary open (2026-10-26).**
F09 one frame/Party unit 320 do not use A4. One bank 120 locator on an authorized
exact EUR copy plus normal Down uses B2 guest-host, A4 absent; endpoint inspected,
151 guards/29 compiled inputs exact. No full pair or extended repeat/other slots.
Four snapshots read with checksum and first 7 visible rows → property 24; no global
coverage inferred. New setters 02AE70 (input/animation/tag), 02AF14 (animation),
02AF5C (state only) distinguished. First BL census for two new targets: 4 updaters/
8 setters; prefix 0578EE assigns animation 24 to human preview. Full path and
ordinary A4 guest-host/full scene witness still open; read control before further
input. Old oracles/CFG/scenes closed, no new build/gameplay fixture/measurement;
Linux/USA/native Windows/new 32:9 to do.
[Locator and next criterion](../rom_data_obj.md#ordinary-ui-property-list-locators).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-HUMAN-ANIMATION completed for the declared EUR scene (2026-10-26).**
Original join 25 halfword/6 switch words: evaluator D2D44→6375C→1092→063230/02AF14
sets 24 for results 1/2, tag 1 + property 16 selects A4. First Wine CLI Dispatch 1840
positive A4 guest-host/15 evaluation B; negative 1400 retained because it stopped
before event 1600, corrected boundary justified by already positive original 1840
evidence. First warning-free Windows test build/frozen 427845f2..; only 5 optional
A4 guard/report lines in validator, product runtime unchanged. First Wine full pair
1840 passes all state-clock/central-wide/OAM/dispatch/all audio and new A4 guest-host
gate; 5 streams with real samples. 152 guards/29 compiled inputs exact; authentic
originals/inputs/private copy, no cart edit or new derived checkpoint. Two private
preflights corrected before Wine, no guest retry. 01/02/03/04/05 declared closed;
old oracles/scenes and negative locators not repeated. Linux/USA/native Windows/new
32:9, full UI/global indirect calls/other families remain open; no performance-space
measurement. [Scene and limits](../rom_data_obj.md#positive-dispatch-use-and-scene-equivalence).


**ROMDATA-02-UNIT-OBJ-FFFF-READER-FIELDS model completed EUR (2026-10-26).**
New lossless model: query 65535, encoded property u16 word + directory u32 word,
two ranges 6B, 980 conditional witnesses/2240 references (196 direct 784 redirect)
with metadata52/archive49 dependencies. Previous valid models and byte aliases
unchanged; no interpreted root/sequence/corrected pointer/cap. First warning-free
Windows build/frozen 83ffcd75.. and first single Wine filter pass 13 guards/3 numeric
roundtrips/980 records/6B and two complete original readers 28 instructions,
registers/RAM guard. 156 build 157 Wine guards/35 compiled inputs exact. 71-pin/
60-query classification and old A4/model/runtime/scene evidence closed without
replay. Publication/owner runtime 03, native clock/ordinary pending; production
e69-d68/64 files 63 resources and USA e8-d7 unchanged. Linux/USA/native Windows/
new 32:9 to do, no fixture/navigation/performance-space measurement.
[Model and limits](../rom_data_unit_obj.md#typed-model-for-known-ffff-reader-fields).


**ROMDATA-03-UNIT-OBJ-FFFF-READER-FIELDS managed/runtime completed EUR (2026-10-26).**
e70-d69: 65 files 64 resources, all 63 previous decoded records/files and e69
preserved; USA e8-d7 unchanged. New numeric resource with real metadata52/script
archive dependencies, model 980 witnesses 2240 refs unchanged. Immutable owner
serves only 2B property + 4B directory from typed fields; gate 3B4/3D4, counters and
two bit fields. Full pointer, live open BIOS/unaligned LDR rotate 16 and guest
control preserved, no invented sequence/target/null/cap. First warning-free Windows
core/frontend build and first two new Wine filters PASS: 20 attach guards, four
dependency guards, publication/lifecycle unavailable directory, all 6B raw altered/
two fields and distinct old owner. Original/interpreter/native 14 paths equal full
state/clock: getter/directory/zero early return/three BIOS prefixes, live RAM/mixed
reads. Two edges/width/mirrors/real BIOS bus; API 2 new 2 raw, CpuSet 2 new 6 raw/
FastSet 4 new 28 raw/DMA 4 new 4 raw preserved. Source/model/A4/old proof closed
without replay. Production CLI 04 and ordinary 05 remain to do, no full sprite/
allocator ABI claimed. Linux/USA/native Windows/new 32:9 pending. No fixture/navigation/
private payload/performance-space/Android/package/commit/push.
[Consumption and limits](../rom_data_unit_obj.md#managed-ffff-reader-fields-and-typed-bus-consumption).


**ROMDATA-04-UNIT-OBJ-FFFF-READER-FIELDS-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen candidate; canonical exact hash+
mtime restored/main content unchanged. First three Wine calls prepare/reuse/old-cache
one frame from unrelated cwd pass 65 generation 67 managed/64 resources/all 65 equal
to dataset 03/two real dependencies/reuse without writes. Original F09 checkpoint and
previous cache intact: attachment proof only, positive ordinary/equivalence not
claimed. Source/model/runtime/old scene closed without replay; USA e8-d7 and Linux/
native Windows/new 32:9 to verify. No fixture/navigation/private payload/performance-
space/Android/package/commit/push.
[CLI and limits](../rom_data_unit_obj.md#actual-production-cli-for-ffff-numeric-fields).


**ROMDATA-05-UNIT-OBJ-FFFF-READER-FIELDS-EXISTING-CONTEXT-LEAD closed, positive ordinary open (2026-10-26).**
New readonly locator on the 81 existing EUR checkpoints: actor+34 FFFF/unit kind
0-1/current job-alternate job predicate from 980 witnesses, both modes/override/
null-dormant-outside retained, without requiring a nonexistent sequence target.
159 potential candidates in 61 snapshots, all kind 0/job 0/alt 0/override 20 clear:
157 null body pointers and two 01480168 outside EWRAM. No body task available in
EWRAM, no registry identity/same assignment/actor validity inferred. This is not
global unreachability evidence and provides no positive scene: ordinary reader/full
pair remain open. Hash/mtime/header/payload/ROM/input/model/closed contract exact.
Source/model/runtime/CLI/126 sequence/old scene closed without replay; no new build/
Wine/gameplay/fixture/navigation/private payload/performance-space. Linux/USA/native
Windows/new 32:9 pending.
[Locator and limits](../rom_data_unit_obj.md#existing-checkpoint-locator-for-ffff-numeric-readers).


**ROMDATA-01-UI-JOB-LOOKUP-SOURCE completed for EUR 20-slot projection (2026-10-26).**
Root 399750 shared by six readers D0538/D0548/D0568/D0588/D05A8/D05C8: job byte
or CBEC8 property 1/4/6/9/0, original truncations and name helper 165E8. Domain
selected before data from 12CA98: index list of 20 RAM slots 02002E78/stride 16,
mode 3 maximum +5/inclusive tie; one occupied slot allows any 0..19. R6 preserved
through CFF70 clear before D05C8. 268 halfword/27 critical pins/9 literal/8 BL
authenticated. No invented global cap/full physical extent/adjacency/game identity;
other raw caller matches are not automatically semantic code. Twenty distinct u8
jobs/20B/one range SHA 969b2cd5.., 20 metadata52 rows reconstructed, disjoint from
all 64 current owners. No sprite target followed. Source child closed; model/Wine
oracle/managed/runtime/CLI/ordinary open. e70-d69/65 files 64 resources and USA 8-7
unchanged; no build/guest/test/fixture/navigation/private payload/performance-space/
Android/package/commit/push. Linux/USA/native Windows/new 32:9 pending.
[Contract and limits](../rom_data_gameplay.md#twenty-slot-job-lookup-source-contract).


**ROMDATA-02-SLOT-JOB-LOOKUP-MODEL model completed EUR (2026-10-26).**
Strict model 20 (slot,u8 job)/20B/source base/hash/byte_at/direct metadata52; Catalog
checks EUR/three producer-reader blocks. Only two module declarations, no Extraction/
runtime registration: e70-d69/65 files 64 resources/USA 8-7 unchanged. First warning-
free Windows frontend build and first single Wine filter PASS: 20 independent entries/
20B/12 JSON-shape-source-dependency guards/roundtrip/disjoint from all 64 owners.
123 original paths: 21 bare job/81 complete property wrappers/21 property 0 prefixes
before 165E8, three high input aliases. All 20B at six original LDRB, callee/full
RAM outside 20B stack/VRAM-OAM-palette preserved. No guest cap 20/null/host fallback,
code parser 268 pins/metadata52/FFFF/A4/old proof not repeated. Managed/runtime 03/
CLI 04/ordinary 05 open; no new fixture/navigation/private payload/performance-space/
Android/package/commit/push. Linux/USA/native Windows/new 32:9 pending.
[Model and limits](../rom_data_gameplay.md#typed-twenty-slot-job-lookup-and-original-reader-proof).


**ROMDATA-03-SLOT-JOB-LOOKUP-RUNTIME managed/runtime completed EUR (2026-10-26).**
e71-d70: 66 files 65 resources/gameplay.slot-job-lookup/slot-job-lookup.json, real
metadata52 dependency; all 64 previous records/files and e70 preserved, USA 8-7
unchanged. Immutable owner 20 u8/20B/page 399/20-record mask, original low 8 input/
slot selection/clear/properties/name/raw outside preserved, no guest cap 20. First
warning-free Windows core/frontend build and two new Wine filters PASS after ENOSPC
stop and retry on temporary filesystem: 18 attach guards/3 dependency guards/
publication/lifecycle unavailable directory/all 20B raw altered/all 20 bits/distinct
FFF owner. Sixteen original/interpreter/native paths equal full state-clock: getter/
properties/name ID prefix/high input/slot 20 raw/live RAM and 12CA98 mode 3 slot 19
→ CFF70 clear → D05C8 before 165E8. Edges/width/mirrors/real BIOS bus; API 2 new
2 raw, CpuSet 20 new 4 raw/FastSet 20 new 12 raw/DMA 20 new 4 raw preserved. Old
64 owners confirm raw neighbors. Source 268 pins/model 123 readers/old proof closed
without replay. CLI 04/ordinary 05 open; Linux/USA/native Windows/new 32:9 pending.
No fixture/navigation/private payload/performance-space/Android/package/commit/push.
[Consumption and limits](../rom_data_gameplay.md#managed-twenty-slot-job-lookup-and-typed-bus-consumption).


**ROMDATA-04-SLOT-JOB-LOOKUP-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen candidate; canonical exact hash+
mtime restored/main content unchanged. First three Wine calls prepare/reuse/old-cache
one frame from unrelated cwd pass 66 generation 68 managed/65 resources/all 66 equal
to dataset 03/metadata52 dependency/reuse without writes. Private root on temporary
filesystem via workspace link after ENOSPC 03, no further cleanup/budget formatting/
build retry. F09/original cache intact: attachment proof only, positive ordinary/
equivalence open. Source/model/runtime/old proof closed without replay; Linux/USA/
native Windows/new 32:9 to verify. No fixture/navigation/private payload/performance-
space/Android/package/commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-production-cli-for-the-twenty-slot-job-lookup).


**ROMDATA-05-SLOT-JOB-LOOKUP-BANK-ORDINARY completed for declared EUR scene (2026-10-26).**
First new CLI bank 120 positive 2 reads/2B/1 slot, with existing copied checkpoint/
Down 8..20 input and production e71 dataset generation-GmxCy2. Sole scope new job
lookup, old A4 negative/B2 positive closed. Only 5 optional guard/report lines in scene
validator, runtime/product/old guards/states unchanged. First warning-free Windows
frontend build and first exact Wine filter PASS: 120 full state-clock from 3660 to
3780/central-wide/OAM/native-fallback dispatch and bit-exact mixed PSG-FIFO left-right-
HLE. All 5 sample counts [263340,263340,31680,31680,263264] positive; new use 2B
1 slot matches CLI. Other 19 ordinary slots targeted only. Inspected CLI capture
284x160: Bank background, UI panels absent from raw framebuffer; no final composite
GUI/full name-actor-allocator ABI/new 32:9 claimed. Input/copy/original/cache/dataset
intact; no fixture/navigation/private payload/performance-space/old proof replay.
Source/model/03/04/05 declared closed, full TODO active/Linux-USA-native Windows-
new 32:9 pending.
[Scene and limits](../rom_data_gameplay.md#ordinary-bank-scene-use-of-the-twenty-slot-job-lookup).


**ROMDATA-01-POST-SLOT-LOOKUP-READERS completed for three EUR RAM APIs (2026-10-26).**
D05E8 any nonnull and D0618 count read only the first words of 20 RAM slots 02002E78/
stride 16; they do not validate pointers/liveness, D05E8 returns early. D0650 increments
byte 02001F68 only if old<=98, leaves 99..255 unchanged (no global clamp). All ignore
incoming args for these addresses; six literals are executable RAM base/offset support,
no new ROM asset root. 68 halfword/30 critical pins/6 literals/three complete blocks
verified; 3/2/1 raw BL matches remain candidates/not semantic completeness. No live RAM
export/new decoder-resource-owner/revision/ROM table cap 20. e71-d70/66 files 65
resources/USA 8-7 unchanged. Next partial D0670 excluded. Source child closed,
old evidence/Bank not repeated; no build/guest/test/fixture/navigation/private
payload/performance-space. Linux/USA/native Windows/new 32:9 pending.
[Live state and limits](../rom_data_gameplay.md#live-state-helpers-following-the-slot-job-lookup).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE completed for the EUR Bank12 sample
(2026-10-26).** A test-only observer wraps native read8/16/32 APIs and subtracts the
20756 merged ranges from the 65 e71 source plans. First warning-free Windows build
and one Wine filter: 3166 context reads/3683 image bytes; 198 reads include 676 bytes
outside the plans, 79 ranges observed. Full state/clock, central-wide/OAM/dispatch
and five audio streams match at each frame 3660–3672. The 676 bytes are leads to
authenticate, including five words 370E70/74/80/84/EB0; no new resource/revision
or global completeness. Exported PC may be stale; interpreter/HLE/DMA/native
constants and other scenes outside sample. 73 input hash-mtime exact; main/C API/
old validators unchanged, only new test module with cfg declaration. No new fixture/
navigation/performance measurement. Old Bank 120/source/model/runtime/CLI
acceptances remain closed. Linux/USA/native Windows/new 32:9 to do.
[Contract, transparency and limits](../rom_data_runtime.md#bounded-native-rom-api-residue-inventory).


**ROMDATA-01-NATIVE-RESIDUE-HIGH-WORD-READERS statically completed EUR
(2026-10-26).** The five observed words 370E70/74/80/84/EB0 support execution:
two DMA register address directories and one Thumb IWRAM entry passed to BX r2.
392 halfword/47 pins/5 literal/4 BL in six complete blocks authenticated; actual
loads 0846/09B2/0D56/3A7E/11DA/11EE distinguished from exported PC. Original callers
prove two 4x4B projections 370E68/78 and one word 370EB0, 36B disjoint from 65
owners; 20B observed classified, 656B lower still to classify. No inferred global
cap, RAM body/ABI/scheduler or new asset/owner/revision; explicit ROM/code dependency
and 676B still outside managed plans. No build/guest/test/fixture/navigation/
measurement/old acceptance repeated. e71-d70/USA 8-7 unchanged; Linux/USA/native
Windows/new 32:9 pending.
[Hardware directories and branch control](../rom_data_runtime.md#high-offset-native-residue-is-hardware-and-branch-support).


**ROMDATA-01-NATIVE-RESIDUE-LOWER-READERS completed for EUR static witnesses
(2026-10-26).** 656B observed: 158 literal words/632B with 184 linear Thumb paths
within 96B of saved contexts (507 distinct halfwords), and 6 words/24B in 5 jump
tables with 82 halfwords of bounds/index/load/MOV pc prefix authenticated.
8/35/69/48/5-entry projections; only 0/0/27/4-6/1 observed. This is not a dynamic
instruction/mode trace or proof of all uses. Together with the 20B high hardware/
branch support, all 676B outside plans have static code-support witnesses; they
remain outside the 65 owners/original ROM dependency. No inferred new family/asset
root/owner/revision/complete migration. No build/guest/test/fixture/navigation/
measurement/old acceptance repeated. e71-d70/USA 8-7 unchanged; interpreter/HLE/
DMA/native constants/other scenes and Linux/USA/native Windows/new 32:9 separate.
[Literals and jump tables](../rom_data_runtime.md#lower-native-residue-has-literal-and-switch-table-witnesses).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE-F09 completed for second EUR sample
(2026-10-26).** Same frozen observer executable, no build: one Wine filter 12 frames
8476→8488 from existing F09 checkpoint/inputs preserves state/clock/buffer/OAM/
dispatch and five audio streams. 13671 context reads/13961B image; 604B outside
plans, 584 shared with Bank and 20 new B/5 words. All 5 have linear Thumb PC-relative
witnesses (20 halfwords); static source, not a dynamic trace/mode/all uses. Union of
two samples 696B still outside 65 owners, no asset/revision inferred. 76 input
hash-mtime and compiled code exact, retention helper 0 paths. No fixture/navigation/
measurement or old F09/Bank acceptance repeated. Other interpreter/HLE/DMA/constant
paths and Linux/USA/native Windows/new 32:9 remain separate.
[Second sample](../rom_data_runtime.md#second-native-api-sample-from-the-existing-f09-checkpoint).


**ROMDATA-01-HOST-ROM-READ-RESIDUE completed for collector and EUR F09 sample
(2026-10-26).** Optional non-default Cargo feature rom-data-read-audit: thread-local
address/reader/length collector on resolved bus and selected host audio/sprite/
tables/keyboard; interpreter instruction fetch excluded, execution/clock/API/
savestate unchanged. First build core compiled/frontend failed due to missing c_void
qualification in test; test-only correction and warning-free incremental build. One
Wine filter: ARM/Thumb/width/mirror/audio/lifecycle probe and 12 frames 8476→8488
state-clock-buffer-OAM-dispatch-five audio identical. 23061 requests/19984B image,
1146B outside 65 owners; includes all 604B outside plans from old API report and adds
542B. 90 new high B/6 ranges remain sources to authenticate, 64B also from SpritePeek.
No format/asset/extent/owner/revision inferred. 76 execution inputs + 1 comparison
report hash-mtime exact, no old evidence/fixture/navigation/measurement. e71-d70/
USA 8-7 unchanged; KeyboardWidths inactive/other threads-scenes/host accesses/typed
HLE fast path out of scope; Linux/USA/native Windows/new 32:9 pending.
[Collector, proof and limits](../rom_data_runtime.md#optional-bus-and-host-rom-read-audit).


**ROMDATA-01-BUS-HOST-HIGH-RESIDUE completed for EUR source contracts
(2026-10-26).** 90 new B classified: 64B two indirect OBJ lists 394F4C/26B/4 objects
and 40D89E/38B/6 objects; 10B parameters (2 phase thresholds, 4 signed vertical
words); 16B Thumb IWRAM control pointers. 295 window halfwords/53 pins/7 literals +
ABI hash reused. Original copy 0924E8 proves D968E8..D97728→03005F28..03006D68,
constructor D9752C writes list+158; raw ROM disassembly at empty RAM target rejected.
One existing F09 producer-args Wine frame confirms 2 readers/10 objects, no old
equivalence repeated. 40D89E from task+24 is an indirect observation, not an invented
literal; command creation not reconstructed. Timing compares OLD counter, post-update
phase 0..2 not a global cap; corrected byte>>1 query has 128 possible indices, only
4..7 new here. 64B lists + 10B parameters still to model/publish; 16B control remain
ROM/code dependency; 452B lower separate. e71-d70/65 owners/USA 8-7 unchanged, no
build/fixture/navigation/measurement. Linux/USA/native Windows/new 32:9 pending.
[Indirect lists and evidence](../rom_data_obj.md#observed-indirect-obj-list-sources-beyond-the-direct-catalog),
[parameters and control](../rom_data_runtime.md#high-bus-and-host-residue-source-contracts).


**ROMDATA-02-OBSERVED-OBJ-LISTS-MODEL completed for two EUR lists
(2026-10-26).** Isolated obj_lists::observed API reuses List/Object: 2 lists/
10 objects/64B, identity/count/shape/field width/strict JSON/source consistency
authenticated, constructor-bound root distinct from ordinary-task argument. First
warning-free Windows frontend build and one Wine filter: 64B/2 spans/14 guards/
6 original paths ROM 1500/170C/copied RAM 030065A8, output/matrices/stack/callee/
external RAM/OAM-VRAM-palette exact. Canonical JSON 4530552b..; 75 input-artifact
guards and compiled code unchanged. Only 2 declarations in existing files; old
Catalog 74/175 objects 1198B/e71-d70/66 files 65 resources/USA 8-7 unchanged. No
publication/owner/revision or ordinary evidence to infer; model closed, managed
runtime/CLI/new use from file still open. No old 74/ABI/copier/scene check repeated,
new fixture/navigation/measurement. 10B parameters and 452B lower separate; Linux/
USA/native Windows/new 32:9 pending.
[Model and checks](../rom_data_obj.md#isolated-typed-model-for-the-two-observed-indirect-lists).


**ROMDATA-03-OBSERVED-OBJ-LISTS-RUNTIME completed for two EUR lists
(2026-10-26).** e72-d71 integrates the two indirect identities into the existing
OBJ resource: 76 lists, 185 objects and 1262 source bytes. The 126 direct references,
66 files/65 resources, same size dependency, all 74 previous lists, other 64 files/
records and e71 remain. USA 8-7 unchanged. First warning-free Windows build and two
Wine filters pass publication/repair/move, 12 attach guards and Load/Undo/backup/
restart; six original/interpreter/native paths have equal full state and clock with
all 64 new ROM bytes altered. Edges/widths/mirror/BIOS, six DMA-CpuSet-FastSet
transfers, API reads 2B old + 2B new and 2B altered raw + 2B new, six host publications
pass. 5571 instructions/2418 dispatch are coverage only; 265 typed bytes served by
native APIs. File exactly matches accepted model, 77 input/executable guards and
compiled code unchanged; artifact completion removes only interrupted/moved copies.
Production CLI and new ordinary equivalence remain open; ten parameter bytes and
452 lower bytes separate. No old model/ABI/scene repeated, new fixture/navigation/
measurement. Linux, USA, native Windows and new 32:9 to verify.
[Integration and evidence](../rom_data_obj.md#managed-observed-indirect-lists-and-typed-consumption).


**ROMDATA-04-OBSERVED-OBJ-LISTS-ACTUAL-CLI completed EUR (2026-10-26).**
First warning-free Windows production build; separate CLI c6d674d7.., canonical
executable hash/mtime exactly restored, main content unchanged with mtime-only Cargo
invalidation. First three Wine commands from unrelated cwd pass create/reuse/old-cache
one frame: 66 generation files/68 managed, all 66 equal to runtime dataset, 65 resources/
76 OBJ spans/size dependency. Reuse without hash/mtime changes. Original F09/inputs/
cache at 16:9: both new lists 00394f4c/0040d89e guest=true host=true; OBJ totals
147 reads/147B/14B count/133B attributes/12 host reads. Positive use, full ordinary
equivalence still open. All 143 CLI guards and implementation/canonical/originals
exact; artifact helper zero paths. No old source/model/runtime/producer logging/
scenario repeated, new fixture/navigation or measurements. Linux/USA/native Windows/
new 32:9 to verify; ten parameter bytes and 452 lower separate.
[CLI and evidence](../rom_data_obj.md#actual-cli-for-the-observed-indirect-lists).


**ROMDATA-05-OBSERVED-OBJ-LISTS-ORDINARY completed for the first EUR F09 frame
(2026-10-26).** Only five optional lines in existing validator require both new lists
guest+host. First warning-free Windows build and one Wine filter: frame 8476→8477,
full state/clock/central-wide images/OAM/dispatch and all audio equal. Five streams
with real samples [2196,2196,264,264,2120]; OBJ counters 147 reads/147B/14 count/
133 attributes/12 host exactly match CLI. 6803 native/22849 fallback coverage only;
75 input/executable guards, originals/cache/dataset/canonical exact. Five-line delta
checked against preimage, other implementations unchanged; artifact helper zero paths.
Source/model/managed/runtime/CLI and declared ordinary closed; other scenes/indirect
calls/composed GUI/ABI/new 32:9 open. No old 12/3530/scenario/model/runtime repeated,
new fixture/navigation or measurements. Ten parameter bytes/452 lower and Linux/USA/
native Windows separate or to verify.
[Equivalence and limits](../rom_data_obj.md#ordinary-first-frame-equivalence-for-the-observed-lists).


**ROMDATA-02-OBSERVED-TASK-PARAMETERS-MODEL completed EUR (2026-10-26).**
Isolated observed_task_parameters API for two independent fragments: phase 1/2 u8
thresholds at 371065..67 and four signed16 corrections indices 4..7 at 394E14..1C.
Identity/indices/strict JSON/10B reconstruction/two spans/code-literal-source hash;
no inferred duration/global cap/physical extent. Phase 0/other bytes outside model;
corrections 0..3 retain previous owner, 8..127 remain external. First warning-free
Windows build and one Wine filter: 27 guards and 18 original prefixes (10 old-counter/
phase-wrap, 8 low-bit alias/signed-sub), RAM/callee/OAM-VRAM-palette exact. Canonical
model 4b52c5f1..; 79 input/executable guards and implementation exact. Only two
declarations in old hubs; pipeline/e72-d71/66 files 65 resources/76 OBJ lists/USA 8-7
unchanged. New 10B disjoint from all owners and not yet published; parameter managed/
runtime/CLI/ordinary open. Artifact helper zero paths. No source/295 pins/old model-
ABI-runtime-CLI-scene repeated, new fixture/navigation/measurement. 452 lower separate;
Linux/USA/native Windows/new 32:9 to verify.
[Model and limits](../rom_data_gameplay.md#isolated-model-for-observed-task-parameter-fragments).


**ROMDATA-03-OBSERVED-TASK-PARAMETERS-RUNTIME completed EUR (2026-10-26).**
e73-d72 publishes gameplay.observed-task-parameters/observed-task-parameters.json:
67 files/66 resources, all 65 previous files/records/e72/76 OBJ lists retained. Two
independent fields, no invented dependency, only 10B/two spans 371065..67 and
394E14..1C; pages 371/394 compose previous owners and ROM, without new cap/ABI/clock/
state. First two Wine filters pass 13 attach refusals/3 full-set guards, publication/
repair/move/lifecycle and 24 original-interpreter-native prefixes with equal full
state/clock and all 10B raw altered. Includes phases 0/3/255, corrections 0/16/255,
edges/mirror/BIOS, six DMA-CpuSet-FastSet, API reads 2 new + 2 old with old raw
altered and 2 new + 2 modified raw. 26B native; 316 instructions/24 dispatch coverage
only. Model 4b52c5f1.. exact, 74 guards and compiled code exact. First build core
compiled, frontend omitted two test arguments: only two zeros corrected, warning-free
retry; no Wine filter failed. Artifact helper removes only interrupted/moved. Production
CLI and ordinary parameters remain open; Linux/USA/native Windows/new 32:9 pending,
452 lower separate. No old model/source/ABI/scenario repeated, new fixture/navigation/
measurement.
[Runtime and evidence](../rom_data_gameplay.md#managed-observed-task-parameters-and-typed-bus-consumption).


**ROMDATA-04-OBSERVED-TASK-PARAMETERS-ACTUAL-CLI completed EUR (2026-10-27).**
Warning-free Windows production build/c7f47bd4..; canonical hash/mtime exactly restored,
main content unchanged. First prepare interrupted by user: runner absent, manifest
without current/result; copy/log archived and same binary reused. Three accepted
commands exit 0: create/reuse/old-cache one frame, 67 generation files/69 managed/
66 resources/two spans 10B/no deps/all 67 exact, reuse without writes. First frame
1B threshold/1 phase but 0 corrections: presence gate explicitly negative, retained.
One new runc 12 frames, motivated by that negative and existing source lead, exit 0
with 20 reads/28B/12 thresholds 16 corrections/2 phases 4 rows. 145 guards and
implementation/originals/cache/canonical/user Atlas unchanged; artifact helper zero
paths, interrupted copy retained. CLI closed; ordinary full state 12 frames to verify.
No prepare/reuse/build/old source-model-runtime-scenario repeated, new fixture/
navigation or measurements. Linux/USA/native Windows/new 32:9 and 452 lower separate
or pending.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-observed-task-parameters).


**ROMDATA-05-OBSERVED-TASK-PARAMETERS-ORDINARY completed EUR (2026-10-27).**
Only five optional lines in existing validator require both groups and all six rows.
First warning-free Windows build and one Wine filter: 12 F09 guest frames 8476→8488,
every full state/clock/central-wide image/OAM/dispatch/audio equal. Five streams with
real samples [26334,26334,3168,3168,26258]; 20 reads/28B/12 thresholds 16 corrections/
2 phases 4 rows exactly match CLI. 88123 native/279744 fallback coverage only.
76 guards, originals/cache/67-file dataset/canonical/code/user Atlas exact. Five-line
delta checked against preimage; artifact helper zero paths. Source/model/managed/
runtime/CLI/declared ordinary closed. First correction-negative frame and interrupted
prepare remain closed; no old source-model-runtime-CLI-collector-scenario repeated,
new fixture/navigation/measurement. Other scenes, indirect calls/ABI/composed GUI/
new 32:9 open; 452 lower and Linux/USA/native Windows separate or pending.
[Equivalence and limits](../rom_data_gameplay.md#ordinary-twelve-frame-equivalence-for-observed-task-parameters).


**ROMDATA-01-BUS-HOST-LOWER-RESIDUE completed EUR (2026-10-27).**
452 bytes/113 words/62 ranges classified with 114 contextual reads: 404B from
102 Thumb PC-relative LDRs on 101 literals, 48B from 12 observed switch entries 7–18.
Load 080A1146/MOV PC 080A1148 retains Thumb; original prefix normalizes u16 zero/>24
to 1 before index, without new global limit. New optional SiteCapture retains previous
schema/keys; PC/mode scope requires opcode/address/width verification, not automatic
attribution for nested HLE/DMA. One warning-free incremental Windows build after
helper error only in test; one Wine filter with new probes and one current F09 arm
12 frames observes all 113 words. 85 guards/cfg preimage/code/Atlas exact; artifact
helper zero paths. No old acceptance repeated, fixture or measurement. The 452B support
code, e73-d72/67 files 66 resources unchanged. New sources 394FBC/371064 phase 0/
371034 require their own contracts; no format/extent inferred from literals. Linux/
USA/native Windows/new 32:9 to verify, global TODO open.
[Contract and limits](../rom_data_runtime.md#lower-bus-and-host-residue-source-contracts).


**ROMDATA-01-ADDITIONAL-CONSTRUCTOR-OBJ source/model completed EUR (2026-10-27).**
394FBC is a table of four conditional 32-bit threshold/list pointer pairs, 32B;
original readers 9DB56/9DBCA and already authenticated constructor prove four lists
394F82/90/9E/AC, 56B/eight objects. Total 88B, no owner overlap. Increment/store/
reload u8 before comparison; 255→0, new phase >3 disables. Old phase not bounded
before lookup: model 0–3, no new global cap. Isolated model with strict JSON/source
projection/list dependency verification. First warning-free Windows build/single
Wine filter passes 13 guards, 15 counter prefixes, four pointer-constructor prefixes
and four new draws with existing copied oracle. 94 guards/code exact, hub declarations
only/helper visibility only; artifact helper zero paths. Retained F09 does not read
these data: no ordinary use claimed. Production e73-d72/67 files 66 resources unchanged;
extraction/runtime owner/positive use to do. Linux/USA/native Windows/new 32:9 pending,
no old acceptance repeated/fixture/navigation/measurement.
[Contract and model](../rom_data_obj.md#conditional-constructor-obj-animation-source-and-model).


**ROMDATA-03-CONSTRUCTOR-ANIMATION-RUNTIME completed EUR (2026-10-27).**
e74-d73/68 files 67 resources publishes 32B table with OBJ dependency; catalog 80 lists/
193 objects 1318B, all 76 previous lists/126 direct refs/65 other files-records exact.
Exact table owner with previous/raw composition, lists served by existing OBJ owner.
First Wine run passes 10 guards/2 halfsets/lifecycle/publication and 13 original-
interpreter-native state-clock prefixes with 88B raw altered: 36B typed native/
20 thresholds 16 pointers/4 phases, 4 external queries without charges, 708 bus and
composition cases. It then stops on test setup error: load_state before ROM restoration.
Failure retained, no full PASS claimed. Only test order corrected; remaining filter on
already validated dataset passes 6 DMA/SWI and 4 host publications/two actual objects
each, without repeating passed portions. Production identical between filters. Final
builds warning-free, 169 guards/code/Atlas exact; artifact helper 0 paths, failed
diagnostics and offline 68 retained, only superseded never-run exe retired. Real CLI/
positive ordinary use to do; retained inactive F09 insufficient. Linux/USA/native
Windows/new 32:9 pending; no new fixture/measurement/replay.
[Consumption and evidence](../rom_data_obj.md#managed-constructor-obj-animation-and-typed-consumption).


**ROMDATA-04-CONSTRUCTOR-ANIMATION-CLI completed EUR (2026-10-27).**
First warning-free Windows production build/1e98de03..; canonical hash/mtime restored
and main content unchanged. Three Wine CLI calls exit 0: prepare/reuse without writes/
runc 0 frames. 68 generation files/70 managed/67 resources, all 68 hashes identical
to accepted runtime and all 67 runtime IDs/hashes exact; old cache loaded. Reuse and
startup hash/mtime unchanged, 145 guards/439 sources exact; helper 0 paths. Zero frames/
dispatch/fallback/new counters: real registration/attach evidence, no positive ordinary
use inferred. No F09/source-model-runtime-scenario repeated, new fixture/navigation/
measurement. Positive ordinary use and Linux/USA/native Windows/new 32:9 remain to verify.
[CLI and limits](../rom_data_obj.md#actual-cli-for-constructor-obj-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE partial EUR (2026-10-27).**
Passive Wine-only observer implemented and optional positive guard in validator
(seven lines; previous loop exact without guard). First warning-free Windows build;
four normal observations exit 0 but NEGATIVE: two movements 160/200 frames from
copied EUR 12006 checkpoint, delayed selection 160, confirmation 200 from new selection
state. Table 0/phases 0/lists unused, no full pair or positive fixture claimed.
84 guards/code/originals exact; previous normal provenance retained without 3530-frame
replay. No manual RAM/task/phase/ROM mutation. Flag setter 9D054/call 6 verified as
lead, normal action/semantics still to prove; no further blind route or forced flag.
Unit open, fixture/diagnostics retained; independent 371064 phase 0 source separate.
Linux/USA/native Windows/new 32:9 pending, no measurement.
[Evidence still open](../rom_data_obj.md#ordinary-constructor-animation-discovery-remains-partial).


**ROMDATA-01..05-CONDITIONAL-PHASE-ZERO completed EUR (2026-10-27).**
371064 phase 0 now typed: conditional thresholds 0/1/2=16/8/8, 3B span plus
8 corrections, 11B in same file/68 files 67 resources/e75-d74. Old 2 rows +
4 corrections/all 66 other records-files exact. No global cap before lookup or ABI
change. Disjoint coverage partition 3 phases/4 corrections. First warning-free Windows
build/single Wine run passes 4 new original prefixes/7 attach/6 native state-clock/
3 DMA and raw edges. First production build/canonical restored, one auto-prepare
28-frame CLI passes all 68 files/3 phases 4 corrections/36 reads 44B/28 thresholds
16 corrections. Only offline log matcher corrected after CLI exit 0; command not
repeated. Same frontend exe: one new full pair 28 frames 8476→8504 passes every
state-clock/image/OAM/dispatch/audio with samples [61446,61446,7392,7392,61370],
exact CLI counts. 215 guards/code/5-line unguarded loop exact; helper 0 paths. No old
12 frames/source/model/runtime/constructor repeated or new fixture/measurement.
Ordinary constructor remains partial; 371034 and Linux/USA/native Windows/new 32:9
platforms still to verify.
[Contract and evidence](../rom_data_gameplay.md#conditional-phase-zero-threshold-extension).


**ROMDATA-01-TASK-CYCLE-DELTAS / ROMDATA-02-TASK-CYCLE-MODEL completed EUR
(2026-10-27).** Conditional projection 371034–371064/48 signed-i8 deltas, disjoint
from all 67 e75 resources. Readers 18E10/18E2A use OLD u8 before normalization;
increment stored as u8 before helper 1464EC/remainder 48, 255→0. Signed addition/
subtraction on halfword task+8 with low 16 store; axis/unit/lifetime not inferred.
No cap introduced: queries 48..255 None, 48/49/50 belong to old owner. First
warning-free Windows build/single Wine filter passes 23 guards/58 original prefixes/
8 helper edges/all 48 rows and RAM-callee guards, 77 frozen inputs exact. Lossless
model/strict JSON; revision e75-d74/68 files 67 resources unchanged. Management/
runtime consumption/CLI/ordinary 48 completed in units below; source test alone did
not claim consumption or scene. Previous constructor ordinary still partial; Linux/
USA/native Windows/new 32:9 pending, no measurement or new ordinary fixture.
[Contract and evidence](../rom_data_gameplay.md#conditional-european-task-cycle-delta-source-and-model).


**ROMDATA-02/04-TASK-CYCLE-RUNTIME completed EUR/Wine (2026-10-27).**
New resource gameplay.task-cycle-deltas/file task-cycle-deltas.json/e76-d75:
69 files 68 resources, 48B signed-i8/one span, all 67 previous resources/files
exact. Typed owner composes new bytes/prior/raw, no cap or ABI change; USA 8-7.
First warning-free Windows build/first Wine filter passes 9 transactional attach
refusals/2 full-set/interrupted new file, 8 original-interpreter-native prefixes
with equal full state/clock and 48 raw B altered, 5 explicitly compiled reads. All
48 rows, read32 boundary with 2 new + 2 old bytes, edges/mirror, 3 DMA/SWI transfers
across owners, unavailable files/Load/Undo with owner retained. 154 guards/code exact;
helper retires only interrupted copy. No source filter or old acceptances repeated.
Production CLI completed below; positive ordinary 48 deltas completed below;
constructor ordinary partial. Linux/USA/native Windows/new 32:9 pending, no measurement.
[Consumption and evidence](../rom_data_gameplay.md#managed-task-cycle-deltas-and-typed-consumption).


**ROMDATA-05-TASK-CYCLE-CLI completed EUR/Wine (2026-10-27).**
First warning-free Windows production build/canonical restored with exact hash mtime;
three Wine commands prepare/reuse/runc 0 exit 0. All 69 files equal accepted Rust
dataset, all 68 runtime IDs/hashes equal manifest, 48 delta rows registered, existing
native cache loaded. Entire 71-file pathset/hash mtime unchanged on reuse and startup;
226 guards/code exact, helper 0 paths. Zero frames/dispatch/fallback/delta reads:
attachment only; ordinary positivity and full pair 48 completed below. No blind route/
manual flag. Linux/USA/native Windows/new 32:9 pending, no measurement.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-task-cycle-deltas).


**ROMDATA-05-TASK-CYCLE-ORDINARY completed EUR/Wine (2026-10-27).**
Neutral Bank 48 scene 3660→3708 selected from source and normal snapshot: initializer
18D6C/default 03, generic 015E48/03-07 and candidate Bank root+D40 flags 03 phase 21,
address consistent with caller 5C1F8. Setter 18E5C retains old 04/08 with new 01:
argument 03 alone does not prove mode. 11/3/89 raw BL are candidates, no call graph/
liveness inferred from RAM alone. One existing CLI without rebuild uses 48 reads 48B/
all 48 phases, dataset 71 paths/hash mtime unchanged. First warning-free frontend build
for 5 positive-guard lines; old loop byte-exact without guard. One Wine full pair passes
every full state/clock/image/OAM/dispatch/five audio streams, samples
[105336,105336,12672,12672,105260], exact CLI counts 48/48/48. 97 guards/code/preimage
exact, helper 0 paths. No navigation/RAM-task-ROM/cartridge tampering or old Bank 12/F09
acceptance repeated. Declared cycle children 01/02/04/CLI 05/ordinary 05 closed, total
migration still open; distinct constructor ordinary partial. Linux/USA/native Windows/
new 32:9 pending, no measurements.
[Scene and evidence](../rom_data_gameplay.md#ordinary-european-task-cycle-scene).


**ROMDATA-01-BANK-BUS-HOST-RESIDUE: EUR/Wine census completed, 90B sources open
(2026-10-27).** Optional SiteCapture::start_all mode preserves empty/single filters
and old keys. First warning-free Windows feature build/single Wine run: new probes and
one decoded Bank 12 arm 3660–3672 with existing normal input, no old full pair repeated.
7430 reads/7532 sites, 497 with interpreter context/7035 None; 7083B image, 1434B
outside 68 owners. All 3683B of old NativeAPI included; 992B already classified,
80 literals/320B and 8 dispatch cells/32B verified now: 1344B code support, no global
127 cap from branch with helper. 90B data remain: 18 program/12 directory/60 OBJ lists,
root 4165A0+4114FC via 5E6F4/8, readers 2C5E0/2C60C with tick-phase-selector u16,
distinct from already typed u8 banks. Contract and model of four authenticated prefixes
completed below; management/consumption remain open. No new resource/revision from
census alone. Twelve typed deltas 21–32 read by interpreter LDRB 18E2A explain the
limit of old NativeAPI. 95 guards/code/default preimage exact, helper 0 paths. e76-d75/
USA 8-7 unchanged, user Atlas preserved; Linux/USA/native Windows/new 32:9 pending,
no measurements/fixture.
[Inventory and new sources](../rom_data_runtime.md#bank-bus-and-selected-host-rom-inventory).


**ROMDATA-01-U16-TASK-OBJ-BANK / ROMDATA-02-U16-TASK-OBJ-MODEL completed within
selected EUR prefix scope (2026-10-27).** Four 2C5E0 producers authenticate roots
41634C/411478, 41647C/4114AC, 4165A0/4114FC, 4166C4/41154C: 50 lists 140 OBJ/
7 seq 49 frames/65 spans 1378B disjoint from 68 owners. Relative-index/payload format
verified, padding excluded; seven unreferenced lists and three empty seq remain data,
not reachability evidence/invented holds. u16 counter-phase-selector, low u8 repeat,
return old list before increment, 255→256/65535→0, clamp/repeat/done latch; no cap
before lookup or global archive boundary. Lossless strict JSON model reuses pure
fields, not u8 timing. First warning-free core build: Wine passes 23 guards/1378B
projection then fails test expectation on ctor return r0. Test-only correction; second
warning-free core build/remaining-only filter passes 23 ctor/93 update/6 producer,
RAM-callee/alias/edges. First filter not declared PASS and 23 guards not repeated.
88 guards/199 core sources exact; helper 0 paths. Core dev serde_json preserves parallel
frontend changes without compiling them. Revision e76-d75/USA 8-7 unchanged: management/
consumption/CLI/ordinary for these banks still open; Linux/USA/native Windows/new 32:9
pending, no measurements/fixture.
[Contract and model](../rom_data_obj.md#european-obj-banks-with-u16-task-counters).


**ROMDATA-02/04-U16-TASK-OBJ-RUNTIME completed in EUR/Wine phases (2026-10-27).**
e77-d76 adds graphics.word-task-obj-animations/file word-task-obj-animations:
70 files 69 resources/all 68 previous records-files identical, 65 spans 1378B/
4 banks/50 lists/140 OBJ/7 seq 49 frames. Strict binding/transactional attach; shared
pure projection retains old owner body and gate, new 411/416 gate and host lookup.
Indexing/counters/guest old-list return and ABI/states/cache unchanged. First warning-
free Windows build; Wine passes publication/interrupted file/9 attach guards/2 full-set,
27 original prefixes 7 ctor 20 update/202B compiled/full state-clock, 1378B/6 classes/
217 mixed reads/66B raw gap/old owner/Load-slot Undo without file/DMA3. Then process
without terminal result: no full PASS claimed, receipts retained, no passed phase
repeated. Remaining build compiled clean but rejected by source guard for 2 parallel
DBG changes; stable snapshot. Host test draft isolated per frame; test-only private
field compilation error and OAM/register origin corrected, product unchanged. Final
warning-free build/single host-only Wine run passes 50 lists/140 OBJ/95 host reads with
65 ROM spans altered, accepted OAM per list and equal raw/typed descriptors. 165 guards/
496 snapshot sources/current ROM inputs exact; helper retires only expected interrupted
copy, retains dataset/logs/preimages/executables. Real CLI subsequently verified below;
ordinary subsequently verified in Bank window below, no old model/census/scene acceptance
repeated. USA e8-d7/Linux/native Windows/new 32:9 pending; no measurements or new
fixture/navigation.
[Runtime contract and evidence](../rom_data_obj.md#managed-runtime-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-CLI completed EUR/Wine (2026-10-27).**
One warning-free Windows production build from accepted runtime snapshot copy;
CLI candidate b3bfe6f425923ce461793771fe6c84b86eb5da7dda69b3b9c7e0f5208eab51ea,
canonical executable hash/mtime restored. One Wine script/3 real commands prepare
created/reused/runc 0 pass: 70 generation files identical to runtime/69 registered
resources with exact hashes/72 managed paths. Reuse and startup without hash mtime
writes; original cache loaded, 0 guest frames/dispatch/fallback and 9 new owner fields
all 0. 154 guards/496 copied sources/current ROM inputs exact/helper 0 paths. Dataset
27463ce4f7ffccc3d0b9c8ab5ee78160a3e1c844ed80f68768f9adea87cb7757.
CLI child closed, separate positive ordinary verified below; no old acceptance or
fixture/navigation repeated. Linux/USA/native Windows/new 32:9 pending, no measurements/
Android/package/commit/push. Parallel DBG changes preserved.
[Real CLI and evidence](../rom_data_obj.md#actual-cli-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-ORDINARY completed EUR/Wine (2026-10-27).**
Bank window 3660→3672 chosen from already accepted census/root 4165A0+4114FC,
byte-identical copies of normal snapshot/inputs, no navigation/new checkpoint/
RAM-task-ROM/cartridge tampering. One existing e77 CLI passes positive use:
292 reads/346B = [44 index 20 count 240 attr 2 seq header 20 frame-list 20 duration]/
20 host, 72 managed paths/hash mtime unchanged. Optional 5-line guard on new owner in
full-pair test; old validator/loop byte-exact with it removed. First warning-free Windows
snapshot frontend build/single Wine full pair first PASS: all 12 full states/clock/
central-wide images/OAM/dispatch and 5 audio streams identical, 9 exact CLI counters.
Dispatch [19962,85770], samples [26334,26334,3168,3168,26258]. 94 guards/496 snapshot
sources/current ROM inputs exact/helper 0 paths. No previous model/runtime/scene/census
acceptance repeated or new production build. Declared source/model/management-consumption/
CLI/ordinary children for this family closed; no evidence for all 4 banks/50 lists/
7 unreferenced lists/empty seq update/indirect index/full archive/full game. Distinct
constructor ordinary partial; whole TODO and Linux/USA/native Windows/new 32:9 open,
no measurements/Android/package/commit/push, parallel DBG preserved.
[Scene and evidence](../rom_data_obj.md#ordinary-european-u16-task-obj-bank-scene).


**ROMDATA-01/05-CONSTRUCTOR-FLAG-REACHABILITY closed for EUR/Wine flag
(2026-10-28); ordinary animation still partial.** Twenty-one source windows link
95EA0/wrapper 9A594/global 0200F4A8/state DC/root 960F0: Move 27/A after B cancel/
validation 9D6C8/different destination → count 1A4/28-29, signed count >1 → enable
99F18. Other 3A→99300 and A5544 case 6→A57BE/case 1→A570A are distinct paths, not
global liveness. Two authenticated normal snapshots read only to choose A-only,
without old Right: one reused Wine observer 160 closes fifth negative test, table 0/
phases 0/lists unused; no full pair/positive fixture or new build. Endpoint state 25/
count 3 does not imply invalid destination. New missing flag/slot criterion during
transition: one existing readonly 64-frame CLI captures 448 RAM-image files, normal
Move and flag 20 active 12186–12230/45 postframes; both slots 394/3A4 always inactive/
counter-phase 0/table 0. Ordinary flag reachability proved, slot creation/callback
still to authenticate; no PC/intraframe attribution. 587 guards/496 frozen sources/
21 window hashes exact, originals/e77 dataset 72 paths unchanged/helper 0 paths. No old
acceptance repeated or RAM/ROM/cart tampered with; static disassembler is not a Linux
game test. New full pair only after positive use of 4 phases/4 lists guest-host; no
more blind Move attempts. Linux/USA/native Windows/new 32:9/full TODO open, no
measurements/Android/package/commit/push.
[Flag and slots](../rom_data_obj.md#ordinary-constructor-flag-activation-and-inactive-slots).


**ROMDATA-01/05-CONSTRUCTOR-SLOT-ACTIVATION closed for source gate and map exclusion
(2026-10-28); ordinary still partial.** Eight new windows authenticate 9DB0C→owner+10/
pair [owner,9B11D]→9E300/145C10→9B11C(actor,owner); same slots 394/3A4 as counter
and 9DBA4. 9DAD4 is a distinct earlier helper. Beyond movement and flag 20, normal
branch 9B460 requires unit FD2, signed XY 0..15, valid cell/flags 02/S8 height > actor
S16 height >>4, anim 36!=40; entry-exit bit 8000, skip mask 10000, free slot. Live
cell root 02007F10, width+8/base+4, stride 2 height-flags, no new ROM asset. Readonly
inspection of existing 64 RAM dumps: FD2/actor flags 0/anim 0, three traversed cells
height 2/flags 0; all 256 declared cells without 02, stable 512B hash. This unchanged
map offers no destinations for the gate; no further blind Move or repeat of old five
negatives/flag capture. 87 guards/8 window hashes exact/helper 0 paths, no new gameplay/
build/Wine-Linux test/capture/schema/runtime/ABI/cache/save. Next criterion: readonly
inventory of existing normal EUR snapshots, suitable map and actor and provenance; no
synthetic states or recreation of retired copies. Full pair only after positive use of
4 phases/4 lists guest-host; ordinary/Linux/USA/native Windows/new 32:9/full TODO open,
no measurements/Android/package/commit/push.
[Cell and slot gate](../rom_data_obj.md#constructor-slot-creation-depends-on-live-cell-flags-and-height).


**ROMDATA-05-CONSTRUCTOR-ELIGIBLE-MAP-INVENTORY closed negative (2026-10-28).**
New readonly criterion on 81 normal EUR snapshots/already verified provenance: all
hash/payload/ROM/16x16 domains valid, only 2 map projections 512B (1 F09/80 river),
zero 02 cells/candidates. Old 606 alternate sequence scan not repeated. 88 snapshot-
report-parser-contract-output guards exact; no build/gameplay/Wine-Linux test/capture/
checkpoint/tampering. Terminal negative bound on 2 retained maps, not all maps/future
script; do not repeat same 81 states or Move here. Parent ordinary still partial; next
source lead cell-flag producer 02007F10/terrain ROM before choosing eligible map. Full
pair only after positive use of 4 phases/4 lists guest-host. Linux/USA/native Windows/
new 32:9/full TODO open, no measurements/deployment.
[Retained maps](../rom_data_obj.md#retained-normal-maps-have-no-eligible-constructor-animation-cells).


**ROMDATA-01-CELL-FLAG-WRITERS closed for EUR census/candidate seeds (2026-10-28).**
Old TERRAIN-MAP-SOURCE from the 24th already closed/352 BG packages reused, no new
archive. New direct census 1DE98 no Thumb BL/aligned pointer (not dead code/indirect),
sole BL 1DD68 to 1D798 reads flags 02. New semantic query of authenticated e77 BG
dataset: 113 auxiliary programs/162 links/zero fill-parent-child/ordered u16 writes,
25 programs and 44 IDs initialize 02; flags 0/1/2/3/4/8/9. No decoder/resource/
revision or old codec/loader acceptance repeated. 24 guards/4 source windows exact/
helper 0 paths, no build/gameplay/Wine-Linux test/capture/checkpoint. Private quote
typo corrected before execution; early cleanup rejects unit without result, then passes
after summary, product unchanged. Asset candidates do not prove normal battle/place
name; next actual map selection script/caller → eligible ID/normal reachable battle.
81 snapshots/5 Move bound/flag capture remain closed. Ordinary 4 phases/4 lists
guest-host/full pair and Linux/USA/native Windows/new 32:9/full TODO open, no
measurements/deployment.
[Seeds and sources](../rom_data_maps.md#constructor-animation-cell-flag-seeds).


**ROMDATA-01-CONSTRUCTOR-ELIGIBLE-MAP-SELECTION: source link closed,
reachable battle still partial (2026-10-28).** 253C8 state 10/2549C reads CCF00(7)
→02002037 low u8→1B270/1B689 loader/state 11; 2540C is dispatch table. Opcode 12
advance 2/callback 126831: byte 1 nonzero→CCF18 variable 7; zero→70C8CC+40*U16(02002170)
+2→variable 7, then sets 0200F390=10. 51 CCF18 candidates, two literal 7 arms
authenticated, no all-writer claim. Typed script query 204 instructions 56 default/
148 literal: 19 eligible literals/10 map args [31,33,34,73,75,89,91,109,115,129];
exact references private, not place names/ordinary reachability or event key = active
index. Two retained normal examples active 35/197 map args 99/85→descriptor 98/84/
consistent default/ineligible, not a repeated 81 scan. 37 guards 9 windows exact, no
build/gameplay/Wine-Linux test/capture/checkpoint/runtime/schema/ABI/cache/save. Next
active-index producer/40B record 70C8CC/map byte 2→normal quest/event, no forced ID/
guessed navigation. Positive 4 phases/4 lists guest-host/full pair and whole TODO/
Linux/USA/native Windows/32:9 open, no measurements/deployment.
[Script selection and limits](../rom_data_maps.md#script-map-selection-and-remaining-battle-reachability).


**ROMDATA-01-ACTIVE-ENCOUNTER-RECORDS: producer and conditional join closed EUR
(2026-10-28), complete catalog and ordinary battle still partial.** Setter 9CF0 writes
low16 r0/r1 to 02002170/172; two direct BL 9D2C/1273A8, no aligned Thumb pointer,
without claiming all writers. 9D04 chooses primary/alternate from row+2 or 71D1EC+
12*low8(arg1), according to row+4. 63944 links live selector 219E, record 70B713B3C+45
and 12B row 71C760; 127388 searches event key on rows 255..1, first descending match,
duplicates distinct. New query over caller u8 domain: 208 direct-primary rows, 54 with
seed 02, 44 also with default-map script. Does not prove mission availability, names
or 40B catalog extent; max reference 442 and separate constant 300 are not limits.
Retained normal pub selector 0/row 0/map arg 11 ineligible, readonly. 42 guards/
23 windows exact; private Python import corrected after stop before output; originals
unchanged. No build/gameplay/Wine-Linux test/capture/checkpoint/new resource. Next
criterion: eligible mission/event actually available from normal checkpoint and
original UI. No forced ID/RAM or further blind Move; 4 phases/4 lists guest-host and
ordinary full pair remain open. Old acceptances not repeated; Linux/USA/native Windows/
new 32:9/full TODO pending, no measurements or deployment.
[Producer and evidence](../rom_data_maps.md#active-index-producer-and-event-record-links).


**ROMDATA-05-ELIGIBLE-QUEST-LOCATOR closed EUR/Wine (2026-10-28): eligible mission
present in normal menu.** Observer 309ba31c/e77 reused, no build: a known 620-frame
path retains missing-menu endpoint 14969; then 300 frames 4 Down+A open White Flowers
detail 15269. Old 620 scene not repeated as acceptance; new checkpoint/selectors
criterion. Originals and copies exact, no RAM/task/ROM/cart edit. Live F44C 02024D5C/
count+1B14/pointers+20+4*ordinal, low 10 mission ID from 62126..5E. New 14-entry
query finds 2 direct-primary/default-script/cell 02 candidates: 58/72. Fifth row and
visible detail confirm 58; F448 0202389C +1450→02002268/ID 58 with 60208..28. Event
58/primary 315/map arg 69/descriptor 68 eligible; Giza Plains in description, but
acceptance and battle not yet reached. 130 guards/496 frozen sources/23 windows exact/
helper 0 paths. Candidate layout 031C/03D0 03001028 rejected because not linked to live
menu. Both Wine workers pass, constructor counters 0 in pub expected, not positive
proof/full pair. Next normal confirmation and verify acceptance/battle entry, then
actor/cells before guided Move. 4 phases/4 lists guest-host/full pair, complete 40B
catalog, Linux/USA/native Windows/new 32:9/full TODO pending; old acceptances closed,
no measurements/deployment.
[Normal mission and evidence](../rom_data_maps.md#normal-pub-menu-identifies-an-eligible-quest).


**ROMDATA-05-WHITE-FLOWERS-BATTLE-NAVIGATION partial EUR/Wine (2026-10-28);
normal acceptance closed.** Exact copy of ID 58 detail→loadout OK→cost/objective→
conditions Yes; 300 gil payment/funds 7709→7409 and subsequent Fee:Paid prove active
mission. Entry initially read as QuitPub is QuitMission: only A:Cancel detail opened,
no cancellation confirmation; B/B/Down/A on Leave leads to map. 27 normal captures/
4840 frames, existing observer 309ba31c/e77, no build/manual RAM-task-ROM/cart write.
All Wine workers pass, table 0, not 27 equivalence acceptances/no full pair. World
labels Cyril/LutiaPass/Nubswood/Sprohm, central patch excluded; camera scroll/overshoot
documented. Giza Plains not proved reachable/absent/not placed. 337 guards/496 sources/
17 exploratory window hashes exact. 92524 init/stale task/031C 03001028 do not prove
world dispatch; 165E8 catalog index 725 Giza and base 718/list 3EE60 are only leads,
not coordinates. Partial unit retains all diagnostic checkpoints, no fixture retirement.
Next source selector/placed locations from retained states, no further blind cursor or
27 bound/5 Move/81 map/old acceptances repeated. Battle/actor/4 phases 4 lists guest-host/
full pair and full TODO/Linux/USA/native Windows/new 32:9 remain open, no measurements
or deployment.
[Acceptance and limits](../rom_data_maps.md#normal-white-flowers-acceptance-and-open-battle-entry).


**ROMDATA-01-WORLD-LOCATION-SELECTOR closed EUR/Wine (2026-10-28).** Corrected
inspection assumptions: 254B2→6B40, count task byte+2 not word+4; literal 2810/0300
is 03002810, not 03001028. Optional passive test: one warning-free Windows build
from 497-source snapshot, one neutral Wine frame 18829→18830/2232 reads 2276 sites
323 scoped, actual RAM matches parser. Parent[03002810]=0200F3A4; nodes[parent+67C]
=08395CA8. 375F8 loop 30/stride 20, byte 0/1*8+8 center, conditional hitbox and
1-pixel snap; 37D6A..70 cursor view+6/+8, 37A44 display Y=world Y−scroll C.
385CC..38668 save base+B0+12*i enabled byte 1 bit 0/byte 3 one-based selector→node
(selector−1). Giza index 7/catalog 725/node 20/source 395E24 is placed at 112,296.
Readonly two southern endpoints: 21 cursor 128,287, 22 cursor 128,330; old 27 bound
not repeated. 152 guards/497 sources/32 windows exact/helper 0 paths. New ROM family
30 nodes 20B: only 2 coordinate B classified, 18B/schema/model/extraction/runtime
open. Next short Left from 21, verify Giza label before A; no forced coordinates/RAM/
map ID. Battle and 4 phases 4 lists guest-host/full pair/full TODO/Linux/USA/native
Windows/new 32:9 open, no measurements/deployment/old acceptances repeated.
[Selector and evidence](../rom_data_maps.md#world-selector-and-placed-region-coordinates).


**ROMDATA-05-GIZA-BATTLE-ENTRY closed EUR/Wine (2026-10-28).** Guided short Left
leads to Giza Plains label; A travel/menu White Flowers/A deployment. Primary 315/
map 69/descriptor 68, live grid 512B e8427b5e.. matches seed packet 56/20 cells 02.
Marsh deployed with position/direction confirmed; START To Battle/Yes/dialogue leads
to Move. Actor FD2 (6,5), Left/Down/Down actual cursor (5,7), normal A activates
4 phases/4 lists guest-host: 75 frames/22 reads 88B = 44 threshold 44 pointer.
Checkpoint 22713 + suffix 11 retained, not yet full pair. 16 Wine captures/3875 frames,
existing 309ba31c/e77/no build; 221 guards 496 sources exact/helper 0 paths. Keys
only on copies, no manual guest RAM/task/ROM/cart save modification. First STARTs before
facing confirmation inert/B returns to placement: corrected and closed, do not repeat.
Next single positive CLI 11 and full state-clock-images-OAM-dispatch-audio comparison
from suffix, without repeating discovery/navigation. 40B catalogs/world nodes 18B/full
TODO/Linux/USA/native Windows/new 32:9 pending; no measurements/deployment/old
acceptances repeated.
[Entry and positive evidence](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE completed EUR/Wine
(2026-10-28).** Exact copy of new normal Giza checkpoint 22713/suffix 11→22724.
One existing production CLI b3bfe6/e77 reuses dataset/cache and passes 22 reads 88B/
44 threshold 44 pointer/all 4 phases. One Wine full pair 309ba31c passes each of
11 state-clock/classic-central-extended images/OAM/dispatch/five audio streams and all
4 lists guest-host; counters match CLI, dispatch 57833/223470, audio
24138/24138/2904/2904/23966. 91 guards 496 sources/72 managed paths exact/helper 0;
no build/preparation/navigation/discovery/old acceptances repeated, guest edit/cart
write/measurements. Constructor ordinary case closed, not coverage of all actors/maps/
full game. Linux/USA/native Windows/new 32:9/full TODO remain pending; 40B catalogs and
new world nodes 18B to complete. u16/DBG work preserved, no deployment.
[Full ordinary evidence](../rom_data_obj.md#ordinary-constructor-animation-positive-giza-scene).


**ROMDATA-01-WORLD-NODE-RECORDS EUR source projection closed (2026-10-28).**
395CA8..395F00/30 records 20B/600B, count from already proven 375F8 loop; no runtime
cap 30 inferred (low u8 getter/work array 32). Producers 31488/32BE6 write root
[parent 03002810+67C]. Bytes 0/1 coordinates; four slots 2/6/10/14 path ID/neighbor/
opaque 2B plus opaque 2B tail: 300 semantic B/300 opaque B retained. 120 slots/
90 active links/30 path 0/45 paths each in 2 directions. Valid node 0 distinct from
path 0; 4DAD0 matches neighbor before path (22 target 0→0), 46EB0 same node or path!=0/
neighbor. 4DB80 graph copies 4 slots/skips path 0/gate 8-27; reference 397018+(path−1)*8
external, road catalog/payload open. 25 windows (21 new 4 reused)/44 pins/42 guards
pass, 600B disjoint from 49746 spans 69 owners. New criterion on existing trace:
35 reads 42 sites coordinates only, no new edge coverage or repeated audit. No build/
game/test/fixture/guest edit/measurement. Next typed lossless model/source plan and
one targeted Wine oracle; extraction/runtime/CLI/ordinary, Linux/USA/native Windows/
new 32:9/full TODO pending.
[Schema and readers](../rom_data_maps.md#world-node-record-source-projection).


**ROMDATA-02-WORLD-NODE-MODEL completed EUR/Wine (2026-10-28).** world_nodes
catalog authenticates exact image/6 windows/600B hash. Model 30 ordered nodes,
4 Link path/neighbor/opaque 2B + opaque 2B tail, lossless source plan 1 span 600B;
strict JSON/shape distinct from authenticity, no inferred 45/29 cap/opaque zero.
One warning-free Windows core-only build/6c2ef15a.., one Wine filter passes 23 guards/
566 original calls with independent return values/read addresses, 300 semantic B
observed/300 opaque B preserved unread/16 byte aliases/24 outside projection. Callee/
SP/RAM outside stack/VRAM/OAM/palette checked, not native/full state-clock/BFS/
ordinary. 24 guards 215 frozen sources/helper 0 exact. No gameplay/frontend build/
old oracles repeated; e77-d76 and schema 1 unchanged. Next managed publication/typed
guest consumption/real CLI/retained world scene. Linux/USA/native Windows/new 32:9/
full TODO pending, no measurements/deployment.
[Model and proof](../rom_data_maps.md#typed-world-nodes-and-original-reader-proof).


**ROMDATA-03/04-WORLD-NODE-RUNTIME completed EUR/Wine (2026-10-28).** e78-d77
publishes world-nodes.json/gameplay.world-nodes/index 69: 71 files 70 resources, all
69 previous records/files exact, USA e8-d7 unchanged. 600B binding/7 statistics
fields, coordinates/path/neighbor/opaque read from model, prior/raw/edge composition
without cap on guest input or replacement of placement/pathfinder/ABI/save/cache.
One Wine filter passes 12 attach/2 full-set/interrupted publication, 15 native full
state-clock prefixes/32B compiled 124 instructions/600B raw altered after Load, alias/
mirror/outside 30-31/equal 255/root RAM. Bus all 600B/4 classes 30 nodes/width/edges/
prior owner, Load/Undo/unavailable files and 3 DMA/CpuSet/CpuFastSet full state-clock
pass. First 2 builds rejected by stale core cache: dep-info points to old copy, touching
new copy ineffective; moved only specific fingerprint. Third warning-free build/
fresh=false/2cf1d882.. from 502 stable sources. 200 guards/502 sources + 215 model
preimages exact; helper retires only interrupted copy, accepted dataset/executable
retained. Real CLI/ordinary scene still to do. No new gameplay/old model or proof
repeated/measurement; Linux/USA/native Windows/new 32:9/road 397018/full TODO pending.
Parallel DBG/u16 work preserved.
[Consumption and evidence](../rom_data_maps.md#managed-world-nodes-and-typed-guest-reads).


**ROMDATA-04-WORLD-NODE-CLI completed EUR/Wine (2026-10-28); ordinary partial.**
Warning-free Windows production build/e0491cdc.. from 502 stable sources; canonical
hash/mtime restored. Three Wine CLI exit 0 prepare created/reused/runc normal 16,
71 files identical to runtime/70 ID hashes exact/73 managed paths unchanged on reuse-
startup, cache 28859 blocks loaded. Normal Giza 01/02 input copies, 19049→19065:
402 reads 402B coordinates/22 nodes; path/neighbor/opaque 0, no full pair. Private
matcher expected relative frame 16, CLI shows absolute 19065: corrected offline after
all 3 exit 0/guards, no repeat. Bound 16 without path/neighbor closed, next source
callers/justified normal state before new observation. 169 guards/502 sources exact/
helper 0. No new fixture/navigation/previous proof/measurement. Ordinary 3 classes/
full pair/road 397018/full TODO/Linux/USA/native Windows/new 32:9 pending; DBG/u16 work
preserved.
[CLI and ordinary limits](../rom_data_maps.md#actual-world-node-cli-and-ordinary-coordinate-use).


**ROMDATA-01-WORLD-NODE-ORDINARY-CALLERS and ROMDATA-05-WORLD-NODE-TRAVEL-ORDINARY
completed EUR/Wine (2026-10-28).** New direct BL query/10 windows 27 guards links
3B012→4DB1C→4DB80 to view 1A (dest−1) different from parent 7BB; output 7C1.
47034 progress!=FF →4DAD0; entity parent 7B8+52*i. Two existing normal checkpoints
17/selected 20→19/progress FF and entity 1 node 0→1 motivate bound 360, not PC/frame
or all-caller inference. One positive CLI 360 and one full pair 360 pass 6841B =
4820 coordinates 864 path 1157 neighbor/29 nodes, all state-clock/classic-central-
extended images/OAM/dispatch/five audio and CLI counters identical. Opaque 0 distinct
from targeted tests. Dispatch 2006894/3620371, audio 790020/790020/95040/95040/789942.
One warning-free Windows build/dabb1ac6.. from 502 sources with optional 5-line guard;
old validator exact without guard, runtime/core unchanged. 103 guards/502 sources/
27 caller/169 CLI exact/helper 0. First 16 path/neighbor negative remains closed; no
new fixture/navigation/guest edit/old proof repeated/measurements. Ordinary case closed,
not all nodes/maps/opaque ordinary/full pathfinder. Linux/USA/native Windows/new 32:9/
road 397018/full TODO pending, DBG/u16 work preserved.
[Ordinary evidence and gates](../rom_data_maps.md#ordinary-world-node-travel-and-caller-gates).


**ROMDATA-01-WORLD-ROAD-PATHS EUR source projection completed (2026-10-28).**
45 path IDs from 90 active links/30 nodes select 45 headers 8B 397018..397180:
u8 count, opaque 3B, u32 ROM pointer. 45 distinct contiguous payloads
395F00..397018/2188 signed i8 XY pairs/4376B, 46 spans 4736B = 4601 semantic
135 opaque. Root store parent 680 reused; 4DC cost byte 0, 4DD0C flag/index/low u8
step/no cap 45, outside step count output zero 32 without pointer/sample load,
reverse order/no negation, two packed i16/return pointer. 4DD70 flag current>target/
4DD84 min node; caller adds node XY*8 to offsets, 46B64 differences max 5 samples/
facing not full ABI. 3 root literals/3 point calls/1 dir/1 min candidate query,
not global absence of indirect calls. 5 endpoints 1 pixel short 4/6/8/10/12 retained;
zero points absent/count 32..85 not constraints. 6 windows 60 pins 17 guards/
70 owners 49747 spans disjoint/helper 0. No build/game/test/fixture/old proof
repeated/measurement. Next lossless model/source plan and targeted Wine oracle;
extraction/runtime/CLI/ordinary/global catalog/opaque meaning/Linux/USA/native
Windows/new 32:9/full TODO pending. DBG/u16 work preserved.
[Road contract](../rom_data_maps.md#world-road-path-source-contract).


**ROMDATA-02-WORLD-ROAD-MODEL completed EUR/Wine (2026-10-28).** Standalone
world_roads model: 45 ordered headers with u8 count/opaque 3/encoded pointer,
2.188 signed XY pairs and shared source plan 46 spans/4.736 B. Strict JSON,
consistent partition and separate authenticity; five short endpoints, possible
zero sample and structural count 0/255 retained without inferred 32..85 cap.
One warning-free Windows core-only build/de9d1e11.. and one Wine filter pass
29 guards/2.537 original calls: 2.188 forward points, 135 reverse, 180 exits,
6 u8 aliases, 6 external headers and 22 order helpers. Exact outputs/returns/read
sequences; all 4.601 semantic B read, 135 opaque B reconstructed/unread.
32 preservation guards/504 build files (218 core) exact/helper 0. No ordinary scene/
fixture/frontend build/measurement or already closed node proof repeated. Schema 1/
e78-d77 unchanged. Next managed extraction/typed guest reads, real CLI and one
positive ordinary scene. Linux/USA/native Windows/new 32:9, global catalog extension
and full TODO pending.
[Model and proof](../rom_data_maps.md#typed-sampled-world-roads-and-original-reader-proof).


**ROMDATA-03-04-WORLD-ROAD-RUNTIME completed EUR/Wine (2026-10-28).**
e79-d78/index 70/gameplay.world-roads/world-roads.json: 72 files 71 resources, all
70 previous records/files exact, USA 8-7/schema 1 unchanged. Immutable count/pointer/
sample/opaque owner from 46 spans 4736B; composes world node at left boundary and
raw right/header 45..255, no movement/facing/pathfinding algorithm replaced. Two
warning-free Windows builds/accepted 52be2981.. from 506 stable files; first filter
closes publication/attach, then fails native case 3 due to omitted page 396. Page gate
fix and remaining native/bus/lifecycle filter without republishing: 12 attach guards/
2 full-set, 12 original/interpreter/native full state-clock prefixes, 67 compiled
instructions/44B explicit, 4736B sources poisoned after Load; all 4736B/4 classes
45 paths/widths/mirrors/edges/prior owner/Load Undo and DMA3 PASS. 221 guards 506
source hashes + 504 model preimages exact; helper retires only interrupted copy. No
scene/model/nodes/fixture repeated, measurement or ABI change. Real CLI and one
positive ordinary scene next; Linux/USA/native Windows/new 32:9/global extent/full
TODO pending.
[Consumption and evidence](../rom_data_maps.md#managed-world-roads-and-typed-guest-reads).


**ROMDATA-04-WORLD-ROAD-CLI completed EUR/Wine (2026-10-28).** Warning-free Windows
production build/1a86d341.. from 506 accepted sources, canonical hash mtime restored.
Three Wine commands prepare created/reused/runc 360: 72 files exact to runtime/71 ID
hashes/74 paths unchanged hash mtime, old 28859 cache blocks loaded. Normal Giza
01 state/02 input copies 19049→19409: 2976 reads/5067B = 885 count/2788 pointer/
1394 sample/0 opaque/40 paths, 3 semantic classes positive. 180 guards 506 hashes/
helper 0; no new scene, navigation, measurement or already closed proof repeated.
Next one ordinary full pair with identical CLI counts; not all 45 payloads/maps/
opaque ordinary/global extent. Linux/USA/native Windows/new 32:9/full TODO remain pending.
[CLI and consumption](../rom_data_maps.md#actual-sampled-road-cli-and-ordinary-consumption).


**ROMDATA-05-WORLD-ROAD-TRAVEL-ORDINARY completed EUR/Wine (2026-10-28).**
One normal Giza 01/02 Wine full pair 360 with CLI dataset e79-d78, 19049→19409:
all 360 state-clock/classic-central-wide/OAM/dispatch/five audio exact. Seven counts
match CLI: 2976 reads 5067B = 885 count 2788 pointer 1394 sample 0 opaque, 40 paths
(any field, not 40 payloads). Optional 5-line guard preserves old validator byte exact;
one warning-free Windows build/4c453f07.. from 506 stable files. 100 guards 506 hashes
+ 180 CLI guards exact/helper 0; no new scene/fixture/navigation/measurement or already
closed source/model/runtime/node proof repeated. Road ordinary case closed; all 45
payloads/maps/opaque ordinary/full ABI/global extent, Linux/USA/native Windows/new 32:9
and full TODO still pending.
[Ordinary equivalence](../rom_data_maps.md#ordinary-sampled-road-travel-equivalence).


**ROMDATA-01-ENCOUNTER-CATALOG-READERS EUR contract completed (2026-10-28).**
Declared physical catalog 442 headers 40B 1..442: origin 70C8CC, first 70C8F4 from
two independent roots, next 710E04 independent 28B family. 17680B, no assumption
that max 442 is count/nor constant 300/nor guest cap. 442 contiguous template links
2850 records 48B/136800B corroborate partition. 17 semantic fields/33B and 7 opaque
per header; original low 16 indices without 442 guard. Index 0 is coordinate prefix:
last 16B payload of header 442, no invented valid header. Header 29+4 pointer to
70CD58 already owned by constructor-alternates: dependency/consistency/skip 4B,
2 new spans 17676B, 14582 semantic/3094 opaque disjoint from 71 owners 49793 spans.
Already owned 192B template retained; unit/coordinate payload separate, not migrated.
27 loaders from cached root/40 windows (39 new 1 reused)/54 pins 68 guards/helper 0
PASS. No build/game/test/fixture/previous proof repeated. Next lossless 442-header
model/shared pointer + one Wine oracle; extraction/runtime/CLI/ordinary, all indirect/
global extent/Linux/USA/native Windows/new 32:9/full TODO pending, no measurement.
[Headers and ownership](../rom_data_maps.md#european-encounter-headers-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-HEADERS-MODEL completed EUR/Wine (2026-10-28).**
Lossless encounter_headers model 442 headers 1..442: 17 fields/opaque 7, 17680B full;
2 spans 17676B exclude 70CD58 len 4 from constructor-alternates owner. Strict JSON/
shape separate from authenticity/dependency consistency; modified shared pointer leaves
spans unchanged but is rejected. Low u16 alias/None 0-443..65535/prior raw, no inferred
count cap/null pointer/enum/opaque zero. One warning-free Windows core-only build
5855af92.. and one Wine filter pass 29 guards/7523 probes: 7072 single-load prefixes
and 451 complete C0034 calls (442 + 6 aliases + 3 outside), all 14586 semantic B incl
4 shared observed, 3094 opaque B reconstructed/unread. Reg/flags/PC prefix, callee/
SP/LR gate, RAM outside stack 4/VRAM/OAM/palette checked; not full ABI/importer/native
clock/ordinary. 33 guards 508 files (221 core) exact/helper 0, e79-d78 unchanged, no
old oracles/scene/fixture/measurements. Next managed/typed bus/source page 70C..710/
dependency, then CLI/ordinary; nested payload/all callers/global/Linux/USA/native
Windows/new 32:9/full TODO still pending.
[Model and proof](../rom_data_maps.md#typed-encounter-headers-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-HEADERS-RUNTIME completed EUR/Wine (2026-10-28).**
e80-d79/index 71/encounter-headers.json: 73 files 72 resources, all 71 previous exact,
constructor-alternates dependency and shared-pointer consistency. Owner 2 spans 17676B
excludes 4 old B, source pages 70C..710, 8 classes/442 header mask/11 stats; guest
algorithms/indices/ABI/cache/save unchanged, USA 8-7/schema 1. One warning-free Windows
build 1c4135f5.. from 510 stable files, one Wine filter: 14 attach guards/3 full-set/
failed owner-state, 26 original/interpreter/native full state-clock prefixes with 17680B
raw poison, 31 compiled instructions/37 new B + 4 old B. All 17676 new + 4 shared B/
8 classes/442 headers/widths/mirrors/edges/prior owner/Load Undo/DMA3 PASS. 206 guards
510 hashes + 508 model preimages exact/helper retires only interrupted. No scene/fixture/
measurement/7523 model proof/already closed node-road-constructor repeated. Next real
CLI/ordinary; nested payload/all callers/global/Linux/USA/native Windows/new 32:9/full
TODO pending.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-headers-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-HEADERS-CLI completed EUR/Wine (2026-10-28).** One warning-free
Windows production build/9d0a772d.. from 510 accepted sources; canonical hash mtime
restored. Three Wine commands prepare created/reused/runc 700 pass: 73 files exact to
runtime/72 ID hashes/75 paths no-write, 28859 cache blocks loaded. Normal Giza 02 menu/
03 input copies 19409→20109: 17 reads/41B, six positive semantic classes/1 header/
0 opaque. 184 guards 510 hashes/helper 0; no new navigation, checkpoint, measurement
or closed proof repeated. Next one full pair with identical CLI counts; not all 442
headers/payload/opaque ordinary/full ABI/global catalog. Linux/USA/native Windows/new
32:9/full TODO pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-header-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-HEADERS-BATTLE-ORDINARY completed EUR/Wine (2026-10-28).**
One normal full pair 700 Giza 02 menu/03 input 19409→20109 with CLI dataset e80-d79:
every full state/clock/classic-central-wide image/OAM/dispatch/5 audio passes. All 11
counts match CLI: 17 reads 41B = 12 count/1 map/4 template pointer/0 parameter/
1 placement count/20 position pointer/3 control/0 opaque, one header; six positive
classes. Parameter ordinary and opaque not observed, previous targeted evidence distinct.
One optional 5-line guard/old validator byte exact, one warning-free Windows build/
43fcc614.. from 510 sources and one Wine filter. 103 ordinary + 184 CLI guards/510
hashes each exact/helper 0. No new fixture/navigation/closed proof repeated or
measurement. Ordinary case closed; all 442 headers/payload/full ABI/global catalog,
Linux/USA/native Windows/new 32:9/full TODO remain pending.
[Equivalence](../rom_data_maps.md#ordinary-encounter-header-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-UNIT-TEMPLATES completed for the EUR source contract
(2026-10-28).** The 442 already authenticated header links partition 2.850 records
48B, 6EA048..70B6A8/136.800B; independent complete projection. New readers separate
selectors/arguments, five u16 equipment values, halfword 12, physical bitset 14..27
with dynamic index without 20B/160-bit cap, controls 28/29/flags 2A/2C/2D and opaque
2E..2F. All 14.250 equipment references within 376 items, selectors consistent with
metadata14/metadata52 owners; observed values are not caps. Old 192B constructor
6ECA78..6ECB38/records 225..228/header 29 excluded from new 136.608B/two spans;
full 48 coherence and header/constructor dependencies required, no duplication among
49.795 spans/72 owners. Eight windows 5 new 3 reused/39 pins/4 BL/41 guards pass,
no build/game/Wine-Linux test/fixture/old proof or census repeated/measurement. Next
strict model + one targeted Wine oracle; managed/runtime/CLI/ordinary/coordinate pool/
global/full ABI/Linux-USA/native Windows/new 32:9/full TODO pending.
[Contract](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-UNIT-TEMPLATE-MODEL completed EUR/Wine (2026-10-28).**
Strict lossless model 2.850 records 48B, full 136.800B/two spans 136.608 new,
192B previous constructor excluded from byte_at but compared in full. Shape/JSON/
header partition/shared coherence/item-metadata14-job references and source auth
separate; full u8/u16/opaque 2, no invented value caps or 160-bit guest cap. One
warning-free Windows core-only build/e0ae0d5c.. from 512 sources/224 core inputs,
one Wine filter passes 38 guards/2.100 selected original prefixes: 1.020 LDR,
1.080 bit including 120 indices beyond 20B. 1531 new/102 old/4 raw distinct bytes
observed, registers/addresses/values and all RAM/VRAM/OAM/palette/ROM exact;
independent full projection, not full ABI/semantic coverage of all 2.850 records.
34 guards 512 hashes + 41 source guards exact/helper 0; no new fixture, measurement
or closed proof repeated. Next e81-d80/index 72 publication + typed consumption,
then CLI/ordinary; Linux-USA/native Windows/new 32:9/coordinate pool/global/full ABI/
full TODO pending.
[Model and evidence](../rom_data_gameplay.md#typed-encounter-unit-templates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-UNIT-TEMPLATE-RUNTIME completed EUR/Wine (2026-10-28).**
e81-d80/index 72/encounter-unit-templates.json, 74 files 73 resources/all 72 previous
exact/five dependencies; 192B constructor remain with old owner. Immutable owner
136.608B/two spans, nine classes and record/header stats, local/global 6EA..70B routing.
One warning-free Windows build/3d4b2af2.. from 514 sources and one Wine filter pass
14 attach/7 full-set/transactional failures, 27 original-interpreter-native prefixes
with full state/clock and 136800B raw altered after Load, 25 new B/8 prior B compiled.
Every 136608 new B + 192 old/9 classes/2846 records 441 headers, widths/mirrors/
boundaries/Load Undo and DMA word 34202/CpuSet 68404 halfword/FastSet 34208 word
pass. 215 guards 514 hashes 512 model preimages exact/helper only interrupted. ABI/
algorithms unchanged; USA 8-7/schema 1. Next real CLI + ordinary; coordinate pool/all
callers/full ABI/global/Linux-USA/native Windows/new 32:9/full TODO pending, no
measurements.
[Consumption and evidence](../rom_data_gameplay.md#managed-encounter-unit-templates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-UNIT-TEMPLATE-CLI completed EUR/Wine (2026-10-28).**
One warning-free Windows production build/afeaef25.. from 514 accepted runtime sources,
canonical hash mtime restored. Three Wine commands prepare created/reused/runc 700
pass: 74 files exact to runtime/73 ID hashes/76 paths no-write, 28859 cache blocks.
Normal Giza 02 menu/03 A 700 input copies 19409→20109: 186 reads/211B, 5 records/
1 header, 8 positive classes and 1 unobserved. Positive selector/argument/equipment/
flag criterion satisfied; all 13 counts retained for subsequent full pair. 184 guards
514 hashes/helper 0, no new fixture/navigation/measurement or closed proof repeated.
Ordinary full pair/all 2850 records 442 headers/full ABI/coordinate pool/global/
Linux-USA/native Windows/new 32:9/full TODO pending.
[CLI and consumption](../rom_data_gameplay.md#actual-encounter-unit-template-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-UNIT-TEMPLATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair 700 19409→20109 with
CLI dataset e81-d80: every full state/clock/classic-central-wide image/OAM/dispatch/
5 audio passes. All 13 counts match CLI: 186 reads 211B, 35 selector/15 argument/
80 equipment/10 auxiliary/41 bit/10 optional/10 flag/10 control/0 opaque, five
records and one header; eight semantic classes positive. Opaque ordinary not observed,
separate bus proof. One optional 5-line guard/old validator byte exact, one warning-
free Windows build/33c61082.. from 514 sources and one Wine filter. 104 ordinary +
184 CLI guards/514 hashes each exact/helper 0; no new fixture/navigation/measurement/
closed proof repeated. Case closed; all 2850 records 442 headers/full ABI/coordinate
pool/global/Linux-USA/native Windows/new 32:9/full TODO pending.
[Equivalence](../rom_data_gameplay.md#ordinary-encounter-unit-template-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-COORDINATES completed for the EUR source contract
(2026-10-28).** 691 lists/2342 XY pairs/4684B 70B6A8..70C8F4 exactly partitioned
by header pointer-count links: no gap/alias/overlap among 49797 spans 73 owners.
323 placement 0 lists 3272B/34 random 680B/12 placement 2 = 88B/322 default 644B,
323 headers with payload 119 all null. Raw u8 XY preserve signed search and unsigned
reads; RNG 02868 nonnegative 15-bit/remainder 146314/mask u16 remain original, no
invented 10/16 cap or safe null fallback. Nine reused + three new helper/RNG windows,
41 pins/38 guards pass; no build/Wine-Linux test/game/fixture/navigation/closed proof
repeated/measurement. Next strict 691-list model + header coherence and one targeted
Wine oracle, then extraction/runtime/CLI/ordinary. Schema 1/e81-d80/USA 8-7 unchanged;
Linux-USA/native Windows/new 32:9/all callers/full ABI/global/full TODO open.
[Contract](../rom_data_maps.md#european-encounter-coordinate-lists-and-original-readers).


**ROMDATA-02-ENCOUNTER-COORDINATE-MODEL completed EUR/Wine (2026-10-28).**
Strict model 691 lists 2342 raw u8 XY pairs/4684B/691 spans; shape/header coherence/
authenticity separate, diagnostic signed view, no safe null fallback/value cap.
First Windows core build fails due to Bus import in test; only import corrected,
second 516-source copy accepted/2fe3e236.. warning-free/227 core inputs, preimages
retained. One Wine filter passes 30 guards/8044 original signed-unsigned prefixes
across all 4684B, 12 isolated high bits and 102 original random prefixes (34 lists *
3 seeds). RNG/index/addresses/u16 mask/writes/callee-SP/outer RAM-VRAM-OAM-palette-ROM
exact; not full ABI, repeated random/exhaustion/native clock/ordinary. 40 guards
516 accepted + 516 failed hashes/38 source guards exact/helper 0; no new fixture/
navigation/closed proof repeated/measurement. Next managed e82-d81/index 73 + typed
bus, then CLI ordinary; Linux-USA/native Windows/new 32:9/all callers/global/full TODO
pending.
[Model and proof](../rom_data_maps.md#typed-encounter-coordinates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-COORDINATE-RUNTIME completed EUR/Wine (2026-10-28).**
e82-d81/index 73/encounter-coordinates.json, 75 files 74 resources/all 73 previous
exact/header dependency. Immutable owner 691 spans 4684B/four roles + XY/list-header
statistics, local and global 70B..70C routing/composition of old owners at edges.
Three warning-free Windows builds, accepted 0f85fbf7.. from 518 sources. First Wine
filter passes publication/14 attach/3 full-set and stops in native harness; second
isolates concatenation beyond stop. Only test corrected with BL pair and two families
that exclude their respective stops. Third filter reuses dataset, retains all 22 cases
and completes native/lifecycle without repeating publication or attach. The 22 original-
interpreter-native cases match full state and clock with 4684B raw altered after Load
(18 loads/index 0 gate/3 random prefixes), 21 new B/2 template/13 header compiled.
All 4684B/691 lists 323 headers/XY roles/691 boundaries/width-mirror/Load Undo/DMA-
CpuSet-FastSet pass. 253 guards/518 hashes 516 model preimages exact/helper only
interrupted; algorithms/ABI unchanged. Next real CLI + ordinary; Linux-USA/native
Windows/new 32:9/full ABI/all callers/global/full TODO pending. No fixture/navigation/
measurement/old proof repeated.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-coordinates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-COORDINATE-CLI completed EUR/Wine (2026-10-28).** One
warning-free Windows production build/b467d4ad.. from 518 accepted sources, canonical
hash mtime restored. Three Wine commands prepare created/reused/runc 700 pass: 75 files
exact to runtime/74 ID hashes/77 paths no-write/28859 cache blocks loaded. Normal Giza
02 menu/03 input copies 19409→20109: 20 reads 20B/1 list 1 header, placement 0-X-Y
positive; all 10 counts retained. 186 guards 518 hashes/helper 0, no new fixture/
navigation/measurement/closed proof repeated. Full pair next, not all 691 lists/roles/
headers/ABI. Linux-USA/native Windows/new 32:9/global/full TODO pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-coordinate-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-COORDINATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair 700 19409→20109 with
CLI dataset e82-d81: every full state/clock/classic-central-wide image/OAM/5 audio
passes; native and fallback dispatch executed. All 10 counts match CLI: 20 reads 20B,
placement 0=20, X 10/Y 10, one list and one header. Random/placement 2/default not
observed here, previously completed targeted tests remain separate. Optional 5-line
guard/old validator byte exact, one warning-free Windows build/de67e015.. from 518
sources and one Wine filter. 104 ordinary + 186 CLI guards/518 hashes each exact/
helper 0. Case closed; all 691 lists 323 headers with payload/placement-RNG/full ABI/
global/Linux-USA/native Windows/new 32:9/full TODO pending. No new fixture, navigation
or performance measurement.
[Equivalence](../rom_data_maps.md#ordinary-encounter-coordinate-battle-entry-equivalence).


**ROMDATA-01-LINKED-METADATA28 completed for the EUR source contract
(2026-10-28).** 347 records 28B/9716B 710E04..7133F8, 17 roots and 28B arithmetic,
next 4B family with two independent readers. Physical boundary, no guest cap 347.
Getter D07A4 low16/low8: scalars, internal view +12, word flag/21 bits, selectors
34..255→0 without data reads. Direct readers u16+22/byte 24..26; opaque bytes 3/15/27
retained. All 347 records reconstructed, no overlap with 74 owners/50488 spans,
20 windows 14 new 6 reused/69 pins/122 guards/helper 0. Static contract only;
model/Wine/runtime/CLI/ordinary next. Schema 1/e82-d81 unchanged. Linux-USA/native
Windows/new 32:9/global/full TODO pending; no build/game/checkpoint/navigation/
measurement or closed proof repeated.
[Contract](../rom_data_gameplay.md#european-indexed-28-byte-metadata-source-contract).


**ROMDATA-02-LINKED-METADATA28 completed EUR/Wine (2026-10-28).** Lossless model
347 records/9716B, unsigned fields/flag32/opaque bytes, shape/source auth separate;
low16/low8 lookup retains scalars/address view +12/21 bits/default without read,
no invented guest cap or null fallback. One warning-free Windows core build/
358dba2c.. from 520 stable sources and one Wine filter: 27 guards/14517 original
prefixes (12088 complete getters 1388 tail 1041 inline), 8675 interpreted bytes
observed/1041 opaque reconstructed, 9 isolated variants 342 prefixes; RAM/callee/
SP-LR/VRAM/OAM/palette/ROM exact. 36 model guards 520 hashes + 122 source exact/
helper 0. Schema 1/e82-d81 unchanged; managed/runtime next then CLI/ordinary. Full ABI/
next 4B family/global/Linux-USA/native Windows/new 32:9/full TODO pending, no
measurement or new fixture/navigation/old closed proof repeated.
[Model and proof](../rom_data_gameplay.md#typed-indexed-28-byte-metadata-and-original-reader-proof).


**ROMDATA-03-04-LINKED-METADATA28-RUNTIME completed EUR/Wine (2026-10-28).**
e83-d82/index 74/linked-metadata28.json: 76 files 75 resources/all 74 previous exact,
one span 9716B/no invented dependency. Immutable typed owner 5 classes/347 records/
8 counters, local + global pages 710..713, previous header/raw successor boundaries.
One warning-free Windows build/040133f5.. from 522 sources, one Wine filter publication/
interruption 14 attach 2 full-set/23 native full state-clock cases with 9716 raw bytes
altered after Load: 17 loads + 6 D07A4 getters, 31 new bytes + 1 header compiled.
All 9716 bytes/5 classes 347 records/all 347 boundaries/Load Undo/DMA-CpuSet-FastSet
3 state-clock pass. 216 guards 522 hashes 520 model preimages exact/helper retires
only interrupted copy. Actual CLI/ordinary next; ABI/USA 8-7/schema 1 unchanged, no
new game/checkpoint/navigation/measurement or old closed proof repeated. Full ABI/
global/next 4B family/Linux-USA/native Windows/new 32:9/full TODO pending.
[Direct consumption](../rom_data_gameplay.md#managed-indexed-28-byte-metadata-and-typed-guest-reads).


**ROMDATA-04-LINKED-METADATA28-CLI registration completed EUR/Wine;
positive ordinary use still open (2026-10-28).** One warning-free Windows production
build/0acb352e.. from 522 sources: only correction of 3 metadata28 messages, inverse
byte exact/other 521 files exact/canonical hash mtime restored. Create/reuse/normal
starts: 76 files exact to runtime/75 ID hashes/78 paths no-write/28859 cache blocks.
Four normal locators Giza 700/Jobs 540/Abilityset 700/BattleTechRinfo 800 have ALL
8 counters 0, retained as negatives not to repeat; no full pair executed. Menus
observed, new input from normal commands on authorized copy; no new checkpoint or
manual guest edit. 220 guards 522 hashes/helper 0. Next: combat path with nonzero
action before positive/full pair. Linux-USA/native Windows/new 32:9/full ABI/global/
full TODO pending; no measurement.
[CLI and limits](../rom_data_gameplay.md#actual-indexed-metadata28-cli-registration-and-ordinary-locators).


**ROMDATA-04-05-LINKED-METADATA28-COMBAT completed EUR/Wine (2026-10-28).**
Normal opening of Steal menu on dry ground at Giza: 42 frames 22669→22711, 15 reads
20 bytes served/five records, 10 halfword + 10 byte; inline/flag/opaque 0. Capture,
existing CLI 0acb352e.. and one Wine full pair match all 8 counts; every full state/
clock/classic-central-wide image/OAM/5 audio passes. No ability effect executed or
all 347 records claimed. One warning-free Windows test build/5a6c81be.. from 523
sources, optional 5-line guard/old validator byte exact and normal-input-only observer;
seven captures 1182 frames on authorized copies, no manual guest/original save edit.
177 guards 523 hashes + 220 CLI/522 hashes exact/78 paths no-write/helper 0. Four old
negative locators remain closed; source/model/runtime/registration or other family
evidence not repeated. Ordinary case closed; effects/all 347/full ABI/global/next 4B
family/Linux-USA/native Windows/new 32:9/full TODO pending, no performance measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-metadata28-steal-menu-equivalence).


**ROMDATA-01-LINKED-RECORDS4 completed for the EUR source contract
(2026-10-28).** Range 7133F8..713744, 844 bytes in 211 physical 4-byte positions;
six roots and original readers with u8 index multiplied by 4, distinct boundary with
next family at stride 8. All four fields read unsigned; original addresses retained
in live context. 1041 references from first three inline positions of 347 metadata28
records reach 194 distinct positions, three references to index 0 and none outside.
No invented 211/max 208 cap, global limit on three indices, enum or null fallback.
Zeros 0/1/209/210 remain distinct physical records; reserved/padding meaning of final
two still to clarify. Full 844-byte reconstruction, no overlap with 75 resources/
50489 spans e83-d82; 13 authenticated windows (6 new, 7 reused), 45 pins and 112
guards exact/helper 0. Static analysis only, without build/test/game/fixture or repeated
closed evidence. Next model and targeted original-reader Wine proof; management/runtime/
CLI/ordinary use of THIS family still to implement. Linux/USA/native Windows/new 32:9/
full ABI/indirect catalogs/full TODO pending; no measurement.
[Contract](../rom_data_gameplay.md#european-linked-four-byte-record-source-contract).


**ROMDATA-02-LINKED-RECORDS4 completed EUR/Wine (2026-10-28).** Lossless core model:
211 physical positions with four u8 fields/844 bytes/one span; strict shape, order/
indices/parser separate from EUR authentication and 13 windows. Original guest address
calculated for all 256 indices, including external 211..255; diagnostic None does not
become guest fallback. Distinct zeros and meaning of final two still open, no invented
enum/cap/global selector limit. One warning-free Windows core build/a7270580.. from
525 stable sources (233 core inputs), single Wine filter: 20 guards/1567 original calls
or prefixes, 1055 readers cover all 844 bytes and 512 full setters cover both contexts/
all 256 indices/low8 aliases/external addresses. Two isolated high-bit variants pass
10 unsigned prefixes and are rejected by base authentication. Full RAM except expected
5 setter bytes/callee/SP-LR/IWRAM/VRAM/OAM/palette/final ROM exact; 9808 instructions
coverage only. 125 guards 525 hashes and 112 source exact/helper 0. Schema 1/e83-d82/
save/native/CAPI unchanged. Next managed/runtime then CLI/ordinary use of this family;
native clock/full ABI/Linux/USA/native Windows/new 32:9/indirect catalogs/full TODO
pending. No game/checkpoint/measurement/repeated previous closed evidence.
[Model and readers](../rom_data_gameplay.md#typed-four-byte-linked-records-and-original-reader-proof).


**ROMDATA-03-04-LINKED-RECORDS4-RUNTIME completed EUR/Wine (2026-10-29).**
e84-d83/index 75/linked-records4.json: 77 files 76 resources, all 75 previous records/
payload exact, one span 844 bytes without invented dependencies. Immutable typed
owner/four fields/211 positions/seven counters; page 713 local + global, bytewise
composition with previous metadata28 and raw successor. No new cap/null/selector limit,
save/native/CAPI ABI and USA 8-7 unchanged. Three Windows builds: first failed due to
test reference and stale menu cache, second warning-free never executed, third warning-
free 33144a8c.. from 551 current sources; only LDR alignment expectations corrected
before Wine. One Wine filter passes publication/interruption 14 attach 2 full-set/
owner-state intact/21 native full state-clock cases with all 844 raw bytes altered
after Load: 13 loads 4 getters, one double prefix and 3 setters/four groups; 41
instructions/21 new bytes 6 previous. All 844 bytes/four fields 211 positions/all
211 boundaries/Load Undo same Arc and DMA3/CpuSet 424 halfword/FastSet 216 word pass.
237 guards 551 hashes + two 551 diagnostics/525 model preimages/seven current hashes
exact; helper retires only interrupted copy. Next real CLI/ordinary use; Linux/USA/
native Windows/new 32:9/full ABI/external catalogs/full TODO pending. No measurement/
new game/fixture/repeated old closed evidence.
[Direct consumption](../rom_data_gameplay.md#managed-four-byte-linked-records-and-typed-runtime-reads).


**ROMDATA-04-LINKED-RECORDS4-CLI registration completed EUR/Wine;
positive ordinary use pending (2026-10-29).** One warning-free Windows production
build/bcf632fa.. from 551 runtime-identical sources, without overlay; only isolated
corelib/prod fingerprints and canonical hash mtime restored. Three Wine CLI children
exit 0 (create/reuse/new 120 start): 77 files exact to runtime, 76 IDs/hashes loaded,
79 managed paths hash mtime no-write, 28859 cache blocks. Steal menu 22711 copy with
Armor selected, A 8 hold 12→22831 shows targeting but all 7 new counters are 0.
Negative case closed, no full pair or positive use inferred; final checkpoint not
saved by CLI command. 187 guards/551 CLI + 551 runtime hashes exact/helper 0, originals
unchanged; no new fixture or repeated old closed evidence. Next normal combat/target
resolution with positive reads then full equivalence. Linux/USA/native Windows/new
32:9/full ABI/external catalogs/full TODO pending, no measurement or assistant change
to Android/package/commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-linked-four-byte-record-cli-registration).


**ROMDATA-04-05-LINKED-RECORDS4-COMBAT completed for one EUR/Wine ordinary case
(2026-10-29).** Normal Fight/Doit/facing commands lead to next turn; suffix 40 frames
24755→24795, 223 reads/223 bytes served, fields 64/132/11/16 and two physical
positions. All 7 counters identical between capture, existing BCF CLI and one Wine
full pair; each of 40 full state/clock/classic-wide pixels/OAM/5 audio passes. One
warning-free Windows build/a5b2fc84.. from 552 sources: normal observer, test module
and optional 5-line guard only; inverse validator exact/core unchanged. Ten captures
2084 frames on authorized copies, no manual guest/ROM/cart edit; one classic CLI frame
UI only, then authenticated readonly framebuffer projections without further execution.
All 76 ID hashes/79 paths no-write/28859 cache, 210 guards 552 hashes + 187 CLI guards
551 hashes exact/helper 0. Previous zero cases remain closed; no source/model/native/
lifecycle/registration repeated. All 4 fields observed, not all 211 positions or attack
success/full ABI. New 8B family/indirect catalogs/Linux-USA/native Windows/new 32:9/full
TODO pending, no measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-linked-four-byte-turn-equivalence).


**ROMDATA-01-CONDITIONAL-RECORDS8 EUR source contract completed
(2026-10-29).** D56A0 scan at root 713744, stride 8 and ordinals 1..127:
127 positions/1016 bytes up to distinct root 713B3C, whose two readers use stride 70.
Byte 00 zero skips position, byte 01 zero wildcard; LE16 keys at +2/+5 and u8
operands at +4/+7. Live flags up to 05FF and u8 counters for key-0600: first flag
requires equality for operands 0/1 and counter>=operand; second flag requires
inequality and counter<operand, only second key 0 bypasses. No invented Boolean
coercion/cap/header index. 76 enabled rows/51 zero, 105 live references 104 flag
1 counter/63 keys/24 alias groups; operand 2 retained, 47 second key 0. Original
ROM pointer outputs in order, count; u32 capacity/filter without capacity 0 entry
guard, both observed callers pass 64/u8 filter. Helper 370EB0/145C10 remains
already classified executable support, not new asset. SHA256
d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b,
full reconstruction/disjoint from 76 owners 50490 spans; 8 windows 6 new 2 reused/
93 pins/96 guards exact/helper 0. Aligned pointer-word query: 1 root/38 successor,
none internal/origin-8 in that representation; 2 call sites verified. Static analysis
only, no build/Wine/game/fixture/old proof repeated; schema 1/e84-d83/77 files
76 resources unchanged. Next family model and Wine readers; managed/runtime/CLI/
ordinary use/Linux-USA/native Windows/new 32:9/full ABI/indirect callers/successor
extent/full TODO pending. No measurement.
[Contract](../rom_data_gameplay.md#european-conditional-eight-byte-record-source-contract).


**ROMDATA-02-CONDITIONAL-RECORDS8 EUR/Wine model/oracle completed
(2026-10-29).** Lossless core model 127 positions/1016 bytes/one span; strict parser,
shape/order separate from base ROM + 8-window authentication. Gate/filter/LE16 keys/
u8 operands preserved without invented enum/cap/null; select retains original pointers/
order/u32 zero-wrap capacity/full u32 filter and live call multiplicity, different
first/second polarity. One warning-free Windows core build c6f5406d.. from 554 sources/
239 core inputs; only new leaf + test + module, inverse hub byte exact, isolated core
test fingerprint. One Wine filter passes 24 guards and 109 original D56CE..D57A4
cycles with real CCE98/CCF00 readers: 16 base scenarios, 93 isolated synthetic variants
with keys 0/05FF/0600/0682/FFFF, operands 0..255, header 255, filter 128/255 vs full
context, capacity 0/1/64/FFFFFFFF. Python oracle and model agree with actual output/
count/capacity/positions/calls; 612 base bytes and 134 synthetic positions observed,
not all 1016 bytes read. EWRAM exact except expected output/IWRAM except 4-byte
return-link scratch/stack-r11r12/VRAM-OAM-palette/final ROM exact; 198372 instructions
coverage only. 120 guards 554 hashes/3 current hashes + 96 source guards exact/helper 0;
no game/checkpoint/navigation/old proof or reconstructed IWRAM helper. Helper entry/
epilogue/full ABI excluded; schema 1/e84-d83/76 resources/save/native/CAPI unchanged.
Next managed/runtime then CLI/ordinary use; Linux-USA/native Windows/new 32:9/indirect
calls/70B successor/full TODO pending. No performance or space measurement.
[Model and original loop](../rom_data_gameplay.md#typed-conditional-records-and-original-selection-loop-proof).


**ROMDATA-03-04-CONDITIONAL-RECORDS8-RUNTIME completed EUR/Wine with combined
proofs (2026-10-29).** e85-d84/index 76/conditional-records8.json: 78 files 77 resources,
76 previous exact/one span 1016B/no deps. Immutable owner 6 classes/127 slots/9 counters,
page 713 local + global and previous records4/raw successor composition; guest/live/
address logic and save/native/CAPI ABI unchanged. Four warning-free Windows builds from
568 consistent current sources; first never run after pre-Wine correction of test-only
iteration limit. First Wine filter passes publication/interruption 14 attach 2 full-set +
16 native full state-clock cases, then fails in the 17th for observer 607 vs 609. Diagnosis
of remaining case alone proves identical raw/typed/native state-clock and 609 actual bytes,
completes bus/lifecycle; final observer-only run passes 609 callbacks = 607 direct +
2 interp_one fallback, no unattributed bytes. Only test corrected; publication/attach/
16 cases/bus not repeated. 17 distinct cases: 14 loads 2 LE prefixes 1 scan; all 1016 raw
bytes altered AFTER Load/setup. Total 631 bytes served by native path = 629 direct
2 fallback and 5 records4 bytes, not distinct union or all compiled. All 1016 bus B/
6 classes 127 slots/127 boundaries/failed owner-full state/Load Undo same Arc and DMA3/
CpuSet 510 halfword/FastSet 256 word full state-clock pass. 261 guards 568 accepted
hashes + three 568 diagnostics/554 model preimages/7 current hashes/77 old 78 new files
exact; helper retires only interrupted copy. Accepted exe f60055e6.., first failure
retained; no game/checkpoint/navigation/previous proof or performance measurement.
Next real CLI and ordinary use; Linux-USA/native Windows/new 32:9/full ABI/indirect
calls/70B successor/full TODO pending.
[Consumption and combined evidence](../rom_data_gameplay.md#managed-conditional-records-and-typed-runtime-reads).


**ROMDATA-04-CONDITIONAL-RECORDS8-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build 72d44985.. from 568 sources
byte-identical to runtime/no overlay, isolated corelib + prod fingerprints and canonical
hash mtime restored. Three Wine CLI children exit 0: create e85-d84/reuse/neutral
startup 1; 78 files exact to runtime/77 ID hashes loaded/80 managed paths no-write/
28859 old blocks. First prepare 0 succeeds; local helper checked index 75 instead of 76:
corrected and only two remaining children resumed, without repeating prepare or build.
Normal before-activation copy 24755→24756 with neutral input, all 9 new counters 0;
no action/navigation/new checkpoint/saved endpoint, no positive ordinary/full pair
inferred. 189 guards 568 CLI + 568 runtime hashes exact/helper 0, originals unchanged.
Source/model/native/lifecycle/old scenes remain closed. Next positive ordinary use and
full comparison; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/70B
successor/full TODO pending, no measurement or assistant change to Android/package/
commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-conditional-record-cli-registration).


**ROMDATA-04-05-CONDITIONAL-RECORDS8-PUB completed for one EUR/Wine ordinary
case (2026-10-29).** Normal Rumors entry in pub; suffix 42f 15181→15223, 468 reads/
468 bytes SERVED, six classes 127/76/152/76/34/3 and 127 physical positions including
51 zero headers, not 127 active records/all fields/all branches/byte union. All 9
counters exact across capture/CLI 72d44985 and full pair 63584945; each of 42 state/
clock/classic-wide pixels/OAM/5 audio equal. One warning-free Windows build from
569 sources: CLI 568 + passive observer + module + optional 5-line guard; inverse
hub-validator byte exact/core-runtime unchanged. Two normal captures 254 frames:
B 180 negative then Up 3/A 12 positive 74, first read rel 42 + 32 continuation,
rolling rel 32→suffix 42. Seed Missions LIST 14969 already retained, no old 620/397
route replay; old PPM classic 240x160 color scale *255//31 authenticated against
FFSTATE, not recent wide format. Readonly projections and images verified, no Wine UI/
guest edit/cart write; final prompt HeardlatestRumors, list/selection/outcome not
inferred. 143 guards 569 hashes + 568 CLI/3 root hashes/80 managed paths no-write/
28859 cache exact, helper 0. CLI e85 generation hw22uM reused without prepare/build/
navigation; first B 0 case remains closed. Declared source/model/runtime/CLI/ordinary
8B closed; next new root 713B3C/stride 70 requires its own contract. Linux-USA/native
Windows/new 32:9/full ABI/indirect calls/full TODO pending; no measurement or assistant
change to Android/package/commit/push.
[Equivalence](../rom_data_gameplay.md#ordinary-conditional-record-rumors-entry-equivalence).


**ROMDATA-01-MISSION-RECORDS70 EUR source contract completed
(2026-10-29).** Catalog 713B3C..71C73C/512 records x 70/35840B, two independent
scans 1..511 plus index 0 getter/successor boundary with own readers; separate constant
300, no new guest cap 512. Getter 65 selectors low16 index/low8 selector: unsigned LE
from byte, packed bits, flags, zero or +0177 mapping and 200 scale, 7 address selectors/
6 inline offsets, aliases 19/20 and views 24/32 with shared scalars. Remaining bytes/
raw values/identity order retained. Already closed mission link reused: field +45 HEX
69 decimal event row *12+71C760; 512 references/173 values/four alias groups, zero
depends on caller; distinct event catalog still to classify. All 35840B reconstructed,
SHA256 bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc,
disjoint from 77 owners 50491 spans/e85-d84/78 files unchanged. 19 windows 9 new
10 cache/157 pins 110 guards exact; old text authenticated opcode-by-opcode against
ROM and current hash, no false historical text guard. 38 base literals/2 origin+70/
2 internal candidates in data not classified readers, aligned query not exhaustive
for indirect calls. Static only, no build/Wine/game/fixture/navigation/previous proof
repeated; helper retires zero paths. Next strict lossless model/Wine getter oracle,
then managed/runtime/CLI/ordinary. Full ABI/unknown field semantics/view counts/
successors/Linux-USA/native Windows/new 32:9/full TODO pending. No performance or
space measurement; parallel menu/content/DBG/package work preserved.
[Contract](../rom_data_gameplay.md#european-mission-record-source-contract).


**ROMDATA-02-MISSION-RECORDS70 EUR/Wine model/oracle completed
(2026-10-29).** New lossless core mission_records70 512 records/35840B/one span,
strict shape/order/JSON separate from exact ROM + 19-window authentication. Remaining
bytes/inline arrays/raw operands and all 65 selectors retained; low16 index/low8
selector query, Some 0 invalid without lookup, addresses calculated even outside owner,
unowned scalar None diagnostic without new guest cap. One warning-free Windows core
build 663e8b87.. from 571 sources/242 core inputs, 569 preimages exact and inverse hub
byte exact; only two leaves/public module, no parallel frontend. One Wine filter passes
21 guards and 698 calls of original D1F30 getter to return with real CEEEC helper:
438 base/all 65 at indices 0/1/58/299/300/511/normalization/default/views outside
owner/scalars 512-65535, 260 isolated synthetic variants on four 70B patterns at record
511. Independent Python/model/results and actual LDRB read order exact; 324 distinct
base bytes/54 synthetic versus 35840B separately reconstructed, views do not dereference
16B inline. EWRAM/VRAM/OAM/palette unchanged, IWRAM exact including expected return
links, SP/R4..R12/Thumb/final ROM exact; 17213 instructions coverage only. 139 guards
571 hashes/3 current 569 preimages + 110 source guards/helper 0. No game/checkpoint/
cart write/navigation/already closed proof or measurement. Schema 1/e85-d84/77 resources
78 files/save/native/CAPI unchanged. Next managed/runtime/native clock/lifecycle then
CLI/ordinary; full ABI/unknown fields/indirect calls/successors/Linux-USA/native Windows/
new 32:9/full TODO pending. Parallel work preserved.
[Model and getter](../rom_data_gameplay.md#typed-mission-records-and-original-getter-proof).


**ROMDATA-03-04-MISSION-RECORDS70-RUNTIME completed EUR/Wine with combined
evidence (2026-10-29).** e86-d85/index 77/mission-records70.json/79 files 78 owners,
77 previous exact/one span 35840B/no deps; immutable owner 10 classes 13 counters/
8 mask words 512 positions/pages 713..71c, previous 8B and raw successors composition.
Two warning-free Windows builds 573 sources/243 core inputs: 73326ad9 first publication/
prior 77/interruption PASS then first attach FAIL historic OBJ cap 1024, other tests
not started. With request for no space limits removed attach caps OBJ 1024/BG 64KiB
and three allocation assertions, parse bounds/source auth/binding unchanged. 455bdb1c
second build/Wine filter ONLY remaining on SAME 79-file generation: 14 attach 2 full-set/
failed owner full state/41 native state-clock (16 loads 25 getters) with 35k raw altered
AFTER load/setup/all 35k bus 10 classes 512 slots/512 edges/Load Undo same Arc/DMA3 pass.
Publication not repeated, first failure retained; no cap fix interpreted as optimization
or RSS measurement. All callbacks including interp_one attributed: 52 new bytes served
5 previous/fallback delta 0, not distinct union/all catalog native; 3 stop-safe families/
168 entries 69 dispatch coverage only. DMA/CpuSet 17922 halfword/FastSet 8968 word full
state-clock identical. 860 guards 573 accepted + 573 diagnostics/571 model/8 current
hashes/78 old 79 new files exact; helper retires only interrupted copy. close-final.py
PASS-combined-scopes, old close-runtime.py not run. Source/model/old scenes not repeated,
no game/checkpoint/navigation/cart write/changed Machine-save-native-CAPI ABI. Next real
CLI then ordinary; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/successors/
full TODO pending. No performance/space measurement, parallel work excluded from build
and preserved.
[Consumption and evidence](../rom_data_gameplay.md#managed-mission-records-and-typed-runtime-reads).


**ROMDATA-04-MISSION-RECORDS70-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build 738b7f20.. from 573 sources
byte-identical to runtime/no overlay, consistent corelib-prod-menu fingerprints and
canonical hash mtime restored. Three Wine children all 0: prepare created/reuse/neutral
startup 1, 79 files exact to runtime/78 ID hashes/81 managed paths no-write/28859 old
native blocks. Private generator stopped on token-index assertion before any Wine: only
helper generation corrected with verified index 77, no source copy/build/CLI preparation
repeated. Normal copy 24755→24756 all 13 counters 0; no action/navigation/new checkpoint/
saved endpoint, positive ordinary full pair or new negative locator phase inferred.
195 guards 573 CLI + 573 runtime/79 files/originals/native cache/loader/canonical exact;
helper 0. Source/model/native/lifecycle/old proofs remain closed. Next positive ordinary
and full comparison; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/successors/
full TODO pending, no measurement/Android/package/assistant commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-mission-record-cli-registration).


**ROMDATA-04-05-MISSION-RECORDS70-MISSIONS completed for one EUR/Wine ordinary
case (2026-10-29).** Normal opening of Giza Plains information, suffix 33 frames
15017→15050: 5 reads/5 bytes SERVED only packed class/one physical position, other
9 classes 0. All 13 counters exact across capture/CLI 738b7f20/full pair 3d21e89c;
each of 33 full state-clock/classic-wide pixels/OAM/5 audio equal, both arms same old
native cache/not global interpreter-native parity. One warning-free Windows build
574 sources = CLI 573 + passive observer/public module/optional 5-line guard, inverse
hub byte exact/core-runtime unchanged. ONE capture 81f from already retained Missions
LIST 14969: Down 3 at 8/A 12 at 40, first read rel 49/guest 15019, rolling 48→suffix
33 A 4 + neutral 29; search STOP. Wine test child 0 PASS; post-capture helper KeyError
steps after complete output repaired only recording, no capture/input/Wine/build repeated.
Already authenticated classic seed 240x160/scale reused; new wide PPM 284x160/bitrep
verified against full FFSTATE, classic PNG shows Giza MISSION info page 1/3 Rank 1. No
mission accepted/battle/outcome/all fields/all 512/union of 5 distinct bytes inferred.
133 guards 574 hashes + 573 CLI/3 current/78 ID hashes 81 managed paths no-write 28859
cache exact, helper 0/originals unchanged. Old 620/397 route/source-model-runtime/
registration not repeated, no forced ID/RAM/cart write. Declared 70B chain closed;
new C73C/C740 arrays and C760 event require own contracts. Linux-USA/native Windows/
new 32:9/full ABI/indirect calls/other ordinary classes/full TODO pending; no performance/
space measurement/Android/package/assistant commit/push.
[Equivalence](../rom_data_gameplay.md#ordinary-mission-information-equivalence).


**ROMDATA-01-SELECTION-BYTE-TABLES EUR source contract completed
(2026-10-29).** C73C..C740 four count thresholds and C740..C75E thirty ordinal
weights: 34 bytes/two spans, SHA256
d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd. Five live
headers 02002E34+6*i counted only if low 3 bits nonzero, all examined; guard <=3 before
u8 threshold load, 4/5 exit. Second independent root, two stride 1 loops ordinals 1..30/
positions 0..29. Real helper 377BC reads raw live u8 02002CC3+12*(low8 ordinal-1),
no global 0/30 guard or Boolean normalization; caller tests low8 nonzero. Sum 0 exits
before second stage; full32 helper rank minus unsigned weight with wrap and signed
comparison, first negative selects, 31 exhausted exits; no invented negative guard/
zero-weight/cached eligibility. Numeric helper 2868/146314/RNG/distribution/full ABI
not modeled. All 34B reconstructed/positions retained/repeated values not deduplicated/
no data pointers; disjoint from 78 owners 50492 spans/e86-d85/79 files unchanged.
6 windows 2 new 4 cache/95 pins 96 guards/helper 0, only 2 new count context and live
helper; no build/Wine/game/fixture/navigation/old proof repeated or measurement. Gap
C75E..C760 two opaque B unowned, next 12B root C760 independent/extent pending. Neutral
ID gameplay.selection-byte-tables. Next model/Wine oracle then managed/runtime/CLI/
ordinary; Linux-USA/native Windows/new 32:9/indirect calls/full TODO pending.
[Contract](../rom_data_gameplay.md#european-selection-byte-table-source-contract).


**ROMDATA-02-SELECTION-BYTE-TABLES EUR/Wine model/stages completed
(2026-10-29).** Strict model 4+30 u8/34B/two spans, shape separate from exact ROM +
six windows; gap 2B excluded/repeated values distinct. One warning-free Windows core
build b40503b8.. 576 files 245 inputs/574 preimages; one Wine filter 18 guards/45 base +
24 synthetic executes D323C count/D327E sum-zero/D32B4 choice with real 377BC. Python/
model/results/live query order and LDRB exact; wrap32/signed rank including extremes,
no weight 0 shortcut or new cap. All 34B base/32 synthetic observed, 34 reconstruction
separate, all RAM/SP-R8..R12/final ROM exact; 30480 instructions coverage only. 123 guards
576 hashes/3 current/574 preimages/96 source guards PASS, helper 0. Schema 1/e86-d85/
79 files unchanged; managed runtime/bus/native clock/Load Undo/DMA/CLI/ordinary next,
numeric helpers/RNG/full ABI/Linux/USA/native Windows/new 32:9/global TODO pending. No
ordinary fixture or closed proof repeated/metrics.
[Model and proof](../rom_data_gameplay.md#typed-selection-byte-tables-and-original-stage-proof).


**ROMDATA-03-04-SELECTION-BYTE-TABLES-RUNTIME completed EUR/Wine
(2026-10-29).** e87-d86/index 78/selection-byte-tables.json/80 files 79 resources,
two spans 4/30B/no deps, all 78 previous records/files identical/schema 1-USA e8
unchanged. Immutable page 71C owner composes prior 70B and raw gap/event root;
6 counters/4+30 distinct positions, no allocation cap or ABI change. One warning-free
Windows build e5b45668.. 578 files 246 core/576 model, one Wine filter publication/
interruption/14 attach 2 full-set/20 native full state-clock/all 34 bus/Load Undo/DMA3
PASS. 12 loads + 8 stages/real 377BC, raw 34 + prior 4 altered only after Load;
76 new B served + 5 prior all attributed to callbacks, fallback source 0, 91 entries
119 dispatch coverage only. Width/mirror/edges/prior/raw, all 4+30 positions and
DMA-CpuSet 20 halfword/FastSet 16 word exact state-clock. 243 guards 578 hashes 576
model/7 current/79 prior exact; helper retires only successful interrupted copy. Real
CLI/ordinary next; numeric helpers/RNG/full ABI/gap-event extent/Linux/USA/native
Windows/new 32:9/global TODO pending, no old proof or fixture repeated/metrics.
[Consumption](../rom_data_gameplay.md#managed-selection-byte-tables-and-typed-runtime-reads).


**ROMDATA-04-SELECTION-BYTE-TABLES-CLI EUR/Wine registration-startup completed
(2026-10-29).** One warning-free production build 22af4132.. same 578 runtime sources/
no overlay/canonical hash mtime restored; one Wine orchestration three commands create-
reuse-neutral 1 exit 0. 80 identical dataset files/79 ID hashes/82 paths hash mtime
unchanged/cache 28859 unchanged. e87-d86/index 78/two spans 34B/no deps verified;
24755→24756 all 6 usage 0, no ordinary acceptance. 192 guards 578 CLI + 578 runtime/
ROM-native cache-TinyCC-original seed-canonical exact/helper 0. No navigation/new
checkpoint/old proof/metrics. Positive ordinary and full pair next; Linux-USA-native
Windows/new 32:9/full ABI/RNG/indirect calls/gap-event/global TODO pending.
[CLI](../rom_data_gameplay.md#actual-selection-byte-cli-registration).


**ROMDATA-04-SELECTION-BYTE-TABLES-ORDINARY completed one EUR/Wine case
(2026-10-29).** Eight new windows/354 direct opcode pins D3724→D3228/low8 argument
nonzero guard; wrapper D6970 passes 1, other 2 callers 0 skip, indirect unknown. Four
readonly seeds guard 6/count 1-2; normal Giza 19049/cached UI selected. One 579-source
8f3bee97 build observer 6 stats + 5-line guard/no core-runtime changes/inverse hub exact.
ONE capture A 12/129f 19049→19178/first 97-19147/rolling 96→suffix 33-19145→19178.
SAME 22af CLI/e87 ZloxMx positive 33 and SAME 8f3bee97 full pair 33 all complete
states-clock/classic-wide/OAM/5 audio PASS; 6 exact counts 1 bus 1 SERVED threshold B
1 position/weights 0, not all 4/30 positions or event/mission outcome. Discovery STOP
after one. Full wide 284x160 bitrep authenticated/classic map calendar PNG inspected.
140 guards 579 hashes 578 CLI/3 current/82 managed/native cache-ROM-seed-canonical
exact/helper 0. Dispatch 203258/519441 and audio [72418,72418,8712,8712,72342] coverage
only; same cache in both arms/not global interpreter-native parity. No closed proof or
fixture recreated/metrics. Source-model-runtime-CLI/one ordinary chain closed; event
12B root C760/alias 71D1EC/next 71D360 requires own contract. Linux-USA-native Windows/
new 32:9/full ABI/RNG/indirect calls/gap-event/global TODO pending.
[Ordinary use](../rom_data_gameplay.md#ordinary-selection-threshold-travel-equivalence).


**ROMDATA-01-EVENT-RECORDS12 EUR source contract completed (2026-10-29).**
71C760..71D360/256x12/3072B/one span SHA e7454120.. disjoint from 79 owners
50494 spans; backscan 255..1/stride 12 + live u8 zero + independent next root stride
27 prove extent, not fit-only/global cap. Layout u8 header 00-key 01/u16 primary 02-
alternate 04/u8 param 06-byte 07/u16 flags 08/opaque 0A 2B, all 3072 roundtrip/no
invented enums-FK caps-zero padding. 71D1EC=base+225x12/31 physical positions
INTERNAL view/same owner, helper 9D04 low8 arg not capped at 31/can leave owner;
ordinal choice 1..30→rows 226..255. Callback 127388 full u16 key/vs byte 01/highest
match 255..1/zero excluded/8 key groups 1 record group. Alt 04 nonzero→alias primary
02 + original alt 04, zero→direct primary/alt 0; key 4A live special/param 06 next
27 root/live effects original. 15 windows 10 new 5 cached/67 pins/112 guards/80
accepted files PASS/helper 0; census 36 base 2 alias 25 next and not all indirect.
Static only/no build-Wine-game-fixture-navigation-old proof replay; schema 1/e87-d86
unchanged. Model/original Wine prefixes next, then managed/runtime/CLI/ordinary;
full ABI/live effects/indirect calls/gap/next 27 extent/Linux-USA-native Windows/
new 32:9/global TODO pending/no metrics.
[Contract](../rom_data_gameplay.md#european-event-record-source-contract).


**ROMDATA-02-EVENT-RECORDS12 EUR/Wine model/prefixes completed (2026-10-30).**
Strict 256x12/3072B/one span/raw flags/opaque 2B, shape order distinct from exact
ROM + 15 windows. Physical None outside owner vs u16 encoded view/uncapped low8 alias,
direct/alias-original alt pair and full16 script highest 255..1/row 0 excluded. One
warning-free core build a4b88a69.. 581 files 248 core/579 preimages/inverse hub exact,
one Wine filter 20 guards 51 base 37 synthetic prefixes 9D04 pair/127388 scan/seven
unsigned loads/u16 view PASS; Python-model-results/LDR order exact. 302 base B 268
synthetic B observed vs separate 3072 roundtrip; RAM exact with expected index store/
push, SP-R8..R12/final ROM verified, 9959 instructions coverage only. 133 guards
581 hashes/3 current/579 preimages 112 source exact/helper 0. Schema 1/e87-d86/80 files
unchanged; managed/native clock/lifecycle/CLI/ordinary next. Full ABI/live effects/
indirect calls/gap/next 27 extent/Linux-USA-native Windows/new 32:9/global TODO pending.
No old proof/ordinary fixture/navigation repeated/metrics.
[Model](../rom_data_gameplay.md#typed-event-records-and-original-prefix-proof).


**ROMDATA-03-04-EVENT-RECORDS12-RUNTIME completed EUR/Wine (2026-10-30).**
e88-d87/index 79/event-records12.json/81 files 80 resources/79 prior exact, 3072B/
one span/no deps/alias row 225 SAME owner; pages 71c-71d/8 classes 11 stats/4 mask
words 256 positions, prior selection 2B + raw gap 2/next root composed, no allocation
cap/guest ABI change. One warning-free build 9225a3bc.. 583 sources 249 core 581 model,
one Wine filter publication-interruption/14 attach 2 full-set/22 native full state-clock/
all 3072 bus 8 classes 256 records 256 borders/Load Undo/DMA3 PASS. 14 loads + 8 pair-
scan-u16 view/4 stop-safe families; raw 3072 + prior 2 altered after Load, 296 SERVED
new B 3 prior all attributed, 2 new B interp_one fallback (not 296 direct native or
distinct union)/51 entries 30 dispatch coverage only. DMA-CpuSet 1538 halfword/FastSet
776 word exact state-clock. 253 guards 583 hashes 581 model/seven current 80 prior exact;
helper only successful interrupted. Actual CLI/ordinary next, full ABI-live effects-
indirect calls-gap-next 27/Linux-USA-native Windows/new 32:9/full TODO pending; no old
proof/fixture/metrics.
[Consumption](../rom_data_gameplay.md#managed-event-records-and-typed-runtime-reads).


**ROMDATA-04-EVENT-RECORDS12-CLI EUR/Wine registration-startup completed
(2026-10-30).** One warning-free production build b9a60501.. SAME 583 runtime sources/
no overlays/canonical hash mtime restored; one Wine orchestration create-reuse-neutral 1
three exit 0. 81 identical payloads 80 ID hashes 83 no-write paths/cache 28859 exact;
e88-d87/index 79/one 3072 span/no deps. 24755→24756 all 11 counts 0/compatibility only,
not positive ordinary. 194 guards 583 CLI + 583 runtime/81 accepted/ROM-native cache-
TinyCC-seed-canonical exact/helper 0. Positive ordinary/full pair next; Linux-USA-native
Windows/new 32:9/full ABI-live effects/indirect calls/gap-next 27/full TODO pending.
No source-model-native/old proof/fixture-navigation replay or metrics.
[CLI](../rom_data_gameplay.md#actual-event-record-cli-registration).


**ROMDATA-04-EVENT-RECORDS12-ORDINARY completed one EUR/Wine case (2026-10-30).**
Original normal Giza White Flowers 19409 seed/cached UI + producer witness reused,
not old 700f route proof. One 584-source 0b6581cb build observer 11 stats + 5-line
guard/inverse hub-validator exact/core-runtime unchanged. ONE A 12 capture 34f
19409→19443/first rel 2-19412/rolling 0 suffix 34; discovery STOP. SAME b9a605 CLI/
e88 generation 3iczRx actual child 0/34f, collector TypeError for OLD usage ID fixed
OFFLINE log registration only/no CLI replay. SAME 0b6581cb full pair 34 all complete
states-clock/classic-wide/OAM/5 audio PASS. 11 counts exact 4 bus 6 SERVED B/key 2-
primary 2-alternate 2/one physical record/other 5 classes 0; not distinct union/all
8-all 256/alias branch/outcome. Full wide 284x160 bitrep authenticated/classic Giza
map mission entry inspected. 126 guards 584 hashes 583 CLI/3 current/83 no-write paths/
80 ID hashes/cache 28859-ROM-seed-canonical exact/helper 0; dispatch 128240/218562/
audio [74612,74612,8976,8976,74488] coverage only. Event source-model-runtime-CLI/
one ordinary chain closed; next 27 root 71D360 contract extent/fields pending,
Linux-USA-native Windows/new 32:9/full ABI-live effects/indirect calls/gap/full TODO
open/no metrics.
[Ordinary use](../rom_data_gameplay.md#ordinary-event-record-mission-entry-equivalence).


**ROMDATA-01-PARAMETER-RECORDS27 completed for observed EUR fragments
(2026-10-30); total extent still open.** The 256 event records prove 59 indices
0..50 and 115..122, stride 27/root 71D360: two ranges 71D360..71D8C1 and
71DF81..71E059/1593B, roundtrip SHA 40d4e406.., 256 links and no overlap with
80 owners/50495 spans e88-d87. 27 windows 23 new 4 reused/70 pins/125 guards,
all 25 root literals covered in cached census. Shared packed fields 01..06,
u16 LE 07-0A-12, explicit i8 0C-15, byte 09 and flags 0D, separate operands
0E0F-1011, opaque 14+16..1A retained. No enum/pointer/padding inferred from values.
Live +A8 readers use u8 without 127/128/59 limit; next root 71E0E0 stride 38 does
not prove 128 records by distance alone. 128 hypothesis rejected; gaps/further indices
raw and unowned. Static only, no build/Wine/input/fixture/old proof repeated or metric.
Wine model/proof, runtime, CLI and ordinary to do; global extent/full ABI/live effects/
indirect calls/Linux-USA-native Windows/new 32:9/full TODO open. Event 12 chain closed.
[Contract](../rom_data_gameplay.md#observed-european-parameter-records-with-27-byte-stride).


**ROMDATA-02-PARAMETER-RECORDS27 completed for 59 observed EUR records
(2026-10-30).** Strict model with physical indices 0..50/115..122, u8/packed fields,
u16 LE, i8/opaque, two spans 1593B; modification validation separate from source auth.
record/byte_at do not respond in gaps or for high indices; encoded_view retains original
low8 without 59/128 limit. One 586-source/251-core build 49c72f02 warning-free, one
Wine filter 25 guards 73 base 48 synthetic: 12 classes of original prefixes, exact
results and read order, signed 127/-128/low16 encoding included. All RAM/SP-R8..R12/
final ROM exact; 95 base B 20 synthetic B distinct observed vs 1593B roundtrip.
739 instructions coverage only. 143 guards/584 preimages/125 source guards/inverse
hub retained/helper 0. No old proof or fixture repeated, nor metrics. Managed runtime
e89-d88/CLI/ordinary to do; global extent/full ABI/helper/live effects/indirect calls/
Linux-USA-native Windows/new 32:9/full TODO open. e88-d87 unchanged.
[Model](../rom_data_gameplay.md#sparse-parameter-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-PARAMETER-RECORDS27-RUNTIME EUR/Wine completed for 59 observed
records (2026-10-30).** e89-d88/index 80/parameter-records27.json/82 files 81 resources,
80 previous exact/no deps/two spans 1593B total; sparse owner 71D..71E composes previous
event and raw gaps, 12 classes 15 stats/four physical masks. No new cap/ABI/save/native
cache change. First 588 build 51e19ef0 passes publication, interruption, 14 attach
2 full-set/17 native full state-clock; test oracle expects 2 previous bytes instead of
4 aligned bytes served, typed/native/API agree on 4. Only test leaf corrected: second
588 build 278d6d23 warning-free/one Wine run only remaining 16 native + bus/lifecycle/
DMA3 PASS, same 82 files 13 objects hash mtime exact; no publication/attach/first 17
cases repeated. Total 33 full state-clock/20 loads 13 prefixes/13 families, 37 SERVED
new B 5 event; not distinct union or all direct native, fallback 0 only for remaining 16/
previous 17 not reported. All 1593B, 12 classes 59 records/59 borders/raw gap-next root/
Load Undo, DMA-CpuSet 1664 halfword/FastSet 832 word exact. 291 guards 588 accepted +
588 diagnostic/252 core/586 model/7 current 81 prior retained; helper only interrupted.
Collector task label corrected offline without build/test replay. accepted-selection.json
authoritative, selection.json diagnostic. Actual CLI/ordinary/global extent/full ABI/
helper-live effects/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[Runtime](../rom_data_gameplay.md#managed-sparse-parameter-records-and-typed-runtime-reads).


**ROMDATA-04-PARAMETER-RECORDS27-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 588-source production bd99ac5b warning-free, SAME corrected runtime/
no overlays/canonical hash mtime finally restored. One Wine orchestration: prepare creates
e89-d88, prepare reuses, runc copied normal checkpoint 24755 neutral 1→24756; three
exit 0/82 runtime-exact payloads/81 ID hashes/84 managed paths hash mtime unchanged/
28859 native cache. Index 80/two spans 1593B total/no deps correct. All 15 counts 0,
compatibility/registration only, not positive ordinary use or equivalence. 197 guards
588 CLI + 588 runtime/ROM-seed-cache-TinyCC-canonical exact/helper 0. No new navigation/
checkpoint or old tests or metrics repeated. Positive ordinary/full pair, global extent/
full ABI/helper-live effects/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[CLI](../rom_data_gameplay.md#actual-sparse-parameter-record-cli-registration).


**ROMDATA-04-PARAMETER-RECORDS27-ORDINARY partial (2026-10-30), positive still to
obtain.** Passive observer 15 stats + module/optional 5-line guard; 589-source 8f0c02b7
warning-free/core-runtime 588 CLI unchanged/inverse preimages exact. Two normal Wine
captures on copies: R 12 GizaInfo 15050→15230/180f page 1/3 unchanged; A 12 at 1/161
WhiteFlowers dialogue 21889→22129/240f. All 15 counts 0: NEGATIVE bounds closed, no
positive suffix/positive CLI/full pair executed. 9 static windows/7 readonly snapshot
gates; accepted script directory/archive/events join identifies opcode 80 mode 2 in
programs 53/65/74/93/95, keys 56/68/77/96/98, events 216..220/params 116..120.
WhiteFlowers 58 param 0/program 163; stop further blind attempts in page or intro,
indirect calls/mode 0 override 0 not excluded. 130 guards 589 + 588 hashes/3 current
84 managed/originals retained, diagnostics kept/no full cleanup. Reuse SAME observer
and bd99 CLI/e89 dataset when relevant scene known; positive/full equivalence/global
extent/Linux-USA-native Windows/new 32:9/full TODO open/no metrics. Update to same unit:
eleven total normal captures, all 15 stats 0, no rebuild. New contexts SprohmInfo 180/
B 32/Nubswood cursor/travel 360 and CyrilPub 240+120; no mission selection inferred
from name or RNG resampling. Six authenticated snapshot gates do not prove liveness.
One neutral 1 Wine inspection/already compiled CLI/layout off exit 0 does not save/adopt
state; readonly framebuffer/UI 76800-pixel projections show only Gil, no readable choice.
233 guards 589 + 588 hashes/3 current 84 managed exact, first receipts retained. Blind
navigation stopped; positive ordinary still open, independent source contracts may
continue without repeating these captures.
[Status and evidence](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).


**ROMDATA-01-WORLD-RECORDS38 completed for the observed EUR catalog
(2026-10-30).** Root 71E0E0/31x38/1178B up to 71E57A,
SHA cd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483.
All 512 missions project exactly indices 0..30; interior root 71E106 and loop 6..29
use SAME owner/records 7..30, next 71E57C reads u32 pointer/unowned gap 2B. No
count-by-distance or cap 31: D347C low8 and D4C4C full R2 u32 retain original
addresses. Byte 01/u16 LE 02 with separate byte 02/u16 LE 04 with low10 copy/
31 thresholds 07 + count clamped 0..30/opaque 00-06 retained. 12 windows 10 new
2 cached/58 pins 107 guards, 1178 roundtrip 512 links/no overlap 81 owners
50497 spans/e89-d88/82 files exact. Static only/no build-Wine-input-fixture/old
evidence or metrics. Wine model/oracle/runtime/CLI/ordinary to do; helper-RNG/
full ABI/live effects/indirect calls/opaque/Linux-USA-native Windows/new 32:9/
full TODO open. Sparse 27 ordinary remains partial, navigation not reopened.
[Contract](../rom_data_gameplay.md#observed-european-world-records-with-38-byte-stride).


**ROMDATA-02-WORLD-RECORDS38 completed EUR/Wine (2026-10-30).** Strict model
31 records/1178B/one span, opaque and all bits retained; editable shape separate
from exact EUR auth/12 window hashes. Physical record/byte_at does not own gap 2B
or indices 31+, original encoded low8/full u32/interior row 1 without cap 31.
Projections byte 01 divisor + post-helper wrap/zero 1, byte 02/u16 LE 02, full
u16/low10 value 04, live count clamp 0..30/signed threshold comparison with raw
255. One 590-source/254-core build ed4d65c8 warning-free, one Wine filter
19 guards 41 base 19 synthetic original PASS; 23 base B 6 synthetic B distinct
observed vs 1178 roundtrip/353 instructions coverage only. Read order/results/
all RAM + expected destinations/SP-R8..R12/final ROM exact. 26 unit guards
107 source/588 preimages/3 current/inverse hub exact, helper 0. No external
helpers-RNG/full ABI/native clock/ordinary claim; no old proof/navigation/fixture
repeated or metric. Schema 1/e89-d88/82 files unchanged. Managed runtime e90-d89/
CLI/ordinary and Linux-USA-native Windows/new 32:9/full TODO to do.
[Model](../rom_data_gameplay.md#world-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-WORLD-RECORDS38-RUNTIME completed EUR/Wine (2026-10-30).**
e90-d89/index 81/world-records38.json/83 files 82 resources/prior 81 exact/no deps/
one span 1178B. Immutable owner 71E composes sparse 27/raw gap/next root, 5 classes
8 stats/one 31 mask without new cap or ABI. One 592 build 271dceba warning-free;
collector guard-label corrected offline on SAME exe, no rebuild. One Wine filter
passes publication/interruption/14 attach 2 full-set/27 native full state-clock
(17 loads, 10 prefixes/10 stop-safe families), 23 SERVED new B 5 prior/fallback
[0, 0], not distinct union/all direct native. All 1178B/5 classes 31 records/
31 borders/mirrors-widths/prior + raw gap-next root/Load slot Undo exact; DMA-CpuSet
704 halfword/FastSet 352 word full state-clock. 269 guards 592 frozen/255 core/
590 model/7 current 82 prior retained, current observer 27 excluded ONLY from frozen
hub. Helper only interrupted copy. Actual CLI/ordinary/full ABI/helper-RNG/live
effects/opaque-indirect/Linux-USA-native Windows/new 32:9/full TODO open. No old
proof/navigation/fixture or metrics repeated.
[Runtime](../rom_data_gameplay.md#managed-world-records-and-typed-runtime-reads).


**ROMDATA-04-WORLD-RECORDS38-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 592-source production 6d95f705 warning-free, SAME runtime/no
overlays/canonical hash mtime restored. One Wine orchestration/three exit 0: prepare
creates e90-d89, prepare reuses, runc copied normal checkpoint 24755 neutral 1→24756.
83 runtime-exact files/82 ID hashes/85 managed paths hash mtime unchanged/cache
28859 exact; index 81/no deps/one span 1178B. Eight counts 0, registration/compatibility
only, not positive ordinary/equivalence. 201 guards 592 CLI + 592 runtime/ROM-seed-
cache-TinyCC-canonical exact/helper 0. No new navigation/fixture/old proof or metric
repeated. Positive ordinary/full pair/full ABI/helper-RNG/live effects/opaque-indirect/
Linux-USA-native Windows/new 32:9/full TODO open.
[CLI](../rom_data_gameplay.md#actual-world-record-cli-registration).


**ROMDATA-04-WORLD-RECORDS38-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One 593-source ee939288 warning-free/passive 8 stats + 5-line guard,
core-runtime 592 CLI unchanged/inverse hubs exact/current 27 observer retained. First
A 12 Nubswood 360f 3448→3808 NEGATIVE closed; readonly CCF00/path 0..6/counter 32→30
explains branch not ready. Four authenticated EUR banks: IN08 physical 13 counter 1,
normal Down 48/96f 3020→3116 ends actual node 18; A 12 single positive capture 189f
3116→3305, first source 157/3274, rolling 45 suffix 3260→3305/discovery STOP. SAME
6d95 CLI one 45/SAME 593 test one full pair 45: each state-clock/classic-wide/OAM/
5 audio exact, 8 identical stats 2 bus 2 SERVED B = 1 threshold + 1 byte 01/one
physical record, other 3 classes 0/not all 5-31. 82 ID hashes 85 managed/cache 28859
no-write, authenticated framebuffer projection/world travel MadmoonDay4, no mission/
arrival inferred. 165 guards/593 + 592 sources/3 current/inverse/originals exact/helper 0.
Normal inputs on copies only, no reset-RNG-RAM-ID/route recreation, no old proof or
metric. Other ordinary/full ABI/helper-RNG/live effects/opaque-indirect/Linux-USA-native
Windows/new 32:9/full TODO open; sparse 27 ordinary remains partial.
[Case and limits](../rom_data_gameplay.md#ordinary-world-record-counter-expiration-equivalence).


**ROMDATA-01-INDEXED-GRAPHICS-ARCHIVES completed for the EUR source contract
(2026-10-30).** Four pointer groups 71E57C/71E61C/71E630/71E658, five language
slots each, 20 archives 649 programs; 24 disjoint spans 74907B source (74827 archives
+ 80 pointer), 278336B output/15631B dictionary prefixes. Codec 05228/0537C with BE
index-offset and fixed external dictionary, distinct 054A0; opaque 4137 is not a
discriminator, raw u8 language without fallback and original low16 start-count. All
source/output bytes reconstructed exactly, 22 windows 20 new 2 cached/101 pins/10 root
literals/12 merged BL/121 guards/no overlap 82 owners 50498 spans/83 accepted files
exact. Other 16 nearby groups/code-dictionary tails/context 3710D4/global extent not
classified. No build-Wine-game-fixture/old proof/metric repeated; schema 1/e90-d89
unchanged. Model/Wine oracle/runtime/CLI/ordinary/full ABI/indirect/Linux-USA-native
Windows/new 32:9/full TODO open; sparse 27 ordinary partial.
[Contract and limits](../rom_data_indexed_graphics.md).


**ROMDATA-02-INDEXED-GRAPHICS-ARCHIVES completed EUR/Wine (2026-10-30).** Strict
model 20 archives/four five-slot groups/649 programs: opaque header, BE index and
offset/literal-dictionary packets/dictionary prefixes/LE words exact, editable shape
separate from exact EUR auth. Host mode 0 width 5 only, no new guest cap; raw language
encoded view remains original for 5/255, low16 start-count/zero-oversize-clamp/invalid
start before init retained. One 595-source/257-core build d78a74d9 warning-free from
593 ordinary + 2 leaves/one public module; one Wine filter 37 guards 20 whole archive
calls + 9 boundary + 9 synthetic + 28 raw selector prefixes PASS. All 74907B/24 spans/
278336 decoded B reconstructed; output/status/globals/SP-R4..R11 and RAM outside 68B
original stack/VRAM-OAM-palette/final ROM exact. 5993129 instructions coverage only.
26 unit guards 121 source/593 preimages/3 current/inverse hub exact. No closed proof/
checkpoint/navigation/performance replay; schema 1/e90-d89 unchanged. Managed runtime/
native clock/CLI/ordinary/mode 3-wider width/full ABI-live uploads/indirect/Linux-USA-
native Windows/new 32:9/full TODO open.
[Model and proof](../rom_data_indexed_graphics.md#typed-model-and-original-wrapper-proof).


**ROMDATA-03-04-INDEXED-GRAPHICS-RUNTIME completed EUR/Wine (2026-10-30).**
e91-d90/index 82/graphics.indexed-archives/indexed-graphics.json/84 files 83 resources/
all 82 prior records-files exact/no deps/24 spans 74907B; immutable owner reconstructed
from header-index-packets-dictionary/7 classes 12 stats/20 banks 649 programs 4 group
masks. No decoder-upload shortcut/new ABI or guest cap. First 597 build fails only Bus
import test-only, diagnostics retained; corrected 597/258 core 0732b3b9 warning-free/
production unchanged. One Wine filter PASS publication-interruption/14 attach 2 full-set/
35 complete state-clock (28 loads + one original 1-program wrapper + 6 raw selectors)/
3 stop-safe families; 272 SERVED new B 4 prior world/fallback [1,0], not distinct union/
all direct native; 232 instructions 59 dispatch coverage only. All 74907 bus B/7 classes
20 archives 649 programs 4 groups/all 24 borders-mirrors-widths/prior world + raw gap/
Load slot Undo/DMA-CpuSet-FastSet mixed 64B full state-clock exact. 282 guards 597
accepted 597 diagnostic/595 model/7 current 83 prior retained. Accepted-selection.json
authoritative, selection.json first FAIL diagnostic. Actual CLI/ordinary/full ABI-live
uploads/mode 3-wider width/global extent/indirect/Linux-USA-native Windows/new 32:9/full
TODO open. No old proof/fixture/navigation/metric repeated.
[Runtime and evidence](../rom_data_indexed_graphics.md#managed-source-views).


**ROMDATA-04-INDEXED-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 597-source production d8767645 warning-free, SAME runtime/no
overlays/canonical hash mtime finally restored. One Wine orchestration/three exit 0:
prepare creates e91-d90, prepare reuses, runc copied normal checkpoint 24755 neutral 1
→24756. 84 runtime-exact files/83 ID hashes/86 managed paths hash mtime unchanged/
cache 28859 exact; index 82/no deps/24 spans 74907B. All 12 counts 0, registration/
compatibility only, no positive ordinary/full pair. 201 guards 597 CLI + 597 runtime/
84 accepted/ROM-seed-cache-TinyCC-canonical exact. No new navigation/fixture/old proof/
metric repeated. Ordinary/full ABI-live uploads/mode 3-wider width/global extent/indirect/
Linux-USA-native Windows/new 32:9/full TODO open.
[CLI and limits](../rom_data_indexed_graphics.md#actual-cli-registration).


**ROMDATA-04-INDEXED-GRAPHICS-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One 598-source 6d40a6a5 warning-free/passive 12 stats + 5-line guard,
597 CLI core-runtime unchanged/inverse hubs exact/current parameter-world observers
retained. 8 windows 6 new 2 cached/453E2→1967C context 3→program 2 MENU; one Start 12
normal capture 41f 4519→4560/first source 9 guest 4529/rolling 41 suffix/discovery STOP.
SAME d876 CLI one 41/all 83 ID hashes 86 managed no-write/cache 28859 exact and SAME
test one full pair every 41 state-clock/classic-wide/OAM/5 audio exact, 12 identical
stats 274 reads 277 SERVED B = selector 4/header 6/index 4/token 67/literal 17/
dictionary header 4/dictionary 175, one archive-program-group/all 7 classes positive/
not all 20-649-4. 144 guards/598 frozen 597 CLI/3 current/inverse/originals exact.
Readonly full wide 284x160 projection/classic PNG shows MENU Party-Clan-AreaList-System/
MoonDay9, no choice/mission outcome. Normal inputs only, no RAM-ID-RNG-ROM forcing or
old 320 Down-A/world 38 daily 45/negative/proof replay/metric. Other ordinary/full ABI/
mode 3-wider width/global extent/indirect/Linux-USA-native Windows/new 32:9/full TODO
open; sparse 27 ordinary remains partial.
[Case and limits](../rom_data_indexed_graphics.md#ordinary-world-map-menu-equivalence).


**ROMDATA-01-LOCALIZED-TABLE-GRAPHICS EUR source contract completed
(2026-10-30).** Eight independent groups 71E590/5A4/5B8/5CC/644/680/6BC/6D0
raw u8 language 02001F6E×4/LE32 pointer→054A0, 40 new custom LZ-RLE/nonindexed
banks. 48 spans 53667 source B (53507 bank + 160 selector)/114880 decoded B, all
tokens/source/output reconstructed exactly; 14 windows 10 new 4 cached/48 pins/
8 root literals/111 guards/no overlap 83 owners 50522 spans/all 84 accepted files
exact. Output per group five banks 1472/2688/768/4928/1920/7488/1344/2368B, no
invented count-by-distance or guest cap/fallback skip. Live RAM/helper transfer/
later work remain original, full transfer ABI not closed. Known 71E5E0 old UI
banks/table alias separate and seven other raw-looking groups/tails/indirect calls/
global extent unclassified. Static only/no build-Wine-game-fixture-navigation/old
proof or metric; schema 1/e91-d90 unchanged. Model/Wine oracle/runtime/CLI/ordinary/
Linux-USA-native Windows/new 32:9/full TODO open.
[Contract and limits](../rom_data_ui_graphics.md#additional-european-table-selected-source-contract).


**ROMDATA-02-LOCALIZED-TABLE-GRAPHICS completed EUR/Wine (2026-10-30).**
New core ui_graphics::table_selected reuses Graphic/custom LZ: strict 8 tables × 5
contiguous BankPointers/40 ordered banks/raw encoded byte + LE words/group outputs,
editable shape distinct from exact EUR auth/14 fingerprints. Physical select None
for 5/255 but original raw encoded view. Separate catalog not yet enabled in managed
UI resource: old 16 banks/two inline switches/e91 unchanged. One 600 frozen/260 core
build cb46ccb1 warning-free from 598 ordinary + two leaves/one public module; one
Wine run 23 guards/40 new original 054A0 calls 56 raw selector prefixes PASS. All
53667 source B/48 spans/114880 decoded B/cursor-produced-returned-size/SP-R4..R11/
full EWRAM/IWRAM outside 40B original stack/VRAM-OAM-palette/final ROM exact.
1335470 instructions coverage only. 26 unit 111 source guards/600 frozen 598 preimages/
3 current/inverse hub exact; no old 16 UI/20 indexed/native 35/41 ordinary/CLI proof
replay/fixture-navigation/metric. Offline control label 23B2E corrected to 0150 word/
1344B, not group set 02A0 halfword, originals/inputs unchanged/no proof replay. Managed
runtime/native clock/CLI/ordinary/full helper ABI/live contexts/71E5E0 alias/seven raw
groups/global extent/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[Model and evidence](../rom_data_ui_graphics.md#typed-table-model-and-new-original-decoder-proof).


**ROMDATA-03-04-LOCALIZED-TABLE-GRAPHICS-RUNTIME completed EUR/Wine
(2026-10-30).** e92-d91/same 84 files 83 IDs/UI index 15 56 banks 10 selectors
106 bindings, all other 82 records-files/old 16 banks-two inline-shared OBJ exact.
40 banks + 40 pointers 80 new runtime bindings byte union 48 canonical spans 53667B
exact; 6 new table counters separate from old 6 UI stats, original decode-scratch-
helper-upload work/ABI retained. One 601-source/260-core d4a73eb2 warning-free from
600 model + one host leaf; ONE Wine PASS publication-changed UI interruption/
14 attach 2 full-set/22 complete state-clock (12 loads + 1 new bank decoder + 9 raw
selectors)/3 stop-safe families. 499 SERVED new B 5 prior indexed/fallback [3,0],
not distinct union/all direct native; 356 instructions 71 dispatch coverage only.
All 53667 bus B 24328 literal 160 selector 40 banks 8 groups/all 48 edges/widths-
mirrors-prior indexed-raw-old UI/Load slot Undo/DMA3 mixed full state-clock exact.
267 guards 601 frozen 600 model/seven current 84 prior exact/helper interrupted only.
No old 40 model/16 UI/20 indexed/native 35/41 ordinary/CLI prepare proofs replay or
fixture-navigation/metric. Actual CLI/ordinary/full helper ABI-live contexts/71E5E0
alias/seven raw groups/global extent-indirect-tails/Linux-USA-native Windows/new 32:9/
full TODO open.
[Runtime and evidence](../rom_data_ui_graphics.md#managed-table-selected-graphics-and-runtime-consumption).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 601 runtime-identical production 31c0f298 warning-free/no overlays/
canonical hash mtime finally exact; ONE Wine orchestration 3 commands child 0 creates-
reuses-neutral 1 normal 24755 copy→24756. 84 runtime-exact files 83 ID hashes 86 managed
no-write/cache 28859 exact, e92-d91/UI index 15 56 banks 10 selectors 106 bindings/same
84 files 83 IDs. Six new table counts 0/registration-compatibility only/no positive
ordinary-full pair. 202 guards 601 CLI + 601 runtime/84 accepted/ROM-seed-cache-TinyCC-
canonical exact. No old 40 model/16 UI/indexed 20/native 22-35/41 ordinary/CLI prepare
proof replay/new checkpoint derivation-navigation/metric. Positive ordinary/full helper
ABI/live contexts/global extent-indirect/71E5E0 alias/seven raw groups/tails/Linux-USA-
native Windows/new 32:9/full TODO open.
[CLI and limits](../rom_data_ui_graphics.md#actual-table-extension-cli-registration).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY partial; two negative bounds
closed EUR/Wine (2026-10-30).** Cached/raw source 244D6→2C8C4/root 71E6D0 and old
normal Fight prediction image select 64f 23851→23915; distinct 71E590 COMBOMENU
asset/30608 constructor + 30FDA redraw and Action seed 23611 two Down+A select 160f
→23771. SAME 31c0 CLI both child 0/all 6 new counts 0/83 ID hashes 86 managed
no-write/cache 28859 exact. Static initializer link does not prove new load; PPM
omits separate HUD/menu, Combo not claimed reached. No observer build/guard/full pair/
endpoint state/longer retry/RNG resampling/old route recreation. Diagnostics retained/
discovery stopped; positive ordinary/full equivalence open pending concrete constructor
context. Source-model-runtime-CLI registration remain closed; Linux-USA-native Windows/
new 32:9/full TODO open/no metrics.
[Status and limits](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).


**ROMDATA-01-LOCALIZED-RAW-TABLE-SOURCES completed for fixed EUR views
(2026-10-30).** Seven independent roots 71E5F4/608/66C/694/6A8/6E4/6F8, five
physical slots each and unbounded u8 language selector: 35 views/42 spans/38220B
(38080 payload + 140 selector), SHA
e11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80. Six prefixes
224/160/768/1600/2048/2432B and only +0200..0380/384B of seventh; lengths from
original copies/alternate branches, not distance/header. Fused BL 12EE24→145230/
CpuSet halfword and 6F8→000AD0/control 80000040 halfword verified, first hypothesis
corrected before output. 32 windows 23 new 9 cached/193 pins/13 root literals/
13 fixed copies/130 guards/no overlap 83 owners 50602 bindings/all 84 accepted files
exact. Four dynamic 6F8 views/u8 task+2−4/u16 offset<<5 do not establish a cap:
metadata/lower prefix/tail and full domain remain open. Static analysis only, schema
1/e92-d91/runtime unchanged; model/Wine oracle/runtime/CLI/ordinary/full ABI/71E5E0
alias/indirect/full TODO open; new Linux-USA-native Windows-32:9 to do, no metric/
build/game/fixture/navigation/old proof repeated.
[Contract and limits](../rom_data_ui_raw_transfers.md#declared-source-contract).


**ROMDATA-02-LOCALIZED-RAW-TRANSFERS completed EUR/Wine (2026-10-30).**
New core ui_raw_transfers/RawTransfers+BankView: seven physical tables/35 ordered
views, 42 spans 38220B/LE words and seventh sparse group +0200..0380 only. Editable
shape distinct from EUR authentication/32 fingerprints; u8 language addresses outside
0..4 raw, no fallback/cap. Build 603 sources/262 core 35bdc5fd warning-free; three
builds and two Wine attempts, one final filter PASS 21 guards/91 original prefixes/
60 isolated helper copies/25 original DMA queue executor copies. All bytes, EWRAM/
VRAM/OAM/palette, IWRAM outside original 36B stack when needed, SP/callee/final ROM
exact. First corrections test-only: stack 20→36B before Wine, boundary between adjacent
tables incorrectly expected raw; diagnostics and preimage retained, model/source
unchanged. 44 unit 130 source guards/601 baseline 603 frozen/3 current/inverse hub exact;
6635 instructions coverage only. Schema e92-d91 and 84 accepted files unchanged;
runtime/native clock/CLI/ordinary/full caller ABI/6F8-lower prefix-metadata-tail domain/
71E5E0 alias/full TODO open. New Linux-USA-native Windows-32:9 to do; no old proof/
game/fixture/checkpoint/navigation/metric repeated.
[Model and evidence](../rom_data_ui_raw_transfers.md#typed-model).


**ROMDATA-03-04-LOCALIZED-RAW-TRANSFERS-RUNTIME completed EUR/Wine
(2026-10-30).** e93-d92/index 83 graphics.ui-raw-transfers/85 files 84 IDs, all
83 previous records/payload exact. 42 spans 38220B from typed raw words and pointers,
six separate counters; original selectors/copies/DMA queue/ABI-clock preserved. Two
builds 605 sources/263 core, accepted 4b219adc warning-free; one Wine filter PASS
publication-interruption/14 attach 2 full-set/23 native full states-clock (12 loads +
1 CpuSet + 1 queue executor + 9 language prefixes)/4 stop-safe families. 939 SERVED
new B 3 previous/fallback [768,0], not distinct union or all direct native; 115
instructions 25 dispatch coverage only. All 38220B/35 views 7 groups/42 edges/width-
mirror/adjacency between new and old/raw gaps/Load slot Undo/four mixed 64B transfers
DMA16-DMA32-CpuSet-FastSet pass. 309 guards 605 frozen 603 model/7 current 84 old/
inverse hub exact. Before Wine only test corrected: 441608 starts previous indexed
archive, not raw gap; product code unchanged between builds and first diagnostic
950bea97 retained. Runner labels e92/DMA3 obsolete metadata only: JSON proves [1,93,92]/
4 transfers, no replay. Real CLI/ordinary/full caller ABI/6F8-metadata-prefix-tail domain/
71E5E0 alias/indirect/full TODO open; new Linux-USA-native Windows-32:9 to do. No old
proof/source-model/game-fixture-checkpoint-navigation/metric repeated.
[Runtime and evidence](../rom_data_ui_raw_transfers.md#managed-raw-transfer-resource).


**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 605 runtime-identical production build ffda6c59 warning-free/no
overlays/canonical hash mtime finally exact; one Wine orchestration three child 0
commands create-reuse-neutral 1/normal 24755 copy→24756. 85 runtime-identical files/
84 ID hashes 87 managed no-write/cache 28859 exact. e93-d92/index 83/35 views 7 tables/
42 bindings 38220B. Six new counters 0: registration/compatibility only, not positive
ordinary use or full pair. 203 guards 605 CLI + 605 runtime/85 accepted/ROM-seed-cache-
TinyCC-canonical exact; no new endpoint/derived checkpoint/action input/navigation/old
proof repeated/metric. Positive ordinary/full ABI/6F8-metadata-prefix-tail domain/
71E5E0 alias/indirect/global catalog/full TODO open; new Linux-USA-native Windows-32:9
to do.
[CLI and limits](../rom_data_ui_raw_transfers.md#actual-cli-registration).

**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-ORDINARY completed for one EUR/Wine
startup (2026-10-31).** CLI e93 observes 5.488 bytes served, including
selectors and payloads for four groups; a 1.200-frame pair using the same
dataset verifies state/clock/video/OAM/dispatch/audio. The same native cache
is used in both branches; other paths and Linux/USA remain to verify.
[Evidence and limits](../rom_data_ui_raw_transfers.md#ordinary-european-boot-equivalence).

**ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN partial (2026-10-31).** Original
calls document selectors 7/8/9 and 1.304 non-overlapping candidate bytes;
other producers read live values and the global limit is unknown. At discovery
there was no runtime owner; subsequent integration is described below.
[Details](../rom_data_ui_raw_transfers.md#bounded-dynamic-view-producer-discovery).

**ROMDATA-02-LOCALIZED-RAW-DYNAMIC-FRAGMENTS completed for the EUR/Wine
model (2026-10-31).** Rows and prefixes have a lossless typed model with
source and boundary verification for 1.304 bytes. Subsequent integration is
documented below; no global selector limit is inferred.
[Model](../rom_data_ui_raw_transfers.md#dynamic-fragment-source-model).

**ROMDATA-03/04-DYNAMIC-RAW-FRAGMENTS EUR/Wine (2026-10-31).** Separate
resource e94-d93/index84, 1.304 bytes in seven authenticated ranges;
reads, composition and one DMA state/clock verified. CLI, native execution,
ordinary use, global domain, Linux and USA remain open.
[Evidence](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).

**ROMDATA-04-DYNAMIC-RAW-FRAGMENTS-CLI EUR/Wine (2026-10-31).** Creation,
reuse and loading of all 85 e94-d93 resources verified with one neutral frame;
identities and managed files exact. Dynamic ordinary use, native execution,
global domain and Linux/USA remain open.
[Evidence](../rom_data_ui_raw_transfers.md#dynamic-fragment-production-cli).

**ROMDATA-04-DYNAMIC-RAW-ORIGINAL-NATIVE EUR/Wine (2026-10-31).** Original
routine, queueing and DMA verified for selectors 7/8/9, both branches and
five languages: 30 isolated cases, equal state/clock on the ROM interpreter,
dataset interpreter and translated dataset code. Ordinary use and global domain,
Linux and USA remain open.
[Evidence](../rom_data_ui_raw_transfers.md#original-dynamic-routine-and-translated-execution).

**ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY partial (2026-10-31).**
Guest gate `0200203d` and activation/deactivation callbacks identified;
14 EUR checkpoints and one Unit320 path show no transfer within the observed
scope. Positive ordinary use and Linux/USA checks remain open.
[Evidence](../rom_data_ui_raw_transfers.md#ordinary-dynamic-reachability-investigation).

**ROMDATA-01-DYNAMIC-RAW-DISPATCH-LIFECYCLE partial EUR/Wine (2026-10-31).**
Callbacks and conditional registration flag identified; the existing Area List
path showed no dynamic reads or active flag at sampled points. Global absence
is not proven; positive ordinary use, global domain and Linux/USA checks remain
open.
[Evidence](../rom_data_ui_raw_transfers.md#dispatcher-lifecycle-and-area-list-observation).

**ROMDATA-01-LOCALIZED-TABLE-ALIAS bounded EUR source (2026-10-31).**
Independent table `71e5e0`, 20 unmanaged bytes, five already managed clan banks,
six original sequences identified. Integration and Linux/USA checks remain open.
[Contract](../rom_data_ui_graphics.md#european-alias-of-the-existing-clan-banks).

**ROMDATA-03-04-LOCALIZED-TABLE-ALIAS EUR/Wine (2026-10-31).** e95-d94
publishes the 20 bytes of table `71e5e0` in the UI resource, reusing the five
clan banks. Reads and 42 interpreter prefixes with state/clock verified; CLI,
native, ordinary use and Linux/USA checks remain open.
[Evidence](../rom_data_ui_graphics.md#managed-european-clan-table-alias).

**ROMDATA-04-CLAN-TABLE-ALIAS-NATIVE EUR/Wine (2026-10-31).** The six
translated original prefixes match the ROM interpreter for 24 state/clock cases;
CLI, ordinary use, full ABI and Linux/USA remain open.
[Evidence](../rom_data_ui_graphics.md#translated-clan-alias-selector-prefixes).

**ROMDATA-04-CLAN-TABLE-ALIAS-CLI EUR/Wine (2026-10-31).** CLI e95 verifies
creation, reuse, identity of the 85 resources and one frame; positive ordinary use
and Linux/USA checks remain open.
[Evidence](../rom_data_ui_graphics.md#clan-alias-production-cli).

**ROMDATA-05-CLAN-TABLE-ALIAS-ORDINARY bounded negative EUR/Wine
(2026-10-31).** The existing 1.200-frame startup shows no alias bus reads in
the observed filter; positive ordinary use and Linux/USA remain open.
[Evidence](../rom_data_ui_graphics.md#clan-alias-ordinary-boot-observation).

**ROMDATA-05-CLAN-TABLE-ALIAS-BANK-ENTRY EUR/Wine (2026-10-31).** Two typed
ordinary alias reads during a Bank entry; a separate 120-frame pair with the same
state and input verifies state/clock, images and audio. Other paths, Linux and
USA remain open.
[Evidence](../rom_data_ui_graphics.md#ordinary-clan-alias-use-during-bank-entry).

**ROMDATA-01-E95-HISTORICAL-READ-RESIDUE bounded (2026-10-31).** Of the
1.146 unmanaged locations in the historical F09/e71 audit, 74 are now covered by
e95; the other 1.072 have previous code-support evidence. This is not a new
observation or a global inventory.
[Evidence](../rom_data_runtime.md#historical-f09-residue-against-e95).

**ROMDATA-01-GAMEPLAY-6E6DBC-LOOKUP bounded EUR source (2026-10-31).**
Projection of 74 two-field u16 rows, supported by original producers;
live indices outside the projection and the global limit remain open. Model and
runtime consumption still need implementation.
[Contract](../rom_data_gameplay.md#european-conditional-four-byte-lookup-projection).

**ROMDATA-02-GAMEPLAY-6E6DBC-LOOKUP EUR/Wine model (2026-10-31).**
Lossless typed model and original readers verified for the 74-row projection;
managed publication and runtime consumption remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-four-byte-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6DBC-LOOKUP managed EUR/Wine (2026-10-31).**
e96-d95 publishes the conditional projection of 74 rows: 85 previous resources
identical, typed reads and boundaries verified, transactional rejection of altered
data. Three original native prefixes and six indices match interpreter state/clock;
indices outside the projection are not bounded. Production CLI, ordinary use and
Linux/USA checks remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-four-byte-lookup).


**ROMDATA-04-GAMEPLAY-6E6DBC-CLI EUR/Wine (2026-10-31).** The production
CLI creates and reuses e96-d95 and advances one frame: 87 files identical to the
accepted dataset, 86 runtime identities verified and no changes to managed files.
Ordinary use and Linux/USA checks remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-ROUTE EUR candidate (2026-10-31).**
An already retained Abilità input and the original first-category branch indicate
a candidate for the typed read; reached state and a nonempty list are hypotheses
to observe. A single 540-frame Wine prefix is the next check; Linux/USA remain
open. No performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY positive EUR/Wine (2026-10-31).**
The already retained ordinary Abilità input produces 55 typed reads/bytes,
27 from the first field and 28 from the second, in the 540-frame segment. The
counter is aggregate; full scene comparison and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#ordinary-conditional-lookup-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-PAIR EUR/Wine (2026-10-31).**
For the same 540-frame Abilità sequence, full state/clock, video, OAM, dispatch
and five audio outputs match between original ROM and managed runtime; 55 typed
bytes cover both fields in the same test. Other scenes and Linux/USA checks remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-scene-differential).


**ROMDATA-01-GAMEPLAY-6E6EE4-ROOT conditional EUR source (2026-10-31).**
Three original readers use a u16 index from a stride-eight descriptor; the type 5
branch has static witnesses for indices 1..33, totaling 132 bytes without overlap
e96. Ordinary reachability, row 0 and global limit are unproven; other types may read
previous owners. Model and runtime remain open; Linux/USA to do, no performance
measurement.
[Contract](../rom_data_gameplay.md#european-conditional-packed-four-byte-projection).


**ROMDATA-02-GAMEPLAY-6E6EE4-PACKED EUR/Wine model (2026-10-31).** Strict model
of 33 typed rows, reconstruction of 132 bytes and original readers checked for zero,
endpoints 1/33 and alias 251 to metadata14. It introduces no guest limits; managed
publication, runtime and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-packed-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6EE4-PACKED managed EUR/Wine (2026-10-31).**
e97-d96 publishes 132 typed bytes: 86 previous resources identical, shared boundaries
and alias 251 still served by metadata14, transactional rejection of changes and two
native prefixes equivalent to interpreter. CLI, ordinary use and Linux/USA checks
remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-packed-lookup).


**ROMDATA-04-GAMEPLAY-6E6EE4-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e97-d96 and advances one frame: 88 files identical to accepted dataset,
87 runtime identities verified and no change to managed files. Ordinary use and
Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY-ROUTE EUR candidate (2026-10-31).** An
already recorded Polly→Hildegard attack contains a wrapper with type 5/index 1
descriptor and an original chain compatible with the reader. Some branches remain
to observe; an earlier suspicion of bit 43 blocking was wrong. One Wine observation
limited to the existing segment can distinguish use.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY bounded negative EUR/Wine
(2026-10-31).** Existing 500-frame F09 combat shows no packed lookup reads. This is
a negative for the observed scene alone, not proof of global absence; positive
ordinary use and Linux/USA remain open. No performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-f09-observation).


**ROMDATA-01-GAMEPLAY-6DF4D4-DESCRIPTORS conditional EUR source
(2026-10-31).** Original count producer supports 23 pointers and 684 eight-byte
records in banks 1..23. Count 11 of bank 22 excludes 48 bytes before the next pointer;
bank 0 remains separate and raw. No global guest limit; model/runtime and Linux/USA
remain open.
[Contract](../rom_data_gameplay.md#european-conditional-eight-byte-descriptor-banks).


**ROMDATA-02-GAMEPLAY-6DF4D4-DESCRIPTORS EUR/Wine model (2026-10-31).**
Lossless model of 23 banks and 684 records, with original counts and writes verified
on Wine; bank 0 and bank 22 gap remain raw and without invented guest limits.
Publication/runtime and Linux/USA checks remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-descriptor-bank-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-DESCRIPTORS managed EUR/Wine
(2026-10-31).** e98-d97 publishes 23 pointers and 684 typed descriptors: 87 previous
resources identical, bank 0 and bank 22 gap remain raw, consistent edit rejected and
five native prefixes equivalent to interpreter. CLI, ordinary use and Linux/USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-descriptor-banks).


**ROMDATA-04-GAMEPLAY-6DF4D4-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e98-d97 and advances one frame: 89 files identical to accepted dataset,
88 runtime identities verified and managed files unchanged. Descriptor counters zero
in the frame; ordinary use and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-banks-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-ROUTE EUR candidate (2026-10-31).** Original
Abilità panel renderer reads pointers and descriptors from unit bank; already retained
state and input reach the panel. e98 use is not yet observed. One 540-frame Wine segment
checks it.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-banks-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY positive EUR/Wine (2026-10-31).** The
540-frame Abilità sequence reads 64 typed bytes: 52 pointers, 6 field 0, 6 field 2;
other fields remain unobserved. Full comparison, other scenes and Linux/USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-descriptor-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-PAIR EUR/Wine (2026-10-31).** In the same
540-frame Abilità sequence, state/clock, video, OAM, dispatch and audio match; in the
same test 52 pointer bytes and 6+6 bytes of fields 0/2 are typed. Field 4/type/byte 7
and Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-01-GAMEPLAY-6DF4D4-BANK0 conditional EUR source (2026-10-31).** Original
producer reads pointer 0 and may enumerate records 1..15 when unit+37 is nonzero;
an initializer may set it for unit+7=24. No ordinary witness in 62 snapshots examined;
record 0 and other indices remain raw, with no invented guest limit. Model/runtime and
Linux/USA remain open.
[Contract](../rom_data_gameplay.md#conditional-descriptor-bank-0-source).


**ROMDATA-02-GAMEPLAY-6DF4D4-BANK0 EUR/Wine model (2026-10-31).** Strict model for
pointer 0 and records 1..15; original predicate, store, mode 3 and scan pass on Wine
for zero/nonzero slots. Record 0 remains raw; ordinary use, publication/runtime and
Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-descriptor-bank-0-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-BANK0 managed EUR/Wine (2026-10-31).** e99-d98
publishes pointer 0 and 15 descriptors: 88 previous resources identical, boundaries
with bank 1 composed and record 0 raw, consistent edit rejected, three native prefixes
equivalent. CLI, ordinary use and Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-descriptor-bank-0).


**ROMDATA-04-GAMEPLAY-6DF4D4-BANK0-CLI EUR/Wine (2026-10-31).** Production CLI
creates and reuses e99-d98 and advances one frame: 90 files identical to accepted
dataset, 89 runtime identities verified and managed files unchanged. Ordinary use and
Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-bank-0-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-ORDINARY-PAIR EUR/Wine (2026-10-31).** Item
information screen uses pointer 0 and fields 0/2/type/byte 7 of records. On the same
800-frame ordinary sequence raw ROM and e99 dataset comparison matches state/clock,
video, OAM, dispatch and audio; field 4 not observed. Both sides share native cache:
Linux, USA and native Windows remain to verify. No performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-item-info-access-to-descriptor-bank-0).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-FIELD4-PAIR EUR/Wine (2026-10-31).** In battle
Item menu an existing 340-frame sequence reads 32 typed bytes of bank 0 field 4. Raw
ROM/e99 comparison matches state/clock, video, OAM, dispatch and audio, using the same
native cache on both sides. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#ordinary-battle-item-menu-bank-0-field-4).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-KIND-RECONCILE EUR/Wine (2026-10-31).** Already
retained battle Item scene log also records 4 pointer bytes and 1 type byte of the
banks 1..23 descriptor resource. This is aggregate use, without individual record/PC
attribution; scene comparison verifies bank 0 without asserting this secondary resource.
Ordinary field 4/byte 7, Linux and USA remain open. No replay or performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-INFO-PAIR EUR/Wine (2026-10-31).** Already
recorded Battle Tech screen reads 10 typed type bytes and 10 byte 7 values of descriptors
in banks 1..23. Over 800 frames raw ROM/e99 comparison matches state/clock, video, OAM,
dispatch and audio; field 4 not observed and native cache shared. Linux, USA and native
Windows remain to verify; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-battle-tech-descriptor-information).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-FIELD4-PAIR EUR/Wine (2026-10-31).** In Steal
menu a retained 280-frame sequence reads 60 typed bytes of field 4 in banks 1..23.
Raw ROM/e99 comparison matches state/clock, video, OAM, dispatch and audio; both sides
use same native cache. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#ordinary-steal-menu-descriptor-field-4).


**ROMDATA-01-GAMEPLAY-6E0B44-MASKS conditional EUR source (2026-10-31).**
Three original readers index u32 masks using metadata52 property 0x24. The 116
authenticated rows produce all indices 0..34; source range 6e0b44..6e0bd0 contains
35 words and is distinct from e99 resources. Dynamic/invalid selectors are not globally
bounded: no guest cap. Model, runtime, ordinary use and Linux/USA remain open; no
performance measurement.
[Contract](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-02-GAMEPLAY-6E0B44-MASKS EUR/Wine model (2026-10-31).** Strict lossless
model of 35 u32 masks/140 bytes, without added guest limits; Wine checks bytes,
boundaries, original selectors and external indices. Publication/runtime, ordinary
use, Linux and USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-03-04-GAMEPLAY-6E0B44-MASKS managed EUR/Wine (2026-10-31).** e100-d99
publishes 35 typed masks, retains exactly 89 previous resources, composes boundaries
with adjacent owners and rejects consistent edits; one original native prefix matches
the interpreter in the six selected cases. CLI, ordinary use, Linux and USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-GAMEPLAY-6E0B44-MASKS-CLI EUR/Wine (2026-10-31).** Production CLI
creates and reuses e100-d99 and advances one neutral frame: 91 files identical to
accepted dataset, 90 runtime identities verified and managed files unchanged. Masks
are not read in the frame; ordinary use and Linux/USA remain open. No performance
measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-05-GAMEPLAY-6E0B44-MASKS-ORDINARY-PAIR EUR/Wine (2026-10-31).**
Retained headgear list reads typed masks 91 times (364 aggregate bytes). Raw ROM/e100
comparison over 560 frames matches state/clock, video, OAM, dispatch and audio; both
sides share native cache. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-DYNAMIC-FRAGMENT-USAGE-CLI EUR/Wine (2026-10-31).** Dynamic fragments
expose separate counters for requests, selector bytes and payload. One neutral frame
of Windows CLI under Wine emits the new line with zero counters, reusing e100 dataset
without changes; ordinary use, Linux and USA remain open. No performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).


**ROMDATA-01-E100-UNOWNED-ROOT-CENSUS EUR candidates (2026-10-31).**
A bounded search in the historical 6b3428..6df2dc range, excluding ranges
already owned by the e100 manifest, finds 71 addresses with possible Thumb
PC-relative readers. These are static candidates: actual instructions, extents,
format and ordinary use remain to be demonstrated. The count does not indicate
migrated resources or overall coverage; no performance measurement.


**ROMDATA-01-E100-ROOT-6B9B80 conditional EUR source (2026-10-31).**
Original readers use OBJ directories and compact sequences with u16 offsets.
Selectors 0..7,9,10 attested by callers cover 174 frames and 232 objects,
2.028 bytes in five ranges without overlap with e100 owners. Sequence 8 and ten
additional lists remain physically consistent candidates without witnesses;
no guest limit is imposed. Model, runtime, ordinary use, Linux and USA remain
open. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).


**ROMDATA-02-E100-ROOT-6B9B80 EUR/Wine model (2026-10-31).**
The typed model reconstructs the 2.028 attested bytes, verifies the source and
preserves semantics outside the projection. A Wine test verifies boundaries,
JSON, tampering and original prefixes for 10 sequences, 174 frames and 61 lists.
Publication/runtime, ordinary use, Linux and USA remain open; no performance
measurement. [Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).


**ROMDATA-03-04-E100-COMPACT-OBJ EUR/Wine (2026-10-31).**
Generation e101-d100 publishes the five attested OBJ ranges: 90 previous
resources identical, bytes and mixed boundaries verified, transactional rejection
of modifications and native/interpreter comparison of one original prefix for
10 managed selectors plus fallback 8. CLI, ordinary use, Linux and USA remain
open; no performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).


**ROMDATA-04-E101-COMPACT-OBJ-CLI EUR/Wine (2026-10-31).**
The production CLI creates and reuses e101-d100, retains 92 files identical to
the accepted dataset and recognizes 91 resources on startup of one neutral frame.
Compact OBJ counters are zero in that frame; ordinary use, Linux and USA remain
open. No performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).


**ROMDATA-01-E101-ROOT-6C6460 conditional EUR source (2026-10-31).**
Six original callers attest selectors 0/11/12/13 of the compact OBJ format:
29 frames, 19 lists and 30 objects, 334 bytes in five ranges without e101
conflicts. The rest of the physical structure has no caller witness; model,
runtime, ordinary use, Linux and USA remain open. No guest limit added or
performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).


**ROMDATA-02-E101-ROOT-6C6460 EUR/Wine model (2026-10-31).**
The sparse model authenticates and reconstructs 334 bytes while excluding gaps;
Wine verifies bytes, boundaries, modifications and original readers for 4 sequences,
29 frames and 19 lists. Runtime, ordinary use, Linux and USA remain open;
no performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).


**ROMDATA-03-04-E101-SECONDARY-COMPACT-OBJ EUR/Wine (2026-10-31).**
Generation e102-d101 publishes 334 attested bytes in five ranges; 91 previous
resources identical, boundaries and transactional rejection verified, native/
interpreter comparison of one prefix for 4 selectors and fallback 1. CLI,
ordinary use, Linux and USA remain open; no performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).


**ROMDATA-04-E102-SECONDARY-COMPACT-OBJ-CLI EUR/Wine (2026-10-31).**
The CLI creates and reuses e102-d101, retains 93 files identical to the accepted
dataset and recognizes 92 resources on startup of one neutral frame. New bank
counters are zero in that frame; ordinary use, Linux and USA remain open. No
performance measurement. [Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).


**ROMDATA-01-E102-ROOT-6BF490 conditional EUR source (2026-10-31).**
Five original callers attest selectors 2/3: 11 frames, 8 lists and 7 objects,
including an empty list, 108 bytes in five ranges without e102 conflicts.
The rest of the structure is not attested by callers; model, runtime,
ordinary use, Linux and USA remain open. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).


**ROMDATA-02-E102-ROOT-6BF490 EUR/Wine model (2026-10-31).**
The sparse model authenticates and reconstructs 108 bytes, including the empty
list; Wine verifies bytes, boundaries, modifications and original readers for
2 sequences, 11 frames and 8 lists. Runtime, ordinary use, Linux and USA remain
open; no performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).


**ROMDATA-03-04-E102-TERTIARY-COMPACT-OBJ EUR/Wine (2026-10-31).**
Generation e103-d102 publishes 108 attested bytes in five ranges; 92 previous
resources identical, boundaries and transactional rejection verified, native/
interpreter comparison of one prefix for 2 selectors and fallback 0. CLI,
ordinary use, Linux and USA remain open; no performance measurement.
[Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).


**ROMDATA-04-E103-TERTIARY-COMPACT-OBJ-CLI EUR/Wine (2026-10-31).**
The CLI creates and reuses e103-d102, retains 94 files identical to the accepted
dataset and recognizes 93 resources on startup of one neutral frame. New bank
counters are zero in that frame; ordinary use, Linux and USA remain open. No
performance measurement. [Evidence](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).
