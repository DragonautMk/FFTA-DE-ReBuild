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
0x0800170c use the direct ROM table at0x14ced8; their host decoder's old IWRAM
selection was corrected against the original instructions during ROMDATA-04.
EUR also contains an identical table at0xd976e8 in the copied-code source area,
with the same relative source/RAM offset as the copied code. Both ranges are
mapped. The protected EUR boot-to-battle replay now observes actual alias reads
at frame5400 (primary first read541), with every full guest/RAM state matching
legacy reads through8476frames; see the
[validation protocol/results](ROM_DATA_VALIDATION.md). This is per-frame source
coverage, not attribution to a particular DMA/PC. An independent USA clean-boot
trace now reaches the original copier at0x0808f74c, frame791. Its own literals
at0x8f764/68/6c specify source0xa38adc..0xa3991c and IWRAM0x03005f28:
3648bytes, including the48byte alias at relative0xe00. Original CpuSet SWI
at0x0814186c performs24decoded halfword reads; all13initializer steps match
legacy full states. Independently inspected EUR entry0x080924e8 and literals
at0x92500/04/08 specify source0xd968e8..0xd97728 and the same destination;
the EUR replay above does not attribute execution to that entry. These blobs
include executable code, which remains in ROM; only48metadata bytes migrate.
Existing consumers and original execution are described in
[sprite routines](../ffta_sprite_routines.md) and implemented in
`ffta_sprites::{decode,decode_battle,visibility_result,ffta_sprite_interpret}`.
Both exact local regional images have been read independently: all four
48-byte ranges have SHA-256
`6ff886b23cde2726a4ceb9cba8c5569bbb4dc6a1185769547f78dfbcb7e2d991`.
Both copied aliases are checked independently, not assumed equal to the primary.
Exact-byte occurrence searches find two copies per region. Extractor revision2
records the newly discovered EUR alias; schema/decoder remain1.
The format is language independent; this does not validate localized assets.

Decoded files retain indexed dimensions rather than a rendered picture.
Host readers must preserve any genuinely changed RAM dimensions; a static
resource cannot replace dynamic RAM without checking the consumer contract.
CPU/DMA/interpreter/native source reads and typed host consumers now use the
decoded files with --rom-data. No original instruction or decompression is
skipped. [Runtime evidence](../rom_data_runtime.md) distinguishes conditional
guest reads from ordinary in-view boot/host use; broader gates remain05.

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
  New1200frame clean boots independently observe four LZ77 sources per region:
  USA56f4a8/5704cc/56fefc/56f52c and EUR76c64c/76d670/76d0a0/76c6d0.
  Consumed lengths125/133/1481/2507decode to160/456/1996/2960bytes, matching
  actual IWRAM3860/EWRAM3cb0post-SWI outputs on Linux and Wine. Original callers
  independently reference88byte descriptors atUSA569104/EUR7662a8; record0
  field0c resolves the first source. Fields04/08/10in record0point to outer
  wrappers containing the observed LZ77 streams. Typed halfword and indexed-pair
  packets now match actual regional writer instructions on Linux/Wine, including
  staged/chunked application. Literal pair syntax retains a destination guard
  after a contradictory original-code probe. The contiguous162-record descriptor
  block is now source-authenticated/typed, with full metadata roundtrip and
  original conditional-selector checks for all records on both targets. Relative
  sources, optional transfer triples/variant source and tail bytes are retained.
  All field04/08/10packet links now resolve352verified source programs per region,
  with wrapper/raw/LZ77/inheritance/dictionary extents and original-loader output
  equivalence for every162records. EUR main field00graphics now has50verified
  custom-LZ sources and direct typed file consumption. EUR transfer programs
  have42typed controls/34shared raw banks, eight-file managed publication and
  original tick/DMA evidence and direct CPU/DMA/native source consumption. All162
  variant pointers are absent in this declared EUR block. Other catalog bounds,
  new USA checks and language completeness remain unverified;
  this is not a complete graphics catalog.
  [Decoder contract, exact evidence and remaining work](../rom_data_graphics.md).
  Managed descriptor extraction/preparation: implemented, v1/e6-d5; bounded
  offline LZ77/packet decoders also tested. Authenticated BG descriptor reads
  now consume a file-derived session projection through CPU/DMA/native APIs,
  with independent counters, real-file regional provenance and native boot
  parity. Graphics payloads still reach VRAM/palette through original producers.
  The complete base-palette links now identify127unique sources per region
  (125LZ77,2raw), matching source/decoded hashes across regions and original
  producers for all162records on Linux/Wine. Preserve allu16bits, including
  bit15. Alternate environment streams now have a separate verified catalog:
  157valid keys in each of two regional tables,314streams/256Boutputs per region,
  custom LZ/RLE with BEsize header. All source/output hashes agree and original
  producers match on Linux/Wine. Five FFdescriptor keys are unavailable;
  caller preconditions remain unknown. Typed production decoding now matches
  retained outputs and new original synthetic/negative cases on Linux/Wine.
  Full banks, original links/keys and consumed source metadata are preserved;
  malformed input/overruns are rejected. Managed extraction/preparation now publishes and authenticates the fifth
  fixed file under e7-d6, retaining e6and all original source metadata. Actual
  runtime consumption of alternates now serves literal source bytes from
  decoded banks through CPU/DMA/native reads, with exact typed links/control
  metadata. Poisoned-source original/native and lifecycle checks pass; declared
  EUR LCD B/TV pub source-use scenes and native GUIs now pass on Linux/Wine.
  Broader scenes and global FFcaller preconditions remain separate. Base words now have typed readable extraction/preparation and direct04
  consumption: decoded banks for palette HLE, exact source views for CPU/DMA/
  native/guest BIOS reads, with original writes/cycles and state parity. Needed:
  indexed pixels, palette identity/indices, transparency, tile order and all
  writes/timing, not only screenshot similarity.
- **Terrain maps and movement planes.** `ffta_stage.rs` observes DMA and
  verifies two calculated 64x64 tile planes in EWRAM, then checks their VRAM
  crop. `ffta_movement.rs` observes a separate dynamic movement plane.
  `ffta_rom::map_producers_verified` independently pins producer code
  0x1af08..0x1b5d0 for each region; this is code, not a map archive. New EUR
  static dataflow ties staged loader1B688/state6→1AE88 to the existing
  root7662A8/162descriptors and all486field04/08/10links into352packet sources.
  Extractor/typed decoder/runtime already use `graphics.bg-descriptors` and
  `graphics.bg-packet-programs`; no duplicate terrain resource is needed.
  Auxiliary patches initialize the normal16×16height/flag grid02007CB0,
  assigned to live context02007F10+4/width+8/height+9. Original accessors read
  heights and mutate flags, so static seeds do not replace live cells or
  calculated EWRAM/VRAM/movement planes. Forty instruction/11literal/nineBL
  fingerprints and exact descriptor/source/ownership checks pass without
  guest execution. Other context paths, ten-byte accessor1DFB4 and complete
  flag semantics remain unclassified. Separate raster palette root4872B0now
  has a bounded source contract:32records/324selector links/33ranges/10496B,
  64index192header10240colorB, no overlap with the49prior owners. Original161halfword
  copy reads a neighboring header or existing alternate-palette owner at
  489BB0, then overwrites the extra destination word; preserve all bus/timing
  behavior without duplicating that source. Strict lossless typed model and
  one Wine original-reader/copy oracle now pass all10496B/324links/15guards,
  64initializers32getters/11rawgetters7rawprefixes3disabled/36full322Bcopies,
  deferred/explicit-source/queuedDMA and synthetic bit15/range words. Whole
  RAMoutside stack/SP/callee checks pass;4753instructions202PCare coverage only.
  Managed e55-d54now publishes51files50resources/all49prior decoded/e54retained,
  actualBGdescriptor dependency and one immutable typed owner. Two new Wine
  filters firstPASS17guards/lifecycle/all10496B and28original-interpreter-native
  complete-state-clock paths, compiled[15index46header24color]B, original copies/
  wrap/queuedDMA/rawneighbors and mixed alternate owner. Widths/mirrors/BIOS
  bus-mode/SWI/DMA checks pass;646guards. ActualproductionCLIcreate/reuse-no-write
  53managedfiles/all51generation equality/oldcache and one1500frame genuine
  coldboot Winefullstate-clock-image-OAM-dispatch-audio pair nowpass. Ordinary
  50880aggregateB=[2index322header50556color],mask2source records includes
  neighbor-header touches; not two full palettes or distinctB.701guards pass.
  Other contexts/newLinux-USA-nativeWindows/new32:9 remain pending.
  [Source and ordinary contracts](../rom_data_maps.md).
- **UI choice lists.** [The bounded EUR contract](../rom_data_ui.md) identifies
  twelve immutable lists/36entries from16direct widget8BBF0callers plus a six-
  entry397A9Ctemplate used by two live filters. Thirteen ranges/42entries/336B
  are disjoint50owners; all18root literals/93instruction fingerprints/static
  byte reconstruction pass. Eightu8fields retain ID/X/Y/opaque and four raw
  navigation values. Live0200F250filter/copy/overrides and unmasked flagged
  coordinate addressing stay original; no guest count/target guard. Strict lossless
  model/shared source plan and one focused Wine original copy/getter/template
  oracle now pass17guards/all336original-loadsiteB/18copy prefixes/44getters/
  43idle calls/64filter prefixes/2raw-coordinate prefixes/703hash-mtime guards;
  warning-free Windows test build. No fullwidget/native/fullstate-clock claim.
  Managed e56-d55now publishes52files51resources/all50prior decoded/e55retained,
  noassetdependency/fullsetcoherence/USAe8-d7 unchanged. Two reduced Wine checks
  pass17guards/lifecycle/all336typedB/13lists and29original-interpreter-native
  complete-state-clock paths/compiled[92ID183coordinate92opaque368nav]aggregateB,
  widths/mirrors/BIOS-SWI-DMA/oldOBJ-raw-new/706guards. Original RAMfilters/input/
  cursors/counts remain live; no guest caps. ActualCLIcreate/reuse52generation/
  54managedfiles no-write/oldcache and one320Party fullstate-clock-image-OAM-
  dispatch-audio pair now pass32aggregateB=[4ID8coordinate4opaque16nav]/1list/
  769guards. Two negative battle contexts remain closed/no pairs; other12lists'
  ordinary use pending, no wholewidget/new32:9 claim. Existing fixtures reused. New Linux/USA/native Windows checks pending.

