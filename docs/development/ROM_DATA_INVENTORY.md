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

Independent reader 128914/literal 128994 and successor reader A1F8/literal A234 bound
D68FD4..D69BC4: 3056 B, a self-delimiting 840 B/210-slot relative directory, 210 u16
counts/420 B, 856 XY pairs/1712 B and 42 FF alignment spans/84 B. All physical storage is
covered once; these do not establish global live index/count/coordinate caps. Original
low16 index/low8 destination/buffer+80 count-low8 return and live map-height query
1DD88 remain distinct. 68 primary-script links reach 56 paths: 66 opcode 44 low8/ignored
byte 3 and 2 opcode 45 u16. Actual archive/command dependencies are checked; source
is disjoint from all 46 e52 owners and shares only the D68 page with earlier owners.

The strict typed model reconstructs/hashes all 3056 B. One warning-free Windows
build and one Wine source oracle pass 24 guards/all independent fields/194 halfword/
5 word fingerprints, 212 original path copies (all 210 lists/two changed live map
cells), 13 raw-index/destination and 13 producer prefixes. 2972 source B observed;
84 alignment B preserved-unread. 40116 instructions/134 PCs coverage only, no native/
full-state-clock/whole movement-task/allocator/ordinary claim. Model source stage
closed; managed publication/direct runtime/CLI/ordinary remain pending, e52-d51/
USA e8-d7 unchanged. No new fixture/navigation/Linux-USA-native Windows/performance
check. [Source/model contract](../rom_data_scripts.md#european-script-coordinate-paths-source-contract).

Managed e53-d52 now publishes 48 files/47 resources/all 46 prior decoded/e52 retained,
USA e8-d7. Actual archive/command links authenticate before one shared typed
index/count/XY/alignment owner activates; D68 merges text/key/coordinate fields,
raw D69BC4 outside retained, guest height/control/ABI unchanged. Reduced Wine
managed/native checks pass 14 guards/lifecycle/all 3056 B/210 paths/856 points/42 alignment
and 24 original-interpreter-compiled full-state-clock cases/four full copies/live
height/all four classes[28,85,76,2]B/mixed owners/edges-BIOS-SWI-DMA. New-test array/
observer errors corrected with evidence retained; successful checks reused.
Production CLI create/reuse-no-write 50 files/old cache/all 48 generation equality and
one 6000-frame genuine EUR ordinary pair pass every full-state-clock-image-OAM-
dispatch-audio with 28 aggregate B = 4 index/14 count/10 XY/one path, 0 alignment; 581 guards.
Other 209 paths/42 alignment targeted-only; whole movement/allocator ABI/new 32:9/
Linux-USA-native Windows unverified, no performance measurement. [Runtime evidence](../rom_data_scripts.md#coordinate-path-production-cli-and-ordinary-scene).


## European auxiliary script archive source projection

Independent reader A1F8/root D69BC4 uses u8 action/slot and two levels of relative
u16 offsets, separate from the primary u32 archive. The declared 4098 B projection
through D6ABC6 has 8 banks/442 slots/126 distinct targets/30 interior entries,
133 physical programs/913 instructions/2275 parameter B/900 index B and 5 internal FF gaps
10 B. Root walks alone miss 69 physical instructions. All 489 relative branch links
resolve to instruction boundaries; structural 908/913 coverage is not ordinary
reachability, and the five remaining instructions stay preserved.

Separate 17x6 B command directory 370F94..370FFA supplies advance u16/Thumb callbacks;
its successor 370FFC is independently consumed. Its two zero-gap B and the last
archive FF 2 B remain unowned. Graphics base D6B4C8 is a ceiling, not archive end;
2306 B after D6ABC6 are excluded. Reassembly/hash/all 4200 B/723 halfword/4 word/static
link formulas/all 47 e53 owner disjointness pass. No build/guest test was run for
source inventory. Strict model/one Wine oracle are verified below; managed/runtime/
CLI/positive ordinary remain pending; no guest action-slot-opcode caps, primary-archive dependency or
whole ABI claim. e53-d52/USA e8-d7 unchanged, new Linux-USA-native Windows pending.
[Contract](../rom_data_scripts.md#european-auxiliary-script-archive-source-contract).


The strict auxiliary directory/archive model now reconstructs all 4200 B without
a ROM shadow; typed program+instruction entries preserve 30 interior links and 133
physical programs. All nested JSON/source identities/own-callback+advance
dependencies are checked. Warning-free Windows builds and the final new Wine
source filter pass 40 guards/442 lookups/913 dispatch+one RAM opcode 3/80 advance/
489 relative suffixes/10 wrap/15 raw/11 wrapper probes. 2887 distinct canonical B are
observed; 1297 other params+10 gaps+6 zero-row B remain preserved/hash-checked unread.
First raw expectation fails on odd LDRH; corrected test-only observer, then a
specific 32-bit high-rotation case added. First failure/earlier narrower passed
retry retained, unchanged model/core reused; only frontend/filter affected.
31839 instructions/177 PCs coverage only; no whole predicates/interpreter/native/
full-state-clock/ordinary claim. e53-d52/USA e8-d7 unchanged. [Model evidence](../rom_data_scripts.md#typed-auxiliary-archive-and-reduced-wine-source-oracle).


Managed EUR e54-d53 now publishes both auxiliary directory/archive resources:
50 files/49 resources/all 47 prior decoded/e53 retained/USA e8-d7. One shared immutable
owner serves 4200 B after actual archive→own directory authentication, with typed
index/callback/advance/opcode/parameter/gaps and no ROM shadow/host interpreter/
guest cap/save-native ABI change. Shared 370 audio/D69 coordinates and raw 370FFA+/
D6ABC6+ remain correct. One warning-free affected Windows build/two exact new
Wine filters pass: the first covers 26 guards/lifecycle/all 4200 B; the second covers 44 original-interpreter-
compiled complete-state-clock prefixes/all 6 classes[6,65,18,19,15,2]B/unaligned
u32 LDRH/interior entries/signed cursor/raw opcodes/widths-mirrors-BIOS-SWI-DMA.
624 instructions/92 PC/52 dispatches coverage only; whole predicates/interpreter/
callback ABI/ordinary unclaimed for that targeted scope. Actual production CLI
create/reuse-no-write 52 managed files/old cache/all 50 file equality and one 6000-frame
positive ordinary Wine full-state-clock-image-OAM-dispatch-audio pair now passes:
96 directory B = 32 advance/64 callback/6 commands, 82 archive B = 16 index/20 opcode/46 parameter/
2 banks/2 programs, 0 alignment/640 guards. Other 11 commands/6 banks/131 programs/5 alignment
spans targeted-only, new Linux-USA-native Windows/32:9 pending without performance/
space work. No new fixture/navigation or old proof repeat.
[Runtime and ordinary](../rom_data_scripts.md#auxiliary-script-production-cli-and-positive-ordinary-scene).


## European conditional normal script-special pose model

The 151 low-selector script references/20 logical metadata14 producer witnesses
retain 108 physical keys/12 roots/32 sequences/241 frames/11 OBJ/64 objects/95 pixel requests.
Strict lossless model reconstructs 41802 B and independently validates overlapping
initial/normal shared views before subtracting their union; only 33 spans/6436 B
have new ownership. One warning-free affected Windows frontend test build and
single reduced new Wine source oracle pass 18 guards/120 constructor prefixes/
53 null/7 proper resets/17 scheduler-real DMA3 uploads/all 5248 new pixel B/whole VRAM.
No old commands 2/6/8 or source/scene/census oracle repeated. Conditional source/
model closed; managed/runtime/CLI/ordinary and alternative/override/high registry/
current-state coverage pending. e61/USA e8 unchanged; new Linux-USA-native Windows-
32:9 pending, no performance/storage measurements.
[Model and scope](../rom_data_unit_obj.md#typed-normal-script-special-pose-model).


The conditional normal script-special owner now publishes/consumes EUR e62-d61:
58 files/57 resources/four actual metadata14-scriptarchive-initial-normal dependencies,
all 56 prior decoded files/resources/coverage and e61 retained, USA e8 unchanged.
One warning-free Windows core/frontend build (core compiled only) and two new Wine
filters pass: first 29 guards/lifecycle/all 6436 new typed B/seven sequences and 23 original-
interpreter-compiled full-state-clock paths/33 edges-mirrors-BIOS-SWI-DMA. Shared
initial/normal owners stay intact under 41802 graph B poisoning; new-role bytes
0 dir/240 index/28 header/920 frame/0 OBJ/5248 pixel, native API 0/36/6/43/0/1. Mixed 2 new/4 normal
transfers and 2 new/2 raw API preserve exact charges. No old commands/source/census/
scene repeat. Actual production CLI 58 generation/60 managed/all 58 match accepted/no-write
reuse and one-frame old cache attachment pass; zero new use/smoke only. Ordinary source/
new pixel and new Linux-USA-native Windows-32:9 pending, without measurements. [Managed runtime](../rom_data_unit_obj.md#conditional-script-special-normal-managed-runtime).


New actual e62 ordinary locators distinguish typed null-source use from pixels:
Snow 6000 reads 48 new index B/zero sequences-pixels; retained RIVER start 618 reads 0 new B.
No full pair/guard build/old census repeat. New seven-sequence static 81 snapshot scan
finds no matching task; program 159 contains a producer but actual trigger remains
unproven. Ordinary new sequence/pixel/equivalence stays open, original/cart inputs
preserved and no performance/other platform work. [Evidence limits](../rom_data_unit_obj.md#script-special-ordinary-source-locators-and-remaining-pixel-gate).


The alternate script-special branch now has a separate source projection over the
same 151 references/20 low-selector witnesses, metadata14 half 06 and original
current alternate assignment with override 20 clear. 84 cells/68 null/14 sequences
64 frames, 9048 B total; 20 new spans total 816 B: 216 index, 8 header, 80 frame
and 512 pixel bytes, with the remaining 8142 alternate + 38 initial + 52 normal
bytes shared. The offline model, warning-free Windows build and single Wine filter pass 19 guards, 108 constructor prefixes/52 null + 2 proper
resets/two real DMA uploads and 512 pixel B/full VRAM exact; initial sandbox Wine
startup before test retained, same binary retry PASS. e62-d61 runtime unchanged.
Ordinary new pixels, high16, override and current state remain explicit limits.
[Contract](../rom_data_unit_obj.md#conditional-alternate-script-special-source-and-typed-model).


EUR e63-d62 publishes/consumes the alternate script-special model: 59 files,
58 resources, actual metadata14/script archive/initial/normal/alternate, 57 previous
data/records/coverage unchanged/e62 retained/USA e8. New owner serves only 20 spans
816 B; 31 guards/lifecycle/all 816 typed B and 24 Wine original-interpreter-compiled
full state/clock paths/all 20 boundaries/BIOS-SWI-DMA/mixed owners/raw pass. Real
CLI/ordinary/high16/override/current actor and new Linux/USA/native Windows/32:9
remain to verify, no performance/space measurements/new fixture.
[Contract and evidence](../rom_data_unit_obj.md#conditional-script-special-alternate-managed-runtime).


EUR e63-d62 production CLI now verified on Wine: create/reuse 59 generation 61 managed,
all 59 match the accepted data, with no writes on reuse; canonical data exactly
restored/old-cache one-frame PASS.
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
witnesses/156 cells/11722 B. Shared 6252 normal + 4420 alternate bytes; 40 new spans
total 1050 B: 384 index, 24 header, 360 frame, 26 OBJ and 256 pixel bytes. The six
sequences/18 frames include two new op 2 argument copies. Offline model/warning-free Windows build/
single new Wine filter first PASS 19 guards/192 constructor aliases/90 null + 6
proper resets/two op 2 argument copies/two DMA uploads 256 pixel B/full VRAM.
e63/USA e8/runtime unchanged; no invented static high-registry identity.
[Contract and limits](../rom_data_unit_obj.md#conditional-forced42-and230script-pose-source-model).


EUR e64-d63 now publishes/consumes script-override 42/230: 60 files 59 resources,
three dependencies: script archive/normal battle/alternate battle; all 58 previous/e63/USA e8
unchanged. Owner: 40 spans/1050 B; 31 guards/lifecycle/all typed B and 27 Wine
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
1908 cells/373842 B; 358566 B shared from 6 owners, 272 new spans/15276 B (2392 index,
84 header, 1920 frame and 10880 pixel bytes), 21 sequences/96 op 1 frames/37 pixel requests.
All 59 e64 owners and 41 halfword 4 literal 8 BL pins pass, no target root/mixed
item/conflict/variable extent. Lossless model and first targeted Wine test pass
26 guards/1196 constructor aliases/567 null + 21 resets/37 DMA/10880 B/full VRAM,
six previous views independently validated/consistent physical aliases. Corrected
warning-free frontend build; EUR e65-d64 publishes 61 files 60 resources with
metadata14/archive and 6 actual previous views. Two targeted Wine filters first PASS
33 attach + 10 dependency guards/lifecycle/all 15276 typed B/21 sequences and
25 original-interpreter-compiled full state-clock paths/272 edges/BIOS-SWI-DMA/
mixed old-raw-new. Real Wine CLI prepare/reuse 61 generation 63 managed/all 61
accepted/no-write/one-frame old cache attach first PASS (all new counters 0/smoke
only); ordinary new pixels/equivalence pending, USA e8 unchanged. New readonly
locator 81 snapshots/21 new sequences finds 0 active or dormant tasks; lead closed/
no guest/build/fixture/full pair or retry; inputs unchanged. Registry identity/generic job/
out of catalog/redirect/current changed remain separate. No measurement or new fixture.
[Contract](../rom_data_unit_obj.md#registry-selectors-and-known-special-unit-source-projection).


High-registry generic unit 0/1 has a separate contract for 16 high refs/seven pairs
and metadata52 route 0 with concrete field per mode. CCC48 reads job+5/alternate+7,
properties 4-5 skip RAM property 0 override; CF getters zero-extend u16. All 116 rows
classified: 100 both concrete/10 normal only/2 both FFFF/4 route FF, 110 normal
100 alternate → 2940 witnesses/6720 refs; initial 110 joint assumption corrected
before targets. 83 halfword/6 literal-dispatch/11 BL pins and all 60 e65 owners
authenticated before targets. 107 sprites/535 pairs/3210 keys/cells/2420 null/
777 proper sequences/5093 frames/45 OBJ lists/231 objects/1895 pixel requests/677280 B;
12 initial index-root targets 48..59 with exact-field witnesses. Full 796992 B/shared
697168 B, 659 new spans/99824 B (5544 index, 504 header, 11760 frame, 82016 pixel),
1386 new cells/1260 null/126 new sequences/588 op 1/268 pixel requests. No new OBJ/
mixed item/role/extent conflict. Lossless model and first targeted Wine pass 34 guards/
2772 new-cell aliases/1260 null/126 proper resets/268 DMA/82016 B/full VRAM/12 initial
root views; six previous views independently validated. First warning-free Windows
frontend build; EUR e66-d65 publishes 62 files/61 resources/metadata52-archive +
6 previous views. First two targeted Wine filters pass 36 attach + 10 dependency
guards/lifecycle/all 99824 typed B/126 new sequences and 25 original-interpreter-
compiled full state-clock paths/659 edges/BIOS-SWI-DMA/mixed old-raw-new. Real Wine
CLI create/reuse: 62 generation/64 managed/all 62 accepted/no-write/one-frame old
cache attach first PASS (all new counters 0/smoke only); ordinary new pixels/
equivalence pending/USA e8 unchanged. New readonly locator 81 snapshots/126 new
sequences finds 0 active/dormant body tasks without current-pointer/end-of-frame
filter; lead closed with no guest/build/fixture/full pair or retry; inputs unchanged. FFFF/
redirect/out of catalog/changed actor/global identity and other platforms separate;
no old proof replay/fixture/navigation/measurements.
[Contract](../rom_data_unit_obj.md#registry-selectors-and-generic-direct-job-source-projection).

The high-registry contract also covers current jobs 80/81/82/85 route FF with known
alternate job route 0/concrete field: 110 normal/100 alternate → 11760 witnesses/
26880 refs. The primary source is FF routing, sprite field comes from alternate row;
98 halfword/6 literal/11 BL pins verify CC35A/unchanged r4/return to same property.
All 116 rows and 928 actual classifications retained (840 concrete/56 FFFF/32 loop).
All 61 current resources/sources authenticated before targets: each of 2940 direct
witnesses has four preimages, same 3210 cells/777 sequences/12 root views/full
796992 B/seven owners. Zero new bytes or resources; e66-d65 unchanged, metadata52/e66
Wine proofs reused without build/guest/fixture/navigation. Ordinary/global identity/
FFFF/nonreturning/out of catalog/changed actor and Linux/USA/native Windows/new 32:9
remain open.
[Redirect and limits](../rom_data_unit_obj.md#registry-generic-jobs-redirected-to-a-known-alternate-job).

Known FFFF results now have a separate reader classification: 196 direct + 784
redirected witnesses/2240 refs, 71 reader pins/three code blocks/two literals.
Property read 083B4C12 returns DC86, directory 083D4830 returns 14AE3F1E;
60 queries/30 words remain in unmapped region 14. CPU aligns/rotates 16, MemMap
uses livebios_open: no ROM sequence/constant null or universal nonreturn proven.
The two ROM reads total 6 B without typed owner; raw fallback retained, 6 B migration/
ordinary reachability still open. No invented ID cap/pointer fix/fake counted asset
or build/guest/fixture/navigation/measurements, e66/USA e8 unchanged.
[Reads and limits](../rom_data_unit_obj.md#known-generic-job-ffff-results-and-raw-bus-reads).

Actor property reader 221C4 now has a distinct source contract: 394C14+2*sprite,
u16→9AD0E/r8→constructor r3→allocation request/task+16. 398 normal/alternate
job-special-override witnesses derive 182 queries before targets; 98 halfword/15 BL/
four code blocks/one literal and 61 owners/files/sources pass. Four new disjoint
ranges totaling 364 B, SHA ad328c8ebccfc43091dc878c06dab33b22521d7b617d4b11232e4a532ffc2208;
139 words 16/43 words 24, all u16 bits retained, raw gaps. No inferred catalog 248/
ID cap, flag bitfield or VRAM unit. Source child and lossless model/Wine oracle now
closed: 20 guards/all 182 getters/364 B/3 raw fullword/four new-task prefixes/two
reuse/four u16 setters/whole RAM guards pass. Existing tasks retain their own +16.
First warning-free Windows frontend build, 82 build/83 Wine guards/11 compiled inputs
unchanged; no full allocator/native-clock/ordinary claim for 02. EUR e67-d66 now
publishes 63 files/62 resources/metadata52-14 dependencies/all 61 previous-e66 retained.
First two Wine filters pass 17 attach + 4 full-set guards/lifecycle/364 typed B/
182 mask records and 16 original-interpreter-compiled full state-clock paths with
364 raw B altered. Four edges/width-mirrors/real BIOS bus/SWI-DMA/mixed new-old-raw
pass; 84 build/86 Wine guards/20 compiled inputs unchanged. Production CLI create/
reuse 63 generation/65 managed/all 63 accepted/no-write/one-frame old cache attach
first PASS (all new counters 0/smoke only), 154 active guards unchanged. New ordinary
F09 Wait 3530 CLI/full pair first PASS with 10 reads/20 aggregate B/two words; every frame
state-clock-central-wide-OAM-dispatch-all audio equal; 155 active guards/21 compiled
inputs unchanged, other 180 words ordinary targeted only. Optional positive guard,
old normal scene child remains closed/no other proof repeated. No full allocator/
new 32:9 claim, USA e8 unchanged. FFFF raw 6 B/other inputs/changed actor/global identity
separate, no repeat of old proof/fixture/navigation/measurements.
[Source and limits](../rom_data_unit_obj.md#actor-property-words-from-known-sprite-assignments).


The contract for additional OBJ property getter callers now also covers the full
byte input window of the two UI LDRBs: 256 queries chosen before targets, not a
catalog extension or fullword getter limit. Builder and four missing paths authenticated
with 680 halfword/25 BL/21 literal-control-flow pins. The two u16 paths update the
property while retaining allocation; known concrete assignments reuse the 182 words.
All 62 owners/files/sources verified: 184 shared words (182 properties + two words from
the four palette offsets), 72 new words/144 B in 394CF4/2, 394CFE/2, 394D0C/128,
394E08/12, SHA 9d1239cc29ec40185c1605aed84b4cf74cedbc21cc217c001efb8348407253f9.
New complement with source contract and lossless model/Wine oracle closed: 14 guards,
512 UI prefixes/all 512 B, eight skips, four high-u16 comparisons, four byte writers
and six setter prefixes pass on first attempt. RAM outside destinations and VRAM/OAM/
palette verified; 109 build/110 Wine guards/13 compiled inputs unchanged. The 184 shared
words come from the two dependencies without duplicate copies. Managed-runtime/CLI/
ordinary pending, e67-d66 and USA e8-d7 unchanged. FFFF raw 6 B/unknown/changed actor/
global identity/computed indirect remain separate; no old proof replay/full UI-native
clock/fixture/navigation/measurement.
[Details](../rom_data_unit_obj.md#additional-property-getter-callers-and-the-ui-byte-input-window).


The OBJ property byte-input complement is now managed and consumed in EUR e68-d67:
64 files/63 resources, all 62 previous records/63 files and e67 retained. Only 72 new
words/144 B; 184 shared words remain with the propertywords and colorpalettes owners,
real dependencies. Two new Wine filters pass 17 attach + four dependency checks/
lifecycle/all 144 B and 72 records, plus original/interpreter/compiled full state-clock
equivalence on getter/UI prefix/mixed/fullword/live RAM. Four edges/width-mirrors/
real BIOS bus/SWI-DMA pass: API 2 new/2 property, 2 new/2 palette, 2 new/2 raw; CpuSet
2 new/4 property; FastSet 28 new/4 property; DMA 4 new/2 palette. No source/model/old
scene replay or measurement; production CLI/ordinary remain pending, USA e8 unchanged.
[Consumption and limits](../rom_data_unit_obj.md#managed-ui-byte-property-words-and-typed-runtime-reads).


The e68-d67 byte-property complement production CLI now passes first Wine prepare/
reuse: 64 generation/66 managed files/63 resources/two dependencies/all 64 equal to
dataset 03, no writes on reuse from different cwd. First warning-free Windows build/
frozen 0c0f5082.. and canonical hash+mtime restored; main content unchanged. One
authentic F09 frame loads previous native cache and uses 1 read/2 B/1 new word, positive
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
B2/14 B already typed are an exact alias. Three additional references project 126 refs/
74 lists/175 objects/1198 B in the same owner `graphics.obj-static-lists`, without a new
family/format. Source closed, model/publication/runtime/CLI/ordinary A4 pending;
e68-d67/USA e8-d7 unchanged. No old CFG campaign/oracle/scenes repeated, build/guest/
fixture/navigation or measurement.
[Details](../rom_data_obj.md#property-selected-ui-list-source-extension).


**ROMDATA-02-03-UI-PROPERTY-OBJ-LIST-EXTENSION completed EUR (2026-10-26).**
e69-d68 integrates three references in the same Catalog/List/Object/owner/format,
126 refs/74 lists/175 objects/1198 B; new A4 only 14 B/two objects, B2 shared. Same
64 files/63 resources and size dependency; all 73 previous records, 62 other files/
records and e68 unchanged. Corrected warning-free Windows build (first error only an
unqualified test constant). First two Wine filters pass: roundtrip/eight guards/
publication-repair-move management/lifecycle; two original-interpreter-compiled
draws full state-clock/709 instructions/4 dispatch/36 compiled B, two host
publications, edges-width-mirror-BIOS and mixed API 2 new/2 B2, DMA-CpuSet 14 new/
2 B2, FastSet 4 prior/14 new/14 B2. Runtime handlers unchanged. 77 build/79 Wine
guards/29 compiled inputs exact. Real CLI 04 and ordinary A4 guest-host 05 still to
do; USA e8-d7/Linux/native Windows/new 32:9 pending. No old evidence/scene replay,
fixture/navigation or performance-space measurement.
[Integration and limits](../rom_data_obj.md#managed-ui-property-list-extension-and-typed-consumption).


**ROMDATA-04-UI-PROPERTY-OBJ-LIST-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen 17488009..; canonical restored
hash+mtime, main content unchanged. First Wine prepare/reuse/old-cache one frame
from /tmp pass: 64 generation/66 managed/63 resources/74 spans/1198 B/all 64 equal to
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
two ranges totaling 6 B, 980 conditional witnesses/2240 references (196 direct/784 redirect)
with metadata52/archive49 dependencies. Previous valid models and byte aliases
unchanged; no interpreted root/sequence/corrected pointer/cap. First warning-free
Windows build/frozen 83ffcd75.. and first single Wine filter pass 13 guards/3 numeric
roundtrips/980 records/6 B and two complete original readers/28 instructions,
registers/RAM guard. 156 build/157 Wine guards/35 compiled inputs exact. 71-pin/
60-query classification and old A4/model/runtime/scene evidence closed without
replay. Publication/owner runtime 03, native clock/ordinary pending; production
e69-d68/64 files/63 resources and USA e8-d7 unchanged. Linux/USA/native Windows/
new 32:9 to do, no fixture/navigation/performance-space measurement.
[Model and limits](../rom_data_unit_obj.md#typed-model-for-known-ffff-reader-fields).


**ROMDATA-03-UNIT-OBJ-FFFF-READER-FIELDS managed/runtime completed EUR (2026-10-26).**
e70-d69: 65 files/64 resources, all 63 previous decoded records/files and e69
preserved; USA e8-d7 unchanged. New numeric resource with real metadata52/script
archive dependencies, model 980 witnesses/2240 refs unchanged. Immutable owner
serves only 2 B property + 4 B directory from typed fields; gate 3B4/3D4, counters and
two bit fields. Full pointer, live open BIOS/unaligned LDR rotate 16 and guest
control preserved, no invented sequence/target/null/cap. First warning-free Windows
core/frontend build and first two new Wine filters PASS: 20 attach guards, four
dependency guards, publication/lifecycle unavailable directory, all 6 B raw altered/
two fields and distinct old owner. Original/interpreter/native 14 paths equal full
state/clock: getter/directory/zero early return/three BIOS prefixes, live RAM/mixed
reads. Two edges/width/mirrors/real BIOS bus; API 2 new/2 raw, CpuSet 2 new/6 raw,
FastSet 4 new/28 raw, DMA 4 new/4 raw preserved. Source/model/A4/old proof closed
without replay. Production CLI 04 and ordinary 05 remain to do, no full sprite/
allocator ABI claimed. Linux/USA/native Windows/new 32:9 pending. No fixture/navigation/
private payload/performance-space/Android/package/commit/push.
[Consumption and limits](../rom_data_unit_obj.md#managed-ffff-reader-fields-and-typed-bus-consumption).


**ROMDATA-04-UNIT-OBJ-FFFF-READER-FIELDS-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen candidate; canonical exact hash+
mtime restored/main content unchanged. First three Wine calls prepare/reuse/old-cache
one frame from unrelated cwd pass: 65 generation/67 managed/64 resources/all 65 equal
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
Root 399750 is shared by six readers D0538/D0548/D0568/D0588/D05A8/D05C8. They read
the job byte or CBEC8 properties 1/4/6/9/0, with original truncations and name helper
165E8. Before reading data, 12CA98 selects an index from 20 RAM slots at 02002E78
with stride 16; mode 3 uses maximum +5 with an inclusive tie. One occupied slot can
permit any index 0..19. R6 is preserved through the CFF70 clear before D05C8.
The evidence authenticates 268 halfwords, 27 critical pins, 9 literals and 8 BLs.
It does not establish a global cap, full physical extent, adjacency or game identity;
other raw caller matches are not automatically semantic code. The projection contains
20 distinct u8 jobs over 20 B in one range, SHA 969b2cd5.., and reconstructs 20
metadata52 rows disjoint from all 64 current owners. No sprite target was followed.
The source child is closed; model, Wine oracle, managed/runtime, CLI and ordinary use
were open at this checkpoint. e70-d69 (65 files/64 resources) and USA 8-7 were
unchanged. No build, guest execution, test, fixture, navigation, private payload,
performance/space measurement, Android, package, commit or push. Linux, USA, native
Windows and new 32:9 remain pending.
[Contract and limits](../rom_data_gameplay.md#twenty-slot-job-lookup-source-contract).


**ROMDATA-02-SLOT-JOB-LOOKUP-MODEL model completed EUR (2026-10-26).**
The strict model records 20 (slot,u8 job) entries, 20 B, source base/hash, byte_at
and direct metadata52. Catalog checks cover EUR and three producer-reader blocks.
Only two module declarations were added, without Extraction/runtime registration:
e70-d69 (65 files/64 resources) and USA 8-7 were unchanged. The first warning-free
Windows frontend build and first single Wine filter passed: 20 independent entries,
20 B, 12 JSON-shape/source/dependency guards, roundtrip and disjointness from all
64 owners. The proof covers 123 original paths: 21 bare job reads, 81 complete
property wrappers and 21 property 0 prefixes before 165E8, plus three high input
aliases. All 20 B at the six original LDRB reads, the callee, full RAM outside the
20 B, stack and VRAM-OAM-palette were preserved. No guest cap of 20, null or host
fallback was introduced. The code parser's 268 pins and metadata52/FFFF/A4/old
proofs were not repeated. Managed/runtime 03, CLI 04 and ordinary 05 were open at
this checkpoint. No new fixture, navigation, private payload, performance/space
measurement, Android, package, commit or push. Linux, USA, native Windows and new
32:9 remain pending.
[Model and limits](../rom_data_gameplay.md#typed-twenty-slot-job-lookup-and-original-reader-proof).


**ROMDATA-03-SLOT-JOB-LOOKUP-RUNTIME managed/runtime completed EUR (2026-10-26).**
Release e71-d70 contains 66 files and 65 resources, including
gameplay.slot-job-lookup/slot-job-lookup.json and its real metadata52 dependency.
All 64 previous records/files and e70 were preserved; USA 8-7 was unchanged. The
immutable owner covers 20 u8 entries/20 B on page 399 with a 20-record mask. It
preserves original low 8 input bits, slot selection, clear, properties, name and raw
outside reads, without a guest cap of 20. After an ENOSPC stop and retry on a
temporary filesystem, the first warning-free Windows core/frontend build and first
two new Wine filters passed: 18 attach guards, 3 dependency guards, publication and
lifecycle with an unavailable directory, all 20 B of raw data altered, all 20 bits
and a distinct FFF owner. Sixteen original/interpreter/native paths matched full
state and clock: getter, properties, name ID prefix, high input, slot 20 raw, live
RAM and 12CA98 mode 3 slot 19 → CFF70 clear → D05C8 before 165E8. Edges, widths,
mirrors and the real BIOS bus were preserved; mixed API reads covered 2 new/2 raw,
CpuSet 20 new/4 raw, FastSet 20 new/12 raw and DMA 20 new/4 raw. The old 64 owners
confirmed raw neighbors. Source 268 pins, model 123 readers and old proof were
closed without replay. CLI 04 and ordinary 05 were open at this checkpoint; Linux,
USA, native Windows and new 32:9 remain pending. No fixture, navigation, private
payload, performance/space measurement, Android, package, commit or push.
[Consumption and limits](../rom_data_gameplay.md#managed-twenty-slot-job-lookup-and-typed-bus-consumption).


**ROMDATA-04-SLOT-JOB-LOOKUP-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
The first warning-free Windows production build produced a frozen candidate. The
canonical file's exact hash and mtime were restored, with main content unchanged.
The first three Wine calls for prepare/reuse/old-cache, one frame from an unrelated
working directory, passed: 66 generation entries, 68 managed entries, 65 resources,
all 66 equal to dataset 03, the metadata52 dependency and reuse without writes. The
private root was on a temporary filesystem via a workspace link after ENOSPC in 03;
there was no further cleanup, budget formatting or build retry. F09 and the original
cache were intact. This proves attachment only; positive ordinary use and equivalence
were still open. Source, model, runtime and old proof were closed without replay;
Linux, USA, native Windows and new 32:9 remain to verify. No fixture, navigation,
private payload, performance/space measurement, Android, package, commit or push.
[CLI and limits](../rom_data_gameplay.md#actual-production-cli-for-the-twenty-slot-job-lookup).


**ROMDATA-05-SLOT-JOB-LOOKUP-BANK-ORDINARY completed for declared EUR scene (2026-10-26).**
The first new CLI bank 120 run observed positive use: 2 reads/2 B/1 slot, with an
existing copied checkpoint, Down 8..20 input and production e71 dataset
generation-GmxCy2. The only new scope was job lookup; old A4 negative and B2
positive evidence remained closed. Only 5 optional guard/report lines changed in the
scene validator; runtime, product, old guards and states were unchanged. The first
warning-free Windows frontend build and first exact Wine filter passed: 120 full
state-clock comparisons from 3660 to 3780, central-wide/OAM/native-fallback dispatch
and bit-exact mixed PSG-FIFO left-right-HLE. All 5 sample counts
[263340,263340,31680,31680,263264] were positive; new use of 2 B in 1 slot matched
the CLI. The other 19 ordinary slots were targeted only. The inspected CLI capture
was 284x160: Bank background, with UI panels absent from the raw framebuffer. No
final composite GUI, full name-actor-allocator ABI or new 32:9 was claimed. Input,
copy, original, cache and dataset were intact; no fixture, navigation, private
payload, performance/space measurement or old proof replay. Source, model and
03/04/05 are closed within their declared scope; the full TODO remains active and
Linux, USA, native Windows and new 32:9 remain pending.
[Scene and limits](../rom_data_gameplay.md#ordinary-bank-scene-use-of-the-twenty-slot-job-lookup).


**ROMDATA-01-POST-SLOT-LOOKUP-READERS completed for three EUR RAM APIs (2026-10-26).**
D05E8 checks for any nonnull slot and D0618 counts slots by reading only the first
words of 20 RAM slots at 02002E78 with stride 16. They do not validate pointers or
liveness, and D05E8 returns early. D0650 increments byte 02001F68 only if old<=98,
leaving 99..255 unchanged; there is no global clamp. All ignore incoming arguments
for these addresses. Six literals support the executable RAM base/offset, with no
new ROM asset root. The proof verifies 68 halfwords, 30 critical pins, 6 literals
and three complete blocks; 3/2/1 raw BL matches remain candidates, not semantic
completeness. There is no live RAM export, new decoder-resource-owner, revision or
ROM table cap of 20. e71-d70 (66 files/65 resources) and USA 8-7 were unchanged.
The next partial D0670 was excluded. The source child closed without repeating old
evidence or Bank; no build, guest execution, test, fixture, navigation, private
payload or performance/space measurement. Linux, USA, native Windows and new 32:9
remain pending.
[Live state and limits](../rom_data_gameplay.md#live-state-helpers-following-the-slot-job-lookup).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE completed for the EUR Bank12 sample
(2026-10-26).** A test-only observer wraps the native read8/16/32 APIs and subtracts
20756 merged ranges from the 65 e71 source plans. The first warning-free Windows
build and one Wine filter report 3166 context reads/3683 image bytes; 198 reads
include 676 bytes outside the plans, with 79 ranges observed. Full state/clock,
central-wide/OAM/dispatch and five audio streams match at each frame 3660–3672.
The 676 bytes are leads to authenticate, including five words 370E70/74/80/84/EB0;
no new resource/revision or global completeness is claimed. Exported PC may be
stale; interpreter/HLE/DMA/native constants and other scenes are outside the
sample. All 73 input hash-mtime checks are exact; main/C API/old validators are
unchanged, with only a new test module and cfg declaration. No new fixture,
navigation or performance measurement. Old Bank 120/source/model/runtime/CLI
acceptances remain closed. Linux, USA, native Windows and new 32:9 remain to do.
[Contract, transparency and limits](../rom_data_runtime.md#bounded-native-rom-api-residue-inventory).


**ROMDATA-01-NATIVE-RESIDUE-HIGH-WORD-READERS statically completed EUR
(2026-10-26).** The five observed words 370E70/74/80/84/EB0 support execution:
two DMA register address directories and one Thumb IWRAM entry passed to BX r2.
The proof authenticates 392 halfwords/47 pins/5 literals/4 BL in six complete
blocks; actual loads 0846/09B2/0D56/3A7E/11DA/11EE are distinguished from exported
PC. Original callers prove two 4x4 B projections 370E68/78 and one word 370EB0,
36 B disjoint from 65 owners; 20 B observed are classified, with 656 B lower still
to classify. No global cap, RAM body/ABI/scheduler or new asset/owner/revision is
inferred; ROM/code remains an explicit dependency and 676 B remain outside managed
plans. No build, guest execution, test, fixture, navigation, measurement or repeat
of old acceptance. e71-d70/USA 8-7 unchanged; Linux, USA, native Windows and new
32:9 pending.
[Hardware directories and branch control](../rom_data_runtime.md#high-offset-native-residue-is-hardware-and-branch-support).


**ROMDATA-01-NATIVE-RESIDUE-LOWER-READERS completed for EUR static witnesses
(2026-10-26).** The 656 B observed comprise 158 literal words/632 B with 184
linear Thumb paths within 96 B of saved contexts (507 distinct halfwords), and 6
words/24 B in 5 jump tables with 82 authenticated halfwords of bounds/index/load/
MOV pc prefix. The projections have 8/35/69/48/5 entries; only 0/0/27/4-6/1 were
observed. This is not a dynamic instruction/mode trace or proof of all uses.
Together with the 20 B of high hardware/branch support, all 676 B outside plans
have static code-support witnesses; they remain outside the 65 owners and depend
on the original ROM. No new family/asset root/owner/revision or complete migration
is inferred. No build, guest execution, test, fixture, navigation, measurement or
repeat of old acceptance. e71-d70/USA 8-7 unchanged; interpreter/HLE/DMA/native
constants, other scenes and Linux/USA/native Windows/new 32:9 remain separate.
[Literals and jump tables](../rom_data_runtime.md#lower-native-residue-has-literal-and-switch-table-witnesses).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE-F09 completed for second EUR sample
(2026-10-26).** The same frozen observer executable was used without a build: one
Wine filter over 12 frames 8476→8488 from the existing F09 checkpoint/inputs
preserves state/clock/buffer/OAM/dispatch and five audio streams. It reports 13671
context reads/13961 B image, with 604 B outside plans, 584 shared with Bank and
20 new B/5 words. All 5 have linear Thumb PC-relative witnesses (20 halfwords):
static source, not a dynamic trace/mode or all uses. The union of the two samples
is 696 B still outside 65 owners, with no asset/revision inferred. All 76 input
hash-mtime checks and compiled code are exact; the retention helper found 0 paths.
No fixture, navigation, measurement or repeat of old F09/Bank acceptance. Other
interpreter/HLE/DMA/constant paths and Linux/USA/native Windows/new 32:9 remain
separate.
[Second sample](../rom_data_runtime.md#second-native-api-sample-from-the-existing-f09-checkpoint).


**ROMDATA-01-HOST-ROM-READ-RESIDUE completed for collector and EUR F09 sample
(2026-10-26).** The optional non-default Cargo feature rom-data-read-audit provides
a thread-local address/reader/length collector on the resolved bus and selected
host audio/sprite/tables/keyboard. Interpreter instruction fetch is excluded;
execution/clock/API/savestate are unchanged. The first build compiled core but
failed in the frontend because a test omitted c_void qualification; a test-only
correction produced a warning-free incremental build. One Wine filter covers the
ARM/Thumb/width/mirror/audio/lifecycle probe and 12 frames 8476→8488 with identical
state/clock/buffer/OAM/dispatch and five audio streams. It reports 23061 requests/
19984 B image, with 1146 B outside 65 owners; this includes all 604 B outside plans
from the old API report and adds 542 B. The 90 new high B/6 ranges remain sources
to authenticate; 64 B also come from SpritePeek. No format/asset/extent/owner/
revision is inferred. Hash-mtime checks are exact for 76 execution inputs + 1
comparison report; no old evidence, fixture, navigation or measurement. e71-d70/
USA 8-7 unchanged. KeyboardWidths was inactive; other threads/scenes, host accesses
and the typed HLE fast path are out of scope. Linux/USA/native Windows/new 32:9
remain pending.
[Collector, proof and limits](../rom_data_runtime.md#optional-bus-and-host-rom-read-audit).


**ROMDATA-01-BUS-HOST-HIGH-RESIDUE completed for EUR source contracts
(2026-10-26).** The 90 new B are classified: 64 B in two indirect OBJ lists
394F4C/26 B/4 objects and 40D89E/38 B/6 objects; 10 B of parameters (2 phase
thresholds, 4 signed vertical words); and 16 B of Thumb IWRAM control pointers.
The proof reuses 295 window halfwords/53 pins/7 literals and the ABI hash. Original
copy 0924E8 proves D968E8..D97728→03005F28..03006D68; constructor D9752C writes
list+158. Raw ROM disassembly at an empty RAM target was rejected. One existing
F09 producer-args Wine frame confirms 2 readers/10 objects; old equivalence was
not repeated. 40D89E from task+24 is an indirect observation, not an invented
literal; command creation was not reconstructed. Timing compares the OLD counter;
post-update phase 0..2 is not a global cap. The corrected byte>>1 query has 128
possible indices, with only 4..7 new here. The 64 B of lists + 10 B of parameters
remain to model/publish; the 16 B of control remain a ROM/code dependency, with
452 B lower separate. e71-d70/65 owners/USA 8-7 unchanged; no build, fixture,
navigation or measurement. Linux/USA/native Windows/new 32:9 pending.
[Indirect lists and evidence](../rom_data_obj.md#observed-indirect-obj-list-sources-beyond-the-direct-catalog),
[parameters and control](../rom_data_runtime.md#high-bus-and-host-residue-source-contracts).


**ROMDATA-02-OBSERVED-OBJ-LISTS-MODEL completed for two EUR lists
(2026-10-26).** The isolated obj_lists::observed API reuses List/Object: 2 lists/
10 objects/64 B. Identity/count/shape/field width/strict JSON/source consistency
are authenticated; the constructor-bound root is distinct from the ordinary-task
argument. The first warning-free Windows frontend build and one Wine filter cover
64 B/2 spans/14 guards/6 original paths at ROM 1500/170C and copied RAM 030065A8;
output/matrices/stack/callee/external RAM/OAM-VRAM-palette are exact. Canonical
JSON is 4530552b..; 75 input-artifact guards and compiled code are unchanged. Only
2 declarations were added to existing files; the old Catalog of 74 lists/175
objects/1198 B, e71-d70/66 files/65 resources and USA 8-7 are unchanged. No
publication/owner/revision or ordinary evidence is inferred: the model is closed,
while managed runtime/CLI/new use from file were still open at this checkpoint.
No old 74/ABI/copier/scene check was repeated and no new fixture, navigation or
measurement was performed. The 10 B of parameters and 452 B lower remain separate;
Linux/USA/native Windows/new 32:9 are pending.
[Model and checks](../rom_data_obj.md#isolated-typed-model-for-the-two-observed-indirect-lists).


**ROMDATA-03-OBSERVED-OBJ-LISTS-RUNTIME completed for two EUR lists
(2026-10-26).** e72-d71 integrates the two indirect identities into the existing
OBJ resource: 76 lists, 185 objects and 1262 source bytes. The 126 direct
references, 66 files/65 resources, same size dependency, all 74 previous lists,
other 64 files/records and e71 remain. USA 8-7 is unchanged. The first warning-free
Windows build and two Wine filters pass publication/repair/move, 12 attach guards
and Load/Undo/backup/restart; six original/interpreter/native paths have equal full
state and clock with all 64 new ROM bytes altered. Edges/widths/mirror/BIOS, six
DMA-CpuSet-FastSet transfers, API reads of 2 B old + 2 B new and 2 B altered raw +
2 B new, and six host publications pass. The 5571 instructions/2418 dispatch are
coverage only; native APIs served 265 typed bytes. The file exactly matches the
accepted model; 77 input/executable guards and compiled code are unchanged. Artifact
completion removes only interrupted/moved copies. Production CLI and new ordinary
equivalence were open at this checkpoint; ten parameter bytes and 452 lower bytes
remain separate. No old model/ABI/scene was repeated and no new fixture, navigation
or measurement was performed. Linux, USA, native Windows and new 32:9 remain to
verify.
[Integration and evidence](../rom_data_obj.md#managed-observed-indirect-lists-and-typed-consumption).


**ROMDATA-04-OBSERVED-OBJ-LISTS-ACTUAL-CLI completed EUR (2026-10-26).**
The first warning-free Windows production build produced separate CLI c6d674d7..;
the canonical executable hash/mtime were exactly restored, with main content
unchanged and mtime-only Cargo invalidation. The first three Wine commands from an
unrelated working directory pass create/reuse/old-cache one frame: 66 generation
files/68 managed, all 66 equal to the runtime dataset, 65 resources/76 OBJ spans/
size dependency. Reuse made no hash/mtime changes. With original F09/inputs/cache
at 16:9, both new lists 00394f4c/0040d89e have guest=true and host=true; OBJ totals
are 147 reads/147 B/14 B count/133 B attributes/12 host reads. This is positive
use; full ordinary equivalence was still open. All 143 CLI guards and the
implementation/canonical/originals are exact; the artifact helper found zero
paths. No old source/model/runtime/producer logging/scenario was repeated and no
new fixture, navigation or measurement was performed. Linux/USA/native Windows/
new 32:9 remain to verify; ten parameter bytes and 452 lower remain separate.
[CLI and evidence](../rom_data_obj.md#actual-cli-for-the-observed-indirect-lists).


**ROMDATA-05-OBSERVED-OBJ-LISTS-ORDINARY completed for the first EUR F09 frame
(2026-10-26).** Only five optional lines in the existing validator require both new
lists in guest and host. The first warning-free Windows build and one Wine filter
compare frame 8476→8477: full state/clock/central-wide images/OAM/dispatch and all
audio are equal. Five streams contain real samples [2196,2196,264,264,2120]; OBJ
counters of 147 reads/147 B/14 count/133 attributes/12 host exactly match the CLI.
The 6803 native/22849 fallback counts are coverage only; 75 input/executable guards,
originals/cache/dataset/canonical are exact. The five-line delta was checked against
the preimage, with other implementations unchanged; the artifact helper found zero
paths. Source/model/managed/runtime/CLI and declared ordinary acceptance are closed;
other scenes/indirect calls/composed GUI/ABI/new 32:9 remain open. No old 12/3530/
scenario/model/runtime check was repeated and no new fixture, navigation or
measurement was performed. Ten parameter bytes/452 lower and Linux/USA/native
Windows remain separate or to verify.
[Equivalence and limits](../rom_data_obj.md#ordinary-first-frame-equivalence-for-the-observed-lists).


**ROMDATA-02-OBSERVED-TASK-PARAMETERS-MODEL completed EUR (2026-10-26).**
The isolated observed_task_parameters API covers two independent fragments: phase
1/2 u8 thresholds at 371065..67 and four signed16 corrections, indices 4..7, at
394E14..1C. It authenticates identity/indices/strict JSON/10 B reconstruction/two
spans/code-literal-source hash, without inferring duration, a global cap or physical
extent. Phase 0/other bytes are outside the model; corrections 0..3 retain their
previous owner and 8..127 remain external. The first warning-free Windows build and
one Wine filter cover 27 guards and 18 original prefixes (10 old-counter/phase-wrap,
8 low-bit alias/signed-sub), with RAM/callee/OAM-VRAM-palette exact. Canonical model
4b52c5f1.., 79 input/executable guards and the implementation are exact. Only two
declarations were added in old hubs; pipeline/e72-d71/66 files/65 resources/76 OBJ
lists/USA 8-7 are unchanged. The new 10 B are disjoint from all owners and were not
yet published; parameter managed/runtime/CLI/ordinary remained open at this
checkpoint. The artifact helper found zero paths. No source/295 pins/old model-ABI-
runtime-CLI-scene checks were repeated and no new fixture, navigation or measurement
was performed. The 452 lower remain separate; Linux/USA/native Windows/new 32:9 are
to verify.
[Model and limits](../rom_data_gameplay.md#isolated-model-for-observed-task-parameter-fragments).


**ROMDATA-03-OBSERVED-TASK-PARAMETERS-RUNTIME completed EUR (2026-10-26).**
e73-d72 publishes gameplay.observed-task-parameters/observed-task-parameters.json:
67 files/66 resources, with all 65 previous files/records, e72 and 76 OBJ lists
retained. The two independent fields have no invented dependency and cover only
10 B/two spans 371065..67 and 394E14..1C. Pages 371/394 compose previous owners
and ROM without a new cap/ABI/clock/state. The first two Wine filters pass 13 attach
refusals/3 full-set guards, publication/repair/move/lifecycle and 24 original-
interpreter-native prefixes with equal full state/clock and all 10 raw B altered.
They include phases 0/3/255, corrections 0/16/255, edges/mirror/BIOS, six DMA-
CpuSet-FastSet transfers, API reads of 2 new + 2 old with old raw altered and 2 new
+ 2 modified raw. Native APIs served 26 B; the 316 instructions/24 dispatch are
coverage only. Model 4b52c5f1.., 74 guards and compiled code are exact. The first build
compiled core but the frontend omitted two test arguments: only two zeros were
corrected, then a warning-free retry succeeded; no Wine filter failed. The artifact
helper removes only interrupted/moved copies. Production CLI and ordinary parameters
were still open at this checkpoint; Linux/USA/native Windows/new 32:9 remain pending,
with 452 lower separate. No old model/source/ABI/scenario was repeated and no new
fixture, navigation or measurement was performed.
[Runtime and evidence](../rom_data_gameplay.md#managed-observed-task-parameters-and-typed-bus-consumption).


**ROMDATA-04-OBSERVED-TASK-PARAMETERS-ACTUAL-CLI completed EUR (2026-10-27).**
A warning-free Windows production build produced c7f47bd4..; canonical hash/mtime
were exactly restored and main content unchanged. The first prepare was interrupted
by the user: runner absent, manifest without current/result. Its copy/log were
archived and the same binary reused. Three accepted commands exit 0: create/reuse/
old-cache one frame, with 67 generation files/69 managed/66 resources/two spans of
10 B/no dependencies/all 67 exact, and reuse without writes. The first frame has
1 B threshold/1 phase but 0 corrections: the presence gate is explicitly negative
and retained. One new 12-frame runc, motivated by that negative and an existing
source lead, exits 0 with 20 reads/28 B/12 thresholds/16 corrections/2 phases/
4 rows. The 145 guards and implementation/originals/cache/canonical/user Atlas are
unchanged; the artifact helper found zero paths and the interrupted copy is retained.
CLI is closed; ordinary full state for 12 frames was still to verify. No prepare/
reuse/build/old source-model-runtime-scenario was repeated and no new fixture,
navigation or measurement was performed. Linux/USA/native Windows/new 32:9 and
452 lower remain separate or pending.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-observed-task-parameters).


**ROMDATA-05-OBSERVED-TASK-PARAMETERS-ORDINARY completed EUR (2026-10-27).**
Only five optional lines in the existing validator require both groups and all six
rows. The first warning-free Windows build and one Wine filter compare 12 F09 guest
frames 8476→8488: every full state/clock/central-wide image/OAM/dispatch/audio is
equal. Five streams contain real samples [26334,26334,3168,3168,26258]; 20 reads/
28 B/12 thresholds/16 corrections/2 phases/4 rows exactly match the CLI. The 88123
native/279744 fallback counts are coverage only. The 76 guards, originals/cache/
67-file dataset/canonical/code/user Atlas are exact. The five-line delta was checked
against its preimage; the artifact helper found zero paths. Source/model/managed/
runtime/CLI/declared ordinary acceptance are closed. The first correction-negative
frame and interrupted prepare remain closed; no old source-model-runtime-CLI-
collector-scenario was repeated and no new fixture, navigation or measurement was
performed. Other scenes, indirect calls/ABI/composed GUI/new 32:9 remain open; 452
lower and Linux/USA/native Windows remain separate or pending.
[Equivalence and limits](../rom_data_gameplay.md#ordinary-twelve-frame-equivalence-for-observed-task-parameters).


**ROMDATA-01-BUS-HOST-LOWER-RESIDUE completed EUR (2026-10-27).**
The classified 452 bytes/113 words/62 ranges have 114 contextual reads: 404 B from
102 Thumb PC-relative LDRs on 101 literals, and 48 B from 12 observed switch entries
7–18. Load 080A1146/MOV PC 080A1148 retains Thumb; the original prefix normalizes u16
zero/>24 to 1 before indexing, without a new global limit. The new optional SiteCapture
retains the previous schema/keys; PC/mode scope requires opcode/address/width
verification, not automatic attribution for nested HLE/DMA. One warning-free
incremental Windows build followed a helper error only in the test; one Wine filter
with new probes and one current F09 arm over 12 frames observed all 113 words.
The 85 guards/cfg preimage/code/Atlas are exact; the artifact helper found zero paths.
No old acceptance was repeated and no fixture or measurement was added. The 452 B
support code; e73-d72/67 files/66 resources are unchanged. New sources 394FBC/
371064 phase 0/371034 require their own contracts; no format/extent is inferred
from literals. Linux/USA/native Windows/new 32:9 remain to verify; the global TODO
is open.
[Contract and limits](../rom_data_runtime.md#lower-bus-and-host-residue-source-contracts).


**ROMDATA-01-ADDITIONAL-CONSTRUCTOR-OBJ source/model completed EUR (2026-10-27).**
394FBC is a 32 B table of four conditional 32-bit threshold/list pointer pairs.
Original readers 9DB56/9DBCA and the already authenticated constructor prove four
lists 394F82/90/9E/AC, 56 B/eight objects. Total: 88 B, with no owner overlap.
The u8 is incremented/stored/reloaded before comparison; 255→0, and a new phase
>3 disables. The old phase is not bounded before lookup: model 0–3, no new global
cap. The isolated model verifies strict JSON/source projection/list dependency.
The first warning-free Windows build and single Wine filter pass 13 guards, 15
counter prefixes, four pointer-constructor prefixes and four new draws with the
existing copied oracle. The 94 guards/code are exact; only hub declarations/helper
visibility changed, and the artifact helper found zero paths. Retained F09 does not
read these data: no ordinary use is claimed. Production e73-d72/67 files/66 resources
is unchanged; extraction/runtime owner/positive use remained to do. Linux/USA/native
Windows/new 32:9 remain pending, with no old acceptance repeated and no fixture,
navigation or measurement.
[Contract and model](../rom_data_obj.md#conditional-constructor-obj-animation-source-and-model).


**ROMDATA-03-CONSTRUCTOR-ANIMATION-RUNTIME completed EUR (2026-10-27).**
e74-d73/68 files/67 resources publishes the 32 B table with an OBJ dependency; the
catalog has 80 lists/193 objects/1318 B, with all 76 previous lists/126 direct refs/
65 other files-records exact. The exact table owner composes previous and raw data;
lists are served by the existing OBJ owner. The first Wine run passes 10 guards/
2 halfsets/lifecycle/publication and 13 original-interpreter-native state-clock
prefixes with 88 raw B altered: 36 B typed native/20 thresholds/16 pointers/4 phases,
4 external queries without charges, and 708 bus and composition cases. It then stops
on a test setup error: load_state before ROM restoration. The failure is retained;
no full PASS is claimed for that run. Only test order was corrected; a remaining
filter on the already validated dataset passes 6 DMA/SWI and 4 host publications/
two actual objects each, without repeating passed portions. Production was identical
between filters. Final builds were warning-free; 169 guards/code/Atlas are exact.
The artifact helper found 0 paths; failed diagnostics and offline 68 were retained,
with only a superseded never-run executable retired. Real CLI/positive ordinary use
remained to do; retained inactive F09 was insufficient. Linux/USA/native Windows/new
32:9 remain pending; no new fixture, measurement or replay.
[Consumption and evidence](../rom_data_obj.md#managed-constructor-obj-animation-and-typed-consumption).


**ROMDATA-04-CONSTRUCTOR-ANIMATION-CLI completed EUR (2026-10-27).**
The first warning-free Windows production build produced 1e98de03..; canonical
hash/mtime were restored and main content unchanged. Three Wine CLI calls exit 0:
prepare/reuse without writes/runc 0 frames. There are 68 generation files/70 managed/
67 resources; all 68 hashes match the accepted runtime and all 67 runtime IDs/hashes
are exact, with old cache loaded. Reuse and startup hash/mtime are unchanged, and
145 guards/439 sources are exact; the helper found 0 paths. Zero frames/dispatch/
fallback/new counters provide real registration/attach evidence, but no positive
ordinary use is inferred. No F09/source-model-runtime-scenario was repeated and no
new fixture, navigation or measurement was performed. Positive ordinary use and
Linux/USA/native Windows/new 32:9 remained to verify.
[CLI and limits](../rom_data_obj.md#actual-cli-for-constructor-obj-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE partial EUR (2026-10-27).**
A passive Wine-only observer and optional positive validator guard were implemented
(seven lines; the previous loop is exact without the guard). The first warning-free
Windows build was followed by four normal observations that exit 0 but are NEGATIVE:
two movements of 160/200 frames from a copied EUR 12006 checkpoint, delayed selection
160 and confirmation 200 from a new selection state. Table 0/phases 0/lists were
unused; no full pair or positive fixture is claimed. The 84 guards/code/originals are
exact; previous normal provenance was retained without a 3530-frame replay. There
was no manual RAM/task/phase/ROM mutation. Flag setter 9D054/call 6 was verified as
a lead, while the normal action/semantics still need proof; no further blind route
or forced flag. The unit remains open and fixture/diagnostics retained; independent
371064 phase 0 source is separate. Linux/USA/native Windows/new 32:9 remain pending;
no measurement.
[Evidence still open](../rom_data_obj.md#ordinary-constructor-animation-discovery-remains-partial).


**ROMDATA-01..05-CONDITIONAL-PHASE-ZERO completed EUR (2026-10-27).**
371064 phase 0 is now typed: conditional thresholds 0/1/2 = 16/8/8, a 3 B span
plus 8 corrections, 11 B in the same file/68 files/67 resources/e75-d74. The old
2 rows + 4 corrections and all 66 other records-files are exact. There is no global
cap before lookup or ABI change. The disjoint coverage partition is 3 phases/
4 corrections. The first warning-free Windows build and single Wine run pass 4 new
original prefixes/7 attach/6 native state-clock/3 DMA and raw edges. The first
production build restores the canonical executable; one auto-prepare 28-frame CLI
passes all 68 files/3 phases/4 corrections/36 reads/44 B/28 thresholds/16 corrections.
Only the offline log matcher was corrected after CLI exit 0; the command was not
repeated. With the same frontend executable, one new full pair over 28 frames
8476→8504 passes every state-clock/image/OAM/dispatch/audio comparison with samples
[61446,61446,7392,7392,61370] and exact CLI counts. The 215 guards/code/5-line
unguarded loop are exact; the helper found 0 paths. No old 12-frame/source/model/
runtime/constructor checks were repeated and no new fixture or measurement was made.
Ordinary constructor use remains partial; 371034 and Linux/USA/native Windows/new
32:9 platforms remain to verify.
[Contract and evidence](../rom_data_gameplay.md#conditional-phase-zero-threshold-extension).


**ROMDATA-01-TASK-CYCLE-DELTAS / ROMDATA-02-TASK-CYCLE-MODEL completed EUR
(2026-10-27).** The conditional projection covers 371034–371064/48 signed-i8 deltas,
disjoint from all 67 e75 resources. Readers 18E10/18E2A use the OLD u8 before
normalization; the increment is stored as u8 before helper 1464EC/remainder 48,
with 255→0. Signed addition/subtraction applies to halfword task+8 with a low 16
store; axis/unit/lifetime are not inferred. No cap is introduced: queries 48..255
return None, while 48/49/50 belong to the old owner. The first warning-free Windows
build and single Wine filter pass 23 guards/58 original prefixes/8 helper edges/all
48 rows and RAM-callee guards; 77 frozen inputs are exact. The model is lossless
with strict JSON; e75-d74/68 files/67 resources are unchanged. Management/runtime
consumption/CLI/ordinary 48 were completed in the units below; the source test alone
did not claim consumption or a scene. Previous constructor ordinary use remains
partial; Linux/USA/native Windows/new 32:9 remain pending, with no measurement or
new ordinary fixture.
[Contract and evidence](../rom_data_gameplay.md#conditional-european-task-cycle-delta-source-and-model).


**ROMDATA-02/04-TASK-CYCLE-RUNTIME completed EUR/Wine (2026-10-27).**
The new resource gameplay.task-cycle-deltas/file task-cycle-deltas.json is part of
e76-d75: 69 files/68 resources, 48 B signed-i8/one span, with all 67 previous
resources/files exact. The typed owner composes new bytes/prior/raw without a cap or
ABI change; USA remains 8-7. The first warning-free Windows build and first Wine
filter pass 9 transactional attach refusals/2 full-set/interrupted new file, 8
original-interpreter-native prefixes with equal full state/clock and 48 raw B altered,
and 5 explicitly compiled reads. They cover all 48 rows, a read32 boundary with
2 new + 2 old bytes, edges/mirror, 3 DMA/SWI transfers across owners, unavailable
files and Load/Undo with the owner retained. The 154 guards/code are exact; the helper
retires only the interrupted copy. No source filter or old acceptances were repeated.
Production CLI and positive ordinary use of the 48 deltas were completed below;
constructor ordinary use remains partial. Linux/USA/native Windows/new 32:9 remain
pending; no measurement.
[Consumption and evidence](../rom_data_gameplay.md#managed-task-cycle-deltas-and-typed-consumption).


**ROMDATA-05-TASK-CYCLE-CLI completed EUR/Wine (2026-10-27).**
The first warning-free Windows production build restored the canonical executable
with exact hash and mtime; three Wine commands prepare/reuse/runc 0 exit 0. All 69
files equal the accepted Rust dataset, all 68 runtime IDs/hashes equal the manifest,
48 delta rows are registered and the existing native cache is loaded. The entire
71-file pathset/hash/mtime remains unchanged on reuse and startup; 226 guards/code
are exact and the helper found 0 paths. Zero frames/dispatch/fallback/delta reads
provide attachment evidence only; ordinary positivity and the full pair for 48 were
completed below. No blind route or manual flag. Linux/USA/native Windows/new 32:9
remain pending; no measurement.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-task-cycle-deltas).


**ROMDATA-05-TASK-CYCLE-ORDINARY completed EUR/Wine (2026-10-27).**
The neutral Bank 48 scene 3660→3708 was selected from source and a normal snapshot:
initializer 18D6C/default 03, generic 015E48/03-07 and candidate Bank root+D40 flags
03 phase 21, with address consistent with caller 5C1F8. Setter 18E5C retains old
04/08 with new 01: argument 03 alone does not prove mode. The 11/3/89 raw BL are
candidates; no call graph/liveness is inferred from RAM alone. One existing CLI
without rebuilding uses 48 reads/48 B/all 48 phases; dataset 71 paths/hash/mtime
remain unchanged. The first warning-free frontend build adds 5 positive-guard lines;
the old loop is byte-exact without the guard. One Wine full pair passes every full
state/clock/image/OAM/dispatch/five audio streams, with samples
[105336,105336,12672,12672,105260] and exact CLI counts 48/48/48. The 97 guards/
code/preimage are exact; the helper found 0 paths. No navigation, RAM-task-ROM or
cartridge tampering, or repeat of old Bank 12/F09 acceptance. Declared cycle
children 01/02/04/CLI 05/ordinary 05 are closed, while total migration remains open
and distinct constructor ordinary use remains partial. Linux/USA/native Windows/new
32:9 remain pending; no measurements.
[Scene and evidence](../rom_data_gameplay.md#ordinary-european-task-cycle-scene).


**ROMDATA-01-BANK-BUS-HOST-RESIDUE: EUR/Wine census completed, 90 B sources open
(2026-10-27).** Optional SiteCapture::start_all mode preserves empty/single filters
and old keys. The first warning-free Windows feature build and single Wine run cover
new probes and one decoded Bank 12 arm, 3660–3672, with existing normal input; the
old full pair was not repeated. It reports 7430 reads/7532 sites, 497 with interpreter
context/7035 None; 7083 B image, 1434 B outside 68 owners. All 3683 B of the old
NativeAPI are included; 992 B were already classified, with 80 literals/320 B and
8 dispatch cells/32 B verified now: 1344 B code support, with no global 127 cap
inferred from the branch with helper. The remaining 90 B are data: 18 program/
12 directory/60 OBJ lists, root 4165A0+4114FC via 5E6F4/8, readers 2C5E0/2C60C
with a tick-phase-selector u16, distinct from already typed u8 banks. The contract
and model of four authenticated prefixes were completed below; management/
consumption remained open at this checkpoint. The census alone yields no new
resource/revision. Twelve typed deltas 21–32 read by interpreter LDRB 18E2A explain
the old NativeAPI limit. The 95 guards/code/default preimage are exact; the helper
found 0 paths. e76-d75/USA 8-7 unchanged and user Atlas preserved; Linux/USA/native
Windows/new 32:9 pending, with no measurements or fixture.
[Inventory and new sources](../rom_data_runtime.md#bank-bus-and-selected-host-rom-inventory).


**ROMDATA-01-U16-TASK-OBJ-BANK / ROMDATA-02-U16-TASK-OBJ-MODEL completed within
selected EUR prefix scope (2026-10-27).** Four 2C5E0 producers authenticate roots
41634C/411478, 41647C/4114AC, 4165A0/4114FC, 4166C4/41154C: 50 lists/140 OBJ/
7 sequences/49 frames/65 spans/1378 B disjoint from 68 owners. Relative-index/
payload format is verified, with padding excluded; seven unreferenced lists and
three empty sequences remain data, not reachability evidence or invented holds.
The u16 counter-phase-selector, low u8 repeat, old-list return before increment,
255→256/65535→0 and clamp/repeat/done latch are preserved; no cap is inferred
before lookup or at the global archive boundary. The lossless strict JSON model
reuses pure fields, not u8 timing. The first warning-free core build runs Wine:
23 guards/1378 B projection pass, then a test expectation fails on constructor
return r0. A test-only correction and second warning-free core build/remaining-only
filter pass 23 constructor/93 update/6 producer checks and RAM-callee/alias/edges.
The first filter is not declared PASS and the 23 guards were not repeated. The
88 guards/199 core sources are exact; the helper found 0 paths. Core dev serde_json
preserves parallel frontend changes without compiling them. e76-d75/USA 8-7 are
unchanged: management/consumption/CLI/ordinary for these banks were still open at
this checkpoint; Linux/USA/native Windows/new 32:9 pending, with no measurements
or fixture.
[Contract and model](../rom_data_obj.md#european-obj-banks-with-u16-task-counters).


**ROMDATA-02/04-U16-TASK-OBJ-RUNTIME completed in EUR/Wine phases (2026-10-27).**
e77-d76 adds graphics.word-task-obj-animations/file word-task-obj-animations:
70 files/69 resources with all 68 previous records-files identical, 65 spans/1378 B/
4 banks/50 lists/140 OBJ/7 sequences/49 frames. Strict binding/transactional attach
and the shared pure projection retain the old owner body and gate, new 411/416 gate
and host lookup. Indexing/counters/guest old-list return and ABI/states/cache are
unchanged. The first warning-free Windows build runs Wine: publication/interrupted
file/9 attach guards/2 full-set pass, as do 27 original prefixes (7 constructor/
20 update), 202 B compiled/full state-clock, 1378 B/6 classes/217 mixed reads/66 B
raw gap/old owner/Load-slot Undo without file/DMA3. The process then has no terminal
result: no full PASS is claimed, receipts are retained and the passed phase was not
repeated. The remaining build compiled clean but a source guard rejected it because
of 2 parallel DBG changes; a stable snapshot was used. A host-test draft was
isolated per frame; a test-only private-field compilation error and OAM/register
origin were corrected, without product changes. The final warning-free build and
single host-only Wine run pass 50 lists/140 OBJ/95 host reads with 65 ROM spans
altered, accepted OAM per list and equal raw/typed descriptors. The 165 guards/
496 snapshot sources/current ROM inputs are exact. The helper retires only the
expected interrupted copy and retains dataset/logs/preimages/executables. Real CLI
and ordinary use in the Bank window were subsequently verified below, without
repeating old model/census/scene acceptance. USA e8-d7/Linux/native Windows/new
32:9 remain pending; no measurements or new fixture/navigation.
[Runtime contract and evidence](../rom_data_obj.md#managed-runtime-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-CLI completed EUR/Wine (2026-10-27).**
One warning-free Windows production build used the accepted runtime snapshot copy.
The CLI candidate is b3bfe6f425923ce461793771fe6c84b86eb5da7dda69b3b9c7e0f5208eab51ea;
canonical executable hash/mtime were restored. One Wine script with 3 real commands,
prepare created/reused/runc 0, passes: 70 generation files identical to runtime/
69 registered resources with exact hashes/72 managed paths. Reuse and startup make
no hash/mtime writes; the original cache loads, with 0 guest frames/dispatch/
fallback and all 9 new owner fields at 0. The 154 guards/496 copied sources/current
ROM inputs are exact; the helper found 0 paths. Dataset:
27463ce4f7ffccc3d0b9c8ab5ee78160a3e1c844ed80f68768f9adea87cb7757. CLI is closed;
separate positive ordinary use was verified below. No old acceptance or fixture/
navigation was repeated. Linux/USA/native Windows/new 32:9 pending; no measurements/
Android/package/commit/push. Parallel DBG changes preserved.
[Real CLI and evidence](../rom_data_obj.md#actual-cli-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-ORDINARY completed EUR/Wine (2026-10-27).**
The Bank window 3660→3672 was chosen from the already accepted census/root
4165A0+4114FC with byte-identical copies of the normal snapshot/inputs; no navigation,
new checkpoint or RAM-task-ROM/cartridge tampering. One existing e77 CLI passes
positive use: 292 reads/346 B = [44 index/20 count/240 attr/2 seq header/20 frame-
list/20 duration]/20 host, with 72 managed paths/hash/mtime unchanged. An optional
5-line guard covers the new owner in the full-pair test; the old validator/loop is
byte-exact with it removed. The first warning-free Windows snapshot frontend build
and single Wine full pair first PASS: all 12 full states/clock/central-wide images/
OAM/dispatch and 5 audio streams identical, with 9 exact CLI counters. Dispatch
[19962,85770], samples [26334,26334,3168,3168,26258]. The 94 guards/496 snapshot
sources/current ROM inputs are exact; the helper found 0 paths. No previous model/
runtime/scene/census acceptance was repeated and there was no new production build.
Declared source/model/management-consumption/CLI/ordinary children for this family
are closed; there is no evidence for all 4 banks/50 lists/7 unreferenced lists/empty
sequence update/indirect index/full archive/full game. Distinct constructor ordinary
use is partial; the whole TODO and Linux/USA/native Windows/new 32:9 remain open,
with no measurements/Android/package/commit/push and parallel DBG preserved.
[Scene and evidence](../rom_data_obj.md#ordinary-european-u16-task-obj-bank-scene).


**ROMDATA-01/05-CONSTRUCTOR-FLAG-REACHABILITY closed for EUR/Wine flag
(2026-10-28); ordinary animation still partial.** Twenty-one source windows link
95EA0/wrapper 9A594/global 0200F4A8/state DC/root 960F0: Move 27/A after B cancel/
validation 9D6C8/different destination → count 1A4/28-29, with signed count >1 →
enable 99F18. Other 3A→99300 and A5544 case 6→A57BE/case 1→A570A are distinct
paths, not global liveness. Two authenticated normal snapshots were read only to
choose A-only, without the old Right: one reused Wine observer over 160 frames closes
the fifth negative test, with table 0/phases 0/lists unused; no full pair, positive
fixture or new build. Endpoint state 25/count 3 does not imply an invalid destination.
For the new missing flag/slot criterion during transition, one existing readonly
64-frame CLI captures 448 RAM-image files: normal Move and flag 20 active during
12186–12230/45 postframes, while both slots 394/3A4 remain inactive with counter-
phase 0/table 0. Ordinary flag reachability is proved, but slot creation/callback
still need authentication; no PC/intraframe attribution. The 587 guards/496 frozen
sources/21 window hashes are exact; originals/e77 dataset 72 paths are unchanged and
the helper found 0 paths. No old acceptance was repeated and RAM/ROM/cart were not
tampered with; a static disassembler is not a Linux game test. A new full pair is
allowed only after positive use of 4 phases/4 lists guest-host; no more blind Move
attempts. Linux/USA/native Windows/new 32:9/full TODO remain open, with no
measurements/Android/package/commit/push.
[Flag and slots](../rom_data_obj.md#ordinary-constructor-flag-activation-and-inactive-slots).


**ROMDATA-01/05-CONSTRUCTOR-SLOT-ACTIVATION closed for source gate and map exclusion
(2026-10-28); ordinary still partial.** Eight new windows authenticate
9DB0C→owner+10/pair [owner,9B11D]→9E300/145C10→9B11C(actor,owner), using the same
slots 394/3A4 as the counter and 9DBA4. 9DAD4 is a distinct earlier helper. Beyond
movement and flag 20, normal branch 9B460 requires unit FD2, signed XY 0..15, a
valid cell/flags 02/S8 height > actor S16 height >>4, anim 36!=40, entry-exit bit
8000, skip mask 10000 and a free slot. The live cell root is 02007F10, width+8/
base+4, stride 2 height-flags, without a new ROM asset. Readonly inspection of the
existing 64 RAM dumps found FD2/actor flags 0/anim 0, three traversed cells with
height 2/flags 0 and all 256 declared cells without 02; the 512 B hash is stable.
This unchanged map offers no destinations for the gate; no further blind Move or
repeat of the old five negatives/flag capture. The 87 guards/8 window hashes are
exact; the helper found 0 paths. No new gameplay/build/Wine-Linux test/capture/
schema/runtime/ABI/cache/save. The next criterion is a readonly inventory of
existing normal EUR snapshots, suitable map and actor and provenance, without
synthetic states or recreation of retired copies. A full pair is allowed only after
positive use of 4 phases/4 lists guest-host; ordinary/Linux/USA/native Windows/new
32:9/full TODO remain open, with no measurements/Android/package/commit/push.
[Cell and slot gate](../rom_data_obj.md#constructor-slot-creation-depends-on-live-cell-flags-and-height).


**ROMDATA-05-CONSTRUCTOR-ELIGIBLE-MAP-INVENTORY closed negative (2026-10-28).**
The new readonly criterion covers 81 normal EUR snapshots with already verified
provenance: all hash/payload/ROM/16x16 domains are valid, but only 2 map projections
of 512 B exist (1 F09/80 river), with zero 02 cells/candidates. The old 606
alternate sequence scan was not repeated. The 88 snapshot-report-parser-contract-
output guards are exact; no build/gameplay/Wine-Linux test/capture/checkpoint/
tampering. This terminal negative is bounded to 2 retained maps, not all maps or
future script; do not repeat the same 81 states or Move here. The ordinary parent
remains partial; the next source lead is cell-flag producer 02007F10/terrain ROM
before choosing an eligible map. A full pair is allowed only after positive use of
4 phases/4 lists guest-host. Linux/USA/native Windows/new 32:9/full TODO remain
open, with no measurements or deployment.
[Retained maps](../rom_data_obj.md#retained-normal-maps-have-no-eligible-constructor-animation-cells).


**ROMDATA-01-CELL-FLAG-WRITERS closed for EUR census/candidate seeds (2026-10-28).**
The already closed TERRAIN-MAP-SOURCE from the 24th and its 352 BG packages were
reused, without a new archive. A new direct census found no Thumb BL/aligned pointer
to 1DE98 (not proof of dead code or absent indirect calls); the sole BL from 1DD68
to 1D798 reads flags 02. A new semantic query of the authenticated e77 BG dataset
covers 113 auxiliary programs/162 links/zero fill-parent-child/ordered u16 writes:
25 programs and 44 IDs initialize 02; flags are 0/1/2/3/4/8/9. No new decoder,
resource or revision was created; old codec/loader acceptance was not repeated. The 24 guards/4 source windows
are exact and the helper found 0 paths; no build/gameplay/Wine-Linux test/capture/
checkpoint. A private quote typo was corrected before execution; early cleanup
rejected the unit without a result, then passed after the summary, with product
unchanged. Asset candidates do not prove a normal battle or place name; the next
lead is the actual map selection script/caller → eligible ID/normal reachable battle.
The 81 snapshots/5 Move bound/flag capture remain closed. Ordinary 4 phases/4 lists
guest-host/full pair and Linux/USA/native Windows/new 32:9/full TODO remain open,
with no measurements or deployment.
[Seeds and sources](../rom_data_maps.md#constructor-animation-cell-flag-seeds).


**ROMDATA-01-CONSTRUCTOR-ELIGIBLE-MAP-SELECTION: source link closed,
reachable battle still partial (2026-10-28).** State 10 of handler 253C8 reaches
2549C, reads variable 7 through CCF00 from 02002037, and narrows it to a low u8.
It calls 1B270, creates the 1B689 loader and advances to state 11; 2540C is the
dispatch table. Opcode 12 advances by 2 and calls 126831. If instruction byte 1
is nonzero, CCF18 writes it to variable 7; otherwise byte 2 of the record at
`70C8CC + 40 * U16(02002170)` is written to variable 7. Both paths then set
0200F390 to 10. Of 51 CCF18 candidates, two literal-7 arms were authenticated;
this is not a claim that all writers were found.

A typed script query found 204 instructions: 56 use the default and 148 use a
literal. The 19 eligible literals use 10 map arguments
[31,33,34,73,75,89,91,109,115,129]. Exact references remain private; these do
not establish place names or ordinary reachability, and an event key is not
necessarily the active index. Two retained normal examples have active indices
35/197, map arguments 99/85 and descriptors 98/84. They are consistent with the
default path and ineligible maps; the 81-snapshot scan was not repeated. All 37
guards and 9 source windows matched. There was no build, gameplay, Wine/Linux
test, capture, checkpoint, runtime, schema, ABI, cache or save change.

The next lead was the active-index producer and byte 2 of the 40B record at
70C8CC, to find a normal quest or event without forced IDs or guessed navigation.
At this checkpoint, the positive 4 phases and 4 lists through guest and host,
the full pair, the whole TODO, Linux, USA, native Windows and 32:9 remained
open; no measurements or deployment occurred.
[Script selection and limits](../rom_data_maps.md#script-map-selection-and-remaining-battle-reachability).


**ROMDATA-01-ACTIVE-ENCOUNTER-RECORDS: producer and conditional join closed EUR
(2026-10-28), complete catalog and ordinary battle still partial.** Setter 9CF0
writes the low 16 bits of r0/r1 to 02002170/172. Two direct BLs at 9D2C/1273A8
were found, with no aligned Thumb pointer; this does not establish all writers.
Depending on row+4, 9D04 chooses the primary or alternate value from row+2 or
`71D1EC + 12 * low8(arg1)`. At 63944, live selector 219E selects a 70-byte record
at 713B3C; byte 45 selects a 12B row at 71C760. Callback 127388 searches event
keys in rows 255..1, taking the first descending match; duplicate keys remain
distinct.

A query over the caller's u8 domain found 208 direct-primary rows, 54 with seed
02 and 44 also referencing a default-map script. It does not prove mission
availability, names or the 40B catalog extent: the maximum reference 442 and a
separate constant 300 are not limits. Read-only inspection of the retained
normal pub state found selector 0, row 0 and ineligible map argument 11. All 42
guards and 23 windows matched. A private Python import was corrected after the
first attempt stopped before output; originals were unchanged. There was no
build, gameplay, Wine/Linux test, capture, checkpoint or new resource.

The next criterion was an eligible mission or event actually available from a
normal checkpoint and the original UI, without forced ID/RAM or further blind
Move attempts. At this checkpoint, the 4 phases and 4 lists through guest and
host and the ordinary full pair remained open. Old acceptances were not repeated;
Linux, USA, native Windows, new 32:9 and the full TODO remained pending, with
no measurements or deployment.
[Producer and evidence](../rom_data_maps.md#active-index-producer-and-event-record-links).


**ROMDATA-05-ELIGIBLE-QUEST-LOCATOR closed EUR/Wine (2026-10-28): eligible mission
present in normal menu.** The existing observer 309ba31c and e77 dataset were reused
without a build. A known 620-frame path retained the previously missing menu endpoint
at 14969; another 300 frames with 4 Down presses and A opened the White Flowers detail
at 15269. The old 620-frame scene was not repeated as acceptance: the new criterion
was the checkpoint and its selectors. Originals and copies remained exact, with no
RAM, task, ROM or cartridge edit.

The live list context F44C is 02024D5C, with count at +1B and pointers at
+20+4*ordinal; 62126..5E derives the low 10-bit mission ID. A new query of 14 entries
found 2 direct-primary/default-script/cell 02 candidates, 58/72. The fifth row and
visible detail confirm 58; F448 is 0202389C, whose +1450 pointer selects 02002268
with ID 58, as authenticated by 60208..28. Event 58, primary 315, map argument 69
and descriptor 68 are eligible. The description names Giza Plains, but acceptance
and battle had not yet been reached at this checkpoint. All 130 guards, 496 frozen
sources and 23 windows matched; the helper found 0 paths. Candidate layout 031C/03D0
at 03001028 was rejected because it was not linked to the live menu. Both Wine
workers passed; constructor counters of 0 in the pub were expected and did not prove
positive use or a full pair.

The next step was normal confirmation and verification of acceptance and battle
entry, followed by actor/cell inspection before guided Move. At this checkpoint,
4 phases and 4 lists through guest and host, the full pair, complete 40B catalog,
Linux, USA, native Windows, new 32:9 and the full TODO remained pending. Old
acceptances remained closed; no measurements or deployment occurred.
[Normal mission and evidence](../rom_data_maps.md#normal-pub-menu-identifies-an-eligible-quest).


**ROMDATA-05-WHITE-FLOWERS-BATTLE-NAVIGATION partial EUR/Wine (2026-10-28);
normal acceptance closed.** From an exact copy of the ID 58 detail, normal input
completed loadout OK, cost/objective and conditions Yes. Payment of 300 gil, funds
changing from 7709 to 7409 and the later Fee:Paid label prove the mission was active.
An entry initially read as QuitPub was QuitMission: only the A:Cancel detail opened,
with no cancellation confirmation. B/B/Down/A on Leave reached the map.

The unit used 27 normal captures and 4840 frames with the existing 309ba31c observer
and e77 dataset, with no build or manual RAM, task, ROM or cartridge write. All Wine
workers passed and table use was 0; the captures were not 27 equivalence acceptances
and no full pair was run. Visible world labels included Cyril, LutiaPass, Nubswood
and Sprohm; the central patch was excluded, and camera scroll and overshoot were
documented. This did not prove Giza Plains reachable, absent or unplaced. All 337
guards, 496 sources and 17 exploratory window hashes matched. The 92524 initializer,
stale task data and 031C 03001028 did not prove world dispatch. Catalog 165E8 index
725 for Giza and base 718/list 3EE60 were leads, not coordinates. The partial unit
retained all diagnostic checkpoints; no fixtures were retired.

The next lead was the source selector and placed locations from retained states,
without further blind cursor attempts or repeating the 27 bound, 5 Move, 81 map or
old acceptances. At this checkpoint, battle, actor state, 4 phases and 4 lists through
guest and host, the full pair, full TODO, Linux, USA, native Windows and new 32:9
remained open; no measurements or deployment occurred.
[Acceptance and limits](../rom_data_maps.md#normal-white-flowers-acceptance-and-open-battle-entry).


**ROMDATA-01-WORLD-LOCATION-SELECTOR closed EUR/Wine (2026-10-28).** Inspection
corrected two assumptions: 254B2 calls 6B40, and the scheduler count is task byte+2,
not word+4; literal halves 2810/0300 give 03002810, not 03001028. An optional
passive test used one warning-free Windows build from a 497-source snapshot and one
neutral Wine frame, 18829→18830. It observed 2232 reads and 2276 sites, 323 scoped;
actual RAM matched the parser. Parent[03002810]=0200F3A4 and
nodes[parent+67C]=08395CA8.

The 375F8 loop visits 30 nodes at stride 20; bytes 0/1 multiplied by 8 and increased
by 8 give the center, with a conditional hitbox and 1-pixel snap. At 37D6A..70 the
cursor is view+6/+8; 37A44 draws display Y as world Y minus scroll C. At 385CC..38668,
save base+B0+12*i uses enabled byte 1 bit 0 and byte 3 as a one-based selector of node
(selector−1). Giza index 7, catalog 725, node 20 and source 395E24 place it at
112,296. Read-only southern endpoints were step 21 with cursor 128,287 and step 22
with cursor 128,330; the old 27 bound was not repeated. All 152 guards, 497 sources
and 32 windows matched; the helper found 0 paths.

For the new ROM family of 30 nodes of 20B each, only 2 coordinate B were classified;
the other 18B, schema, model, extraction and runtime remained open. The next step was
a short Left from 21 and verification of the Giza label before A, without forced
coordinates, RAM or map ID. At this checkpoint, battle, 4 phases and 4 lists through
guest and host, the full pair, full TODO, Linux, USA, native Windows and new 32:9
remained open. No measurements, deployment or old acceptance repeats occurred.
[Selector and evidence](../rom_data_maps.md#world-selector-and-placed-region-coordinates).


**ROMDATA-05-GIZA-BATTLE-ENTRY closed EUR/Wine (2026-10-28).** A guided short Left
reached the Giza Plains label; A traveled there, the White Flowers menu opened, and
A entered deployment. The live primary index was 315, map argument 69 and descriptor
68. The 512B grid e8427b5e.. matched seed packet 56 and its 20 cells 02. Marsh was deployed with
position and direction confirmed; START To Battle, Yes and dialogue led to Move.
Actor FD2 was at (6,5); Left/Down/Down moved the actual cursor to (5,7). Normal A
activated 4 phases and 4 lists through guest and host: 75 frames and 22 reads of 88B,
comprising 44 threshold and 44 pointer bytes. Checkpoint 22713 and suffix 11 were
retained; a full pair had not yet been run.

The unit used 16 Wine captures and 3875 frames with the existing 309ba31c observer
and e77 dataset, without a build. All 221 guards and 496 sources matched; the helper
found 0 paths. Keys were applied only to copies, with no manual guest RAM, task, ROM
or cartridge-save modification. Initial START presses before facing confirmation
were inert and B returned to placement; the issue was corrected and closed, without
repeating it. The next step was one positive CLI 11 and a full state-clock-images-
OAM-dispatch-audio comparison from the suffix, without repeating discovery or
navigation. At this checkpoint, 40B catalogs, 18B world-node data, the full TODO,
Linux, USA, native Windows and new 32:9 remained pending; no measurements,
deployment or old acceptance repeats occurred.
[Entry and positive evidence](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE completed EUR/Wine
(2026-10-28).** An exact copy of the normal Giza checkpoint 22713 and its 11-frame
suffix reached 22724. The existing production CLI b3bfe6 reused the e77 dataset and
cache, passing with 22 reads of 88B: 44 threshold bytes and 44 pointer bytes across
all 4 phases. One Wine full pair with validator 309ba31c passed each of 11 serialized
states and guest clocks, classic/central/extended images, OAM, dispatch and five audio
streams, with all 4 lists used by guest and host readers. Counters matched the CLI;
dispatch was 57833/223470 and audio was 24138/24138/2904/2904/23966.

All 91 guards, 496 sources and 72 managed paths matched; the helper found 0 paths.
There was no build, preparation, navigation, discovery, repeat of old acceptances,
guest edit, cartridge write or measurement. The ordinary constructor case is closed
for this scene, not all actors, maps or the full game. Linux, USA, native Windows,
new 32:9 and the full TODO remained pending, as did the 40B catalogs and the 18B
remaining in the new world nodes. u16 and DBG work was preserved; no deployment
occurred.
[Full ordinary evidence](../rom_data_obj.md#ordinary-constructor-animation-positive-giza-scene).


**ROMDATA-01-WORLD-NODE-RECORDS EUR source projection closed (2026-10-28).** The
projection at 395CA8..395F00 has 30 records of 20B, or 600B. The count comes from the
already proven 375F8 loop and does not establish a runtime cap of 30: getters narrow
to low u8 and work arrays have 32 entries. Producers 31488/32BE6 write the root at
[parent 03002810+67C]. Bytes 0/1 are coordinates; slots 2/6/10/14 hold path ID,
neighbor and opaque 2B, followed by an opaque 2B tail. The projection retains 300
semantic B and 300 opaque B. Its 120 slots include 90 active links and 30 path 0
slots; 45 paths each appear in 2 directions.

Node 0 is distinct from path 0. Reader 4DAD0 matches neighbor before path, yielding
0 for target 0 at 22 nodes; 46EB0 accepts the same node or requires path!=0 before
matching a neighbor. The 4DB80 graph copies 4 slots, skips path 0 and has an 8-27
gate; external reference 397018+(path−1)*8 leaves the road catalog and payload open.
All 25 windows (21 new and 4 reused), 44 pins and 42 guards passed; the 600B are
disjoint from 49746 spans belonging to 69 owners. A new criterion on an existing
trace found 35 reads at 42 sites, coordinates only, with no new edge coverage or
repeated audit. No build, game, test, fixture, guest edit or measurement occurred.
The next step was a typed lossless model/source plan and one targeted Wine oracle;
extraction, runtime, CLI, ordinary use, Linux, USA, native Windows, new 32:9 and the
full TODO remained pending.
[Schema and readers](../rom_data_maps.md#world-node-record-source-projection).


**ROMDATA-02-WORLD-NODE-MODEL completed EUR/Wine (2026-10-28).** The world_nodes
catalog authenticates the exact image, 6 windows and 600B hash. The model represents
30 ordered nodes and 4 Link slots with path, neighbor and opaque 2B, plus an opaque
2B tail. Its lossless source plan has 1 span of 600B. Strict JSON and shape checks
are separate from authenticity; the model does not infer a 45/29 cap or require
opaque bytes to be zero.

One warning-free Windows core-only build, 6c2ef15a.., and one Wine filter passed
23 guards and 566 original calls, with independent return values and read addresses.
All 300 semantic B were observed; 300 opaque B were preserved unread. The checks
included 16 byte aliases and 24 calls outside the projection. Callee registers, SP,
RAM outside the stack, VRAM, OAM and palette were checked; this was not native, full
state-clock, BFS or ordinary-use acceptance. All 24 preservation guards and 215
frozen sources matched; the helper found 0 paths. There was no gameplay, frontend
build or repeat of old oracles; e77-d76 and schema 1 were unchanged. The next steps
were managed publication, typed guest consumption, real CLI and a retained world
scene. Linux, USA, native Windows, new 32:9 and the full TODO remained pending, with
no measurements or deployment.
[Model and proof](../rom_data_maps.md#typed-world-nodes-and-original-reader-proof).


**ROMDATA-03/04-WORLD-NODE-RUNTIME completed EUR/Wine (2026-10-28).** e78-d77
publishes world-nodes.json as gameplay.world-nodes at index 69: 71 files and 70
resources, with all 69 previous records and files exact and USA e8-d7 unchanged.
The 600B binding and 7 statistics fields cover reads of coordinates, path, neighbor
and opaque data from the model. The owner composes with prior, raw and edge reads
without imposing a guest-input cap or replacing placement, pathfinding, ABI, save
or cache behavior.

One Wine filter passed 12 attachment guards, 2 full-set guards and interrupted
publication checks. It also passed 15 native full state-clock prefixes covering 32B of compiled source and 124
compiled instructions, with all 600B raw source bytes altered after Load;
cases included alias, mirror, outside 30-31, equal 255 and a RAM root override. Bus
checks covered all 600B, 4 field classes, 30 nodes, widths, edges and a prior owner.
Load, Undo and unavailable-file behavior passed, as did 3 DMA/CpuSet/CpuFastSet
full state-clock checks.

The first 2 builds were rejected because stale core-cache dep-info pointed to an
old copy; touching the new copy was ineffective, so only the specific fingerprint
was moved. The third build was warning-free, fresh=false, 2cf1d882.., from 502 stable
sources. All 200 guards, 502 sources and 215 model preimages matched. The helper
retired only the interrupted copy, retaining the accepted dataset and executable.
At this checkpoint, real CLI and an ordinary scene remained to do. No new gameplay,
repeat of the old model or proof, or measurement occurred. Linux, USA, native
Windows, new 32:9, road 397018 and the full TODO remained pending; parallel DBG/u16
work was preserved.
[Consumption and evidence](../rom_data_maps.md#managed-world-nodes-and-typed-guest-reads).


**ROMDATA-04-WORLD-NODE-CLI completed EUR/Wine (2026-10-28); ordinary partial.**
A warning-free Windows production build, e0491cdc.., used 502 stable sources; the
canonical hash and mtime were restored. Three Wine CLI commands exited 0: prepare
created the dataset, prepare reused it and runc ran normal input for 16 frames.
All 71 files matched the runtime dataset, 70 ID hashes matched, and 73 managed paths
were unchanged on reuse and startup; the cache loaded 28859 blocks.

Exact copies of normal Giza 01/02 state and input covered 19049→19065: 402 reads of
402B of coordinates across 22 nodes. Path, neighbor and opaque usage were 0, and no
full pair was run. A private matcher expected relative frame 16 while the CLI showed
absolute frame 19065; it was corrected offline after all 3 exits of 0 and guard checks,
without a repeat. The 16-frame bound without path/neighbor use was closed. The next
lead was source callers or a justified normal state before another observation. All
169 guards and 502 sources matched; the helper found 0 paths. No new fixture,
navigation, repeat of previous proof or measurement occurred. At this checkpoint,
ordinary use of 3 classes, the full pair, road 397018, the full TODO, Linux, USA,
native Windows and new 32:9 remained pending; DBG/u16 work was preserved.
[CLI and ordinary limits](../rom_data_maps.md#actual-world-node-cli-and-ordinary-coordinate-use).


**ROMDATA-01-WORLD-NODE-ORDINARY-CALLERS and ROMDATA-05-WORLD-NODE-TRAVEL-ORDINARY
completed EUR/Wine (2026-10-28).** A direct BL query, 10 windows and 27 guards link
3B012→4DB1C→4DB80 to view 1A (dest−1), distinct from parent 7BB, with output at
7C1. Reader 47034 calls 4DAD0 when progress!=FF; the entity is at parent 7B8+52*i.
Two existing normal checkpoints, with node 17, selected 20→19, progress FF and
entity 1 moving node 0→1, justify the 360-frame bound. They do not establish an
exact PC/frame attribution or all callers.

One positive CLI run of 360 frames and one full pair of 360 frames passed with
6841B: 4820 coordinates, 864 path and 1157 neighbor bytes across 29 nodes. All
state-clock, classic/central/extended images, OAM, dispatch and five audio streams
matched, and counters matched the CLI. Opaque usage of 0 remains distinct from
targeted tests. Dispatch counts were 2006894/3620371 and audio counts were
790020/790020/95040/95040/789942. One warning-free Windows build, dabb1ac6.., used
502 sources and an optional 5-line guard. Removing the guard restored the old
validator exactly; runtime and core were unchanged. All 103 guards, 502 source
hashes, 27 caller guards and 169 CLI guards matched; the helper found 0 paths.

The first 16-frame negative path/neighbor bound remains closed. There was no new
fixture, navigation, guest edit, repeat of old proof or measurement. This closes
the ordinary case, not all nodes or maps, ordinary opaque use or the full
pathfinder. Linux, USA, native Windows, new 32:9, road 397018 and the full TODO
remained pending; DBG/u16 work was preserved.
[Ordinary evidence and gates](../rom_data_maps.md#ordinary-world-node-travel-and-caller-gates).


**ROMDATA-01-WORLD-ROAD-PATHS EUR source projection completed (2026-10-28).**
The 45 path IDs from 90 active links across 30 nodes select 45 headers of 8B at
397018..397180, each with a u8 count, opaque 3B and u32 ROM pointer. The 45 distinct
contiguous payloads at 395F00..397018 contain 2188 signed i8 XY pairs, or 4376B.
The 46 spans total 4736B: 4601 semantic and 135 opaque. The root store at parent 680
was reused.

Reader 4DC uses byte 0 for cost; 4DD0C takes flag, index and low u8 step without a
45 cap. A step outside the count writes zero 32 without loading a pointer or sample;
reverse order does not negate coordinates, and the reader packs two i16 values and
returns the output pointer. Reader 4DD70 selects the flag for current>target, while
4DD84 finds the minimum node; the caller adds node XY*8 to offsets. Reader 46B64 uses
at most 5 sample differences for facing, without proving the full ABI. A candidate
query found 3 root literals, 3 point calls, 1 direction and 1 minimum call; it does
not rule out indirect calls globally. Five endpoints, 4/6/8/10/12, are 1 pixel short
and retained; the absence of zero points and observed count range 32..85 are not
constraints.

All 6 windows, 60 pins and 17 guards passed; the projection is disjoint from 70 owners
and 49747 spans. The helper found 0 paths. No build, game, test, fixture, repeat of old
proof or measurement occurred. The next steps were a lossless model/source plan and
targeted Wine oracle; extraction, runtime, CLI, ordinary use, global catalog, opaque
meaning, Linux, USA, native Windows, new 32:9 and the full TODO remained pending.
DBG/u16 work was preserved.
[Road contract](../rom_data_maps.md#world-road-path-source-contract).


**ROMDATA-02-WORLD-ROAD-MODEL completed EUR/Wine (2026-10-28).** The standalone
world_roads model contains 45 ordered headers with u8 count, opaque 3 bytes and
encoded pointer, and 2,188 signed XY pairs. Its shared source plan has 46 spans and
4,736 B. Strict JSON and a consistent partition are checked separately from
authenticity. The 5 short endpoints, a possible zero sample and structural counts
0/255 are retained without inferring a 32..85 cap.

One warning-free Windows core-only build, de9d1e11.., and one Wine filter passed
29 guards and 2,537 original calls: 2,188 forward points, 135 reverse, 180 exits,
6 u8 aliases, 6 external headers and 22 order helpers. Outputs, returns and read
sequences matched exactly. All 4,601 semantic B were read; 135 opaque B were
reconstructed and remained unread. All 32 preservation guards and 504 build files
(218 core) matched; the helper found 0 paths. No ordinary scene, fixture, frontend
build, measurement or repeat of the closed node proof occurred. Schema 1 and e78-d77
were unchanged. The next steps were managed extraction, typed guest reads, real CLI
and one positive ordinary scene. Linux, USA, native Windows, new 32:9, global catalog
extension and the full TODO remained pending.
[Model and proof](../rom_data_maps.md#typed-sampled-world-roads-and-original-reader-proof).


**ROMDATA-03-04-WORLD-ROAD-RUNTIME completed EUR/Wine (2026-10-28).** e79-d78
publishes gameplay.world-roads and world-roads.json at index 70: 72 files and 71
resources. All 70 previous records and files were exact; USA 8-7 and schema 1 were
unchanged. The immutable count, pointer, sample and opaque owner uses 46 spans of
4736B. It composes with the world node at the left boundary and raw data at the
right and for headers 45..255; movement, facing and pathfinding remain unchanged.

Two warning-free Windows builds used 506 stable files, with 52be2981.. accepted. The
first filter closed publication and attachment, then failed native case 3 because page
396 was omitted. After fixing the page gate, the remaining native/bus/lifecycle
filter passed without republishing: 12 attachment guards, 2 full-set guards and 12
original/interpreter/native full state-clock prefixes. Those prefixes used 67 compiled
instructions and 44B explicit source bytes, with 4736B of source poisoned after Load.
All 4736B, 4 field classes, 45 paths, widths, mirrors, edges, prior ownership, Load,
Undo and 3 DMA/CpuSet/CpuFastSet checks passed. All 221 guards, 506 source hashes
and 504 model preimages matched; the helper retired only the interrupted copy.

No scene, model, node or fixture was repeated, and no measurement or ABI change
occurred. Real CLI and one positive ordinary scene were next; Linux, USA, native
Windows, new 32:9, global extent and the full TODO remained pending.
[Consumption and evidence](../rom_data_maps.md#managed-world-roads-and-typed-guest-reads).


**ROMDATA-04-WORLD-ROAD-CLI completed EUR/Wine (2026-10-28).** A warning-free
Windows production build, 1a86d341.., used 506 accepted sources; canonical hash and
mtime were restored. Three Wine commands prepared a new dataset, reused it and ran
runc for 360 frames. All 72 files matched the runtime dataset, 71 ID hashes matched,
and 74 paths retained hash and mtime; the old cache loaded 28859 blocks.

Copies of normal Giza 01 state and 02 input covered 19049→19409: 2976 reads of 5067B,
comprising 885 count, 2788 pointer, 1394 sample and 0 opaque bytes across 40 paths.
All 3 semantic classes were positive. All 180 guards and 506 source hashes matched;
the helper found 0 paths. No new scene, navigation, measurement or repeat of closed
proof occurred. The next step was one ordinary full pair with identical CLI counts;
this did not establish all 45 payloads, maps, ordinary opaque use or global extent.
Linux, USA, native Windows, new 32:9 and the full TODO remained pending.
[CLI and consumption](../rom_data_maps.md#actual-sampled-road-cli-and-ordinary-consumption).


**ROMDATA-05-WORLD-ROAD-TRAVEL-ORDINARY completed EUR/Wine (2026-10-28).** One
normal Giza 01/02 Wine full pair of 360 frames used CLI dataset e79-d78 from
19049→19409. All 360 state-clock, classic/central/wide, OAM, dispatch and five audio
checks matched. Seven counts matched the CLI: 2976 reads of 5067B, comprising
885 count, 2788 pointer, 1394 sample and 0 opaque bytes, with 40 paths used in any
field, not necessarily 40 payloads. An optional 5-line guard preserved the old
validator byte for byte when removed; one warning-free Windows build, 4c453f07..,
used 506 stable files. All 100 guards, 506 source hashes and 180 CLI guards matched;
the helper found 0 paths.

No new scene, fixture, navigation, measurement or repeat of closed source, model,
runtime or node proof occurred. The ordinary road case was closed; all 45 payloads,
maps, ordinary opaque use, full ABI, global extent, Linux, USA, native Windows, new
32:9 and the full TODO remained pending.
[Ordinary equivalence](../rom_data_maps.md#ordinary-sampled-road-travel-equivalence).


**ROMDATA-01-ENCOUNTER-CATALOG-READERS EUR contract completed (2026-10-28).**
The declared physical catalog has 442 headers of 40B, indexed 1..442. Its indexing
origin is 70C8CC and the first header at 70C8F4 is supported by two independent roots;
the next family at 710E04 uses independent 28B records. The catalog spans 17680B.
Neither the previously observed maximum 442 nor constant 300 is assumed to be a
count or guest cap. Links from all 442 headers form a contiguous chain of 2850
48B templates, totaling 136800B, and corroborate the partition.

Each header has 17 semantic fields covering 33B and 7 opaque bytes. Original readers
narrow indices to low 16 bits without a 442 guard. Index 0 is a coordinate prefix:
its last 16B are payload selected by header 442, not a valid header. The pointer at
header 29+4, address 70CD58, is already owned by constructor-alternates. The new
plan checks that dependency and consistency, skips the shared 4B and has 2 new spans
of 17676B: 14582 semantic and 3094 opaque. These are disjoint from 71 owners and
49793 spans. The existing 192B template remains owned; unit and coordinate payloads
are separate and were not migrated.

The query identified 27 loaders from cached roots. All 40 windows (39 new and 1
reused), 54 pins and 68 guards passed; the helper found 0 paths. No build, game, test,
fixture or repeat of prior proof occurred. The next step was a lossless 442-header
model with the shared pointer and one Wine oracle; extraction, runtime, CLI, ordinary
use, all indirect callers, global extent, Linux, USA, native Windows, new 32:9 and
the full TODO remained pending, without measurement.
[Headers and ownership](../rom_data_maps.md#european-encounter-headers-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-HEADERS-MODEL completed EUR/Wine (2026-10-28).** The
lossless encounter_headers model contains 442 headers indexed 1..442, with 17 fields
and 7 opaque bytes each, covering 17680B. Its 2 spans own 17676B and exclude the
4-byte field at 70CD58 owned by constructor-alternates. Strict JSON and shape checks
are separate from authenticity and dependency consistency: a modified shared pointer
can leave the new spans unchanged but is rejected. The model preserves low u16 aliases
and returns None for 0 and 443..65535, allowing prior/raw fallback. It does not infer
a count cap, null-pointer policy, enum restriction or opaque-zero rule.

One warning-free Windows core-only build, 5855af92.., and one Wine filter passed
29 guards and 7523 probes: 7072 single-load prefixes and 451 complete C0034 calls
(442 declared headers, 6 aliases and 3 outside). All 14586 semantic B, including
the 4 shared bytes, were observed; 3094 opaque B were reconstructed but unread.
Register, flags and PC for prefixes; callee registers, SP and LR for the gate; and
RAM outside stack 4, VRAM, OAM and palette were checked. This does not establish full
ABI, importer, native clock or ordinary use. All 33 guards and 508 files (221 core)
matched; the helper found 0 paths. e79-d78 was unchanged, with no old oracle, scene,
fixture or measurement repeated.

The next steps were managed publication, typed bus access, source pages 70C..710 and
the dependency, then CLI and ordinary use. Nested payloads, all callers, global
extent, Linux, USA, native Windows, new 32:9 and the full TODO remained pending.
[Model and proof](../rom_data_maps.md#typed-encounter-headers-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-HEADERS-RUNTIME completed EUR/Wine (2026-10-28).**
e80-d79 publishes encounter-headers.json at index 71: 73 files and 72 resources, with
all 71 prior records exact. It requires constructor-alternates and validates the
shared pointer. The owner has 2 spans of 17676B excluding the 4 old bytes, source
pages 70C..710, 8 field classes, a 442-header mask and 11 statistics. Guest algorithms,
indices, ABI, cache and save behavior were unchanged, as were USA 8-7 and schema 1.

One warning-free Windows build, 1c4135f5.., used 510 stable files. One Wine filter
passed 14 attachment guards, 3 full-set guards and failed-owner/state retention. It
also passed 26 original/interpreter/native full state-clock prefixes with 17680B of
raw headers poisoned, covering 31 compiled instructions, 37 new B and 4 old B. All
17676 new and 4 shared B, 8 classes, 442 headers, widths, mirrors, edges, prior owner,
Load and Undo passed, as did 3 DMA/CpuSet/CpuFastSet checks. All 206 guards,
510 source hashes and 508 model preimages matched; the helper retired only the
interrupted copy. No scene, fixture, measurement, 7523-probe model proof or closed
node, road or constructor proof was repeated. Real CLI and ordinary use were next;
nested payloads, all callers, global extent, Linux, USA, native Windows, new 32:9
and the full TODO remained pending.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-headers-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-HEADERS-CLI completed EUR/Wine (2026-10-28).** One
warning-free Windows production build, 9d0a772d.., used 510 accepted sources; the
canonical hash and mtime were restored. Three Wine commands passed: prepare created
the dataset, prepare reused it and runc executed 700 frames. All 73 files matched the
runtime dataset, 72 ID hashes matched and 75 paths were unchanged; 28859 cache blocks
loaded. Copies of normal Giza 02 menu and 03 input covered 19409→20109, with 17 reads
of 41B, six positive semantic classes, 1 header and 0 opaque bytes.

All 184 guards and 510 source hashes matched; the helper found 0 paths. No new
navigation, checkpoint, measurement or repeat of closed proof occurred. The next step
was one full pair with identical CLI counts. This did not establish all 442 headers,
payload, ordinary opaque use, full ABI or the global catalog. Linux, USA, native
Windows, new 32:9 and the full TODO remained pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-header-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-HEADERS-BATTLE-ORDINARY completed EUR/Wine (2026-10-28).**
One normal 700-frame full pair used Giza 02 menu and 03 input, 19409→20109, with CLI
dataset e80-d79. Every full state and clock, classic/central/wide image, OAM, dispatch
and 5 audio streams passed. All 11 counts matched the CLI: 17 reads of 41B, comprising
12 count, 1 map, 4 template-pointer, 0 parameter, 1 placement-count, 20 position-pointer,
3 control and 0 opaque bytes, for one header and six positive classes. Ordinary
parameter and opaque reads were not observed; the earlier targeted evidence remains
separate.

An optional 5-line guard restored the old validator byte for byte when removed. One
warning-free Windows build, 43fcc614.., used 510 sources, and one Wine filter passed.
All 103 ordinary and 184 CLI guards and each set of 510 hashes matched; the helper
found 0 paths. No new fixture, navigation, repeat of closed proof or measurement
occurred. The ordinary case was closed; all 442 headers, payload, full ABI, global
catalog, Linux, USA, native Windows, new 32:9 and the full TODO remained pending.
[Equivalence](../rom_data_maps.md#ordinary-encounter-header-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-UNIT-TEMPLATES completed for the EUR source contract
(2026-10-28).** The 442 authenticated header links partition 2,850 records of 48B at
6EA048..70B6A8, totaling 136,800B; this is an independent complete projection.
The readers distinguish selectors and arguments, five u16 equipment values, halfword
12, the physical bitset at 14..27, controls 28/29, flags 2A/2C/2D and opaque bytes
2E..2F. Dynamic bit indices have no proven 20B or 160-bit guest cap. All 14,250
equipment references fall within 376 items; selectors agree with metadata14 and
metadata52 owners, but observed values are not caps.

The existing constructor owner retains 192B at 6ECA78..6ECB38, records 225..228
under header 29. The new ownership plan excludes these bytes from its two spans of
136,608B, requires full 48-byte coherence and header/constructor dependencies, and
does not duplicate any of 49,795 spans owned by 72 owners. All eight windows (5 new
and 3 reused), 39 pins, 4 BL targets and 41 guards passed. No build, game, Wine/Linux
test, fixture, repeat of old proof or census, or measurement occurred. The next steps
were a strict model and one targeted Wine oracle; managed/runtime/CLI/ordinary use,
coordinate pool, global extent, full ABI, Linux-USA, native Windows, new 32:9 and the
full TODO remained pending.
[Contract](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-UNIT-TEMPLATE-MODEL completed EUR/Wine (2026-10-28).**
The strict lossless model contains 2,850 records of 48B, totaling 136,800B; its two
spans own 136,608 new bytes. The 192B owned by the previous constructor are excluded
from byte_at but compared in full. Shape and JSON validation, header partition, shared
coherence, item/metadata14/job references and source authentication remain distinct.
The model retains full u8/u16 values and opaque 2 bytes, without inventing value caps
or a 160-bit guest cap.

One warning-free Windows core-only build, e0ae0d5c.., used 512 sources, including
224 core inputs. One Wine filter passed 38 guards and 2,100 selected original
prefixes: 1,020 LDR and 1,080 bit cases, including 120 indices beyond the 20B field.
The probes observed 1531 new, 102 old and 4 raw distinct bytes; registers, addresses,
values, RAM, VRAM, OAM, palette and ROM were exact. The full projection was checked
independently; this is not full ABI or semantic coverage of all 2,850 records.
All 34 guards, 512 source hashes and 41 source-contract guards matched; the helper
found 0 paths. No new fixture, measurement or repeat of closed proof occurred. The
next steps were e81-d80/index 72 publication and typed consumption, then CLI and
ordinary use. Linux-USA, native Windows, new 32:9, coordinate pool, global extent,
full ABI and the full TODO remained pending.
[Model and evidence](../rom_data_gameplay.md#typed-encounter-unit-templates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-UNIT-TEMPLATE-RUNTIME completed EUR/Wine (2026-10-28).**
e81-d80 publishes encounter-unit-templates.json at index 72: 74 files, 73 resources,
all 72 previous records exact and five dependencies. The 192B constructor range stays
with its old owner. The immutable owner has two spans totaling 136,608B, nine byte
classes, record/header statistics and local/global 6EA..70B routing.

One warning-free Windows build, 3d4b2af2.., used 514 sources. One Wine filter passed
14 attachment guards, 7 full-set guards and transactional failure cases, along with
27 original/interpreter/native prefixes matching full state and clock after 136800B
of raw source was altered following Load. Compiled reads covered 25 new B and 8 prior
B. All 136608 new B and 192 old B, 9 classes, 2846 records, 441 headers, widths,
mirrors, boundaries, Load and Undo passed. DMA used 34202 words, CpuSet 68404
halfwords and FastSet 34208 words; these also passed. All 215 guards, 514 source
hashes and 512 model preimages matched; the helper retired only the interrupted copy.
ABI and algorithms were unchanged, as were USA 8-7 and schema 1. Real CLI and
ordinary use were next; the coordinate pool, all callers, full ABI, global extent,
Linux-USA, native Windows, new 32:9 and the full TODO remained pending, without
measurements.
[Consumption and evidence](../rom_data_gameplay.md#managed-encounter-unit-templates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-UNIT-TEMPLATE-CLI completed EUR/Wine (2026-10-28).**
One warning-free Windows production build, afeaef25.., used 514 accepted runtime
sources; the canonical hash and mtime were restored. Three Wine commands passed:
prepare created the dataset, prepare reused it and runc executed 700 frames. All 74
files matched runtime, 73 ID hashes matched and 76 paths were unchanged; the cache
loaded 28859 blocks. Copies of normal Giza 02 menu and 03 A input covered 700 frames,
19409→20109: 186 reads of 211B across 5 records and 1 header, with 8 positive classes
and 1 unobserved. The positive selector/argument/equipment/flag criterion was met;
all 13 counts were retained for the subsequent full pair.

All 184 guards and 514 hashes matched; the helper found 0 paths. No new fixture,
navigation, measurement or repeat of closed proof occurred. The ordinary full pair,
all 2850 records and 442 headers, full ABI, coordinate pool, global extent, Linux-USA,
native Windows, new 32:9 and the full TODO remained pending.
[CLI and consumption](../rom_data_gameplay.md#actual-encounter-unit-template-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-UNIT-TEMPLATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair covered 700 frames,
19409→20109, using CLI dataset e81-d80. Every full state and clock, classic/central/
wide image, OAM, dispatch and 5 audio streams passed. All 13 counts matched the CLI:
186 reads of 211B, comprising 35 selector, 15 argument, 80 equipment, 10 auxiliary,
41 bit, 10 optional, 10 flag, 10 control and 0 opaque bytes, across five records and
one header. Eight semantic classes were positive; ordinary opaque use was not
observed and remains covered separately by the bus proof.

An optional 5-line guard restored the old validator byte for byte when removed. One
warning-free Windows build, 33c61082.., used 514 sources and one Wine filter passed.
All 104 ordinary and 184 CLI guards and each set of 514 hashes matched; the helper
found 0 paths. No new fixture, navigation, measurement or repeat of closed proof
occurred. This case was closed; all 2850 records and 442 headers, full ABI, coordinate
pool, global extent, Linux-USA, native Windows, new 32:9 and the full TODO remained
pending.
[Equivalence](../rom_data_gameplay.md#ordinary-encounter-unit-template-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-COORDINATES completed for the EUR source contract
(2026-10-28).** Header pointer/count links partition 691 lists containing 2342 XY
pairs, or 4684B, at 70B6A8..70C8F4. There are no gaps, aliases or overlaps among
49797 spans and 73 owners. The roles are 323 placement 0 lists totaling 3272B,
34 random lists totaling 680B, 12 placement 2 lists totaling 88B and 322 default
lists totaling 644B. Payloads belong to 323 headers; all pointers are null in 119.

Raw u8 XY values preserve signed search and unsigned reads. RNG 02868 returns a
nonnegative 15-bit value; remainder 146314 and the u16 mask remain original guest
behavior. Neither the observed 10 nor the 16-bit mask establishes a guest cap, and
null pointers do not acquire a safe fallback. Nine reused and three new helper/RNG
windows, 41 pins and 38 guards passed. No build, Wine/Linux test, game, fixture,
navigation, repeat of closed proof or measurement occurred. The next steps were a
strict 691-list model with header coherence and one targeted Wine oracle, then
extraction, runtime, CLI and ordinary use. Schema 1, e81-d80 and USA 8-7 were
unchanged; Linux-USA, native Windows, new 32:9, all callers, full ABI, global extent
and the full TODO remained open.
[Contract](../rom_data_maps.md#european-encounter-coordinate-lists-and-original-readers).


**ROMDATA-02-ENCOUNTER-COORDINATE-MODEL completed EUR/Wine (2026-10-28).**
The strict model retains 691 lists, 2342 raw u8 XY pairs, 4684B and 691 spans. Shape,
header coherence and authenticity are separate checks. A signed diagnostic view is
available; no safe null fallback or value cap is inferred.

The first Windows core build failed because a test lacked the Bus import; only that
import was corrected. The accepted second 516-source copy, 2fe3e236.., built without
warnings with 227 core inputs; preimages were retained. One Wine filter passed 30
guards and 8044 original signed/unsigned prefixes across all 4684B, plus 12 isolated
high-bit cases and 102 original random prefixes across 34 lists and 3 seeds. RNG,
index, addresses, u16 mask, writes, callee registers, SP, outer RAM, VRAM, OAM,
palette and ROM were exact. This does not establish full ABI, repeated random
selection or exhaustion, native clock or ordinary use.

All 40 guards, 516 accepted and 516 failed-build source hashes, and 38 source guards
matched; the helper found 0 paths. No new fixture, navigation, repeat of closed proof
or measurement occurred. The next step was managed e82-d81/index 73 and typed bus
access, followed by CLI and ordinary use; Linux-USA, native Windows, new 32:9, all
callers, global extent and the full TODO remained pending.
[Model and proof](../rom_data_maps.md#typed-encounter-coordinates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-COORDINATE-RUNTIME completed EUR/Wine (2026-10-28).**
e82-d81 publishes encounter-coordinates.json at index 73: 75 files and 74 resources,
with all 73 previous records exact and an encounter-header dependency. The immutable
owner covers 691 spans and 4684B, with four roles and XY/list/header statistics.
Local and global routing covers 70B..70C and composes with old owners at the edges.

Three warning-free Windows builds used 518 sources; 0f85fbf7.. was accepted. The
first Wine filter passed publication, 14 attachment and 3 full-set guards, then
stopped in the native harness. A second isolated concatenation beyond a stop. Only the
test was corrected with a BL pair and two families that exclude their respective
stops. The third filter reused the dataset, retained all 22 cases and completed the
native/lifecycle checks without repeating publication or attachment. All 22 original/
interpreter/native cases matched full state and clock after 4684B of raw source was
altered following Load: 18 loads, an index 0 gate and 3 random prefixes. Compiled
reads covered 21 new B, 2 template B and 13 header B.

Bus and lifecycle checks passed for all 4684B, 691 lists, 323 headers, XY roles,
691 boundaries, widths, mirrors, Load, Undo, DMA, CpuSet and FastSet. All 253 guards,
518 source hashes and 516 model preimages matched; the helper retired only the
interrupted copy. Algorithms and ABI were unchanged. Real CLI and ordinary use were
next; Linux-USA, native Windows, new 32:9, full ABI, all callers, global extent and
the full TODO remained pending. No fixture, navigation, measurement or repeat of old
proof occurred.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-coordinates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-COORDINATE-CLI completed EUR/Wine (2026-10-28).** One
warning-free Windows production build, b467d4ad.., used 518 accepted sources; the
canonical hash and mtime were restored. Three Wine commands passed: prepare created
the dataset, prepare reused it and runc executed 700 frames. All 75 files matched
the runtime dataset, 74 ID hashes matched and 77 paths remained unchanged; 28859
cache blocks loaded. Copies of normal Giza 02 menu and 03 input covered 19409→20109:
20 reads of 20B, across 1 list and 1 header, with placement 0, X and Y positive. All
10 counts were retained.

All 186 guards and 518 source hashes matched; the helper found 0 paths. No new
fixture, navigation, measurement or repeat of closed proof occurred. A full pair was
next; this did not establish all 691 lists, roles, headers or ABI. Linux-USA, native
Windows, new 32:9, global extent and the full TODO remained pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-coordinate-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-COORDINATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair covered 700 frames,
19409→20109, using CLI dataset e82-d81. Every full state and clock, classic/central/
wide image, OAM and 5 audio streams matched; native and fallback dispatch executed.
All 10 counts matched the CLI: 20 reads of 20B, with placement 0=20, X 10 and Y 10,
covering one list and one header. Random, placement 2 and default roles were not
observed here; prior targeted tests remain separate.

An optional 5-line guard restored the old validator byte for byte when removed. One
warning-free Windows build, de67e015.., used 518 sources and one Wine filter passed.
All 104 ordinary and 186 CLI guards and each set of 518 hashes matched; the helper
found 0 paths. This case was closed; all 691 lists and 323 headers with payload,
placement/RNG, full ABI, global extent, Linux-USA, native Windows, new 32:9 and the
full TODO remained pending. No new fixture, navigation or performance measurement
occurred.
[Equivalence](../rom_data_maps.md#ordinary-encounter-coordinate-battle-entry-equivalence).


**ROMDATA-01-LINKED-METADATA28 completed for the EUR source contract
(2026-10-28).** The physical partition contains 347 records of 28B, or 9716B, at
710E04..7133F8. It is supported by 17 roots and 28B arithmetic; the following 4B
family has two independent readers. The physical boundary does not establish a guest
cap of 347. Getter D07A4 narrows the index to low16 and selector to low8. It exposes
scalars, an internal view at +12, a flag word and 21 bits; selectors 34..255 return
0 without reading record data. Direct readers use u16 at +22 and bytes 24..26;
opaque bytes 3/15/27 are retained.

All 347 records were reconstructed without overlap with 74 owners or 50488 spans.
All 20 windows (14 new, 6 reused), 69 pins and 122 guards passed; the helper found
0 paths. This is a static contract only; model, Wine, runtime, CLI and ordinary use
were next. Schema 1 and e82-d81 were unchanged. Linux-USA, native Windows, new 32:9,
global extent and the full TODO remained pending; there was no build, game,
checkpoint, navigation, measurement or repeat of closed proof.
[Contract](../rom_data_gameplay.md#european-indexed-28-byte-metadata-source-contract).


**ROMDATA-02-LINKED-METADATA28 completed EUR/Wine (2026-10-28).** The lossless
model retains 347 records and 9716B of unsigned fields, flag32 and opaque bytes.
Shape and source authentication are separate. Low16/low8 lookup preserves scalars,
the address view at +12, 21 bits and a default with no data read; it introduces no
guest cap or null fallback.

One warning-free Windows core build, 358dba2c.., used 520 stable sources. One Wine
filter passed 27 guards and 14517 original prefixes: 12088 complete getters, 1388
tail loads and 1041 inline prefixes. They observed 8675 interpreted bytes; 1041
opaque bytes were reconstructed. The 9 isolated variants passed 342 prefixes. RAM,
callee registers, SP-LR, VRAM, OAM, palette and ROM were exact. All 36 model guards,
520 source hashes and 122 source-contract guards matched; the helper found 0 paths.
Schema 1 and e82-d81 were unchanged. Managed/runtime integration was next, followed
by CLI and ordinary use. Full ABI, the following 4B family, global extent, Linux-USA,
native Windows, new 32:9 and the full TODO remained pending. No measurement, new
fixture, navigation or repeat of old closed proof occurred.
[Model and proof](../rom_data_gameplay.md#typed-indexed-28-byte-metadata-and-original-reader-proof).


**ROMDATA-03-04-LINKED-METADATA28-RUNTIME completed EUR/Wine (2026-10-28).**
e83-d82 publishes linked-metadata28.json at index 74: 76 files and 75 resources, with
all 74 previous records exact. Its one 9716B span introduces no invented dependency.
The immutable typed owner has 5 classes, 347 records and 8 counters; local and
global pages 710..713 compose with the previous header and raw successor.

One warning-free Windows build, 040133f5.., used 522 sources. One Wine filter passed
publication/interruption, 14 attachment guards and 2 full-set guards. It also passed
23 native full state-clock cases with 9716 raw bytes altered after Load: 17 loads and
6 D07A4 getters, with 31 new bytes and 1 header byte consumed by compiled code.
Checks passed for all 9716 bytes, 5 classes, 347 records, all 347 boundaries, Load,
Undo and 3 DMA/CpuSet/FastSet state-clock comparisons. All 216 guards, 522 source
hashes and 520 model preimages matched; the helper retired only the interrupted copy.
Actual CLI and ordinary use were next; ABI, USA 8-7 and schema 1 were unchanged.
There was no new game, checkpoint, navigation, measurement or repeat of closed proof.
Full ABI, global extent, the next 4B family, Linux-USA, native Windows, new 32:9 and
the full TODO remained pending.
[Direct consumption](../rom_data_gameplay.md#managed-indexed-28-byte-metadata-and-typed-guest-reads).


**ROMDATA-04-LINKED-METADATA28-CLI registration completed EUR/Wine;
positive ordinary use still open (2026-10-28).** One warning-free Windows production
build, 0acb352e.., used 522 sources. Its only correction changed 3 metadata28 messages;
the inverse was byte-exact, the other 521 files were exact, and canonical hash and
mtime were restored. Create, reuse and normal starts retained 76 files identical to
the runtime dataset, 75 matching ID hashes and 78 unchanged paths; 28859 cache blocks
loaded.

Four normal locators, Giza 700, Jobs 540, Abilityset 700 and BattleTechRinfo 800,
had all 8 counters at 0. They are retained negative observations, not cases to repeat,
and no full pair was run. Menus were observed using new normal-command input on an
authorized copy, without a new checkpoint or manual guest edit. All 220 guards and
522 source hashes matched; the helper found 0 paths. The next lead was a combat path
with a nonzero action before a positive/full pair. Linux-USA, native Windows, new
32:9, full ABI, global extent and the full TODO remained pending; no measurement
occurred.
[CLI and limits](../rom_data_gameplay.md#actual-indexed-metadata28-cli-registration-and-ordinary-locators).


**ROMDATA-04-05-LINKED-METADATA28-COMBAT completed EUR/Wine (2026-10-28).**
Normally opening the Steal menu on dry ground at Giza covered 42 frames, 22669→22711,
with 15 reads serving 20 bytes across five records: 10 bytes from halfword fields
and 10 from byte fields.
Inline, flag and opaque usage were 0. Capture, existing CLI 0acb352e.. and one Wine
full pair matched all 8 counts; every full state and clock, classic/central/wide image,
OAM and 5 audio streams passed. No ability effect was executed, and coverage of all
347 records was not claimed.

One warning-free Windows test build, 5a6c81be.., used 523 sources, an optional 5-line
guard that restored the old validator byte for byte when removed, and a normal-input-
only observer. Seven captures covered 1182 frames on authorized copies, without manual
guest or original-save edits. All 177 guards, 523 source hashes, 220 CLI guards and
522 CLI hashes matched; 78 paths remained unchanged and the helper found 0 paths.
The four old negative locators remained closed; source, model, runtime, registration
and other-family evidence was not repeated. The ordinary case was closed; effects,
all 347 records, full ABI, global extent, the next 4B family, Linux-USA, native Windows,
new 32:9 and the full TODO remained pending, with no performance measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-metadata28-steal-menu-equivalence).


**ROMDATA-01-LINKED-RECORDS4 completed for the EUR source contract
(2026-10-28).** The range 7133F8..713744 contains 844 bytes in 211 physical 4-byte
positions. Six roots and original readers use a u8 index multiplied by 4; the next
family has a distinct boundary and stride 8. All four fields are read unsigned,
and original addresses are retained in live context. The first three inline positions
of 347 metadata28 records provide 1041 references to 194 distinct positions; three
references select index 0 and none select outside the partition.

Neither 211 nor the observed maximum 208 is a guest cap, and no global limit on the
three indices, enum or null fallback is inferred. Zero records 0/1/209/210 remain
distinct; reserved or padding meaning for the final two is still unproven. All 844
bytes were reconstructed without overlap with 75 resources or 50489 spans of e83-d82.
All 13 authenticated windows (6 new, 7 reused), 45 pins and 112 guards matched; the
helper found 0 paths. This was static analysis only, with no build, test, game,
fixture or repeat of closed evidence. The next steps were a model and targeted Wine
original-reader proof, then managed/runtime, CLI and ordinary use of this family.
Linux, USA, native Windows, new 32:9, full ABI, indirect catalogs and the full TODO
remained pending, without measurement.
[Contract](../rom_data_gameplay.md#european-linked-four-byte-record-source-contract).


**ROMDATA-02-LINKED-RECORDS4 completed EUR/Wine (2026-10-28).** The lossless core
model contains 211 physical positions, each with four u8 fields: 844 bytes in one
span. Strict shape, order, index and parser checks are separate from EUR
authentication and 13 windows. Original guest addresses are calculated for all 256
indices, including external 211..255; diagnostic None is not a guest fallback. Zero
records remain distinct and the meaning of the final two stays open, without an
enum, cap or global selector limit.

One warning-free Windows core build, a7270580.., used 525 stable sources including
233 core inputs. One Wine filter passed 20 guards and 1567 original calls or prefixes:
1055 reader probes covered all 844 bytes, while 512 full setter calls covered both
contexts, all 256 indices, low8 aliases and external addresses. Two isolated high-bit
variants passed 10 unsigned prefixes and were rejected by base authentication. Full
RAM except the expected 5 setter bytes, callee registers, SP-LR, IWRAM, VRAM, OAM,
palette and final ROM were exact; 9808 instructions are coverage only. All 125 guards,
525 hashes and 112 source guards matched; the helper found 0 paths.

Schema 1, e83-d82, save, native and CAPI layouts were unchanged. Managed/runtime,
CLI and ordinary use of this family were next; native clock, full ABI, Linux, USA,
native Windows, new 32:9, indirect catalogs and the full TODO remained pending. No
game, checkpoint, measurement or repeat of earlier closed evidence occurred.
[Model and readers](../rom_data_gameplay.md#typed-four-byte-linked-records-and-original-reader-proof).


**ROMDATA-03-04-LINKED-RECORDS4-RUNTIME completed EUR/Wine (2026-10-29).**
e84-d83 publishes linked-records4.json at index 75: 77 files and 76 resources, with
all 75 previous records and payloads exact. Its single 844-byte span has no invented
dependencies. The immutable owner serves four fields in 211 positions, with seven
counters. Local and global routing covers page 713 and composes bytewise with the
previous metadata28 owner and raw successor. No cap, null or selector limit was added;
save, native and CAPI ABI and USA 8-7 were unchanged.

Three Windows builds were recorded. The first failed because of a test reference and
stale menu cache; the second was warning-free but never executed. The third,
33144a8c.., was warning-free and used 551 current sources; only LDR alignment
expectations were corrected before Wine. One Wine filter passed publication and
interruption checks, 14 attachment guards, 2 full-set guards and owner/state retention.
It also passed 21 native full state-clock cases after all 844 raw bytes were altered
following Load: 13 loads, 4 getters, one double prefix and 3 setters across four
groups. They covered 41 instructions, 21 new bytes and 6 previous bytes.

All 844 bytes, four fields, 211 positions and all 211 boundaries passed, as did Load,
Undo with the same Arc, DMA3, CpuSet 424 halfwords and FastSet 216 words. All 237
guards, 551 source hashes, two 551 diagnostic sets, 525 model preimages and seven
current hashes matched; the helper retired only the interrupted copy. Actual CLI and
ordinary use were next; Linux, USA, native Windows, new 32:9, full ABI, external
catalogs and the full TODO remained pending. No measurement, new game, fixture or
repeat of old closed evidence occurred.
[Direct consumption](../rom_data_gameplay.md#managed-four-byte-linked-records-and-typed-runtime-reads).


**ROMDATA-04-LINKED-RECORDS4-CLI registration completed EUR/Wine;
positive ordinary use pending (2026-10-29).** One warning-free Windows production
build, bcf632fa.., used 551 sources identical to runtime, without an overlay. Only
corelib/production fingerprints were isolated; the canonical hash and mtime were
restored. Three Wine CLI children exited 0 for create, reuse and a new 120-frame
start. All 77 files matched runtime, 76 IDs/hashes loaded and 79 managed paths kept
their hashes and mtimes; 28859 cache blocks loaded.

A copy of the Steal menu at 22711, with Armor selected and A pressed at relative frame 8 and held for 12 frames, reached
22831 and showed targeting, but all 7 new counters stayed 0. This negative case was
closed without inferring positive use or a full pair; the CLI did not save the final
checkpoint. All 187 guards and 551 CLI plus 551 runtime hashes matched; the helper
found 0 paths and originals were unchanged. No new fixture or repeat of closed
evidence occurred. The next step was normal combat/target resolution with positive
reads, then full equivalence. Linux, USA, native Windows, new 32:9, full ABI, external
catalogs and the full TODO remained pending; no measurement or assistant change to
Android, package, commit or push occurred.
[CLI and limits](../rom_data_gameplay.md#actual-linked-four-byte-record-cli-registration).


**ROMDATA-04-05-LINKED-RECORDS4-COMBAT completed for one EUR/Wine ordinary case
(2026-10-29).** Normal Fight/Doit/facing commands reached the next turn. A 40-frame
suffix, 24755→24795, served 223 bytes in 223 reads: fields 64/132/11/16 across two
physical positions. All 7 counters matched between capture, existing BCF CLI and one
Wine full pair; each of 40 full states and clocks, classic/wide pixels, OAM and 5
audio streams passed. One warning-free Windows build, a5b2fc84.., used 552 sources:
a normal observer, test module and optional 5-line guard only. The inverse validator
was exact and core unchanged.

Ten captures covered 2084 frames on authorized copies, without manual guest, ROM or
cartridge edits. One classic CLI frame was UI only; subsequent framebuffer projections
were authenticated read-only without further execution. All 76 ID hashes and 79 paths
were unchanged, 28859 cache blocks loaded, and 210 guards, 552 source hashes, 187 CLI
guards and 551 CLI hashes matched; the helper found 0 paths. Prior zero cases remained
closed, without repeating source, model, native, lifecycle or registration proof.
All 4 fields were observed, not all 211 positions, attack success or full ABI. The
new 8B family, indirect catalogs, Linux-USA, native Windows, new 32:9 and the full TODO
remained pending, with no measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-linked-four-byte-turn-equivalence).


**ROMDATA-01-CONDITIONAL-RECORDS8 EUR source contract completed
(2026-10-29).** The D56A0 scan begins at 713744 with stride 8 and visits ordinals
1..127: 127 positions and 1016 bytes ending at the distinct root 713B3C, whose two
readers use stride 70. Byte 00 zero skips a position and byte 01 zero is a wildcard;
LE16 keys are at +2/+5 and u8 operands at +4/+7. Keys up to 05FF address live flags;
higher keys address u8 counters using key-0600. For operands 0/1 the first flag
predicate requires equality and the second inequality. Other flag operands pass those
comparisons without coercion. Counter predicates require counter>=operand for the first and counter<operand for the second; only second key 0
bypasses its predicate. No Boolean coercion, cap or header index is invented.

The source has 76 enabled rows and 51 zero rows, with 105 live references: 104 flags,
1 counter, 63 keys and 24 alias groups. Operand 2 is retained and 47 second keys are
0. Accepted outputs are original ROM pointers in order, with a count. Capacity and
filter are u32; there is no entry guard for capacity 0. Both observed callers pass 64
and a u8 filter. Helper 370EB0/145C10 remains classified as executable support, not a
new asset. Full reconstruction has SHA256
d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b and is disjoint
from 76 owners and 50490 spans. All 8 windows (6 new, 2 reused), 93 pins and 96
guards matched; the helper found 0 paths. An aligned pointer-word query found 1 root
and 38 successor references, none internal or at origin-8 in that representation;
2 call sites were verified.

This was static analysis only, with no build, Wine, game, fixture or repeat of old
proof. Schema 1, e84-d83, 77 files and 76 resources were unchanged. The next steps
were a family model and Wine readers; managed/runtime, CLI, ordinary use, Linux-USA,
native Windows, new 32:9, full ABI, indirect callers, successor extent and the full
TODO remained pending. No measurement occurred.
[Contract](../rom_data_gameplay.md#european-conditional-eight-byte-record-source-contract).


**ROMDATA-02-CONDITIONAL-RECORDS8 EUR/Wine model/oracle completed
(2026-10-29).** The lossless core model retains 127 positions and 1016 bytes in one
span. Strict parser, shape and order checks are separate from base ROM and 8-window
authentication. Gate, filter, LE16 keys and u8 operands remain literal, without an
enum, cap or null rule. Selection preserves original pointers and order, u32
capacity including zero-wrap, full u32 filter, live call multiplicity and the
different first/second predicate polarity.

One warning-free Windows core build, c6f5406d.., used 554 sources including 239 core
inputs. Only the new leaf, test and module were added; the inverse hub was byte-exact
and the core-test fingerprint isolated. One Wine filter passed 24 guards and 109
original D56CE..D57A4 cycles with real CCE98/CCF00 readers: 16 base scenarios and
93 isolated synthetic variants. These covered keys 0/05FF/0600/0682/FFFF, operands
0..255, header 255, filter 128/255 versus full context, and capacity 0/1/64/FFFFFFFF.
The Python oracle and model agreed with actual output, count, capacity, positions and
calls. They observed 612 base bytes and 134 synthetic positions, not all 1016 bytes.

EWRAM matched except expected output; IWRAM matched except 4-byte return-link scratch;
stack, r11/r12, VRAM, OAM, palette and final ROM were exact. The 198372 instructions
are coverage only. All 120 guards, 554 source hashes, 3 current hashes and 96 source
guards matched; the helper found 0 paths. No game, checkpoint, navigation, old proof
or reconstructed IWRAM helper was used. Helper entry, epilogue and full ABI were
excluded; schema 1, e84-d83, 76 resources, save, native and CAPI were unchanged.
Managed/runtime, CLI and ordinary use were next; Linux-USA, native Windows, new 32:9,
indirect calls, the 70B successor and the full TODO remained pending. No performance
or space measurement occurred.
[Model and original loop](../rom_data_gameplay.md#typed-conditional-records-and-original-selection-loop-proof).


**ROMDATA-03-04-CONDITIONAL-RECORDS8-RUNTIME completed EUR/Wine with combined
proofs (2026-10-29).** e85-d84 publishes conditional-records8.json at index 76: 78
files and 77 resources, with the previous 76 exact. Its single 1016B span has no
dependencies. The immutable owner has 6 classes, 127 slots and 9 counters. Page 713
is routed locally and globally, composing with prior records4 and the raw successor;
guest/live/address logic and save/native/CAPI ABI were unchanged.

Four warning-free Windows builds used 568 consistent current sources. The first was
never run after a pre-Wine correction to a test-only iteration limit. The first Wine
filter passed publication/interruption, 14 attachment guards, 2 full-set guards and
16 native full state-clock cases, then failed case 17 when the observer counted 607
rather than 609 bytes. Diagnosis of that case alone found identical raw/typed/native
state-clock and 609 actual bytes and completed bus/lifecycle checks. A final observer-
only run passed 609 callbacks: 607 direct and 2 interp_one fallback, with no
unattributed bytes. Only the test was corrected; publication, attachment, the 16
cases and bus proof were not repeated.

The 17 distinct cases comprised 14 loads, 2 LE prefixes and 1 scan, with all 1016
raw bytes altered after Load/setup. Native paths served 631 bytes, 629 direct and
2 fallback, plus 5 records4 bytes; these are not a distinct union or all compiled
instructions. All 1016 bus B, 6 classes, 127 slots and 127 boundaries passed, as did
failed-owner/full-state retention, Load, Undo with the same Arc, DMA3, CpuSet 510
halfwords and FastSet 256 words with full state-clock. All 261 guards, 568 accepted
hashes, three 568 diagnostic sets, 554 model preimages, 7 current hashes and 77 old/
78 new files matched; the helper retired only the interrupted copy. Accepted
executable f60055e6.. and the initial failure were retained. No game, checkpoint,
navigation, prior proof or performance measurement was repeated. Real CLI and
ordinary use were next; Linux-USA, native Windows, new 32:9, full ABI, indirect
calls, the 70B successor and the full TODO remained pending.
[Consumption and combined evidence](../rom_data_gameplay.md#managed-conditional-records-and-typed-runtime-reads).


**ROMDATA-04-CONDITIONAL-RECORDS8-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build, 72d44985.., used 568
sources byte-identical to runtime with no overlay; corelib and production
fingerprints were isolated and canonical hash/mtime restored. Three Wine CLI children
exited 0: create e85-d84, reuse and neutral startup 1. All 78 files matched runtime,
77 ID hashes loaded and 80 managed paths stayed unchanged; the old cache loaded
28859 blocks.

After the first prepare exited 0, a local helper checked index 75 instead of 76.
It was corrected and only the two remaining children resumed, without repeating
prepare or build. A normal before-activation copy covered 24755→24756 with neutral
input and all 9 counters at 0. No action, navigation, new checkpoint or saved endpoint
was produced, and neither positive ordinary use nor a full pair was inferred. All 189
guards and 568 CLI plus 568 runtime hashes matched; the helper found 0 paths and
originals were unchanged. Source, model, native, lifecycle and old scenes remained
closed. Positive ordinary use and a full comparison were next; Linux-USA, native
Windows, new 32:9, full ABI, indirect calls, the 70B successor and the full TODO
remained pending. No measurement or assistant change to Android, package, commit or
push occurred.
[CLI and limits](../rom_data_gameplay.md#actual-conditional-record-cli-registration).


**ROMDATA-04-05-CONDITIONAL-RECORDS8-PUB completed for one EUR/Wine ordinary
case (2026-10-29).** Normal Rumors entry in the pub produced a 42-frame suffix,
15181→15223, serving 468 bytes in 468 reads. Six classes accounted for
127/76/152/76/34/3 bytes across 127 physical positions, including 51 zero headers.
This is not 127 active records, all fields or branches, or a distinct-byte union.
All 9 counters matched across capture, CLI 72d44985 and full pair 63584945; each of
42 states and clocks, classic/wide pixels, OAM and 5 audio streams matched.

One warning-free Windows build used 569 sources: CLI 568 plus passive observer,
module and optional 5-line guard. The inverse hub/validator was byte-exact and core
and runtime unchanged. Two normal captures covered 254 frames: B for 180 frames was
negative, then Up 3/A 12 was positive over 74 frames. The first read was at relative
42, followed by 32 continuation frames; a rolling relative 32 checkpoint supplied the
42-frame suffix. The Missions LIST seed at 14969 was already retained; old 620/397
routes were not replayed. An old classic 240x160 PPM using color scale *255//31 was
authenticated against FFSTATE, not the recent wide format. Read-only projections and
images were verified without Wine UI, guest edits or cartridge writes; the final
prompt HeardlatestRumors did not establish a list, selection or outcome.

All 143 guards, 569 source hashes, 568 CLI hashes, 3 root hashes and 80 managed paths
matched; the 28859 cache remained exact and the helper found 0 paths. CLI e85
generation hw22uM was reused without prepare, build or navigation; the first B 0 case
remained closed. The declared source, model, runtime, CLI and ordinary 8B case was
closed. The next root, 713B3C with stride 70, requires its own contract. Linux-USA,
native Windows, new 32:9, full ABI, indirect calls and the full TODO remained pending;
no measurement or assistant change to Android, package, commit or push occurred.
[Equivalence](../rom_data_gameplay.md#ordinary-conditional-record-rumors-entry-equivalence).


**ROMDATA-01-MISSION-RECORDS70 EUR source contract completed
(2026-10-29).** The catalog at 713B3C..71C73C has 512 records of 70 bytes, totaling
35840B. Two independent scans cover indices 1..511; a getter reaches index 0 and the
successor boundary has its own readers. The separate constant 300 does not establish
a guest cap of 512. The getter has 65 selectors, narrowing index to low16 and selector
to low8. Its unsigned little-endian results include bytes, packed bits, flags, zero or
+0177 mapping and 200 scaling. The 7 address selectors expose 6 inline offsets, with
19/20 aliases and views 24/32 sharing scalar storage. Other bytes, raw values and
physical order are retained.

The closed mission link is reused: field +45 in HEX, or 69 decimal, selects event row
*12+71C760. The 512 references have 173 values and four alias groups; zero handling
depends on the caller and the distinct event catalog still needs classification.
All 35840B reconstructed with SHA256
bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc, disjoint from
77 owners and 50491 spans; e85-d84 and 78 files were unchanged. All 19 windows
(9 new, 10 cached), 157 pins and 110 guards matched. Old text was authenticated
opcode by opcode against ROM and its current hash, without claiming a historical text
guard. An aligned query found 38 base literals, 2 origin+70 literals and 2 internal
candidates in data without classified readers; it is not exhaustive for indirect calls.

This was static inspection only: no build, Wine, game, fixture, navigation or repeat
of prior proof. The helper retired zero paths. A strict lossless model and Wine getter
oracle were next, followed by managed/runtime, CLI and ordinary use. Full ABI, unknown
field semantics, view counts, successors, Linux-USA, native Windows, new 32:9 and the
full TODO remained pending. No performance or space measurement occurred; parallel
menu, content, DBG and package work was preserved.
[Contract](../rom_data_gameplay.md#european-mission-record-source-contract).


**ROMDATA-02-MISSION-RECORDS70 EUR/Wine model/oracle completed
(2026-10-29).** The lossless core mission_records70 model retains 512 records,
35840B and one span. Strict shape, order and JSON checks are separate from exact ROM
and 19-window authentication. Remaining bytes, inline arrays, raw operands and all
65 selectors are retained. Queries narrow index to low16 and selector to low8;
invalid selectors return Some 0 without lookup, address views are calculated even
outside the owner, and unowned scalar None is diagnostic, not a new guest cap.

One warning-free Windows core build, 663e8b87.., used 571 sources and 242 core inputs;
569 preimages and the inverse hub were byte-exact. Only two leaves and a public module
were added, without the parallel frontend. One Wine filter passed 21 guards and 698
calls of original getter D1F30 through return with the real CEEEC helper. The 438 base
cases covered all 65 selectors at indices 0/1/58/299/300/511, normalization, defaults,
views outside the owner and scalars 512-65535. The 260 isolated synthetic variants used
four 70B patterns at record 511. Independent Python/model results and actual LDRB read
order matched. The 324 distinct base bytes and 54 synthetic bytes are separate from
reconstruction of all 35840B; address views do not dereference the 16B inline area.

EWRAM, VRAM, OAM and palette were unchanged; IWRAM including expected return links,
SP, R4..R12, Thumb state and final ROM were exact. The 17213 instructions are coverage
only. All 139 guards, 571 hashes, 3 current and 569 preimages, and 110 source guards
matched; the helper found 0 paths. No game, checkpoint, cartridge write, navigation,
repeat of closed proof or measurement occurred. Schema 1, e85-d84, 77 resources,
78 files and save/native/CAPI layouts were unchanged. Managed/runtime, native clock
and lifecycle were next, then CLI and ordinary use; full ABI, unknown fields,
indirect calls, successors, Linux-USA, native Windows, new 32:9 and the full TODO
remained pending. Parallel work was preserved.
[Model and getter](../rom_data_gameplay.md#typed-mission-records-and-original-getter-proof).


**ROMDATA-03-04-MISSION-RECORDS70-RUNTIME completed EUR/Wine with combined
evidence (2026-10-29).** e86-d85 publishes mission-records70.json at index 77:
79 files and 78 owners, with 77 previous records exact and one 35840B span without
dependencies. The immutable owner has 10 classes, 13 counters, 8 mask words and 512
positions on pages 713..71c, composing with prior 8B and raw successors.

Two warning-free Windows builds used 573 sources and 243 core inputs. The first,
73326ad9, passed publication, prior 77 and interruption, then failed the first attach
at the historical OBJ cap 1024; other tests had not started. Following the request
for no space limits, attach caps OBJ 1024/BG 64KiB and three allocation assertions
were removed, while parse bounds, source authentication and binding remained unchanged.
The second build, 455bdb1c, ran only the remaining Wine filter on the same 79-file
generation: 14 attachment guards, 2 full-set guards, failed-owner/full-state retention
and 41 native state-clock cases (16 loads and 25 getters) with 35k raw bytes altered
after Load/setup. All 35k bus bytes, 10 classes, 512 slots and 512 edges passed, as
did Load, Undo with the same Arc and DMA3. Publication was not repeated and the
first failure was retained; the cap fix was not presented as an optimization or RSS
measurement.

All callbacks including interp_one were attributed: 52 new bytes and 5 previous bytes
served, with fallback delta 0, not a distinct union or full native catalog. The 3
stop-safe families, 168 entries and 69 dispatches are coverage only. DMA/CpuSet
17922 halfwords and FastSet 8968 words matched full state-clock. All 860 guards,
573 accepted and 573 diagnostic hashes, 571 model preimages, 8 current hashes and
78 old/79 new files matched; the helper retired only the interrupted copy.
close-final.py passed combined scopes; old close-runtime.py was not run. Source,
model and old scenes were not repeated, with no game, checkpoint, navigation, cartridge
write or Machine/save/native/CAPI ABI change. Real CLI and ordinary use were next;
Linux-USA, native Windows, new 32:9, full ABI, indirect calls, successors and the
full TODO remained pending. No performance or space measurement occurred; parallel
work was excluded from the build and preserved.
[Consumption and evidence](../rom_data_gameplay.md#managed-mission-records-and-typed-runtime-reads).


**ROMDATA-04-MISSION-RECORDS70-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build, 738b7f20.., used 573 sources
byte-identical to runtime with no overlay, consistent corelib/production/menu
fingerprints and restored canonical hash/mtime. Three Wine children exited 0: prepare
created the dataset, prepare reused it and neutral startup ran 1 frame. All 79 files
matched runtime, 78 ID hashes matched and 81 managed paths stayed unchanged; 28859 old
native blocks loaded.

A private generator stopped on a token-index assertion before Wine. Only the helper
generation was corrected with verified index 77; no source copy, build or CLI
preparation was repeated. A normal copy covered 24755→24756 with all 13 counters at
0. No action, navigation, new checkpoint or saved endpoint occurred, and neither a
positive ordinary full pair nor a new negative-locator phase was inferred. All 195
guards, 573 CLI and 573 runtime hashes, 79 files, originals, native cache, loader and
canonical executable matched; the helper found 0 paths. Source, model, native,
lifecycle and old proofs remained closed. Positive ordinary use and a full comparison
were next; Linux-USA, native Windows, new 32:9, full ABI, indirect calls, successors
and the full TODO remained pending, without measurement, Android/package change,
assistant commit or push.
[CLI and limits](../rom_data_gameplay.md#actual-mission-record-cli-registration).


**ROMDATA-04-05-MISSION-RECORDS70-MISSIONS completed for one EUR/Wine ordinary
case (2026-10-29).** Normal opening of Giza Plains information produced a 33-frame
suffix, 15017→15050: 5 reads serving 5 bytes, only from the packed class and one
physical position; the other 9 classes were 0. All 13 counters matched across
capture, CLI 738b7f20 and full pair 3d21e89c. Each of 33 full states and clocks,
classic/wide pixels, OAM and 5 audio streams matched. Both arms used the same old
native cache, not a global interpreter/native parity claim.

One warning-free Windows build used 574 sources: CLI 573 plus passive observer,
public module and optional 5-line guard. The inverse hub was byte-exact and core and
runtime unchanged. One capture covered 81 frames from the retained Missions LIST at
14969: Down 3 at 8, A 12 at 40, first read at relative 49/guest 15019, and rolling
48→suffix 33 with A 4 plus neutral 29; the search stopped. The Wine test child exited
0 and passed; a post-capture helper KeyError in steps was repaired only in the record,
without repeating capture, input, Wine or build. The authenticated classic seed at
240x160/scale was reused. A new wide PPM at 284x160/bitrep was verified against full
FFSTATE; the classic PNG showed the Giza MISSION information page 1/3 Rank 1. No
mission acceptance, battle, outcome, all fields, all 512 records or union of 5 distinct
bytes was inferred.

All 133 guards, 574 source hashes, 573 CLI hashes, 3 current hashes, 78 ID hashes and
81 managed paths matched; the 28859 cache was exact, the helper found 0 paths and
originals were unchanged. The old 620/397 route and source/model/runtime/registration
were not repeated; no ID, RAM or cartridge write was forced. The declared 70B chain
was closed; new C73C/C740 arrays and the C760 event need separate contracts. Linux-USA,
native Windows, new 32:9, full ABI, indirect calls, other ordinary classes and the
full TODO remained pending, with no performance or space measurement, Android/package
change, assistant commit or push.
[Equivalence](../rom_data_gameplay.md#ordinary-mission-information-equivalence).


**ROMDATA-01-SELECTION-BYTE-TABLES EUR source contract completed
(2026-10-29).** C73C..C740 contains four count thresholds and C740..C75E contains
thirty ordinal weights: 34 bytes in two spans with SHA256
d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd. All five live
headers at 02002E34+6*i are examined and counted only if their low 3 bits are
nonzero. A <=3 guard precedes the u8 threshold load; counts 4/5 exit. Two stride 1
loops use the independent second root for ordinals 1..30, positions 0..29.

Helper 377BC reads the raw live u8 at 02002CC3+12*(low8 ordinal-1) without a global
0/30 guard or Boolean normalization; its caller tests whether the low8 value is
nonzero. A sum of 0 exits before the second stage. That stage subtracts each eligible
unsigned weight from the full32 helper rank with wrap and a signed comparison: the
first negative result selects, while exhaustion at 31 exits. No negative guard,
zero-weight shortcut or cached eligibility is invented. Numeric helpers 2868/146314,
RNG distribution and full ABI are not modeled.

All 34B were reconstructed with positions retained, repeated values distinct and no
data pointers. They are disjoint from 78 owners and 50492 spans; e86-d85 and 79 files
were unchanged. All 6 windows (2 new, 4 cached), 95 pins and 96 guards matched; the
helper found 0 paths. Only the 2 count-context and live-helper windows were new. No
build, Wine, game, fixture, navigation, repeat of old proof or measurement occurred.
The two opaque B in gap C75E..C760 remain unowned; the next 12B root C760 is independent
and its extent pending. The neutral ID is gameplay.selection-byte-tables. Model and
Wine oracle were next, then managed/runtime, CLI and ordinary use. Linux-USA, native
Windows, new 32:9, indirect calls and the full TODO remained pending.
[Contract](../rom_data_gameplay.md#european-selection-byte-table-source-contract).


**ROMDATA-02-SELECTION-BYTE-TABLES EUR/Wine model/stages completed
(2026-10-29).** The strict model holds 4+30 u8 values, or 34B, in two spans. Shape is
separate from exact ROM and six-window authentication; the 2B gap is excluded and
repeated values remain distinct. One warning-free Windows core build, b40503b8.., used
576 files, 245 inputs and 574 preimages. One Wine filter passed 18 guards and ran 45
base plus 24 synthetic cases through D323C count, D327E sum-zero and D32B4 choice with
the real 377BC helper. Python, model, results, live-query order and LDRB reads matched;
wrap32 and signed rank including extremes were preserved without a weight 0 shortcut
or new cap.

All 34B of base and 32 synthetic bytes were observed, separate from reconstruction of
34 bytes. RAM, SP, R8..R12 and final ROM were exact; 30480 instructions are coverage
only. All 123 guards, 576 hashes, 3 current hashes, 574 preimages and 96 source guards
passed; the helper found 0 paths. Schema 1, e86-d85 and 79 files were unchanged.
Managed runtime, bus, native clock, Load, Undo, DMA, CLI and ordinary use were next;
numeric helpers, RNG, full ABI, Linux, USA, native Windows, new 32:9 and the global
TODO remained pending. No ordinary fixture, repeat of closed proof or metrics occurred.
[Model and proof](../rom_data_gameplay.md#typed-selection-byte-tables-and-original-stage-proof).


**ROMDATA-03-04-SELECTION-BYTE-TABLES-RUNTIME completed EUR/Wine
(2026-10-29).** e87-d86 publishes selection-byte-tables.json at index 78: 80 files and
79 resources. Its two spans contain 4/30B with no dependencies; all 78 previous
records and files, schema 1 and USA e8 were unchanged. The immutable page 71C owner
composes with the prior 70B owner and raw gap/event root. It has 6 counters and 4+30
distinct positions, with no allocation cap or ABI change.

One warning-free Windows build, e5b45668.., used 578 files, 246 core inputs and the
576-file model. One Wine filter passed publication/interruption, 14 attachment and
2 full-set guards, 20 native full state-clock cases, all 34 bus bytes, Load, Undo and
DMA3. The cases included 12 loads and 8 stages with real 377BC; raw 34 bytes and prior
4 bytes were altered only after Load. All 76 new B and 5 prior B served were attributed
to callbacks, with fallback source 0. The 91 entries and 119 dispatches are coverage
only. Width, mirror, edges, prior/raw composition, all 4+30 positions and DMA/CpuSet
20 halfwords/FastSet 16 words matched full state-clock. All 243 guards, 578 hashes,
576 model preimages, 7 current hashes and 79 prior records matched; the helper retired
only the successfully interrupted copy. Real CLI and ordinary use were next; numeric
helpers, RNG, full ABI, gap/event extent, Linux, USA, native Windows, new 32:9 and the
global TODO remained pending. No old proof or fixture was repeated and no metrics
were collected.
[Consumption](../rom_data_gameplay.md#managed-selection-byte-tables-and-typed-runtime-reads).


**ROMDATA-04-SELECTION-BYTE-TABLES-CLI EUR/Wine registration-startup completed
(2026-10-29).** One warning-free production build, 22af4132.., used the same 578
runtime sources without overlay; canonical hash and mtime were restored. One Wine
orchestration ran three commands for create, reuse and neutral startup 1, all exiting
0. The 80 dataset files were identical, 79 ID hashes matched, 82 paths kept hashes and
mtimes, and the 28859-block cache was unchanged. e87-d86/index 78, two spans totaling 34B and
no dependencies were verified. The 24755→24756 interval had all 6 usage counters at
0, so ordinary acceptance was not claimed.

All 192 guards and 578 CLI plus 578 runtime hashes matched, as did ROM, native cache,
TinyCC, original seed and canonical executable; the helper found 0 paths. No navigation,
new checkpoint, old proof or metrics occurred. Positive ordinary use and a full pair
were next; Linux-USA, native Windows, new 32:9, full ABI, RNG, indirect calls, gap/event
and the global TODO remained pending.
[CLI](../rom_data_gameplay.md#actual-selection-byte-cli-registration).


**ROMDATA-04-SELECTION-BYTE-TABLES-ORDINARY completed one EUR/Wine case
(2026-10-29).** Eight new windows and 354 direct opcode pins link D3724→D3228 to a
nonzero low8 argument guard. Wrapper D6970 passes 1; the other 2 callers pass 0 and skip,
while indirect callers remain unknown. Four read-only seeds had guard 6 and counts
1-2; normal Giza 19049 with cached UI was selected. One 579-source build, 8f3bee97,
added an observer for 6 statistics and a 5-line guard without core/runtime changes;
the inverse hub was exact.

One A 12 capture covered 129 frames, 19049→19178. The first read was at relative 97,
guest 19147; rolling frame 96 supplied a 33-frame suffix, 19145→19178. The same
22af CLI and e87 ZloxMx generation ran the positive 33 frames, and the same 8f3bee97
full pair passed all 33 complete states and clocks, classic/wide images, OAM and 5
audio streams. All 6 counts matched: 1 bus read, 1 threshold B served and 1 position,
with weights at 0. This does not establish all 4/30 positions or an event/mission
outcome; discovery stopped after one case. A full 284x160 bitrep wide image was
authenticated and the classic map calendar PNG inspected.

All 140 guards, 579 hashes, 578 CLI hashes, 3 current hashes, 82 managed paths and
the native cache, ROM, seed and canonical executable matched; the helper found 0 paths.
Dispatch 203258/519441 and audio [72418,72418,8712,8712,72342] describe coverage only;
both arms used the same cache, without a global interpreter/native parity claim. No
closed proof or fixture was recreated and no metrics were collected. Source, model,
runtime, CLI and one ordinary chain were closed; the event 12B root C760, alias 71D1EC
and next 71D360 need their own contract. Linux-USA, native Windows, new 32:9, full ABI,
RNG, indirect calls, gap/event and the global TODO remained pending.
[Ordinary use](../rom_data_gameplay.md#ordinary-selection-threshold-travel-equivalence).


**ROMDATA-01-EVENT-RECORDS12 EUR source contract completed (2026-10-29).**
The range 71C760..71D360 has 256 records of 12 bytes, or 3072B in one span, with
SHA e7454120... It is disjoint from 79 owners and 50494 spans. A backward scan from
255..1 at stride 12, a live u8 selector including zero and an independent next root
with stride 27 establish the physical extent, not a global cap inferred from fit.
The lossless layout has u8 header 00 and key 01, u16 primary 02 and alternate 04,
u8 parameter 06 and byte 07, u16 flags 08 and opaque 2B at 0A. All 3072 bytes
round-trip without invented enums, foreign-key caps or zero-padding rules.

Address 71D1EC equals base+225x12: it is an internal view with 31 physical positions
remaining and the same owner. Helper 9D04 narrows its argument to low8 but is not
capped at 31 and can address outside the owner; ordinal choices 1..30 select physical
rows 226..255. Callback 127388 compares a full u16 key to byte 01, taking the highest
matching row from 255..1 and excluding row zero. The 8 duplicate-key groups and 1
identical-record group are retained. When alternate 04 is nonzero, the helper uses
primary 02 from the alias and the original alternate 04; otherwise it uses direct
primary and alternate 0. The special key 4A path, parameter 06's next 27-byte root
and live effects remain original guest behavior.

All 15 windows (10 new, 5 cached), 67 pins, 112 guards and 80 accepted files passed;
the helper found 0 paths. The census found 36 base, 2 alias and 25 next-root literals,
not all indirect references. This was static inspection only, without a build, Wine,
game, fixture, navigation or repeat of old proof; schema 1 and e87-d86 were unchanged.
Model and original Wine prefixes were next, followed by managed/runtime, CLI and
ordinary use. Full ABI, live effects, indirect calls, the gap, next 27-byte extent,
Linux-USA, native Windows, new 32:9 and the global TODO remained pending, without metrics.
[Contract](../rom_data_gameplay.md#european-event-record-source-contract).


**ROMDATA-02-EVENT-RECORDS12 EUR/Wine model/prefixes completed (2026-10-30).**
The strict model retains 256x12 bytes, or 3072B in one span, including raw flags and
opaque 2B. Shape and order are separate from exact ROM and 15-window authentication.
Physical None outside the owner remains distinct from a u16 encoded view and uncapped
low8 alias. Direct/alias primary and original alternate pairs are retained, as is the
full16 script scan selecting the highest match in 255..1 and excluding row 0.

One warning-free core build, a4b88a69.., used 581 files, 248 core inputs and 579
preimages; its inverse hub was exact. One Wine filter passed 20 guards, 51 base and
37 synthetic prefixes for the 9D04 pair, 127388 scan, seven unsigned loads and u16
view. Python, model, results and LDR order matched. The 302 base B and 268 synthetic B
observed are separate from the 3072-byte round-trip. RAM was exact with the expected
index store and push; SP, R8..R12 and final ROM were verified. The 9959 instructions
are coverage only. All 133 guards, 581 hashes, 3 current hashes, 579 preimages and
112 source guards matched; the helper found 0 paths. Schema 1, e87-d86 and 80 files
were unchanged. Managed/native clock, lifecycle, CLI and ordinary use were next; full
ABI, live effects, indirect calls, the gap, next 27-byte extent, Linux-USA, native
Windows, new 32:9 and the global TODO remained pending. No old proof, ordinary fixture
or navigation was repeated, and no metrics were collected.
[Model](../rom_data_gameplay.md#typed-event-records-and-original-prefix-proof).


**ROMDATA-03-04-EVENT-RECORDS12-RUNTIME completed EUR/Wine (2026-10-30).**
e88-d87 publishes event-records12.json at index 79: 81 files and 80 resources, with
79 prior records exact. Its one 3072B span has no dependencies, and alias row 225
uses the same owner. Pages 71c-71d contain 8 classes, 11 statistics, 4 mask words and
256 positions. The prior selection 2B, raw gap 2 and next root compose at the edges;
no allocation cap or guest ABI change was introduced.

One warning-free build, 9225a3bc.., used 583 sources, 249 core inputs and the 581-file
model. One Wine filter passed publication/interruption, 14 attachment and 2 full-set
guards, 22 native full state-clock cases, all 3072 bus bytes, 8 classes, 256 records,
256 boundaries, Load, Undo and DMA3. The cases included 14 loads and 8 pair/scan/u16
view cases in 4 stop-safe families, with raw 3072 bytes and prior 2 bytes altered
after Load. All 296 new B and 3 prior B served were attributed; 2 new B came from
interp_one fallback, so 296 is neither direct native nor a distinct union. The 51
entries and 30 dispatches are coverage only. DMA/CpuSet 1538 halfwords and FastSet
776 words matched full state-clock.

All 253 guards, 583 hashes, 581 model preimages, seven current hashes and 80 prior
records matched; the helper retired only the successfully interrupted copy. Actual
CLI and ordinary use were next; full ABI, live effects, indirect calls, the gap, next
27-byte family, Linux-USA, native Windows, new 32:9 and the full TODO remained pending.
No old proof or fixture was repeated and no metrics were collected.
[Consumption](../rom_data_gameplay.md#managed-event-records-and-typed-runtime-reads).


**ROMDATA-04-EVENT-RECORDS12-CLI EUR/Wine registration-startup completed
(2026-10-30).** One warning-free production build, b9a60501.., used the same 583
runtime sources with no overlays; canonical hash and mtime were restored. One Wine
orchestration ran create, reuse and neutral startup 1, with all three exiting 0. All
81 payloads were identical, 80 ID hashes matched, 83 paths remained unchanged and the
28859-block cache was exact. e88-d87/index 79 has one 3072-byte span and no
dependencies. All 11 counts were 0 from 24755→24756, proving compatibility only, not
positive ordinary use.

All 194 guards, 583 CLI and 583 runtime hashes, 81 accepted files, ROM, native cache,
TinyCC, seed and canonical executable matched; the helper found 0 paths. Positive
ordinary use and a full pair were next. Linux-USA, native Windows, new 32:9, full ABI,
live effects, indirect calls, the gap, next 27-byte family and the full TODO remained
pending. No source/model/native or old proof, fixture or navigation was replayed, and
no metrics were collected.
[CLI](../rom_data_gameplay.md#actual-event-record-cli-registration).


**ROMDATA-04-EVENT-RECORDS12-ORDINARY completed one EUR/Wine case (2026-10-30).**
The normal Giza White Flowers seed at 19409, cached UI and producer witness were reused,
not the old 700-frame route proof. One 584-source build, 0b6581cb, added an observer
for 11 statistics and a 5-line guard; the inverse hub/validator was exact and core and
runtime unchanged. One A 12 capture covered 34 frames, 19409→19443, with its first
read at relative 2/guest 19412 and a rolling 0 checkpoint for the 34-frame suffix;
discovery stopped. The same b9a605 CLI and e88 generation 3iczRx ran for 34 frames;
the child exited 0. A collector TypeError for an old usage ID was fixed offline in
log registration only, without a CLI replay.

The same 0b6581cb full pair passed all 34 complete states and clocks, classic/wide
images, OAM and 5 audio streams. All 11 counts matched: 4 bus reads serving 6 B,
2 key, 2 primary and 2 alternate bytes, across one physical record; the other 5
classes were 0. This is not a distinct-byte union or coverage of all 8 classes,
all 256 records, the alias branch or the outcome. A full 284x160 bitrep wide image
was authenticated and the classic Giza map mission entry inspected. All 126 guards,
584 source hashes, 583 CLI hashes, 3 current hashes, 83 unchanged paths, 80 ID hashes,
the 28859 cache, ROM, seed and canonical executable matched; the helper found 0 paths.
Dispatch 128240/218562 and audio [74612,74612,8976,8976,74488] are coverage only.

The event source, model, runtime, CLI and one ordinary chain were closed. The next
27-byte root 71D360 still needs an extent/field contract; Linux-USA, native Windows,
new 32:9, full ABI, live effects, indirect calls, the gap and the full TODO remained
open, without metrics.
[Ordinary use](../rom_data_gameplay.md#ordinary-event-record-mission-entry-equivalence).


**ROMDATA-01-PARAMETER-RECORDS27 completed for observed EUR fragments
(2026-10-30); total extent still open.** The 256 event records establish 59 indices,
0..50 and 115..122, at stride 27 from root 71D360. They form two ranges,
71D360..71D8C1 and 71DF81..71E059, totaling 1593B, with round-trip SHA 40d4e406...
All 256 links resolve without overlap with 80 owners and 50495 spans in e88-d87.
All 27 windows (23 new and 4 reused), 70 pins and 125 guards passed; the cached
census covers all 25 root literals.

Shared packed fields 01..06, u16 LE fields 07-0A-12, explicit i8 fields 0C-15,
byte 09, flags 0D, separate operands 0E0F-1011 and opaque bytes 14+16..1A remain
intact. No enum, pointer or padding is inferred from values. Live +A8 readers use u8
without a 127/128/59 limit; the next root 71E0E0 at stride 38 does not prove 128
records by distance alone. The 128-record hypothesis was rejected; gaps and further
indices remain raw and unowned. This was static inspection only, without build, Wine,
input, fixture, repeat of old proof or metrics. Wine model/proof, runtime, CLI and
ordinary use were next; global extent, full ABI, live effects, indirect calls,
Linux-USA, native Windows, new 32:9 and the full TODO remained open. The event 12
chain stayed closed.
[Contract](../rom_data_gameplay.md#observed-european-parameter-records-with-27-byte-stride).


**ROMDATA-02-PARAMETER-RECORDS27 completed for 59 observed EUR records
(2026-10-30).** The strict model retains physical indices 0..50/115..122, u8 and
packed fields, u16 LE, i8 and opaque data in two spans totaling 1593B. Validation of
modified values is separate from source authentication. record/byte_at do not answer
for gaps or high indices; encoded_view retains the original low8 computation without
a 59/128 limit.

One warning-free 586-source, 251-core build, 49c72f02, and one Wine filter passed
25 guards, 73 base cases and 48 synthetic cases across 12 original-prefix classes.
Results and read order matched, including signed 127/-128 and low16 encoding. RAM,
SP, R8..R12 and final ROM were exact. The 95 distinct base B and 20 synthetic B
observed are separate from the 1593B round-trip; 739 instructions are coverage only.
All 143 guards, 584 preimages and 125 source guards matched, the inverse hub was
retained and the helper found 0 paths. No old proof or fixture was repeated and no
metrics were collected. Managed runtime e89-d88, CLI and ordinary use were next;
global extent, full ABI, helper/live effects, indirect calls, Linux-USA, native
Windows, new 32:9 and the full TODO remained open. e88-d87 was unchanged.
[Model](../rom_data_gameplay.md#sparse-parameter-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-PARAMETER-RECORDS27-RUNTIME EUR/Wine completed for 59 observed
records (2026-10-30).** e89-d88 publishes parameter-records27.json at index 80:
82 files and 81 resources, with 80 previous records exact. It has no dependencies
and two spans totaling 1593B. The sparse owner on 71D..71E composes with prior event
records and raw gaps, with 12 classes, 15 statistics and four physical masks. No new
cap, ABI, save or native-cache change was introduced.

The first 588-source build, 51e19ef0, passed publication, interruption, 14 attachment
and 2 full-set guards, and 17 native full state-clock cases. Its test oracle expected
2 prior-owner bytes instead of the 4 aligned bytes actually served; typed, native and
API counts agreed on 4. Only the test leaf was corrected. A second warning-free
588-source build, 278d6d23, ran only the remaining 16 native cases plus bus/lifecycle
and DMA3 checks in one Wine run. The same 82 files and 13 objects retained hashes
and mtimes; publication, attachment and the first 17 cases were not repeated.

The combined 33 full state-clock cases covered 20 loads and 13 prefixes in 13 families,
serving 37 new B and 5 event B. These are not a distinct-byte union or proof that all
reads were direct native: fallback was 0 only for the remaining 16 cases and was not
reported for the previous 17. Checks passed for all 1593B, 12 classes, 59 records,
59 borders, raw gaps and the next root, Load, Undo, DMA/CpuSet 1664 halfwords and
FastSet 832 words with exact state-clock. All 291 guards, 588 accepted and 588
diagnostic hashes, 252 core inputs, 586 model preimages, 7 current hashes and 81 prior
files were retained; the helper retired only the interrupted copy. A collector task
label was corrected offline without replaying build or test. accepted-selection.json
is authoritative; selection.json is diagnostic. Actual CLI and ordinary use, global
extent, full ABI, helper/live effects, indirect calls, Linux-USA, native Windows, new
32:9 and the full TODO remained open.
[Runtime](../rom_data_gameplay.md#managed-sparse-parameter-records-and-typed-runtime-reads).


**ROMDATA-04-PARAMETER-RECORDS27-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One warning-free 588-source production build, bd99ac5b, used the same
corrected runtime sources without overlays; canonical hash and mtime were restored in
a finally block. One Wine orchestration ran prepare to create e89-d88, prepare to
reuse it, and runc on a normal checkpoint copy with neutral input for 1 frame,
24755→24756. All three exited 0. The 82 payloads matched runtime, 81 ID hashes
matched and 84 managed paths kept hashes and mtimes; the native cache loaded 28859
blocks. Index 80, two spans totaling 1593B and no dependencies were verified.

All 15 counters were 0. This establishes registration and compatibility, not positive
ordinary use or equivalence. All 197 guards and 588 CLI plus 588 runtime hashes matched,
as did ROM, seed, cache, TinyCC and canonical executable; the helper found 0 paths. No
new navigation, checkpoint, repeat of old tests or metrics occurred. Positive ordinary
use and a full pair, global extent, full ABI, helper/live effects, indirect calls,
Linux-USA, native Windows, new 32:9 and the full TODO remained open.
[CLI](../rom_data_gameplay.md#actual-sparse-parameter-record-cli-registration).


**ROMDATA-04-PARAMETER-RECORDS27-ORDINARY partial (2026-10-30), positive still to
obtain.** A passive observer reports 15 statistics, with a module and optional 5-line
guard. The warning-free 589-source build, 8f0c02b7, retained the 588-source CLI core
and runtime unchanged; inverse preimages were exact. Two normal Wine captures on
copies had all 15 counts at 0: R 12 from GizaInfo ran 15050→15230 for 180 frames and
left page 1/3 unchanged; A 12 at relative 1/161 in White Flowers dialogue ran
21889→22129 for 240 frames. These negative bounds are closed; no positive suffix,
positive CLI or full pair was executed.

The 9 static windows and 7 read-only snapshot gates support a join of the accepted
script directory, archive and events. It identifies opcode 80, mode 2 in programs
53/65/74/93/95, keys 56/68/77/96/98, events 216..220 and parameters 116..120.
White Flowers uses event 58, parameter 0 and program 163. Further blind attempts on
that page or intro were stopped; indirect calls and mode 0 reads when the override is
0 are not excluded. All 130 guards, 589 and 588 hashes, 3 current hashes, 84 managed
paths and originals were retained; diagnostics were kept without full cleanup. The
same observer, bd99 CLI and e89 dataset can be reused when a relevant scene is known.
Positive/full equivalence, global extent, Linux-USA, native Windows, new 32:9 and the
full TODO remained open, without metrics.

An update to the same unit brought the total to eleven normal captures, all with 15
statistics at 0 and no rebuild. New contexts included SprohmInfo 180, B 32, Nubswood
cursor/travel 360 and CyrilPub 240+120; no mission ID was inferred from its name and
no RNG resampling was attempted. Six authenticated snapshot gates did not prove
liveness. One neutral 1-frame Wine inspection with the already compiled CLI and layout
off exited 0 without saving or adopting state; read-only framebuffer/UI projections
of 76800 pixels showed only Gil, with no readable choice. All 233 guards, 589 and 588
hashes, 3 current hashes and 84 managed paths matched; first receipts were retained.
Blind navigation stopped, and positive ordinary use remains open. Independent source
contracts may continue without repeating these captures.
[Status and evidence](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).


**ROMDATA-01-WORLD-RECORDS38 completed for the observed EUR catalog
(2026-10-30).** Root 71E0E0 covers 31 records of 38 bytes, or 1178B, through 71E57A,
with SHA cd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483.
All 512 missions project indices 0..30. The interior root 71E106 and loop 6..29 use
the same owner for records 7..30; next root 71E57C reads u32 pointers across an unowned
2B gap. Neither distance nor the physical catalog introduces a cap of 31: D347C uses
low8 and D4C4C retains full R2 u32 addresses.

Byte 01, u16 LE 02 with separate byte 02, u16 LE 04 with its low10 copy, 31 thresholds
at 07 with count clamped 0..30, and opaque bytes 00 and 06 are retained. All 12 windows
(10 new, 2 cached), 58 pins and 107 guards passed. The 1178-byte round-trip and 512
links showed no overlap with 81 owners or 50497 spans; e89-d88 and 82 files remained
exact. This was static only, with no build, Wine, input, fixture, old evidence replay or
metrics. Wine model/oracle, runtime, CLI and ordinary use were next; helper/RNG, full
ABI, live effects, indirect calls, opaque meaning, Linux-USA, native Windows, new
32:9 and the full TODO remained open. Sparse 27 ordinary use stayed partial and
navigation was not reopened.
[Contract](../rom_data_gameplay.md#observed-european-world-records-with-38-byte-stride).


**ROMDATA-02-WORLD-RECORDS38 completed EUR/Wine (2026-10-30).** The strict model
retains 31 records, 1178B and one span, including opaque bytes and all bits. Editable
shape is separate from exact EUR authentication and 12 window hashes. Physical
record/byte_at ownership excludes the 2B gap and indices 31+, while original encoded
low8, full u32 and interior row 1 views retain their addresses without a 31 cap.

The projections retain the byte 01 divisor and post-helper wrap/zero-to-1 behavior,
byte 02 separately and as u16 LE 02, full u16/low10 value 04, live count clamp 0..30
and signed threshold comparison including raw 255. One warning-free 590-source,
254-core build, ed4d65c8, and one Wine filter passed 19 guards, 41 base and 19
synthetic original prefixes. The 23 distinct base B and 6 synthetic B observed are
separate from the 1178-byte round-trip; 353 instructions are coverage only. Read
order, results, RAM with expected destinations, SP, R8..R12 and final ROM were exact.
All 26 unit guards, 107 source guards, 588 preimages, 3 current hashes and inverse hub
matched; the helper found 0 paths. No external helper/RNG, full ABI, native clock or
ordinary claim was made, and no old proof, navigation, fixture or metric was repeated.
Schema 1, e89-d88 and 82 files were unchanged. Managed runtime e90-d89, CLI, ordinary
use, Linux-USA, native Windows, new 32:9 and the full TODO remained to do.
[Model](../rom_data_gameplay.md#world-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-WORLD-RECORDS38-RUNTIME completed EUR/Wine (2026-10-30).**
e90-d89 publishes world-records38.json at index 81: 83 files and 82 resources, with
81 prior records exact, no dependencies and one 1178B span. The immutable owner on
71E composes with sparse 27 records, the raw gap and the next root; it has 5 classes,
8 statistics and one 31-position mask, without a new cap or ABI change.

One warning-free 592-source build, 271dceba, was used after a collector guard label was
corrected offline on the same executable, without rebuilding. One Wine filter passed
publication/interruption, 14 attachment and 2 full-set guards, and 27 native full
state-clock cases: 17 loads and 10 prefixes in 10 stop-safe families. They served
23 new B and 5 prior B with fallback [0, 0]; this is not a distinct-byte union or an
all-direct-native claim. All 1178B, 5 classes, 31 records and 31 borders passed, as
did mirrors, widths, prior/raw gap/next-root composition, Load, slot Undo and
DMA/CpuSet 704 halfwords/FastSet 352 words with full state-clock.

All 269 guards, 592 frozen sources, 255 core inputs, 590 model preimages, 7 current
hashes and 82 prior records were retained. The current 27 observer was excluded only
from the frozen hub; the helper retired only the interrupted copy. Actual CLI and
ordinary use, full ABI, helper/RNG, live effects, opaque/indirect semantics, Linux-USA,
native Windows, new 32:9 and the full TODO remained open. No old proof, navigation,
fixture or metrics were repeated.
[Runtime](../rom_data_gameplay.md#managed-world-records-and-typed-runtime-reads).


**ROMDATA-04-WORLD-RECORDS38-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One warning-free 592-source production build, 6d95f705, used the same
runtime sources without overlays; canonical hash and mtime were restored. One Wine
orchestration had three exits of 0: prepare created e90-d89, prepare reused it, and
runc used a copied normal checkpoint with neutral input for 1 frame, 24755→24756.
All 83 files matched runtime, 82 ID hashes matched and 85 managed paths retained hash
and mtime; the 28859 cache was exact. Index 81 has no dependencies and one 1178B span.
All eight counts were 0, establishing registration and compatibility only, not
positive ordinary use or equivalence.

All 201 guards and 592 CLI plus 592 runtime hashes matched, as did ROM, seed, cache,
TinyCC and canonical executable; the helper found 0 paths. No new navigation, fixture,
repeat of old proof or metric occurred. Positive ordinary use and a full pair, full
ABI, helper/RNG, live effects, opaque/indirect semantics, Linux-USA, native Windows,
new 32:9 and the full TODO remained open.
[CLI](../rom_data_gameplay.md#actual-world-record-cli-registration).


**ROMDATA-04-WORLD-RECORDS38-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One warning-free 593-source build, ee939288, added a passive observer
for 8 statistics and a 5-line guard. Core/runtime matched the 592-source CLI, inverse
hubs were exact and the current 27 observer was retained. The first normal A 12
Nubswood capture, 3448→3808 over 360 frames, was a closed negative. Read-only CCF00,
path 0..6 and a counter changing 32→30 explained that the branch was not ready.

Among four authenticated EUR banks, IN08 had enabled physical row 13 with counter 1.
A normal Down 48 capture of 96 frames, 3020→3116, ended at actual node 18. One A 12
capture then ran 189 frames, 3116→3305; the first source read was at relative 157,
guest 3274, and a rolling checkpoint supplied a 45-frame suffix, 3260→3305. Discovery stopped. The
same 6d95 CLI ran one 45-frame suffix, and the same 593-source test ran one full pair
of 45 frames. Every state and clock, classic/wide image, OAM and 5 audio streams
matched. All 8 statistics matched: 2 bus reads serving 2 B, 1 threshold and 1
byte 01 across one physical record, with the other 3 classes at 0. This does not prove
all 5 classes or 31 records were used.

All 82 ID hashes and 85 managed paths were unchanged, and the 28859 cache was reused.
An authenticated framebuffer projection showed world travel on Madmoon Day4 without
inferring mission or arrival. All 165 guards, 593 and 592 source hashes, 3 current
hashes, inverse and originals matched; the helper found 0 paths. Normal inputs used
copies only, without reset, RNG, RAM or ID edits, route recreation, old proof or
metrics. Other ordinary coverage, full ABI, helper/RNG, live effects, opaque/indirect
semantics, Linux-USA, native Windows, new 32:9 and the full TODO remained open; sparse
27 ordinary use stayed partial.
[Case and limits](../rom_data_gameplay.md#ordinary-world-record-counter-expiration-equivalence).


**ROMDATA-01-INDEXED-GRAPHICS-ARCHIVES completed for the EUR source contract
(2026-10-30).** Four pointer groups at 71E57C/71E61C/71E630/71E658 have five
language slots each: 20 archives and 649 programs. Their 24 disjoint spans contain
74907B of source, comprising 74827 archive bytes and 80 pointer bytes. Reconstructed
output totals 278336B, with 15631B of dictionary prefixes. Codec 05228/0537C uses a
big-endian index/offset and fixed external dictionary, distinct from 054A0. Opaque
4137 is not a format discriminator; language remains raw u8 without fallback and
start/count retain their original low16 behavior.

All source and output bytes were reconstructed exactly. All 22 windows (20 new,
2 cached), 101 pins, 10 root literals, 12 merged BL calls and 121 guards passed;
the source did not overlap 82 owners or 50498 spans, and 83 accepted files remained
exact. The other 16 nearby groups, code/dictionary tails, context 3710D4 and global
extent were not classified. No build, Wine, game, fixture, repeat of old proof or
metric occurred; schema 1 and e90-d89 were unchanged. Model, Wine oracle, runtime,
CLI, ordinary use, full ABI, indirect calls, Linux-USA, native Windows, new 32:9 and
the full TODO remained open; sparse 27 ordinary use stayed partial.
[Contract and limits](../rom_data_indexed_graphics.md).


**ROMDATA-02-INDEXED-GRAPHICS-ARCHIVES completed EUR/Wine (2026-10-30).** The
strict model retains 20 archives in four five-slot groups and 649 programs: opaque
header, big-endian index and offsets, literal/dictionary packets, dictionary prefixes
and little-endian words. Editable shape is separate from exact EUR authentication.
The host model covers mode 0, width 5 only, without adding a guest cap. Raw language
encoded views remain original for 5/255; low16 start/count, zero/oversize count clamp
and invalid start before initialization are retained.

One warning-free 595-source, 257-core build, d78a74d9, used 593 ordinary sources,
2 new leaves and one public module. One Wine filter passed 37 guards, 20 whole-archive
calls, 9 boundary cases, 9 synthetic cases and 28 raw selector prefixes. All 74907B
in 24 spans and 278336 decoded B were reconstructed. Output, status, globals,
SP-R4..R11, RAM outside the original 68B stack, VRAM, OAM, palette and final ROM were
exact. The 5993129 instructions are coverage only. All 26 unit guards, 121 source
guards, 593 preimages, 3 current hashes and the inverse hub matched. No closed proof,
checkpoint, navigation or performance replay occurred; schema 1 and e90-d89 were
unchanged. Managed runtime, native clock, CLI, ordinary use, mode 3 and wider widths,
full ABI/live uploads, indirect calls, Linux-USA, native Windows, new 32:9 and the
full TODO remained open.
[Model and proof](../rom_data_indexed_graphics.md#typed-model-and-original-wrapper-proof).


**ROMDATA-03-04-INDEXED-GRAPHICS-RUNTIME completed EUR/Wine (2026-10-30).**
e91-d90 publishes graphics.indexed-archives/indexed-graphics.json at index 82:
84 files and 83 resources, with all 82 prior records and files exact. The 24 spans
contain 74907B and have no dependencies. The immutable owner reconstructs header,
index, packets and dictionary, with 7 classes, 12 statistics, 20 banks, 649 programs
and 4 group masks. It adds no decoder/upload shortcut, ABI change or guest cap.

The first 597-source build failed only because a test lacked the Bus import; its
diagnostics were retained. The corrected 597-source, 258-core build, 0732b3b9, was
warning-free and production code unchanged. One Wine filter passed publication and
interruption, 14 attachment and 2 full-set guards, and 35 complete state-clock cases:
28 loads, one original 1-program wrapper and 6 raw selectors in 3 stop-safe families.
They served 272 new B and 4 prior world B with fallback [1,0]; these are neither a
distinct-byte union nor all direct native. The 232 instructions and 59 dispatches
are coverage only.

All 74907 bus B, 7 classes, 20 archives, 649 programs, 4 groups, all 24 borders,
mirrors, widths, prior world/raw gap composition, Load, slot Undo and mixed 64B
DMA/CpuSet/FastSet checks matched full state-clock. All 282 guards, 597 accepted and
597 diagnostic hashes, 595 model preimages, 7 current hashes and 83 prior files were
retained. Accepted-selection.json is authoritative; selection.json records the first
failure. Actual CLI and ordinary use, full ABI/live uploads, mode 3 and wider widths,
global extent, indirect calls, Linux-USA, native Windows, new 32:9 and the full TODO
remained open. No old proof, fixture, navigation or metric was repeated.
[Runtime and evidence](../rom_data_indexed_graphics.md#managed-source-views).


**ROMDATA-04-INDEXED-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One warning-free 597-source production build, d8767645, used the same
runtime sources without overlays; canonical hash and mtime were restored in a finally
block. One Wine orchestration ran three commands, all exiting 0: prepare created
e91-d90, prepare reused it and runc used a copied normal checkpoint for 1 neutral
frame, 24755→24756. All 84 files matched runtime, 83 ID hashes matched and 86 managed
paths retained hashes and mtimes; the 28859 cache was exact. Index 82 has no
dependencies and 24 spans totaling 74907B.

All 12 counts were 0. This establishes registration and compatibility only, not
positive ordinary use or a full pair. All 201 guards, 597 CLI and 597 runtime hashes,
84 accepted files, ROM, seed, cache, TinyCC and canonical executable matched. No new
navigation, fixture, repeat of old proof or metric occurred. Ordinary use, full ABI/
live uploads, mode 3 and wider widths, global extent, indirect calls, Linux-USA, native
Windows, new 32:9 and the full TODO remained open.
[CLI and limits](../rom_data_indexed_graphics.md#actual-cli-registration).


**ROMDATA-04-INDEXED-GRAPHICS-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One warning-free 598-source build, 6d40a6a5, added a passive observer
for 12 statistics and a 5-line guard. The 597-source CLI core/runtime were unchanged,
inverse hubs were exact and current parameter/world observers retained. The 8 windows
(6 new, 2 cached) link 453E2→1967C, context 3, to program 2 MENU. One normal Start 12
capture covered 41 frames, 4519→4560; the first source read was at relative 9/guest
4529 and the rolling checkpoint supplied a 41-frame suffix. Discovery stopped.

The same d876 CLI ran the 41-frame suffix with all 83 ID hashes and 86 managed paths
unchanged and cache 28859 exact. The same test executable ran one full pair: all 41
states and clocks, classic/wide images, OAM and 5 audio streams matched. All 12
statistics agreed: 274 reads serving 277 B, comprising selector 4, header 6, index 4,
token 67, literal 17, dictionary header 4 and dictionary 175 bytes. One archive,
program and group were used and all 7 classes were positive; this does not cover all
20 archives, 649 programs or 4 groups. All 144 guards, 598 frozen and 597 CLI hashes,
3 current hashes, inverse and originals matched. A read-only 284x160 full-wide
projection and classic PNG showed MENU Party-Clan-AreaList-System/MoonDay9, without
inferring a choice or mission outcome.

Only normal input was used, with no forced RAM, ID, RNG or ROM, and no repeat of old
320 Down-A, world 38 daily 45, negative or other proofs, or metrics. Other ordinary
coverage, full ABI, mode 3 and wider widths, global extent, indirect calls, Linux-USA,
native Windows, new 32:9 and the full TODO remained open; sparse 27 ordinary use stayed
partial.
[Case and limits](../rom_data_indexed_graphics.md#ordinary-world-map-menu-equivalence).


**ROMDATA-01-LOCALIZED-TABLE-GRAPHICS EUR source contract completed
(2026-10-30).** Eight independent groups at 71E590/5A4/5B8/5CC/644/680/6BC/6D0
select a LE32 pointer using raw u8 language at 02001F6E×4 and call 054A0. They cover
40 new custom LZ-RLE, nonindexed banks. The 48 spans contain 53667 source B, comprising
53507 bank and 160 selector bytes, and decode to 114880 B. All tokens, source and
output were reconstructed exactly.

All 14 windows (10 new, 4 cached), 48 pins, 8 root literals and 111 guards passed;
the spans did not overlap 83 owners or 50522 spans, and all 84 accepted files remained
exact. Each group's five banks produce 1472/2688/768/4928/1920/7488/1344/2368B,
respectively. No count-by-distance, guest cap, language fallback or skipped load is
invented. Live
RAM, helper transfer and later work remain original; full transfer ABI was not closed.
The known 71E5E0 old UI banks/table alias is separate, while seven other raw-looking
groups, tails, indirect calls and global extent remain unclassified. This was static
inspection only, without build, Wine, game, fixture, navigation, old proof or metrics;
schema 1 and e91-d90 were unchanged. Model, Wine oracle, runtime, CLI, ordinary use,
Linux-USA, native Windows, new 32:9 and the full TODO remained open.
[Contract and limits](../rom_data_ui_graphics.md#additional-european-table-selected-source-contract).


**ROMDATA-02-LOCALIZED-TABLE-GRAPHICS completed EUR/Wine (2026-10-30).** The new
core ui_graphics::table_selected model reuses Graphic/custom LZ. It strictly retains
8 tables × 5 contiguous BankPointers, 40 ordered banks, raw encoded bytes, LE words
and group outputs. Editable shape is separate from exact EUR authentication and 14
fingerprints. Physical select returns None for 5/255, while the original raw encoded
view remains available. The separate catalog was not yet enabled in the managed UI
resource: the old 16 banks, two inline switches and e91 were unchanged.

One warning-free 600-source, 260-core build, cb46ccb1, used 598 ordinary sources,
two leaves and one public module. One Wine run passed 23 guards, 40 new original
054A0 calls and 56 raw selector prefixes. All 53667 source B in 48 spans, 114880
decoded B, cursor, produced/returned size, SP-R4..R11, full EWRAM, IWRAM outside the
original 40B stack, VRAM, OAM, palette and final ROM matched. The 1335470 instructions
are coverage only. All 26 unit and 111 source guards, 600 frozen sources, 598 preimages,
3 current hashes and inverse hub matched. Old 16 UI, 20 indexed, native 35, 41 ordinary
and CLI proofs were not replayed; no fixture, navigation or metric was added.

An offline control label at 23B2E was corrected to 0150 words/1344B, not the group
set 02A0 halfword label; originals and inputs were unchanged and no proof was rerun.
Managed runtime, native clock, CLI, ordinary use, full helper ABI/live contexts, the
71E5E0 alias, seven raw groups, global extent, indirect calls, Linux-USA, native
Windows, new 32:9 and the full TODO remained open.
[Model and evidence](../rom_data_ui_graphics.md#typed-table-model-and-new-original-decoder-proof).


**ROMDATA-03-04-LOCALIZED-TABLE-GRAPHICS-RUNTIME completed EUR/Wine
(2026-10-30).** e92-d91 keeps 84 files and 83 IDs; the UI resource at index 15 contains
56 banks, 10 selectors and 106 bindings. All other 82 records/files, the old 16 banks,
two inline selectors and shared OBJ links remained exact. The 40 banks and 40
pointers add 80 runtime bindings whose byte union equals 48 canonical spans and 53667B.
The 6 new table counters are separate from the old 6 UI statistics. Original decode,
scratch, helper, upload work and ABI remain in use.

One warning-free 601-source, 260-core build, d4a73eb2, used the 600-source model plus
one host leaf. One Wine run passed changed-UI publication/interruption, 14 attachment
and 2 full-set guards, and 22 complete state-clock cases: 12 loads, 1 new-bank decoder
and 9 raw selectors in 3 stop-safe families. They served 499 new B and 5 prior indexed
B with fallback [3,0]; this is neither a distinct-byte union nor all direct native.
The 356 instructions and 71 dispatches are coverage only. All 53667 bus B, 24328
literal B, 160 selector B, 40 banks, 8 groups, all 48 edges, widths, mirrors, prior
indexed/raw/old UI composition, Load, slot Undo and mixed DMA3 full state-clock passed.
All 267 guards, 601 frozen sources, 600 model preimages, seven current hashes and 84
prior files matched; the helper retired only the interrupted copy.

No old 40-model, 16-UI, 20-indexed, native 35, 41-ordinary or CLI preparation proof
was replayed; no fixture, navigation or metric was added. Actual CLI and ordinary use,
full helper ABI/live contexts, 71E5E0 alias, seven raw groups, global extent, indirect
calls, tails, Linux-USA, native Windows, new 32:9 and the full TODO remained open.
[Runtime and evidence](../rom_data_ui_graphics.md#managed-table-selected-graphics-and-runtime-consumption).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One warning-free production build, 31c0f298, used 601 sources identical
to runtime without overlays; canonical hash and mtime were restored in a finally block.
One Wine orchestration ran 3 commands exiting 0: create, reuse and neutral startup
for 1 frame from normal copy 24755→24756. All 84 files matched runtime, 83 ID hashes
matched and 86 managed paths were unchanged; cache 28859 was exact. e92-d91 UI index
15 has 56 banks, 10 selectors and 106 bindings, retaining 84 files and 83 IDs.

All six new table counters were 0. This establishes registration and compatibility
only, not positive ordinary use or a full pair. All 202 guards, 601 CLI and 601 runtime
hashes, 84 accepted files, ROM, seed, cache, TinyCC and canonical executable matched.
No old 40-model, 16-UI, indexed 20, native 22-35, 41-ordinary or CLI preparation
proof was replayed, and no new checkpoint, navigation or metric was added. Positive
ordinary use, full helper ABI/live contexts, global extent, indirect calls, 71E5E0
alias, seven raw groups, tails, Linux-USA, native Windows, new 32:9 and the full TODO
remained open.
[CLI and limits](../rom_data_ui_graphics.md#actual-table-extension-cli-registration).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY partial; two negative bounds
closed EUR/Wine (2026-10-30).** Cached source links 244D6→2C8C4 to root 71E6D0; an
old normal Fight prediction image selected a 64-frame target-confirm input from
23851→23915. A distinct 71E590 COMBOMENU asset has constructor 30608 and redraw
30FDA; an Action seed at 23611 used two Down presses and A over 160 frames, ending at
23771. The same 31c0 CLI exited 0 for both; all 6 new counts stayed 0, while 83 ID
hashes, 86 managed paths and cache 28859 remained exact.

The static initializer link does not prove a new load. The PPM omits separate HUD/menu
content, so reaching Combo was not claimed. No observer build, guard, full pair,
endpoint state, longer retry, RNG resampling or old route recreation occurred.
Diagnostics were retained and discovery stopped. Positive ordinary use and full
equivalence remain open pending a concrete constructor context. Source, model,
runtime and CLI registration remain closed; Linux-USA, native Windows, new 32:9 and
the full TODO remain open, without metrics.
[Status and limits](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).


**ROMDATA-01-LOCALIZED-RAW-TABLE-SOURCES completed for fixed EUR views
(2026-10-30).** Seven independent roots, 71E5F4/608/66C/694/6A8/6E4/6F8, have five
physical slots each and retain the uncapped u8 language selector. Their 35 views and
42 spans contain 38220B, comprising 38080 payload and 140 selector bytes, with SHA
e11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80. Six prefixes
cover 224/160/768/1600/2048/2432B; only +0200..0380, or 384B, of the seventh group
is included. Lengths follow original copies and alternate branches, not distances or
headers.

The fused BL 12EE24→145230/CpuSet halfword and 6F8→000AD0 with control 80000040
halfword were verified; the first hypothesis was corrected before output. All 32
windows (23 new, 9 cached), 193 pins, 13 root literals, 13 fixed copies and 130 guards
matched without overlap with 83 owners or 50602 bindings; all 84 accepted files were
exact. Four dynamic 6F8 views use the u8 at task+2 minus 4 and u16 offsets shifted left 5
without establishing a cap. Metadata, the lower prefix, tail and full domain remain open.

This was static analysis only: schema 1, e92-d91 and runtime were unchanged. Model,
Wine oracle, runtime, CLI, ordinary use, full ABI, 71E5E0 alias, indirect references
and the full TODO remained open; new Linux-USA, native Windows and 32:9 checks were
to do. No metric, build, game, fixture, navigation or repeat of old proof occurred.
[Contract and limits](../rom_data_ui_raw_transfers.md#declared-source-contract).


**ROMDATA-02-LOCALIZED-RAW-TRANSFERS completed EUR/Wine (2026-10-30).** The new
core ui_raw_transfers/RawTransfers+BankView model has seven physical tables and 35
ordered views, with 42 spans totaling 38220B of LE words. Only +0200..0380 of the
seventh group is included. Editable shape is separate from EUR authentication and
32 fingerprints; u8 language addresses outside 0..4 remain raw without fallback or
cap.

The accepted warning-free build, 35bdc5fd, used 603 sources and 262 core inputs.
Three builds and two Wine attempts produced one final passing filter: 21 guards,
91 original prefixes, 60 isolated helper copies and 25 original DMA queue executor
copies. All bytes, EWRAM, VRAM, OAM, palette, IWRAM outside the original 36B stack
when needed, SP, callee registers and final ROM matched. The first corrections were
test-only: the stack guard changed from 20→36B before Wine, and an adjacent-table
boundary was corrected after it had been incorrectly expected to be raw. Diagnostics
and preimage were retained; model and source did not change.

All 44 unit and 130 source guards, 601 baseline and 603 frozen hashes, 3 current
hashes and inverse hub matched; 6635 instructions are coverage only. Schema e92-d91
and 84 accepted files were unchanged. Runtime, native clock, CLI, ordinary use, full
caller ABI, the 6F8 lower prefix/metadata/tail domain, 71E5E0 alias and full TODO
remained open. New Linux-USA, native Windows and 32:9 checks were to do; no old proof,
game, fixture, checkpoint, navigation or metric was repeated.
[Model and evidence](../rom_data_ui_raw_transfers.md#typed-model).


**ROMDATA-03-04-LOCALIZED-RAW-TRANSFERS-RUNTIME completed EUR/Wine
(2026-10-30).** e93-d92 publishes graphics.ui-raw-transfers at index 83: 85 files and
84 IDs, with all 83 previous records and payloads exact. Its 42 spans cover 38220B
from typed raw words and pointers with six separate counters. Original selectors,
copies, DMA queue, ABI and clock behavior are preserved.

Two builds used 605 sources and 263 core inputs; accepted 4b219adc was warning-free.
One Wine filter passed publication/interruption, 14 attachment and 2 full-set guards,
and 23 native full state-clock cases: 12 loads, 1 CpuSet, 1 queue executor and 9
language prefixes in 4 stop-safe families. They served 939 new B and 3 prior B with
fallback [768,0], not a distinct union or all direct native; 115 instructions and
25 dispatches are coverage only. All 38220B, 35 views, 7 groups, 42 edges, widths,
mirrors, new/old adjacency, raw gaps, Load, slot Undo and four mixed 64B transfers
(DMA16/DMA32/CpuSet/FastSet) passed. All 309 guards, 605 frozen hashes, 603 model
preimages, 7 current hashes, 84 old files and inverse hub matched.

Before Wine only the test was corrected: 441608 starts a previous indexed archive,
not a raw gap. Product code was unchanged between builds and first diagnostic
950bea97 was retained. Runner labels e92/DMA3 were obsolete metadata only: JSON proves
[1,93,92] and 4 transfers, without replay. Real CLI and ordinary use, full caller ABI,
6F8 metadata/prefix/tail domain, 71E5E0 alias, indirect references and the full TODO
remained open; new Linux-USA, native Windows and 32:9 checks were to do. No old proof,
source/model, game, fixture, checkpoint, navigation or metric was repeated.
[Runtime and evidence](../rom_data_ui_raw_transfers.md#managed-raw-transfer-resource).


**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One warning-free production build, ffda6c59, used 605 sources identical
to runtime without overlays; canonical hash and mtime were restored in a finally block.
One Wine orchestration ran three commands with child exit 0: create, reuse and neutral
startup for 1 frame from normal copy 24755→24756. All 85 files matched runtime,
84 ID hashes matched and 87 managed paths were unchanged; cache 28859 was exact.
e93-d92/index 83 contains 35 views, 7 tables and 42 bindings totaling 38220B.

The six new counters were 0, establishing registration and compatibility only, not
positive ordinary use or a full pair. All 203 guards, 605 CLI and 605 runtime hashes,
85 accepted files, ROM, seed, cache, TinyCC and canonical executable matched. No new
endpoint, derived checkpoint, action input, navigation, repeat of old proof or metric
occurred. Positive ordinary use, full ABI, the 6F8 metadata/prefix/tail domain, 71E5E0
alias, indirect references, global catalog and the full TODO remained open; new
Linux-USA, native Windows and 32:9 checks were to do.
[CLI and limits](../rom_data_ui_raw_transfers.md#actual-cli-registration).

**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-ORDINARY completed for one EUR/Wine
startup (2026-10-31).** CLI e93 observes 5,488 bytes served, including
selectors and payloads for four groups; a 1,200-frame pair using the same
dataset verifies state/clock/video/OAM/dispatch/audio. The same native cache
is used in both branches; other paths and Linux/USA remain to verify.
[Evidence and limits](../rom_data_ui_raw_transfers.md#ordinary-european-boot-equivalence).

**ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN partial (2026-10-31).** Original
calls document selectors 7/8/9 and 1,304 non-overlapping candidate bytes;
other producers read live values and the global limit is unknown. At discovery
there was no runtime owner; subsequent integration is described below.
[Details](../rom_data_ui_raw_transfers.md#bounded-dynamic-view-producer-discovery).

**ROMDATA-02-LOCALIZED-RAW-DYNAMIC-FRAGMENTS completed for the EUR/Wine
model (2026-10-31).** Rows and prefixes have a lossless typed model with
source and boundary verification for 1,304 bytes. Subsequent integration is
documented below; no global selector limit is inferred.
[Model](../rom_data_ui_raw_transfers.md#dynamic-fragment-source-model).

**ROMDATA-03/04-DYNAMIC-RAW-FRAGMENTS EUR/Wine (2026-10-31).** Separate
resource e94-d93/index84, 1,304 bytes in seven authenticated ranges;
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
(2026-10-31).** The existing 1,200-frame startup shows no alias bus reads in
the observed filter; positive ordinary use and Linux/USA remain open.
[Evidence](../rom_data_ui_graphics.md#clan-alias-ordinary-boot-observation).

**ROMDATA-05-CLAN-TABLE-ALIAS-BANK-ENTRY EUR/Wine (2026-10-31).** Two typed
ordinary alias reads during a Bank entry; a separate 120-frame pair with the same
state and input verifies state/clock, images and audio. Other paths, Linux and
USA remain open.
[Evidence](../rom_data_ui_graphics.md#ordinary-clan-alias-use-during-bank-entry).

**ROMDATA-01-E95-HISTORICAL-READ-RESIDUE bounded (2026-10-31).** Of the
1,146 unmanaged locations in the historical F09/e71 audit, 74 are now covered by
e95; the other 1,072 have previous code-support evidence. This is not a new
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
2,028 bytes in five ranges without overlap with e100 owners. Sequence 8 and ten
additional lists remain physically consistent candidates without witnesses;
no guest limit is imposed. Model, runtime, ordinary use, Linux and USA remain
open. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).


**ROMDATA-02-E100-ROOT-6B9B80 EUR/Wine model (2026-10-31).**
The typed model reconstructs the 2,028 attested bytes, verifies the source and
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


**ROMDATA-01/02/04-E107-ROOT-6CC124 EUR/Wine (2026-11-03).**
Three original constructor calls witness selectors 0/1/2: 21 frames, five lists
and four objects, with one empty list. The contiguous 102-byte bank has no
current-owner overlap; following padding/candidate excluded and no guest cap
introduced. e108-d107 publishes its typed JSON while preserving all 97 previous
resources. Source/model, byte/boundary/transaction and native valid-selector
substeps pass; the initial full runtime filter failed only on its expected-zero
counter oracle for selector 256. A retained-input continuation confirms the raw
zero directory offset returns to two owned count bytes, with full state/clock
matching. Combined native cases serve 14 typed bytes; budget check passes. Production CLI
create/reuse/startup also passes: 99 files equal runtime, 98 resource identities
match and all 101 managed files are unchanged. Neutral new-bank counters remain
zero in that initial frame. A later normal IN-19 EUR Roulette confirmation in
Mage Tourney serves 4,744 reads/5,831 new-bank bytes; the same 600-frame native
raw/managed pair passes full state/clock/render/OAM/dispatch/audio. Animation
completion and ordinary selector/list completeness remain unproved. Linux,
USA and native Windows remain pending; no benchmark.
[Contract](../rom_data_obj.md#ordinary-roulette-producer-and-eur-validation).


**ROMDATA-01/02/03/04/05-UI-CONTEXT-TEMPLATES EUR/Wine (2026-11-03).**
A fresh 12-frame native API inventory identifies record 5 of the category-selected
UI context templates 394e44..394f20. Original selector image 0..10 and copy length 20
prove eleven records/220 bytes; no adjacency/global domain inferred. 48 opcode pins,
full mapping, exact copied IWRAM helper and no e108 overlap pass. Typed x/y,
LE16 allocation count, flags and 15 opaque bytes retain every bit. e109-d108 publishes
99 resources/100 files, preserving all 98 earlier resources/bytes. CLI create/reuse
and focused source/integrity/boundary/in-memory rejection checks pass; 256 original
categories and 22 native/raw copy/coordinate prefixes match state-clock with 308
typed bytes. Ordinary seed 11960→11972 serves 5 reads/20 bytes and passes full raw/managed
state-clock-render-OAM-dispatch-five-stream audio equality; all 99 CLI identities match.
One current dataset only; old baseline retired under input/lease/host checks,
evidence and originals retained. Complete UI extent/consumers/all-record ordinary
coverage, Linux/USA/native Windows and remaining ROM/mod work open; no benchmark.
[Contract](../rom_data_ui.md#category-selected-ui-context-templates).


**ROMDATA-01/02/03/04-COMPACT-CATALOG EUR/Wine (2026-11-03).**
A bounded retained 118-reader query yields 65 calls/36 roots. Excluding eight
prior compact owners leaves 41 calls/29 new roots/38 selectors; 37 original Wine
windows and 12 constructor save/restore pins authenticate the selected flows.
Only witnessed directory cells/sequences/referenced lists are included:
207 frames/160 list identities/888 objects, 6650 bytes/65 union spans, no 99-owner overlap.
The generic e110-d109 model/runtime publishes 100 resources/101 files; all 99 prior
resources/bytes unchanged. Canonical builds warning-free, 709 source hashes stable;
initial signal 143 interruption retained, same-source continuation accepted.
One Wine source/native filter passes full bytes/boundaries/coherent rejection
in memory, all new reader fields and 67 native state-clock cases (187 typed bytes,
including 35 out-of-projection owned intersections). CLI create/reuse/startup
recognizes 100 identities, 103 managed files unchanged. Twelve startup frames
have new counters 0, not ordinary proof. Actions 303/310/312 are absent from the
human active bank; no cast of rejected leads performed. Ordinary/global/indirect
coverage, Linux/USA/native Windows and other ROM/mod work remain open; no benchmark.
One current dataset; old comparison retired with evidence and real inputs retained.
[Contract](../rom_data_compact_catalog.md).