- **OBJ graphics, descriptor lists and animation.** `ffta_sprites.rs` decodes
  a signed/unsigned count followed by packed six-byte OBJ records, reads tile,
  priority/palette/flip fields and observes partial OAM DMA. Lists can live in
  ROM or RAM; animation, owner state, matrices and OAM are dynamic. Profiles
  and copied routine source ranges are independently hash gated per region in
  `ffta_rom.rs`; these are not a catalog of all lists or frames. A bounded EUR
  direct-literal catalog now identifies107references and66static uncompressed
  lists/160objects/1092source bytes. Typed signed coordinates and every packed
  attribute field reproduce both original EUR draws in1056Wine calls, including
  whole OAM buffers/matrices/guards. Managed e14-d13preparation and typed guest/
  native/host consumption pass focused Wine publication/source/native/lifecycle
  checks and one397-frame ordinary scene/actual CLI pair; see [the OBJ contract](../rom_data_obj.md). Other indirect/table
  roots, frame timing, sprite graphics/palettes and cross-resource links still
  need discovery. A further independently located EUR UI family has ten indexed
  OBJ/sequence bank pairs,167lists/284objects/95sequences/184frames,282consumed
  source ranges. Relative indices and u16durations match original constructors/
  scheduler body on Wine; e15-d14managed and typed native/host/lifecycle checks
  pass with all raw sources altered. Three empty sequence direct-root domains are now classified below;
  their indirect/live scheduling remains unverified; one160-frame ordinary menu/actual CLI pair now proves positive
  index/list/frame/duration and host consumption with full-state/audio parity. Conservative control-flow
  discovery also locates sixteen additional references/seven new direct lists,
  now migrated under e16-d15: current static totals are 123 references/73 lists/
  173 objects/1184 source bytes. All original 66 records and other decoded files
  remain unchanged. Three focused Wine integration checks and one positive
  240-frame boot language-screen/actual CLI pair pass with new-list guest/host
  coverage. Other switch/indirect callers remain unclassified. New Linux/USA
  verification pending. EUR e17-d16now adds an independently rooted eleventh
  pair b4d8e8/b4f4e8: one list/two objects/five zero-count holds, eight new
  source ranges/58bytes. Explicit selectors and duration-source aliases retain
  the original nonrepeat contract and reuse typed header/index owners. All
  ten prior bank records remain unchanged; focused Wine original/read/native/
  host/lifecycle checks and one positive240-frame Party/actual CLI pair pass.
  Constructor-fed sources at38008/88328are now resolved: e18-d17adds five language-selected
  pairs from two five-entry pointer tables and one old-pair reference. New
 149lists/574objects/144frames/166ranges/4984Bretain original selector loads
  and duration256/counter-wrap semantics; current16pair catalog has317lists/
 105sequences/328frames. All11prior bank records/other files stay unchanged.
  Focused Wine source/original/native/host/lifecycle and a positive400-frame
  “Engage!”ordinary scene/actual CLI pair pass. The three older empty-record direct-root domains are classified in
  [the OBJ contract](../rom_data_obj.md#earlier-empty-sequence-caller-domains):
  twelve calls select0; eighteen initializer mode producers select0..3 and
  two updates select2/3, excluding the empty indices. The unproved hold
  extension is rejected; indirect/live mutation remains unverified. All thirty
  root literals/47instruction pins/50owners pass static checks. Six possible
  selector bytes stay raw; duration aliases reuse existing indices8/16/4. No
  resource/model/revision change or guestcap follows. Current e19-d18adds39custom OBJ-related word banks
  and six five-entry selector tables (13direct/nine bank roots plus six
  canonical indexed paths/thirty roots),45disjoint ranges/62674source B and
  131872decoded B. Source-authenticated literals/pointers feed original
  decoder/copy paths with live scratch transforms, partial uploads and RAM
  tails retained. Fifteenth obj-graphics.json leaves all earlier files intact;
  Wine source/managed/native/lifecycle and400-frame Engage parity/use pass.
  Other graphics/palettes and new Linux/USA verification remain open.
- **Normal unit OBJ initial pose.** The original
  2D394→233A8→CF144→22788 path supplies fullword animation0/part0. Existing
  metadata52 property4/metadata14 halfword04 project121IDs through394834,
  first pointer cells and121counted sequences/477operation1frames. Six
  OAMlists/31objects atA50020+relativeoffset determine333raw4bpp requests,
  135168B/4224decoded indexed tiles at898A40+relativeoffset. 00B14 is a RAM
  DMA queue constructor, not a codec;00800 executes realDMA3. The strict
  typed model/sourceplan owns415spans146358B, disjoint all51e56owners, with
  actual existing producer dependencies and opaque bytes preserved. Static
  94instruction/13BL/6literal fingerprints and one new Wine source-model
  oracle pass121originalreset paths/oneconstructorprefix/477scheduler-DMA
  uploads/allpixelB/16shape-sizeprobes/11guards.1797reserved/OAM/tailB remain
  preserved but unread in those source probes. EUR e57-d56 now publishes53files/
  52resources/all51prior decoded/resource records/e56retained/USAe8-d7 with
  actual metadata52/14dependencies. Two new Wine filters pass17guards/lifecycle/
  all146358typedB/30original-interpreter-compiled complete-state-clock paths,
  rawID/animation/liveRAMframe, original scheduler/DMA and415edges/mirrors/BIOS/
  SWI-DMA/new-raw-oldOBJmerging. Actual productionCLIcreate/reuse-no-write55files/
  all53generationfiles and one existing Party-unit320fullstate-clock-image-OAM-
  dispatch-allaudio pair pass10962aggregateB/3520pixelB/2sequences; other119ordinary
  sequences remain targeted-only. Job0/1values65535 unresolved;
  no guestcap/fullsprite/actioncatalog, otherposes/parts/IDs/palette claim.
  [Contract, typed model and limits](../rom_data_unit_obj.md).
- **Normal-mode actor battle poses (source/model).** A finite original-selector
  projection over121normal metadata52/14 IDs plus42 includes29body/8extra-part
  bases,10980logical keys/8374physical cells/5293nulls,2581sequences17894frames,
  150OBJlists676objects and3216raw4bpp requests. Byte-identical initial/equipment
  shares146358/6984B retain prior ownership; new6021spans1455474B are disjoint
  from all other54current owners. One warning-free Windows build/one new exact
  Wine sourcefilter firstPASS12guards/16474constructor prefixes/allthreeparts/
  5293null+2451sequence resets/100commands/2858scheduler-DMA/all1063200newpixelB.
  No old-owner original-reader replay or every actor/action reachability claim.
  Alternate247cross-product is rejected because animation104aliases directory
  394834; no invented extent/count cap. Dynamic/script/alternate pairing and
  actor pairing remains open. EUR e60-d59publishes/consumes typed sources with
  actual metadata52/14/initial/equipment dependencies;56files55resources preserve
  all54old records/e59/USAe8. Two reduced Wine filters firstPASS17guards/lifecycle/
  all1455474newtypedB and32original-interpreter-compiled complete-state-clock
  paths, allparts/null/rawIDs/commands0-2-3-6-7-8/6021edges/mirrors/BIOS/SWI-DMA/
  mixedoldowners/raw. ActualCLIcreate/no-write reuse56generation/58managedfiles
  andone existingF09Wait3530fullstate-clock-image-OAM-dispatch-allaudio pair pass
  842236aggregateB/815360pixelB/fivesequences, matchingCLI. Source/model/managed/
  native/CLI/ordinary declared children closed; other2446newordinary sequences
  targeted-only/alternate-script-livepairing/Linux-USA-nativeWindows pending.
  [Contract and typed model](../rom_data_unit_obj.md#normal-mode-battle-poses-source-projection-and-typed-model).
- **Alternate battle selector views (static source contract).**65alternate IDs/
  5850logical keys resolve5682physical cells:4709nulls/953counted-sequence cells
  naming706sequences2771frames, plus20explicit known-index-root views. All20
  aliases have initial-pose directory/index witnesses; the one4Bdirectory/index
  self-overlap is sprite247/animation104→sprite0. No sequence-count filter,
  dropped binding or adjacency extent repairs the rejected blanket interpretation.
  30OBJlists108objects/1203pixel requests387008B give3321new spans410086B after
  exact6232initial+52604normal-battle+22equipment sharedB. All55e60owner source
  hashes/overlap checks pass; no other conflict or mixed typed-item ownership.
  Tagged source model/one original Wine filter now pass15guards/all5850bindings/
  10544constructorprefixes including40shared-target aliases/4428null+606proper
  resets/five newcommands/1079scheduler-DMA/all342112newpixelB/wholeVRAM. No fake
  frames or oldoriginal acceptance replay. EUR e61-d60now publishes/consumes
  3321new spans with five actual dependencies;57files56resources/all55oldrecords/
  e60retained/USAe8unchanged. Two new Wine filters firstPASS20managed guards/all
  410086typedB/606newsequences and34original-interpreter-compiled fullstate-clock
  paths/all3321edges/BIOS-SWI-DMA/mixedraw-oldowner composition. StrictNull rejects
  extra fields while valid JSON stays identical. Source/model/managed/native
  and actualCLI closed:57generation59managed no-write reuse/all57acceptedfiles
  equal/oneframeoldcache attachment passes with zero newalternate use. Positive
  ordinary/live reachability pending;81existingEURsnapshots have no current
  newalternate-task lead, no new fixture/navigation/fullpair.
  [Contract and shared target views](../rom_data_unit_obj.md#alternate-battle-selectors-and-explicit-shared-target-views).
- **Script49 actor binding (live selector contract).** Decimal49(0x31), callback
  126CC0, has167five-byte instructions/18actor selectors/13bases.151selectors
  resolve live unit identity in twelve+twenty-four264Brecords;16resolve the live
  script registry/fallback, none encodes an immediate sprite. Context lookup
  matches unit pointers and uses current actor+34sprite; groupbase is low16of
  u16group×4, facing signedactor+1F, body flag0and samebase extra parts. Nearby31
  only patches byte01in new-unit branch; existing units retain it, and32only
  repositions. No static blanket sprite join or new ownership is justified.
  Exact existing scripts plus67instruction/nine literal/eighteen BL pins pass;
  no build/guest run. Selector classification closed, resolved query census/
  sprite pairing still pending. e61/USAe8unchanged.
  [Live selector contract](../rom_data_unit_obj.md#script-animation-selectors-require-live-actor-bindings).
  Newtest-onlyfunctional Wine6000census on retained snowstate/input resolves six
  command49sources with selector16/group2to sprite46/animation8/parts0-1-2,
  eighteen real constructor queries. Body is oldnormal ownership; two null
  cells37C760/37C764(8B)are unowned, no new sequence/pixels. Original49/body
  fallback steps and retained native constructor preserved. First all-native
  preflight failed before guest frames, corrected rebuild/actual census pass.
  No fixedselector→sprite claim; stable null-cell source contract and other161
  instructions remain open, no new model/runtime/generation.
  [Observed query evidence](../rom_data_unit_obj.md#source-linked-script-query-observation).
- **Normal special-unit script poses (conditional source contract).** Exact
  CBD24unit-byte4selector match/CBAD4special predicate/CF144logical metadata14
  normal field and original9B824assignment prove a conditional source join with
  override20clear/current same-unit normal sprite/facing0..3. All151low script49
  instructions give20selector/base references,18physical normal-sprite/base
  pairs/12sprites/108queries76nulls/32sequences241frames/11OBJlists64objects/
  95pixelrequests35968B. No blanket all-sprite product or failed row removal.
  All56e61owner audit finds4780initial+30586normal sharedB only, no mixed item/
  internal-role/extent conflict. New33spans6436B=240index28header920frame5248pixel;
  60newcells53nulls/sevennewsequences46frames(allop1)/17newpixel requests. Two
  observed37C760/64null cells belong to this ROM-derived plan, not private state.
  27instruction/one literal/threeBL fingerprints pass; no build/guest run.
  Source contract closed, model/originalWine/runtime/CLI/ordinary still pending;
  alternate/override/high16/current-state cases remain separate, e61/USAe8stable.
  [Conditional source projection](../rom_data_unit_obj.md#conditional-normal-script-poses-for-special-unit-selectors).
- **Equipment OBJ poses (source model).** Original9BA84→CE1EC item properties
  3/9 and9C07A→22788 select252eligible item records/61sprites124..184, normal
  bases0/4/8 andparts0/1. Explicit action12/13 overrides add185/bases0/4;
  action16/17 selects126/base16 except item45→124. Two physical orientations
  give748cells/86null/258sequences3439frames. Only1911operation1frames reference
  OAM/pixels;609op6/888op8/31op0 retain command words/tails. Twelve OBJ lists/
  25objects yield280raw4bpp requests37440B/1170decoded indexed tiles. Exact593
  spans110666B (248directory2992index69812sequence174OAM37440pixels) are disjoint
  all53e58resource sources.73instruction/4BL/4literal/twodispatch static checks
  and one new Wine source oracle pass1496constructor-prefix aliases/86null+
  258sequence resets/sixcommand cases/280scheduler-DMA/allpixelB/10guards.
  EUR e59-d58 publishes and consumes typed fields/55files54resources with actual
  item dependency/all53old decoded/resource records/e58 retained/USAe8-d7. Two
  new Wine filters pass17guards/lifecycle/all110666typedB and26original-interpreter-
  compiled complete-state-clock paths, nulls/commands/liveRAMframe/593edges/
  mirrors/BIOS/SWI-DMA/new-raw-oldowner composition. ActualCLI create/reuse-no-
  write55generation/57managed files passes; Sheep1000/Fight700 equipment-negative
  locators have no fullpairs. Authorized copiedEURPolly→Hildegard confirmedFight500
  singleordinary fullpair now passes allstate-clock-central-wide-OAM-dispatch-
  audio; equipment4477aggregateB/1408pixelB/twosequences exactlyequalactuale61CLI.
  Other256sequences ordinary remain targeted-only. No live cap/adjacency extent or actor-body/
  script pairing claim; new Linux/USA/nativeWindows pending.
  [Equipment contract and model](../rom_data_unit_obj.md#equipment-poses-source-model).
- **Color-Mode unit/UI palettes.** Five original helpers CF49C/CF4E4/CF51C/
  CF5A0/CF5DC and70direct Thumb calls prove30spans5750B=2873fullu16colors+
  fouru8slice selectors, disjoint all52e57owners. Complete512B base blocks,
  exact supplemental fragments,12Bshared colors and9special-unit selectors
  follow actual reads/copies, not adjacency; metadata14 supplies51active
  projections across93records. Preserve low2mode/low8argument/mode3default/
  companionmode1alias and all raw holes. Strict source model/sharedplan and
  corrected reduced Wine filter pass165original pointers/99copies/30staged
  readers/54special copies/12slice prefixes/all5750B/eightguards. First oracle
  had a destination-register seed failure; nearby original MOV/LDR decoding
  fixes the test, no model difference. EUR e58-d57 now publishes54files53resources
  with metadata14dependency/all52prior decoded/resource records/e57retained/
  USAe8-d7. Immutable typed colors/selectors/raw-gap/prior-owner composition
  pass two new Wine filters:17guards/lifecycle/all5750B and28original-interpreter-
  compiled complete-state-clock paths, low8/mode3/liveRAMpalette/30edges/mirrors/
  BIOS-SWI-DMA. Mixed2new+20raw+4oldUIB preserves exact2/4charges. ActualCLI
  create/no-write reuse56managed/all54generationfiles and one positive ordinary
  Lutia1700fullstate-clock-image-OAM-dispatch-allaudio pair pass1102aggregate
  colorB=[512unit512companion78supplemental]/fourfragments. Party320/boot240
  newfamily-negative locators have no fullpairs. Remaining26ordinary fragments/
  othermodes/shared-special-selector/wholeallocator-scheduler remain pending;
  source/model/managed/native proofs stay closed.
  [Contract and validation](../rom_data_color_palettes.md).
- **Raw UI palettes.** EUR e20-d19adds21source-authenticated raw banks/
 1328full-u16words/2656B/23direct-derived-Color-Mode references for both BG/OBJ
  palette-shadow halves. Original dispatcher145c14→copiedIWRAM03005ee8is
  independently initialized from176ROM bytes; actual selectors/copy paths,
  native poisoned-source/full-state/clock/bus/DMA and immutable lifecycle pass
  Wine, followed by a positive240-frame ordinary boot/CLI pair. This distinct
  sixteenth file leaves all14earlier decoded files unchanged. Original live
  fades/palette publication remain; indirect sources and new Linux/USA checks
  stay open. [Contract](../rom_data_ui_palettes.md).
- **Additional custom UI BG/OBJ graphics.** EUR e21-d20adds nine raw custom
  banks/ten checked literal original decode/upload paths,57038source B/
 84160decoded B with no earlier-resource overlap. Return-length/count propagation
  resolves callers omitted by the old constant-count OBJ filter. Eight sources
  feed BG and oneOBJ(two paths);43bc28uploads544of2944decoded bytes. Original
  source/native/lifecycle and one positive320-frame ordinary Clan/CLI Wine
  pair pass. Seventeenth file ui-graphics.json retains all15earlier decoded
  files/e20; original work/clock/VRAM remain live. Current e22-d21extends this
  same file to16banks/two language selector groups/two shared OBJ links,
 26ranges/71474B/119168decodedB. Seven new banks and ten pointer literals add
 17ranges/14436B; original code jump tables/language/fallbacks and5056OBJ/2688BG
  split remain. Reduced Wine original/native/lifecycle pass every new bank;
  all9oldUIrecords/other15decoded files/e21retained. One1200-frame English intro
  full-state/image/audio/CLI pair positively consumes4selectorB/2347new-sourceB/
 1124literalB. Other ordinary languages/indirect sources
  and new Linux/USA checks remain open. [Contract](../rom_data_ui_graphics.md).
- **Text and fonts.** `ffta_dialogue.rs` classifies already rendered layers
  and portraits, not strings, font encodings or localized text archives.
  `ffta_mouse.rs` identifies a name-entry row-count table via independently
  checked regional literals: EUR 0x3ad2fe (three pages, six bytes each), USA
  0x3a8320 (two pages). That table describes navigation widths, not character
  codes or text resource pointers. A bounded EUR short-string catalog now has
  five independently located selector tables with107entries each:535strings,
 478compact/57extended,5468source bytes; only its65referenced packed4bpp glyph
  slots/metrics are extracted. Original language/copy/font/width readers match
  every pointer/output/guard in two reduced Wine checks (1331calls). Ninth EUR
  managed file under1/e11-d10authenticates all670ranges; publication/reuse/
  repair/move/transactional activation and actual CLI pass Wine. Runtime now
  consumes typed indices/codes/glyph words/metrics through fixed CPU/DMA/native
  pages and separate counters, preserving original rendering and guest work.
  Five altered-source original/native chains (20calls/full states/clock equal),
  source/SWI/DMA and unavailable-copy lifecycle pass reduced Wine. Other short
  A broad offline model now covers all9pointer selectors/45tables/16100strings
  and29relative selectors/145archives/17035programs from the declared reader
  block, with128referenced glyphs and33581disjoint source ranges. Two reduced
  Wine checks match independent inventories and original pointer boundaries/
  archive roots/300script outputs/63new font slots. Dynamic commands retain
  operands; the controlled oracle supplies valid one-based context1, not a
  gameplay claim. Broad managed EUR1/e12-d11now adds a tenth file with33581
  source ranges, complete-set authentication and atomic prepare/reuse/repair/
  move/transactional activation plus actual CLI verified on Wine. e11/eight
  prior decoded files and USAe8remain unchanged. Broad runtime is still
  directly consumed via typed pointer/string/index/header/program/font fields,
  including decoded custom-LZ literals and all8source counters. Fifteen
  original/native chains with33581raw sources altered match90full states/clocks;
  synthetic encoded-literal alteration and unavailable-copy lifecycle pass Wine.
  Earlier overlapping subset is authenticated without duplicate ownership.
  One retained397-frame pub transition matches full states/images/audio with
  positive raw-program/font use; actual CLI reuse/old-cache parity passes Wine.
  The auxiliary archive at4f95c4now provides three typed name-keyboard programs,
  18navigation widths and180derived cells. Its last referenced glyph3174ends
  exactly at4f885c, bounding the complete3175-slot packed font/metric bank.
  Both files join managed EUR1/e13-d12and direct CPU/DMA/native typed consumption;
  the complete bank takes over earlier overlapping font sources. Source/original,
  managed provenance/repair/move, native altered-source/full-state/clock,
  lifecycle and one397-frame ordinary font transition/actual CLI pass on Wine.
  Keyboard ordinary scenes, other catalogs/control semantics/Unicode and new
  Linux/USA checks remain open. See the
  [text/font contract](../rom_data_text.md); five selector IDs are verified for
  this declared subset, not the complete localized text catalogs.
- **Gameplay tables.** Unit structures at EWRAM 0x020223a4, stride 0x90,
  are live state, not a static unit database; see
  [character metrics](../ffta_character_metric.md). Battle/shop mouse readers
  validate transient tasks/inventory. A bounded original shop/property chain now
  roots EUR item table6e0bd0..6e3ad0:376records/32bytes/12032B,19property field
  selectors, independent1..361/362..375enumeration bounds and following table
  root. Raw halfword/byte/signed fields and live02002030modifier branches are
  distinguished; all32record bytes roundtrip and no e22resource overlaps.
  Current e23-d22publishes eighteenth items.json/resource gameplay.item-records
  and directly consumes typed fields via original CPU/DMA/native logic, retaining
  all16previous decoded files/e22. Reduced Wine original14371+240isolated-negative
  and managed/native/lifecycle checks pass;86native paths with altered source
  match complete states/clocks. One positive320-frame Unit full-state/image/audio/
  CLI Wine pair uses40typed signed-field bytes. New Linux/USA checks remain pending. Following
 20byte table/item semantics/job/ability/law/AI catalogs still need contracts;
  live inventory/modifiers and secondary reads remain guest state/code/ROM.
  The following item-linked table now has a standalone typed source contract:
 225records/20bytes/4500B/count0..9/opaque/nine pairs,472active entries/eightFF,
  all376item owners checked,220referenced rows and independent next root6e4c64.
  Wine2244original cases/4synthetic queries plus8actualFFpredecessor gates pass.
  All unused bytes remain exact. EUR e24-d23now adds item-links.json with parsed
  item-owner dependency and typed CPU/DMA/native count/selector/value/opaque
  views. Every17old decoded file/e23retained; reduced Wine managed/lifecycle/
 62native paths with4500raw bytes altered match full states/clocks, including
  a typed crossing read of both adjacent owners. One positive560-frame ordinary
  headgear inventory full-state/image/audio/CLI Wine pair consumes574new bytes
 (287count/205selector/82value); first slots-only locator is negative, not rerun.
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
  Following metadata2 owns 44 key/value records / 88 bytes at
  6e6c44..6e6c9c, including real row43. Original direct u8 indexing reads all44;
  inverse u8 lookup scans only43 and returns first stored key / miss0. Parsed
  metadata52 key owners, independent next48Bsource, strict lossless source/JSON
  guards and one Wine348real+13synthetic oracle pass. Live byte7 wrapper prefix
  retains its frame; global caller bounds/semantic value names remain unproved.
  EUR e28-d27adds linked-metadata2.json/gameplay.linked-metadata2,23files/
  22resources; all21prior decoded files/e27retained. Two focused Wine managed/
  lifecycle/native checks pass21paths with88rawB altered/equal full states-clock,
  typed boundary/DMA/SWI. One positive1840Pub mission full-state/image/audio/
  actualCLI pair reads4value/0keyB; inverse key use stays targeted-only.
  Negative ChangeJobs/Rumors/Missions locator evidence remains closed.
  Default48owns6e6c9c..6e6ccc/48Bfrom exact constructor copy length,
  with structural byte/halfword fields and original signed-ID/selector/argument/
  ORflags patches. Mode1/2and four alternate IDs20..23stay distinct. One Wine
  oracle passes271real+30synthetic constructor prefixes after original IWRAM
  copy initialization; allocation/live-unit tails and four alternate192B/ROM
  pointer/following240B remain unowned. EUR e29-d28adds fixed default-constructor-
  template.json/resource gameplay.default-constructor-template,24files/23resources,
  all22previous decoded files/e28retained. Two focused Wine managed/native/lifecycle
  checks pass16paths/all48rawB altered/fullstates-clocks equal/IWRAMhelper and
  eight alternate/rejected paths0newreads. One397Pub full-state/image/audio/
  actualCLI pair reads96newBacross every structural field category.
  Static next contract separately roots four alternate48Brecords/192B at
  6eca78..6ecb38and pointer70cd58..70cd5c/4B,196Btotal/noe29overlap. Original
  gatedID20..23copy and five equipment-slot reads perrecord/parsed376item owners
  checked; byte01meaning/neighboring headers unclaimed. Strict typed4record/
  pointer catalog and one Wine24real+8synthetic original-copy/live-patch oracle
  pass source/JSON/fixedarray/order/ID/item-owner/stack/outerRAM guards. EUR e30-d29
  adds constructor-alternate-templates.json/gameplay.constructor-alternate-templates,
  25files/24resources/all23previous decoded files/e29retained. Two focused Wine
  managed/lifecycle/native checks pass14paths/both raw source spans (196Btotal) altered/equal
  fullstates-clocks, native pointer/all4recordsmask15 and5zero-new-use paths.
  One3600Lutia deployment full-state/image/audio/actualCLI pair reads45B(4pointer,
  41record20B/mask1); remaining records/full-copy behavior targeted-only. Closed
  source24+8/default271+30/native16/397proofs not repeated.
  Next static14Bcontract:93records/1302Bphysical6e6f6c..6e7482/logicalIDs2..94,
  original addressbias6e6f50/predicate>1, independent next12Bsource6e7484and
  unowned2Bgap. Exact reconstruction/noe30overlap/parsed field-range checks pass;
  byte02live-job link supported, byte01semantic owner unproved. Strict lossless
  model and one Wine504real+13synthetic getter/live-palette/initializer-prefix
  oracle pass exact source/JSON/bias/owner/stack/unit/palette/outerRAM guards.
  EUR e31-d30 adds linked-metadata14.json/gameplay.linked-metadata14,
  26 files/25 resources/all24prior decoded files/e30 retained. Two focused Wine
  managed/native/lifecycle checks pass37paths/raw1302Baltered/fullstate-clock
  equality; normal0/1no new source. One397Pub full-state/image/audio/actualCLI
  pair reads27B/two records/all four field categories. Separate6e6dbc4Broot/
  12Broot contracts, global bounds and newLinux/USA/nativeWindows remain open.
  Two distinct12Bindexed banks6e7484..6e78d4/92rows and6e78d4..6e7994/16rows
  now have a strict lossless source model:108rows/1296B/5184two-bit pairs.
  Shared135a1c reader uses full32row/signed-wrapping slot/u8mode; caller loops
  slots0..43, all48stored pairs retained. Next6e7994global root excluded.
  One reduced Wine source oracle passes1640real/10raw-outside-wrapping/384
  synthetic cases plus exact LDRB/address/source/JSON/callee/stack/RAM guards.
  EUR e32-d31 now adds linked-metadata12.json/gameplay.linked-metadata12,
  27files/26resources/all25earlier decoded files/e31 retained. Two focused Wine
  managed/native/lifecycle checks pass49paths/raw1296Baltered/fullstate-clock
  equality/bothbanks positive/three outside0newB and bank-boundary/SWI/DMA.
  One2200-Echo Grass confirmation full-state/image/audio/actualCLI pair is positive.
  Global bounds/flag semantics/global bank and newLinux/USA remain pending.
  The separately rooted global pair source6e7994has a minimum44pairs/11B
  source contract from original1377b4RAM-mask loop0..43/mode0. Threebytes
  before independently rooted6e79a2remain unowned; no12Browstride inferred.
  Strict fixed11×4groups and one Wine102real/10raw-wrap/91synthetic oracle
  pass wrapper/bothbits/live-mask/earlyexit/address/callee/stack/RAM/JSON/source
  guards. EUR e33-d32 now adds global-pair-flags.json/gameplay.global-pair-flags,
  28files/27resources/all26prior decoded files/e32retained. Two focused Wine
  managed/native/lifecycle checks pass28paths/raw11Baltered/fullstate-clock
  equality/newtypeduse/prior-typed and raw-tail SWI/DMA crossings. ActualCLI
  creation/no-write reuse/oldcache/cwd pass;2200EchoGrass0globalB is negative,
  no fullpair. Positiveordinary globaluse/fullpair/highmode/newLinux/USA
  remain pending, bounded descriptor/live-mask gates guide future selection.
  Evaluation parameters have a separate38Bsource contract:7divisor-weight
  terms/2conditional pairs at6e79a2..6e79b4and5class1..5threshold records at
  6e79c6..6e79da. First unsignedu8quotient/remaining6signed32/wrap/low8bonus
  predicates/originalLCG/signedlow16%256thresholds retain original code.
  Strict model and one Wine132actual-source/52mutated-source oracle pass,
  including12live-score prefixes and originalRNG/class0skip.14Bscoregap/4Brow0
  unowned; no32/24Bgap inference/foreignowner. EUR e34-d33 adds evaluation-
  parameters.json/gameplay.evaluation-parameters,29files/28resources/all27prior
  decoded files/e33 retained. Two focused Wine managed/native/lifecycle checks
  pass27prefixes/raw38Baltered/fullstate-clock equality/classmask31/class0zero/
  SWI-DMA/rawgap checks. One1840Pubmission full-state/image/audio/actualCLI
  pair reads15scoreB; later400frame Yes/fee CLI/Winefullpair reads1threshold
  class1/mask1, classes2..5remain targeted-only. Whole tails/
  semanticlabels/newLinux/USA remain pending.
  Next static10Bcontract roots6e79da/*10/0default and normal index0..99from
  04B3E0..04B416,100records/1000Bthrough6e7dc2.12zeroBbeforeindependent6e7dce
  remain unowned; candidate101rows rejected. Eightu8deltas feed original live
  level/progress/flags through04870c; byte08feeds live itemprice discount,
  byte09structuralunused. Unsigned/signed/u8wrap/min-price/zero behavior and
  actual parsed Items base are distinguished; no storedforeignkey inferred.
  Exactroundtrip/noe34overlap/roots/stride/fields pass. Strict source model and
  one Windows/Wine oracle pass362real/68synthetic publisher/delta/full-price
  cases with actual parsed Items and complete EWRAM/SP/callee guards. Source
  work closed. EUR e35-d34adds30thfile/29thresource; all28old decoded files/
  e34retained. Two focused Wine managed/native/lifecycle checks pass27original/
  interpreter/native paths, raw1000Baltered/fullstate-clock equality, typed
  boundaries/mirrors/BIOS/SWI/DMA/coexistence. ActualCLI no-write reuse and
  one600Shop fullstate/image/audio pair read7discountB/record4; ordinary
  deltas targeted-only, newLinux/USA remain pending.
  Next static progression-gates roots6e7dce/stride12/index0..63,768Bto6e80ce;
  independent next root D23E6→D24A8. Byte00unused/halfword01return/byte03output/
  eight live unsigned minima and original u64mask order preserved structurally;
  row63zero return still marks bit63. Roundtrip/noe35overlap/opcodes and strict
  source model/one Wine full0487c8oracle80real+25synthetic pass all64rows/eight
  liveunsignedminima/maskorder/zero-success/704readbytes/fullRAM/SP/callee
  guards. EUR e36-d35adds31stfile/30thresource/all29prior decoded/e35retained;
  two focused Wine managed-native-lifecycle checks pass19original/typed/native
  fullstate-clock paths/raw768Baltered/two-page typed bus/zero-success/fullmask/
  SWI-DMA/gap/coexistence. ActualCLI no-write/oldcache and one1000Wine normal
  dispatchcompletion fullstate/image/audio pair pass209requirements/8deltaB;
  return/outputordinary targeted-only. Separate400frame Yes/fee prefix closes
  class1threshold ordinary use; downstreamreceipt-FK/newLinux/USA remain pending.
  Next selection gates own64x16/1024B6e80de..6e84de fromD2390physical1..64,
  notrow0. Hardunsigned minima01/02/04, ANYactive06..0d and flag0e preserve
  physical firstmatch, zero-condition store0 and exhausted64no-store. Actual
  row52sentinel/53..64storage distinguished; independentnextroot6e84e0leaves2B.
  Static contract/roundtrip/noe36overlap and strict source-only model pass; one
  Wine fullD2390/reset-count-flag oracle138real+80synthetic checks831actualfieldB/
  all1024Broundtrip/ANY/unsigned/zero-terminal/no-store/fullRAM-SP-callee.
  EUR e37-d36nowadds32ndfile/31stresource, all30prior decoded/e36retained.
  Two focused Wine managed/native/lifecycle checks pass18fullD2390paths/raw1024B
  altered/fullstate-clock, all6fields/64recordmask/bus-BIOS/16Brow0+2Bgapraw/
  SWI-DMA/oldgate2-raw16-new2. ActualCLIcreate/no-write reuse/oldcache and one
  1000Wine ordinary dispatchcompletion fullstate-image-audio pair pass277newB/
  all6fields/first26records;400Yes-fee0Bclosednegative/no fullpair. NewLinux-USA/
  all64ordinary/semanticdisplay remain pending.
  Next aggregate key table owns6e9a5c..6e9cb6/602B(300u16slots+ffff),
  D22FCglobal and13FB1Ccontext counters share exactsource, next root29AF4atend.
  Strict source-only model/one Wine32real+26synthetic oracle pass both complete
  counters/all602B/key+2ffu16wrap/duplicates/earlystop/flag36ORlinked017f/
  contextisolation/fullRAMreadonly-SP-callee guards. OtherindexedUIreader roots
  static-only. EUR e38-d37nowadds33rdfile/32ndresource/all31prior decoded/e37
  retained. Two focused Wine managed-native-lifecycle checks pass16fullglobal/
  context paths/raw602Baltered/fullstates-clocks/key-terminal-300slotmask/
  bus-BIOS/SWI-DMA/typed2raw2terminal crossing. ActualCLIcreate/reuse-no-write/
  oldcache and one1000Wine ordinary dispatch fullstate-image-audio pair pass
  1204sourceB(1200key/4terminal)/all300slotmask; countsinclude rereads, both
  ordinaryentrypoints notinferred. NewLinux-USA/wholeotherUIreaders pending.
  [Item contract/evidence](../rom_data_gameplay.md).
- **Audio samples, instruments, songs and sequencing.** `engine.rs` detects
  driver families; `mp2k.rs`, `gax.rs` and `rdrv.rs` shadow live voices while
  leaving original mixing/sequencing running. `MemView` resolves ROM/RAM
  ranges, and each driver has its own sample/loop/instrument contract. Driver
  detection is not an exhaustive instrument or song index. Exact EUR now has
  [original M4A root contracts](../rom_data_audio.md):3player configurations/
  36B,640directory entries/5120B,455unique headers/10080B including186aliases
  to a zero-track header,1610track roots and four bank roots. Independent
  lossless decoding/static readers/live retained-state pointers pass. Strict
  source model/457ranges/15236B and focused Wine original-reader649cases+one
  raw lookup+six synthetic variants pass;13477distinct original-read B,1759B
  retained-only. EUR e41-d40managed audio directory publication/typed CPU-DMA-
  native consumption/lifecycle pass two focused Wine checks/44fullstate-clock
  paths; productionCLI and one400-frame ordinary Wine pair pass179sourceB
  (36directory/79header/64track-pointer), three songs/headers and0player reads. There
  is no local640index guard or generic128tone bound. New producer inventory
  follows1610track roots/177047bytecode B to390bank/program pairs,429tones,
  347keymap B and277PCM waves/1931528nominalpayload B. One focused Wine check
  confirms390BDcopies/348mapped links/276extra signed interpolation endpoints
  plus one fixed8nominal endpoint. Source plan736ranges/1941759newBexcludes4B
  e41alias. Shared bounded parser/strict tone-keymap-signedPCM-CGBmodel now pass
  one Wine decoder/independent-hash/alias-dependency check and2original CGBcopy
  checkpoints. e42-d41nowpublishes audio-tones.json/37files/36resources with
  actual directory alias dependency; all35prior decoded/e41retained. Two reduced
  Wine checks pass26guards/lifecycle/all1941759B/25guest-native fullstate-clock
  checkpoints/736edges-SWI-DMA. Actual MP2Khook/tick reads typed header/PCMvalues
  for all277waves; bit-exact nonzero shadow samples/completehoststate-clock pass.
  ProductionCLIcreate/reuse-no-write39managedfiles/oldcache and one400ordinary
  Winefullstate-image-audio pair pass516956guestB/3093208hostB(1056header/
  3092152PCM), CGBordinary0. CLIdefaultshadowoff explicit; differential arms it
  after load. Partial probes are not fullmixerABI.
  New strict sequence source model identifies105253physical instructions with
  single ownership over805ranges177047B and1610directory roots. Canonical branch/
  pattern IDs and typed explicit/inherited operands preserve dynamic sequencing.
  49clockB/36originalcommandwords144B/CD8-9dispatch8B add3spans/201B:808sources/
  177248Bdisjoint from all36prior owners. One Wine model/source check passes
  allfields/hashes/27guards/404neworiginal-reader checkpoints. e43-d42nowpublishes
  sequences:38files37resources/all36prior/e42retained/actualdirectory+tonebank
  dependencies. Two Wine checks pass25guards/lifecycle/all177248B/allinstruction-
  table masks/35original-native fullstate-clock checkpoints/808edges-SWI-DMA/
  liveRAM runningoverride/rawfallback/tone-sequence guest+generic-host boundary.
  ProductionCLIcreate/reuse-no-write40files/oldcache and one400ordinary Wine
  fullstate-image-audio pair pass819B(366opcode/262operand/191clock),185source
  records includinglookahead;0command/extended/generic-host. Seven further driver
  tables370b20..370d04/484Bhave typed source model:180PCMpackedkeys/12u32bases/
  12u16samplecounts/132CGBpackedkeys/12i16bases/60noise/16volume. OneWinecheck
  passes1138neworiginalreader probes, all484B/source-JSON and15guards; sample
  selector0/13..15 and volume16/64/255aliases remain original/no addedbounds.
  Eight targeted gain returns confirm balancedroute levels16/23/31 and only
  single-side cap15; globalordinary16-domain remains unproved. e44-d43publishes/
  consumes39files38resources/all37prior/e43retained withactualsequence boundary
  owner. Two reduced Wine checks pass19guards/lifecycle/all484B/424entry bits/
  28original-interpreter-native fullstate-clock paths/mixedThumb-ARM/adjacent
  aliases/boundaries-BIOS-SWI-DMA/generic-host. ProductionCLIcreate/reuse-no-write
  41files/oldcache and one400ordinary Winefullstate-clock-image-audio pair pass
  756aggregate byte reads/67entries fromPCM-CGBkey-base tables;0samplecount/noise/
  volume/generic-host ordinaryreads remain explicit. Bank gaps, other
  wrapper roots/LUTs, language variants and references stay open.
  Offline catalog: implemented for this initial
  EUR directory/producedtone-keymap-PCM-CGB-sequence-driver scope; managed extractor: e44-d43; runtime:
  guest mixer/DMA and shadow samplers. A PCM export alone loses sequencing,
  pitch, envelopes, loops and original guest timing.
- **Executable code and opaque data.** Native labels and copied IWRAM
  instruction ranges are code, explicitly excluded from decoded assets.
  Unknown regions stay in the original ROM. Do not label a sweep of plausible
  compression headers as a verified asset catalog. Code/data interleaving,
  embedded pointers/constants and opaque regions prevent ROM-free startup.

## Version 1 base dataset contract

The managed set contains `graphics.obj-dimensions`, `graphics.bg-descriptors`
`graphics.bg-base-palettes`, `graphics.bg-alternate-palettes` and
`graphics.bg-packet-programs`. IDs remain semantic: OBJ index `shape * 4 + size`,
BG record index within the authenticated162-record block; source offsets are
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
- `bg-descriptors.json`: all162typed metadata records with exact source
  roundtrip, validated relative links/transfer triples and retained tail bytes.
  It does not contain linked graphics/program payloads. Its source range is
  independently authenticated for each profile; file hashes cannot authorize
  edited base semantics. All five resource files contribute to complete dataset identity.
- `bg-base-palettes.json`: all162record-indexed base palettes as fullu16banks,
  with source aliases preserved in162manifest ranges and a dependency on BG
  descriptor IDs. Known source codecs raw0/LZ7710are bounded and exact extents
  exclude padding. Preserve bit15and bank/index order; alternate environment
  programs use the separate verified alternate family. Exact consumed source bytes are
  retained beside banks for compatibility reads; both forms must agree.
  Decoded banks feed verified LZ77 HLE while raw CpuSet uses file-derived reads.
- `bg-alternate-palettes.json`: two157-key catalogs of eight16-wordu16banks,
 162descriptor keys including five unavailableFF, original relativeu16links
  and exact consumed custom-codec metadata. Two314Blink tables plus314streams
  retain their own manifest ranges/hashes and a dependency on descriptors.
  All fields and file hashes are authenticated; current activation verifies this
  family and its guest source reads now consume decoded literal bank bytes.
- `bg-packet-programs.json`:352unique typed programs/486descriptor links,
  parent graph, exact encoded and post-wrapper body metadata plus ordered writes.
 352source ranges depend on BG metadata. Every field/file hash is authenticated,
  with source-equal e8publication/repair/retention verified for EUR/Wine. Current
  EUR runtime consumption now retains typed values for source literals and
  predecoded HLE body writes, preserving original dynamic fill and guest timing.
  USA stays verified-only. [Contract](../rom_data_graphics.md#direct-european-packet-consumption).
- EUR `bg-main-graphics.json`:50typed graphics resources/162links, exact encoded
  metadata, destination offsets and full little-endian words. Fixed source views
  serve original CPU/DMA/native readers; literal bytes come from decoded words.
- EUR `bg-transfer-programs.json`:42controls/34shared raw tile banks/97active
  links and162absent variants,76authenticated source ranges. Current e10-d9
  publication validates the whole set and includes every file hash in identity;
  this transfer family now serves typed control/tile bytes through existing
  CPU/DMA/native readers without replacing live animation state or guest timing.
- Schema and decoder identity are separate from source identity and compiled
  code identity. For a base dataset, matching its own declared hashes is
  insufficient: verify the expected source ranges and reconstructed source
  digest against the selected exact image. User edits belong to a future mod
  layer; do not silently accept them as base data.
- Keep fixed names and bounded input:256KiB manifest (including162base and316alternate
  provenance entries),16KiB OBJ,256KiB BG descriptors,1MiB base palettes,
 4MiBalternate palettes,64MiBpacket programs and64MiBtotal. These guard malformed input, not extracted-data space targets.
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
  Normal enablement awaits05.
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
LCD A=0, LCD B=1(second alternate catalog), TV=2(first). Only Save changes Yes
commits the candidate. Four direct Thumb callers per region and the original
setter/confirmation are inventoried; no local descriptor-FF guard was found.
Global FFreachability is unresolved. See the [bounded caller contract](../rom_data_graphics.md#alternate-palette-callers-and-color-mode).
Worldmap/Party mode changes alone produced0alternate reads and cannot count as
actual alternate-palette scene coverage.
The subsequent Cyril-pub BG62transition does consume both catalogs: declared
Linux/Wine LCD B/TV full-state/audio and GUI checks pass. See the [scene results](ROM_DATA_VALIDATION.md#alternate-palette-e7-scene-and-gui-acceptance).

### Descriptor-linked packet programs

The declared field04/08/10catalog has122paired/117halfword/113auxiliary unique
sources per exact region:352disjoint ranges,398227encoded bytes,523794decoded
body bytes. Four-byte wrapper01/11selects raw/LZ77body; upper24bits are a parent
descriptor ID or FFFFFFinitial fill. All aliases/content/parent graphs match
EUR/USA, no cycles, maximum2nodes. Paired fill is live EWRAM02007f5e; other two
families fill zero. Preserve this initialization and ordered writes.
Four Linux/Wine original-loader cases each pass486wrapper invocations with
complete destination/guards/SP checks. The core's new referenced_extent bounds
raw dictionary reads separately from stream termination. See the [complete
catalog contract](../rom_data_graphics.md#complete-descriptor-packet-catalogs).
The sixth managed e8file authenticates these programs and now serves their
actual runtime source/body reads for EUR/Wine. Field00and transfer programs
have separate verified EUR contracts below; broader catalogs remain open.

### Typed packet resource model: EUR priority

The bounded offline packet model now retains352unique programs and486descriptor
aliases, ordered u16/u32writes, original parent graph and exact encoded/body
metadata. Source authentication rejects coherent base edits. Linux/Wine EUR
checks reuse all retained original bodies; readable JSON is byte-identical.
The new model is verified for EUR only; USA remains pending by user instruction.
See the [typed contract](../rom_data_graphics.md#typed-descriptor-packet-programs).
The file is published/verified in the e8-d7managed dataset and consumed by the
EUR CPU/DMA/native/HLE paths. Three focused Wine checks and one397frame actual
native transition pass with positive typed consumption and complete-state parity.
New Linux/USA checks remain pending. See the [consumer contract](../rom_data_graphics.md#direct-european-packet-consumption).

### Main graphics field00: verified EUR offline catalog

The162descriptor links name50disjoint sources:23tag20,27tag22; exact consumed
sizes total621923bytes and decoded outputs835232bytes. Staged producer0801b688
phase1selects custom BE-size LZ/RLE080054a0, the same original decoder used by
alternate palettes. These are not BIOS Huffman streams. Tag20's upper24bits
declare output bytes; tag22's upper24bits are a byte destination offset followed
by a nested20header and the same custom stream. Inner/outer sizes agree.
All outputs/offsets are whole32-byte tiles and fit the original0x4fe0-byte
graphics destination. Phase1writes the compact output to EWRAM020091a0;
phases2/3queue DMA32 transfers of each half to target+offset(+half for phase3).
Tag22does not supply or replace the live destination prefix.
The normal producer at0801b2cc reads field00at0801b310..324 but branches for
12/02/10and a copy case; it alone does not prove how20/22are prepared.
The separate palette-copy indirect pointer atROM370eb4 selects IWRAM03005ee9;
08145c14is only the bxr3trampoline. It is not an independently located main
graphics decoder.

ROMDATA-01-BG-MAIN-GRAPHICS / ROMDATA-02-BG-GRAPHICS-DATA are complete for
this declared EUR catalog. The bounded typed model retains little-endian words,
offsets, exact compressed metadata and all alias links; canonical IDs follow
first descriptor occurrence. Validation rejects truncations, invalid references,
overruns, overlap, wrong IDs, size mismatch and coherent source edits.
Nine focused Wine checks pass, including50original staged decoded outputs with
exact live source cursors, full destination/guards/SP, and phase2/3DMA layout for
one20and one22source:54calls/11142126instructions. Readable JSON roundtrip and
base authentication pass. Evidence:target/rom-data-bg-main-graphics/
windows-europe/result.json, windows-results.json and bounded disassemblies.
Managed EUR e9publication and direct file-backed runtime consumption are now
complete for the declared Wine scope: retained words/metadata/literal maps,
fixed769..895source pages, original guest instructions/clock and separate
positive-use counters. Four poisoned-source original/native cases and owner
lifecycle pass, followed by one397frame full-state/image/audio pub transition
and actual CLI parity/no-write reuse. New Linux/USA verification remains pending.
[Current consumer contract](../rom_data_graphics.md#direct-european-main-graphics-consumption).


### EUR descriptor transfer and variant programs

The five transfer slots have83/6/5/3/0active links across162descriptors:
97links,42control sources and34shared raw tile banks. All162variant links are
zero in this verified block; no absent pointer is dereferenced or treated as
an invented fallback.76disjoint source ranges total968control/199296tile bytes
and do not overlap existing e9resource ranges. Controls store u16transfer byte
length, u8frame count plus retained byte3, and ordered u16duration/source-tile
pairs. Source offsets are tiles of32bytes. Frame/tick/DMA progress is live RAM,
not extracted state. Tile banks retain full little-endian u32words and exact
maximum referenced extents across shared aliases. Independent stable IDs follow
first descriptor/slot occurrence; typed links preserve VRAM destinations.

Original creator08021f10, updater08021bd8and dispatcher08021d10confirm the
contract in four reduced EUR/Wine animation cases, using retained slot5only for
original IWRAM helper initialization. Two loops plus boundary match frame
countdowns/order and complete VRAM after every DMA chunk, including34partial
chunks with the original1024-byte limit:634calls/80766instructions. Five focused
Wine checks pass (3core/1JSON/1private), source/JSON authentication and all97links
match the independent static classifier. New Linux/USA checks remain pending.
Managed transfer publication is now complete for EUR/Wine under1/e10-d9,
with the eighth fixed JSON file and all76ranges. Four focused Wine checks and
three actual CLI commands pass authentication/publication/reuse/repair/move/
retention and transactional activation. Direct control/tile source consumption
now passes reduced EUR/Wine provenance/lifecycle and declared397frame scene/CLI
proof: all76raw spans poisoned in two original/native cases, full states/clocks
equal after60calls,66compiled controlB/9984native-DMA tileB. The actual pub scene
uses1203controlB/21504tileB with every state/image/audio sample equal. New Linux/
USA and broader scene/GUI/archive checks remain pending.
[Contract and evidence](../rom_data_graphics.md#european-background-transfer-programs).


### EUR adjacent lookup and five trade parameter records

ROMDATA-01-GAMEPLAY-ADJACENT-HALFWORD-LOOKUP-ROOTS is partial:16literal references
root6e9cb6, next independent6e9d8a, but the natural106halfword domain is unproved.
Fixed64/67 and bounded lists16..20/21,22,4,23 do not establish all UI producers.
No model or ownership is inferred from values or neighbor extent.

ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE96-ROOTS is complete for the declared static
contract: selectorD0744 explicitly roots five96B records, IDs2..6,480B at
6e85bc..6e879c. Independent next root and biased price source agree. D2728's30
live RAM rows bound the internal count0..30, not a30-record ROM bank. Strict
source model retains64opaqueB/u16markup40-41/30discountB42-5f per record;
count0 aliases markup byte41. D26AC retains original separate70B-source D1F30
property36/zero skip/live flags/unsigned divide/wrap arithmetic. Windows73640
warning-free/Wine52757passes499real+160edited-source+15separate injected operand
cases; all160price-fieldB observed/320opaqueBroundtrip-only/fullRAM-SP-callee
and strict source/JSON guards pass. e38-d37 is unchanged; managed/runtime/
ordinary and newLinux-USA-nativeWindows remain pending. [Contract](../rom_data_gameplay.md#trade-parameters-source-contract).


ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-TRADE-PARAMETERS-CONSUMPTION
are complete for declared EUR/Wine: e39-d38/34files/33resources/all32prior/e38
retained/USAe8-d7. Windows21694warning-free; Wine20164two checks pass managed/
native/lifecycle/strict source bindings and28full paths/raw480Baltered/fullstate-
clock parity, all480B/mask31/fieldedges/rawgap222/SWI-DMA/priorowner coexistence.
Compiled opaque/markup-low/shared41/discount0-5-10-10, no opaque original price
read inferred. Source499+160+15closed; CLI/ordinary/newLinux-USA-nativeWindows
pending. [Contract](../rom_data_gameplay.md#managed-and-direct-trade-parameters).


ROMDATA-05-GAMEPLAY-TRADE-PARAMETERS-SCENE-CLI is complete for declared EUR/Wine:
production46740warning-free/canonicalrestored; actualCLI17191create/reuse-no-write/
oldcache/unrelatedcwd with36managedfilehash-mtime guards. First400existingpre-Yes
fee acceptance locator uses14sourceB(markup-low7/shared417),mask1physicalID2;
rereads included. One Wine89539full400pair matches every state-clock-image-OAM-
dispatch/mixed-PSG-FIFO-HLEaudio sample, same source counts. FinalPublist/7109funds
image inspected. Source499+160+15/native28/managed/lifecycle/e34threshold400/e38
proofs remain closed. NewLinux-USA-nativeWindows/ordinary discounttail-opaque-
all5records/othercallers remain pending. [Evidence](../rom_data_gameplay.md#ordinary-trade-parameter-fee-acceptance).


### EUR fourteen packed item effect parameter records

ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE7-ROOTS is complete for the declared static
contract:11literal readers/14×7B/98B6e879c..6e87fe, existing itemIDs362..375
subtract362 and independent next root agree; no e39overlap. Nine5-bit
coefficients,8-bit cap and3opaque bits roundtrip and match split-byte getters.
CFE28property/indexlow8 and CFDA8fullu16item-minus362 cap index remain distinct;
live current/quantity/cap clamp stays original. UI cap>99 branches and arbitrary
indices remain unverified/raw. Source model/oracle is now complete: Windows94173
warning-free/Wine94185passes540real+4explicit raw-cap aliases+166synthetic cases,
all98B/read-onlyfullRAM/SP-callee/strictJSON-bitwidth-itemowner-source guards.
Managed/runtime/ordinary/newLinux-USA-nativeWindows pending; e39-d38 unchanged.
[Contract](../rom_data_gameplay.md#item-effect-parameter-static-contract).


ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-ITEM-EFFECT-PARAMETERS-CONSUMPTION
are complete for declared EUR/Wine: e40-d39/35files/34resources/all33prior/e39
retained/USAe8-d7, actual parsed item owner dependency. Windows93110warning-free/
Wine21177two checks pass50full original-interpreter-native paths/raw98Baltered/
complete state-clock equality, all98B/14recordmask16383/7Brecord-fieldedges/BIOS/
SWI-DMA/adjacenttrade2-itemeffect2crossing/publication-retention-lifecycle/strict
source guards. Compiled coefficient/shared05/shared06bytes36/37/23. Source540+4+
166not repeated; CLI/ordinary/newLinux-USA-nativeWindows/fullUI remain pending.
Existing DS22populatedbank proof is USA-only, not EUR evidence.
[Contract](../rom_data_gameplay.md#managed-and-direct-item-effect-parameters).


ROMDATA-05-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SCENE-CLI is complete for declared
EUR/Wine: production4620warning-free/canonicalrestored; CLI59404create-reuse-no-write
37managedfilehash-mtimeguards/50707oldcache from unrelatedcwd. New ordinary guest
input from retained EUR IN13reaches CyrilBank/List/ownedGoblinLv4/Play. Existing
pre-Yes Cyril state supplies measured120,216/place18; earlier wrong-town/guessed
longhold navigation trials are closed, not rerun. One Wine5779full400pair matches
every state-clock-image-OAM-dispatch/mixed-PSG-FIFO-HLEaudio sample, positive3B
(coefficient1/shared051/shared061)/mask1record0. Ordinary mode is mixed native/
interpreted; source/native/earlier ordinary proofs stayclosed. FinalCLIseparateUI
panels absent as previously known, no food-count claim. Other14item/20monster/
cap>99UI/newLinux-USA-nativeWindows checks pending.
[Evidence](../rom_data_gameplay.md#ordinary-item-effect-parameters-in-the-european-monster-bank).

### Reclassified palette-effect sources (24 October 2026)

Roots370eb0/b94690 formerly queued as FFTA audio are not audio resources.
370eb0 is a copied-IWRAM zero-clear/copy/fill helper vector and stays executable
support metadata. b94690is palette-effect dispatch: state1/readu16kind/stride4,
fixed constructor kinds1/2/4/8/16,17word68Bprojection including12nullholes.
No liveRAM max16guard/fulltableextent is inferred. All38e44owners are disjoint;
static7literal/5producer/opcode/hash checks pass without guest execution.
[Source contract](../rom_data_graphics.md#palette-effect-dispatch-source-contract).
Model/sourceWine/publication/runtime/ordinary remain pending; audio migration
of these false leads is closed. Other unselected audio/graphics/families stay open.
Typed17slot/68Bpalette dispatch model nowretains canonicalu16kinds/guestThumb
code offsets/nulls with strict shape/source authentication. Windows71666warning-
free/Wine22360one focused check passes31original prefixes/all68B/12guards/
low16liveRAM/rawoutside/state gates/predictedproducerstores.1216protected inputs
unchanged; no fullcallback/native/stateclock/ordinary claim. Managede44unchanged;
publication/runtime/ordinary/newLinux-USA-nativeWindows remain pending.
e45-d44managed palette dispatch nowpublished/consumed:40files39resources/all38
prior/e44retained/USAe8-d7. Windows67564warning-free/Wine46138two reduced checks
pass12guards/publication-retention-lifecycle/all68B/17slotmask/10original-native
fullstate-clock prefixes/24callback-4nullcompiledB/livekind/rawaliases/BIOS-SWI-
DMA/boundaries. No fullcallback ABI; actualCLI/ordinary remains pending.
Actuale45productionCLIcreate/reuse-no-write42files/oldcache nowpassWine56076;
neutralboot240closed0use/no fullpair. New normal English-confirm1000CLI60304/
oneWine35467fullstate-clock-image-OAM-dispatch-audio pair pass5944aggregate
callback-byte reads/kind1only(mask2)/null0. Othercallbacks targeted-only; no
RAM/statepayload/save edits. Source31/native10/oldfamily proofs not repeated.
Paletteparameter/program producers and otherfamilies/newLinux-USA-nativeWindows
remain open; no performance/space measurements.

### Palette-program producers: partial inventory (24 October 2026)

Actualpalette task080d7fac/public080d8214-080d8234 has308directBLcandidates;
285exactargument expressions root179programs.3fixed-count8Blist sources add5,
2explicit3wordstack copies add1; saved-R4bridge reuses a knownroot. Combined185
programs/1000commands,16135candidateprogram B/124ranges plus88producerB, all16223
disjointfrom39e45owners. Six-byte header/countu8 and6Bcommand/u16relativeoffset
contracts distinguish direct/queued colors and minimum1countedcolor sequences.
20indexed producers now have explicit task-ID/message/constant/counted-loop
projections:97cells388B add45programs, total230programs/1135commands/18221programB
plus476producerB=18697B/165ranges, disjoint39e45owners. All185earlier records
retained; mixed39bcc4owns2pointers only. These are no globalRAM guards/fulltable
extent claim. SourceWine/model/publication/runtime/ordinary pending, e45unchanged/
no newguest/build/tests or oldrepeat.
[Contracts](../rom_data_graphics.md#palette-program-producer-inventory).


### Palette programs: source model verified (24 October 2026)

Strict typed230programs/1135commands/25producer tables reconstruct165ranges/
18697B; canonical IDs/raw fields/list delays-reserved/null slot preserved.
Windows warning-free/oneWine96989check passes all18221programB and18681distinct
allB at original Thumb readers;16reserved list B are preserved/unread.97indexed
cells/8listrows/2stacktables,6wrap-clamp copy inputs/3mincount-period/1rawdirection
probe and16shape-JSON-source guards pass. Earlier1645artifacts/newtestexe unchanged.
This closes the source oracle/model child, not fullallocator/callback/native/
stateclock/ordinary. e45-d44unchanged; managedpublication/runtime/ordinary next.
[Source model](../rom_data_graphics.md#palette-program-source-model-and-original-reader-validation).


### Palette programs: managed runtime (24 October 2026)

EUR e46-d45publishes41files/40resources, all39prior decoded/e45retained/USAe8-d7.
165bindings18697Bserve typed header/command/parameter/producer fields; no guest
sequencer/index/save/native ABI change. One Wine lifecycle check passes12guards/
publication-create-reuse-repair-move/missingfile-Load-Undo-backup-Restart/all18697B/
230programs25tables. Native-only check passes24original-interpreter-native full
state-clock prefixes, generated API[16header52command66parameter12producer]B,
rawcommand65535/coloroffset65534/index2/widths-mirrors-BIOS-SWI-DMA. Two testboundary
failures retained and fixed via prefix guard respecting native chaining/events;
production emitter unchanged/source-managed checks notrerun. ActualCLIcreate/
reuse-no-write43files/oldcache pass. English-confirm1000newprogram-use0/closed
negative/no fullpair.10protected worldmap-bank snapshots have no newpalette task.
Actual ordinary EUR Montblanc Richiamo/Pecorelle CLI and one1000-frame Wine
fullstate-clock-classic240x160-image-OAM-dispatch-audio pair pass170sourceB/
4programs (32header/90command/48parameter),0producer. Generic typed bus-or-host
consumption guard corrected; selected palette assertion unchanged. Other226
programs/producer tables retain targeted-only coverage. NewLinux-USA-nativeWindows
and new32:9 scene checks remain open.
[Consumption](../rom_data_graphics.md#managed-palette-programs-and-typed-guest-reads).


### Blend programs: bounded producer inventory (24 October 2026)

The neighboring080d77c4task/080d79ac-080d79d0constructors control BLDCNT,
BLDALPHA andBLDY through the compact PPU shadow; they are not e46palette roots.
All248retained direct calls are classified:235constant/2conditional/6indexed/
1inline/1mutableRAMtemplate/3dynamicRAM. Current CFG-block validation rejects
the false080e2ec8code root; that caller copies10B from0839a5f0then overrides its
RAMduration. Six bounded projections supply22pointer cells; no global RAM guard
or complete physical table extent is inferred. Inline0/1variants follow explicit
message-ID branches; primary and secondary descriptor constructors are distinct.

160programs/345commands own2340B(960header/1380command), plus88producerB:
2428B/158ranges, disjoint all40e46owners/no repeated logical byte. Header6B is
u16control/coefficients/count, command4B is u16duration/u8first/u8second. Static
independent reconstruction/allhashes/44consumer opcodes/DMA register mapping
pass.135alpha/20brighten/5darken;5instantffffcommands. Typed model/sourceWine/
publication/runtime/ordinary remain pending; no new guest run/build/test or
performance campaign. EUR e46-d45/USAe8-d7 remain. [Source contract](../rom_data_graphics.md#blend-program-producer-inventory).


### Blend programs: typed source model (24 October 2026)

Strict lossless `rom_data::blend_programs` reconstructs all160programs/345commands/
6tables22pointers/2428B158ranges, preserving packed reserved fields and raw
u16duration/u8targets/canonicalreferences. Exact EUR identity/110reader-producer
fingerprints, strict JSON/order/count/bitwidth/reference/overlap/source guards.
Windows12341warning-free; oneWine94935source check passes160headers/345commands/
22cells/5instantchains/templatecopy-RAMoverride/2packedheader/6duration/3delay/
index65535rawprefixes.2376sourceB observed;52nonalpha timed second fields remain
preserved/hashchecked only for these initial-mode probes.17shape/3source-edit/
2JSON/2ROMidentity guards pass;52943instructions264PCs arecoverage, no timing.
Managed/runtime/native/fullclock-ABI/CLI/ordinary/newLinux-USA-nativeWindows stay
pending; e46-d45 unchanged. [Model and scope](../rom_data_graphics.md#typed-blend-program-source-model).


### Blend programs: managed runtime (24 October 2026)

EUR e47-d46publishes42files/41resources/all40prior decoded/e46retained/USAe8-d7.
158bindings2428Bserve typed header/command/producer fields on13pages; no guest
sequencer/index/save/native ABI change. Reduced Wine checks pass12guards/
publication-retention-lifecycle/all2428B/160program6tables and20original-native
complete-state/clock prefixes. Compiled reads[34header70command16producer]B;
source modes/wait-instant-delay/templatecopy/live2→1mode/rawcommand65535/index3/
previouspaletteindex5/width-mirror-BIOS-SWI-DMA/new2raw2/new2old2 pass.
1034instructions/295PCs/93dispatches arecoverageonly. Initial compile argument/
import fixes retained; corrected source-set coherence separately passes a new
focused Wine negative guard, without closed native/lifecycle repeats. No whole
callback/allocator/scheduler ABI. ActualCLIcreate/reuse-no-write44managedfiles/
oldcache and one1000Wine ordinary EUR Pecorelle fullstate-clock-classic240x160-
image-OAM-dispatch-audio pair nowpass18aggregateB (10header/8command),1program,
0producer; other159programs/tables targeted-only. NewLinux-USA-nativeWindows/
new32:9 scene checks remain pending. [Consumption](../rom_data_graphics.md#managed-blend-programs-and-typed-guest-reads).

## Bounded European task descriptor sources

Three adjacent catalogs contain341primary/20auxiliary/51script records at3997b0/
39a258/39a2f8:412records3296B/3spans, disjoint41e47owners. Original constructors
080d7018/080d7328 use different ID formulas; nullableThumb callbacks/contextsize/
rawmetadata are distinct. Independent graphics source39a490and directory kind0/
ID413reference at3aca2c establish the suffix endpoint39a488; no global live ID cap
or whole callback ABI.53BLcandidates retained; unresolved CFGs are not proof of
absence. Static independent roundtrip/hash/59halfword12word contract checks pass.
Strict typed model now passes one reduced Wine source oracle: all3296Breconstructed/
3156Bobserved,140suffix metadataBpreserved-unread,415main/59secondary/346halfword/
346byte/18low16/9branchprefixes and23shape-source-JSON-identityguards. Warning-free
Windows build;7191instructions39PCs coverageonly. e48-d47managedruntime nowpasses
13guards/lifecycle/all3296typedB/412records/3catalogs and29original/native fullstate-
clock prefixes, generatedAPI44callback22context9metadataB, raw/aliases/oldblend
owner/boundaries-BIOS-SWI-DMA.43files42resources/all41prior decoded/e47retained;
USAe8-d7unchanged. ActualproductionCLIcreate/reuse-no-write45files/oldcache andone
1000Winefullstate-clock-classic-image-OAM-dispatch-all-audio pair nowpass28aggregate
sourceB/4records2catalogs (16callback8context4metadata). Other408records/thirdcatalog
targeted-only/new32:9pending. Source/model/runtime/CLI/ordinary children closed;
no wholeallocator/callback/schedulerABI or newLinux/USA claim. [Contract and scope](../rom_data_tasks.md). Linux/USA remain pending; no new
performance/space measurements.

## European effect/event producer directories

The bounded source contract covers75event16Brows/1200B,51selector4Brows/204B and
22+6evenThumbsame-state dispatchwords/112B:1516B/four spans disjoint42e48owners.
Original increment/compare74andindependent6Bnextcatalog3acebc boundevents; separate
u3lookup3ad240bounds selectors. No countfromshape or51taskdescriptors.19event+
51selector logical links resolve toengine.task-descriptors. Originalrelative
jumps/u16index/q0underflow/secondaryIDwrap/defaultbranches staylive. Source-model
strictJSON/92halfword8word guards/independent reconstruction andoneWineoraclepass:
77header77delay53selector43param32dispatch19frontdoor-low16-increment prefixes,
1106Bobserved/300eventraw110selectorparamBpreserved-unread,27guards.2582instructions/
70PC coverageonly; no fullinterpreter/native/stateclock/ordinary/reachabilityclaim.
Managed e49-d48integration nowpasses13guards/lifecycle/all1516typedB and29original-
interpreter-native fullstate-clock prefixes/[24event6selector24dispatch]API B,
raw/defaults/evenstate/previousfont11B/samepagepalette4B/BIOS-SWI-DMA.44files43resources
retainall42prior decoded/e48; realtaskdescriptor dependency. ActualCLIcreate/
reuse-no-write46managedfiles/oldcache passes. Pecorelle1000/schoolcoldboot3000/suspendboot-to-battle8476
locators readzero newbytes: closednegative/no event-requiredfullpairs. New normal
EUR boot/name/snowball/street/bedroom progression yields19genuine snapshots;
one1800-frame book locator/fullpair from39420 on actual e52generation passes
allfullstate-clock-image-OAM-dispatch-audio with5aggregateB=1selector+4dispatch,
one selector/one table,470hash-mtime guards. Ordinaryeventrows0/other50selectors/
other dispatch table targeted-only, entry124e1creachability unknown.
USAe8-d7unchanged/Linux-USA-nativeWindows/new32:9 pending/no performance.
[Contract and limits](../rom_data_events.md#positive-ordinary-book-transition).

## European script command directory

The next independent EUR source projection contains 115 six-byte commands,
690 bytes at3acebc..3ad16e. Three original literal roots and byte-assembled
callback/advance readers are verified. Two independent byte-lookup roots at
1263c4/126468 bound the end; an initial 116-row candidate ending at3ad174 was
rejected before the source plan/model because it included that distinct lookup.
All 115 callbacks are unique non-null Thumb BX entries; no asset dependency.
The projection is disjoint from all 43 e49 owners. Original u8 opcode arithmetic
has no new cap, and signed stream cursors/u16 advance wrap/relative targets/
return0x110 loop remain live. Strict model/hash roundtrip/106 halfword and six
word fingerprints and one corrected Wine oracle pass all 690 observed bytes,
117 callback/117 advance/115 BX/four signed-cursor/ten relative-target/five loop
prefixes and 19 guards. First Wine failure was an ADDS expected-register mistake
in the test, corrected without a production/model change. 2905 instructions/
67 PCs are coverage only. Managed e50-d49 nowpublishes45files44resources/all43
prior decoded/e49retained/USAe8-d7unchanged. Two reduced Wine checks pass13guards/
lifecycle/all690B and26original-interpreter-native fullstate-clock prefixes with
14advance/28callback API B. Raw115/255/signedcursor/wrap/relative-targets/BL-BX/
returnloops/adjacent previousowners4+4B/mixed previous2new2/new2raw2/BIOS-SWI-DMA
agree. ActualCLIcreate/reuse-no-write47managedfiles/oldcache andone8476-frame Wine
EURsuspendboot-to-battle fullstate-clock-image-OAM-dispatch-audio pair pass
42B=14advance28callback/7commands. Other108commands targeted-only; new32:9/Linux-
USA-nativeWindows pending.
[Contract, evidence and limits](../rom_data_scripts.md).

## European relative script archive and event keys

Bounded static inventory identifies115464B/two disjoint ranges:114528Barchive at
B946EC..BB064C and936B/234keys atD68C2C..D68FD4. Independent localized-text and
coordinate-index roots bound both ends. Keydomains/first relative offsets agree
with62topbanks/233nestedslots (248+932indexB).213unique targets/20repeated links
include5headeralias links and2FFfill links.207nonempty physical command segments
contain35048storage instructions/110opcodes/78212parameterB;45FFfillspans88B remain
readable.493command5alinks target49existing effectselectors; actualdependencies
areengine.script-command-directory andengine.effect-event-directories. Keybyte2
selects one of five unmigrated localized text archives; byte3raw is preserved.
Independent reconstruction/234keys/233nonnulltargetformulas/128halfword5wordfingerprints/
all115464B/disjoint44owners pass without guestexecution. First builder's empty-
program assertion exposedtwofilltargets; correctedbeforeplanwritten. Notlinear
CFG/wholeargument semantics/reachability/newguestcaps. Later model/runtime
results below cover separate criteria; inventory left e50-d49/USAe8-d7 unchanged. [Contract](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract).

The whole115464Barchive/key model now has strict lossless Program/Header/Fill
references and complete physical reconstruction, noROMshadow/aliascopies. Actual
command/effect dependencies validate lengths and493selector links. Windows91283
warningfree/oneWine85436 first-pass verifies238keyaddress/234two-level/234textbank/
35054opcode/2397relative-target/493selectorparam prefixes,42333ownedBobserved,
73131Bpreserved-unread and40guards.224382instructions62PCcoverageonly; notwhole
interpreter/native/fullstate-clock/ordinary. Model leaves e50-d49/USAe8-d7 unchanged,
source-model scope only; managed/runtime results follow. [Model evidence](../rom_data_scripts.md#typed-relative-archive-model-and-reduced-wine-source-oracle).


Managed e51-d50 adds script-archive.json/engine.script-archive,46files/45resources,
all44prior decoded files/e50 retained, USAe8-d7 unchanged. Real command/effect
asset dependencies and two typed source bindings/115464bytes authenticate before
activation. Two reduced Wine checks pass13guards/lifecycle/allbytes and28original/
interpreter/native complete-state/clock prefixes. Compiled API categories read
37index/4opcode/7parameter/12key/1fill bytes, preserving raw IDs/unaligned rotation/
header-fill views/previous palette owner/boundaries/BIOS-SWI-DMA. Production CLI
create/reuse-no-write48managedfiles/oldcache passes; retained8476-frame EUR
locator reads27B=8index7opcode10parameter2key, one bank/program/key, no fill.
One Wine8476-frame pair from the actual CLI generation passes all full states/
clocks/central-wide images/OAM/dispatch/mixed-PSG-FIFO-bitwise-HLE audio, same
positive archive use and366hash-mtime guards. Other61banks/206programs/233keys/all45fills
retain targeted coverage. No new32:9/wholeABI claim; newLinux-USA-nativeWindows
pending. [Runtime evidence](../rom_data_scripts.md#managed-relative-archive-and-typed-guest-reads).


## European script-selected localized text

Five language roots BB064C/C076F0/C5FBF0/CB7C48/D10644, selected by08009ab0,
independently bound1803744B through the next root/D68C2Cscript-key root.63top
entries per language agree with key-bank maximum62/minimum offsetFC; first word
is a shared late bank0, not a length. Eleven repeated normal links per language
and26root-relative bank61variants resolve to77physical banks per language.
Live byte division by10selects the variant; a distinct remainder helper adds
24times the remainder to the u16message. Full-u16 and low-byte special-message
branches, raw key IDs, fallback language/key and live state remain original.

385banks/41395u16index entries including385sentinels name41010programs. Existing
Program/customLZ grammar gives9813compressed extended/5709raw extended/25488raw
compact programs,212011commands/11forms,147glyph IDs1..3112 already in the font
asset; actual dependencies script-archive/font.complete-bank. The whole source
partition is1260top+520variant+82790bank-index+1684407program+34767alignment B.
34314gaps are1..3B; four mixed gaps reject a uniform-fill assumption before any
model/runtime/Wine. Exact bytes remain explicit, without inferred semantics.

Independent static whole-source reconstruction/hash/15195key-variant formulas/
349halfword13word guards/actual dependencies/glyphs/disjoint45e51owners pass.
No guest execution/build/test or prior proof repeated. Typed model/Wine/runtime/
CLI/ordinary pending; EUR e51-d50/USAe8-d7 unchanged. [Contract and evidence](../rom_data_text.md#script-selected-localized-text-source-contract).


The strict whole-source script_text model now shares physical banks through
typed top/variant references and reconstructs all relative/u16index words,
Program codec bytes, sentinels and exact slack without alias payload copies.
Actual key/font dependencies validate. First Wine strict-JSON failure exposed
unit-variant extra-field acceptance; corrected to empty struct, kept assertion
and all first-attempt evidence. Corrected Windows55123warning-free/single new
Wine34745passes all1803744B/independent fields/hashes and22shape/6source-edit/
3dependency/12JSON-shape/2identity guards. Original7language/1170key-bank/
1280live quotient/256live remainder/90raw-special/1170u16index/837header/
837source-vs-typed output cases pass, including249compressed bodies/tails.
2116900instructions/834PCcoverage only; not native fullstate-clock/ordinary/whole
ABI. Source model closed, publication/runtime/CLI/ordinary remain pending;
e51-d50/USAe8-d7 unchanged, new Linux/USA/native Windows pending. [Model evidence](../rom_data_text.md#typed-script-selected-text-model-and-reduced-wine-source-oracle).


Managed e52-d51 now publishes script-text.json/text.script-archives,47files/
46resources/all45prior decoded/e51retained/USAe8-d7, with actual key/font deps.
Five typed bindings1803744B reconstruct all index/header/control/decoded program/
slack fields and merge previous owners/raw bytes on BB0/D68. Shared literal map
keeps old categories; one affected core Wine check/new managed-native filters
pass13guards/lifecycle/allbytes/all masks and35original/interpreter/compiled
full-state-clock prefixes, including raw selectors/wrap/sentinels/width-mirror-
realBIOS-CpuSet-FastSet-DMA. Direct tail literals0is exact, generated tails remain
preserved. Real old0/1/2source163B also pass.379instructions/65PCs/71dispatches
coverage only. Production CLI create/reuse-no-write49managedfiles/oldcache passes;
8476battle locator is negative0text/no fullpair, distinct school3000positive
reads289B/one language/bank/two programs. One Wine3000pair from CLI generation passes every full state/clock/image/OAM/
dispatch/mixed-PSG-FIFO-bitwise-HLE sample, same source use/468guards;
other records/variants/slack targeted-only, newLinux-USA-nativeWindows pending.
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
