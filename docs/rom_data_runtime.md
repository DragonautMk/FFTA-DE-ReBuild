# Direct decoded resource consumption

The current E170-era Windows GNU `recomp` test executable passed its complete
non-ignored `rom_data::` filter under Wine (`e170-rom-data-synthetic-regression-
cpyhx1cy`): 31 passed, zero failed, and 451 ignored. The executed synthetic
checks cover strict formats, publication/reuse/relocation, corruption and
partial-write recovery, cancellation, competing publishers and portable paths.
They do not load the E170 dataset or replace private-ROM integration filters;
Linux, USA, native Windows and performance remain outside this result.

An additional EUR/Wine regression loads the authenticated E170 dataset and
checks all 74,225 distinct boundaries of its 54,792 declared source spans
(`e170-all-source-boundaries-wine-yeg0bdt_`). With every backing-ROM byte
inverted, 445,350 probes immediately at or before those boundaries across
mirrors 08/0a/0c verify that bus and native API halfword/word reads match the
four byte reads and that mirrors agree. The active bounded-range owner serves
96 bytes during the check. Both original and inverted backing values occur;
the counts include repeated observations and are not an ownership inventory.
The result covers sampled boundary composition, not every source byte, guest
execution, timing, or a scene. Inputs were protected, scratch removed, and
Wine drained.

EUR/Wine `e170-all-source-bytes-wine-ffhtpvms` also scans all 10,948,338
distinct bytes in the 54,792 declared E170 source spans with every backing-ROM
byte inverted. All returned original-valued bytes; 43,793,352 byte reads cover
three bus mirrors and the native API. Another 8,186,274 checks compare aligned,
fully covered word reads with byte composition. A separate uncovered byte
returns the inverted backing value across the mirrors and native API, and the
bounded-range owner reports 704 served bytes during the scan. Byte values do
not attribute individual reads to resource owners. This does not cover guest
execution, arbitrary unaligned interior reads, timing or other platforms.
Protected inputs passed, successful scratch was removed and Wine drained.

EUR/Wine `e170-unaligned-source-wine-8_1wpiix` checks every odd halfword
and every non-word-aligned word whose bytes are all inside the union of the
E170 manifest's source spans. With the complete backing ROM inverted, 5,459,217
halfword offsets and 8,159,411 word offsets match the original ROM bytes
through both the bus and generated-code API in mirrors 08/0a/0c: 81,711,768
read comparisons. This includes fully covered accesses crossing adjacent
source spans. Protected inputs passed, Wine drained and scratch was removed.
The union defines coverage, not individual resource attribution. Mixed
covered/uncovered accesses beyond the earlier boundary samples, guest CPU
instruction behavior, timing, scenes and other platforms remain outside this
check.

EUR/Wine `e170-all-resource-lifecycle-wine-i73r4f54` checks all 10,948,338
distinct source bytes of the 113-resource E170 manifest after each of five
synthetic operations: failed attachment from a missing replacement path,
slot Load, Undo, backup restoration and restart. Each pass returns the original
bytes with the in-memory backing ROM inverted; the range and conditional-word
owners both record reads in every pass. The same owner and expected complete
machine state survive each operation, and the protected dataset and ROM remain
unchanged. This checks direct APIs with synthetic saves; it does not remove the
active dataset, exercise GUI or Fast, execute guest code, or attribute every
individual read to a particular resource. Native Windows, Linux and USA are
outside the result. Successful scratch was removed and Wine drained.

EUR e88-d87 adds [event records](rom_data_gameplay.md#managed-event-records-and-typed-runtime-reads):
81files/80resources, one3072-byte span/no dependency, all79 previous records/
payloads exact. The shared row225 view retains the same owner; original script/
event logic and uncapped outside reads remain guest work. One focused Wine
filter passes publication/interruption,14attach/two full-set guards,22complete-
state/clock native cases, all3072bus bytes/eight classes/256 records/borders,
Load/Undo and three DMA/SWI transfers.296 served new bytes include two through
interp_one fallback; this is not a distinct-byte union. [Actual CLI create/reuse/startup](rom_data_gameplay.md#actual-event-record-cli-registration)
now passes81identical payloads/80IDhash83unchanged managed paths; one neutral
frame has all11 counters zero. [One ordinary mission-entry34 suffix](rom_data_gameplay.md#ordinary-event-record-mission-entry-equivalence)
now passes every complete state/clock/central-wide image/OAM/five audio stream
with4reads6served key/primary/alternate bytes/one record and exact11CLI/pair
counts. Wider ordinary fields/records and Linux/USA/native Windows remain
pending. No performance measurement.

EUR e87-d86 adds [selection-byte tables](rom_data_gameplay.md#managed-selection-byte-tables-and-typed-runtime-reads):
80files/79resources, two4/30-byte spans/no dependency, all78 previous records/
payloads exact. The immutable owner serves both arrays while count, raw live
eligibility, numeric helpers and selection remain guest instructions. One
focused Wine filter passes publication/interruption,14attach/two full-set guards,
20complete-state/clock native cases, all34bus bytes/4+30positions, raw gap/prior
composition, Load/Undo and three DMA/SWI transfers. [Actual CLI create/reuse/startup](rom_data_gameplay.md#actual-selection-byte-cli-registration)
now passes80identical payloads/79IDhash82unchanged managed paths; one neutral
frame uses none of the new source. [One ordinary travel33 suffix](rom_data_gameplay.md#ordinary-selection-threshold-travel-equivalence)
now passes every complete state/clock/central-wide image/OAM/five audio stream
with one served threshold byte/one position and exact six CLI/pair counters.
Weights and other positions remain targeted-only; Linux/USA/native Windows
remain unverified. No performance measurement.

EUR e78-d77 adds [world nodes](rom_data_maps.md#managed-world-nodes-and-typed-guest-reads):
71files/70resources, all69prior records/files retained, USAe8-d7 unchanged.
The600-byte typed owner supplies coordinates, ordered path/neighbor links and
opaque fields while keeping original placement/pathfinding and raw/RAM outside
behavior. One reduced Wine filter passes publication/12attach/2fullset guards,
15complete-state-clock native prefixes, all600bus bytes and three transfer/
Load-Undo checks. [Actual CLI creation/no-write reuse and normal16-frame use](rom_data_maps.md#actual-world-node-cli-and-ordinary-coordinate-use)
now pass71identical generation files/70runtime IDs/73unchanged managed paths,
with402coordinate bytes/22nodes. That16-frame zero path/neighbor bound stays
closed. The [later ordinary travel360case](rom_data_maps.md#ordinary-world-node-travel-and-caller-gates)
now passes all states/clock/images/OAM/dispatch/five audio streams with6841B
(4820coordinate/864path/1157neighbor),29nodes and exactCLIcounts. Opaque use0
remains targeted-only. No performance measurements or old acceptance replay.


EUR e71-d70 adds the [twenty-slot job lookup](rom_data_gameplay.md#managed-twenty-slot-job-lookup-and-typed-bus-consumption):
66files/65resources, all64prior decoded files/records and e70retained, actual
metadata52dependency. Two focused Wine filters pass publication/lifecycle and
complete-state/clock original/interpreter/native reads, live slot selection and
mixed bus/SWI/DMA. The owner serves only20bytes; original getters and all outside
inputs stay live. Actual production CLI create/no-write reuse/old-cache one-frame
attachment now pass66generation/68managed files, all66matching the accepted dataset.
One Bank120Winefullpair now passes complete state/clock/captured buffers/OAM/
dispatch/all five audio streams, with2newjobbytes/one slot record matching CLI.
Other19ordinary slots and Linux/USA/native Windows/new32:9remain pending.
No performance/storage measurements.

EUR e70-d69 adds the [two numeric FFFF reader fields](rom_data_unit_obj.md#managed-ffff-reader-fields-and-typed-bus-consumption):
65 files/64 resources, all63 earlier decoded files/records and e69 retained.
The immutable owner serves only six bytes; encoded addresses and original live
unmapped BIOS/unaligned CPU behavior stay intact. Two focused Wine filters pass
publication/lifecycle/authentication and complete-state/clock native/bus/SWI/DMA
checks. Actual production CLI create/no-write reuse and old-cache one-frame
attachment now pass65generation/67managed files, all65 equal the accepted dataset.
Ordinary positive reachability/equivalence remain pending. USA remains e8-d7;
Linux/native Windows/new32:9 remain to be checked. No performance/storage measurements.

EUR e69-d68 extends the [property-selected static UI OBJ lists](rom_data_obj.md#managed-ui-property-list-extension-and-typed-consumption)
within the existing resource: 126 references/74 lists/175 objects/1198 source bytes.
The new list contributes 14 bytes; the B2 alias retains its previous owner.
There are still 64 files/63 resources, with all 73 previous list records and
62 other resource files retained. Two focused Wine filters pass publication,
authentication/lifecycle, original/interpreter/compiled state-clock, host decode
and mixed-source bus/SWI/DMA checks. Actual CLI create/no-write reuse now passes
64 generation/66 managed files, all64 accepted, plus old-cache one-frame attach.
One Dispatch1840 Wine pair now passes every full state/clock, central/wide image,
OAM, dispatch and audio stream, with positive A4 guest/host use. USA stays e8-d7;
Linux/native Windows/new32:9 checks remain to be done.
No performance or storage measurements.

EUR e68-d67 adds [UI byte property words](rom_data_unit_obj.md#managed-ui-byte-property-words-and-typed-runtime-reads):
64 files/63 resources, with all 62 previous records and 63 files retained.
The new owner serves only 72 words/144B; 184 shared words retain their actor-property
and palette owners as actual dependencies. Two focused Wine filters pass managed
lifecycle and original/interpreter/compiled state-clock checks, including mixed
old/new/raw bus and SWI/DMA reads. Actual production CLI create/reuse now passes
64 generation/66 managed files, all64 accepted and no-write reuse. One authentic
F09 frame loads the old cache and uses1read/2B/oneword. A new F09Wait3530 Wine
fullpair now passes everyframe state-clock-image-OAM-dispatch-allaudio, with
476reads/952aggregateB/4words positive; other68words targeted-reader only.
USA stays e8-d7; Linux/native Windows/new32:9 checks remain to be done.
No performance/storage measurement or previous acceptance replay.

EUR e67-d66 adds [actor property words](rom_data_unit_obj.md#managed-actor-property-words-and-typed-runtime-reads):
63 files/62 resources with actual metadata52/metadata14 dependencies. All61prior
files/records and e66are retained; USA stays e8-d7. The immutable owner serves
four ranges/364B from182u16records, preserving raw holes, previous owners and
original allocation/task reuse. First two focused Wine filters pass17attach+
4full-set guards/lifecycle/all364typedB and16original/interpreter/compiled
complete-state-clock paths with raw sources altered. Four range boundaries,
widths/mirrors/real-BIOS bus, CpuSet/FastSet/DMA and mixed old/new/raw reads pass.
Actual production CLI create/reuse63generation65managed/all63accepted/no-write
and oneframeoldcacheattachment pass (newusage0/smokeonly). One new F09Wait3530
ordinary CLI/fullpair now passes10reads/20aggregateB/twowords with everyframe
state-clock-central-wide-OAM-dispatch-allaudio equality. Other180words retain
targeted source-reader coverage; new Linux/USA/nativeWindows/new32:9 remain pending. Source/model/earlier proofs stay
closed; no gameplay fixture, navigation or performance/storage measurement.

EUR e66-d65adds [registry generic direct-job poses](rom_data_unit_obj.md#managed-registry-generic-direct-job-poses-and-typed-guest-reads):
62files61resources, actual metadata52/archive and six previous typed-view
dependencies, all60prior resources/e65retained/USAe8-d7. The new owner serves
659newspans99824B; previous owners/rawholes/original tasks-DMA remain intact.
Two focused Winefilters firstPASS36attach+10dependencyguards/lifecycle/allnewB/
126newseq and25original-interpreter-compiled complete-state-clock paths/659edges/
BIOS-SWI-DMA/mixed2new4alternate+2new2raw. Six previous owners retain shared reads.
ActualproductionCLIprepare/reuse62generation64managed/all62matchaccepted/no-write/
oneframeoldcacheattach firstPASS(allnewusage0/smokeonly). Ordinarynewpixel/
fullscene and FFFF/redirect/outofcatalog/
changedactor/globalregistryidentity remain separate. Linux-USA-nativeWindows-
new32:9 pending; source/model/oldproofs closed/no repeat, no performance or
storage measurements.

EUR e60-d59 adds the [normal-mode battle pose owner](rom_data_unit_obj.md#normal-battle-managed-publication-and-runtime-reads):
6021new spans1455474B, with exact shared initial/equipment views retaining their
older owners.56files55resources preserve all54previous decoded records and e59;
USA stays e8-d7. Two new reduced Wine filters pass17guards/lifecycle/alltyped
bytes and32original/interpreter/compiled complete-state-clock paths, allsix
APIcategories/6021edges/mirrors/BIOS/DMA/raw holes and older-owner composition.
Source/model acceptance remains closed. ActualCLIcreate/no-write reuse56generation/
58managedfiles and one genuine3530frame F09movement fullstate-clock-image-OAM-
dispatch-allaudio pair now pass842236aggregateB/815360pixelB/fivesequences. Other
2446newsequence ordinary uses retain targeted coverage. New Linux/USA/native
Windows/new32:9 checks stay open, without performance or storage measurements.

EUR `v1/e58-d57` adds [Color-Mode unit/UI palettes](rom_data_color_palettes.md#managed-publication-and-typed-runtime-reads):
30spans5750B/2873u16colors/fouru8selectors. The owner returns typed words and
selectors with per-byte raw-gap/previous-owner composition; live modes, copies,
fades, RAM and save/native ABI remain original.54files53resources authenticate
with metadata14dependency; all52prior decoded/resource records/e57 and USAe8-d7
remain. Two reduced Wine filters pass17guards/lifecycle/all5750B and28complete
original/interpreter/compiled state-clock paths. MixedSWI/DMA preserve2new+20raw+
4oldUI bytes with exact2/4charges; all30edges/mirrors/BIOSbus and liveRAMpass.
Source/model/oldfamily proofs were not rerun. ActualCLI create/reuse-no-write
56managedfiles/all54generationfiles and one positive ordinaryLutia1700 fullstate-
clock-image-OAM-dispatch-allaudio pair pass1102aggregate colorB/fourfragments.
Party320/boot240newfamily-negative locators receive no fullpairs; remaining26
ordinary fragments and newLinux-USA-nativeWindows/32:9 stay pending. No metrics.

EUR `v1/e57-d56` adds [normal unit initial-pose sources](rom_data_unit_obj.md#managed-publication-and-direct-runtime-reads):
415 spans146358B/121 producer IDs/477 frames/333 raw4bpp requests. The immutable
owner reconstructs source bytes from typed directory/index/header/frame/OAM
fields and decoded palette indices, while original RAM/scheduling/DMA/VRAM and
save/native ABI remain.53files52resources authenticate with actual metadata52/14
dependencies; all51prior decoded files/resource records/e56 and USAe8-d7 remain.
Two reduced Wine filters pass17guards/lifecycle/all146358typedB and30 complete
original/interpreter/compiled state-clock paths, raw ID/animation, live RAM
frames, all415edges/mirrors/BIOSbus/2SWI/1DMA. Mixed transfers preserve2new+
1276raw+8oldOBJbytes with exact2/8owner charges; old OBJ shares page394 correctly.
Actual production CLI create/reuse-no-write55managed files/all53generation files
match the accepted set with the old cache. One320-frame existing Party-unit
fullstate-clock-central/wideimage-OAM-dispatch-all-audio pair passes with10962
aggregate new B=[8dir16index4header252frame7162OAM3520pixel]/2sequences and764guards.
Wholeunit/allocator ABI, other119ordinary sequences, poses/parts/IDs/palettes/
newLinux-USA-nativeWindows/32:9 remain open.

EUR `v1/e47-d46` adds `blend-programs.json` /
`graphics.blend-programs`:160programs/345commands/6producer tables,
158ranges/2428bytes.42files/41resources authenticate; all40prior decoded files/
e46 and USAe8-d7 remain. Typed header/command/producer fields serve original
CPU/DMA/BIOS/native reads with previous-owner/raw merging and mutable RAM intact.
Reduced Wine checks pass12publication/authentication/lifecycle guards/all2428B/
160programs6tables and20original-interpreter-native complete-state/clock prefixes.
Compiled APIs read[34header70command16producer]B, including live mode override,
raw command65535/tableindex3 and previous palette-owner index5/boundary.
One additional focused source-set guard verifies early rejection of a missing
preceding extraction owner; no closed source/lifecycle/native proof repeat.
Actual production CLI create/reuse-no-write44managedfiles/oldcache and one
1000-frame Wine ordinary EUR Pecorelle fullstate-clock-classic240x160-image-OAM-
dispatch-audio pair pass18aggregate sourceB (10header/8command),1program,
0producer. Other159programs/tables retain targeted-only coverage. NewLinux-USA-
nativeWindows/new32:9 scene checks remain pending.
[Blend consumption](rom_data_graphics.md#managed-blend-programs-and-typed-guest-reads).

EUR `v1/e46-d45` adds `palette-programs.json` /
`graphics.palette-programs`:230programs/1135commands/25producer tables,
165ranges/18697bytes.41files/40resources authenticate; all39prior decoded files/
e45 and USAe8-d7remain. Typed header/command/parameter/producer fields serve
CPU/DMA/BIOS/native reads; guest sequencing, live indices and raw outside reads
remain original. One reduced Wine lifecycle check passes12guards/all18697B/
230programs/25tables/create-reuse-repair-move/load-Undo-backup-Restart. A native-only
check passes24original-interpreter-native complete-state/clock prefixes with all
fourfield categories and raw command65535/coloroffset65534/index2 behavior.
The test endpoint guard corrects a chained-prefix harness issue; production
emitter unchanged. Production CLI create/reuse-no-write43files/oldcache passes.
Normal English-confirm1000is coverage-negative0use/no fullpair. Ordinary EUR
Montblanc Richiamo/Pecorelle now passes actual CLI plus one1000-frame Wine pair:
170aggregate source bytes (32header/90command/48parameter),4programs,0producer.
Every complete state/clock/classic240x160 image/OAM/dispatch and all five audio
streams agree. The generic scene guard now accepts typed bus or host consumers;
family-specific positive-use requirements remain. Other226programs and producer
tables retain targeted-only coverage. NewLinux-USA-nativeWindows/new32:9 scenes
remain pending.
[Palette programs](rom_data_graphics.md#managed-palette-programs-and-typed-guest-reads).

EUR `v1/e45-d44` adds `palette-effect-dispatch.json` /
`graphics.palette-effect-dispatch`:17slots/68bytes,40files/39resources/all38prior
decoded files/e44retained, USAe8-d7. Typed guest callback/null words feed original
CPU/DMA/BIOS/native reads with liveu16kind/state gates/rawoutside semantics.
Two reduced Wine checks pass12guards/lifecycle/all68bytes/17slot mask and
10original-native fullstate-clock prefixes/BIOS-SWI-DMA/boundaries. No callback
rewrite/save-native ABI change. Actual productionCLIcreate/reuse-no-write42files/
oldcache and one normal English-confirm1000frame Winepair pass full states/clocks/
images/OAM/dispatch/audio and5944aggregate callback-byte reads, kind1only(mask2).
Neutralboot240is closednegative0use/no fullpair; othercallbacks/nulls targeted-only.
Remainingasset/newLinux-USA-nativeWindows work stays open.
[Palette dispatch](rom_data_graphics.md#managed-palette-effect-callback-consumption).

EUR `v1/e44-d43` adds `audio-driver-tables.json` /
`audio.m4a-driver-tables`: seven spans / 484 bytes, 39 files / 38 resources,
all37 prior decoded files/e43 retained and USAe8-d7. Actual parsed sequence
boundaries authenticate the new owner. CPU/DMA/BIOS/native/generic-host reads
reconstruct typed fields and preserve sample0/13–15 and volume16/31/64/255
adjacent-owner/raw aliases, without new guest clamps. Seven byte counters and
424 entry bits stay outside saves; guest clock/RAM/native ABI remain original.
Two reduced Wine checks pass19guards/lifecycle/all484bytes/424entry bits and
28original-native complete-state/clock paths, mixed Thumb/ARM/aliases/BIOS/SWI/DMA/
boundary-host reads. Actual productionCLIcreate/reuse-no-write41managedfiles/
oldcache and one400ordinary Winefull-state/clock/image/audio pair pass756source-
byte reads (132PCMkey/528PCMbase/32CGBkey/64CGBbase),67physical entries. Sample-
count/noise/volume/generic-host ordinary reads are zero; other work/newLinux-USA-
nativeWindows remain open.
[Driver contract](rom_data_audio.md#managed-driver-tables-and-original-integer-reads).

EUR `v1/e43-d42` adds `audio-sequences.json` / `audio.m4a-sequences`:
808 spans / 177,248 bytes of typed instructions and driver tables. All 38 files /
37 resources authenticate with actual directory/tonebank dependencies; all 36
prior decoded files and e42 remain, USA stays e8-d7. Two focused Wine checks pass
25 guards/lifecycle/all bytes/instruction-table masks/35 original-interpreter-
native state-clock checkpoints/808 boundaries-BIOS-SWI-DMA/live RAM running status/
raw fallback and two-owner tone/sequence guest/generic-host reads. No host
sequencer or guest timing/save/native ABI change. Production CLI create/reuse
without rewriting 40 managed files, old-cache dispatch and one 400-frame ordinary
Wine full-state/clock/image/audio pair pass with 819 new bytes (366 opcode,
262 operand, 191 clock) and 185 instruction source records observed including
lookahead. Command/extended/generic-host reads are zero in this scene. Other
asset/LUT/GUI/archive/mod and new Linux/USA/native Windows work remain open.
[Sequence consumption](rom_data_audio.md#managed-sequence-publication-and-typed-guest-reads).

EUR `v1/e42-d41` adds `audio-tones.json` / `audio.m4a-tones`:
429 tones, 347 map bytes, 277 signed PCM waves and two CGB waves; 736 source
spans / 1,941,759 bytes, with an actual directory dependency for the four-byte
alias. All 37 files / 36 resources authenticate; all 35 prior decoded files,
e41 and USA e8-d7 remain. Two reduced Wine checks pass publication/26 guards/
lifecycle/all typed bytes/25 original-interpreter-native state-clock checkpoints/
736 boundaries/BIOS/SWI/DMA. Actual MP2K frame-hook and tick samplers read typed
headers/samples for all 277 waves with nonzero bit-exact shadow audio and equal
complete host state/clocks. Production CLI create/reuse-no-write (39 managed
files), old-cache dispatch and one 400-frame ordinary Wine pair pass all states,
clocks, images and audio with 516,956 guest source bytes plus 3,093,208 host bytes
(1,056 header / 3,092,152 PCM). Ordinary CGB reads are zero. Default headless CLI
shadow is off; the differential arms it after load. Internal probes are partial.
Complete sequence export, driver LUTs, unselected assets and new Linux/USA/native
Windows remain open.
[Audio consumption](rom_data_audio.md#typed-tone-and-sample-consumption).

EUR `v1/e41-d40` adds `audio-directory.json`/`audio.m4a-directory`:
typed3player/640song/455shared-header fields reconstruct15236Bacross457spans.
All36files/35resources authenticate; all34prior decoded files/e40generations
and USAe8-d7remain. Nonserialized field counters/record bitsets observe use;
raw gaps/dummy-header-wave alias, guest mixer/sequencer/RAM/clock/ABI remain.
Two focused Wine checks pass publication/25coherent guards/retention/lifecycle,
44original-interpreter-native paths with raw15236Baltered/fullstate-clock
equality and allspans/allbytes/edges-BIOS/SWI-DMA/rawtail. ProductionCLIcreate/
reuse-no-write/oldcache and one400-frame Wine ordinary fullstate-image-audio
pair pass179B(36directory/79header/64track-pointer),3songs/headers/0player reads.
Full bank/sample/sequence/newLinux-USA-nativeWindows pending.
[Contract](rom_data_audio.md#managed-publication-and-typed-source-consumption).

EUR `v1/e40-d39` adds fourteen packed7B `item-effect-parameters.json`
records/98source bytes, with explicit item-record owner dependency. All35files/
34resources authenticate together; all33prior decoded files/e39generations and
USAe8-d7remain unchanged. Typed page6e8coefficient/shared05/shared06/14recordmask
reads merge adjacent trade ownership, preserving original getter/fullu16cap vs
low8index/raw-cap aliases/live current/quantity/clamp/instructions/clock/ABI.
Two reduced Wine checks pass50original-interpreter-native paths/raw98Baltered/
fullstate-clock equality, all98B/mask16383/edges-BIOS/SWI-DMA/two-owner crossing/
publication-retention-lifecycle. ActualCLIcreate/reuse-no-write/oldcache and one
400Wine EUR GoblinPlay fullstate-image-audio pair pass3newsourceB,mask1record0.
Ordinary replay is mixed native/interpreted; other14item/20monster/cap>99UI/
newLinux-USA-nativeWindows paths remain pending.
[Contract](rom_data_gameplay.md#ordinary-item-effect-parameters-in-the-european-monster-bank).


EUR `v1/e39-d38` adds five96B `trade-parameters.json` records, physical
IDs2..6,480source bytes. All34files/33resources authenticate together; prior
32decoded files/e38generations and USAe8-d7remain unchanged. Source-equal typed
reads merge page6e8owners/raw gaps, retain shared byte41, and keep original
selector/price/getter/live flags/count/division/clock/save-nativeABI. Separate
opaque/markup-low/shared41/discount-tail counters and five-record mask observe
usage. Two reduced Wine checks pass publication/authentication/retention/lifecycle,
28original-interpreter-native paths/raw480Baltered/fullstate-clock equality,
all480B/5recordmask/field-boundaries/SWI-DMA/rawgap/priorowner-coexistence.
ActualCLIcreate/reuse-no-write/oldcache and one400Wine fee acceptance fullstate-
image-audio pair pass14sourceB(7markup-low/7shared41),mask1physicalID2; rereads
included. Other ordinary fields/rows/newLinux-USA-nativeWindows remain pending.
[Contract](rom_data_gameplay.md#ordinary-trade-parameter-fee-acceptance).

EUR `v1/e38-d37` adds `aggregate-flag-keys.json` /
`gameplay.aggregate-flag-keys`:300ordered u16slots and a terminalffff/602source
bytes. All33files/32resources authenticate together, all31prior decoded files/
e37generation retained, USAe8-d7. Two focused Wine checks pass16fullglobal/context
counter paths with raw602Baltered/fullstate-clock equality, key/terminal/300slot
coverage and publication/lifecycle/bus-BIOS/SWI-DMA guards. Original live flag/
linked predicates/context/key-wrap/count/clock/save-nativeABI remain active.
Source32+26is closed. ActualCLIcreate/reuse-no-write/oldcache and one1000Wine
ordinary dispatch fullstate/image/audio pair pass1204sourceB(1200key/4terminal)/
all300slotmask; includes rereads and does not infer both ordinary entrypoints.
NewLinux-USA-nativeWindows/other UIreaders remain pending. [Contract](rom_data_gameplay.md#managed-and-direct-aggregate-flag-keys).

EUR `v1/e37-d36` adds `progression-selection-gates.json` /
`gameplay.progression-selection-gates`:64physical records/1024source bytes.
All32files/31resources authenticate together, all30prior decoded files/e36
remain unchanged; USA stays e8-d7. Original reset/count/ANY/flag/terminal-store
instructions, clocks and save/nativeABI remain active. Two focused Wine checks
pass18original/interpreter/native fullstate-clock paths with raw1024B altered,
publication/lifecycle/binding and bus/SWI/DMA/gap/coexistence checks. Source138+80
is closed. ActualCLIcreate/no-write reuse/oldcache and one1000Wine ordinary
dispatch fullstate/image/audio pair pass277newB/all6fields/first26records; the
400Yes-fee0Blocator is closed negative/no fullpair. NewLinux-USA-nativeWindows/
all64ordinary rows/semantic consumers remain pending. [Contract](rom_data_gameplay.md#managed-and-direct-progression-selection-gates).

EUR `v1/e34-d33` adds `evaluation-parameters.json` /
`gameplay.evaluation-parameters`:7score terms,2conditional bonuses and5class
thresholds,38source bytes in two disjoint ranges. All29files/28resources
authenticate together; all27earlier decoded files/e33 generations remain
unchanged. Original live score getters, unsigned-u8/signed32/wrapping arithmetic,
conditional predicates, RNG, clock and save/nativeABI remain active. Two focused
Wine managed/native/lifecycle checks pass27prefixes with raw38Baltered and
complete state/clock equality; all5threshold classes are positive. One1840Pub
mission full-state/image/audio/actualCLI pair reads15score bytes. Threshold
ordinary class1 use now passes a later400frame Yes/fee fullpair; classes2..5
remain targeted-only. NewLinux/USA/nativeWindows remain pending.
[Contract](rom_data_gameplay.md#managed-and-direct-evaluation-parameters).

EUR `v1/e33-d32` adds `global-pair-flags.json` /
`gameplay.global-pair-flags`:44two-bit fields/11source bytes with no inferred
semantic dependency or padding ownership. All28files/27resources authenticate
together; all26earlier decoded files/e32 generations remain unchanged. Two
focused Wine managed/native/lifecycle checks pass28paths with raw11B altered
and complete state/clock equality, including prior/new typed and raw-tail
crossings. Original wrapper/live-mask/earlyexit/clock/save/nativeABI remain.
ActualCLIcreates/reuses without writes/oldcache/unrelatedcwd. EchoGrass2200is
negative for this new source; positive ordinary global use/fullscene pair remain
pending, along with newLinux/USA/nativeWindows checks.
[Contract](rom_data_gameplay.md#managed-and-direct-global-packed-pair-flags).

EUR `v1/e32-d31` adds `linked-metadata12.json` /
`gameplay.linked-metadata12`: two indexed packed-pair banks, 108 records /
1,296 source bytes. All 27 files / 26 resources authenticate together; all
25 earlier decoded files/e31 generations remain unchanged. No unproved
semantic dependency is added. Typed CPU/native/SWI/DMA bytes preserve original
full32 row, signed/wrapping slot, u8 mode, clock and save/native ABI. Two focused
Wine managed/native/lifecycle checks pass 49 paths with all raw bytes altered
and complete state/clock equality; both banks are positive and three unowned
paths read zero new bytes.
One positive 2200-frame Echo Grass confirmation full-state/image/audio/actualCLI pair passes.
New Linux/USA/native Windows checks remain pending.
[Contract](rom_data_gameplay.md#managed-and-direct-twelve-byte-packed-pair-banks).

EUR `v1/e31-d30` adds `linked-metadata14.json` /
`gameplay.linked-metadata14`: 93 special-unit records / 1,302 bytes with the
parsed metadata52 byte02 owner. All 26 files / 25 resources authenticate together;
all 24 prior decoded files and e30 generations remain unchanged. Typed CPU,
native, SWI and DMA reads preserve original getters, live palette books/copies,
descriptor overrides, clock and save/native ABI. Two focused Wine managed/native/
lifecycle checks pass 37 paths with raw source bytes altered and complete state/
clock equality; normal IDs0/1 consume no new bytes. One positive 397-frame Pub
full-state/image/audio/actual CLI pair reads 27 bytes from two records across
all field categories. New Linux/USA/native Windows checks remain pending.
[Contract](rom_data_gameplay.md#managed-and-direct-fourteen-byte-special-unit-metadata).

EUR `v1/e30-d29` serves four48Bconstructor records and4Bbase pointer from
`gameplay.constructor-alternate-templates`,196Btotal. Typed pointer/header/argument/
full-halfword/flag/tail bytes preserve original ROM/IWRAMcopy/live patches/mode/
allocation/clock/save/native ABI. All25files/24resources authenticate together with
actual parsed Items dependency; all23prior decoded files/e29retained. Two focused
Wine managed/native/lifecycle checks pass14paths/all196rawBaltered/equal fullstates-
clocks, pointer/all4recordsmask15/five zero-new-use paths and both-span bus/SWI/DMA.
One3600Lutia deployment full-state/image/audio/actualCLI pair reads45B(pointer4/
record20fields41), mask1; other records/full-copy targeted-only. NewLinux/USA/
nativeWindows/neighboring headers/following240Bpending.
[Contract](rom_data_gameplay.md#managed-and-direct-constructor-alternates).

EUR `v1/e29-d28` serves one48Bdefault constructor template from
`gameplay.default-constructor-template`. Typed header/argument/full-halfword/flag/
tail bytes preserve original ROM/IWRAM copy/live patches/mode/alternate selection/
allocation/clock/save/native ABI. All24files/23resources authenticate together,
all22prior decoded files/e28retained. Two focused Wine managed/lifecycle/native
checks pass16prefixes with48rawBaltered/fullstates-clocks equal and8alternate/
rejected paths0newreads, plus all-byte bus/SWI/DMA/adjacent crossing. One397Pub
full-state/image/audio/actualCLI pair reads96newBacross all structural categories.
New Linux/USA/native Windows and alternate192B/pointer/next240Bremain pending.
[Contract](rom_data_gameplay.md#managed-and-direct-default-constructor-template).

EUR `v1/e28-d27` serves44stored key/value pairs/88source bytes from
`gameplay.linked-metadata2`. Typed bytes preserve original forward44/inverse43,
first stored key/duplicates/miss0/u8truncation/live caller instructions and clock.
All23files/22resources authenticate together; actual parsed metadata52key owners,
all21previous decoded files/e27and save/native ABI remain. Two focused Wine
managed/lifecycle/native checks pass21paths with raw88B altered/equal complete
states-clocks, all-byte bus/SWI/DMA and adjacent typed crossing. One positive
1840Pub mission full-state/image/audio/actualCLI pair reads4value/0keyB;
inverse key use remains targeted-only. New Linux/USA/native Windows remain
pending. [Contract](rom_data_gameplay.md#managed-and-direct-metadata2-consumption).

EUR `v1/e27-d26` serves 20 two-pair records / 80 source bytes from
`gameplay.linked-metadata4`. Typed selectors/values preserve original counting,
contiguous walking, unsigned values/signed comparisons, tally wrapping and
live ability scans. Actual parsed metadata52 ownership is checked; save/native
ABI and all 20 prior decoded files/e26 stay unchanged. Two focused Wine
managed/native/lifecycle checks pass 56 sampled prefixes with all new raw bytes
altered / equal complete states and clocks, plus all-byte bus/SWI/DMA and
adjacent typed crossing. One positive 540-frame Change Jobs full-state/image/
audio/actual CLI pair reads 32 selector / 6 value bytes. New Linux/USA/native
Windows remain pending. See the [pair contract](rom_data_gameplay.md#managed-and-direct-metadata4-consumption).

EUR `v1/e26-d25` serves 116 records / 6032 source bytes from
`gameplay.linked-metadata52`. Typed header/routing/packed/tail/opaque bytes feed
original CPU/DMA/native readers without replacing the packed getter, redirects,
alternate indices, live-record inputs, clock or save/native ABI. All 19 previous
decoded files and e25 generations remain unchanged. Two focused Wine checks pass
75 sampled native paths with all new raw bytes altered / equal complete states
and clocks, actual parsed link-owner bounds, lifecycle and adjacent typed
metadata16/metadata52 crossing ownership. One positive 540-frame Change Jobs full-state/image/audio/actual CLI pair
passes with 664 new bytes; ordinary tail/opaque use is not claimed. New
Linux/USA/native Windows checks remain pending. See the
[metadata52 contract](rom_data_gameplay.md#managed-and-direct-metadata52-consumption).

EUR `v1/e25-d24` serves128sixteen-byte records/2048source bytes from
`gameplay.linked-metadata16`. Exact typed alias/raw fields feed original
CPU/DMA/native reads while live-list IDs/indices, RNGstate/selection, sums,
register-ASR and all guest work/clock stay active. All18prior decoded files/e24
and save/native ABI remain unchanged. Two reduced Wine authentication/lifecycle/
native checks pass42sampled paths with2048new raw bytes altered/full states and
clocks equal, plus all-byte/BIOS/SWI/DMA and a2old-link+2newmetadata crossing.
One positive520-frame Clan Mission Item full-state/image/audio/actual CLI pair
passes with6typed display bytes; ordinary alias/flag/value use is not claimed.
Linux/USA/native Windows
remain pending. See [metadata16contract](rom_data_gameplay.md#managed-and-direct-metadata16-consumption).

EUR `v1/e24-d23` serves225counted item-linked records/4500source bytes
from `gameplay.item-links`, with an explicit parsed item-owner dependency.
Typed count/selector/value/opaque fields preserve live query/unit publication,
all original work/clock and save/native ABI. All17earlier decoded files/e23
remain unchanged. Reduced Wine authentication/lifecycle/native62paths with
altered raw source match full states/clocks; a crossing word also proves both
adjacent typed owners. One positive560-frame headgear inventory full-state/image/
audio/actual CLI Wine pair consumes574new bytes(287count/205selector/82value).
See [item-link contract](rom_data_gameplay.md#managed-and-direct-item-link-consumption).
New Linux/USA/native Windows and wider gameplay catalogs remain pending.

EUR `v1/e23-d22` also serves376static item records/12032source bytes
from `gameplay.item-records`. Typed halfword/raw/signed/tail bytes preserve every
record bit through original CPU/DMA/native reads; live bonus/inventory, property
dispatch/clamp and secondary table work stay original. All16previous decoded
files/e22generations and save/native ABI remain. Reduced Wine original/native/
authentication/lifecycle checks pass, with altered raw sources/full state/clock
equality and separate source/field counters. One positive320-frame Unit Wine
full-state/image/audio/actual CLI pair passes with40typed signed-field bytes.
See [item contract](rom_data_gameplay.md).
New Linux/USA/native Windows and broader gameplay catalogs remain pending.

EUR `v1/e22-d21` extends custom UI graphics to16banks/two language
selectors/two explicit shared OBJ owners. Typed asset-pointer literals preserve
live language choice, English fallback in Clan and skip-outside0..4in the
second switch. Original5056OBJ/2688BG split uploads retain exact word data.
Seven new banks/ten pointer literals add17disjoint ranges/14436source bytes;
all nine previous UI bank records, every other decoded file and e21generations
remain unchanged. New selector/localized-source/localized-literal counters
distinguish this scope. Reduced Wine original/authentication/lifecycle/native
checks pass all seven new payloads, both selectors and shared-owner tails;
complete states/clocks match with new spans altered. Save/native ABI remains.
One positive1200-frame English intro full-state/image/audio/actual CLI Wine pair
passes with4selector bytes/2347new-source bytes/1124new literal bytes.
New Linux/USA/native Windows checks stay pending.

EUR `v1/e21-d20` also serves nine custom BG/OBJ UI graphics banks via
`graphics.ui-custom-banks`. Exact typed u32literal positions and codec metadata
feed original CPU/DMA/native reads; original decoded return lengths, shifts/
masks, live scratch and full/partial uploads remain guest instructions and
clock work. No added host ROM consumer is needed: PPU reads resulting live
VRAM. All15earlier decoded files/e20generations stay unchanged. Family and
per-bank counters prove all-nine native ownership and ordinary Clan source43bc28
use; save/native ABI and unavailable-file Load/Undo/backup/restart remain.
Reduced Wine original/native/lifecycle and one positive320-frame Clan
full-state/image/audio/actual CLI pair pass. New Linux/USA/native Windows and
other indirect sources remain pending. See [custom UI graphics](rom_data_ui_graphics.md).

EUR `v1/e20-d19` additionally serves21raw UI palette banks/1328u16words
through `graphics.ui-raw-palettes`. Both BG/OBJ shadow halves, original Color-Mode
selectors, copied IWRAM memcpy instructions, live fades and palette publication
remain active; the PPU reads dynamic palette RAM. Exact typed-word ROM views
serve CPU/DMA/native/guest-BIOS bus reads with nonserialized family/per-bank
coverage. All14earlier decoded files/e19generations and save/native ABI stay
unchanged. Reduced Wine source/native/lifecycle and one positive240-frame
boot full-state/image/audio/actual CLI pair pass. New Linux/USA/native Windows
remain pending. See [raw UI palettes](rom_data_ui_palettes.md).

EUR `v1/e19-d18` also serves 39 custom OBJ-related graphics banks and
six language selector tables from `graphics.obj-custom-banks`. Exact encoded
literal positions read typed decoded u32 words; header/token positions retain
authenticated codec metadata. Typed pointer order reconstructs original ROM
pointers. Original decoder instructions, scratch transforms, CpuSet/FastSet/DMA,
partial uploads and live RAM tails remain, and the PPU consumes resulting live
VRAM. No additional host ROM graphics decoder is introduced. Nonserialized
`source_bytes`, `decoded_literal_bytes` and `selector_bytes` counters identify
use of this family. Save/native identity and ABI stay source-equal; immutable
ownership survives Load/Undo/backup/restart with files unavailable. All thirteen
previous decoded files/e18generations are retained; USA stays e8-d7. Reduced
Wine checks and one positive 400-frame full-state/image/audio “Engage!” scene
pass. New Linux/USA/native Windows checks remain pending. See the
[custom OBJ graphics contract](rom_data_obj.md#custom-obj-word-banks-in-e19).

EUR `v1/e18-d17` also projects two five-entry language selector tables
and five newly rooted OBJ/sequence bank pairs through original CPU/DMA/native
reads. Ordered typed roots resolve to retained banks, host lists share that
owner and original u16 duration 256/final counter-wrap behavior remains.
Separate localized table/bank/host counters gate positive ordinary consumption.
Source/native/host/lifecycle checks and one 400-frame “Engage!” full-state/audio/
actual CLI pair pass on Wine; earlier eleven bank records and every other
decoded file/e17generation remain unchanged. See [the language bank contract](rom_data_obj.md#language-selected-ui-banks-in-e18).

EUR `v1/e17-d16` additionally consumes an eleventh OBJ bank with five
proved zero-count holds. Original counts stay zero; typed selectors and duration
references preserve the original nonrepeat task behavior, with shared header/
index owners and no overlapping source spans. New selector/host counters gate
this bank separately. Reduced Wine source/native/host/lifecycle checks and a
positive 240-frame Party full-state/audio/actual CLI pair pass. The previous ten
bank records and all other decoded files/e16generations remain unchanged.
See [the zero-count hold contract](rom_data_obj.md#zero-count-hold-bank-in-e17).

EUR `v1/e16-d15` extends the static OBJ catalog to 73 lists/173 objects
from 123 authenticated direct references, retaining e15 and all previous typed
records. Per-source nonserialized guest/host flags identify new-list use in
usage reports and scene gates. The seven-list extension passes focused Wine
source/native/host/lifecycle checks and a positive 240-frame language-screen
full-state/audio/actual CLI pair. New Linux/USA verification remains pending.
See [the extension contract](rom_data_obj.md#additional-direct-lists-in-e16).

EUR `v1/e15-d14` also retains indexed OBJ animation banks: relative
indices, list fields, sequence counts, frame IDs and duration words feed original
CPU/DMA/native and host paths. Live task counters and all original guest work
remain. Source/native/lifecycle acceptance and one positive160-frame ordinary menu
transition/actual CLI pass on reduced Wine. See [the indexed bank contract](rom_data_obj.md#indexed-ui-animation-banks).

EUR `v1/e14-d13` extends the same immutable owner to directly rooted static OBJ
lists. Typed counts and decoded coordinate/attribute fields serve original
CPU/DMA/native reads and host draw/visibility/battle readers. Dynamic RAM lists
remain live. See [static OBJ contract and verification](rom_data_obj.md).

`play`, `frames` and `runc` accept developer opt-in `--rom-data`. They prepare
and load `graphics.obj-dimensions`, `graphics.bg-descriptors` and
`graphics.bg-base-palettes` and `graphics.bg-alternate-palettes` from the selected
ROM's decoded dataset. The sixth e8-d7file, `graphics.bg-packet-programs`, now
also serves EUR packet source/body reads from immutable typed values and exact
control metadata. EUR reports `runtime=decoded`; USA stays authenticated and
`runtime=verified-only` pending its consumer validation.
EUR prepares a seventh `graphics.bg-main-graphics` JSON, introduced in `v1/e9-d8`,
authenticating every source and file hash into dataset identity. This family
now reports `runtime=decoded`: custom-codec literals come from retained typed
words, with exact original headers/tokens and unchanged guest instructions.
USA keeps
its existing six-file `v1/e8-d7` contract. New Linux/USA verification is pending.
EUR `v1/e10-d9` additionally authenticated an eighth
`graphics.bg-transfer-programs` JSON and its76source ranges. This new family
reports `runtime=decoded`; original transfer readers now use retained typed
controls/tile words through the same CPU/DMA/native bus paths. Original live
frame/countdown/flags/copy cursors and every guest instruction/cycle remain.
EUR `v1/e11-d10` added a ninth `text.short-strings` JSON with670source
ranges. It participates in whole-set authentication/identity and reports
`runtime=decoded`: original readers obtain table/string/glyph/metric bytes from
retained typed indices/codes/words/widths, with separate counters. Original
expansion/copy/render work and guest clock remain intact. See the
[direct text/font contract](rom_data_text.md#direct-consumption).
EUR `v1/e12-d11` added the broad `text.catalogs` JSON with33581source
ranges. It joins whole-set authentication/identity and reports `decoded`:
typed pointers, strings, archive indices/headers/program bytes and font data
serve original CPU/DMA/native reads. Compressed literals come from decoded
units/padding; original controls and guest work remain intact. The overlapping
earlier subset stays authenticated but `verified-only`, with no duplicate read
owner/counter attribution. See the [broad direct-consumer proof](rom_data_text.md#broad-direct-consumption).
One retained EUR/Wine397-frame pub transition now matches every state/clock,
image/OAM/dispatch/audio with positive27decoded program/8064glyph/93metric bytes.
One actual CLI no-write e12reuse/old cache matches the retained legacy row.
[Declared scene/CLI scope](rom_data_text.md#ordinary-transition-and-production-cli).
Normal enablement awaits the remaining ROMDATA-05 regional and native
Windows gates.
Other resource families and mod support remain open in the
[full TODO](development/ROM_DATA_TODO.md). These bounded metadata migrations do not provide ROM-free execution or
complete graphics extraction.

EUR `v1/e13-d12` added `font.complete-bank`/`text.name-keyboard` as two
fixed readable JSON files, bringing that version to12files/11resources; e14adds
the thirteenth static OBJ file and e15the fourteenth animation file described
above. e18retains that set; e19adds custom OBJ graphics and e20adds raw UI palettes and e21adds custom UI graphics; e22extends their localized selectors/payloads; e23adds static item records; current e24adds counted item links.
All3175packed glyph slots and metrics, three keyboard programs/indices and
navigation widths are immutable typed runtime sources. The original routines
still build cell pages and render/copy data with unchanged guest work/clock.
The complete bank owns overlapping earlier text-font reads. Host mouse row
widths use the same typed owner; live selection and selectable codes stay RAM.
Prior e12files remain retained, USA stays e8-d7, and save/native identities and
ABI remain source-equal. Reduced Wine source/authentication/native/lifecycle
checks and one397-frame ordinary transition/actual CLI pass; ordinary keyboard
scene, new Linux/USA and native Windows coverage remain pending.
See the [complete font/keyboard contract](rom_data_text.md#complete-european-font-and-name-keyboard).

## Sources and typed readers

The twelve shape/size records are decoded into typed width/height entries.
Runtime construction parses the captured file bytes, checks their manifest
hash and exact regional source/provenance, and refuses edited base data even
when their file hashes were recomputed. The manifest's relative file names and
supported resource set stay fixed; unsupported data do not silently use ROM.

The independently verified sources are USA direct table 0x149510 and copied-code
alias 0xa398dc; EUR direct table 0x14ced8 and copied-code alias 0xd976e8. Each range
is 48 bytes, with the shared source digest recorded in the
[inventory](development/ROM_DATA_INVENTORY.md). Searching both exact local
images finds exactly these two occurrences per region. The fourth range was
found during the runtime provenance investigation; extractor revision 2
records it. The original USA initializer now has independent clean-boot
execution/provenance coverage, described below; the EUR replay observes alias
reads without attributing them to an initializer PC.
That alias expansion retained schema/decoder1 and selected `v1/e2-d1`,
preserving `e1-d1`, because the OBJ format was unchanged.
The managed metadata extension introduced `v1/e3-d2`; `v1/e6-d5` added exact
palette source views beside readable banks. The `v1/e8-d7` generation also
contains alternate palettes and packet programs; EUR consumes packets while
USA remains verified-only. Earlier revisions including preparation-only
palette e5and e7are retained.
EUR's `v1/e9-d8` adds authenticated main graphics while retaining e8and the
same prior decoded files. Save and native cache identities stay source-equal.
The EUR e10generation adds transfer programs while retaining e9and
the same prior decoded resource bytes.
EUR e11adds the bounded short-text/font subset, retaining e10and all seven
prior decoded files byte-for-byte. See the [text contract](rom_data_text.md).
EUR e12adds the broad text catalogs, retaining e11and all eight earlier decoded
files byte-for-byte.
Manifest/OBJ stay TOML; BG metadata and base palettes use readable JSON.
The borrowed complete-file view authenticates all resources before changing
the live object and includes every resource ID/file hash in dataset identity.
OBJ, descriptors and base palettes remain in the immutable session and report
`runtime=decoded`; alternate palettes also have direct consumption below. BG metadata and palette CPU/DMA/native reads consume
file-derived projections, with independent counters. The palette LZ77 HLE
consumes decoded bank bytes; raw CpuSet keeps its original bus-read loop.
The alternate palette and packet paths are described below. Other linked
payloads still remain ROM until their direct-consumption units.

`RuntimeResources` in `gba-core::rom_data` owns immutable typed entries and
little-endian source projections:48bytes for OBJ,14256bytes for BG. At the
historical three-family boundary, owned memory was448bytes shared/OBJ including Arc counters and identity strings,
plus28672bytes for BG and94713bytes for palette models, decoded byte views,
source bindings and file identity:123833bytes total on both tested64-bit targets. Enforced bounds were declared
before testing:shared/OBJ<=1KiB and extraBG<=64KiB. Historical OBJ-only
measurements below retain their432byte scope. The machine also holds one optional Arc handle; allocator overhead and
process RSS are separate ROMDATA-05 measurements. The resource contains no file
handle, filesystem callback, translation pointer or serialized guest state.
There is no allocation or I/O on resource reads.

Those historical allocation caps no longer reject authenticated attachment.
During the EUR mission-record integration, the shared structure's additional
resource handle crossed the old OBJ cap. Per the requested deferral of space
limits, e86-d85 removes the two attach allocation checks and three corresponding
legacy assertions. Parsing bounds, source/format/dependency authentication and
transactional attachment remain active. No new allocation/RSS or performance
campaign is run; the old measurements retain their historical scope.
The BG projection regression test also checks retained allocation accounting
before and after its reads, rather than reapplying the historical 1 KiB shared
owner and 64 KiB BG caps. Its byte/width/mirror, counter and clock assertions
remain active; adding other resource handles must not fail this BG test.

The ROM arms of `MemMap::read8/read16/read32` check only fixed source
pages:four for OBJ, four BG pages per region and fixed palette asset regions
USA56f..697/EUR76c..894. A regional dispatch supplies migrated bytes, including
unaligned/crossing accesses; adjacent opaque bytes retain the original path.
GPIO/EEPROM, mirrors and bounds/open-bus behavior keep their existing order.
This is not a general manifest/resource lookup on every bus access. DMA already
uses these bus reads, and generated native loads already call the same RtApi
callbacks. No guest instruction, modeled cycle, write, IRQ or DMA is skipped. Verified
palette HLE calls use the already decoded bank bytes and the same ordered
`write_out`; their guest producer/control flow still executes. The source ROM buffer is retained for code and unmigrated data.

Host sprite draw decoding reads the typed dimensions directly. Copied battle
and visibility routines retain their RAM table at IWRAM 0x03006d28: unchanged
records may use the typed file data, while genuinely changed RAM records keep
their guest values and are counted. Shape 3 and out-of-table reads keep their
old opaque-memory behavior. Table validation also uses the decoded source
projection when active rather than rereading the old ROM bytes.

The investigation corrected an existing EUR host-reader error: the two main
draw routines use the direct ROM table, not the copied RAM table. The exact
EUR literal at ROM 0x15ac points to guest 0x0814ced8. Both legacy and decoded
host paths now use that verified source; copied battle/visibility readers
still use RAM. This can correct extended sprite placement/clipping for flips
and off-screen objects. Original guest instructions remain unchanged.

## Identity, snapshots and diagnostics

ROM, decoded file and dataset have distinct SHA-256 identities. The dataset
identity includes source, schema, extractor/decoder revisions and every
prepared resource ID/file hash in the pinned order, including BG metadata.
Only a source-equal unmodified base can enter this runtime API. Native cache
keys and verified ROM profiles remain tied to the original image: generated
loads use bus callbacks and the accepted guest representation is byte-equal.
No RtApi layout, generated C or translation revision changed. Future mods must
establish a separate compatibility/cache/save identity before being accepted.

The machine's optional Arc resource owner and counters are skipped by Serde.
Snapshot payloads and version 1 ROM/BIOS identity remain compatible. Successful
Load retains the live session resource object after transactional validation;
failed loads do not replace it. Slot Load/Undo follows that path. Backup restore
clones the selected resources into its prepared clean-boot machine before
loading the boot state or replacing the save. A running session needs no file
access after activation; a new activation with missing files fails rather than
falling back. Managed startup can regenerate those files from the source ROM.

For the unmodified E170 EUR base, Wine `e170-save-identity-wine-_mjaz8sl`
checks compatibility in both directions using synthetic machine and cartridge
data. Identical raw and managed machine states produce identical snapshot bytes;
each mode loads the other's snapshot, retaining its own resource-owner status.
The normal restart persistence path writes synthetic cartridge bytes in each
mode and the other mode reads them unchanged. Invalid snapshot magic, version,
ROM/BIOS identities, checksum and payload are rejected without changing the
machine; a different ROM cannot attach the EUR dataset. An invalid boot state
also leaves both the machine and the existing synthetic `.sav` unchanged.
Inputs were protected, Wine drained and successful scratch was removed.

The cartridge `.sav` remains the original raw game-format backup data: this
check does not add a dataset identity header or establish that arbitrary saves
belong to the selected game. The result covers direct APIs and the unmodified
EUR base, not GUI operations, actual game-save semantics, mod compatibility or
deferred platforms. Future data modifications that change guest state or save
semantics require an explicit compatibility identity and rejection policy before
activation or save writes.

Activation prints `ROM_DATA runtime=decoded` with resource/source/dataset/file
identities and resident bytes. Shutdown/headless completion prints
`ROM_DATA_USE` counters for OBJ bus, typed host ROM/RAM, changed RAM and
validation, plus separate BG bus operations and projected byte counts.
Palette counters distinguish source bus operations/bytes from decoded LZ77
calls/output bytes. Exact compatibility views cover compressed streams and
raw headers; they are validated against banks rather than recompressed.
`bus_source_mask` bit 0 identifies the direct/primary source, bit 1 the copied
alias. Counters describe use, not timing or speedup. An in-view boot scene can
legitimately show zero guest metadata reads because the original draw routine
fetches dimensions only for clipping/flips; host file use is recorded separately.

## Current base-palette consumption evidence

The162record-indexed models cover127unique sources per region (125LZ77/2raw).
The owner authenticates exact encoded bytes and decoded banks together. Raw
body reads come from bank bytes; compressed/header reads come from the file's
source representation. LZ77 SWI11/12only select a bank at an authenticated
source start, including cartridge wait-state mirrors. Dynamic RAM and other
ROM sources keep the existing decoder. Real BIOS guest code can read every
original stream byte through the bus; no actual BIOS-ROM execution is inferred
from the byte-equivalence checks.

The existing HLE charges no per-source-byte cycles or device effects for these
ROM spans. Both paths execute the same SWI, instruction cost/return hook and
ordered halfword writes, including paletteRAM/VRAM/OAM destination semantics.
This preserves the emulator's modeled timing, not a new hardware BIOS claim.

Linux and Windows GNU/Wine pass210serial core tests each (7optional ignored),
including all widths/mirrors/edges/native callbacks, poisoned source views,
DMA16/32, SWI11/12, raw CpuSet, all destination types and full snapshot equality.
Frontend defaults pass101Linux/100Wine (22optional ignored). Four original
producer cases (EUR/USA ×Linux/Wine) each pass1296calls/72792instructions and
4208native dispatches after poisoning every old source span, including headers.
Interpreter registers/CPSR/clock/RAM match at every step and native boundary;
complete final states match. Each file owner records1272decoded LZ77calls /
231936output bytes and3544source bus reads. Load/Undo/backup, failed binding,
corrupt/deleted files and continued use after deletion preserve the live owner.

New EUR1200-frame native pairs match every full state, image, OAM, dispatch and
mixed/PSG/FIFO/HLE audio on both targets. Each consumes one160byte decoded
palette and53BG reads. Actual CLI checks cover e5refusal/retention, current
extract/verify/create/reuse/repair/move; interpreter/native1200-frame pairs have
positive palette diagnostics and identical legacy frame/final-state hashes.
See [the graphics contract](rom_data_graphics.md#direct-base-palette-consumption).
The subsequent e6recorded-scene checks now pass240worldmap,3530battle/menu
and8476transition frames on both targets, including every full state/image/OAM/
dispatch/audio stream. Stable segments have zero palette reads; the transition
consumes one160Bdecoded palette. See the [scene results](development/ROM_DATA_VALIDATION.md#base-palette-e6-scene-acceptance).
GUI/costs, native Windows and default enablement remain05gates.

## Current BG consumption evidence

The authenticated162-record block atUSA569104/EUR7662a8 now supplies all
fixed-source bus projections. No payload decoder or original producer is
bypassed. Linux/Wine pass204core tests (7optional ignored), including every
width/mirror/edge, native API reads, CPU/DMA and unchanged snapshots. The
frontend defaults pass98Linux/97Wine (18optional ignored).

Real e3files feed2608selector invocations per region/target, with raw descriptor
ROM bytes poisoned after activation. Original interpreter, file-backed
interpreter and actual TinyCC code agree on registers/CPSR/clock/EWRAM/IWRAM;
restoring only the test poison gives identical complete snapshots. Each case
records134424original instructions,5836native dispatches,660event-boundary
interpreter shells,3867native BG byte-load callbacks and3888total BG reads.
No zero-fallback claim is made. Missing/corrupt activation is transactional;
reads survive deletion of the activated BG file, Load, Slots Undo and backup
clean boot with the same resource owner and retained counters.

New EUR native1200-frame clean-boot pairs pass every complete state, central/
wide image, OAM, dispatch and mixed/PSG/both FIFO/HLE audio stream (HLE by bits)
on Linux and Wine independently. Both reach53BG reads; all five audio streams
are nonempty. Existing target-specific TinyCC objects are reused. Actual
production CLI evidence and detailed limits are in the
[graphics runtime section](rom_data_graphics.md#direct-descriptor-consumption).
Broader BG scenes/GUI/costs and native Windows remain separate05gates.

## Retained OBJ evidence and scope

- Linux release suites pass 191 core and 95 runtime tests; Windows GNU/Wine
  passes 191 and 94. Optional local-ROM/native tests are run separately. Tests
  poison the old ROM table bytes and check every bus width, mirror and source
  boundary, native API reads, CPU registers/cycles, DMA and mutable RAM handling.
- A 48-case original-ROM instruction matrix runs both main draw routines in
  EUR/USA with all twelve shape/size records, flips and clipping positions.
  Registers, CPSR, clock, RAM writes and host replacement attributes match at
  every original step. Each region records 239 decoded guest reads and 288
  typed host reads. No original instruction bytes are included in the source.
- Private file/lifecycle tests pass on both targets/regions: file provenance,
  byte-equal pre-activation states, rejected missing-file activation, resident
  reads after file deletion, slot Load/Undo and backup clean-boot resource
  retention. State schema and cartridge save format are unchanged.
- Actual native loops compare all 600 full states, central/wide images, OAM,
  dispatch counts and mixed/PSG/FIFO/HLE streams (HLE by float bits). Linux and
  Windows/Wine each pass against their own legacy reference, using existing
  TinyCC ELF objects. The Windows cache has a `.dll` filename but is loaded by
  TinyCC, not the OS DLL loader. Each run consumes 3,519 typed host records;
  guest-read coverage comes from the original-routine matrix, not this boot.
- Actual `frames` and `runc` commands on both platforms match their corresponding
  legacy 120-frame hashes and report decoded activation/use. ROM/library hashes
  stay unchanged. Actual interpreter GUI runs create, close, move the portable
  folder and reuse the identical generation with positive host file-use counts;
  captures are inspected and exits are clean.

The initial boot gate failed on zero guest reads even while all 600 comparisons
matched. It exposed both a missing EUR copied alias and the host reader's wrong
source. The failed probes are retained; source inspection and the stronger
original-routine matrix establish the correct conditional-read coverage.
The final boot result is reported at its actual scope. A missing TinyCC path
and missing helper working directory were setup failures before useful runs;
existing fixtures/binaries were reused after correcting only those helpers.

The next ROMDATA-05 tranche now passes every full state, central/wide image,
OAM, dispatch and complete mixed/PSG/FIFO/HLE stream for worldmap240frames,
observed battle/menu/enemy3530frames and protected-save boot-to-battle8476frames
on Linux and Windows/Wine independently. Audio observation is configured equally
after loading snapshots; HLE compares bits with actual samples. The copied EUR
alias is read at frame5400, mask3, while guest RAM/clock/RNG remain equal.
The USA initializer trace is not inferred from this EUR replay.

A separate USA test boots without a save or forced PC, using demo inputs,
and reaches the original copier at0808f74c on frame791/clock222325013. Its13
original instructions return to08023f76; every complete state matches a legacy
instance starting from that real caller checkpoint. CpuSet at0814186c copies
3648bytes to IWRAM5f28, including24decoded halfword reads for the48byte table
at6d28. A separate probe poisons only the raw alias after authenticated file
activation: the entire copied blob and final state still match. This passes
independently on Linux and Wine. It proves the original USA initializer's file
provenance, not full-USA scene/audio parity or native Windows execution.

Five native Linux alternating timing pairs versus a corrected legacy binary
without resource routing measure +1.0081% worldmap/-0.2355% battle mean cost;
p95/p99 also pass predetermined limits. This is a neutral result, not a speedup.
Seven first/cached helper observations give median32.416/21.019ms on Linux and
109.222/91.891ms on Wine after source read. Wine cached maximum183.130ms is
retained; the100ms gate applies to the median. Native process peak RSS deltas
are672KiB median/752KiB maxima over seven pairs; owned resources432B. A managed
one-generation root uses1164logical/32768allocated bytes on the host filesystem.
[Protocol, exact costs, comparisons and reproduction](development/ROM_DATA_VALIDATION.md).

Portable archives include these resource help documents. Synthetic Linux tar/
Windows zip marker checks prove the selected inputs exclude neighboring private
resources, ROM/save/state/recordings, native caches and generated code; no new
release export is claimed. Actual decoded-resource GUI native/interpreter
checks now pass on Linux/Wine: held/toggled Fast, pause/resume preserving Fast,
return to1x, Slot Load/Undo/reload and a separate in-menu backup clean boot.
Real callbacks at44.1/48kHz report0device errors, over-period callbacks and
producer drops; muted output and transition-inclusive profiles do not establish
audibility or steady zero-starvation. Protected files and source hashes match.
[Detailed GUI evidence and limits](development/ROM_DATA_VALIDATION.md#actual-gui-and-callback-lifecycle).
Actual Docker archives also pass395Linux/392Windows-Wine tests (24optional
ignored each), privacy/identity checks and extracted-runtime GUI Load/Undo/
reload with automatic bundled TinyCC and fresh portable resources. The Docker
context/COPY lists now include both resource guides required by the packager.
Linux requires at most GLIBC2.35; Windows PE/resources/dependencies are checked.
Native Windows remains before normal enablement; the user currently has only
Linux/Wine. Other families/mods stay open. Canonical runtime and archive bytes are preserved;
a prior BUILD-INFO metadata hash variation is retained in the
[artifact evidence](development/ROM_DATA_VALIDATION.md#actual-portable-archives).
Game-derived test artifacts remain local/ignored.

The user's current priority is functional migration without performance
measurement or space-reduction work; optimization comes later. Prior benchmark/
RSS limits and results are historical, and no performance campaign should be
resumed from an older next-action line. Guest clock/state/audio correctness,
source provenance and save/native compatibility checks remain required.

## Alternate palette banks

The verified `e7-d6`dataset now retains both alternate catalogs in immutable
session resources. Literal payload reads for the original custom decoder come
from fullu16banks; typed relative links and authenticated control metadata
preserve all original guest instructions/writes/cycles. These bus paths serve
CPU, DMA and native API reads without runtime disk access. Native cache and
snapshot identities remain based on the original image; no ABI/schema bump.

`ROM_DATA_USE resource=graphics.bg-alternate-palettes` reports `bus_reads`,
`source_bytes`, `table_bytes` and `decoded_literal_bytes`. The last counter
proves decoded bank use independently of preparation/control-byte reads.
Positive original/native poisoned-source and separate native-API bank probes,
full-state/clock equivalence and deleted-file Load/Undo/backup/restart pass on
Linux/Wine. The translated path includes existing interpreter shells; a cold
first-frame CLI check is activation/cache evidence with0alternate reads.
See the [complete contract and results](rom_data_graphics.md#direct-alternate-palette-consumption).
Original EUR Color Mode LCD B/TV now has actual Cyril-pub source-use evidence:
four397-frame Linux/Wine legacy/decoded native pairs match every full state,
image/OAM/dispatch and audio stream. Actual GUIs present both modes and close0
with positive typed link/literal reads. See the [scene protocol and limits](development/ROM_DATA_VALIDATION.md#alternate-palette-e7-scene-and-gui-acceptance).
Global FFpreconditions, broader regional/gameplay/catalog/language coverage,
archives and native Windows remain separate. No performance measurements were selected.

## Packet dataset activation and EUR consumption

The e8-d7six-file set now includes typed ordered packet programs in preparation,
source verification and complete dataset identity. Packet activation failures
preserve the current owner/guest state; a source-equal file annotation changes
the complete identity, leaving prior family identities and guest snapshots
unchanged. [Managed packet checks](rom_data_graphics.md#managed-packet-program-datasets)
pass with four focused Wine cases and seven actual Windows CLI operations;
old cache dispatch is positive in a single cold activation frame. This is no
packet-use proof. Subsequent04now binds actual EUR reads: the owner retains
typed ordered writes, source/body control metadata and shared decoder-derived
literal maps. Raw packet values and LZ literals are reconstructed from typed
words; existing LZ77HLE writes the predecoded typed body through its normal
destination writer. CPU/DMA/native/guest-BIOS bus reads retain exact original
bytes. The fixed EUR asset pages include895, and reads crossing adjacent
palette/packet spans combine both file-derived views.

No guest packet instruction, dynamic initial fill, inheritance, continuation,
write or cycle is bypassed. Mutable RAM and unregistered sources retain their
original paths. Host resources/owned reads survive Load/Undo/backup/restart and
unavailable source files without entering snapshots. Schema/extractor/decoder
remain1/8/7; native/save/profile/cache identities remain source-equal.

Three focused Wine checks pass: a synthetic boundary/SWI/DMA/owner test, nine
EUR original/native calls with every packet source span poisoned, and a
temporary-copy lifecycle test. The nine calls match complete states/clock,
destinations, guards and stack, with18compiled typed bytes and10506typed HLE
bytes. One397frame LCD B Cyril-pub transition matches every full state, image,
OAM, dispatch and mixed/PSG/FIFO/HLE sample, consuming2640typed packet bytes
through three HLE calls. Its raw typed bus count is0; the component probe
provides that distinct evidence. See [the full contract](rom_data_graphics.md#direct-european-packet-consumption).
New Linux/USA, broader scenes, GUI/archive and native Windows checks remain
pending. No performance/space campaign or normal-enablement claim is made.

The [EUR task descriptor resource](rom_data_tasks.md#managed-task-descriptors-and-typed-guest-reads)
in e48-d47adds43rdfile/42ndresource, retaining all41prior decoded files/e47. Three
catalog spans/3296B reconstruct nullable callback words, context sizes and raw
metadata; no guest ID cap/allocator/scheduler/save-native ABI change. Two reduced
Wine checks pass13guards/lifecycle/alltypedbytes/412records3catalogs and29original-
interpreter-native complete-state-clock prefixes, including raw/cross-catalog
aliases/previousblendowner and boundaries-BIOS-SWI-DMA. CompiledAPI reads44callback/
22context/9metadataB, aggregate coverage only. ActualproductionCLIcreate/reuse-
no-write45managedfiles/oldcache andone1000WinePecorelle fullstate-clock-classic-
image-OAM-dispatch-all-audio pair pass28aggregateB=16callback8context4metadata/
4records2catalogs; other408records/thirdcatalog targeted-only. Source-model oracle
stays closed; new32:9/Linux/USA/nativeWindows pending, no wholeABI claim.

[Effect/event directories](rom_data_events.md#managed-publication-and-typed-guest-reads)
in e49-d48add44thfile/43rdresource and the actualengine.task-descriptors dependency,
retaining42prior decoded/e48/USAe8-d7.1516B/four bindings onpages124/128/3ac/3ad
serve typed event/selector/even-MOV-PC fields without a byte shadow, new liveindex
cap, hostsequencer or save/native ABI. Two reduced Wine checks pass13guards/
lifecycle/alltypedbytes and29original-interpreter-native fullstate-clock prefixes;
compiledAPI reads[24event6selector24dispatch]aggregateB. Raw/defaults/u16wrap/font
alias11B/samepagepalette4B/BIOS-SWI-DMA/new2raw2 pass; no wholeinterpreter/
reachability claim. ActualCLIcreate/reuse-no-write46managedfiles/oldcache passes.
Pecorelle1000/schoolcoldboot3000/suspendboot-to-battle8476 locators readzero
newbytes: closednegative/no event-requiredfullpairs. New normal EUR progression
reaches a genuine frame39420bedroom/book snapshot. One1800-frame Wine locator
and one fullstate-clock-central/wide-image-OAM-dispatch-all-audio pair on the
actual e52generation pass5aggregateB=1selector+4dispatch/one selector/one table,
470hash-mtime guards. Eventrows0/other50selectors/other table remain targeted-only;
Linux/USA/nativeWindows/new32:9 remain pending. No product rebuild/performance
measurement. [Ordinary scope](rom_data_events.md#positive-ordinary-book-transition).

## European script command directory fields

EUR e50-d49 adds `script-command-directory.json` / `engine.script-command-directory`
with115commands/690source bytes on existing pages3ac/3ad. Typed u16 advances and
Thumb callbacks reconstruct original field bytes; prior event/selector owners
and raw lookup bytes merge at mixed boundaries. Guest signed cursor/u8 opcode/
relative targets/advance wrap/BL-BX/return loops stay original. The directory has
no asset dependency, while source-set coherence requires the preceding owner.
Save/native ABIs are unchanged; counters remain outside serialized state.

Two reduced Wine filters pass13managed guards/publication/lifecycle/all690B and
26original-interpreter-native complete-state/clock prefixes. Compiled API reads
14advance/28callback aggregate bytes; adjacent prior events4B/selectors4B,
previous2+new2/new2+raw2, widths/mirrors/real BIOS/CpuSet/FastSet/DMA pass.
314instructions/70PCs/32native dispatches are coverage only. This is controlled
field/address/call-prefix integration. ActualCLIcreate/reuse-no-write47managedfiles/
oldcache andone8476-frame Wine EURsuspendboot-to-battle fullstate-clock-image-OAM-
dispatch-audio pair pass42B=14advance28callback/7commands. Other108commands remain
targeted-only; new32:9/Linux-USA-nativeWindows pending. [Evidence and limits](rom_data_scripts.md#managed-publication-and-direct-command-fields).


## European relative script archive

EUR e51-d50 adds `script-archive.json` / `engine.script-archive`,
46files/45resources, retaining all44earlier decoded files/e50 and USAe8-d7.
Activation checks real command/effect-directory dependencies. Two source bindings
cover115464bytes on pagesB94..BB0/D68; typed top/nested index words, opcodes,
parameters, keys and fills serve original CPU/DMA/BIOS/native reads. Per-byte
merging preserves the palette-effect owner on pageB94 and adjacent raw data.
No new guest ID bounds, host sequencer or save/native ABI; counters remain
outside serialized state.

Two reduced Wine checks pass13guards/lifecycle/all115464bytes and28original/
interpreter/native complete-state/clock prefixes. Compiled APIs read37index/
4opcode/7parameter/12key/1fill aggregate bytes. Raw0/234/239/65535/low16/unaligned
rotation/header-fill targets/q0/relative-target/return-loop, previous-owner4B,
widths/mirrors/realBIOS/CpuSet/FastSet/DMA and mixed raw2+new2/new2+raw2 agree.
169instructions/66PCs/28dispatches are coverage only. Actual production CLI
create/reuse-no-write48managedfiles/oldcache passes. One retained ordinary
8476-frame EUR locator reads27archive bytes (8index/7opcode/10parameter/2key),
one bank/program/key and zero fill. One Wine8476-frame pair from that CLI
generation passes every full state/clock, central/wide image, OAM/dispatch and
mixed/PSG/FIFO/bitwise-HLE sample; all366hash-mtime guards pass.
Other61banks/206programs/233keys/all45fills retain targeted coverage. New32:9/
Linux/USA/native Windows remain pending. [Evidence and limits](rom_data_scripts.md#archive-production-cli-and-ordinary-acceptance).


## European script-selected localized text

Current EUR e52-d51 adds `script-text.json` / `text.script-archives`,47files/
46resources, retaining45prior decoded files/e51and USAe8-d7. Actual script-key/
font dependencies validate before activation. Five bindings/1803744bytes serve
typed top/variant/bank indices, headers, encoded controls, decoded glyph/command/
operand/terminator fields and exact slack. Shared `TextProgramMap` maps encoded
literals to typed units for both old and new text owners; old categories remain.
Decoded tails are preserved but have0direct source literals, being generated by
original LZ controls/backreferences. Guest expansion/live RAM/clocks/save/native
ABI remain original, counters and coverage masks stay outside saves.

An affected shared-mapper literal-poison Wine check and two new managed/native
filters pass13guards/lifecycle/allbytes/all masks and35original/interpreter/
compiled complete-state-clock prefixes. Compiled categories
[21,17,17,13,6,4,5,1,3,0,3]B, raw selectors/wrap/sentinels/BB0-D68prior-owner merging/
raw after old key owner/width-mirror-realBIOS-CpuSet-FastSet-DMA agree.163real
old-format0/1/2program source bytes verify the shared map;379instructions/65PCs/
71dispatches count coverage only. Production CLI create/reuse-no-write49files/
oldcache passes. The8476battle context is negative0new text/no fullpair; the
retained school3000locator is positive289B across one language/bank/two programs,
zero variants/operand/tail/slack. One Wine3000-frame pair from that CLI generation
passes every full state/clock, central/wide image, OAM/dispatch and mixed/PSG/FIFO/
bitwise-HLE sample; same positive source use and468hash-mtime guards.
Other records retain targeted coverage; new32:9/Linux/USA/native Windows pending.
[Evidence and limits](rom_data_text.md#script-text-production-cli-and-ordinary-school-dialog).

## European coordinate path fields

EUR e53-d52publishes `script-coordinate-paths.json` /
`engine.script-coordinate-paths`,48files47resources/all46prior decoded/e52
retained/USAe8-d7. Actual script-archive/command dependencies authenticate68links;
one3056Bbinding serves typed relative indices/u16counts/XY/alignment through the
shared immutable owner. No host movement routine, ROM byte shadow, new guest
index/count cap or save/native ABI change. The original reader retains live RAM
height and low16/low8control. D68merges previous text, keys and the new directory;
rawD69BC4outside data stays live.

The new managed Wine filter passes14coherent/missingfile guards/both dependencies,
create/reuse/repair/move/e52retention/Load-Undo-backup-Restart/unavailable files,
all3056typedB/210paths/856points/42alignment. Corrected native-observer filter
passes24original/interpreter/compiled complete-state-clock cases, including four
full copies/live height/raw arguments/allfour compiled classes[28,85,76,2]B,
triple-owner boundaries/widths/mirrors/realBIOS/CpuSet/FastSet/DMA. Initial new-test
array/observer errors are retained and corrected; successful core/managed/source
proofs are reused. No whole movement/allocator ABI claim.

Actual productionCLIcreate/reuse-no-write50managedfiles/oldcache/all48file
equality and one6000-frame genuine EUR snowball/transition pair pass every full
state/clock/central-wide image/OAM/dispatch/mixed-PSG-FIFO-bitwise-HLE sample.
Source use28B=4index14count10XY/one path,0alignment;581hash-mtime guards. Other
209paths/42alignment records targeted-only; newLinux-USA-nativeWindows/new32:9
remain pending. No performance/space measurement.
[CLI and ordinary scope](rom_data_scripts.md#coordinate-path-production-cli-and-ordinary-scene).


## European auxiliary script directory and archive

EUR e54-d53 publishes two separate files/resources for the auxiliary directory
and u16-relative archive,50files49resources/all47prior decoded/e53retained/
USAe8-d7. The archive authenticates its own actual directory dependency before
one immutable owner serves typed advance/callback/index/opcode/parameter/
alignment fields. No ROMshadow/hostscript/newguestcap/save-nativeABI change.
Sourcepage370 retains audio owners and D69 merges preceding coordinates with
auxiliary indices; raw370FFA+/D6ABC6+ remain original fallback.

One warning-free affected Windows build and two new Wine filters pass first
execution:26coherent/missingfile guards/create-reuse-repair-move/e53retention/
unavailable-files/Load-Undo-backup-Restart/sameowner/all4200B.44original/interpreter/
compiled complete-state-clock prefixes pass allsix APIclasses[6advance65callback
18index19opcode15parameter2alignment]B/interior entries/signedcursor/wrap/rawopcodes/
raw8:1u32LDRH/fullaction2vs102/mixedowners/edges-widths-mirrors-realBIOS-CpuSet-
FastSet-DMA.624instructions92PC52dispatchescoverageonly, not wholepredicates/
interpreter/callbackABI/ordinary for the targeted checks. ActualproductionCLI
create/reuse-no-write52managedfiles/oldcache/all50generation equality now passes.
One6000frame genuine EUR ordinary pair passes every complete state/clock/image/
OAM/dispatch/mixed-PSG-FIFO-bitwise-HLE sample, directory96B=32advance64callback/
6commands, archive82B=16index20opcode46parameter/2banks2programs/0alignment;
640hash-mtime guards pass. Other11commands/6banks/131programs/5alignment spans
remain targeted-only. NewLinux-USA-nativeWindows/32:9pending, no performance/
space measurement/newfixture/navigation. [Evidence](rom_data_scripts.md#auxiliary-script-production-cli-and-positive-ordinary-scene).

## European map raster palettes

EUR e55-d54 publishes `map-raster-palettes.json` / `graphics.map-raster-palettes`:
51generation files/50resources, all49prior decoded files unchanged/e54retained,
USAe8-d7. Actual BGdescriptor links authenticate before one immutable typed
index/header/color owner serves33ranges10496B. New pages487..489overlay only
owned bytes; at489BB0the earlier alternate-palette owner retains its index word.
Original signed/unsigned selectors, raw arguments, live windows,322Bcopies,
extra read/wrap/clock and DMA scheduling remain guest work. No ROMshadow,
host raster/DMA interpreter, guestcap or save/native/cache ABI change.

One warning-free affected Windows core/frontend build and two new Wine filters
pass first execution:17guards/publication/lifecycle/create-reuse-repair-move/
e54retention/unavailable-files/Load-Undo-backup-Restart/sameowner/all10496B.
28original-interpreter-compiled paths match complete states/clocks with both
new sources and adjacent314Bpalette index poisoned, compiled[15index46header
24color]B. Immediate/deferred/raw/sentinel/queuedDMA/copy-wrap, mixedowners/raw
neighbors and widths/mirrors/realBIOSbusmode/fourSWIs/oneimmediateDMA pass.
638instructions204PC55dispatches are coverage only, not wholecamera/rasterDMA/
ordinaryABI.646hash-mtime guards pass. ActualproductionCLIcreate/reuse-no-write
53managedfiles/all51generation equality/oldcache and one1500frame genuine
coldboot Winefullstate-clock-image-OAM-dispatch-audio pair nowpass,50880aggregateB=
2index322header50556color/25440busreads/programmask2 (neighbor-header touches
included, not two full palette initializations/distinctB). Audio samples
[3291096,3291096,395212,395212,3269758],701guards. Othercontexts/wholecamera-rasterABI/
newLinux-USA-nativeWindows/new32:9 remain pending. No performance/space or new
fixture/navigation work. [CLI and scene](rom_data_maps.md#raster-palette-production-cli-and-positive-ordinary-boot).


## UI choice lists

EUR e56-d55 publishes `ui-choice-lists.json` / `engine.ui-choice-lists`, with
52generation files/51resources/all50prior decoded unchanged/e55retained; USAe8-d7
unchanged. Thirteen immutable typed spans/336B expose original identifier,
unsigned cursor X/Y, opaque byte3and four full raw navigation bytes. Per-byte
composition preserves previous same-page OBJ owners and raw gaps. Original
signed copy counts, filtered RAM lists, overrides, input/cursor/tasks and
source-independent RAM reads remain guest work. No raw shadow, host navigation
replacement or guest count/target cap; save/native identity and ABI unchanged.

One warning-free affected Windows core/frontend test build and two focused Wine
filters pass17guards/full-set coherence/lifecycle/all336typedB=[42ID84coordinate
42opaque168nav]/13lists/transactional attachment/unavailableLoad-Undo-backup-
Restart. Twenty-nine original-interpreter-compiled full-state/clock component
paths pass with all336new+14oldOBJrawBpoisoned; compiled aggregate categoryB=
[92,183,92,368]. Widths/mirrors/realBIOS bus/3SWI/1DMA and mixed2old+2raw+2newB
copies pass.706hash-mtime guards;3010instructions186PC38dispatch coverage only.
No completewidget ABI claim. ActualCLIcreate/reuse52generation54managedfiles/
oldcache now passes no-write, and one320frame ordinaryParty complete-state-clock-
image-OAM-dispatch-audio pair passes32aggregateUIB=[4ID8coordinate4opaque16nav]/
1list/769hash-mtime guards. Other12lists'ordinaryuse/newLinux-USA-nativeWindows/
new32:9 remain pending. Pecorelle1000/Moogle600are terminal0UI/no fullpairs. No performance/space, new fixture or
navigation. [Contract and evidence](rom_data_ui.md#managed-publication-and-typed-runtime-reads).

[Actual CLI/Party scene](rom_data_ui.md#production-cli-and-positive-ordinary-party-scene)
reuses authentic existing snapshot/input and the same accepted dataset; no
source/model/native/oldscene acceptance is reopened.


## Equipment OBJ poses

EUR e59-d58 publishes `unit-obj-equipment.json` / `graphics.unit-obj-equipment`
with actual `gameplay.item-records` dependency,55generationfiles54resources/
all53prior decoded-resource records/e58 retained/USAe8-d7. The immutable typed
owner serves exact directory/index/null/header/frame/OAM/pixel bytes; packed
pixels come from decoded palette indices. Temporary source buffers are dropped,
raw holes and earlier owners compose per byte. Original facing, commands0/6/8,
RAM task pools, scheduler, DMA, VRAM, save/native ABI and cache stay active.

One warning-free Windows core/frontend test build and two new exact Wine filters
pass on first execution:17guards/publication/transaction/lifecycle/create-reuse-
repair-move/unavailable Load-Undo-backup-Restart/sharedowner/all110666typedB/
258sequences. Twenty-six original/interpreter/compiled paths match complete
states/guestclocks with all new sources plus24oldOBJ-initialB altered. Compiled
API aggregate category bytes `[32,24,6,47,32,1]` cover directory/index/header/
frame/OAM/pixels; actual DMA equivalence is checked separately. Fullword raw
selectors, nulls, live RAM frame,593edges/widths/mirrors/realBIOS bus/2SWI/1DMA
pass. Mixed2new+1028raw+8oldOBJ B charges2/8; unalignedAPI retains2edited raw B.
831build/833runtime hash-mtime guards and eleven source inputs are intact.
1511instructions502PC97dispatch coverage only, no whole allocator/scheduler or
ordinary ABI claim. Source/model/oldproofs not rerun. ActualCLI create/reuse-no-
write55generation/57managed files passes from unrelatedcwd/oldcache; retained
Sheep1000/Fight700 are equipment-negative/no fullpairs. Authorized copiedEURPolly
→Hildegard confirmedFight500singlefullpair now passes allstate-clock-image-OAM-
dispatch-audio/4477equipmentB/1408pixelB/two sequences exactlymatchingactuale61CLI.
Other256ordinary sequences and newLinux-USA-nativeWindows/32:9 retain pending
coverage. No performance/space campaign.
[Contract and targeted evidence](rom_data_unit_obj.md#equipment-managed-publication-and-runtime-reads).


EUR `v1/e61-d60` adds `unit-obj-alternate-battle.json` /
`graphics.unit-obj-alternate-battle`,57files56resources/all55prior decoded
resources and e60retained/USAe8unchanged. The immutable typed owner serves only
3321new spans410086B with actual metadata52/14/initial/equipment/normal
dependencies. All20shared index-root targets stay explicit non-sequences under
older ownership. Two reduced Wine filters firstPASS20managed guards/lifecycle/
all new bytes/606sequences and34original/interpreter/compiled complete-state-clock
paths, genuine extra-part nulls/shared-root prefixes/commands2-3-8/uploads-DMA/
all3321edges/BIOS-SWI-DMA/mixed raw-oldowner reads. StrictNull parsing rejects
extra fields while canonical valid JSON remains identical. Source/model/managed/
native and actualCLI children are closed. Actual Wine CLI creates/reuses all57
accepted generation files/59managed files without rewriting; one-frame oldcache
attachment passes with zero new alternate use. Positive ordinary alternate use,
script/live reachability and new Linux/USA/nativeWindows/new32:9 remain pending. No performance/storage campaign
or new gameplay checkpoint.
[Alternate runtime contract](rom_data_unit_obj.md#alternate-managed-publication-and-typed-runtime-reads).


## Bounded native ROM API residue inventory

`native_rom_data_residue_tests::native_api_rom_residue_preserves_bounded_scene`
is an opt-in, ignored frontend test for finding native ROM API reads outside
an accepted manifest's source plans. It replaces present entries in a test-only
copy of `BlockTable` with a trampoline. The original table and library stay alive;
only `RtApi.read8/read16/read32` are wrapped. The production dispatcher, normal
API, resource owners, extraction revisions and save formats are unchanged.

The observer records unique address/width/context tuples, physical ROM offsets,
per-byte image and unowned masks, and the exported PC and outer dispatch key.
It subtracts sorted merged source intervals from the authenticated manifest.
No read values, durations, cost census, startup/RSS or storage budgets are
collected. An exported PC can be stale, and an outer dispatch key can precede
internally chained blocks; neither is automatically the load instruction.
Interpreter fallback, host/HLE/direct DMA reads, folded native constants,
requests beginning outside the physical ROM image and other scenes are outside
this inventory. Adjacent observed bytes do not establish an asset's extent.

The first EUR check used the existing Bank checkpoint/Down input for 12 frames
(guest3660–3672), the accepted e71-d70 dataset/65resources and retained native
cache. One Windows frontend build was warning-free. One Wine filter passed
per-frame serialized Machine (including clock), central/wide buffers, OAM,
native/fallback dispatch and mixed/PSG/FIFO-left/FIFO-right/HLE audio, including
float bits. All five sample counts were positive. Dispatch totals19962/85770
are execution coverage only. This is observer transparency, not a replay of
the already closed Bank120 resource-equivalence acceptance.

The observer saw3166 unique context reads over3683 physical image bytes;
198context reads include676unowned bytes in79observed contiguous ranges.
Five high-offset words at370E70/74/80/84/EB0 and lower executable-region reads
are source leads requiring original reader/domain authentication. They are not
new decoded assets or evidence that every other ROM byte is migrated.

All73selected input hash/mtime guards, including source/copy checkpoint, input,
ROM, native cache, canonical executable, TinyCC and66accepted dataset files,
remain exact. Production hub changes are exactly one `cfg(test)` declaration;
main, C API, existing validators and all resource revisions remain unchanged.
Parallel documentation/artifact-retention work is preserved separately. The
required local-artifact completion helper found no disposable copies in this
unit. No new checkpoint, navigation, cartridge write or performance campaign
was performed. Linux, USA, native Windows, other scenes and new32:9 remain
pending. Local results: `target/rom-data-native-rom-data-residue/`.


## High-offset native residue is hardware and branch support

The five high-offset words from the Bank12 native API report now have a static
source contract, authenticated against six complete EUR code blocks:392halfwords,
47critical pins, five source-base literals and four BLtargets. Exported PCs
were used only to locate the readers; actual loads are0846/09B2/0D56,3A7E and
11DA/11EE. No additional game execution was required.

The conditional source plans are two16Bdirectories at370E68 and370E78, plus
one4Bword at370EB0. Original D40 and0918 loops prove indices0..3 for the first
directory. Original0800selection also bounds its selected index0..3 and forwards
it to3A6C; D40 forwards its bounded index too.3A6C itself has no input guard, so
these callers do not impose a global cap on that routine or establish a whole
physical table extent. The directories both contain four DMA register block
addresses; original code writes base/+4/+8 or polls bit31 at base+8. These are
hardware destinations, not pointers to ROM graphics or other asset payloads.
The local bus register decoder independently identifies the four12B DMA blocks.

11CC loads the firstword at370EB0 into r2 twice, then calls145C10. That exact
helper is `BX r2`, establishing an indirect branch rather than a fill-value
argument. The word selects Thumb IWRAM entry03005E79. This classification does
not authenticate the mutable RAM body, its copy origin, full callback ABI or
scheduler behavior. No RAM code or mutable state is exported as a base asset.

The36B conditional plans are disjoint from all65existing owners; only20B/five
words were actually observed in the Bank12 report. Those20B are now classified
as executable hardware/branch support, while656observed lower bytes remain a
separate source-classification task. At that source milestone all676bytes remained outside exported owner plans:
classification did not itself turn them into managed resources. E189 later
integrates the two DMA directories as described below; the separate helper
vector and other original ROM/code support remain open.

Exact ROM, static tool and saved address-report hash/mtime guards pass. No
build, game test, checkpoint, navigation, performance measurement or replay of
closed acceptance occurred. Static disassembly is not Linux game validation.
e71-d70/65resources and USAe8-d7 remain unchanged; Linux/USA/native Windows and
new32:9 checks remain pending. Local source evidence:
`target/rom-data-native-residue-high-word-readers/source-contract.json`.


## Lower native residue has literal and switch-table witnesses

All656lower observed bytes from the same Bank12 report now have bounded static
reader witnesses. The raw candidate map was insufficient on its own: for each
of158literal words (632B),184saved contexts were matched to an original Thumb
PC-relative load within96B of the exported PC. Each intervening halfword was
checked against the exact ROM, with no branch, call, return, PC-writing operation
or trap before that load. The184paths contain507distinct instruction halfwords.
These are static original-code witnesses consistent with the saved contexts;
the observer did not capture a dynamic instruction/mode trace, so this does
not establish exact dynamic load-site attribution or every use of each word.

The six remaining observed words (24B) belong to five guarded switch tables.
82prefix halfwords establish the bounds, index shift, original table-base
literal, word load and `MOV pc,r0` for each. Conditional projections are8entries
at022274,35at02540C,69at0CB818,48at0CBEF8 and5at125430. Only entry0,entry0,
entry27,entries4/6 and entry1 respectively occurred in this report. The fourth
reader truncates its selector to8bits; values above47 remain in the original
compare/branch loop. The bounds establish conditional address plans, not
whole-table identity, all callers or ordinary coverage of all165entries.
Unused target values and branch bodies were not decoded into an asset model.

Together with the separately authenticated high-offset hardware/branch support,
all676observed unowned bytes have these scoped static code-support witnesses.
At that milestone they remained outside the65managed owner plans, and no new
game asset, decoder, resource, owner or revision was inferred. E211 later owns
the five switch tables and the separately observed `0a1150` table; E212 owns the
153 previously unowned literal words in this witness set, as described below.
Other code-support dependencies and whole-game/ROM coverage remain open;
no global coverage of interpreter, HLE, DMA, folded constants or other scenes
is claimed.

Exact source ROM, static tool and retained address-report/summary hash-mtime
guards pass; implementation is unchanged. No build, guest execution, test,
fixture, navigation or performance measurement was added. Prior Bank12/Bank120,
source/model/runtime/CLI acceptances were not replayed. Static decoding is not
Linux game validation; Linux, USA, native Windows and new32:9 remain pending.
Local evidence: `target/rom-data-native-residue-lower-readers/`.


## Second native API sample from the existing F09 checkpoint

The same frozen observer executable was reused without a build for a second
12frame sample: the authentic existing F09 checkpoint/original input, guest8476
through8488, same accepted e71-d70dataset/native cache/16:9/zoom0 settings.
One Wine filter passed every full Machine/clock, central/wide framebuffer, OAM,
dispatch and all five audio comparisons. Samples were positive in all streams;
dispatch88123/279744 is mixed execution coverage, not a performance measurement.
No old F09resource-equivalence criterion or Bank acceptance was replayed.

The observer recorded13671unique context reads over13961physical image bytes.
184context reads include604unowned bytes in66observed contiguous ranges;
584source bytes overlap the Bank report and20bytes/five words are additional:
005B0C,0226F0/4/8 and023080. All five additional contexts have original Thumb
straightline PC-relative load witnesses (20distinct halfwords) with the same
static-versus-dynamic limits as the Bank witnesses. Existing source-address
classification is reused as source evidence, not a proof of every current use.
The combined two-scene unowned source-address union is696B, still outside
managed plans. No new game asset/resource/revision is inferred.

All76actual input hash/mtime guards and compiled implementation hashes remain
exact, including the frozen executable and retained Bank classification reports.
The required local-artifact completion helper succeeded with zero disposable
paths. No build, new fixture, navigation, cartridge write, timing/cost/RSS or
storage-budget campaign occurred. Interpreter/host-HLE/direct DMA/folded
constants/exact dynamic instruction and mode attribution/other scenes remain
outside this API inventory. Linux, USA, native Windows and new32:9 remain pending.
Local results: `target/rom-data-native-rom-data-residue-f09/`.


## Optional bus and host ROM read audit

The `rom-data-read-audit` Cargo feature in `recomp` enables the matching
`gba-core` diagnostic feature. It is absent from the default feature set, as
confirmed by locked offline Cargo metadata. Its thread-bound `Capture` guard
collects unique reader/address/length requests in a thread-local set and resets
on finish or drop. It records no values, guest state, clocks or durations.
There are no new Machine/MemMap fields, native C API or save-format changes.
The default executable was not rebuilt for this diagnostic unit.

Feature-gated hooks record successful physical-ROM reads through the normal
bus (native APIs, interpreter data loads, BIOS and DMA), MP2K host values, sprite
peek/typed lists and OBJ-table/keyboard-width helpers. Existing region/bounds/
GPIO/EEPROM and typed-owner/raw-fallback choices are preserved. Only interpreter
instruction fetch/decode is suppressed; execution and HLE are observed after
the guard ends. Code/signature authentication, unselected direct host access,
other threads/scenes, folded constants and managed typed HLE fast paths remain
outside the collector. Reader tags identify access points, not exact guest
instructions, actors or asset extents. The audit does not replace the source
reader/domain proof required before a new decoder or owner is added.

The focused ignored frontend test is
`host_rom_read_audit_tests::bus_and_host_rom_reads_preserve_bounded_scene`.
Compile the Windows frontend tests with `--features rom-data-read-audit`, then
select only that test on Wine. It needs the existing ROM/native/state/input/
accepted-dataset paths and a fresh `FFTA_ROM_READ_AUDIT_REPORT` path. All fixtures
and reports remain private. Normal builds have no collection hooks; removing
the feature blocks restores exact mem/exec/MP2K preimages. Getter return shapes
retain equivalent typed/fallback behavior, and large hub formatting/CRLF stays.

The first feature build compiled the collector but failed on a missing frontend
test `c_void` qualification. After that test-only correction, the incremental
Windows build succeeded without diagnostics. One Wine filter passed: actual
original ARM/Thumb instruction-fetch exclusion in uncached/cold-cache paths,
bus widths/three mirrors/audio tag/capture lifetime, and12existing F09frames
8476–8488 comparing every serialized Machine/clock, central/wide framebuffer,
OAM, dispatch and mixed/PSG/FIFO-left/FIFO-right/HLE-float-bit audio. All five
sample counts were positive; dispatch88123/279744 is coverage only.

The scene has23061unique requests over19984physical image bytes:
15122Bus,7924AudioValue,1ObjSourceTable,6SpriteList and8SpritePeek. KeyboardWidths
was not exercised.1146source bytes are outside65current manifest owners;64of
them are also read by SpritePeek. All13961image bytes/604unowned bytes from the
closed native API F09sample are included, adding542unowned bytes. This comparison
uses the retained report without rerunning its acceptance.90new bytes at high
offsets remain concrete source leads:371065–67,394E14–1C,394F4C–66,397F80–84,
397F88–94 and40D89E–C4. They have no inferred format, table extent or new resource.

All76execution input hash/mtime guards, frozen executable/compiled sources and
one retained comparison-report guard pass. The artifact completion helper found
zero disposable paths. Existing datasets, native cache, original checkpoints,
input and canonical executable are unchanged. e71-d70/65resources/USAe8-d7 remain;
no new fixture, navigation, previous acceptance replay or performance/storage
campaign. New Linux/USA/native Windows/new32:9 remain pending. Local results:
`target/rom-data-host-rom-read-residue/`.


### Bounded audit tools

[DBG-06](debug_rom_audit.md) now exposes the collectors through replay profiles,
CLI `debug-rom-audit create/run/cancel` and menu Audit replay. An additive
`TraceCapture` limits unique events, filters readers/physical ranges/windows
and records actual typed/raw serving, first/last guest frame and interpreter
scope. Legacy Capture/SiteCapture keys and projections remain unchanged.
The optional hooks preserve default feature-disabled code and guest/save/native
ABI. Context inherited by HLE/DMA remains distinct from a proved direct operand.

The report deduplicates physical byte unions across mirrors and widths, compares
an explicitly identified source plan, and states dropped requests, partial
windows and missing hooks. A known owner does not imply actual typed service;
outside-plan bytes remain source leads. The accepted source/runtime extension
campaigns above were not replayed to expose these tools. Usage and the new
bounded raw/typed state-clock checks are in the linked feature guide.

## High bus and host residue source contracts

All90new high-offset bytes from the F09 bus/host audit have scoped original
reader contracts.64B are two [ordinary indirect OBJ lists](rom_data_obj.md#observed-indirect-obj-list-sources-beyond-the-direct-catalog)
whose later model, publication and bounded ordinary validation are closed
below. The remaining bytes are:

- Timing thresholds at371065/66, witnessed by018DDA. The source base is371064;
  a live phase byte indexes it before any normalization. Original code stores
  counter+1 but compares the OLD u16counter with the u8threshold; on advance it
  increments the phase, resets a new phase above2 to0 and clears the counter.
  Conditional phases0..2 follow that update, not a global prelookup3cap or a
  whole physical-table extent. Only phases1/2 were observed. Threshold bytes
  are not converted into invented frame-duration units.
- Four signed16corrections at394E14..1C. Original0264EC..F8 reads a byte at
  task+7, clears its low bit for addressing and subtracts the selected signed
  word at394E0C+2*(byte>>1) from projected r2. The original query range is0..127;
  only new indices4..7 are declared here, following existing typed indices0..3.
  This is no new guest cap, whole-table or all-use/dynamic-PC claim.
- Four words at397F80/88/8C/90. Original loads branch through145C0C/10/18
  (`BX r1/r2/r4`) to Thumb IWRAM entries. The last two select the existing
  visibility/draw readers030062EC/030065A8. They are executable control support,
  not pointers to exported graphics payloads. No whole vtable or full tree,
  allocator, scheduler or callback ABI is inferred.

At this source milestone, the ten parameter bytes were unmodeled and all90bytes
were outside the e71owner plans. Later64bytes become typed OBJ lists in e72 as
recorded below. The ten parameter bytes now have an
[isolated typed model](rom_data_gameplay.md#isolated-model-for-observed-task-parameter-fragments),
with27guards/18original prefixes in one Wine filter. Later e73 managed/runtime
integration serves exactly those ten bytes, as recorded in the parameter notes.
At that source milestone, the 16 control bytes retained their original ROM/code
dependency. E188 subsequently integrates those four words and a separately
observed fifth; see the [scoped runtime proof](#e188-observed-iwram-obj-callback-words).
The lower452new audit bytes remain separate. No resource, owner or revision was
introduced by the source milestone. One bounded producer-argument Wine frame was
needed to identify the64B list readers, with all selected inputs preserved;
old collectors/Bank/F09 equivalence and ABI oracles were not rerun. Static
disassembly is not Linux game validation. No build/new fixture/navigation or
performance/storage measurement; Linux/USA/native Windows/new32:9 remain pending.


## Observed indirect OBJ runtime extension

The existing static-list owner now serves two additional authenticated source
spans in EUR e72-d71, totalling 64 bytes. Its candidate pages add40D; the prior
394 page already covers the first span. Typed field reconstruction and earlier
owner/raw composition retain the existing handlers, counters and source-use
tracking. No guest ABI, state schema or mutable machine fields change.
Two Wine filters pass managed lifecycle, six full-state/clock original versus
interpreter/native paths with raw sources altered, boundaries/mirrors/BIOS,
six SWI/DMA transfers and six host publication checks. See the
[OBJ integration evidence](rom_data_obj.md#managed-observed-indirect-lists-and-typed-consumption).
Actual production CLI create/no-write reuse/oneframe old-cache attachment now
passes, with both new source identities positive through guest and host. One
bounded firstF09frame Wine pair now passes full state/clock/captured buffers/OAM/
dispatch/all audio and requires both new sources. Source/model/managed/runtime/
CLI/declaredordinary children are closed; broader scenes and Linux/USA/native
Windows/new32:9 remain pending. Prior scene results stay closed.


## Observed task parameter runtime extension

EUR e73-d72 adds a bounded typed owner for the existing10byte parameter model,
with independent threshold/correction fields and per-kind/row coverage. Only
371065..67/394E14..1C are served; candidate pages371/394 compose earlier-owner
and raw bytes. Original counter/phase/correction instructions and guest clocks
remain live. Two first Wine filters pass13attach/threefull-set guards/lifecycle,
24complete-state/clock original versus interpreter/native prefixes with raw
altered, outside queries, edges/mirrors/BIOS, six SWI/DMA and mixed native API
reads. The corrected frontend build is warning-free; first failure was limited
to two test-call arguments. Actual CLI now passes create/no-write reuse67files/
69managed plus oldcache attachment. Firstframe is correction-negative; new12frame
CLI use reaches both kinds/all six rows. One exact Wine12frame pair now passes
every fullstate-clock/capturedbuffer/OAM/dispatch/allaudio comparison and requires
both kinds/all six rows, matchingCLIcounts. Source/model/managed/runtime/CLI/
declaredordinary children are closed; broader scenes and Linux/USA/native
Windows/new32:9 remain pending. See the
[parameter integration evidence](rom_data_gameplay.md#managed-observed-task-parameters-and-typed-bus-consumption).


## Lower bus and host residue source contracts

`SiteCapture` extends the optional `rom-data-read-audit` feature with a separate,
thread-bound collection filtered by aligned physical words. It preserves the
address-only `Capture` schema and keys. Each read adds an optional interpreter
execution PC and Thumb state; outside that scope it records `None`. Nested DMA
or HLE accesses can inherit the scope, so attribution to an instruction's own
operand requires checking the original opcode, effective address and width.
It records no payload bytes or timings and changes no Machine/native/save ABI.
The execution guard is compiled only with this nondefault feature.

The source-only ignored test is
`lower_rom_read_sites_tests::european_lower_rom_words_record_interpreter_sites`.
Besides the existing private ROM/cache/state/input/dataset paths it takes
`FFTA_STATE_TEST_LOWER_SITE_WORDS` (113 selected word offsets) and a new
`FFTA_STATE_TEST_LOWER_SITE_ROOT`. All reports and fixtures remain local.
One Windows feature build retry fixed only a test-local audio helper after an
actual compile failure; the successful build was warning-free. One Wine filter
passed cached/uncached original LDR scope, outside-scope/reset and old-key probes,
then observed one decoded F09 arm, frames8476–8488, using the current67file EUR
dataset. This is reader-site evidence; the already closed scene equivalence
and collector acceptance were not repeated.

All452selected bytes (113words,62ranges) were observed in114context reads.
Every read is a four-byte bus access matching its original Thumb opcode/address:
101literal words/404B from102executed PC-relative LDR sites, and12switch words/48B
at0A116C–0A11A0 from load080A1146 followed by `MOV PC,r0` at080A1148. The latter
preserves Thumb state despite even targets. The source prefix0A1124–0A114A
normalizes a low-u16 input of zero or greater than24 to1, subtracts1, guards
index<=23 and uses table0A1150. Only observed indices7–18 are classified;
this is neither a global raw-input cap nor a claim about all table uses.
Literal-shaped candidates/exported roots alone had been insufficient; their
rejected candidate maps remain distinct from this executed-reader proof.

The452bytes are code support, leaving e73-d72/67files66resources/USAe8-d7
unchanged. Together with the earlier high90-byte classification, the extra542
observed bytes comprise74typed data bytes and468code-support bytes. Pointed
payloads require separate contracts:394FBC from loads0809DB40/0809DBB4,
phase0 at371064, and signed-byte lead371034. No format or extent follows from
an address-like literal or a plausible header. Other scenes/threads/readers,
all-use completeness and full-ROM extraction remain open.

All85execution guards, compiled source hashes, old read-key probe and user Atlas
hashes/mtimes passed. Removing only the new cfg lines gives exact exec/frontend
preimages; production runtime/schema/revisions remain unchanged. Artifact
completion found no disposable paths. Linux/USA/native Windows/new32:9 remain
pending; no checkpoint/navigation or performance/storage measurements.
Evidence: `target/rom-data-bus-host-lower-residue/` including
`windows-sites-result.json`, `source-contract.json`, `preservation-result.json`
and `artifact-cleanup.json`. The frozen diagnostic executable is
c831b960bc0bb28b6a958f1d1b80b0379af663cbde055cb59cd20f550d317bd4.


## Constructor OBJ animation runtime

EUR e74-d73 adds an exact32B threshold/pointer owner, composed with the existing
OBJ owner extended by four lists/56B. Authentication, native/bus/DMA/host
acceptance and the retained test-setup failure are described in
[the OBJ feature contract](rom_data_obj.md#managed-constructor-obj-animation-and-typed-consumption).
Actual CLI prepare/reuse/zero-frame registration nowpasses all68files/67runtime
resource hashes; ordinary positive use and deferred platforms remain open.


## Conditional phase-zero parameter ownership

EUR e75-d74 extends the prior parameter owner by byte371064, covering phases0–2
without capping guest inputs. Three phase coverage bits precede four correction
bits; source spans3B/8B total11B, same68files67resources/all66other records-files.
[The phase-zero contract](rom_data_gameplay.md#conditional-phase-zero-threshold-extension)
records model/native/actualCLI/new28frame ordinary acceptance and raw fallback.
Constructor ordinary discovery and deferred platforms remain separate.


## Conditional task-cycle delta ownership

EUR e76-d75 appends a48-byte signed cycle owner on page371, composed with the
previous phase-threshold owner/raw ROM. One Wine filter passes eight original/
typed-interpreter/native fullstate-clock prefixes, all48rows, a mixed C-API
read with two bytes from each owner, three crossing DMA/SWI transfers and owner
retention through unavailable files/Load/Undo. All67previous resources remain
byte-identical. The [cycle contract](rom_data_gameplay.md#managed-task-cycle-deltas-and-typed-consumption)
records the exact signed/index/store/remainder semantics and authentication.
The106compiled instructions/37dispatches are coverage only. Actual production
CLI prepare/reuse/zero-frame attachment nowpasses all69files/68runtime hashes
and no-write reuse. Ordinary cycle48nowpasses one source-backed neutral Bank
fullpair3660–3708, all48rows/48reads48B/every state-clock-buffers-OAM-dispatch-
allaudio equal; counts match actualCLI. Deferred platforms and broader scenes
remain separate.


## Bank bus and selected host ROM inventory

The existing optional diagnostic now has an explicit `SiteCapture::start_all()`
mode, collecting execution scope before target addresses are known. The old
word-filtered constructor still treats an empty set as empty; address-only
`Capture` keys/schema stay unchanged. `rom-data-read-audit` remains absent from
default builds, with no new Machine/MemMap fields, C API, save/native-cache ABI,
extractor revision or production build. Removing the two new frontend cfg lines
restores its preimage exactly; all core/default-feature manifests are unchanged.

One first Windows feature build is warning-free, frozen
ff74819f4a8645fb1869b034f8c1714d386f9154f46447bd3b9e2921c34da30c.
One first Wine filter passes targeted empty/single/all filter, original ARM
instruction-fetch exclusion, projection and drop probes, then a single current
EUR decoded Bank12source arm over guest3660–3672 with the existing normal input,
current e76-d75/69-file dataset and original native cache. The all-site projection
exactly equals the independent address-only capture in that same arm. Dispatch
[19962,85770] and five actual sample counts[26334,26334,3168,3168,26258] match the
retained old Bank12report. These are coverage checks, not timings or a second
fullpair; the completed Bank48ordinary/F09/source/model/CLI proofs were not rerun.

The new report has7430unique read keys and7532site keys:497with interpreter/HLE
scope and7035outside it. Reader keys are3810Bus/3601AudioValue/10SpritePeek/
8SpriteList/1ObjSourceTable; KeyboardWidths is unexercised. It covers7083physical
image bytes and1434outside current68manifest owners. All3683image bytes from
the old native-only Bank sample are included. Of these unowned bytes,992already
have scoped source classifications from earlier Bank/F09contracts. New exact
opcode/address/width checks identify80Thumb PC-relative literal words/320bytes.
Four new dispatch cells plus four new cells in the already proved A1146table
add32bytes of executable control support, bringing this sample's total to1344.
MOV PC,r0 preserves Thumb despite even targets. The96110case has only a direct
<=126conditional path: larger values call09A230with unproved return behavior,
so no global127input cap is inferred. The A1146prefix contract is reused; its
old observation and classification are not repeated.

The remaining90observed data bytes are source leads:18bytes of relative
sequence-index/header/frame fields around4114FC,12bytes of object-list directory
entries4165D0–DC and60bytes in three counted OBJ lists416688–C4. Captured original
readers are2C5FA/2C5FE/2C61A/2C620/2C632 and1500-family sprite reads; SpritePeek
also reads the60list bytes. Executed source literals5E6F4/5E6F8 point to object
root4165A0and sequence root4114FC. The2C5E0/2C60C task uses a u16 selector/tick/
phase, distinct from the established4E494/4E4E4u8-counter animation producers.
Four declared producer prefixes now have a separate authenticated
[source/model contract](rom_data_obj.md#european-obj-banks-with-u16-task-counters).
Addresses and observed spans alone did not establish those bounds; managed and
runtime integration remain separate work.
No new resource or archive extent is declared by this inventory.

The same capture pins twelve already typed cycle bytes/phases21–32 to the
original Thumb LDRB at18E2A. This is actual interpreter data access and explains
the old native-only API blind spot without attributing a task address. Inherited
HLE/DMA scope still requires opcode/operand validation; native/host access
outside interpreter scope remains None. Other scenes/threads, unselected direct
host readers, typed HLE bypasses, folded native constants and code/signature
reads remain outside this bounded census.

All95actual input/output/frozen/source-contract guards and compiled implementations
pass, including the user-owned Atlas files. Required artifact completion finds
zero eligible paths. Private evidence is under `target/rom-data-bank-bus-host-residue/`:
`windows-bank/reads-and-sites.json`, `source-classification.json`,
`control-contracts.json`, `managed-cycle-sites.json`, build/Wine results and
preservation/cleanup receipts. Current datasets, original states/inputs/cache,
canonical executable and Atlas are unchanged. Linux/USA/native Windows/new32:9
remain pending; no new fixture/navigation/production build or performance/storage
campaign. The90byte lead now belongs to the four-prefix source/model contract;
managed/runtime children remain open. This census stays closed.


## Typed u16-task OBJ bank runtime

EUR e77-d76 adds the65span1378byte owner for four source-authenticated u16task
OBJ banks, preserving all68previous resource records/files. Guest reads compose
new typed fields with prior owners and raw padding; host list lookup shares the
same immutable field records. The original constructor/updater still owns all
live indexing and counter semantics. See the [managed runtime contract and
staged Wine evidence](rom_data_obj.md#managed-runtime-for-the-u16-task-obj-banks).
The current stages prove managed publication, native/interpreter full-state/
clock equality, bus/transfer and Load/Undo retention. A remaining-only Wine
filter passes50host lists/140OBJ/95typed host reads and accepted OAM publications
with poisoned raw source bytes, completing the runtime unit across staged
checks. The first interrupted filter is not a fullPASS; its closed stages were
not replayed. The [actual e77 CLI](rom_data_obj.md#actual-cli-for-the-u16-task-obj-banks)
nowpasses create/reuse/zero-frame attachment:70identical generationfiles/
69runtimeIDs, no managed writes on reuse/startup, all newusagefields zero.
The [ordinary Bank12 child](rom_data_obj.md#ordinary-european-u16-task-obj-bank-scene)
also nowpasses: one existing-CLI positive observation and one Wine fullpair
compare all12complete states/clock/images/OAM/dispatch/all5audio streams, with
292reads/346source bytes/all6field classes/20host reads exactly matching the CLI.
Only a5line optional test guard is added; the old validator is byteexact when
removed. This is one ordinary window, not allbank/list/fullgame coverage. Existing census/scene/model
acceptance remains closed, USA e8-d7 unchanged, Linux/nativeWindows/new32:9
pending and no performance measurements.


EUR e79-d78 adds `gameplay.world-roads` / `world-roads.json`: 45 selected road
headers and2,188signed XY samples, 46 source spans/4,736bytes. Runtime reads
come from immutable typed count/pointer/sample/opaque fields, composing with the
old world-node boundary and raw bytes outside the projection. Seven usage
fields report bus/source/count/pointer/sample/opaque bytes and paths used.
No host movement/facing/pathfinder replaces guest routines. First Wine filter
passes72files/71resources/all70prior exact and strict attach, then finds a missing
396page gate. After the routing fix, a remaining-only filter passes
12native fullstate-clock prefixes, all source classes/widths/mirrors/lifecycle
and DMA/CpuSet/CpuFastSet3. Actual CLI/ordinary case and Linux/USA/native Windows
remain pending; no measurements. See [managed world roads](rom_data_maps.md#managed-world-roads-and-typed-guest-reads).


The actual e79-d78production CLI and one ordinary sampled-road pair now pass on
Wine: create/reuse/startup72files71resources74managed paths unchanged, then
Giza360full states-clock/images/OAM/dispatch/fiveaudio with all seven counts
identical to CLI. Positive count/pointer/sample use covers5,067source bytes;
40paths means any field, not40sample payloads. Opaqueordinary0remains targeted-
only. Linux/USA/nativeWindows and broader catalog/scenes pending. See
[ordinary road equivalence](rom_data_maps.md#ordinary-sampled-road-travel-equivalence).


EUR e80-d79 adds `gameplay.encounter-headers` / `encounter-headers.json`, with
an explicit constructor-alternate dependency. Two source fragments provide
17,676new header bytes; the existing owner retains the four shared pointer
bytes. Global dispatch covers70C..710. Eleven usage fields distinguish eight
byte classes and442header IDs without altering guest indexing or algorithms.
One reduced Wine integration filter passes73files/72resources/all71prior exact,
14attach/three full-set guards,26native fullstate-clock prefixes, all owned and
shared bytes/widths/mirrors/LoadUndo and DMA/CpuSet/CpuFastSet. Actual CLI/ordinary
and Linux/USA/nativeWindows remain next. See [managed encounter headers](rom_data_maps.md#managed-encounter-headers-and-typed-guest-reads).


## Encounter-header ordinary scene guard

`FFTA_STATE_TEST_REQUIRE_ENCOUNTER_HEADERS=1` adds positive-use assertions for
unit counts, map arguments, unit-template pointers, placement counts, position
pointers and control bytes in the existing per-frame equivalence validator. It
reports all eleven header usage fields, including placement parameters and
opaque bytes. The accepted Wine700 Giza battle-entry pair matches all11 fields
to production CLI usage and every complete state/guest clock/image/OAM/audio;
placement parameters and opaque bytes remain zero in this ordinary case. The
five-line optional block preserves older guards and the validator preimage
exactly. [Proof and limits](rom_data_maps.md#ordinary-encounter-header-battle-entry-equivalence).


## Encounter unit-template typed owner

EUR e81-d80 registers `gameplay.encounter-unit-templates` / `encounter-unit-templates.json`
with header/constructor/item/metadata14/metadata52 dependencies. The immutable
136,608-byte two-fragment owner skips192 prior constructor bytes and composes
reads per byte, with global/local page routing6EA..70B. Nine byte-class counters
and record/header usage retain shared-only ownership. One reduced Wine filter
passes74-file publication/72prior exact/14attach/sevenfullset guards,27 poisoned
native fullstate-clock prefixes, every new/prior source byte, Load/Undo and
DMA/CpuSet/FastSet. Actual CLI and ordinary use are next. No guest algorithm or
save/native ABI changes. [Proof and limits](rom_data_gameplay.md#managed-encounter-unit-templates-and-typed-guest-reads).


## Unit-template ordinary scene guard

`FFTA_STATE_TEST_REQUIRE_ENCOUNTER_UNIT_TEMPLATES=1` requires positive ordinary
use of the eight observed semantic template byte classes and reports all13
usage fields. The accepted Wine700 Giza battle-entry pair matches every complete
state/guest clock/image/OAM/audio and all counts to the production e81-d80 CLI.
Five records/one header are used; opaque bytes stay zero. Removing the optional
five-line guard restores the older validator exactly. [Proof and limits](rom_data_gameplay.md#ordinary-encounter-unit-template-battle-entry-equivalence).


## Encounter coordinate typed owner

EUR e82-d81 adds `gameplay.encounter-coordinates` / `encounter-coordinates.json`
with an explicit encounter-header dependency. The immutable owner validates691
spans/4,684 bytes and header pointer/count/role coherence before activation.
Local/global pages70B..70C compose prior template/header borders bytewise;
statistics distinguish four roles, X/Y,691 lists and323 headers. Staged
Wine checks pass75-file publication/73prior exact/14attach/threefullset guards,
22 complete-state-clock native cases, all source bytes and list boundaries,
Load/Undo and three transfers. Original placement/RNG/ABI remain unchanged.
Actual CLI/ordinary and Linux/USA are next.
[Proof and limits](rom_data_maps.md#managed-encounter-coordinates-and-typed-guest-reads).


## Encounter coordinate ordinary scene guard

`FFTA_STATE_TEST_REQUIRE_ENCOUNTER_COORDINATES=1` requires positive ordinary
coordinate source reads, placement0, X/Y and list/header use, and reports all ten
usage fields. The accepted Wine700 Giza battle-entry pair matches every complete
state, guest clock, classic/central/wide image, OAM and all audio streams, with
all counts equal to production e82-d81 CLI. One list/header is used; random,
placement2 and default roles stay unobserved in this window. Removing the
optional five-line block restores the prior validator exactly.
[Proof and limits](rom_data_maps.md#ordinary-encounter-coordinate-battle-entry-equivalence).


## Indexed metadata28 ordinary scene guard

`FFTA_STATE_TEST_REQUIRE_METADATA28=1` requires positive ordinary reads/source
bytes/distinct records and at least one interpreted halfword, byte, inline or
flag class. It reports all eight counters. The accepted42-frame Wine Steal menu
case matches the actual production CLI exactly:15reads/20served bytes,10halfword/
10byte bytes/five records, other three classes zero. Every complete state/clock,
classic/central/wide image, OAM and all five audio streams match the original-ROM
arm. Removing only the optional five-line block restores the previous validator.

The ignored `metadata28_scene_tests::normal_gameplay_observes_metadata28` observer
accepts private `FFTA_METADATA28_SCENE_ROOT` plus the existing ROM/native/state/
input/dataset/frame variables. It runs normal keyboard frames on a copied seed,
checks the original ROM identity, and retains normal endpoints and a pre-read
suffix for later CLI/equivalence verification. It makes no guest state edits.
A successful capture alone is not equivalence; positive counters and the separate
full pair remain required. [Case, provenance and limits](rom_data_gameplay.md#ordinary-metadata28-steal-menu-equivalence).


## Linked four-byte record ownership

EUR e84-d83 adds `gameplay.linked-records4` / `linked-records4.json` to the managed
generation:77 files/76 resources, all75 prior records and payloads unchanged.
An immutable typed owner serves7133F8..713744 through the guest bus, with
per-byte composition against prior metadata28 and raw successor bytes. One
targeted Wine filter verifies native complete states/guest clocks, all844 source
bytes, all211 record ends, Load/Undo and three transfers. Actual CLI registration
is verified through create/reuse/startup; the initial120-frame target-screen
locator remains a closed zero-use observation. A later following-turn40-frame
case now observes all four fields and passes complete state/guest-clock/pixel/
OAM/five-audio equality with all seven counts matching capture and CLI.
Linux/USA remain pending.
[Binding, proof and limits](rom_data_gameplay.md#managed-four-byte-linked-records-and-typed-runtime-reads).

[Ordinary case and limits](rom_data_gameplay.md#ordinary-linked-four-byte-turn-equivalence).


## Conditional eight-byte gameplay records

EUR e85-d84 adds `gameplay.conditional-records8` at index76, one1016-byte span
in `conditional-records8.json`; all preceding76 resources and payloads remain
exact. An immutable owner serves six logical field classes and127 physical
slots, composing page713 with records4 and raw successor bytes. Live predicates
and selection remain original guest code. Combined Wine proofs cover17 native
state/clock cases, all source bytes/record ends, failed attachment, Load/Undo
and three transfers. Native-path accounting includes the real `interp_one`
fallback:607 direct plus two fallback bytes in the selection case equal609
actual bytes in both typed and native arms. The initial observer failure stays
recorded; passing earlier scopes are preserved rather than repeated. Actual CLI
creation/reuse/startup now passes with78 files/77 logged ID hashes/80 managed
paths unchanged during reuse and one neutral frame. A later normal42-frame
Rumors-entry case observes all six field classes and matches full state/clock,
classic/wide pixels, OAM and five audio streams, with all nine counters identical
between capture, CLI and pair. Linux and USA remain pending.
[Binding, evidence and limits](rom_data_gameplay.md#managed-conditional-records-and-typed-runtime-reads).

[Actual CLI registration and limits](rom_data_gameplay.md#actual-conditional-record-cli-registration).

[Ordinary case and limits](rom_data_gameplay.md#ordinary-conditional-record-rumors-entry-equivalence).


## Mission-record ownership

EUR e86-d85 adds `gameplay.mission-records70` at index77, one35,840-byte span
in `mission-records70.json`:79 files/78 resources, all77 prior payloads and
records exact. The immutable owner covers512 physical records, ten storage
classes and13 counters across pages713..71C. Guest getter/event logic remains
original. Combined Wine scopes verify publication,14 attach/two full-set cases,
41 native full-state/clock cases, all source bytes/512 boundaries, Load/Undo and
three transfers. The old shared/OBJ and BG allocation caps are removed under the
requested deferral of space limits; source/format/dependency authentication and
transactional binding remain active.

Actual production CLI creation/reuse/startup now passes with573 frozen sources,
79 runtime-identical files,78 logged ID/hash pairs and81 managed paths unchanged
during reuse/startup. One neutral frame24755..24756 has all13 new counters zero;
a later33-frame Giza Plains mission-information case now passes complete
state/clock/classic-wide pixels/OAM/five-audio equality with all13 counts exact
across capture, CLI and pair. It observes five served packed bytes/one record,
other nine classes zero, not all fields/512 records or mission outcome. No closed
source/model/native/old ordinary proof or performance campaign is repeated.
Linux and USA remain unverified.
[Binding, evidence and limits](rom_data_gameplay.md#managed-mission-records-and-typed-runtime-reads).

[Actual CLI registration](rom_data_gameplay.md#actual-mission-record-cli-registration).

[Ordinary mission-information case](rom_data_gameplay.md#ordinary-mission-information-equivalence).


## Sparse parameter-record ownership

EUR e89-d88 adds `gameplay.parameter-records27` at index80:82 files/81 resources,
all80 prior payloads exact, two spans totaling1593 bytes for59 observed physical
indices. The immutable owner composes with the previous event owner and raw
holes across pages71D..71E;12 storage classes/15 counters retain sparse physical
identity without adding a guest bound or changing save/native/C API ABI.

Combined reduced Wine checks pass publication/14 attach/two full-set guards,
33 complete-state/clock cases, all1593 bytes/59 borders, Load/Undo and three
transfers. An oracle count error after17 native cases is corrected only in the
test leaf; the remaining16 cases and lifecycle checks reuse the same dataset
and13 objects. Earlier successful checks are not repeated. Actual production
CLI and ordinary positive use remain pending, as do the full catalog extent,
Linux and USA. No performance campaign was run.
[Binding, evidence and limits](rom_data_gameplay.md#managed-sparse-parameter-records-and-typed-runtime-reads).


Actual e89 production CLI creation/reuse/startup now passes with588 matching
source files,82 runtime-identical payloads,81 logged ID/hash pairs and84 managed
paths unchanged during reuse/startup. One neutral frame24755→24756 has all15
parameter counters zero. Positive ordinary use and full scene equivalence remain
pending. [CLI evidence](rom_data_gameplay.md#actual-sparse-parameter-record-cli-registration).


## World-record ownership

EUR e90-d89 adds`gameplay.world-records38` at index81:83 files/82 resources,
all81 prior records/payloads exact. Its immutable31-record/1178-byte owner
composes page71E with the prior sparse parameter owner and raw gaps/next root.
Five physical classes/eight counters preserve original uncapped addresses and
guest logic. One reduced Wine filter passes publication/14 attach/two full-set
guards,27 native complete-state/clock cases, all source bytes/31 boundaries,
Load/slot Undo and three DMA/BIOS transfers. Actual callbacks serve23 new
and5 prior bytes, fallback[0, 0]; these are served counts, not distinct
coverage or an all-direct-native claim. Actual CLI/ordinary use and Linux/USA
remain pending; no performance measurements were made.
[Binding and limits](rom_data_gameplay.md#managed-world-records-and-typed-runtime-reads).


Actual e90 production CLI creation/reuse/startup now passes with592 matching
source files,83 runtime-identical files,82 logged ID/hash pairs and85 managed
paths unchanged during reuse/startup. One neutral frame24755→24756 has all
eight world-record counters zero. Positive ordinary use and full scene
equivalence remain pending.
[CLI evidence](rom_data_gameplay.md#actual-world-record-cli-registration).


One later45-frame ordinary world-travel suffix now passes complete state/clock,
classic/wide pixels, OAM and five-audio equality using the same old native cache.
Actual CLI and pair match all eight counters: two reads/two served bytes,
one threshold/one byte01/one physical record; other three classes remain zero.
The source-backed trigger follows normal daily counter expiration in an
authenticated IN08 copy, after an earlier IN13 travel bound was closed negative.
Other ordinary records/classes and Linux/USA remain pending.
[Ordinary case and limits](rom_data_gameplay.md#ordinary-world-record-counter-expiration-equivalence).


## Indexed graphics source ownership

EUR e91-d90 adds [indexed graphics archives](rom_data_indexed_graphics.md#managed-source-views)
at index82:84files/83resources,24spans/74907source bytes, all82 prior resource
records/payloads exact. Seven physical classes/twelve counters retain typed
source bytes,20archive/649program/fourgroup masks, prior/raw border composition
and original guest decoding/uploads. Corrected597source Windows build is
warning-free after a test-only missingBus import; first diagnostic retained.
One reduced Wine filter passes publication/interruption/14attach/twofullset
guards,35complete-state/clock native cases,allsourcebytes/classes/24edges,
Load/slotUndo and three64B mixed DMA/BIOS transfers. Callbacks serve272new/four
priorworldbytes, fallback[1,0], not distinct union or all-direct-native coverage.
282guards/597accepted597diagnostic/595model/sevencurrent83prior preserved; no
closedproof/ordinaryfixture/navigation/performance replay. ActualCLI/ordinary,
noncanonical modes/widths and Linux/USA/nativeWindows/new32:9 remain pending.

Actual e91 productionCLI creation/reuse/startup now passes from the same597
accepted runtime sources:84identical payloads/83logged IDhashes/86managed paths
nowrite/cache28859 unchanged/canonicalhashmtime restored. One neutral normal
frame24755→24756 has all twelve new counters zero, registration only.
201guards597CLI+597runtime/84accepted originals exact; no ordinary fixture/
navigation/oldproof/metrics replay. Positiveordinary/fullpair and deferred
platforms remain pending. [CLI evidence](rom_data_indexed_graphics.md#actual-cli-registration).


One later41-frame [ordinary world-map MENU case](rom_data_indexed_graphics.md#ordinary-world-map-menu-equivalence)
now passes complete state/clock/classic-wide/OAM/five-audio equivalence with
the same old native cache. Actual CLI and pair match all12 counters:274reads/
277served bytes across allseven classes/onearchive-program-group. OneStart12
normal capture from copied4519worldmap stopsafterfirstsource+31 at4560; no
old320Clanroute orworld38daily45 acceptance replay.144guards598frozen597CLI/
threecurrent86managed/originals exact. Other ordinary archives/programs/languages
and Linux/USA/native Windows remain pending; no performance measurements.


## Table-selected custom UI extension

EUR e92-d91 extends the existing [custom UI resource](rom_data_ui_graphics.md#managed-table-selected-graphics-and-runtime-consumption)
at index15 to56banks/10selectors/106bindings. Same84files/83IDs, all82other
records-files andold16banks/twoinline/sharedOBJ exact;80additionalbindings
cover the same53667B/48canonical spans. Sixnewfamily counters preserve the
oldUIstats meanings and originalguest decode/upload work. One601-source
warning-free Windows build/oneWinefilter passespublication-interruption/14attach
twofullset/22complete-state-clock cases andallsource/48edges/LoadUndo/DMA3.
499servednewB5priorindexed/fallback[3,0] are callbackcounts, notdistinct union
orall-direct-native coverage.267guards601frozen600model/sevencurrent84prior
exact; nooldproof/ordinaryfixture/navigation/performance replay. ActualCLI/
ordinary use and deferred platforms remain pending.

Actuale92 productionCLI registration nowpassesone601sourcebuild/oneWine
creation-reuse-neutral1/threeexit0:84runtimefiles/83IDhash/86managednowrite/
cache28859exact/canonicalfinallyrestore. Guest24755→24756 has allsixnewtable
counterszero, compatibilityonly;202guards601CLI+601runtime/84accepted inputs
exact. Positiveordinary/fullpair and deferredplatforms remainpending.
[Actual CLI evidence](rom_data_ui_graphics.md#actual-table-extension-cli-registration).


EUR e93-d92 adds independent `graphics.ui-raw-transfers` at index83,85 files/
84 IDs, retaining all83 prior records/payloads exactly. Its35 fixed typed-word
views and seven raw language pointer tables serve42 spans/38220 bytes through
the shared immutable bus with six separate counters. The seventh bank owns
only+0200..0380; its successor archive remains under the prior indexed owner.
Original copy helpers, queues, DMA and guest clocks remain active. One focused
Wine filter passes publication/interruption,14attach2fullset guards,23 native
complete-state/clock cases, all bytes/borders/widths/mirrors, Load/slot Undo and
four64-byte mixed DMA16/DMA32/CpuSet/FastSet transfers. Native callback coverage
includes939 new served bytes/three prior with fallback768/0, not all direct
native execution. Two605-source Windows builds include a pre-Wine test-only
prior-owner boundary correction; accepted4b219adc is warning-free. Actual CLI/
ordinary use, dynamic index domain/full caller ABI and new
Linux/USA/native Windows/32:9 checks remain pending. See
[raw transfer contract and evidence](rom_data_ui_raw_transfers.md#managed-raw-transfer-resource).

The real e93 CLI now passes create/reuse/no-write/startup under Wine: one
warning-free605-source productionffda6c59 build, canonical hash/mtime restored,
three child exits0,85 runtime-identical files/84 IDhashes/87 managed paths
unchanged and28859 old native blocks loaded. A retained normal24755 checkpoint
copy advances one neutral frame; all six new raw counters0, registration only.
203 preservation guards pass; no positive ordinary/full-scene claim.
[Actual CLI evidence](rom_data_ui_raw_transfers.md#actual-cli-registration).

## Historical F09 residue against e95

The retained 12-frame F09 bus and selected-host audit was collected under
e71-d70. Reclassifying its 1,146 distinct then-unowned bytes against the
accepted e95-d94 source intervals found 74 bytes covered by managed owners
and 1,072 still outside them at e95. E249 later reconciles all 1,146 against
E228. These are comparisons of historical addresses, not new execution or an
assertion that all those reads occur today.

All 1,072 remaining bytes match earlier scoped code-support evidence: literal
or switch-table readers, hardware control words, and IWRAM branch pointers.
The associated original-source hashes or values were checked again against the
exact EUR ROM. The 452-byte executed-site evidence and static reader witnesses
keep their original attribution limits; neither proves every later use. The
older e93 code-candidate bitmap alone covers only 52 of these bytes and does
not include every literal pool, so it must not be used as a complete classifier.
No new asset, format or overall ROM completeness is inferred. Evidence is in
`target/rom-data-e95-historical-residue/`; other scenes and readers remain
unclassified. Linux and USA checks remain pending, and no performance
measurement or old audit replay was performed.


## Conditional lookup managed projection

The EUR e96-d95 owner projects 296 authenticated bytes at `6e6dbc` from 74
two-field records. Guest arithmetic and out-of-projection reads are unchanged;
strict activation rejects coherent edits, and the resource reports per-field
read counters. The focused Wine checks cover source bytes, boundaries and three
translated original reader prefixes, not ordinary gameplay, Linux or USA.
[Contract and evidence](rom_data_gameplay.md#managed-conditional-four-byte-lookup).


## Conditional packed lookup managed projection

EUR e97-d96 owns 132 bytes at `6e6ee8..6e6f6c`, projected from typed bit
fields. Other selectors remain unrestricted; the existing metadata14 owner
continues to serve overlapping lookup addresses beyond the projection. The
focused Wine test covers publication, original native prefixes, mixed owner
boundaries and coherent edit rejection, not ordinary gameplay or Linux/USA.
[Contract and evidence](rom_data_gameplay.md#managed-conditional-packed-lookup).
That packed owner also covers the physical metadata14 descriptor-1 alias at
`6e6f5e..6e6f6c`. A controlled original encounter-importer prefix reads its
first byte from the packed owner in interpreter and translated Wine runs with
poisoned sources and matching states/clocks (`56wpztn8`). One ordinary EUR
battle-entry interpreter frame also reaches the original reader and matches
raw state, clock, image, OAM and mixed audio (`gpir8qjh`). E170 Wine
`3s1xbvpx` compares 240 recorded battle-entry frames with the row altered,
including state, clock, video, OAM and nonempty audio; five typed row requests
occur. Generated dispatch can fall back to interpretation, and full importer
and other scene/consumer coverage remain open.

E170 Wine `_hfk7lti` observes a full ordinary `080ccf9c` converter call
from entry to return: the original row reader receives one typed byte while
the source row is altered, and raw/managed interpreter state, clock, video,
OAM and mixed audio match at return. This covers one call, not the `080ce18c`
batch wrapper, all consumers or per-read native attribution. [Details](rom_data_gameplay.md#special-encounter-descriptor-aliases-the-packed-lookup).

Controlled E170 Wine batch test `g2cugem4` executes `080ce18c` through return
for an authenticated two-unit header and verifies limits 0–3, a skipped
occupied destination, typed alias reads and raw/managed state, clock, video,
OAM and mixed-audio equality. Occupancy is synthetic and input is fixed;
ordinary batch reachability and other headers or consumers are not established.

On the recorded 240-frame route, E170 Wine `049hqes5` observes all five typed
row requests inside generated dispatches, without interpreter instruction
scope. This does not identify the guest PC of an unscoped access or exclude
every nested non-interpreter helper; exact per-instruction attribution and
other routes remain open.

Wine `1gs1xgeu` then wraps the test runtime API and confirms that all five
row requests directly invoke generated-code memory API callbacks, serving five
packed-owner bytes with none nested in `interp_one`. The 240-frame comparison
passes with and without the wrappers (`0zpte7lm`). The exact guest instruction
PC of each callback and other routes remain unproved.


## Conditional descriptor banks managed projection

EUR e98-d97 serves 23 pointer words and 684 eight-byte records through typed
fields, preserving the raw bank-0 pointer and bank-22 gap. The focused Wine
check verifies publication, mixed edges, original native prefixes and coherent
edit rejection, not ordinary gameplay or Linux/USA.
[Contract and evidence](rom_data_gameplay.md#managed-conditional-descriptor-banks).


## Conditional descriptor bank 0 managed projection

EUR e99-d98 serves pointer 0 and records 1–15 from authenticated typed fields;
record 0 stays raw and adjacent bank-1 reads compose with the existing owner.
The focused Wine test covers publication, mixed-owner edges, original native
prefixes and coherent edit rejection, not ordinary gameplay or Linux/USA.
[Contract and evidence](rom_data_gameplay.md#managed-conditional-descriptor-bank-0).

## E139 ordinary high-ROM read inventory

Three no-input interpreter frames from the authenticated EUR context-event
checkpoint were audited against the E139 manifest over physical offsets
`300000..1000000`. The capture observed 505 high-ROM requests, dropped none,
and excluded instruction fetches. Thirty-four distinct bytes were outside
declared source spans; no observed raw request overlapped a declared span.
The 12 bytes at `370e68..6c`, `370e74..78` and `370eb0..b4` are already
classified above as hardware/branch support. The other 22 bytes occur at
`3ae42c..2e`, `3ae440..46`, `3ae448..4c`, `3ae44e..52` and `3ae454..5a`;
their observed scopes include context code and the original OBJ reader. They
are source leads, not established asset boundaries or complete domains.

Evidence: `e139-ordinary-residue-wine-zmi0glsw`. Interpreter scopes may include
nested HLE/DMA work; declared source ownership does not itself prove typed
service. This covers one short scene, not all reads or game paths. Linux, USA
and native Windows remain open; no performance measurement was made.
The two language-position source plans and the adjacent selected OBJ lists
were subsequently added in E141 and E140 respectively; see their scoped
[contracts](rom_data_obj.md#conditional-language-position-pairs-e141).

The same bounded inventory was later run with E141 on three no-input frames
from the separately authenticated EUR frame-814 coordinate checkpoint
(`e141-early-residue-wine-5b7xk8ul`). It recorded 309 high-ROM requests with
no drops or raw events overlapping declared source spans. The only four
unowned bytes are `370eb0..b4`, the previously classified copied-IWRAM helper
vector. This is a negative result only for this short window; it does not
establish complete ROM coverage or exercise other regions or platforms.

A further three-frame E141 audit starts from the authenticated later
`08140e1e` coordinate checkpoint (`e141-late-residue-wine-px0r_2m1`). It
records 89 high-ROM requests without drops or raw events overlapping declared
sources. The 164 distinct unowned bytes consist of the previously classified
`370eb0..b4` helper vector and 160 bytes at `3ae38c..3ae42c`, observed as
word reads in the `08140e7e` interpreter scope. This is a source lead, not yet
a proven data format, full extent, destination or global consumer domain.
The indexed tick and neighboring gate source contracts were subsequently
modeled in E142; [their scope](rom_data_obj.md#context-activation-tick-and-gate-source-contract)
does not establish other consumers or a global domain.

An E142 audit waits without input for the first activation and then captures
three interpreter frames (`e142-first-event-residue-wine-ih9nxtd5`). Of 2,281
high-ROM requests, none were dropped or read raw across a declared source.
The 12 unowned bytes comprise the known `370eb0..b4` helper vector and eight
bytes at `3ae16c..3ae174`, observed as halfword reads in the `08145230`
interpreter scope. The latter are a source lead, not yet an authenticated
record format, full extent, destination or consumer domain.
The eight-byte target rectangle and its original copy/interpolation path were
subsequently modeled and checked in E143; see the
[scoped contract](rom_data_obj.md#context-target-rectangle-source-contract).
E144 also serves six additional geometry sources with distinct runtime
accounting; their controlled Wine validation and remaining ordinary-use limits
are described in the [source contract](rom_data_obj.md#additional-context-geometry-sources). A separate
three-frame Wine inventory at context tick 1320 found no unowned high-ROM
data beyond the previously classified helper vector and no raw events
overlapping declared E144 spans (`e144-late-residue-wine-qpvutjx9`). This is a
negative only for that window, not global source coverage. A distinct
three-frame inventory of a retained EUR second-battle deployment records 102
unowned high-ROM bytes in eight observed ranges, with no dropped requests or
raw events overlapping declared E145 spans (`e145-battle-residue-wine-3pq4rwye`).
Some ranges are known executable control words; the others are source leads,
not established asset extents. The observed 102 bytes are subsequently
classified: 36 bytes are previously authenticated hardware/branch control,
14 bytes at `395088` and 50 at `40dcba` are served as OBJ lists in E146/E147,
and the two observed bytes at `37101d..1f` belong to the three-byte E148 battle
threshold fragment. This classification covers only that window; the draw-list
pointer producer and other scenes remain open. See the
[battle list contract](rom_data_obj.md#battle-constructor-list-at-395088). E145 extends the
same source owner with two earlier context geometries; the old six-record
representation remains accepted. See the
[scoped contract](rom_data_obj.md#earlier-context-geometry-source-contracts).


## E150 ordinary Llednar read inventory

Wine unit `e150-llednar-residue-wine-ek23m5tm` inventories three neutral
interpreter frames from the authenticated EUR Llednar battle state against
E150/d149. Of 3,477 high-ROM requests, none were dropped and none of the
observed raw events overlapped declared source spans. The 52 distinct bytes
outside the manifest comprise 20 bytes of previously classified DMA/branch
support at `370e70..78`, `370e80..88` and `370eb0..b4`; 16 bytes of previously
classified IWRAM branch support at `397f80..84` and `397f88..94`; and 16 bytes
of source leads at `394e1c..20`, `395014..1c` and `398df0..f4`. The latter
are observed read ranges, not established asset boundaries or formats.
Instruction attribution may include nested execution. This is a bounded
three-frame sample, not global coverage; Linux, USA and native Windows remain
unverified, and performance was not measured.

The EUR readers for those 16 bytes were examined in Wine unit
`e150-llednar-sources-dis-wine-o4t2h6dz`. Original `080264f6` reads two
signed corrections at indices 8 and 9 of the previously authenticated table
at `394e0c`; the existing managed fragment covers indices 4 through 7.
Original OBJ draw readers consume `395014..1c`; its signed count is one,
consistent with one six-byte entry after the two-byte count. Original `080a3076`
reads halfwords through base `398df0` with its input shifted left one; that
reader has no local bounds check. The observed two indices do not establish
the full table or callers. The OBJ list's producer and all three source domains
remain open; these observations alone add no managed resource.

Wine `e152-llednar-lookup-observer-wine-ro1bzrrb` observes `080a3076`
reading both `398df0` and `398df2` from callback returns `0809b1c1` and
`0809b1cf` during three neutral frames. The callback passes the same signed
actor+`7e` value to the unguarded lookup twice; these two observed entries do
not bound other inputs or establish the full table. See the
[Llednar OBJ source](rom_data_obj.md#ordinary-llednar-draw-list-at-395014) for
the related ordinary setter observation.


## E153 Llednar dialogue read inventory

Wine `e153-llednar-dialogue-residue-wine-g51e3yzn` inventories three neutral
interpreter frames from an authenticated retained EUR Llednar dialogue against
E153/d152. Of 2,990 high-ROM requests, none were dropped and none of the raw
events overlapped declared source spans. The 43 unowned bytes include 36 bytes
of previously classified hardware and branch support, plus observed ranges
`371008..0b` (three bytes) and `397fa4..a8` (four bytes). The latter remain
source leads, not established asset boundaries. The bounded sample does not
establish global coverage; Linux, USA and native Windows remain pending. No
performance measurement.

Wine `e153-dialogue-source-dis-wine-sro_6w4u` classifies the remaining
observations. At `0809b6fc`, word `397fa4` supplies Thumb IWRAM address
`03006bbd` to the established `08145c10` indirect branch helper: this is
conditional control support, not an exported asset, and the mutable IWRAM
body is not authenticated by that classification. At `08013152`, original
code increments task+`38` but reads the *old* signed byte as an index from
`370ffc`; after the draw it masks the stored byte with `1f`. Thus repeated
normal iterations yield a conditional 32-byte source `370ffc..37101c`, but
the first lookup, external writers and all consumers have no inferred cap.
The dialogue inventory observed indices 12..14 only. The conditional span is
disjoint from E153 declared sources and ends at the separately managed
`37101c` threshold fragment. Its SHA-256 is
`8352f94c00b26d3b0834df0b3f1f16de59302e3dc4edf92de8625902dbdc12cc`.


## E154 Llednar confirmation menu inventory

Wine `e154-llednar-menu-residue-wine-yq2cxie0` records three neutral
interpreter frames from the retained EUR Llednar confirmation menu against
E154/d153. Of 2,982 high-ROM requests, none were dropped and no raw event
overlapped declared source spans. All 36 distinct unowned bytes are previously
classified DMA and IWRAM-branch support; no new data-source lead was observed
in this window. Other frames and scenes, Linux, USA and native Windows remain
open. No performance measurement.


## E154 title language selector read inventory

Wine `e154-title-language-residue-wine-ry8vy8i9` inventories three neutral
interpreter frames at the authenticated EUR title-language selector against
E154/d153. Of 997 high-ROM requests, none were dropped and no raw event
overlapped declared source spans. The 62 unowned bytes consist of the known
four-byte IWRAM helper vector at `370eb0`, four observed bytes at `3adcfc`,
and 54 bytes in six observed ranges at `b8ed06..b8ee76`. These are reader
leads only, not inferred source extents; other frames, Linux, USA and native
Windows remain open. The first launch was refused before execution because its
protected plan path pointed into retired scratch; correcting it to the retained
evidence path allowed the run. No performance measurement.


## E155 main title menu read inventory

Wine `e155-title-main-residue-wine-2gcsrjfn` records three neutral interpreter
frames from the authenticated EUR main title menu against E155/d154. Of 611
high-ROM requests, none were dropped and no raw event overlapped declared
sources. The eight unowned bytes comprise the previously classified `370eb0`
helper vector and four observed bytes at `3adca4..a8`. No read from the
distinct `b8ed28/b8ecb4` bank occurred in this window. This is a bounded
negative, not proof of global absence or asset extent. Linux, USA and native
Windows remain pending; no performance measurement.


## E157 EUR title-opening OBJ sources

The managed static OBJ resource now serves the 19 absolute pointers and
11 counted lists from the conditional EUR title-opening contract. Existing
datasets remain readable with this optional directory absent; the EUR
extractor revision is 157 and decoder revision is 156. Wine controlled
original initializer and draw comparisons pass (`pfxplazm`); production CLI
create/reuse/verify and one-frame startup pass (`shuvdeal`). Ordinary drawing
and native-dispatch attribution remain pending. Linux, USA
and native Windows remain pending. No performance measurement.


## E157 neutral opening read inventory

Wine `e157-opening-inventory-wine-czpokl0v` inventories three neutral
interpreter frames from the retained EUR pre-draw checkpoint against E157.
No requests were dropped and no raw request overlapped declared source
spans. The eight unowned bytes consist of the known four-byte IWRAM helper
vector at `370eb0` and four observed bytes at `3adeb0..3adeb4`, read at
original PCs `0813a2ce` and `0813a2e2`. The latter are leads, not a proved
source extent. Other frames and scenes, Linux, USA and native Windows remain
open. No performance measurement.


## E158 EUR opening coordinate sources

The layered OBJ resource serves 19 conditional coordinate pairs and 19
separately sourced halfwords, preserving the intervening bytes and older
dataset parsing. Wine original-reader and three-frame ordinary interpreter
comparisons pass (`b8vylqkk`). Production CLI create/reuse/verify and
one-frame managed startup pass (`tucnagm9`). A later neutral EUR opening
continuation observes all 19 sparse halfwords and matches one original frame
in Wine interpreter mode (`htdztv4y`); other scenes and native dispatch remain
open. Linux, USA and native Windows remain pending. No performance measurement.


## E158 opening sparse-branch read inventory

Wine `e158-sparse-residue-wine-b81cszl2` inventories three neutral
interpreter frames from the retained EUR pre-sparse checkpoint against E158.
No requests were dropped and no raw request overlapped declared source spans.
The only four unowned bytes are the previously classified `370eb0..b4`
copied-IWRAM helper vector. This bounded window does not establish absence
of other sources elsewhere. Linux, USA and native Windows remain pending;
no performance measurement.


## E158 early-opening read inventory

Wine `e158-early-bank-inventory-wine-lnfgdwch` inventories three neutral
interpreter frames from the retained EUR early-opening checkpoint against
E158. No requests were dropped and no raw request overlapped declared source
spans. The only four unowned bytes are the previously classified `370eb0..b4`
copied-IWRAM helper vector. The same checkpoint's earlier E157 inventory had
also identified the four coordinate bytes now owned in E158. This bounded
window does not establish absence of other sources elsewhere. Linux, USA and
native Windows remain pending; no performance measurement.

## E158 title-language publication read inventory

Wine `e158-title-post-publication-residue-wine-13oct80b` starts at the
retained EUR title checkpoint immediately after the original French-language
publication and observes the next three interpreter frame boundaries against
E158. The initial boundary can finish a partial frame, and the checkpoint's
pressed A key is retained throughout: the generic report's old description of
the interval as no-input is inaccurate. The corrected observer records its
active-low key value and input policy; Wine
`e158-title-post-publication-input-report-wine-tc_c7quu` confirms `03fe`
and the same source inventory. Of 224 matched requests none were
dropped, and no raw request overlapped a declared source. The only four
unowned bytes are the previously classified `370eb0..b4` copied-IWRAM helper
vector. This bounded window does not establish absence of other sources or
validate other languages, scenes, native dispatch, Linux, USA or native Windows.
No performance measurement was made.

## European bounded range lookup

EUR schema 1/e161-d160 adds `engine.bounded-range-lookup`, an independent
11-record source at `37140c..371464`. Original routine `01abc0` scans the
eight-byte records in order, compares the low 16 bits of its input to the
inclusive halfword bounds at offsets 2 and 4, and on the first match writes
the selector byte and the input's offset from the lower bound. On no match it
leaves both output destinations unchanged. The byte at offset 1 and the last
two bytes are preserved as opaque; guest code continues to perform the scan
and writes. The original loop covers 11 records and an independently rooted
structure starts at the exclusive end. Source SHA-256 is
`4cae38f2a735087e740409ca58e5f19bce1614805347c472990b8419e493bb7d`.

Wine unit `e161-range-lookup-wine-t2mh2ei8` passed publication and reuse,
preserved all 103 prior e160 resources, and served all 88 source bytes with
the raw bytes altered. Boundary/mirror reads and source guards passed. The
original full scan matched the typed path in 46 state/clock cases covering the
record bounds and an unmatched value, through interpretation and generated
dispatch; generated dispatch may fall back to interpretation. The initial
build `1zxknk54` failed because three size-check calls lacked the new argument;
the first correction `_0ta2f6l` introduced a syntax error. Corrected build
`zlxh65v6` and the Wine test passed. A nonfatal Wine D-Bus diagnostic appeared
and the server drained.

Production Wine CLI unit `e161-range-lookup-cli-wine-emoffrjv` created, reused
and verified an e161 dataset whose 105 files matched the tested generation.
That generation stayed unchanged through reuse and an 11-frame world-map run;
the run read four indexed-graphics context bytes and zero range-lookup or
substitution callback bytes. This establishes registration and bounded startup,
not absence of those reads in other scenes.

A protected full-ROM Thumb BL candidate census `bounded-range-caller-census-
yeizv0zb` finds two encoded calls to `01abc0`, at `01ac18` and `01ac78`, in
the existing disassembly of wrappers `01abfc` and `01ac54`. A separate census
`bounded-range-wrapper-census-bpuinw0u` finds respectively 32 and nine encoded
Thumb calls to those wrapper entries, with no exact Thumb-pointer occurrence;
41 local windows are retained in `bounded-range-caller-windows-tq88gscc`.
Both original direct wrapper prefixes were then checked under E170/Wine
(`bounded-range-wrapper-wine-xfsk4jgx`): 23 values covering every recorded
lower and upper bound plus one unmatched input, in each wrapper and both
interpreter/generated dispatch, for 92 complete-state/clock comparisons with
all 88 source bytes altered. The unmatched output sentinels are preserved.
The comparison stops after the helper returns, before later rendering; it does
not prove execution of all upstream callers or guest input bounds.

Protected census `bounded-range-direct-literal-census-_s7xtdr9` scans all
aligned EUR ROM words for pointers into `37140c..371464`, including the three
normal mirror pairs. The only match is the base literal at `01abe8`; the only
candidate Thumb PC-relative load of that literal is `01abca`, in the identified
reader, and no ARM PC-relative load candidate was found. That aligned census
alone does not exclude unaligned pointers, computed addresses, RAM copies or
indirect reads.

Protected census `bounded-range-unaligned-pointer-census-l4aomd4j` also scans
every byte offset of the authenticated EUR ROM for exact little-endian
32-bit pointers into this 88-byte range through mirrors 08, 0a and 0c. It finds
only the same aligned literal at `01abe8`, and no unaligned occurrence. This
does not exclude computed or differently encoded addresses, RAM-built pointers,
other mappings or indirect reads; an occurrence alone would not prove use.

E170 Wine boundary regression `bounded-range-e170-boundary-wine-pbj0c8fy`
flips the range source and its four immediate bytes on each side, then checks
all 1-, 2- and 4-byte reads wholly in that window through mirrors 08, 0a and
0c. All 852 requests are typed with zero raw/unclassified/dropped requests;
the adjacent substitution and geometry owners each serve 63 bytes, and the
range owner serves 1,848 bytes. This proves the tested mixed-owner bus/API
composition, not ordinary execution or the complete set of consumers.

E170 Wine lifecycle test `bounded-range-e170-lifecycle-wine-p55pua3g`
checks a private dataset copy. Both a missing range file and a structurally
valid altered range file with its manifest hash updated are rejected on
reattachment, preserving the active owner and complete machine state. While
the replacement is invalid, slot Load and Undo restore their respective full
states and retain the owner. All 88 range bytes remain available with the raw
source altered, and a fresh owner serves the same bytes after restoring the
copy. Inputs were protected, test scratch was removed and Wine drained. This
uses the slot API and a synthetic machine; GUI operations, Fast, backup
restoration, restart and other resources are not covered by that test.

Separate E170 Wine test `bounded-range-e170-backup-wine-tg0_or33` makes its
dataset copy unavailable and calls the direct backup-restore and restart APIs
with synthetic cartridge saves. Each operation preserves the attached owner,
restores the expected complete boot state and cartridge bytes, and still serves
all 88 range bytes with the raw source altered. Inputs were protected, scratch
was removed and Wine drained. This does not cover GUI actions, Fast or the
lifecycle of every migrated resource.

The earlier full-byte ownership audit also identified a separate conditional
context: original registry readers `019172` and `019424` can address all 22
four-byte words of this range using byte indices 233..254. Wine
`bounded-range-indirect-registry-wine-44qwhqj_` compares both original prefixes
for every such index, neighboring indices 232 and 255, and their `0x100`
aliases in interpreter and generated instruction dispatch: 192 complete
state/clock cases. All 88 range-source bytes are altered in the managed copy;
the owner serves four bytes for each of 176 in-range cases and zero for the 16
neighbor cases. This stops before callback invocation. It establishes the
conditional read behavior, not that these byte values occur in ordinary play
or are safe to invoke; generated blocks may fall back to interpretation.

Patterns and disassembly do not prove code reachability or exclude computed
calls or alternate entries.

A wider numerical census (`bounded-range-nearby-roots-census-wdb_l7ok`)
finds 18 aligned ROM words pointing from 1,023 bytes before the range through
its end, including the direct reader and the known computed readers. Four
words have no candidate PC-relative immediate load in the scoped Thumb/ARM
screen. Protected audit `bounded-range-four-root-use-audit-eqni5tiq` finds
that their four bytes lie entirely within one E170 PCM wave source and three
BG graphics source spans, respectively, and finds no exact little-endian ROM
pointer to any byte of those words on mirrors 08/0a/0c. The source ownership
and lack of exact references do not exclude alternate or computed uses. Wine
windows `7zra4rc1` cover the
other 11 Thumb load candidates beyond the five previously identified sites.
One candidate follows a return in an apparent literal pool, but an alternate
entry is not ruled out. For the other local windows, byte-indexed loads from
`371020`, `37101c`, `371064` and `371034` cannot reach `37140c` using their
shown unsigned or signed byte indices. For the `371024` geometry loads, the
normal entry at `017708` requires the relevant dimension to equal eight;
the shown loops constrain both row and column calculations to the 16 bytes
`371024..371034` (`bounded-range-geometry-windows-rm4e5mhp`). These are
bounded static conclusions, not a proof of ordinary reachability or complete
control flow. Unaligned literals, alternate entries, larger or dynamic
offsets, RAM-built pointers and indirect loads remain outside the census.

A third conditional consumer is substitution dispatcher `019e94`, whose
byte index can calculate the same 22 words as indices 173..194. Wine
`bounded-range-dispatcher-domain-wine-knxnixuk` compares its original prefix
for those indices, neighbors 172/195 and `0x100` aliases: 96 full-state/clock
cases in interpreter and generated instruction dispatch with all 88 range
bytes altered. Eight neighboring cases serve no range bytes; each of 88
in-range cases serves four. The test stops before indirect invocation and
therefore proves neither ordinary use nor callback safety.

Indexed-context getter `01972c` can also address the range owner through
indices 206..227. Wine `bounded-range-context-overlap-wine-3lrpx1l2`
compares those indices, neighbors 205/228 and `0x100` aliases with all 88
range-source bytes altered: 96 complete state/clock cases in interpreter and
generated instruction dispatch. Each of 88 in-range cases serves one range
byte; the eight neighbor cases serve none. These controlled inputs do not
establish ordinary reachability or exclude other consumers, and generated
blocks may fall back to interpretation.

A separate aligned ARM BL pattern census
`bounded-range-arm-caller-census-odbnk6j8` finds no direct candidates to the
reader or either wrapper; it does not exclude indirect or register calls.
A passive search of 428 previously protected target checkpoint paths
(`bounded-range-checkpoint-pointer-scan-50n30lt7`) validates 339 unique
payloads, of which 15 are invalid or another region, and finds no exact Thumb
pointer to the 16 nearest preceding push candidates. Nearest pushes need not
be actual function starts, serialized matches need not prove execution, and
other possible entries/checkpoints remain unexamined. Eight paths under a
symbolic ancestor were excluded after the validation runner refused them.

Three distinct recorded E170 Wine CLI windows read zero range-source bytes:
240 frames entering the Giza battle (`bounded-range-battle-locator-dpeuvczs`),
372 frames from Royal Valley's system menu to Mission (`bounded-range-mission-
locator-hn5vy0hm`), and 120 frames of Bank entry (`bounded-range-bank-locator-
bfqintcy`). These are bounded aggregate-counter negatives, not proof that the
candidate calls cannot run elsewhere. Protected inputs passed and Wine drained;
no raw/managed pair was run for a negative window. The Royal Valley checkpoint's
prior save ancestry is not independently established.

A separate source-guided EUR Bank continuation starts from the retained
`open-monster-3` state, whose EWRAM task-list candidate contains callback
`05b625` and state byte 24. The task resides in EWRAM; an earlier scan restricted
to IWRAM headers (`b8j9lctj`) did not find it, while the EWRAM scan
`_zm3zlw2` finds this candidate among five saved task slots. The original
state-24 code tests input bit 4 before
either `05be34` or `05be8a` invokes wrapper `01abfc`. A new isolated Select
input produces a positive E170 Wine locator (`5n6dsl9_`): 23 source bytes and
five records, while the retained neutral/Down windows above remain negative.
The source and saved task are a conditional lead; the locator alone does not
prove which instruction executed or establish pre-checkpoint ancestry.

Wine pair `bounded-range-bank-pair-wine-acebndyu` repeats that 120-frame input
with all 88 managed source bytes altered. Full machine state, guest clock,
central/wide framebuffer, OAM, dispatch counts and nonempty mixed, PSG, both
FIFO and HLE audio match each frame. At guest frame 3669 the trace records 12
typed requests, 23 bytes across five records, zero raw/unclassified or dropped
requests. The production CLI reports the same byte/record totals. The 254,538
generated and 819,919 fallback dispatches are aggregate; all 12 events in that
trace lack interpreter instruction attribution. A separate interpreter replay
of the positive frame (`hbu4s4d0`) observes `05be8a → 01abfc → 01abc0` and
12 typed requests at original reader PCs `01abce`, `01abd4`, `01abda` and
`01abde`. Raw and managed interpreter state, clock, video, OAM and audio match
for that frame with the source bytes altered. The initial locator `asue_kh1`
produced positive logs but its unit failed the private-input retention guard;
`5n6dsl9_` explicitly retains the input for this pair.

The initial attribution attempt (`6s36aeat`) compared a native raw frame with
an interpreter managed frame and stopped at clocks 1030804556 and 1030804546,
respectively, before its state or output assertions. The raw/raw diagnostic
`ln8zprqs` reproduces those endpoints without managed resources; one additional
interpreter step advances to 1030804570, skipping the native clock, so complete
state equality at a common clock was not established. Its audio buffers match
at the observed, unequal endpoints; the original result key incorrectly says
`at_aligned_clock`. The source test now names that comparison accurately.
A Sol consultation (`/root/sol_range_clock_diagnosis`, requested
`gpt-6.1-sol`/`xhigh`; effective settings unconfirmed) identified that the
protected legacy native cache lacks the current event-guard tag. The follow-up
`je_s2es2` confirms interpreter PC `08000450`, native PC `0800044c` and an
unmasked pending IRQ in both; the source loop suffix is consistent with the
10-cycle difference. Protected snapshot audit `7tgvy6tq` finds two differing
payload bytes, accounted for by the serialized PC and a unique varint matching
clock, but does not independently parse the full schema. Native instruction
attribution and cross-mode IRQ equivalence for that legacy object remain
unproved. Primary Terra and effort could not be independently confirmed.

The same raw/raw frame using the previously retained event-guarded translation
`qpbgtmmw` passes (`u_28pjng`): both paths stop at clock 1030804546 and
`08000450`, with equal complete serialized state and audio. A separate 120-frame
raw/managed E170 pair on that translation (`jyionxk1`) also passes the complete
state, clock, video, OAM and audio comparison, with 23 typed source bytes and
five records, zero raw/unclassified or dropped requests. Both dispatch modes
occur. This is one isolated guarded translation; other scenes are not established.

An isolated portable test now calls the same cache selector used by `play`.
It cold-builds revision 8, removes an older revision only inside its owned
temporary root, and then reuses the same file without changing its hash or
modification time. The selected object has 29,196 blocks, 58,381 occurrences
of the event-guard tag and exactly the SHA-256 of the guarded translation
above. A 120-frame Bank pair using that selected object also matches full
state, clock, video, OAM and audio with 23 typed range bytes. Wine unit
`bounded-range-production-cache-wine-kitubxih` remains **FAIL**: although
both tests exited successfully and Wine drained, its temporary ROM copy
violated the runner's private-input retention check and the unit was retained.
Read-only audit `bounded-range-production-cache-evidence-audit-s_g0xk37`
confirms the results, object hash and unchanged protected inputs without
rerunning the tests or reclassifying the failed unit. This calls the selector
in a test process, not a packaged translation; tag presence does not prove
guard coverage for every block.

The actual portable Windows GUI subsequently cold-built and loaded the same
29,196-block object under Wine/Xvfb. Its unit
`bounded-range-production-gui-cache-wine-wamu6k3z` remains **FAIL**: it reached
`STATUS playing`, but the bounded GUI replay aborted because no GPU presenter
was available, before any replay frame was verified. Read-only audit
`bounded-range-production-gui-cache-evidence-audit-p7tuq7iz` confirms the
startup log, the exact object hash and unchanged protected inputs. This is
evidence of that isolated GUI cache selection, not a successful GUI replay.

An independent pair of actual GUI debug sessions runs the authenticated Bank
Select input under Wine/Xvfb (`bounded-range-bank-gui-pair-wine-t6rwu_qk`).
The raw and E170 managed interpreter sessions match complete state hashes and
guest clocks at the initial point and after 8, 20 and 120 frames; the managed
session reports 12 range reads serving 23 bytes across five records. Both
windows close normally, protected inputs remain unchanged and Wine drains.
The Vulkan presenter uses llvmpipe. This comparison uses unaltered ROM bytes
and does not compare video or audio per frame; it does not supersede the
separate altered-source native pair or establish Fast coverage.

A second independent Wine/Xvfb GUI pair uses native dispatch for the same
120-frame input (`bounded-range-bank-gui-native-pair-wine-g3fnb1x5`). Both
sessions select the previously verified guarded translation; complete-state
hashes and clocks match at the same four endpoints, the managed session serves
23 range bytes across five records, and both sessions close normally. Protected
inputs remain unchanged, scratch is removed and Wine drains. The managed log
contains a debug-control access-denied diagnostic and the raw log a D-Bus
assertion; the requested controls and both closes nevertheless succeeded. Raw
source bytes were unaltered and video/audio were not compared per frame;
native dispatch may fall back to interpretation.

Wine/Xvfb `e170-gui-load-undo-post-o9pxy5_a` follows the same Bank input in
isolated raw and E170 managed GUI sessions, using native dispatch and paused
debug-control steps. The actual menu performs Load, Undo and Load again; at
each operation both sessions have equal complete-state hashes and guest clocks,
and Undo restores the pre-load state. Repeating the same 120-frame Bank input
after the second Load reaches the original stepped state in both sessions.
The managed run reports 46 range-source bytes across the two passes; the
otherwise equivalent single-pass GUI run `w0cdul_b` reports 23. The copied
slot and protected inputs remain unchanged, both windows close normally, Wine
drains and successful scratch is removed. This is endpoint and aggregate-use
evidence, not per-frame video/audio comparison or exact per-phase attribution;
generated dispatch may fall back to interpretation. Wine logged transient
debug-control access-denied messages and a D-Bus assertion while both sessions
and all requested controls nevertheless completed. The earlier `2myj8gap`
attempt stopped before Load because its driver looked for an obsolete submenu.

Wine/Xvfb `e170-gui-backup-different-xssda1xe` exercises the actual backup
menu in isolated raw and E170 managed Bank sessions. Each starts with a
synthetic 65,536-byte save and its backup; after the first 120 controlled
frames, the driver replaces the current test save with different synthetic
bytes and verifies the difference before navigating the paused menu. The
confirmation defaults to Back; Restore and restart restores the backup bytes,
leaves the backup unchanged and resets the session to frame zero. Loading the
copied Bank slot and repeating the same 120-frame input reaches the previous
stepped state. Complete-state hashes and guest clocks match between raw and
managed at all five endpoints. The managed counter reports 46 range bytes for
both passes together; the earlier single-pass GUI run reports 23. This is an
aggregate count, not direct attribution to either phase.

The copied slot and protected inputs are unchanged, both windows close, Wine
drains and successful scratch is removed. This checks endpoint state and a
synthetic save, not per-frame video/audio, real-time gameplay, a personal save,
native Windows, other scenes or mods; generated dispatch may fall back to
interpretation. The sentinel was checked before menu navigation, not immediately
before confirmation. Wine logged transient debug-control access-denied messages
and a D-Bus assertion despite successful controls and clean exits. Earlier
attempts `49yki2lw` and `caqte4ns` stopped on driver assumptions about the
backup directory and the page shown after restoration, respectively.

Wine/Xvfb `bounded-range-bank-gui-fast-traced-wine-a6u0qv7x` follows the Bank
input with 12 neutral frames in the managed GUI. The UI trace records the Fast
toggle followed by eight completed frames, and the menu reports Fast 400% ON.
A raw native-dispatch session takes the same input and 12 neutral controlled
frames; complete-state hashes and clocks match at all four checked endpoints.
The managed session reports 23 range-source bytes over its whole execution;
the counter does not attribute any of those reads to Fast itself. Both sessions
close normally, protected inputs pass, scratch is removed and Wine drains. The
comparison uses unaltered ROM bytes and does not compare video or audio each
frame; generated dispatch can fall back to interpretation. Earlier Fast run
`sk4e6n7b` passed but did not retain the event order.

Wine/Xvfb `bounded-range-bank-gui-fast-positive-wine-hr3m809u` activates
Fast in the paused menu before the first guest frame, then executes 48 traced
frames including Select. The managed GUI reports 12 range reads serving 23
bytes over five records. A raw native-dispatch session replays the exact 48
observed input masks and matches complete state and clock at the initial and
final endpoints. Both sessions use the guarded translation, close normally,
and preserve protected inputs; Wine drains and scratch is removed. The
earlier attempt `kpt4doby` failed before guest execution because its driver
looked for the wrong menu label. ROM bytes were not altered, video/audio were
not compared per frame and generated dispatch may fall back to interpretation.

Other reader contexts, legacy cross-mode IRQ equivalence, broader Wine
regressions, Linux, USA and native Windows remain open.
No performance measurement was performed.


## European tilemap geometry projections

EUR schema 1/e162-d161 publishes `graphics.tilemap-geometry` for six
independently rooted eight-byte groups at `371464..371494`. Each group
represents four little-endian halfwords. The first three are read by the
constructor at `0801c7a0`; the next three are read by `0801f208` and, for
the width and height groups, by additional original readers. The fields
retain separate tilemap-byte, width and height values for the constructor
and global state, even where the current ROM values agree. The guest's
index remains an unbounded byte: these four-entry physical projections do
not impose a guest bound, and reads outside the covered addresses compose
with other owners or raw ROM.

Wine source units `e162-adjacent-range-dis-wine-i36cx5vz`,
`e162-adjacent-callers-dis-wine-vl8mhv5l` and
`e162-adjacent-second-dis-wine-_9loiqbv` authenticate the reader and five
observed direct call sites; those five load index 0. This is a bounded scan
and does not establish all indirect callers or the global selector domain.
The 48-byte source SHA-256 is
`955f43accb6baf3c8d6743c1838ecd5724f154ff1da17648f93f409b6f65b6e7`.

Wine `e162-tilemap-geometry-wine-5zpi8vam` passes creation and reuse,
preserves all 104 prior resources, and verifies all 48 typed source bytes,
source guards and mixed boundaries with altered ROM source bytes. It checks
24 original-reader cases over indices 0–4 and 255, including the full
`0801f208` return and the `0801c7a0` prefix, against interpreter and generated
dispatch states and guest clocks. Generated dispatch can fall back to the
interpreter. Production Wine CLI `e162-tilemap-geometry-cli-wine-j4r9sopl`
creates, reuses and verifies all 106 files against the tested generation. Its
11-frame world-map window reads no geometry source bytes; this is not evidence
of absence in other scenes.

An existing E169 production Wine continuation from a recorded home checkpoint
reads 56 managed geometry source bytes in 28 bus reads, spanning two aggregate
groups (`e181-text-reader-registry-cli-wine-mr5vn2od`). Its raw and managed
paths match framebuffer and OAM for 1,800 frames, final serialized state and
final image. Read-only evidence audit
`e169-special-geometry-existing-evidence-db6l43_0` verifies the retained logs,
input guards and runtime counter implementation. The counters do not identify
the groups or readers. A subsequent E170 Wine pair
(`geometry-special-ordinary-wine-lby31j3d`) identifies the two groups as
state width and state height (group indices 4 and 5): 28 typed requests serve
56 source bytes. With all 48 geometry bytes and 3,856 special-pose index bytes
altered in the managed machine, full state, guest clock, central/wide video,
OAM and five nonempty audio streams match at each of the 1,800 frames. The
trace dropped no requests; it identifies addresses, not the original reader
instruction. The checkpoint's recorded lineage was not independently
reconstructed end-to-end.

Other readers and index domains, broader Wine regression checks, Linux, USA and
native Windows remain open. No performance measurement was made.


## E213 European eight-row tile lookup

E205-d204 extends `graphics.tilemap-geometry` with the 16 signed bytes at
`371024..371034`, arranged as four physical rows. The original routine at
`08017708` uses these values in its eight-row drawing paths, including the two
horizontal orientations. Nonnegative values select tile offsets; the original
negative-value branches remain guest work. The following byte belongs to the
existing task-cycle resource. The new source projection imposes no new guest
index bound and does not claim other consumers or alternate entries.

Earlier source inspection `bounded-range-geometry-windows-rm4e5mhp` establishes
the conditional dimensions and both lookup loops. The exact reader window and
the 16-byte source are authenticated during extraction; the lookup SHA-256 is
`22b8321c6014682807e4102853d6cd15cd56652bda09f45215746eb84f4d2acf`.

Wine `e213-tilemap-wine-y3wk6uev` verifies every source byte, mixed boundaries
and ROM mirrors, the adjacent task-cycle owner, 122 unchanged resources,
source authentication, owner retention and five original interpreter and
generated-dispatch cases. The non-eight-row case makes no lookup reads;
generated dispatch may use interpreter fallback. Wine
`e213-home-scene-wine-7fmxa2bo` matches state, guest clock, video, OAM,
dispatch and audio on every frame of a 600-frame recorded home continuation
with the lookup's managed ROM backing altered. It serves 1,048 lookup bytes
across all four rows; that counter is an aggregate, not a unique-byte count
or PC attribution. Wine CLI `e213-tilemap-cli-wine-ga0p99vm` passes create,
reuse and verify with all 124 files matching the tested dataset.

The reader, scene and offline CLI proofs preceded a diagnostic-only correction
that preserves the existing six-group counter. Their data and guest execution
paths did not change. Wine `e213-counter-wine-li2pps9d` checks the final counter
behavior: the new rows do not change the legacy group count, and all six old
groups remain countable. The initial scene launch was refused before Wine
started because the artifact lease was active during a concurrent build; the
retry followed its release and passed. The Wine servers drained and the local
overlay avoided the previously observed D-Bus assertion.

Static inventory `e213-home-residual-d9rkhtks` leaves 2,110 of the 8,544
originally unowned E207 sample bytes outside E205 source ownership. Other
consumers, scenes, real-BIOS and UI save/Load paths remain open; Linux, USA and
native Windows are deferred. No performance comparison was performed.

## E162 bounded European scene equivalence

Wine `e162-scene-matrix-wine-ig6dmb65` compares the unmodified ROM path
with E162 managed resources at four authenticated retained EUR checkpoints:
the opening, world menu, language selector and battle menu. Each pair uses
three identical input frames. At every boundary the serialized machine state
and guest clock, framebuffer, wide framebuffer, OAM and captured audio match.
Loading each existing state after attaching the dataset also preserves the
active resources for these windows. The two menu cases use A, the language
selector uses Down, and the opening uses neutral input; no completed transition
is inferred from these short windows.

All 75 declared substitution callback words, the 88-byte range lookup and
the 48-byte geometry projection were altered in the managed ROM copy while
running. The bounded trace recorded no reads in those selected ranges in any
of the four windows, so this is equivalence evidence, not a positive-use proof
for the new sources. It does not establish absence elsewhere. The first Wine
attempt `pau01iy9` stopped before scene execution because the diagnostic API
accepts at most 32 filter ranges; the corrected test captures the enclosing
range and retains only events overlapping the actual owned intervals. The
server drained for both attempts.

Separately, Wine `e162-cold-boot-pair-wine-dcq3ml1a` runs the production
Windows CLI for eight neutral frames with no supplied save or state. The raw
and managed final machine-state hashes and output images match; the managed
run reads zero geometry source bytes. The snapshot-hash option does not measure
time, and timing collectors were disabled. This checks the final state, not
every frame. Both comparisons are limited to these windows; full scenes, other
inputs, further consumers, Linux, USA and native Windows remain open. No
performance measurement was made.

## E170 post-indexed ROM read inventory

Wine `e170-post-indexed-raw-audit-wine-108ejhdv` observes the next 12 neutral
frames after the retained guarded indexed-archive continuation. The audited
input matches the original recording for that window. The E170 dataset, ROM,
checkpoint, input and guarded translation were protected; the instrumented and
reference machines match complete state, clock, video, OAM, dispatch and audio
at every frame. Wine drained and successful scratch was removed.

The instrumented bus and selected host readers report 33,077 distinct physical
ROM bytes, of which 4,680 are outside E170 manifest source spans. Static
reconciliation `e170-post-indexed-reconcile-c8paxoub` verifies prior source
witnesses against the exact ROM. Of the unowned bytes, 3,600 fall inside the
previously documented 3,648-byte copied-IWRAM block containing executable
support; another 12 match hardware/branch-support sources. Of the remaining
1,068 lower-ROM bytes, 500 overlap earlier scoped source witnesses and 568 do
not match those particular witnesses. The old witnesses do not identify the
current reader PCs, and an unmatched byte is not by itself an asset. In
particular, the byte reads at `14d2dc..14d2ec` were investigated separately
below.

This is one bounded scene and only the instrumented readers; it does not prove
global coverage, classify every byte, or establish exact instruction sites.
Other scenes and consumers, Linux, USA and native Windows remain open. No
performance measurement was made.

## E207 home-scene ROM read inventory

Wine `e207-home-audit-wine-kkbk45be` inventories the first 600 recorded frames
of the previously verified European home continuation using the E202-d201
dataset. The ROM, recording, snapshot, compiled code, dataset, executable and
source preimages were protected. Reference and managed machines match complete
state, guest clock, video, OAM, dispatch and five nonempty audio streams at
every frame. The Wine server drained and the local Bluetooth overlay avoided
the previously observed D-Bus assertion in this run.

The selected bus and host readers observed 97,551 distinct ROM bytes; 8,544
were outside the E202 manifest's source spans. Of those, 6,436 are at or
above `300000`, including 6,041 bytes in three observed intervals
`3fb21c..3fbb52`, `3fc3b0..3fcb75` and `3fd4f0..3fdb8e` (ends exclusive).
These are sampled read ranges, not authenticated asset boundaries. The report
does not identify guest instruction sites, infer ownership or prove that an
unowned byte is data; attribution requires its original reader and source
contract. The E206 reconciliation left 2,108 sampled bytes; later ownership
reduced this to 272 at E219 and 172 at E220. E221 owns the remaining 172
(`e230-home-residual-dnr49xgd`): all 8,544 addresses in this earlier sample now
have owners. Other scenes and readers, Linux, USA and native Windows remain open.
No performance measurement was made.

## E215 Llednar battle ROM read inventory

Wine `e215-llednar-audit-wine-31k333ug` compares 600 neutral frames from a
retained European Llednar battle checkpoint, guest frames 30924–31524, using
the E206-d205 dataset. The checkpoint and neutral continuation match the
retained plan and its protected inventory. The supplied save's earlier ancestry
is unknown; no new cheat or state forcing was used in this continuation.
Reference and managed machines match complete state, guest clock, video, OAM,
dispatch and five nonempty audio streams at every frame. This does not assert
equality with the earlier helper's separately recorded endpoint.

The selected bus and host readers report 255,853 distinct physical ROM bytes,
of which 4,281 are outside the E206 manifest. Of those, 453 are at or above
`300000` and 3,828 below it. The largest high intervals observed are
`416254..41634a` (246 bytes), `41142c..41146e` (66 bytes), and
`4161e0..416220` (64 bytes), with exclusive ends. They are sampled addresses,
not authenticated asset boundaries, and the audit does not identify executing
PCs. Instruction fetches, folded constants, unselected host readers and other
scenes remain outside its scope.

The Wine child passed and its server drained without the D-Bus assertion
using the local overlay. The original unit was marked FAIL during scratch
retention because its generated neutral `.input` file was treated as private;
the entire failed unit remains retained. Recovery
`e215-llednar-audit-recovery-nocdwh09` independently rechecks all 37 protected
input paths, the Wine log, neutral input, full report hash and address ownership
against the E206 manifest, without replaying the game. It preserves the full
address report and passes its own validation unit. E216 attributes 424 sampled
bytes, E217 owns another 16, E218 another nine, E219 one more, E220 another
500, E221 another 956, E222 another 468, E223 another 1,772, E224 another
108, E225 another 16 and E226 another eight lower bytes. E227 covers two high
bytes and E228 the last high byte: all 4,281 sampled bytes now have E219 source
owners (`e228-llednar-residual-3blfucoi`). Other scenes, consumers, Linux, USA and
native Windows remain open; no performance comparison was made.

## European display-control source tables at 14d2dc

Wine site observation `e170-post-indexed-lower-sites-wine-9ei8p1cy` records
byte reads at both ends of `14d2dc..14d2ec` in the original copy loop at
`08147e0c`; other accesses to those bytes have no interpreter scope. The same
window records original word readers at `0801f47a/7e` and `0801f52c/30` for
the adjacent `14d2ec..14d2fc` table. Static source reconciliation
`e170-context-control-source-s94ilj14` verifies that each of the two caller
branches loads three original pointers, copies two eight-byte arrays to its
stack through the same original byte-copy routine, and copies four aligned
IWRAM pointers. The original caller then combines selected halfwords and uses
the selected pointer for live structure writes. The two arrays are represented
as four little-endian halfwords each; the pointer table has four 32-bit entries.
These shapes follow the original copy operations, not a guessed neighboring
extent. The three spans total 32 bytes, have source SHA-256
`5c5502e34c5e00cb3c556b9bd4f9e8582c6dec845b71c15669f362f30d020760`, and do
not overlap any E170 source binding.

The scoped copy-loop reads and the two static caller paths establish a bounded
source contract; the trace alone does not tie each copy invocation to a
specific caller. The pointers refer to live IWRAM rather than replacing those
mutable structures. No guest selector cap, full value semantics or global
caller domain is inferred.

E171-d170 publishes `graphics.display-control-tables` as two arrays of four
halfwords and four IWRAM pointers in `display-control-tables.json`. The EUR
source image, original reader/copy instructions, pointer literals and source
spans are authenticated before publication. The runtime reconstructs the
three exact source spans from the typed data, composing with earlier owners or
raw ROM for other bytes; it does not replace the live structures in IWRAM.
All 113 E170 resource files and their manifest entries match the predecessor.

Wine `e171-display-control-wine-8bb1ykny` verifies creation and reuse, source
and mixed-boundary reads across the three ROM mirrors with the 32 backing ROM
bytes inverted, coherent-data/overlap rejection and transactional failed
attachment. Four original reader paths each match complete state and guest
clock in interpreter and compiled execution. A separate 12-frame guarded
raw/managed continuation matches state, guest clock, video, OAM, dispatch and
nonempty audio at each frame while those managed source bytes are inverted;
aggregate counters report 32 halfword-source and 32 pointer-source bytes.
Production CLI Wine `e171-display-control-cli-wine-ikwqgxwg` creates, reuses
and verifies a 114-resource generation; all 115 files match the tested dataset
and reuse/verification leave that generation unchanged. Both Wine units drained
the shared server. Generated dispatch may fall back to interpretation, and
aggregate counts do not identify individual instructions. The 536 other
unmatched lower-ROM bytes, other readers, selector domains, Linux, USA and
native Windows remain open.
No performance measurement was made.

## E188 observed IWRAM OBJ callback words

EUR E188-d187 extends `graphics.obj-static-lists` with five observed Thumb
callback words at `397f80..397f94`. Their addresses point into the original
block copied to IWRAM, and their Thumb bits are retained. The resource owns only
the five encoded words; it does not freeze the live executable bytes, restrict
other guest pointers, or replace the original allocator and dispatch behavior.
The previously documented IWRAM copy and OBJ draw bridges establish the source
provenance for this bounded projection.

Source `e188-control-source-rm56pdw9` authenticates the 20 bytes and the
original copy/reader windows. Wine traces
`e188-control-trace-wine-sr60q_xk` and
`e188-control-tail-trace-wine-fo65p2xu` observe all five original readers
through interpreter fallback. Wine `e188-control-wine-ryj41yi9` preserves the
previous OBJ fields and 119 other resources, rejects coherent file/hash
edits, checks poisoned mirrors and mixed boundaries, and compares five original
interpreter/separately compiled load-and-branch entries, including target PC,
Thumb state, return register, complete state and guest clock.

Wine `e188-control-scene-wine-ch94duvy` compares the recorded 600-frame
Red Spring action while all 20 backing ROM bytes are altered in the managed
machine. All five words are read from the extracted resource, and full state,
guest clock, central/wide video, OAM, dispatch and audio match every frame.
Wine `e188-control-cli-wine-3z2jda18` creates, reuses and verifies 121 files
matching the tested generation. Other callback words, callers and live IWRAM
code paths remain open, as do Linux, USA and native Windows. Historical
revision-pinned OBJ fixtures need adaptation before reuse. No performance
measurements were performed.

## E189 DMA register directories

EUR E189-d188 publishes `engine.dma-register-directories`, containing two
four-word DMA register address directories at `370e68..370e88`. The guest
continues to compute indices, access hardware and perform DMA. The 32 managed
bytes include 24 sampled as unowned in E172; the projection does not impose a
guest index bound. Source `e189-dma-source-_zu8l44g` authenticates the original
directories and reader windows. Wine `e189-dma-trace-wine-vgsu5kue` observes
both tables during native dispatch, without identifying the exact instruction.

Wine `e189-dma-wine-598qsw1v` passes publication, mixed boundaries, 16
interpreter/separately compiled reader cases and four controlled DMA register
writes with matching full state and guest clock. The first attempt
`e189-dma-wine-gsiv7n3d` failed because the test invoked a chained native store
twice; the corrected retry passed. Wine `e189-dma-scene-wine-dgpightt`
compares the recorded 600-frame Red Spring action with all 32 backing bytes
altered in the managed machine. Both directories are served from the resource;
full state, guest clock, central/wide video, OAM, dispatch and audio match each
frame. Wine `e189-dma-cli-wine-98p9_ms6` creates, reuses and verifies 122 files
matching the tested generation. Other DMA callers, Linux, USA and native
Windows remain open. E190 subsequently integrates the separate IWRAM helper
vector. No performance measurements were performed.

## E190 IWRAM memory-helper vector

EUR E190-d189 publishes `engine.iwram-memory-helpers` for the three Thumb
pointer words at `370eb0..370ebc`. The guest continues to select and execute
its IWRAM code; the resource owns only the encoded pointer bytes and does not
bound other guest indices or freeze mutable instructions. Source
`e190-helper-source-2muj949h` authenticates the vector, selected original
readers and matching ROM code prefixes. The previously verified original
initializer and copied code are described in the [palette source contract](rom_data_ui_palettes.md#source-and-consumer-contract).
All 12 new bytes were sampled as unowned in E172.

Wine `e190-helper-trace-wine-1jrexdnx` observes all three reads: the first in
native dispatch without exact instruction attribution and the other two with
interpreter fallback. Wine `e190-helper-wine-r9rce5o_` preserves the other 121
resources, rejects coherent file/hash edits, checks altered backing bytes and
mirrored/mixed boundaries, and compares three interpreter/separately compiled
load-and-branch entries with matching full state and guest clock.

Wine `e190-helper-scene-wine-94ao1rz2` compares the recorded 600-frame Red
Spring action with all 12 backing bytes altered in the managed machine. All
three pointers are served from the resource; full state, guest clock,
central/wide video, OAM, dispatch and audio match every frame. Wine
`e190-helper-cli-wine-h_owgk16` creates, reuses and verifies 123 files matching
the tested generation. Other callers, live IWRAM code paths, Linux, USA and
native Windows remain open. No performance measurements were performed.

## E191 independently rooted IWRAM OBJ callbacks

EUR E191-d190 extends `graphics.obj-static-lists` with the two separately
rooted Thumb callback words at `397fa4` and `397fb8`. Their original code is
part of the copied IWRAM block described in the E188 contract; only the
encoded words are managed. Guest indexing, callback selection and mutable
IWRAM instructions remain under guest control. All eight bytes were sampled
as unowned in E172.

Source `e191-callback-source-m460uxk9` authenticates the source words,
selected reader windows and copied-code prefixes. Wine
`e191-callback-trace-wine-35oh5q8o` observes the original reads at
`0809b6fc` and `0809e40e` through interpreter fallback. Wine
`e191-callback-wine-gj6h0gcr` preserves the previous OBJ fields and 121 other
resources, rejects coherent file/hash edits, checks altered backing bytes and
mirrored/mixed boundaries, and compares both original interpreter/separately
compiled load-and-branch entries with matching full state and guest clock.

Wine `e191-callback-scene-wine-_vpbz8be` compares the recorded 600-frame Red
Spring action with all eight backing ROM bytes altered in the managed machine.
Both callbacks are served from the resource; full state, guest clock,
central/wide video, OAM, dispatch and audio match every frame. Wine
`e191-callback-cli-wine-fttzwwhr` creates, reuses and verifies 123 files
matching the tested generation. The initial build launch `89y445bs` used an
invalid relative script path; `e_8ykhv5` was interrupted and the subsequent
`k1nk95_g` completed. Other callbacks, callers and live IWRAM paths, Linux,
USA and native Windows remain open. No performance measurements were made.

EUR E192-d191 extends the runtime's linked-records4 resource with three
[callback partitions](rom_data_gameplay.md#e192-linked-record-callback-tables);
the original guest still selects and executes the callbacks.

EUR E193-d192 also serves the adjacent [action-callback records](rom_data_gameplay.md#e193-action-callback-records);
the original guest retains selection, dispatch and reads outside that partition.

## E194 adjacent offset tables

EUR E194-d193 publishes `engine.adjacent-offset-tables` for five independently
rooted physical tables at `3ad460..3ad480`: four four-byte rows followed by
four four-byte tables. The next separately rooted source starts at `3ad480`.
All original byte values, including padding and values later read as signed,
are preserved. The projection does not bound guest indices or replace original
selection and arithmetic; reads outside the owned range continue through their
existing owners or the ROM. Five bytes were observed unowned in E172.

Source `e194-offset-source-r364fmf9` authenticates the 32 bytes, selected
reader windows and successor. Wine `e194-offset-trace-wine-uxtnxes3` observes
reads from four tables through interpreter fallback. Wine
`e194-offset-wine-ug76r3cd` preserves all 122 previous resources, rejects a
coherent file/hash edit, checks altered, mirrored and mixed boundary reads, and
compares six original interpreter/separately compiled byte readers with full
state and guest clock equal. This includes controlled signed-byte readers and
the separately rooted fifth table, whose ordinary use remains unproved.

Wine `e194-offset-scene-wine-dh8ic7ya` compares the recorded 600-frame Red
Spring action with all 32 backing bytes altered in the managed machine. Four
tables are served from the resource; full state, guest clock, central/wide
video, OAM, dispatch and audio match every frame. Wine
`e194-offset-cli-wine-7ind1kk9` creates, reuses and verifies 124 files matching
the tested generation. Reader and scene logs contain D-Bus pending-call
assertions before successful completion; their cause remains unestablished.
Other readers, fifth-table ordinary use, Linux, USA and native Windows remain
open. No performance measurements were made.

EUR E195-d194 also serves [two tilemap alternatives and their selector](rom_data_graphics.md#e195-alternate-tilemap-rectangles)
through the existing observed-tilemap resource.

## E220 European Thumb switch-table resources

EUR E211-d210 publishes `engine.thumb-switch-tables`: 23 physical tables with
696 even Thumb destinations and their 23 table-root words, covering 2,876
authenticated source bytes. Twenty-two use a guarded unsigned comparison and
common word-load/`MOV pc` sequence; the reader rooted at `0cbef4` instead uses
its original byte-sized selector and compare loop. Source windows, root
literals, bounds, all destinations and previous-owner disjointness are checked
against the exact EUR ROM. The physical table extents are not new guest input
limits: the original comparisons, exceptional paths, state and branches remain
unchanged, and outside reads continue through previous owners or raw ROM. The
existing text-reader owner immediately before `0191a4` is preserved.

Wine `e220-thumb-switches-wine-83yciuax` verifies every source byte, preserves
125 previous resources, rejects coherent edits, checks mixed and mirrored
boundaries, DMA/CpuSet/CpuFastSet and snapshot-owner retention. It compares 92
guarded original interpreter/generated-prefix cases plus 46 direct load-sequence
cases outside the physical ranges. The test uses interpreter execution for the
indirect `MOV pc`; it does not claim that instruction was natively generated.
Initial unit `e220-thumb-switches-wine-nwvu89dw` failed because generated blocks
chained past a test stop; the corrected test passes, and the failed unit remains
retained. The production build predates this test-only correction; its guarded
production inputs and executable were unchanged.

Wine `e220-llednar-switches-wine-01z89owc` compares 600 retained European
Llednar battle frames with all 2,876 ROM backing bytes altered. All 23 roots and
23 destination tables are read from the extracted resource; state, guest clock,
video, OAM, dispatch and five nonempty audio streams match every frame. Counts
are aggregate reads, not unique entries or executing-PC attribution. The earlier
save ancestry is unknown; no new cheat or state forcing was used. Wine production
CLI `e220-thumb-switches-cli-wine-lr21g7ro` creates, reuses and verifies 127
files matching the tested dataset. The Wine servers drained using the local
Bluetooth overlay without the known assertion.

Static reconciliation `e220-llednar-residual-sxj55u1f` attributes 500 more
previously unowned sampled bytes, leaving 3,331, including three at or above
`300000`. These address samples do not prove executing PCs, other uses or
complete ROM coverage. Other tables, index domains, consumers and scenes,
real-BIOS, UI save/Load, Linux, USA and native Windows remain open. No
performance comparison was performed.

## E221 European Thumb PC-relative literals

EUR E212-d211 publishes `engine.thumb-pc-literals`: 326 independently located,
aligned little-endian words (1,304 bytes), each paired with an original Thumb
PC-relative `LDR` source witness. The model preserves each word's exact bits,
without assuming that an address-shaped value is dereferenced or identifying
any pointed-to asset. The original instruction, guest control flow and all
reads outside the selected words remain unchanged. Source authentication checks
the exact EUR ROM, the original opcode and its computed literal address, and
disjointness from previous owners.

The source set combines 153 previously unowned words from the earlier Bank
static-reader witnesses, 100 from F09 executed literal sites and 73 from E170
instruction-scoped observations. Each selected E170 opcode, width and effective
address was checked, but instruction scope alone can include nested accesses.
The earlier Bank contexts were static witnesses rather than dynamic PC traces.
The five remaining Bank literal roots and F09 root `0a114c` were already owned
by E211 switch tables; the new source does not replace those owners.

Wine `e221-thumb-literals-wine-e3auifw5` verifies all 1,304 source bytes and
126 unchanged previous resources, coherent-edit rejection, mixed and mirrored
boundaries including the adjacent switch owners, DMA/CpuSet/CpuFastSet and
snapshot-owner retention. It compares 32 original interpreter/generated reader
cases across 16 representative literals, preserving complete state and guest
clock. Wine `e221-llednar-literals-wine-qct03qw7` compares 600 retained EUR
battle frames with every selected ROM backing word altered. The resource serves
239 distinct selected literals; state, guest clock, video, OAM, dispatch and
five nonempty audio streams match each frame. The battle audit identifies
addresses rather than executing PCs. The earlier save's ancestry is unknown; no
new cheat or state forcing was used.

Wine production CLI `e221-thumb-literals-cli-wine-t5x79vsh` creates, reuses and
verifies 128 files matching the tested dataset. Static reconciliation
`e221-llednar-residual-2bz86_ck` attributes 956 more sampled bytes and leaves
2,375 unowned, including three at or above `300000`. Another static join,
`e221-redspring-prior-sites-so67hagh`, shows that all 105 selected E203 prior
site words are now owned. It does not reclassify all Red Spring addresses or
prove executing PCs in that scene. Other literals, instruction paths, scenes,
real-BIOS, UI save/Load, Linux, USA and native Windows remain open. Wine servers
drained using the local Bluetooth overlay without the known assertion; no
performance comparison was performed.

## E222 additional statically anchored Thumb literals

EUR E213-d212 extends `engine.thumb-pc-literals` with 117 aligned four-byte
words (468 bytes), preserving the original 326 literal entries and their
coverage mask. The new entries preserve exact word bits and original Thumb
PC-relative reader locations; no pointed-to asset or guest meaning is inferred.
The source plan `e222-literal-source-gzj3f255` authenticates the exact EUR ROM
and existing translation, checks the original LDR operand, and anchors each
reader to an exported translated block through a conservative straight-line
original-instruction path of at most 96 bytes. The anchors and E215 addresses
are static evidence, not proof of the executing PC in the battle. All source
spans are disjoint from earlier owners, and original control flow and outside
reads remain unchanged.

Wine `e222-literal-extension-wine-0qe1983j` verifies all 468 additional source
bytes, the 326 prior literal values and source bindings, the legacy coverage
mask and 126 other unchanged resources. It rejects coherent edits and checks
mixed and mirrored boundaries, 32 original interpreter/generated reader cases,
DMA/CpuSet/CpuFastSet and snapshot-owner retention. Wine
`e222-llednar-extra-literals-wine-osmbx9xm` compares 600 retained EUR battle
frames with all 117 new ROM backing words altered. All 117 are served by the
resource; state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Earlier save ancestry is unknown; no new cheat or
state forcing was used.

Wine production CLI `e222-literal-extension-cli-wine-ajwb625v` creates, reuses
and verifies 128 files matching the tested dataset. Static reconciliation
`e222-llednar-residual-f73ffu_6` attributes 468 more sampled bytes and leaves
1,907 unowned, including three at or above `300000`. Other literals, callers,
scenes, real-BIOS, UI save/Load, Linux, USA and native Windows remain open.
The Wine servers drained using the local Bluetooth overlay without the known
assertion; no performance comparison was performed.

## E223 battle-observed Thumb literal words

EUR E214-d213 extends `engine.thumb-pc-literals` with 443 aligned four-byte
words, preserving the existing 443 words, both previous masks and all other
resources. The authenticated original Thumb LDR instruction, PC-relative
address and source bytes are checked before publishing. Values remain plain
words: the resource does not infer pointed-to assets or replace guest control
flow. Reads compose with prior owners and original backing bytes.

Wine capture `e223-llednar-full-sites-744c5av7` filters these 443 previously
sampled candidate words over 600 retained battle frames. It records 473 reads
inside interpreter execution scopes and matches the original LDR opcode,
effective address and four-byte width. Scopes can include nested HLE or DMA
accesses, so this evidence does not prove every access was made directly by the
scoped instruction, identify all callers, or establish complete literal pools.
The initial capture `rv5lkw9c` failed from a missing TinyCC path and batch
capture `9663ynlq` failed because its final 12-frame filter had no matches;
both Wine servers drained. The full capture supersedes neither limitation.

Windows test build `c4yhfgcu` and Wine `qeb1_wme` verify all 1,772 new source
bytes, the 443 preserved words and masks, 126 unchanged resources, coherent edit
rejection, mixed and mirrored boundaries, 32 original interpreter/generated
reader cases, DMA/CpuSet/CpuFastSet and snapshot-owner retention. Wine battle
`v5vdxrct` compares 600 retained EUR frames with all 443 new backing words
altered. Each word is consumed; state, guest clock, video, OAM, dispatch and
five nonempty audio streams match every frame. Earlier save ancestry is unknown;
no new cheat or state forcing was used.

Windows production build `fbx5uf3o` and Wine CLI `73o_xfzb` create, reuse and
verify 128 files matching the tested dataset. Static reconciliation `_lf4p19t`
attributes 1,772 more sampled bytes and leaves 135, including three at or above
`300000`. Other literals, readers, scenes, real-BIOS, UI save/Load, Linux, USA
and native Windows remain open. Wine servers drained using the local Bluetooth
overlay without the known assertion; no performance comparison was performed.

## E224 additional battle switch destinations

EUR E215-d214 extends `engine.thumb-switch-tables` with four destination arrays:
127 words at `0960f0`, 12 at `09bad0`, five at `0ac2e8` and 19 at `0ce214`.
The 163 even Thumb targets (652 bytes) are decoded from the authenticated EUR
source, while their four root words remain owned by `engine.thumb-pc-literals`.
The previous 23 switch tables and their masks, and all 126 other resources,
remain unchanged. Original guest instructions and selection behavior remain in
control; reads beyond the projected destinations still use prior owners or the
original backing ROM. In particular, the `0960da` out-of-range helper may
return, so the projection is not an added guest index guard.

Wine capture `e224-llednar-residual-sites-qyrfym7w` observed all 135 bytes left
by E214 over 600 retained battle frames; three switch load instructions have
interpreter scopes and the fourth table was read outside such a scope. Wine
original-code windows `e224-llednar-reader-disassembly-rit3fxua` establish the
four root literals, selector comparisons, load sequences and destination counts.
Execution scopes may include nested accesses; the observation alone does not
prove every caller or all other scene paths.

Windows test build `co8nrtac` and Wine `6k14siel` verify all new destination
bytes, the already-owned roots, 23 preserved switch tables and masks, 126 other
unchanged resources, coherent edit rejection, mixed/mirrored boundaries, 16
original interpreter/generated guarded prefixes, eight out-of-range original
load sequences, DMA/CpuSet/CpuFastSet and snapshot-owner retention. The indirect
`MOV pc` remains on the interpreter fallback in these tests. Wine battle
`lzpuxs2a` compares 600 retained EUR frames with all 652 new backing bytes
altered; all four arrays are consumed and state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. The earlier save
ancestry is unknown; no new cheat or state forcing was used.

Windows production build `f4xz16wj` and Wine CLI `4j66od25` create, reuse and
verify 128 matching files. Static reconciliation `03amsoxp` attributes 108 of
the 135 previously unowned sampled bytes, leaving 27, including three high.
Other destinations, callers, scenes, real-BIOS, UI save/Load, Linux, USA and
native Windows remain open. Wine servers drained with the local Bluetooth
overlay; no performance comparison was performed.

## E225 derived OBJ dimension views

EUR E216-d215 extends the source bindings of `graphics.obj-dimensions` with
12 four-byte records at `14d2fc` and another 12 at `14d32c`. The first view
contains base-two width and height exponents in eight-pixel units; the second
contains pixel widths and heights. Both are reconstructed from the existing
validated shape/size dimensions, with two authenticated zero bytes per record.
The dimensions file and all 126 other resources remain unchanged. Exact source
bytes, hashes, original reader instructions and root literals are verified
before publication; no raw-ROM shadow or inferred asset is added.

The original reader at `0801fb2a..0801fb4c` indexes both views. Its reserved
shape can address into the second view and beyond it; the runtime composes
physical bytes from the selected view, adjacent prior owner, or original backing
without imposing a new shape guard. The preceding display-control source is a
separate owner, and mixed reads across its boundary preserve both projections.
Wine site evidence `qyrfym7w` and original-code windows `rit3fxua` identify the
sampled reads and reader. Execution scopes may include nested accesses and do
not establish all callers.

Windows test build `2aefh61b` and Wine `riylj8n_` verify 96 source bytes, the
unchanged dimensions file and 126 other resources, coherent edit rejection,
mixed/mirrored boundaries including the previous owner and raw fallback, all
16 original shape/size combinations in interpreted and generated prefixes,
DMA/CpuSet/CpuFastSet and snapshot-owner retention. Wine battle `o9_mw7ee`
compares 600 retained EUR frames with both backing views altered; both views
are consumed and state, guest clock, video, OAM, dispatch and five nonempty
audio streams match each frame. Earlier save ancestry is unknown; no new cheat
or state forcing was used.

Windows production build `hbubspau` and Wine CLI `e_jxiaof` create, reuse and
verify 128 matching files. Static reconciliation `53vwjg84` attributes 16 of
the 27 previously unowned sampled bytes, leaving 11 in three intervals. Other
readers, scenes, real-BIOS, UI save/Load, Linux, USA and native Windows remain
open. Wine servers drained with the local Bluetooth overlay; no performance
comparison was performed.

## E226 copied halfword lookup tables

EUR E217-d216 publishes `engine.copied-halfword-lookups`: two authenticated
four-halfword tables at `14cf18` and `14cf20`, plus two four-byte PC-relative
root words at `3658` and `3680`. The first root at `3630` remains owned by
`engine.thumb-pc-literals`. The resource models 24 source bytes without embedding
their values or changing the other 127 resources.

The original routines at `0800360c`, `08003634` and `0800365c` copy eight
bytes onto the guest stack, then index that copy using a zero-extended byte.
The runtime preserves the original copy and stack behavior without adding a
selector guard: indices beyond the four copied entries continue to read the
original stack. Wine disassembly `e226-bitmask-reader-disassembly-u0b955k1`
authenticates the original readers, copy helper, roots and tables. Earlier Wine
observation `e224-llednar-residual-sites-qyrfym7w` saw the first table outside
an interpreter scope; it does not establish all callers.

Windows test build `nkbn7myx` and Wine reader `j75qi78l` verify all 24 source
bytes, unchanged prior resources and root ownership, coherent edit rejection,
mixed/mirrored boundaries, ten original interpreted/generated prefixes and
18 full original interpreter cases across all three routines, including indices
4 and 255. DMA/CpuSet/CpuFastSet, state and clock, and dataset/snapshot owner
retention also pass. An initial build `kcmxyfda` failed on a test argument
syntax error that was corrected before the passing build.

Wine battle `jv89wjir` compares 600 retained EUR frames with the selected raw
backing altered; the first table is read, while no new root read was observed.
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
each frame. Earlier save ancestry is unknown; no new cheat or state forcing was
used. Windows production build `seb61478` and Wine CLI `if6d22yl` create, reuse
and verify 129 matching files. Static reconciliation `u6ti5mxt` attributes
eight of the 11 previously unowned sampled bytes, leaving three high bytes.
Other readers, index domains, scenes, real-BIOS, UI save/Load, Linux, USA and
native Windows remain open. Wine servers drained with the local Bluetooth
overlay; no performance comparison was performed.

## E227 indexed halfwords at 39a5c8

EUR E218-d217 publishes `gameplay.indexed-halfwords-39a5c8`, three authenticated
little-endian halfwords at `39a5c8..39a5ce`. The source values are decoded from
the authenticated ROM; the original root word at `0e21f4` remains owned by
`engine.thumb-pc-literals`. All 128 earlier resources and files remain unchanged.

Wine disassembly `e227-halfword-reader-disassembly-iptx2__f` shows the original
path initializes r4 to zero and may set it to one or two before the indexed
halfword load at `080e21e4`. The runtime projects six physical bytes without
adding an index guard: later addresses follow prior ownership or raw fallback.
The local selection is not a claim about all callers or the full guest ABI.

Windows test build `zpk3zrya` and Wine reader `ebfw8yn_` verify all six source
bytes, the previous root, all 128 prior resources, coherent edit rejection,
mixed/mirrored boundaries, and ten interpreted/generated original reader
prefixes for indices 0, 1, 2, 3 and 255. DMA/CpuSet/CpuFastSet, state and
clock, and dataset/snapshot owner retention also pass. Wine battle `jv5_43gi`
compares 600 retained EUR frames with the six raw backing bytes altered. The
first halfword is read; state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame. Earlier save ancestry is unknown; no
new cheat or state forcing was used.

Windows production build `zd6ouzoi` and Wine CLI `w3f33_xp` create, reuse and
verify 130 matching files. Static reconciliation `l5w20wvc` attributes two of
the three previously unowned sampled bytes, leaving one high byte. Other
readers, selector paths, scenes, real-BIOS, UI save/Load, Linux, USA and native
Windows remain open. Wine servers drained with the local Bluetooth overlay;
no performance comparison was performed.

## E228 script byte lookup window

EUR E219-d218 publishes `engine.script-byte-lookup-3ad16e`: the six physical
bytes between the existing script-command directory ending at `3ad16e` and
the selector directory beginning at `3ad174`, plus the four-byte second root
at `126468`. The first root at `1263c4` remains owned by
`engine.thumb-pc-literals`. All 129 earlier resources and files remain
unchanged. This bounded physical window does not assert a complete semantic
lookup domain or classify the trailing bytes as valid selectors.

Original readers at `081263b2..081263ba` and `08126454..0812645c` subtract
one from a guest byte and read at the resulting physical address. The runtime
adds no selector guard: zero reaches the prior command directory, and later
addresses compose with the existing selector directory or raw fallback. Wine
source-site capture `qyrfym7w` observed a byte in this window in the retained
battle; original windows `rit3fxua` identify both readers and roots.

Windows test build `dd1n8nny` and Wine reader `cu0__ocg` verify all ten source
bytes, 129 prior resources, both adjacent owners, the prior root, coherent edit
rejection and mixed/mirrored boundaries. Forty original interpreted/generated
prefixes cover both readers and selectors 0 through 8 and 255, including
underflow and outside-window behavior. DMA/CpuSet/CpuFastSet, state and clock,
and dataset/snapshot owner retention also pass. Wine battle `frcsvz7a` compares
600 retained EUR frames with the selected source backing altered; the observed
lookup byte is consumed and state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame. Earlier save ancestry is unknown; no
new cheat or state forcing was used.

Windows production build `wmqqy5ac` and Wine CLI `a2w8hrj6` create, reuse and
verify 131 matching files. Static reconciliation `3blfucoi` attributes the last
previously unowned sampled byte. This is an address-only inventory, not proof
that other scenes, readers, source domains or the whole ROM are covered.
Real-BIOS, UI save/Load, Linux, USA and native Windows remain open. Wine servers
drained with the local Bluetooth overlay; no performance comparison was performed.

## E229 home-scene anchored Thumb literals

EUR E220-d219 extends `engine.thumb-pc-literals` with 25 aligned words
(100 bytes) sampled in the E207 home continuation. The old 886 words, their
source bindings and masks, and 129 other resource files remain unchanged.
Values are decoded as original word bits without inferring pointed-to assets
or replacing guest control flow.

Source plan `e229-home-literal-source-1j0fo_ra` authenticates the EUR ROM and
translation cache, each PC-relative Thumb LDR, and an at-most-96-byte
straight-line original path from an exported translated block to the reader.
These are static anchors: the home address inventory and original scopes do
not prove which instruction executed or establish a complete literal pool.
Source ranges are disjoint from previous owners.

Windows test build `ji4fe7wn` and Wine reader `yc40amn_` verify the 100 source
bytes, previous literals/masks and other resources, coherent edit rejection,
mixed/mirrored boundaries, 48 original interpreted/generated reader cases,
DMA/CpuSet/CpuFastSet and dataset/snapshot owner retention. Wine home scene
`29mql8p4` compares 600 recorded EUR frames with all 25 new backing words
altered; all 25 are consumed, and state, guest clock, video, OAM, dispatch and
five nonempty audio streams match each frame.

Windows production build `0fx28clc` and Wine CLI `__imn0tp` create, reuse and
verify 131 matching files. Static reconciliation `ypnkepk0` attributes 100
of the 272 previously unowned sampled bytes, leaving 172. This address-only
inventory does not establish global coverage; other readers, scenes, real-BIOS,
UI save/Load, Linux, USA and native Windows remain open. Wine servers drained
with the local Bluetooth overlay; no performance comparison was performed.

## E230 home-scene observed literals and switch destinations

EUR E221-d220 extends `engine.thumb-pc-literals` with 40 observed-scope words
(160 bytes) and `engine.thumb-switch-tables` with two destination arrays at
`09eff0..09f060` and `09f07c..09f098` (35 words, 140 bytes). Their two root
words are among the new literal sources. The earlier 911 literal words, 27
switch tables and 128 other resource files retain their prior contents.

Wine site capture `e230-home-residual-sites-zmsp5rfg` observes all 172 bytes
left by E220 in the 600-frame E207 home continuation. Forty words match original
Thumb PC-relative LDR opcodes, effective addresses and four-byte reads; three
other words match indexed loads. Original windows
`e230-indirect-reader-disassembly-5i77x__r` identify the two switch arrays and
their original comparisons: the first accepts indices 0..27 on this path; the
second subtracts one and compares against six. Runtime ownership projects exact
physical bytes without adding a guest selector guard or replacing guest control
flow. Execution scopes may include nested HLE/DMA accesses and do not prove
every caller or complete literal pools.

Windows test build `c0z9530s` and Wine reader `gwh_mblj` verify all 300 source
bytes, prior values/masks and other resources, coherent edit rejection, mixed
and mirrored boundaries, 80 interpreted/generated literal cases, eight guarded
switch cases, four outside-guard and four outside-load cases, transfers and
dataset/snapshot owner retention. The indirect MOV-PC uses interpreter fallback.
An initial Wine reader `zp_otq7h` failed because the expected hash for the
second table was mistyped; the hash was corrected before the passing build.

Wine home scene `18841zsj` compares 600 recorded EUR frames with all new literal
and switch backing bytes altered; all 40 words and both arrays are consumed,
and state, guest clock, video, OAM, dispatch and five nonempty audio streams
match each frame. Windows production build `ro1aypjc` and Wine CLI `q0do0j4t`
create, reuse and verify 131 matching files. Static reconciliation `dnr49xgd`
attributes the remaining 172 sampled bytes. The address-only inventory does not
prove full ROM coverage; other readers, scenes, real-BIOS, UI save/Load, Linux,
USA and native Windows remain open. Wine servers drained with the local
Bluetooth overlay; no performance comparison was performed.

## E233 Red Spring observed Thumb literals

EUR E222-d221 extends `engine.thumb-pc-literals` with 196 observed-scope
words (784 bytes). The previous 951 literal words and all 129 other resource
files retain their contents. The ROM digest, original Thumb LDR opcode and
effective source address are checked; source bytes are hashed and published
without inferring their meaning or imposing a guest bound.

Wine capture `e232-redspring-residual-sites-av1hj_7p` observes these words in
the retained 600-frame Red Spring continuation. Scopes may contain nested
HLE/DMA accesses and do not establish all direct callers. Windows test build
`e233-redspring-literals-build-lbz9wa2n` and Wine reader
`e233-redspring-literals-wine-4sf2ed90` verify all 784 bytes, previous literal
values and masks, the other 129 resources, coherent edit rejection, mixed and
mirrored boundaries, all 196 original interpreter and generated reader cases
(392 total), DMA/CpuSet/CpuFastSet and dataset/snapshot owner retention.

Wine scene `e233-redspring-literals-scene-wine-8ze5sj_t` compares 600 recorded
frames with all 196 new backing words altered. Every new word is consumed;
state, guest clock, video, OAM, dispatch and five nonempty audio streams match
each frame. Windows production build `e233-redspring-literals-cli-build-5kb4mqhv`
and Wine CLI `e233-redspring-literals-cli-wine-wdd7lpip` create, reuse and
verify 131 matching files. Static reconciliation `e233-redspring-residual-pz3zxelt`
reduces the E172 address-sample residual from 952 bytes to 168 (42 words).

The address-only sample does not prove complete source extents or full-ROM
coverage. The other 42 words were unowned at E233 and are served by later
E223–E226 increments. Other scenes/readers, real-BIOS, UI save/Load, Linux,
USA and native Windows remain open. Wine servers drained with the local
Bluetooth overlay; no performance comparison was performed.

## E234 Red Spring indirect switch source plan

Wine disassembly `e234-redspring-indirect-reader-disassembly-3ubkkgml`
inspects ten original Thumb readers scoped by the E232 Red Spring capture.
Each loads a word from an indexed destination array and moves it to PC; the
original compare and branch guard the represented path. Static plan
`e234-redspring-switch-source-plan-rtqw8txm` authenticates ten distinct roots,
148 destinations and 592 source bytes against the exact EUR ROM and E222
manifest. Every root is already owned by the Thumb literal resource; none of
the destination spans overlaps a declared E222 owner. The plan fingerprints
the relevant code and source bytes and confirms same-state destination form.

Sixteen words (64 bytes) in these tables overlap the 168 unowned E172 sample
bytes. This is a source plan: none of the 592 destination bytes is newly
published here. Scopes may include nested HLE/DMA accesses, and the original
guards describe these particular paths, not a global guest selector bound.
Consumption, other uses, the remaining 104 sampled bytes, Linux, USA and
native Windows remain open. Wine drained with the local Bluetooth overlay;
no performance comparison was made.

## E235 Red Spring switch destinations

EUR E223-d222 extends `engine.thumb-switch-tables` with the ten E234 guarded
destination arrays: 148 words and 592 source bytes. Their root words remain
owned by `engine.thumb-pc-literals`; 29 previous switch tables and all 129
other resource files retain their contents. The original code and source
fingerprints, root pointers and target form are validated. Runtime ownership
projects the physical array bytes without adding a guest selector guard or
replacing the original branch.

Windows test build `e235-redspring-switches-build-244xn28h` and Wine reader
`e235-redspring-switches-wine-wd1z4xka` verify all destination bytes, prior
table values/masks and other resources, coherent edit rejection, mixed and
mirrored boundaries, 40 original guarded interpreter/generated cases, 20
outside-guard and 20 outside-load cases, DMA/CpuSet/CpuFastSet and dataset/
snapshot owner retention. Indirect MOV-PC uses interpreter fallback in these
controlled checks.

Wine scene `e235-redspring-switches-scene-wine-hg3vf9nv` compares 600 recorded
Red Spring frames with all ten new backing arrays altered. Each array is
consumed, and state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame; the test does not claim that every destination is
used during normal play. Windows production build
`e235-redspring-switches-cli-build-dqkl2h_0` and Wine CLI
`e235-redspring-switches-cli-wine-fzyln36o` create, reuse and verify 131
matching files. Static reconciliation `e235-redspring-residual-c_2f3g_b`
reduces the E172 address-sample residual from 168 to 104 bytes.

The address-only sample and instruction scopes do not prove every caller,
selector, scene or full-ROM coverage. The other 26 sampled words were unowned at
E235 and are served by later E224–E226 increments. Real-BIOS, UI save/Load,
Linux, USA and native Windows remain open. Wine servers drained with the local
Bluetooth overlay; no performance comparison was performed.

## E236 Red Spring unscoped literal anchors

Static unit `e236-redspring-literal-anchors-rlq73_nk` checks the 104 E172
sampled bytes still outside the E223 manifest against the exact EUR ROM and
the retained translated-code block roots. It finds 22 complete literal words
(88 bytes) whose original Thumb PC-relative LDRs have straight-line paths
from exported native block roots at most 96 bytes earlier. Their source and
path hashes are recorded, and their bytes are outside existing declared
source ranges. Four other sampled words have no eligible match in this scan.

These anchors are static: E232 observed the addresses outside an instruction
scope, so the scan does not prove the E172 executing PCs, complete literal
pools or all callers. This unit publishes no new owner. E237 subsequently serves
the 22 anchored words, and E239/E240 serve the other four sampled words. Other
scenes, Linux, USA and native Windows remain open; no performance comparison was
made.

## E237 Red Spring anchored Thumb literals

EUR E224-d223 extends `engine.thumb-pc-literals` with 22 statically anchored
words (88 bytes). The previous 1,147 literal words and all 129 other resource
files retain their contents. Extraction authenticates the exact ROM, original
LDR opcode, effective address and straight-line anchor path. Runtime ownership
serves the exact source bytes without adding a guest selector guard.

Source plan `e236-redspring-literal-anchors-rlq73_nk` uses exported native
block roots and original instruction paths; the E172 address capture did not
attribute an executing PC. Windows test build `e237-redspring-anchored-build-yw_b5x53`
and Wine reader `e237-redspring-anchored-wine-d0vf7iwd` verify all 88 source
bytes, previous values/masks and other resources, coherent edit rejection,
mixed and mirrored boundaries, 44 original interpreted/generated reader cases,
DMA/CpuSet/CpuFastSet and dataset/snapshot owner retention.

Wine scene `e237-redspring-anchored-scene-wine-ao5jx0xh` compares 600 recorded
Red Spring frames with all 22 new backing words altered. Every new word is
consumed; state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Windows production build
`e237-redspring-anchored-cli-build-lruh4cdc` and Wine CLI
`e237-redspring-anchored-cli-wine-dl95px1f` create, reuse and verify 131
matching files. Static reconciliation `e237-redspring-residual-55zvsm8r`
reduces the E172 address-sample residual from 104 to 16 bytes.

The static anchors do not prove executed PCs, complete literal pools or all
callers. The other four sampled words were unowned at E237 and are served by
E239/E240. Other scenes, real-BIOS, UI save/Load, Linux, USA and native Windows
remain open. Wine servers drained with the local Bluetooth overlay; no
performance comparison was performed.


## E238 Red Spring remaining-source plan

Wine disassembly `e238-redspring-source-disassembly-jqg8wh29` identifies two
original guarded switch readers at `080148c6` and `08015276`, with already-owned
root words at `148cc` and `1527c`. Their original unsigned comparisons describe
89 and 88 destination entries respectively; the target sampled words belong
to entry 82 of each. No guest selector limit is added by this observation.

Wine disassembly `e238-redspring-byte-disassembly-56f3ef_x` shows the original
reader at `08109630` loads an already-owned root at `109710`, sets a copy
length of eight bytes and calls the original copy helper with source
`14d84c..14d854` and a guest stack destination. Static source plan
`e238-redspring-source-plan-b39y_6rn` authenticates the exact EUR ROM, code,
roots, helper and all three currently unowned source spans against the E224
manifest. The two arrays contain 177 destinations (708 bytes); the copy source
is eight bytes. Together, their spans include all 16 still-unowned E172 sampled
bytes. No source values are embedded here.

The E172 capture was outside instruction scopes: the overlapping addresses do
not prove which PCs executed or rule out other callers. This is a source plan,
not extracted or consumed data, and the sample is not a complete ROM inventory.
At E238, publishing all three sources remained open. E239 and E240 subsequently
publish and validate them for the selected EUR/Wine continuation. Other paths,
Linux, USA and native Windows remain open. Wine servers drained with the Bluetooth
overlay; no performance comparison was made.


## E239 final Red Spring switch arrays

EUR E225-d224 extends `engine.thumb-switch-tables` with the two E238 guarded
destination arrays: 177 words and 708 source bytes. Their root words remain
owned by `engine.thumb-pc-literals`; 39 previous switch tables and all 129
other resource files retain their contents. The exact ROM, original reader
and source fingerprints, root pointers and destination forms are authenticated.
Runtime ownership projects these physical bytes without adding a selector guard
or changing the original branch.

Windows test build `e239-redspring-final-switch-build-80mq3upk` and Wine reader
`e239-redspring-final-switch-wine-2zdfj3c8` verify all destination bytes,
previous resources and masks, coherent edit rejection, mixed and mirrored
boundaries, eight original guarded interpreter/generated cases, four
outside-guard and four outside-load cases, DMA/CpuSet/CpuFastSet and dataset/
snapshot owner retention. Indirect MOV-PC uses interpreter fallback in these
controlled checks.

Wine scene `e239-redspring-final-switch-scene-wine-6vjw7n51` compares 600
recorded Red Spring frames with both new backing arrays altered. Both arrays
are consumed, and state, guest clock, video, OAM, dispatch and five nonempty
audio streams match each frame; this does not establish every selector.
Windows production build `e239-redspring-final-switch-cli-build-vsbf1rc3` and
Wine CLI `e239-redspring-final-switch-cli-wine-q_skv5h9` create, reuse and
verify 131 matching files. Static reconciliation `e239-redspring-residual-n1l57jp6`
reduces the E172 address-sample residual from 16 to eight bytes.

The address-only sample does not prove its executing PCs, other callers,
selectors, scenes or full-ROM coverage. At E239, the eight-byte copy source remained open; E240 subsequently publishes
it. Real-BIOS, UI save/Load, Linux, USA and native Windows remain open. Wine servers drained
with the local Bluetooth overlay; no performance comparison was made.


## E240 final Red Spring copy source

EUR E226-d225 extends `engine.copied-halfword-lookups` with the eight source
bytes at `14d84c..14d854`. The existing two tables and their two additional
roots retain their values and source order; the root at `109710` remains owned
by `engine.thumb-pc-literals`, and all 129 other resource files remain unchanged.
The exact EUR ROM, original copy reader, root, copy helper and source bytes are
authenticated. The runtime supplies physical source bytes without changing the
original copy, stack indexing or guest control flow.

Windows test build `e240-redspring-final-copy-build-nt28u8wp` and Wine reader
`e240-redspring-final-copy-wine-e2rslaz3` verify the new bytes and earlier
resources, coherent edit rejection, mixed and mirrored boundaries, the original
copy with interpreted/generated prefixes, DMA/CpuSet/CpuFastSet and dataset/
snapshot owner retention. The copy helper runs through the original guest path.

Wine scene `e240-redspring-final-copy-scene-wine-xwjchks1` compares 600 recorded
Red Spring frames with the new backing bytes altered. The new source is consumed,
and state, guest clock, video, OAM, dispatch and five nonempty audio streams
match each frame. Windows production build `e240-redspring-final-copy-cli-build-gjzrae6s`
and Wine CLI `e240-redspring-final-copy-cli-wine-_jk7dwh3` create, reuse and
verify 131 matching files. Static reconciliation `e240-redspring-residual-_z04mity`
finds no unowned bytes among the retained E172 address sample.

This is one sampled continuation, not a complete ROM inventory or proof of
executing PCs, all consumers, scenes or later accesses. Other scenes/readers,
real-BIOS, UI save/Load, Linux, USA and native Windows remain open. Wine servers
drained with the local Bluetooth overlay; no performance comparison was made.


## E241 post-indexed lower-source reconciliation

Static unit `e241-post-indexed-lower-reconcile-wxrezwcp` joins the 568 bytes
without prior witnesses in the retained 12-frame E170 post-indexed sample to the
E226 manifest. It protects the original inventory, prior reconciliation, exact
EUR ROM and current dataset, verifies all 107 intersecting declared source
spans against the ROM and performs no replay. E226 declares 460 of those
sampled bytes: 408 Thumb literal bytes, 20 switch-table bytes and the 32
display-control bytes published at E171. The remaining 108 bytes form 14
ranges and were observed by the bus reader.

These figures apply only to that previously unmatched subset. The other 500
bytes with prior witnesses and the high copied-IWRAM block are outside this
comparison. A declared source range or sampled address does not prove the
executing PC, complete callers, asset boundaries or global coverage. At E241, source
attribution and ownership for 108 bytes remained open. E243 serves 36 and E245
serves the remaining 72. Other scenes, Linux, USA and native Windows remain open;
no performance comparison was made.


## E242 post-indexed lower reader sites

Wine unit `e242-post-indexed-lower-sites-qpmxbv2e` follows the retained E170
post-indexed continuation for 12 frames using E226, filtering the 27 complete
words that contain the 108 bytes outside its declared source ranges. All 108
bytes are observed. Nine instruction-scoped read records match original Thumb
PC-relative LDR opcodes and effective source addresses for nine words (36
bytes). Nineteen distinct read records have no instruction scope; one source
word also has a scoped witness. The matching sites and source offsets are
recorded for a subsequent authenticated extraction; this capture publishes
no new source.

Execution scopes may include nested HLE or DMA reads. The capture does not
prove every caller, source extent or complete scene equivalence. The other 72
bytes were subsequently attributed to statically anchored readers and served
at E245. Other scenes, Linux, USA and native Windows remain open. Wine drained with the local Bluetooth overlay; no
performance comparison was made.


## E243 post-indexed observed Thumb literals

EUR E227-d226 extends `engine.thumb-pc-literals` with the nine E242 observed
words (36 source bytes). The previous 1,169 literal words and all 129 other
resource files retain their contents. Extraction authenticates the exact EUR
ROM, original Thumb LDR opcode and effective address. Runtime ownership serves
the exact bytes without changing guest instructions or control flow.

Windows test build `e243-post-indexed-literals-build-3nzwlkqc` and Wine reader
`e243-post-indexed-literals-wine-6gxwak7q` verify all nine words, prior resources
and masks, coherent edit rejection, mixed and mirrored boundaries, 18 original
interpreted/generated reader cases, DMA/CpuSet/CpuFastSet and dataset/snapshot
owner retention.

Wine scene `e243-post-indexed-literals-scene-wine-fbe_qfgm` compares 12 recorded
post-indexed frames with all nine backing words altered. Every new word is
consumed; state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Windows production build
`e243-post-indexed-literals-cli-build-058ktp2c` and Wine CLI
`e243-post-indexed-literals-cli-wine-uydq8tce` create, reuse and verify 131
matching files. Static reconciliation `e243-post-indexed-lower-residual-n2okilgf`
reduces the E170 previously unmatched sample residual from 108 to 72 bytes.

Instruction scopes can include nested HLE/DMA accesses and do not prove every
caller, source extent or complete ROM coverage. The other 72 sampled bytes were
served at E245. Other scenes, real-BIOS, UI save/Load, Linux, USA and native
Windows remain open. Wine servers drained with the Bluetooth overlay; no
performance comparison was made.


## E244 post-indexed unscoped literal anchors

Static unit `e244-post-indexed-literal-anchors-0uvhmsty` checks the 72 E170
sampled bytes outside the E227 manifest against the exact EUR ROM and retained
E170 translated-code block roots. It finds all 18 complete words have original
Thumb PC-relative LDRs with straight-line paths from exported native block roots
at most 96 bytes earlier. The scan records opcode, source and path hashes and
checks that the words do not overlap declared E227 sources.

The E242 reads were outside instruction scopes: static anchors do not prove
the executing PCs, complete literal pools or all callers. This unit publishes
no new owner. E245 subsequently extracts and serves these words; other scenes,
Linux, USA and native Windows remain open. No performance comparison was made.


## E245 post-indexed anchored Thumb literals

EUR E228-d227 extends `engine.thumb-pc-literals` with the 18 E244 statically
anchored words (72 source bytes). The previous 1,178 literal words and all 129
other resource files retain their contents. Extraction authenticates the exact
EUR ROM, original LDR opcode, effective address and straight-line anchor path.
Runtime ownership serves the exact bytes without changing guest instructions.

Windows test build `e245-post-indexed-anchored-build-lbnmn2rn` and Wine reader
`e245-post-indexed-anchored-wine-nssz0c5s` verify all 18 words, prior resources
and masks, coherent edit rejection, mixed and mirrored boundaries, 36 original
interpreted/generated reader cases, DMA/CpuSet/CpuFastSet and dataset/snapshot
owner retention.

Wine scene `e245-post-indexed-anchored-scene-wine-fc5v9tzl` compares 12 recorded
post-indexed frames with all 18 backing words altered. Every new word is
consumed; state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Windows production build
`e245-post-indexed-anchored-cli-build-f_2qgtvi` and Wine CLI
`e245-post-indexed-anchored-cli-wine-i52g_6ot` create, reuse and verify 131
matching files. Static reconciliation `e245-post-indexed-lower-residual-rhmbe_o9`
finds no unowned bytes in the 568-byte E170 lower subset that lacked previous
witnesses.

The E242 reads were unscoped; static anchors do not prove executing PCs, all
callers, literal pools or complete ROM coverage. The 500 bytes with earlier
witnesses and high copied-IWRAM block were outside this subset's reconciliation;
E246 subsequently reconciles the full E170 inventory. Other scenes, real-BIOS,
UI save/Load, Linux, USA and native Windows remain
open. Wine servers drained with the Bluetooth overlay; no performance
comparison was made.


## E246 full post-indexed inventory reconciliation

Static unit `e246-post-indexed-full-reconcile-aqz4qh3s` compares all 4,680
bytes outside E170 manifest sources in the retained 12-frame inventory with
E228. It protects the original evidence and exact EUR ROM and verifies all 259
intersecting source-span hashes. E228 declares 1,080 sampled bytes: all 1,068
lower bytes, including the 500 with prior witnesses, and 12 high hardware/helper
bytes. The remaining 3,600 sampled bytes fall inside the documented copied
IWRAM source block at `d968e8..d97728`. The 48-byte metadata alias inside that
block already has an owner; the remaining source block contains executable
support and is retained in ROM under the existing
[inventory contract](development/ROM_DATA_INVENTORY.md).

The address join does not classify each byte as an instruction or asset, prove
its executing PC, cover every reader or establish full-ROM coverage. Other
scenes, source domains, Linux, USA and native Windows remain open. No game was
replayed and no performance comparison was made.


## E248 title and opening inventory reconciliation

Static unit `e248-title-inventory-reconcile-d83mbwp2` joins three retained
EUR/Wine title and opening inventories to the E228 manifest and authenticates
the intersecting source spans against the exact ROM. All 62 bytes previously
unowned in the title-language sample now have declared owners: 58 in layered
OBJ animation data and four in IWRAM helper metadata. All eight previously
unowned bytes in each of the main-title and opening samples also have declared
owners. The samples overlap and together contain 70 distinct bytes.

Earlier Wine route `e155-title-cursor-languages-wine-yx3vvt5x` already exercises
the five cursor choices on the title selector with the relevant backing source
altered. The selector is documented in the title frontend shown at startup;
these observations do not rule out another access path elsewhere. The static
join performs no replay and does not prove current runtime dispatch, every
reader or complete scene/ROM coverage. Other scenes, Linux, USA and native
Windows remain open; no performance comparison was made.


## E249 historical F09 inventory reconciliation

Static unit `e249-f09-historical-reconcile-__2ozlaa` joins all 1,146 bytes
that lay outside E71 source ranges in the retained 12-frame EUR F09 bus and
selected-host inventory to the E228 manifest. It restores the archived report
under the validation lease, verifies its retained hash and exact EUR ROM, and
authenticates all 251 intersecting source spans. E228 declares every sampled
byte: 952 Thumb literal bytes, 84 switch-table bytes, 20 hardware/helper bytes,
80 OBJ-list bytes and ten task-parameter bytes. The restored report was
retired unchanged after the unit passed.

This address-only join performs no replay and does not prove current runtime
service, executing PCs, other readers, scenes or complete ROM coverage. Earlier
scoped source witnesses retain their original limits. Linux, USA and native
Windows remain pending; no performance comparison was made.


## E250 Bank observed Thumb literals

EUR E229-d228 extends `engine.thumb-pc-literals` with 77 source-authenticated
words (308 bytes) from the retained E76 Bank sample. Extraction validates the
exact EUR ROM, each original LDR opcode and its effective address; runtime
reads use the extracted values without changing guest instructions. The previous
1,196 literal words and all 129 other resource files retain their contents.
The captured scopes can include nested HLE or DMA reads and do not establish
every executing caller.

Windows test build `e250-bank-observed-build-ucbajerz` and Wine reader
`e250-bank-observed-wine-biu5a8ky` verify the new words, old resources and
masks, coherent edit rejection, mixed and mirrored boundaries, 154 original
interpreted/generated reader cases, transfers and snapshot ownership. Wine
scene `e250-bank-observed-scene-wine-fwu8_cee` compares 12 recorded Bank frames
starting at guest frame 3660 with all new backing words altered: all 77 are
consumed and state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Production build `e250-bank-observed-cli-build-d3g9inrv`
and Wine CLI `e250-bank-observed-cli-wine-g8lsy8kq` create, reuse and verify
131 matching files. Wine servers drained using the local Bluetooth overlay.

Static join `e250-bank-observed-residual-duj05bki` finds 308 of 320 previously
unowned E76 sample bytes now declared. Three four-byte words at `2abc4`,
`2bfb8` and `5b6ac` remained unowned at E229; E254 subsequently serves them.
This is an address-only sample, not complete ROM coverage. Other readers, scenes, real
BIOS, UI save/Load, Linux, USA and native Windows remain open. No performance
comparison was made.


## E251 Bank unscoped literal anchor scan

Static unit `e251-bank-literal-anchors-hfew8swv` examines the three E250
unowned words against the exact EUR ROM, the E229 manifest and the retained
Bank native translation. It finds no PC-relative Thumb LDR candidate anchored
to an exported translated block by a straight-line path of at most 96 bytes
without crossing a declared data source. The scan covers the first 2 MiB,
where a PC-relative reader for these offsets would lie.

This negative bounded scan does not exclude other instruction forms, indirect
reads, other control paths or a different execution site. It publishes no new
owner and does not attribute the sampled reads. The three words remained unowned
at E229; E254 subsequently serves them. Other scenes, Linux, USA and native
Windows remain open; no replay or performance comparison was performed.


## E252 Bank residual read sites

Protected unit `e252-bank-other-sites-rm1pwc1a` restores the archived E76
Bank site capture under lease, checks its retained digest and the exact EUR
ROM, and inspects the three E250 residual words. Each has one captured 32-bit
Bus read in the 12-frame sample. The captured Thumb instruction scopes are
`2abb4`, `2bfaa` and `5b642`, respectively, and each instruction decodes as
`LDR r0, [r0]` (`6800`). The capture is compatible with an indirect word
reader, but nested HLE/DMA scopes mean the scope alone does not establish
that instruction as the direct cause or determine the input register value.
The archived report was retired unchanged after PASS.

E253 traces the indexed address construction and E254 serves the arrays. Other
scenes and readers, Linux, USA and native Windows remain open; no replay or
performance comparison was performed.


## E253 Bank switch source plan

Wine disassembly `e253-bank-site-disassembly-wine-9uc90bfb` and protected
source plan `e253-bank-switch-source-plan-in0jqgky` trace the three E252
instruction scopes to indexed Thumb switch readers. Each loads an existing
Thumb-literal table root, scales a byte selector by four, reads a destination
and branches through the original `MOV pc, r0`. The first two original unsigned
comparisons admit five and eight entries; the third selector is loaded by
`LDRB` and can address 256 entries. No additional runtime selector guard is
inferred.

The roots at `2abbc`, `2bfb4` and `5b648` already belong to the literal
resource. Their 269 destination words cover 1,076 bytes, including the three
E250 residual words at indices 1, 0 and 24. The plan authenticates original
code and source hashes against the exact EUR ROM, checks all target addresses
and finds no E229 owner overlap for the destination arrays. Wine drained.

Captured scopes can include nested reads; this static plan does not prove all
callers, scenes or full-ROM coverage and publishes no owner. E254 subsequently
serves and validates these arrays in the sampled Bank scene. Other readers and
scenes, Linux, USA and native Windows remain open.
No performance comparison was made.

## E254 Bank switch destinations

EUR E230-d229 extends `engine.thumb-switch-tables` with the three E253
destination arrays: 269 words and 1,076 authenticated source bytes. Their
previously owned roots, all 41 earlier switch tables and the other 129 resource
files retain their contents. Original selector checks and indirect `MOV pc`
semantics remain unchanged; the 256-entry table covers the full byte selector.

Windows test build and Wine reader `e254-bank-switch-wine-retry-akk2y3mv` verify
the sources and previous resources, tamper rejection, boundaries and mirrors,
original interpreted and generated readers, indirect branch, transfers and snapshot ownership. Wine
scene `e254-bank-switch-scene-wine-hcng484s` compares 12 recorded Bank frames
with all three destination arrays altered in the backing ROM. All three are
read; state, guest clock, video, OAM, dispatch and five nonempty audio streams match each frame. Wine CLI
`e254-bank-switch-cli-wine-oxj8x5e8` creates, reuses and verifies 131 matching
files. Static join `e254-bank-sample-residual-re_5dymo` finds no unowned bytes
in the previously residual E76 Bank sample. Wine drained with the local Bluetooth overlay.

The first Wine reader attempt `e254-bank-switch-wine-2kzl5orn` failed because
a synthetic selector value of 256 cannot arise from the original `LDRB`, and
the generated native chain passed a test-only stop inside the helper path.
The corrected test retains the direct out-of-array read check; the failure
was in the test stop, with no runtime data mismatch observed. Other callers,
scenes, real-BIOS, UI save/Load, Linux, USA and native Windows remain open. This sample is not complete ROM coverage;
no performance comparison was made.


## E255 Blue Screw ROM read inventory

Wine unit `e255-blue-screw-audit-wine-jyhm2s_n` compares 600 frames of the
retained EUR Blue Screw confirmation route against E230-d229, from guest frame
7673 to 8273. The recorded state, input, translation, ROM, loader and dataset
were protected. Complete state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame; Wine drained with the local Bluetooth
overlay.

The address-only selected-reader audit records 287,028 distinct requests
covering 164,591 ROM bytes. Of those bytes, 24 were outside the E230 manifest,
all below `300000`; E259 subsequently serves them. These are observed
addresses, not proven data extents or executing PCs. The retained report is the input for separate source tracing.
Other scenes and readers, Linux, USA and native Windows remain open; no
performance comparison was made.


## E256 Blue Screw literal anchor scan

Protected static unit `e256-blue-screw-literal-anchors-0iwncua_` checks the
six E255 sampled words against the exact EUR ROM, E230 manifest and retained
translated-block roots. It finds no PC-relative Thumb LDR with a straight-line
root anchor within 96 bytes and without crossing a declared source. No new
owner is published; instruction scopes were not collected by E255.

Other reader forms and control paths remain possible. The six words were
unowned at E230; E259 subsequently serves them. Other scenes and readers, Linux, USA and native Windows remain open. No replay or
performance comparison was performed.


## E257 Blue Screw residual read sites

Wine unit `e257-blue-screw-sites-wine-m3aveg4f` follows the retained E255
Blue Screw continuation with a filtered source-site capture. All six sampled
words are observed in instruction scopes. Three have matching original Thumb
PC-relative LDR opcode/address witnesses; the other three share a Thumb
`LDR r0, [r0]` scope. This complements E256: the scoped readers need not have
a qualifying exported-block anchor in the static scan.

Scopes can include nested HLE/DMA accesses; the capture does not prove every
caller or a complete indirect table extent, and publishes no owner. E258
classifies the indirect source and E259 serves both groups. Other scenes, Linux, USA and native Windows remain open; no performance comparison
was made. Wine drained with the local Bluetooth overlay.


## E258 Blue Screw source plan

Wine disassembly `e258-blue-screw-disassembly-wine-__998l61` and protected
plan `e258-blue-screw-source-plan-jkaucm5o` authenticate three E257
PC-relative literals and an indirect 15-destination switch array. Its root
is one of the three literal words. The original reader subtracts one from its
selector, compares the result unsigned against 14, then reads the selected
word and uses `MOV pc, r0`; no additional guest index guard is inferred.

The three literals occupy 12 bytes, while the switch destinations occupy 60
nonoverlapping bytes. The plan accounts for all 24 E255 sampled bytes, checks
all destination shapes and validates the code/source hashes against the exact
EUR ROM and E230 manifest. The plan publishes no owner; E259 subsequently
serves the sources. Captured scopes may nest, and other callers, scenes, Linux, USA and native Windows remain open. Wine
drained with the local Bluetooth overlay; no performance comparison was made.


## E259 Blue Screw literal and switch sources

EUR E231-d230 serves the three E258 Thumb literals and the 15-entry switch
array through the existing decoded resources. The 72 authenticated source
bytes include all 24 E255 sampled residual bytes. The original subtraction,
unsigned comparison and indirect `MOV pc` stay unchanged. Earlier literal
and switch sources and all 128 other resource files retain their contents.

Wine reader unit `e259-blue-screw-readers-wine-t4cf550o` verifies both groups,
previous resources, coherent edit rejection, boundaries and mirrors, original interpreted/generated reader
cases, guarded and out-of-array behavior, transfers and snapshot ownership.
Wine scene `e259-blue-screw-scene-wine-dk9h50xm` compares 600 retained Blue
Screw frames with both new sources altered in the backing ROM: all three literals and the switch array
are read; state, clock, video, OAM, dispatch and five nonempty audio streams
match each frame. Wine CLI `e259-blue-screw-cli-wine-9fermhwm` creates, reuses
and verifies 131 matching files. Static join
`e259-blue-screw-sample-residual-uo47n9gw` finds no unowned E255 sampled bytes.
Wine drained with the local Bluetooth overlay.

Captured scopes may include nested reads; this scene does not establish all
callers, source domains or complete ROM coverage. Other scenes, real-BIOS, UI
save/Load, Linux, USA and native Windows remain open. No performance comparison
was made.


## E260 Soil Evidence ROM read inventory

Wine unit `e260-soil-evidence-audit-wine-tfe8gnqy` compares 600 frames of the
retained EUR Soil Evidence confirmation route against E231-d230, from guest
frame 8033 to 8633. The recorded state, input, translation, ROM, loader and
dataset were protected. Complete state, guest clock, video, OAM, dispatch and
five nonempty audio streams match each frame; Wine drained with the local
Bluetooth overlay.

The address-only selected-reader audit records 424,583 distinct requests
covering 241,192 ROM bytes. Of those bytes, 3,066 are outside the
E231 manifest: 772 below and 2,294 at or above `300000`. E262 serves 1,335
of these addresses and leaves 1,731 outside E232 declared sources. These are
observed addresses, not proven data extents or executing PCs. Other scenes and
readers, Linux, USA and native Windows remain open; no performance comparison
was made.


## E263 Soil Evidence sampled residual classification

Protected static unit `e263-soil-residual-classification-ehljhoa2` joins the
E260 address inventory to E232 without replay. The 1,731 bytes still outside
declared sources form 149 disjoint ranges: 772 bytes below `300000` and 959 at
or above it. Bus reads cover all 1,731; observed SpritePeek requests overlap
some of those bytes. The 193 fully sampled aligned lower words include 151
with at least one statically matching Thumb literal-load bit pattern. No
complete, fully sampled aligned LZ stream candidate was found in the upper
residual by this bounded scan.

These are address-based candidates, not proof of executing PCs, code, asset
identities, full source extents or ownership. The scan does not publish a
resource and does not establish full ROM coverage. Other readers, scenes,
Linux, USA and native Windows remain open; no performance comparison was made.


## E264 Soil Evidence lower source-reader sites

Wine unit `e264-soil-lower-sites-wine-bdta3t36` captures filtered reader sites
for the 193 E263 lower words on the retained E232 Soil Evidence continuation,
frames 8033–8633. All 193 words and 772 sampled bytes appear again. Of 203
recorded selected reads, 182 have an interpreter instruction scope and 21 are
outside one. Authenticated original Thumb opcode/address checks identify 135
distinct words with a matching PC-relative literal-load witness. The full
filtered source-site plan and capture remain in the unit's evidence; Wine
drained with the local Bluetooth overlay.

An instruction scope can contain nested HLE/DMA reads, and outside-scope
reads have no executing PC here. This capture publishes no owner, does not
prove every caller or full source extents, and does not repeat the E262
raw/managed scene equivalence proof. The 58 other words, upper residual,
other scenes and consumers, Linux, USA and native Windows remain open. No
performance comparison was made.


## E265 Soil Evidence literal source plan

Protected static unit `e265-soil-literal-source-plan-3f25m75o` authenticates
the 135 distinct E264 PC-relative Thumb literal words against the EUR ROM and
E232 manifest. It verifies 139 instruction witnesses and exact opcode, source
hash and address relationships. The selected 135 words cover 540 bytes, do not
overlap E232 declared sources or previous/candidate reader instructions, and
retain one clean witness each. The plan publishes no resource; E266 later
integrates these sources.

Interpreter scopes may contain nested accesses; these witnesses do not prove
every caller, complete source extents or all scenes. Other sampled words and
upper bytes, Linux, USA and native Windows remain open. No performance
comparison was made.


## E266 Soil Evidence observed literal sources

EUR E233-d232 extends the authenticated Thumb literal resource with the
135 E265 observed words, serving 540 source bytes through the runtime without
changing guest instructions. The previous 1,276 literal words and 129 other
resource files are preserved.

Wine reader `e266-soil-literals-readers-wine-itca7x6l` verifies all prior sources, coherent edit rejection,
mirrors and mixed boundaries, 270 original interpreted/generated reader cases,
DMA/CpuSet/CpuFastSet transfers and snapshot owner retention. Wine scene
`e266-soil-literals-scene-wine-8wxjny7m` compares 600 recorded Soil Evidence frames with all 135 literal
sources altered in the managed backing ROM: all words are consumed (31784
source bytes counted), and state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame. Wine CLI `e266-soil-literals-cli-wine-ptvqd89o` creates, reuses and
verifies 131 matching files. Static join `e266-soil-literal-sample-residual-u2vrc_4i` leaves 1,191 E260 sampled
bytes outside E233 declared sources. Wine drained with the local Bluetooth
overlay.

Captured scopes may contain nested reads; other callers, source domains,
scenes, real-BIOS and UI save/Load remain open. Linux, USA and native Windows
are unverified. Older revision-sensitive tests were updated but not rerun; no
performance comparison was made.


## E267 Soil Evidence remaining lower reader groups

Protected static unit `e267-soil-remaining-sites-classification-olse67o1`
joins the E264 site capture with E233 owners. All 58 remaining lower words
(232 bytes) appear in 64 filtered read rows: 43 within interpreter scope and
21 outside it. The largest group has 30 distinct words in the scope of Thumb
instruction `e3838` (`6800`); the next outside-scope width-four group covers
13 words. Source and available instruction hashes were rechecked against the
EUR ROM. The retained report lists all reader groups and sampled offsets.

Scopes may contain nested reads. Opcode bits and grouping alone do not prove
the effective addresses, table extents, consumers or ownership; outside-scope
reads have no PC here. No owner or new replay was published. Other bytes,
scenes, Linux, USA and native Windows remain open; no performance comparison
was made.


## E268 Soil Evidence e3838 indexed source plan

Wine disassembly `e268-soil-e3838-disassembly-wine-l8md6ii5` and protected
static plan `e268-soil-e3838-source-plan-dvwif_zm` authenticate the original
Thumb sequence at `e382e..e383c`. It reads a signed halfword from RAM, shifts
it by two and loads a word from base `14d580`; its root literal `e3854` is
already owned by the Thumb literal resource. The E267 scope witnesses 30
distinct unowned source words, 120 bytes, at sampled indices between 1 and 32.
The plan records their exact offsets and hashes and publishes no owner; E269
subsequently integrates these selected words.

No guard or complete index domain is established in this code window. A
runtime integration must preserve the original signed arithmetic and raw
fallback for every unselected address, including negative indices; it must
not infer a table bound from the sample. The instruction scope may contain
nested accesses. Writers, other consumers and scenes, Linux, USA and native
Windows remain open. No performance comparison was made.


## E269 Soil Evidence sparse indexed words

EUR E234-d233 adds the 30 E268 observed words (120 source bytes) to the
existing indexed-word resource under a separate Soil Evidence group. The
original signed index arithmetic and root stay in guest code. Only the
authenticated sampled words are intercepted; unselected addresses retain
the previous managed-or-raw behavior. The original 30 indexed records and
129 other resource files are preserved. The resource keeps its existing ID
and filename for compatibility.

Wine reader `e269-soil-e3838-readers-wine-dpssxyfq` verifies old and new sources, coherent edit rejection,
mirrors and mixed boundaries, 74 interpreted/generated cases covering each
selected word and seven unselected indices including negative values,
DMA/CpuSet/CpuFastSet transfers and snapshot owner retention. Wine scene
`e269-soil-e3838-scene-wine-t22ztsv4` compares 600 recorded Soil Evidence frames with all 30 selected
words altered in the managed backing ROM: all are consumed (120 source
bytes counted), and state, guest clock, video, OAM, dispatch and five nonempty
audio streams match each frame. Wine CLI `e269-soil-e3838-cli-wine-oon2tik4` creates, reuses and verifies
131 matching files. Static join `e269-soil-e3838-sample-residual-5aymku00` leaves 1,071 E260 sampled bytes
outside E234 declared sources. Wine drained with the local Bluetooth overlay.

The sample is not a guest bound; index writers, full domain, other readers,
scenes, real BIOS and UI save/Load remain open. Linux, USA and native Windows
are unverified. Older revision-sensitive tests were updated but not rerun; no
performance comparison was made.


## E270 Soil Evidence SpritePeek list candidates

Protected static unit `e270-soil-sprite-list-candidates-16di9jl9` compares
E260 selected reads with E234 sources. It finds 20 distinct count-shaped
SpritePeek reads with matching six-byte-per-object payload requests. Each
complete candidate range is sampled and outside E234 owners; together they
cover 586 bytes, with no candidate range overlap. Their exact offsets, counts
and source hashes are retained in the plan; E271 later integrates the selected
lists.

The address-only audit does not prove the temporal pairing, call site,
producer or complete asset domain. These are candidates, not published
resources. Validate typed list consumption and the original scene before
claiming ownership. Other bytes, scenes, Linux, USA and native Windows remain
open; no performance comparison was made.


## E271 Soil Evidence typed sprite lists

EUR E235-d234 adds the 20 E270 selected lists to the authenticated static OBJ
resource: 91 typed objects and 586 original source bytes. The original 204
lists and 129 other resource files are preserved. Guest bus reads and host
OBJ decoding consume the typed data; unselected sources retain prior behavior.

Wine reader `e271-soil-sprite-lists-readers-wine-gywh4sti` verifies prior sources, coherent edit rejection,
mirrors and mixed boundaries, three original interpreter/host publication
comparisons, transfers and snapshot owner retention. Wine scene `e271-soil-sprite-lists-scene-wine-b3g8olpn`
compares 600 recorded Soil Evidence frames with all 20 backing lists altered:
the host consumes all 20 (20 are also seen on the guest bus), and state,
guest clock, video, OAM, dispatch and five nonempty audio streams match each
frame. Wine CLI `e271-soil-sprite-lists-cli-wine-duz93oci` creates, reuses and verifies 131 matching files.
Static join `e271-soil-sprite-list-sample-residual-ki3vs5jv` leaves 485 E260 sampled bytes outside E235 declared
sources. Wine drained with the local Bluetooth overlay.

The scene establishes consumption on this continuation, not the full pointer
producer, archive domain, every reader or other scenes. Real BIOS, UI
save/Load, Linux, USA and native Windows remain open. Older revision-sensitive
tests were updated but not rerun; no performance comparison was made.


## E272 Soil Evidence E235 sampled residual

Protected static unit `e272-soil-e235-residual-classification-jae45q7h`
joins the E260 selected-reader inventory to E235 without replay. The 485
sampled bytes outside declared sources form 69 disjoint intervals: 112 bytes
below `300000` and 373 at or above it. All remaining selected requests are
Bus reads; no SpritePeek request remains in this address-only residual. Of
28 fully sampled aligned lower words, 13 have possible static Thumb literal
bit patterns. The bounded upper scan finds no complete fully sampled aligned
LZ stream candidate.

Two upper intervals `6c173c..6c1768` and `6c19b4..6c19e0` each contain 44
sampled bytes; proximity to E271 lists does not establish a format or producer.
Static bit patterns and read addresses do not prove executing PCs, source
extents, consumers or ownership. Other scenes, Linux, USA and native Windows
remain open; no performance comparison was made.


## E273 Soil Evidence list-adjacent layout

Protected static unit `e273-soil-list-adjacent-layout-rsm1ehul` authenticates
the two 44-byte E272 Bus intervals beside the E271 lists against the EUR ROM
and E235 manifest. In `6c173c..6c1768`, 20 halfwords added to base `6c1740`
match all 20 E271 list starts; the retained layout records the exact values
and hashes. The second interval `6c19b4..6c19e0` has no such base match under
this bounded scan.

The arithmetic relationship is compatible with the project's compact bank
layout but does not establish the executing reader, selector domain, producer
or ownership. The other interval's format remains unproved. No new replay or
owner was published. Other ranges and scenes, Linux, USA and native Windows
remain open; no performance comparison was made.


## E276 Soil Evidence a83c2 switch source plan

Wine disassembly `e276-soil-a83c2-disassembly-wine-ps_64nk8` and protected
static plan `e276-soil-a83c2-switch-plan-6jwzeppl` authenticate the EUR Thumb
dispatch at `0a83b6..0a83c6`. An unsigned comparison bounds its index to 0–14;
the PC-relative root at `0a83c8` is already owned by `engine.thumb-pc-literals`
and points to 15 even Thumb destinations at `0a83cc..0a8408` (60 bytes).
E267 observed reads of indices 0, 1 and 14 in the retained Soil Evidence
continuation. The destination range has no E236 owner; the root must not be
declared twice.

The sample does not establish all selector writers, caller paths or scenes;
interpreter scopes may contain nested reads. The plan publishes no runtime
owner. Linux, USA and native Windows remain open. No performance comparison
was made.


## E277 Soil Evidence a83c2 switch runtime

EUR E237-d236 adds the 15 E276 authenticated Thumb destinations (60 source
bytes) to the existing switch-table resource. The PC-relative root remains
owned by the Thumb literal resource and the original guest guard remains in
control. The previous 45 switch tables and 129 other resource files are
preserved. Unselected addresses retain the earlier managed or ROM behavior.

Wine reader `e277-soil-a83c2-switch-readers-wine-syjfuorl` verifies source binding, coherent edit rejection,
mirrors and mixed boundaries, the observed indices and outside-guard/fallback
cases through original interpreted and generated instructions, transfers and
snapshot ownership. The final indirect `MOV pc` uses interpreter fallback.
Wine scene `e277-soil-a83c2-switch-scene-wine-76pzfrqd` compares 600 recorded Soil Evidence frames with all 15
destinations altered: the switch resource is consumed and state, guest clock,
video, OAM, dispatch and five nonempty audio streams match each frame. Wine
CLI `e277-soil-a83c2-switch-cli-wine-bx2ywffb` creates, reuses and verifies 131 matching files. Static join
`e277-soil-a83c2-switch-sample-residual-ab5klphb` leaves 385 E260 sampled bytes outside E237 declared sources. Wine
drained with the local Bluetooth overlay.

This continuation does not establish all selector writers, callers or other
scenes; real BIOS, UI save/Load, Linux, USA and native Windows remain open.
Older revision-sensitive tests were updated but not rerun; no performance
comparison was made.


## E278 Soil Evidence remaining switch source plan

Wine disassembly `e278-soil-remaining-disassembly-wine-i0v9h91a` and protected
static plan `e278-soil-remaining-switch-plan-8059c_wi` authenticate nine
additional EUR Thumb dispatch windows scoped in E267. Their original unsigned
guards bound 120 destinations across nine tables (480 source bytes), with no
E237 destination-owner overlap. Each root is already owned by the Thumb
literal resource. E267 scoped reads cover ten destination words (40 bytes);
the rest of each guarded table is authenticated against the source, not
claimed as observed during the retained continuation.

The sample does not establish all selector writers, caller paths or scenes;
interpreter scopes may contain nested reads. This plan publishes no runtime
owner. Linux, USA and native Windows remain open. No performance comparison
was made.


## E279 Soil Evidence remaining switch runtime

EUR E238-d237 adds the nine E278 authenticated Thumb destination arrays: 120
destinations and 480 source bytes. Their PC-relative roots remain owned by the
Thumb literal resource and the original guest guards remain in control. The
previous 46 switch tables and 129 other resource files are preserved.
Unselected addresses retain the earlier managed or ROM behavior.

Wine reader `e279-soil-remaining-switches-readers-wine-tvvu9o31` verifies source binding, coherent edit rejection,
mirrors and mixed boundaries, observed indices and outside-guard/fallback cases
through original interpreted and generated instructions, transfers and
snapshot ownership. The final indirect `MOV pc` uses interpreter fallback.
Wine scene `e279-soil-remaining-switches-scene-wine-tgdwq7za` compares 600 recorded Soil Evidence frames with all nine
destination arrays altered: each selected switch is consumed and state, guest
clock, video, OAM, dispatch and five nonempty audio streams match each frame.
Wine CLI `e279-soil-remaining-switches-cli-wine-7oaysfzd` creates, reuses and verifies 131 matching files. Static join
`e279-soil-remaining-switches-sample-residual-vdq2xyeo` leaves 345 E260 sampled bytes outside E238 declared sources. Wine
drained with the local Bluetooth overlay.

This continuation does not establish all selector writers, callers or other
scenes; real BIOS, UI save/Load, Linux, USA and native Windows remain open.
Older revision-sensitive tests were updated but not rerun; no performance
comparison was made.


## E280 Soil Evidence 146xxx original literal sites

Wine capture `e280-soil-146-sites-wine-q_rr8tp1` resumes the retained EUR
Soil Evidence scene for 600 E238 frames while a test-only table disables
translated entries in `146000..147000`. All 13 E272 candidate words are
observed under interpreter instruction scopes; each original Thumb
PC-relative load and effective source address matches the earlier static
candidate. The capture retains original opcodes and source hashes. No
outside-scope read is reported for these words.

This is source attribution for one modified-dispatch continuation, not a
state/clock/video equivalence proof, a complete caller census or a runtime
owner. Interpreter scopes can contain nested accesses; opcode/address matches
limit that ambiguity. Other scenes, Linux, USA and native Windows remain open.
No performance comparison was made.


## E281 Soil Evidence 146xxx literal runtime

EUR E239-d238 extends the authenticated Thumb literal resource with 13 E280
observed words (52 source bytes). Static plan
`e281-soil-146-literal-plan-cpvihoiy` verifies the original opcodes, effective
addresses and absence of resource or reader-instruction overlaps. The 1,411
previous literal words and 129 other resource files are preserved.

Wine reader `e281-soil-146-literals-readers-wine-h8dj_z7q` verifies exact sources, coherent edit rejection,
mirrors and mixed boundaries, all 13 original interpreted and generated
readers, transfers and snapshot ownership. Wine scene `e281-soil-146-literals-scene-wine-fivwh4bx` compares 600
recorded Soil Evidence frames with all 13 backing words altered: all selected
words are consumed and state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame. Wine CLI `e281-soil-146-literals-cli-wine-f80mzg0p` creates, reuses and
verifies 131 matching files. Static join `e281-soil-146-literals-sample-residual-anz9pb_y` leaves 293 E260 sampled
bytes outside E239 declared sources. Wine drained with the local Bluetooth
overlay. The initial scene driver `jc5t3ros` stopped on a syntax error before
Wine launched; its logs were retained and the corrected retry passed.

The selected continuation does not establish every caller or scene; real
BIOS, UI save/Load, Linux, USA and native Windows remain open. Older
revision-sensitive tests were updated but not rerun; no performance comparison
was made.


## E282 Soil Evidence 14d6d8 native read contexts

Wine capture `e282-soil-14d6d8-sites-wine-7mza_q21` resumes 600 EUR E239
Soil Evidence frames with a test-only native read API observer filtered to
`14d6d8..14d6e0`. All eight sampled bytes appear as separate native byte reads;
the bus-site capture reports all eight outside interpreter scopes. The outer
translated dispatch is `08147dc1`; the exported guest PCs are `08147e0a` and
`08147e0c`. These are contexts only: native block chains and synchronization
can make the exported PC stale, so they do not identify an exact instruction.

The first Windows test build `m6dd0jue` failed on a test-only integer
conversion; build `3___wn__` passed after correction. Wine attempt `dxkh8rfg`
stopped because its driver omitted the TinyCC path; it drained and the
corrected capture passed. Both failures retain their diagnostics.

The capture does not establish source format, an exact reader instruction,
state/clock/video equivalence, a runtime owner, other scenes or complete
callers. Linux, USA and native Windows remain open. No performance comparison
was made.


## E283 Soil Evidence copied halfword source plan

Wine disassembly `e283-soil-14d6d8-disassembly-wine-p9uxyh0n` identifies a
generic byte-copy loop. Wine capture `e283-soil-14d6d8-reader-wine-kg1o5ej2`
executes a test-only interpreter window and attributes all eight bytes at
`14d6d8..14d6e0` to its `147e0c` byte load, with return address `080f0437`.
Caller disassembly `e283-soil-14d6d8-caller-disassembly-wine-84wwsn66` and
protected plan `e283-soil-14d6d8-halfword-plan-_nouuxnz` authenticate the
original eight-byte copy to the stack and subsequent halfword indexing. The
four-halfword source has no E239 owner; its pointer at `0f0558` is already
owned by the Thumb literal resource.

The copy size is established, but no guest index bound is inferred from the
four copied values. The modified-dispatch trace does not establish state
equivalence, complete callers, source domain or other scenes. The plan
publishes no owner. Linux, USA and native Windows remain open. No performance
comparison was made.


## E284 Soil Evidence copied halfword runtime

EUR E240-d239 extends `engine.copied-halfword-lookups` with the E283 observed
four-halfword table (eight source bytes). Its separately owned pointer remains
in the Thumb literal resource, now declared as a dependency. The previous
three tables and 129 other resource files are preserved. Guest copying and
indexing retain their original control flow, including out-of-range reads from
the existing stack rather than an invented four-entry guard.

Wine reader `e284-soil-copied-halfwords-readers-wine-r9a1m88c` verifies exact sources, coherent edit rejection,
mirrors and mixed boundaries, original interpreted and generated root/copy/
index instructions, outside-index behavior, transfers and snapshot owner
retention. Wine scene `e284-soil-copied-halfwords-scene-wine-6xd5_pj5` compares 600 recorded Soil Evidence frames
with all eight backing bytes altered: every selected byte is consumed and
state, guest clock, video, OAM, dispatch and five nonempty audio streams match
each frame. Wine CLI `e284-soil-copied-halfwords-cli-wine-af_b0kli` creates, reuses and verifies 131 matching files.
Static join `e284-soil-copied-halfwords-sample-residual-z7jjst5p` leaves 285 E260 sampled bytes outside E240 declared
sources, with no sampled bytes below `300000` remaining. Wine drained with
the local Bluetooth overlay. The first reader fixture `vi786noo` failed in
its generated-block dispatch loop; its logs were retained and the corrected
retry passed.

The selected continuation does not establish every caller or scene; real
BIOS, UI save/Load, Linux, USA and native Windows remain open. Older
revision-sensitive tests were updated but not rerun; no performance comparison
was made.


## E285 Soil Evidence upper residual reader sites

Wine capture `e285-soil-upper-sites-wine-wgddtcdt` observes all 285 E240
upper residual bytes during 600 retained EUR Soil Evidence frames. The 278
selected read rows include 238 inside interpreter scopes and 40 outside them.
The scoped groups cover 245 distinct sampled bytes; the outside-scope group
covers 40 other bytes in `ae71be..ae787c`. The retained plan records reader
widths, scoped PCs/opcodes, source offsets and hashes for classification.

Instruction scopes can contain nested HLE/DMA reads, while outside-scope reads
have no attributed PC. This is an address/site inventory, not proof of exact
readers, source formats, extents, state equivalence or ownership. Other
continuations, Linux, USA and native Windows remain open. No performance
comparison was made.


## E286 Soil Evidence twelve-byte record plan

Wine disassembly `e286-soil-record-readers-disassembly-wine-cve18bko` and
protected plan `e286-soil-12byte-record-plan-3otuwzz3` identify six E285
scoped EUR records with a 12-byte stride, 72 source bytes and 66 sampled
bytes. The original reader computes the stride from a guest byte index and
branches on the first field; four selected records take type 3, one type 2
and one type 4. The candidate source spans do not overlap E240 owners.

The plan records linked pointer candidates for later verification; it does
not establish all pointed-to formats, complete extents, guest index domains,
other readers/scenes or runtime ownership. Interpreter scopes may contain
nested reads. Linux, USA and native Windows remain open. No performance
comparison was made.


## E287 Soil Evidence type-3 source links

The protected source plan `e287-soil-type3-linked-source-plan-hukink1w` joins
the E285 Wine reader scopes and E286 reader disassembly to four selected
type-3 rows, four eight-byte descriptors and their four LZ wrappers. The
original reader follows the row's descriptor pointer, reads its wrapper
pointer, tests wrapper flags and passes the following payload to the LZ path.
All four compressed streams already belong to `graphics.observed-lz-block`;
one wrapper is already owned by `graphics.observed-tilemap-descriptor`.

The four rows, four descriptors and three previously unowned wrappers occupy
92 distinct source bytes, including 74 E285 sampled bytes. The shared wrapper
and the four compressed streams must remain owned once. This plan does not
install runtime ownership or establish the complete guest index domain, other
readers or scenes. Interpreter scopes may include nested reads. Linux, USA
and native Windows remain open; no performance comparison was made.


## E288 Soil Evidence type-3 transfer runtime

EUR E241-d240 extends the existing observed tilemap row and descriptor
resources with four E287 type-3 rows, four linked descriptors and three new
wrappers (92 source bytes). A fourth wrapper remains owned by the existing
descriptor; all four LZ streams remain owned by the existing LZ resource.
The previous fields and source spans are preserved, and no guest indexing
bound is introduced.

Wine reader `e288-soil-transfer-readers-wine-3f10sxyc` validates publication and reload, unchanged prior
fields and other resources, unique shared ownership, altered-source mirrors
and mixed boundaries, coherent-edit rejection and 32 original interpreter/
generated reader cases. Wine scene `e288-soil-transfer-scene-wine-ixp_q9mt` compares 600 retained Soil
Evidence frames while all 92 new bytes and the shared wrapper are altered:
all four rows/descriptors and three new wrappers are consumed, while state,
guest clock, video, OAM, dispatch and five nonempty audio streams match each
frame. Wine CLI `e288-soil-transfer-cli-wine-ztjvp9z5` creates, reuses and verifies 131 matching files.
The static join `e288-soil-transfer-sample-residual-tn85nxvm` leaves 211 E260 sampled bytes outside E241 declared
sources, with none below `300000`. The previous copied-halfword fixture passes
on Wine as `e288-soil-copied-regression-wine-5_4m68dh` after the revision-sensitive fixture updates. Wine drained
with the local Bluetooth overlay.

The first test build `_v5ux2rq` failed on a missing test `Bus` import; the
corrected build passed. A reader preflight initially referenced the wrong
predecessor unit and stopped before launching Wine; the corrected retry passed.
Other old revision-sensitive Wine fixtures were updated but not rerun. This
single continuation does not establish other types, callers, scenes or all
source domains. Real BIOS, UI save/Load, Linux, USA and native Windows remain
open. No performance comparison was made.


## E289 Soil Evidence type-2 and type-4 source plan

Wine disassembly `e289-soil-other-transfer-readers-disassembly-wine-9u2bzss8`
and `e289-soil-type4-nested-disassembly-wine-hluvg778`, together with the
protected plan `e289-soil-other-transfer-source-plan-3fd5j3nj`, identify
104 currently unowned source bytes in the E286 selected EUR rows. The type-2
path comprises a 12-byte row, an eight-byte list header, two eight-byte
descriptors and two source rectangles of 30 and eight bytes. The original
reader follows the two entries and calls the rectangle copy routine; the E285
scopes include both descriptors and each rectangle's width read.

The type-4 path comprises a 12-byte row, an eight-byte descriptor, a six-byte
mode-7 header and one observed four-byte compressed run. The original dispatch
selects the mode-7 reader, which passes the bytes after the header to a nested
reader. Together the candidate spans include 78 bytes from the 211-byte E241
residual sample. The two bytes at `af2716..af2718` remain unclassified; an
already owned tilemap starts at `af2718`, but a relationship to the type-4
source has not been established. No source ownership was installed by this
plan. Interpreter scopes can contain nested reads; complete guest index
domains, other consumers and scenes remain open. Linux, USA and native
Windows remain pending; no performance comparison was made.


## E290 Soil Evidence type-2 and type-4 runtime sources

EUR E242-d241 serves the E289 selected type-2 row, list header, two
descriptors and two rectangles, plus the selected type-4 row, descriptor,
mode-7 header and first witnessed compressed run. The 104 source bytes extend
the existing tilemap, descriptor and row resources while preserving prior
fields and bindings. The two bytes at `af2716..af2718` remain ROM-backed,
and the adjacent existing tilemap keeps its own source. No full guest-index
range or relationship across that gap is inferred.

Wine reader `e290-soil-types24-readers-wine-oan65r_2` checks extraction/reuse, previous resource fields,
coherent-edit rejection, poisoned mirrors and mixed boundaries, the unowned
gap, retained snapshot ownership and 36 original interpreted/generated
reader cases with equal complete state and guest clock. Wine scene `e290-soil-types24-scene-wine-lrcwewjg`
compares 600 retained Soil Evidence frames with all 104 backing bytes altered:
both rows and the selected list, descriptors, rectangles, header and run are
consumed, while state, clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Wine CLI `e290-soil-types24-cli-wine-okzybzbi` creates, reuses and verifies 131
matching files. Static join `e290-soil-types24-sample-residual-tmqi3aq8` leaves 133 E260 sampled bytes outside
E242 declared sources, none below `300000`. Wine drained with the local
Bluetooth overlay.

The first reader unit `vily3m21` failed because the generated fixture joined
two adjacent loads while its interpreter comparison stopped after one; the
fixture was aligned to the same explicit endpoint and the retry passed.
Older revision-sensitive fixtures were updated but not rerun. This selected
continuation does not establish the unclassified gap, other consumers, full
guest domains or other scenes. Real BIOS, UI save/Load, Linux, USA and native
Windows remain open. No performance comparison was made.


## E291 Soil Evidence indexed four-byte record plan

Wine disassembly `e291-soil-39e1-readers-disassembly-wine-enhec_9u` and
`e291-soil-39e1-init-disassembly-wine-7v1qng1c`, together with protected plan
`e291-soil-39e1-source-plan-v65i8w_f`, authenticate five four-byte range
records at `39e124..39e138` and eleven four-byte sequence records at
`39e138..39e164`. The original reader indexes the first table with a byte from
the second. The sequence reader compares its incremented state to 11, and an
independently referenced source begins at `39e164`. The existing Thumb
literal resource owns the pointer words used by these readers.

The 64 candidate bytes include 30 bytes in the E242 residual sample: all
fields of six sequence records and the first two bytes of three range records.
The plan installs no runtime owner. The compare does not establish a bound for
arbitrary guest indices or initialization on every caller; other consumers,
continuations and scenes remain open. Interpreter scopes may contain nested
reads. Linux, USA and native Windows remain pending; no performance
comparison was made.


## E292 Soil Evidence indexed scene record runtime

EUR E243-d242 serves the five E291 range records and eleven sequence records
through `engine.soil-scene-records`. The resource owns precisely 64 original
bytes and depends on the separately owned Thumb literal roots. Neighboring
compact OBJ records keep their previous owner; accesses outside these spans
retain the ordinary prior-owner or ROM fallback. No runtime limit on guest
indices is introduced.

Wine reader `e292-soil-scene-records-readers-wine-3ob0bs_v` verifies preparation and reuse, all 130 prior
resource definitions and files, poisoned mirrors and mixed boundaries,
rejection of a coherently edited file, retained snapshot ownership and 30
original interpreter/generated reader cases, including out-of-range indices.
Wine scene `e292-soil-scene-records-scene-wine-vg1kiwv5` compares 600 retained Soil Evidence frames with all
64 source bytes altered and verifies actual reads from both record groups.
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
each frame. Wine CLI `e292-soil-scene-records-cli-wine-xhag7oue` creates, reuses and verifies 132 matching files.
Static join `e292-soil-scene-records-sample-residual-xzombbo8` leaves 103 E260 sampled bytes outside E243 sources,
none below `300000`. Wine drained using the local Bluetooth overlay.

The first build `5_mtfneb` identified three budget-test calls missing the new
argument; these were corrected before the successful build. Other callers,
initial guest state, source domains, scenes, real BIOS and UI save/Load remain
open. Older revision-sensitive fixtures were updated but not rerun; Linux,
USA and native Windows remain pending. No performance comparison was made.


## E293 Soil Evidence indirect eight-byte record plan

Wine disassemblies `e293-soil-indirect-readers-disassembly-wine-u03116_g`,
`e293-soil-indirect-roots-disassembly-wine-8lucbl5h` and
`e293-soil-indirect-init-disassembly-wine-ezcwvq_7`, together with protected
plan `e293-soil-indirect-records-source-plan-tcv9a6zh`, authenticate five
eight-byte records at `39aaac`, `39aad4`, `39ab5c`, `39e214` and `39e21c`.
The reader uses an eight-byte stride, a halfword at offset zero, two bytes
at offsets two and three, and two halfwords at offsets four and six. The
40 candidate bytes include 35 of the E243 residual sample. The compact OBJ
source immediately before `39e214` and the existing Thumb literal owners
remain separate.

Five source-backed call sites pass pointers compatible with these records to
the original initializer; one can reach `39aad4` with a byte index of four.
Each of these call sites supplies a count of one. This static correspondence
does not establish the ancestry of the sampled reads, bound arbitrary guest
indices or prove that other callers use the same count. Nested reader scopes
are possible. The plan installs no runtime owner; other callers, neighboring
bytes, scenes, Linux, USA and native Windows remain open. No performance
comparison was made.


## E294 Soil Evidence indirect record runtime

EUR E244-d243 serves the five E293 indirect records through
`engine.soil-indirect-records`. The resource owns precisely 40 authenticated
bytes and depends on the separately owned Thumb literal roots. Neighboring
compact OBJ data retains its existing owner; accesses outside the five spans
retain prior-owner or ROM fallback. No runtime guest-pointer bound is added.

Wine reader `e294-soil-indirect-records-readers-wine-cp1uz_wj` verifies preparation and reuse, all 131 prior
resource definitions and files, poisoned mirrors and mixed boundaries,
rejection of a coherently edited file, retained snapshot ownership and 76
original interpreter/generated cases, including out-of-range reads and the
existing pointer literals. Wine scene `e294-soil-indirect-records-scene-wine-y9ussbh4` compares 600 retained Soil
Evidence frames with all 40 source bytes altered and verifies reads from all
five records. State, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Wine CLI `e294-soil-indirect-records-cli-wine-ahnerpan` creates, reuses and verifies 133
matching files. Static join `e294-soil-indirect-records-sample-residual-vrbl28sb` leaves 68 E260 sampled bytes outside
E244 sources, none below `300000`. Wine drained using the local Bluetooth
overlay.

The static pointer links do not establish actual ancestry of the sampled
reads or complete guest domains. Other callers, neighboring bytes, scenes,
real BIOS and UI save/Load remain open. Older revision-sensitive fixtures
were updated but not rerun; Linux, USA and native Windows remain pending.
No performance comparison was made.


## E295 Soil Evidence indexed byte runtime

EUR E245-d244 serves the six E285 witnessed bytes at `39e224..39e22a`
through `engine.soil-indexed-bytes`. Original code at `080f05c6..080f05ce`
loads the already-owned literal `f05fc`, adds a guest-provided byte index,
reads the byte and compares it to zero. The source begins immediately after
the E244 indirect record; the two resources keep separate ownership.
Unselected indices continue through prior-owner or ROM fallback. Six observed
bytes do not establish the complete original table or guest index domain.

Wine reader `e295-soil-indexed-bytes-readers-wine-eb4g2qel` verifies preparation and reuse, all 132 prior
resource definitions and files, poisoned mirrors and mixed boundaries,
rejection of a coherently edited file, retained snapshot ownership and 18
original interpreter/generated cases including indices 6, 7 and 255. Wine
scene `e295-soil-indexed-bytes-scene-wine-_fgc_4sp` compares 600 retained Soil Evidence frames with all six
source bytes altered and verifies reads at all six indices. State, guest
clock, video, OAM, dispatch and five nonempty audio streams match each frame.
Wine CLI `e295-soil-indexed-bytes-cli-wine-ttlz5pdo` creates, reuses and verifies 134 matching files. Static
join `e295-soil-indexed-bytes-sample-residual-hgpyj2zs` leaves 62 E260 sampled bytes outside E245 sources, none
below `300000`. Wine drained using the local Bluetooth overlay.

The previous indirect-record fixture's fallback probe was moved beyond the
new source. Other callers, indices, neighboring bytes, scenes, real BIOS and
UI save/Load remain open. Older revision-sensitive fixtures were updated but
not rerun; Linux, USA and native Windows remain pending. No performance
comparison was made.


## E298 Soil Evidence scoped indexed values plan

Wine disassembly `e298-soil-remaining-readers-disassembly-wine-0c4y0luh`,
earlier disassembly `e293-soil-indirect-roots-disassembly-wine-8lucbl5h`,
and protected plan `e298-soil-scoped-values-source-plan-bk3xxzcf` identify
six individually observed values at `39a498`, `39a828`, `39a878`, `39abf4`,
`39ac14` and `39aa84`. Their widths are respectively four, four, one, four,
two and one bytes; all 16 bytes are in the E246 residual sample. The
corresponding original readers reach them through existing Thumb literal
owners. The value at `39abf4` is a Thumb code pointer; its low bit must be
preserved, and this static observation does not establish invocation.

The plan authenticates only these physical entries and installs no runtime
owner. It does not establish the complete tables, guest indices, caller
ancestry or callback execution, and scoped reads may include nested accesses.
Other scenes, Linux, USA and native Windows remain open. No performance
comparison was made.


## E299 Soil Evidence scoped indexed values runtime

EUR E247-d246 serves the six E298 observed values through
`engine.soil-scoped-values`. The resource owns exactly the 16 authenticated
source bytes, with their original one-, two- or four-byte widths. The Thumb
mode bit of the code pointer is preserved. Existing Thumb literal roots keep
their owner; unselected indices and neighboring bytes use prior-owner or ROM
fallback. This does not establish complete tables or callback invocation.

Wine reader `e299-soil-scoped-values-readers-wine-4zlbm5q6` verifies preparation and reuse, all 134 prior
resource definitions and files, poisoned mirrors and mixed boundaries,
rejection of a coherently edited file, retained snapshot ownership and 36
original interpreter/generated load cases including unowned fallback and
previously owned pointer roots. Wine scene `e299-soil-scoped-values-scene-wine-qeu3cp12` compares 600 retained
Soil Evidence frames with all 16 backing bytes altered and verifies reads of
all six values. State, guest clock, video, OAM, dispatch and five nonempty
audio streams match each frame. Wine CLI `e299-soil-scoped-values-cli-wine-r4c7x_zi` creates, reuses and verifies
136 matching files. Static join `e299-soil-scoped-values-sample-residual-248p0qae` leaves 22 E260 sampled bytes outside
E247 sources, none below `300000`. Wine drained using the local Bluetooth
overlay.

The first reader `67sxyper` stopped after one interpreter instruction while
the generated block combined two adjacent loads. The fixture now compares at
one shared endpoint; its retry passed. Other indices, actual callback
invocation and ancestry, scenes, real BIOS and UI save/Load remain open. Older revision-sensitive fixtures were updated
but not rerun; Linux, USA and native Windows remain pending. No performance
comparison was made.

## E305 Firaga SpritePeek list candidates

Protected static unit `e305-firaga-sprite-list-candidates-u3pn58da` compares
E302 selected reads with E249 sources. It finds 44 count-shaped SpritePeek
reads with matching six-byte-per-object payload requests. All complete
candidate ranges were sampled outside E249 owners; together they contain 347
objects and cover 2,170 bytes, with no candidate range overlap. Exact offsets,
counts and source hashes are retained in the plan. It publishes no resource.

The address-only inventory does not prove temporal pairing, original call
sites, the pointer producer or the complete asset domain. Typed consumption and
the affected scene must be verified before claiming ownership. Other sampled
bytes, scenes, Linux, USA and native Windows remain open. No performance
comparison was made.

## E306 Firaga typed sprite lists

EUR E250-d249 adds the 44 E305 selected lists to the authenticated static OBJ
resource: 347 typed objects and 2,170 original source bytes. The previous 224
lists and 135 other resource files are preserved. Guest bus reads and host OBJ
decoding consume the typed data; unselected sources retain prior behavior.

Wine reader `e306-firaga-sprite-lists-readers-wine-zlctezch` verifies prior
sources, coherent-edit rejection, mirrors and mixed boundaries, three original
interpreter/host comparisons, transfers and snapshot owner retention. Wine
scene `e306-firaga-sprite-lists-scene-wine-pa5aamwj` compares 606 retained
Firaga frames with all 44 backing lists altered. All 44 are consumed by both
the guest and host; state, guest clock, video, OAM, dispatch and five nonempty
audio streams match every frame. The source save's earlier history is unknown.

Wine CLI `e306-firaga-sprite-lists-cli-wine-h5s9hkgs` creates, reuses and
verifies 137 matching files. Static join
`e306-firaga-sprite-list-sample-residual-qgpr0gm3` leaves 2,823 E302 sampled
bytes outside E250 declared sources. Wine drained with the local Bluetooth
overlay. The scene does not establish the complete pointer producer, all
consumers or full-ROM coverage. Other scenes, real BIOS, UI save/Load, older
revision-sensitive fixtures, Linux, USA and native Windows remain open. No
performance comparison was made.

## E319 Firaga observed Thumb literals

EUR E257-d256 adds seven PC-relative LDR operands at `eb9c0..eb9dc` to
`engine.thumb-pc-literals`. Protected source plan
`e319-firaga-eb9c0-literals-source-plan-i2d5wcm0` authenticates all seven
original instruction/source pairs and all 28 bytes against the E302 address
sample. Existing literal entries and their owners retain their prior ordering
and values. Guest reads outside these selected words retain normal fallback.

Wine reader `e319-firaga-eb9c0-literals-readers-wine-ej_mhq65` verifies creation and reuse, prior literal and other
resource preservation, coherent-edit rejection, poisoned mirrors and mixed
boundaries, snapshot ownership, 14 original
interpreter/generated reader cases and three DMA/CpuSet/CpuFastSet cases.
Wine scene `e319-firaga-eb9c0-literals-scene-wine-g9opsvws` compares 606 retained Firaga frames with all seven words
altered in managed backing ROM; the selected resource records
28 byte reads and mask 127, confirming use of
all seven words. State, guest clock, video, OAM, dispatch and five nonempty
audio streams match every frame. The source save's earlier history remains
unknown. Wine CLI `e319-firaga-eb9c0-literals-cli-wine-kq1iltos` creates, reuses and verifies 143 matching files.
The Wine server drained using the local Bluetooth overlay.

Static join `e319-firaga-eb9c0-literals-sample-residual-r8fvlc_h` finds 1,241 E302 sampled bytes outside E257 declared
sources. The sample is address-only, not full-ROM coverage or attribution of
every read to a caller. Other original callers, tables, scenes, real BIOS and
UI save/Load remain open. Forty-nine older revision-sensitive fixtures were
updated but not rerun; fixtures expecting an unchanged literal resource need
adaptation. Linux, USA and native Windows remain to be checked. No performance
comparison was made.

## E320 Firaga transfer program source plan

Protected plan `e320-firaga-39cb9c-source-plan-ilwy805q` identifies a
European transfer program selected through the original count and pointer
tables at `39cbda` and `39cbd4`. The selected program at `39cb9c` contains
four 12-byte records: one references a two-entry palette list and three
reference compressed graphics streams. Original Thumb disassembly traces the
caller and task callbacks to the record handlers, including the 12-byte stride
and the format-header lookup. The plan authenticates 121 currently unowned
source bytes; 98 occur in the address-only E302 sample. It does not publish
these sources or prove which original instruction caused each sampled read.

The three format headers lead to the streams at `af5660`, `af4998` and
`af5454`. Their original encoded and decoded hashes match the existing
`graphics.observed-lz-block` sources verified by E304 reader
`e304-firaga-lz-readers-wine-6taiimnv` and 606-frame Wine scene
`e304-firaga-lz-scene-wine-ugltru8s`. Future publication must reuse those
owners instead of duplicating the compressed streams. The two palette-list
entries point to `ae6db8` and `ae6db0`; their payload extents, overlap and
complete original consumer contract were not established by E320. E322 later
identifies the selected adjacent row prefixes; other rows and consumers remain
open.

Wine disassembly units `e320-firaga-39cb9c-consumer-wine-4gdp8uwp`,
`e320-firaga-39cb9c-callback-wine-9_vg1v5x`,
`e320-firaga-39cb9c-reader-wine-hzx7t6al`,
`e320-firaga-39cb9c-program-wine-y91zg36h` and
`e320-firaga-39cb9c-handlers-wine-j0o4fvrw` passed and drained the Wine
server. Typed publication and guest use, other selectors, producers and
scenes, Linux, USA and native Windows remain open. No performance comparison
was made.

## E321 Firaga transfer program runtime

EUR E258-d257 publishes typed controls for the selected E320 program: caller
literals, the selected pointer and count, four transfer records, the two-entry
palette-list wrapper, three LZ wrappers and their format headers. The original
three compressed streams remain owned by `graphics.observed-lz-block`; this
resource does not claim their bytes or the unresolved palette payloads. The
runtime composes earlier owners with the selected bytes and normal fallback.

Wine reader `e321-firaga-transfer-program-readers-wine-tfkrk5a_` confirms previous resources are unchanged, rejects a
coherent altered file and manifest, checks poisoned mirrors and mixed boundaries
including prior LZ owners, compares 36 original interpreter/generated reader
cases and exercises DMA, CpuSet and CpuFastSet with matching states and clocks.
It also preserves the active owner across missing-dataset and snapshot cases.

Wine scene `e321-firaga-transfer-program-scene-wine-sh2x442o` compares 606 retained Firaga frames with all 121 selected
backing bytes altered; every source group is consumed: [8, 4, 1, 184, 11, 16, 9, 54, 26, 5, 25, 11].
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
every frame. The source save's earlier history remains unknown. Wine CLI
`e321-firaga-transfer-program-cli-wine-hj1_inpx` creates, reuses and verifies 144 matching files. All Wine servers
drained with the local Bluetooth overlay.

Static join `e321-firaga-transfer-program-sample-residual-h_gw4p3y` leaves 1,143 E302 sampled addresses outside E258 sources.
The sample is address-only and neither attributes every access to an instruction
nor establishes full-ROM coverage. Palette payload extents, other selectors,
producers, consumers and scenes, real BIOS and UI save/Load remain open. Older
revision-sensitive fixtures need adaptation and rerun; Linux, USA and native
Windows remain to be checked. No performance comparison was made.

## E324 Firaga stack initialization source plan

Protected source plan `e324-firaga-14d480-source-plan-52tdqmk9` authenticates
five original Thumb LDR literals at `d8660..d8674` and their five adjacent
source blocks at `14d480..14d4ac`. Original routine `d8538` loads three pairs
of words, requests a four-byte copy through the previously validated
`147dc0` helper, and copies four more words to its stack. Wine disassembly
`e324-firaga-14d480-consumer-wine-i51e67_6` and the E283 copy-helper evidence
pass with Wine drained.

The source plan covers 64 previously unowned bytes, all present in the E302
address-only sample. It authenticates original source addresses and instruction
pairs but does not attribute each sampled access to a specific PC. Typed
publication, the meaning and domain of live selectors, other callers and scenes,
Linux, USA and native Windows remain open. No performance comparison was made.

## E325 Firaga stack initialization runtime

EUR E260-d259 publishes the E324 original LDR literals and stack initialization
fields as typed words: five roots, three word pairs, one copied word and four
final words. All 64 authenticated source bytes are supplied through the guest
bus with normal prior-owner and ROM fallback. The game's original stack and
live selectors retain their existing behavior.

Wine reader `e325-firaga-stack-initialization-readers-wine-g2cv2hbo` confirms 144 prior resources are unchanged, rejects a
coherent altered file and manifest, checks poisoned mirrors and mixed
boundaries, compares 26 original interpreter/generated reader cases and two
original copy-loop cases, and exercises DMA, CpuSet, CpuFastSet and snapshot
ownership with matching states and clocks.

Wine scene `e325-firaga-stack-initialization-scene-wine-w0ru_ree` compares 606 retained Firaga frames with all 64 selected
backing bytes altered; all six source groups are used: [20, 8, 8, 8, 4, 16].
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
every frame. The source save's earlier history remains unknown. Wine CLI
`e325-firaga-stack-initialization-cli-wine-_t8_0ngi` creates, reuses and verifies 146 matching files. All Wine servers
drained using the local Bluetooth overlay.

Static join `e325-firaga-stack-initialization-sample-residual-smedangh` leaves 1,065 E302 sampled addresses outside E260 sources.
The sample is address-only and does not establish full-ROM coverage or attribute
every access to an instruction. Live selector domain, other callers and scenes,
real BIOS and UI save/Load remain open. Older revision-sensitive fixtures need
adaptation and rerun; Linux, USA and native Windows remain to be checked. No
performance comparison was made.

## E326 Firaga queued halfword-transfer source plan

Protected source plan `e326-firaga-40e844-source-plan-yuq_1w_3` identifies one
EUR transfer of 128 halfwords from `40e844..40e944`, plus the original source
base and destination literals at `ac9a0` and `ac9a8`: 264 unowned source bytes,
all present in the E302 address-only sample. The selected original producer at
`ac94e..ac95a` forms the source from base `0840e344` and an index shifted by
8, requests 128 halfwords and calls the existing queue helper. The observed
index is 5. No image format, full allocation extent or other index domain is
inferred from this selected transfer.

Wine capture `e326-firaga-40e844-dma-wine-d2e7v3m2` attributes all 128 target
requests to one immediate DMA3 transfer to `06013700`. A bounded follow-up
`e326-firaga-40e844-queue-wine-y_wimttr` joins a source-pointer store to the
same queue node and its later DMA use. Wine disassembly
`e326-firaga-enqueue-caller-wine-h4sr9ejw` authenticates the producer and
call. The native exported PC is a context, not an asserted exact store site;
the queue executor's own return address is not treated as producer ancestry.
All Wine servers drained.

The feature-gated diagnostic now scopes DMA provenance to actual source reads,
restores nested scopes and records unclassified target reads separately. A
synthetic immediate-DMA comparison in the Wine capture checks equal complete
state and guest clock with observation enabled. This source plan publishes no
resource. The source save's earlier history, other indices, consumers, scenes,
real BIOS, UI save/Load, Linux, USA and native Windows remain open. No
performance comparison was made.

## E327 Firaga queued halfword-transfer runtime

EUR E261-d260 publishes the E326 selected view as 128 typed halfwords and
two original producer literals: 264 authenticated source bytes. The original
producer, queue, DMA channel and live selector remain active. Other source
indices and neighboring bytes retain their prior owners or ROM fallback; the
selected transfer does not establish a graphics format or a full allocation.

Wine reader `e327-firaga-queued-transfer-readers-wine-eas1b41j` confirms 145 prior resources are unchanged, rejects a
coherent altered file and manifest, checks poisoned mirrors and mixed
boundaries, compares original interpreted/generated literal readers, and runs
the original producer prefix, queue helper and DMA executor. The selected
256-byte VRAM destination and complete guest state/clock agree. DMA, CpuSet,
CpuFastSet and snapshot ownership also pass.

Wine scene `e327-firaga-queued-transfer-scene-wine-9nspb3cr` compares 606 retained Firaga frames with all 264 selected
backing bytes altered; all three source groups are used: [4, 4, 256].
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
every frame. The source save's earlier history remains unknown. Wine CLI
`e327-firaga-queued-transfer-cli-wine-4jv_uinh` creates, reuses and verifies 147 matching files. All Wine servers
drained using the local Bluetooth overlay.

Static join `e327-firaga-queued-transfer-sample-residual-nrrlw5qt` leaves 801 E302 sampled addresses outside E261 sources.
The sample is address-only and does not establish full-ROM coverage. Other
indices, source allocation/format, producers, consumers and scenes, real BIOS
and UI save/Load remain open. Older revision-sensitive fixtures need adaptation
and rerun; Linux, USA and native Windows remain to be checked. No performance
comparison was made.

## E630 European map-height digit bank

The original reader `0801dd88` returns the map-height byte for the
chosen coordinate. The display code at `080ac8ee` divides that value
by ten and computes the remainder. Both copy calls take the same base
at `40e344` and scale the digit by 256 bytes. Each request copies 128
halfwords to OBJ VRAM. The tens output at `06013600` uses a fill when
the quotient is zero. The units output always uses `06013700`.

Ten distinct 256-byte glyphs cover `40e344..40ed44`. The separately
owned OBJ archive begins at `40ed44`. The E327 transfer keeps its
digit 5 interval and two original literal words. The E630 projection
adds the other nine glyphs and the first copy's destination literal.
The physical digit domain covers decimal inputs 0 through 99.

Wine `jnf3ke4h` verifies 133 original instruction halfwords and
2,572 projected bytes. The e367 ownership audit retains the 264
E327 bytes and identifies 2,308 unowned bytes in three spans.
All 15 input guards pass and Wine drains. Result SHA-256
`853b7df8f7d8e0b7c6e387ef2373531598a4af0e71756dd97595661d7fc03419`.
Runtime execution, the live map-height domain, ordinary scenes,
translated readers and deferred platforms are tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E631 European map-height digit runtime

Dataset `e368-d367` publishes the residual 2,308 E630 bytes in
`graphics.map-height-digits`, stored as `map-height-digits.bin`.
The file supplies nine glyphs and the first copy destination at their
original addresses. It depends on the E327 file for the digit 5 and
the two earlier literals. e367 still attaches, and all 168 earlier
resources are byte-identical.

Wine `v_1cj6om` supplies controlled map cells with heights 0 through 99
to the original getter and executes the original divider, queue
producers and DMA executor. The 190 glyph transfers and ten leading
fills match both OBJ VRAM destinations and their guards. The comparison
alters all 2,308 new bytes and the earlier 264 bytes in ROM backing.
The new file serves 43,880 observed bytes. Each earlier source
group is also read. Complete guest state and clock, 288 mixed and
mirrored reads and rejection of two invalid candidates pass.

Windows GNU build `xe1s240g` and Wine CLI creation, verification
and reuse pass. Stage `nox0vrxd` retains the 170-file generation.
All 17 input guards pass, and the server drains. Stage result
SHA-256 `38d3f22b3a61b895bfafe041d34aed26f6a0fc141798aa3444a043f583a3d566`; manifest SHA-256
`7ebda6beff990f2be25337cbed3e3e966c2f54e81affad78cdb8a7c4a9fef147`.
Live height bounds, ordinary scenes, translated execution, Linux, USA
and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E328 Firaga 2d/2e Thumb literal source plan

Protected plan `e328-firaga-2d-2e-literals-source-plan-csk4bbyl` authenticates
41 original PC-relative Thumb LDR instruction/source pairs in the `2d` and
`2e` regions. Their 164 distinct source bytes are all present in the E302
address-only sample and have no E261 owner. Wine disassembly
`e328-firaga-2d-2e-literals-disassembly-wine-edr1c64y` passed and drained the
Wine server. The plan publishes no resource and does not attribute individual
sampled accesses to a specific PC.

Typed publication and an altered-source scene were subsequently verified in
E329 below. Other readers, values, scenes, real BIOS, UI save/Load, Linux, USA
and native Windows remain open. No performance comparison was made.

## E329 Firaga 2d/2e Thumb literal runtime

EUR E262-d261 extends the existing `engine.thumb-pc-literals` resource with
the E328 41 authenticated source words (164 bytes). It preserves the previous
literal entries, other resources and fallback for addresses without an owner.
Original guest instructions remain active and no other reader or source bound
is inferred.

Wine reader `e329-firaga-2d-2e-literals-readers-wine-537ey132` confirms all prior entries and 145 other resources are
unchanged; it checks poisoned source bytes, mirrors and mixed boundaries,
rejects coherent data/hash edits and compares 82 original interpreted/generated
reader cases. DMA, CpuSet, CpuFastSet and snapshot owner retention pass.

Wine scene `e329-firaga-2d-2e-literals-scene-wine-ag5y0o6_` compares 606 retained Firaga frames while the selected
backing bytes are altered. All 41 words are consumed (mask
`0x1ffffffffff`); state, guest clock, video, OAM, dispatch and five
nonempty audio streams match at every frame. The source save's earlier history
is unknown. Wine CLI `e329-firaga-2d-2e-literals-cli-wine-stvyq0wt` creates, reuses and verifies 147 matching files.
All Wine servers drained using the local Bluetooth overlay.

Static join `e329-firaga-2d-2e-literals-sample-residual-yfp0eet3` leaves 637 E302 sampled addresses outside E262 sources.
This address-only sample does not identify every original reader or establish
full-ROM coverage. Other readers, values and scenes, real BIOS and UI save/Load
remain open. Older revision-sensitive fixtures need adaptation and rerun;
Linux, USA and native Windows remain to be checked. No performance comparison
was made.

## E330 Firaga eb83/ebc Thumb literal source plan

Protected plan `e330-firaga-eb83-ebc-literals-source-plan-iomybbqw`
authenticates ten original PC-relative Thumb LDR instruction/source pairs
against the European ROM and two retained disassemblies of the `eb830` and
`eba5c` functions. The 40 distinct source bytes all occur in the E302
address-only sample and have no E262 owner. Reconciliation confirms 637
sampled bytes remain outside E262; if these 40 bytes are subsequently owned,
597 would remain. The plan publishes no resource and does not attribute
individual sampled accesses to a specific PC.

Runtime publication and an altered-source scene were subsequently verified in
E331 below. Other readers, values, scenes, real BIOS, UI save/Load, Linux, USA
and native Windows remain open. No performance comparison was made.

## E331 Firaga eb83/ebc Thumb literal runtime

EUR E263-d262 extends the existing `engine.thumb-pc-literals` resource with
the E330 ten authenticated source words (40 bytes). It preserves the previous
literal entries, other resources and fallback for addresses without an owner.
Original guest instructions remain active and no other reader or source bound
is inferred.

Wine reader `e331-firaga-eb83-ebc-literals-readers-wine-u78tirug` confirms all prior entries and 145 other resources are
unchanged; it checks poisoned source bytes, mirrors and mixed boundaries,
rejects coherent data/hash edits and compares 20 original interpreted/generated
reader cases. DMA, CpuSet, CpuFastSet and snapshot owner retention pass.

Wine scene `e331-firaga-eb83-ebc-literals-scene-wine-uzod2dru` compares 606 retained Firaga frames while the selected
backing bytes are altered. All ten words are consumed (mask
`0x3ff`); state, guest clock, video, OAM, dispatch and five
nonempty audio streams match at every frame. The source save's earlier history
is unknown. Wine CLI `e331-firaga-eb83-ebc-literals-cli-wine-t7kh3_rm` creates, reuses and verifies 147 matching files.
All Wine servers drained using the local Bluetooth overlay.

Static join `e331-firaga-eb83-ebc-literals-sample-residual-s6x3bxvv` leaves 597 E302 sampled addresses outside E263 sources.
This address-only sample does not identify every original reader or establish
full-ROM coverage. Other readers, values and scenes, real BIOS and UI save/Load
remain open. Older revision-sensitive fixtures need adaptation and rerun;
Linux, USA and native Windows remain to be checked. No performance comparison
was made.

## E332 Firaga 013/019/0cf Thumb literal source plan

Protected plan `e332-firaga-code-literals-source-plan-loawcszx` authenticates
14 original PC-relative Thumb LDR instruction/source pairs against the
European ROM. Wine disassembly
`e332-firaga-code-literals-disassembly-wine-nbsow8qc` covers the selected
`013`, `019` and `0cf` windows and drained the Wine server. The code-candidate
map used for selection is speculative. The 56 distinct source bytes all
occur in the E302 address-only sample and have no E263 owner. Reconciliation
confirms 597 sampled bytes remain outside E263; if these 56 bytes are
subsequently owned, 541 would remain. The plan publishes no resource and does
not attribute individual sampled accesses to a specific PC.

Runtime publication and an altered-source scene were subsequently verified in
E333 below. Other readers, values, scenes, real BIOS, UI save/Load, Linux, USA
and native Windows remain open. No performance comparison was made.

## E333 Firaga 013/019/0cf Thumb literal runtime

EUR E264-d263 extends the existing `engine.thumb-pc-literals` resource with
the E332 14 authenticated source words (56 bytes). It preserves the previous
literal entries, other resources and fallback for addresses without an owner.
Original guest instructions remain active and no other reader or source bound
is inferred.

Wine reader `e333-firaga-code-literals-readers-wine-0j4qqizf` confirms all prior entries and 145 other resources are
unchanged; it checks poisoned source bytes, mirrors and mixed boundaries,
rejects coherent data/hash edits and compares 28 original interpreted/generated
reader cases. DMA, CpuSet, CpuFastSet and snapshot owner retention pass.

Wine scene `e333-firaga-code-literals-scene-wine-lcs3hcvx` compares 606 retained Firaga frames while the selected
backing bytes are altered. All 14 words are consumed (mask
`0x3fff`); state, guest clock, video, OAM, dispatch and five
nonempty audio streams match at every frame. The source save's earlier history
is unknown. Wine CLI `e333-firaga-code-literals-cli-wine-kijs396o` creates, reuses and verifies 147 matching files.
All Wine servers drained using the local Bluetooth overlay.

Static join `e333-firaga-code-literals-sample-residual-we6mxexq` leaves 541 E302 sampled addresses outside E264 sources.
This address-only sample does not identify every original reader or establish
full-ROM coverage. Other readers, values and scenes, real BIOS and UI save/Load
remain open. Older revision-sensitive fixtures need adaptation and rerun;
Linux, USA and native Windows remain to be checked. No performance comparison
was made.

## E334 Firaga mixed Thumb literal source plan

Protected plan `e334-firaga-mixed-literals-source-plan-t56gh8bq` authenticates
57 original PC-relative Thumb LDR instruction/source pairs against the
European ROM. Wine disassembly
`e334-firaga-mixed-literals-disassembly-wine-5rc48zzd` covers six selected
code windows and drained the Wine server. The 228 distinct source bytes all
occur in the E302 address-only sample and have no E264 owner. Reconciliation
confirms 541 sampled bytes remain outside E264; if these 228 bytes are
subsequently owned, 313 would remain. The plan publishes no resource. Linear
disassembly and the address-only sample do not prove that any individual PC
executed for a sampled access.

Runtime publication and an altered-source scene were subsequently verified in
E335 below. Other readers, values, scenes, real BIOS, UI save/Load, Linux, USA
and native Windows remain open. No performance comparison was made.

## E335 Firaga mixed Thumb literal runtime

EUR E265-d264 extends the existing `engine.thumb-pc-literals` resource with
the E334 57 authenticated source words (228 bytes). It preserves the previous
literal entries, other resources and fallback for addresses without an owner.
Original guest instructions remain active and no other reader or source bound
is inferred.

Wine reader `e335-firaga-mixed-literals-readers-wine-8ipblmws` confirms all prior entries and 145 other resources are
unchanged; it checks poisoned source bytes, mirrors and mixed boundaries,
rejects coherent data/hash edits and compares 114 original interpreted/generated
reader cases. DMA, CpuSet, CpuFastSet and snapshot owner retention pass.

Wine scene `e335-firaga-mixed-literals-scene-wine-ixvjw955` compares 606 retained Firaga frames while the selected
backing bytes are altered. All 57 words are consumed (mask
`0x1ffffffffffffff`); state, guest clock, video, OAM, dispatch and five
nonempty audio streams match at every frame. The source save's earlier history
is unknown. Wine CLI `e335-firaga-mixed-literals-cli-wine-7jyq36ot` creates, reuses and verifies 147 matching files.
All Wine servers drained using the local Bluetooth overlay.

Static join `e335-firaga-mixed-literals-sample-residual-77a2186_` leaves 313 E302 sampled addresses outside E265 sources.
This address-only sample does not identify every original reader or establish
full-ROM coverage. Other readers, values and scenes, real BIOS and UI save/Load
remain open. Older revision-sensitive fixtures need adaptation and rerun;
Linux, USA and native Windows remain to be checked. No performance comparison
was made.

## E336 Firaga residual Thumb literal source plan

Protected plan `e336-firaga-residual-literals-source-plan-6fg1_6yj` authenticates
52 original PC-relative Thumb LDR instruction/source pairs against the
European ROM. Wine disassembly
`e336-firaga-residual-literals-disassembly-wine-d9ijk6ak` covers 19 selected
code windows and drained the Wine server. The 208 distinct source bytes all
occur in the E302 address-only sample and have no E265 owner. Reconciliation
confirms 313 sampled bytes remain outside E265; if these 208 bytes are
subsequently owned, 105 would remain. The plan publishes no resource. Linear
disassembly and the address-only sample do not prove that any individual PC
executed for a sampled access.

Runtime publication and an altered-source scene were subsequently verified in
E337 below. Other readers, values, scenes, real BIOS, UI save/Load, Linux, USA
and native Windows remain open. No performance comparison was made.

## E337 Firaga residual Thumb literal runtime

EUR E266-d265 extends `engine.thumb-pc-literals` with the E336 52 authenticated
source words (208 bytes), preserving previous literals, other resources and
fallback for addresses without an owner. Original guest instructions remain
active; no other reader or source bound is inferred.

Wine reader `e337-firaga-residual-literals-readers-wine-tmmrcebi` confirms all prior literals and 145 other resources are
unchanged, rejects coherent data/hash edits, checks poisoned mirrors and mixed
boundaries, and compares 104 interpreted/generated original-reader cases. DMA,
CpuSet, CpuFastSet and snapshot owner retention pass.

Wine scene `e337-firaga-residual-literals-scene-wine-m1fas74s` compares 606 retained Firaga frames with the selected
backing bytes altered; all 52 words are consumed (mask `0xfffffffffffff`).
State, guest clock, video, OAM, dispatch and five nonempty audio streams match
at every frame. The source save's earlier history is unknown. Wine CLI `e337-firaga-residual-literals-cli-wine-qgtz2kbo`
creates, reuses and verifies 147 matching files. All Wine servers drained with
the local Bluetooth overlay.

The original build unit `e337-firaga-residual-literals-build-mve1jq1_` remains
FAIL after a broken pipe at the paused session's output; its command exited 0
and cleanup completed. Protected independent reconciliation
`e337-residual-build-evidence-reconciled-06cq110j` verified source/driver and
evidence hashes, executable hash and PE architecture; no rebuild was needed.

Static join `e337-firaga-residual-literals-sample-residual-29h_ckxv` leaves 105 E302 sampled addresses outside E266 sources.
The address-only sample does not establish every original reader or full-ROM
coverage. Other readers, values and scenes, real BIOS, UI save/Load, older
revision-sensitive fixtures, Linux, USA and native Windows remain open. No
performance comparison was made.

## E340 Firaga scoped index 2 source plan

Protected source plan `e340-firaga-scoped-index2-source-plan-b3kir03p`
reconciles the E302 address sample with E267 and the earlier authenticated
Soil scoped readers. The sample includes an unowned word at `39a820` and a
byte at `39a876`, corresponding to index 2 of bases `39a818` and `39a874`.
The original instructions at `080e61cc` and `080e61d6` can read these
entries, using the same guest byte index. The second read is conditional on
the word being nonzero; the authenticated index-2 word is nonzero. E267
already owns index 4 of each table.

The sample contains no executing PCs and does not establish that either
access used these readers, or that they occurred in one invocation. The
five-byte source plan publishes no runtime data and does not establish table
extents, all indices, callers or scenes. Runtime publication and selected Wine
validation were subsequently verified in E342 below. Linux, USA and native
Windows remain open. No performance comparison was made.

## E342 Firaga scoped index 2 runtime

EUR E268-d267 extends `engine.soil-scoped-values` with the E340 word and
byte at physical index 2, retaining the six earlier values and the previous
serialized representation. The guest index remains unrestricted; other
physical entries retain prior-owner or ROM fallback. The two added values
are validated as a pair and authenticated against the original ROM.

Wine reader `e342-firaga-scoped-index2-readers-wine-4cdn3xwg` verifies the earlier representation and all 145
other resources, rejects incomplete or forged data, checks altered mirrors and
mixed boundaries, and compares 12 original interpreted/generated load cases
for indices 2, 3 and 4. Wine scene `e342-firaga-scoped-index2-scene-wine-5o3bgt5b` compares 606 retained Firaga
frames with the five source bytes altered and consumes both new entries. Full
state, guest clock, video, OAM, dispatch and five nonempty audio streams match
at every frame. The source save's earlier history remains unknown. Wine CLI
`e342-firaga-scoped-index2-cli-wine-54wdcjp2` creates, reuses and verifies 147 matching files. Wine drained with the
local Bluetooth overlay.

Static join `e342-firaga-scoped-index2-sample-residual-81qecn8v` leaves 78 E302 sampled bytes outside E268 sources.
The address-only sample does not establish individual executing PCs, a common
invocation, all consumers or complete table extent. Other indices and scenes,
real BIOS, UI save/Load, older revision-sensitive Wine fixtures, Linux, USA
and native Windows remain open. No performance comparison was made.

## E344 Firaga palette switch source plan

Protected plan `e344-firaga-palette-switch-source-plan-o0cbwelm` authenticates
two EUR palette-command dispatches at roots `0d7b7c` and `0d7d70`, whose root
words already belong to `engine.thumb-pc-literals`. Original unsigned guards
select 12 and 14 word destinations at `0d7b80..0d7bb0` and `0d7d74..0d7dac`.
All 26 destination values are even and lie in `08000000..081fffff`; the 104
source bytes are disjoint from E268 sources. Twelve bytes appear in the E302
address-only sample, so publication would leave 66 sampled bytes outside its
sources. The plan does not publish data or attribute individual sample reads to PCs.

Sol consultation `/root/sol_e343_residual_diagnosis` identified the candidates
from the retained palette disassemblies; GPT-6.1 Sol/xhigh was requested, with
effective configuration unconfirmed. The protected plan independently checks
the EUR ROM, original instructions, roots, targets, owner ranges and sample.
Runtime publication and selected Wine validation were subsequently verified
in E345 below. Other consumers/scenes, Linux, USA and native Windows remain
open. No performance comparison was made.

## E345 Firaga palette switch runtime

EUR E269-d268 extends `engine.thumb-switch-tables` with the two E344
palette-command destination arrays: 26 words, 104 source bytes. Prior literal
roots and tables remain owned by their existing resources; original guest guards
and fallback remain active. The previous 32-table representation remains
parseable, and incomplete 33-table data is rejected.

Wine reader `e345-firaga-palette-switch-readers-wine-n12uhnkp` verifies the earlier tables and 145 other resources,
rejects forged data, checks altered mirrors and mixed boundaries, and compares
52 guarded interpreter/generated cases and eight out-of-range paths. The
indirect `MOV pc` uses interpreter fallback. Direct out-of-range loads, DMA,
CpuSet, CpuFastSet and snapshot owner retention also pass.

Wine scene `e345-firaga-palette-switch-scene-wine-19we6cc1` compares 606 retained Firaga frames with all 104 source
bytes altered; both new tables are consumed. State, guest clock, video, OAM,
dispatch and five nonempty audio streams match at each frame. The source save's
earlier history is unknown. Wine CLI `e345-firaga-palette-switch-cli-wine-lkro3br9` creates, reuses and verifies 147
matching files. Wine drained with the local Bluetooth overlay.

Static join `e345-firaga-palette-switch-sample-residual-no7zdoby` leaves 66 E302 sampled bytes outside E269 sources. The
address-only sample does not establish individual executing PCs, all readers,
scenes or full-ROM coverage. Other cases, real BIOS, UI save/Load, older
revision-sensitive Wine fixtures, Linux, USA and native Windows remain open.
No performance comparison was made.

## E346 Firaga short native residual context

Wine observer `e346-firaga-residual-native-context-wine-hle43zmn` reused the
E269 dataset and retained Firaga checkpoint for 12 frames. The observed and
unwrapped native paths matched complete state, guest clock, video, OAM,
dispatch and nonempty audio at each frame; Wine drained. Joining the observer
with the 66 addresses remaining in the E302 sample found no intersection.

This is a negative result only for the short selected native-API capture. It
does not exclude later reads, interpreter fallback, host/HLE or direct DMA
reads, other scenes or different source-save history. Native exported PCs may
be stale and outer dispatch can include chained blocks. A longer targeted
Wine capture or an independently authenticated source contract is required
before using those addresses to attribute readers. No performance comparison
was made.

## E347 Firaga signed halfword reader candidates

Protected pointer scan `e347-firaga-14cf-pointer-xrefs-zh94vzfa` joins the
E302 sample to E269 and finds 24 unowned bytes in two parallel halfword
clusters. Retained Wine disassembly
`e336-firaga-residual-literals-disassembly-wine-d9ijk6ak` shows original signed
halfword readers at `08007aba` and `08007ad0`, using base `14cf30` and, for
the latter, an additional `80` bytes. Each scales the low input byte by two.
The sampled pairs match physical indices 18, 23, 24, 25, 26 and 32 in both
views. The address-only E302 sample does not establish the executing PCs.

The full low-byte physical read domain was subsequently authenticated in E348
below; neither reader establishes a complete semantic table extent. Runtime
publication, other indices, readers, scenes, Linux, USA and native Windows
remain open. No performance comparison was made.

## E348 Firaga signed halfword physical projection plan

Protected plan `e348-firaga-14cf-projection-source-plan-bq_cnj9n` authenticates
the exact EUR instruction sequences and prior literal roots for the E347
readers. For each arbitrary low input byte, the first reads a signed halfword
from `14cf30 + 2 * index` and the second from `14cfb0 + 2 * index`. The two
256-entry physical views overlap by 384 bytes; their union is the 640-byte
interval `14cf30..14d1b0`, with no E269 owner. All 24 E347 sample bytes occur
in that union, so publication would leave 42 sampled bytes outside its sources.

This union describes addresses permitted by the instructions, not proven
reachable inputs or one semantic table. The E302 sample does not establish
executing PCs. Runtime publication and selected Wine validation were later
verified in E349 below; other consumers, scenes, Linux, USA and native Windows
remain open. No performance comparison was made.

## E349 Firaga signed halfword physical projection runtime

EUR E270-d269 adds the 640-byte E348 union as an explicit signed physical
projection in `engine.copied-halfword-lookups`. The two new readers access
this projection directly; no claim is made that it is copied by the guest.
Previous halfword groups and the separately owned literal roots are preserved,
and guest indices and fallback remain unchanged. The preceding representation
without the projection remains parseable.

Wine reader `e349-firaga-signed-halfwords-readers-wine-lh4xw9gy` verifies prior data and 145 other resources, rejects
invalid or forged content, checks altered mirrors and mixed boundaries, and
compares 84 original interpreted/generated reader cases including low-byte
aliases. DMA, CpuSet, CpuFastSet and snapshot owner retention also pass.
Wine scene `e349-firaga-signed-halfwords-scene-wine-0w4y_z_n` compares 606 retained Firaga frames with all 640 source
bytes altered; both physical regions are consumed. State, guest clock, video,
OAM, dispatch and five nonempty audio streams match at each frame. The source
save's earlier history is unknown. Wine CLI `e349-firaga-signed-halfwords-cli-wine-q2nr5p9n` creates, reuses and
verifies 147 matching files. Wine drained with the local Bluetooth overlay.

Static join `e349-firaga-signed-halfwords-sample-residual-io3xsjg4` leaves 42 E302 sampled bytes outside E270 sources. The
source union does not prove arbitrary input reachability, a semantic table
extent, individual sample PCs, all readers or full-ROM coverage. Other scenes,
real BIOS, UI save/Load, older revision-sensitive Wine fixtures, Linux, USA
and native Windows remain open. No performance comparison was made.

## E350 Firaga indexed field candidates

Protected pointer scan `e350-firaga-39a-39cc-pointer-xrefs-u0fvo5nk` examines
40 residual E302 bytes near `39a` and `39cc`. Retained Firaga constructor
disassemblies show five compatible index-2 physical fields: bytes at `39cc96`,
`39ccba` and `39ccd2`, a halfword at `39cca8`, and a word at `39ccfc`. Their
original reader instructions are `080eb93e`, `080ebc10`, `080eb962`,
`080eb8c0` and `080eb990`, respectively; existing literal roots point to the
corresponding bases. These nine bytes are in the address-only E302 sample,
which does not establish their executing PCs or a complete table extent.

The source contracts were subsequently authenticated in E351 below. Runtime
publication, the other residual addresses, indices, readers, scenes, Linux,
USA and native Windows remain open. No performance comparison was made.

## E351 Firaga indexed field source plan

Protected plan `e351-firaga-index2-fields-source-plan-tmv0x8by` authenticates
the exact EUR instruction sequences, five existing literal roots and five
index-2 physical fields identified in E350. The nine source bytes are disjoint
from E270 owners and all occur in the E302 address-only sample; publication
would leave 33 sampled bytes outside its sources. Widths and original guest
indexing are preserved. The source plan publishes no runtime data.

The sample lacks executing PCs and these physical fields do not establish
complete tables or guest index bounds. Runtime publication and selected Wine
validation were later verified in E352 below; other consumers/scenes, Linux,
USA and native Windows remain open. No performance comparison was made.

## E352 Firaga indexed field runtime

EUR E271-d270 extends `graphics.firaga-compact-cells` with the five E351
index-2 physical fields, totaling nine source bytes. Existing compact data,
the separately owned literal roots and original guest fallback are preserved.
The preceding representation without these fields remains parseable; the
resource does not impose a guest index bound.

Wine reader `e352-firaga-index2-fields-readers-wine-enxi8h4h` verifies prior data and 145 other resources, rejects
forged content, checks altered mirrors and mixed boundaries, and compares 40
original interpreted/generated field and root cases including adjacent
indices. DMA, CpuSet, CpuFastSet and snapshot owner retention also pass.
Wine scene `e352-firaga-index2-fields-scene-wine-jn68cocs` compares 606 retained Firaga frames with the nine source
bytes altered; all five fields are consumed. State, guest clock, video, OAM,
dispatch and five nonempty audio streams match at each frame. The source save's
earlier history is unknown. Wine CLI `e352-firaga-index2-fields-cli-wine-zbyfd8qq` creates, reuses and verifies 147
matching files. Wine drained with the local Bluetooth overlay.

Static join `e352-firaga-index2-fields-sample-residual-pxnn7sao` leaves 33 E302 sampled bytes outside E271 sources. The
address-only sample does not establish individual executing PCs, complete
table extents, all readers or full-ROM coverage. Other scenes, real BIOS, UI
save/Load, older revision-sensitive Wine fixtures, Linux, USA and native
Windows remain open. No performance comparison was made.

## E353 Firaga indirect index 2 source plan

Protected plan `e353-firaga-indirect-index2-source-plan-lojq8qbl` authenticates
the existing eight-byte-stride reader and its prior literal root at `0e4b84`.
Index 2 selects the record at `39aac4..39aacc`, disjoint from E271 owners.
Seven of the eight bytes occur in the E302 address-only sample, so publishing
the record would leave 26 sampled bytes outside its sources. The established
record layout preserves its halfword, byte and halfword fields; no guest
index bound is inferred. The source plan publishes no runtime data.

The sample does not establish executing PCs or pointer ancestry. Runtime
publication and selected Wine validation were later verified in E354 below;
other indices, callers, scenes, Linux, USA and native Windows remain open.
No performance comparison was made.

## E354 Firaga indirect index 2 runtime

EUR E272-d271 extends `engine.soil-indirect-records` with the authenticated
index-2 record at `39aac4..39aacc`. The previous five records, their literal
roots, serialized representation and original guest fallback are preserved;
no guest index bound is asserted.

Wine reader `e354-firaga-indirect-index2-readers-wine-b7cio2t3` verifies 20
original interpreter/generated cases, altered source mirrors and mixed
boundaries, authentication, the prior representation and 145 unchanged
resources, plus snapshot ownership. Wine scene
`e354-firaga-indirect-index2-scene-wine-lx36oq5k` consumes the new record
with its eight original bytes altered over 606 retained Firaga frames. State,
guest clock, video, OAM, dispatch and five nonempty audio streams match at
each frame. The source save's earlier history is unknown. Wine CLI
`e354-firaga-indirect-index2-cli-wine-_s7qf3ro` creates, reuses and verifies
147 matching files; Wine drained with the local Bluetooth overlay.

Static join `e354-firaga-indirect-index2-sample-residual-61erx_sh` leaves 26
E302 sampled bytes outside E272 sources. Seven of the eight new source bytes
occur in that address-only sample; this does not establish executing PCs,
pointer ancestry, all consumers or complete coverage. Other indices, callers,
scenes, real BIOS, UI save/Load, older revision-sensitive Wine fixtures,
Linux, USA and native Windows remain open. No performance comparison was made.

## E355 Firaga linked transfer candidates

Wine disassembly `e355-firaga-residual-readers-disassembly-wine-fvczyws1`
and the existing E342 scoped values identify a conditional one-row path:
`080e61c4..080e61de` loads the index-2 word at `39a820` and byte at
`39a876`, passing pointer `0839a758` and count one to `080db580`. The exact
EUR bytes at `39a758..39a764` match the existing type-3 transfer row layout
and point to an eight-byte descriptor at `39a678..39a680`. That descriptor
points to wrapper `b0dd58`, already owned by
`graphics.observed-tilemap-descriptor`; its compressed stream has a separate
existing owner. Prior E286/E287 evidence identifies the row stride and the
original type-3 row and descriptor readers. Neither candidate span is owned
in E272; 17 of their 20 bytes are in the address-only E302 residual.

The original count-one path is conditional on index 2 and unchanged list
state; it does not bound arbitrary callers or later modifications. The E302
sample does not establish executing PCs or prove this ancestry was used in
that continuation. The protected linked-source plan in E356 below authenticates the original
instructions, spans, dependencies, disjointness and intersection; runtime
publication and selected Wine validation were later verified in E357 below. Other scenes, Linux, USA and native Windows
remain open. Focused Sol `/root/sol_e343_residual_diagnosis` was requested
with xhigh; effective model and effort were not independently confirmed.
No performance comparison was made.

## E356 Firaga linked transfer source plan

Protected plan `e356-firaga-linked-transfer-source-plan-8or7qly_` verifies
the exact EUR index-2 pointer and count, original instruction fingerprints,
type-3 row `39a758..39a764`, descriptor `39a678..39a680`, and the links
to existing wrapper `b0dd58` and its 55-byte compressed stream at `b0dd5c`.
Both new spans are disjoint from E272 owners, while the pointer/count, wrapper
and stream retain their existing owners. Seventeen of the 20 source bytes
occur in the E302 address-only residual; publishing them would leave nine.

This plan publishes no runtime data; selected runtime publication and Wine
validation were later verified in E357 below. The one-row path is conditional
on index 2 and unchanged task state; the sample does not establish executing
PCs or prove that path. Other indices, callers, scenes, Linux, USA and native Windows
remain open. No performance comparison was made.

## E357 Firaga linked transfer runtime

EUR E273-d272 extends the existing tilemap row and descriptor resources with
the E356 12-byte type-3 row and eight-byte descriptor. The previously owned
wrapper and compressed stream remain uniquely owned; prior fields, serialized
representations, guest indexing and fallback are preserved.

Wine reader `e357-firaga-linked-transfer-readers-wine-be8l1_ah` verifies six
original interpreter/generated reader cases, the previous representations and
144 unchanged resources, shared ownership, forged-data rejection, altered
mirrors and mixed boundaries, and snapshot ownership. Wine scene
`e357-firaga-linked-transfer-scene-wine-izp2e4pq` consumes both new spans
with all 20 original bytes altered over 606 retained Firaga frames. State,
guest clock, video, OAM, dispatch and five nonempty audio streams match at
each frame. The source save's earlier history is unknown. Wine CLI
`e357-firaga-linked-transfer-cli-wine-6dug_n2c` creates, reuses and verifies
147 matching files; Wine drained with the local Bluetooth overlay.

Static join `e357-firaga-linked-transfer-sample-residual-ah1pd9hx` leaves
nine E302 sampled bytes outside E273 sources. The address-only sample does
not establish individual PCs or prove the conditional index-2 path was used.
Other indices, callers, scenes, real BIOS, UI save/Load, older revision-sensitive
Wine fixtures, Linux, USA and native Windows remain open. The first build
`8st6vdhe` was superseded after a test-only scene flag collision was separated;
corrected build `akrs7470` passes. No performance comparison was made.

## E358 Firaga caller-bounded palette projection plan

Protected plan `e358-firaga-palette-projection-source-plan-dtoi3884`
authenticates the EUR reader at `080138ec..0801390a`, its root literal
`01390c` and the selected caller at `08013918`. The caller accepts nonzero
values 1–4 for this path; combined with the reader's low two selector bits,
the physical halfword indices are 0–15 at `40fd70..40fd90`. The selected
caller writes the result to palette memory. This 32-byte interval is unowned
in E273, directly between two existing UI palette banks; its literal remains
owned by `engine.thumb-pc-literals`. Two bytes occur in the address-only
E302 residual, so publication would leave seven.

The bound applies only to this caller, not arbitrary calls or later changes.
The sample supplies no executing PCs and does not prove this path. Runtime
publication and selected Wine validation were later verified in E359 below;
other callers, scenes, Linux, USA and native Windows remain open. No performance comparison was made.

## E359 Firaga palette projection runtime

EUR E274-d273 extends `graphics.ui-raw-palettes` with the 32-byte E358
physical projection at `40fd70..40fd90`. The 21 previous banks, their source
bytes and the separately owned literal root remain unchanged. The original
reader and guest fallback remain active for values outside the selected span;
the caller-specific bound is not applied to other callers.

Wine reader `e359-firaga-palette-projection-readers-wine-4vc70dqr` verifies
32 original interpreter/generated cases across the 16 selected halfwords,
the prior representation and 145 unchanged resources, altered mirrors and
mixed boundaries, source authentication, an out-of-projection fallback and
snapshot ownership. Wine scene
`e359-firaga-palette-projection-scene-wine-mrf6k1gq` consumes the new source
with all 32 original bytes altered over 606 retained Firaga frames. State,
guest clock, video, OAM, dispatch and five nonempty audio streams match at each
frame. The source save's earlier history is unknown. Wine CLI
`e359-firaga-palette-projection-cli-wine-x3vrl3f5` creates, reuses and verifies
147 matching files; Wine drained with the local Bluetooth overlay.

Static join `e359-firaga-palette-projection-sample-residual-w9mdf6yf` leaves
seven E302 sampled bytes outside E274 sources. The address-only sample does
not establish individual PCs or prove the caller path was used. Other callers,
scenes, real BIOS, UI save/Load, older revision-sensitive Wine fixtures,
Linux, USA and native Windows remain open. No performance comparison was made.

## E360 Firaga CCEC indirect record source plan

Protected plan `e360-firaga-ccec-indirect-source-plan-zt7zwucy` authenticates
the EUR indexed reader at `080eb988..080eb9a0`, its separately owned literal
root and the E352 index-2 word `39ccfc`. That word points to the eight-byte
physical record `39ccec..39ccf4`; the selected caller passes it with count one
to the already authenticated indirect-record consumer at `080e389c`. The
record is unowned in E274, and seven of its eight bytes comprise the remaining
E302 address-only residual. Publication would leave no unowned bytes in that
one sample.

The pointer/count relationship is conditional; the sample does not establish
executing PCs or prove that ancestry. Runtime publication and selected Wine
validation were later verified in E361 below; other indices, callers, scenes
and full-ROM coverage remain open. Linux,
USA and native Windows remain open. No performance comparison was made.

## E361 Firaga CCEC indirect record runtime

EUR E275-d274 extends `engine.soil-indirect-records` with the E360 eight-byte
record at `39ccec..39ccf4`. The previous records and serialized representation
remain valid; the indexed pointer and literal root keep their existing owners.
Original guest indexing and fallback remain active.

Wine reader `e361-firaga-ccec-indirect-readers-wine-zcuhebx4` verifies 20
original interpreter/generated cases, including the separately owned indexed
pointer, previous representation and 145 unchanged resources, source
authentication, altered mirrors and mixed boundaries, fallback and snapshot
ownership. Wine scene `e361-firaga-ccec-indirect-scene-wine-_jvgs94x` consumes
the new record with all eight original bytes altered over 606 retained Firaga
frames. State, guest clock, video, OAM, dispatch and five nonempty audio
streams match at each frame. The source save's earlier history is unknown.
Wine CLI `e361-firaga-ccec-indirect-cli-wine-z4yf7_da` creates, reuses and
verifies 147 matching files; Wine drained with the local Bluetooth overlay.

Static join `e361-firaga-ccec-indirect-sample-residual-idrflo71` finds zero
E302 sampled bytes outside E275 declared sources. This is limited to that
one address-only continuation; it does not establish individual executing PCs,
prove the conditional pointer path, or establish full-ROM coverage. Other
indices, callers, scenes, real BIOS, UI save/Load, older revision-sensitive
Wine fixtures, Linux, USA and native Windows remain open. No performance
comparison was made.

## E362 White Flowers ROM read inventory

Wine `e362-white-flowers-audit-wine-tqqmw74v` inventories selected bus and
host readers during 700 recorded EUR White Flowers battle-entry frames with
E275-d274. It reuses the authenticated checkpoint and existing input. Raw and
managed machines match complete state, guest clock, video, OAM, dispatch and
five nonempty audio streams at every frame; Wine drained with the local
Bluetooth overlay.

Of 250,884 distinct observed physical ROM bytes, 8,039 are outside declared
E275 sources: 2,708 below `300000` and 5,331 at or above it. The retained
`audit.json` records address, width and selected reader but not executing PCs.
These are source leads, not proven data extents or a claim that every byte is
an asset; original readers and code/support exclusions require classification.
The selected continuation does not cover every reader or scene, and the source
save's earlier history is unknown. Linux, USA and native Windows remain open;
no performance comparison was made.

## E363 White Flowers residual grouping

Protected static unit `e363-white-flowers-residual-groups-fa24opzf` joins the
E362 address-only report to the exact EUR ROM and E275 manifest without replay.
All 8,039 sampled bytes outside declared sources occur in Bus reads, grouped
into 326 contiguous sampled intervals. Of these, 3,600 fall within the copied
IWRAM source block `d968e8..d97728` previously documented as executable support
in E246; its separately owned metadata remains unchanged. The other 4,439
sampled bytes still require classification. The largest include 1,270 bytes at
`43b36e..43b864` and 320 at `3ba0d8..3ba218`; ranges describe observations,
not established assets.

The bounded scan records 647 possible Thumb literal words; raw instruction
bit patterns may occur in data and do not establish execution or ownership.
The report does not establish individual PCs, extents, other scenes or full-ROM
coverage. Linux, USA and native Windows remain open; no performance comparison
was made.

## E364 White Flowers indexed archive candidate

Protected static scan `e364-white-flowers-high-xrefs-2v0ucvhi` records four
possible literal links around the three largest high-ROM E363 intervals. The
`3ba0d8..3ba218` halfword interval has candidate roots at `12bf7c/12bf80`;
bit-pattern matches alone do not establish executing instructions or a format.

A focused read-only diagnosis identifies a stronger lead for the
`43b36e..43b864` byte interval: the candidate indexed archive begins at
`43b36c`, and the original caller at `08089eb4..08089eca` selects a pointer
from `395560` using a live language byte and calls existing indexed decoder
`0800537c`. The first pointer addresses that archive. Preliminary parsing is
consistent with 33 fixed-size programs. E365 below authenticates the selected
pointer group, reconstruction, original code, ownership and sampled intersection
before publication. The sampled addresses
do not establish that this particular caller executed. Other languages,
callers, scenes, Linux, USA and native Windows remain open. Focused Sol
`/root/sol_e343_residual_diagnosis` was requested with xhigh; effective model
and effort were not independently confirmed. No performance comparison was
made.

## E365 White Flowers localized indexed archive source plan

Protected plan `e365-white-flowers-indexed-archive-source-plan-5d3a1lp3`
authenticates the EUR five-pointer group at `395560`, four caller literals at
`089fbc..089fcc`, original selector and decoder code, and five disjoint source
archives at `43b36c`, `44c99c`, `45e4c8`, `46f4f4`, and `480ffc`. Their
respective source lengths are 1,272, 1,281, 1,286, 1,271, and 1,285 bytes.
An independent static parser reconstructs each as 33 programs of 64 decoded
bytes. E366 below verifies production parsing and selected runtime consumption.

The selected 6,431 source bytes do not overlap E275 owners and include 1,274
E362 sampled bytes; publication would leave 6,765 sampled bytes outside
declared sources, including 3,600 previously documented copied-code support
bytes. The sample lacks executing PCs and does not prove caller ancestry.
Selection of five physical slots is conditional; arbitrary guest selector
values remain unrestricted. Other paths and scenes, Linux, USA and native
Windows remain open. No performance comparison was made.

## E366 White Flowers localized indexed archive runtime

EUR E276-d275 extends `graphics.indexed-archives` with the E365 five physical
archives, their pointer group and four caller literals. All 6,431 selected
source bytes are reconstructed from typed data; existing sources and the
previous serialized representation remain valid. The guest still evaluates
the live selector and reads outside the five selected slots normally.

Wine reader `e366-white-flowers-indexed-readers-wine-3d7s19no` verifies all
five production-parser reconstructions and 2,112-byte outputs, with 15
original interpreter/generated cases, 145 unchanged resources, altered
mirrors and mixed boundaries, the adjacent existing owner, unselected fallback,
source validation and snapshot ownership. Initial attempt
`e366-white-flowers-indexed-readers-wine-hh_l3ixb` failed because its test
incorrectly assumed slot 5 was unowned; the corrected test verifies that owner
and uses unowned slot 30. Wine drained in both runs.

Wine scene `e366-white-flowers-indexed-scene-wine-y_1th6uh` verifies 700
retained White Flowers frames with all selected backing bytes altered. The
first archive is consumed while full state, guest clock, video, OAM, dispatch
and five nonempty audio streams match every frame; other language scenes are
not established by this continuation. Wine CLI
`e366-white-flowers-indexed-cli-wine-j6kabfzv` creates, reuses and verifies
147 matching files with the local Bluetooth overlay and Wine server drain.

Static join `e366-white-flowers-indexed-sample-residual-djytd17l` leaves 6,765
E362 address-only sampled bytes outside declared sources: 3,600 are previously
documented copied-code support and 3,165 require classification. The sample
lacks executing PCs and does not establish the conditional caller's ancestry,
other scenes, callers or full-ROM coverage. Other ordinary language paths,
real BIOS, UI save/Load, revision-sensitive older Wine fixtures, Linux, USA
and native Windows remain open. No performance comparison was made.

## E367 White Flowers queued halfword rows source plan

Wine disassemblies `e367-white-flowers-halfword-roots-dis-wine-z47ofoc0`,
`e367-white-flowers-copy-routine-dis-wine-jk8i1ag1` and
`e367-white-flowers-queue-executor-dis-wine-lqsxdeqq` identify an original
conditional source selection at `0812bf10..0812bf38`. Zero selects `3ba0d8`;
nonzero selects `3ba218`. The caller registers a queued transfer via
`08000be4`, and the original queue executor uses eight rows of 20 halfwords,
with 40-byte source and 64-byte destination strides. Both 320-byte source
projections and the three contiguous literal words at `12bf7c..12bf88` have
no E276 owner.

Protected plan `e367-white-flowers-queued-rows-source-plan-f2kk2tp7`
authenticates source and code fingerprints, original constants, prior owners
and the address-only sample. The 652 selected source bytes include 328 E362
sampled bytes; publication would leave 6,437, including 3,600 previously
documented copied-code support. E368 below publishes the selected runtime
sources, but this conditional plan does not establish executing PCs, caller
ancestry, all transfer states or other scenes. Linux, USA and native Windows remain open. No performance comparison
was made.

## E368 White Flowers queued halfword rows runtime

EUR E277-d276 extends `graphics.observed-tilemap` with two conditional
20 × 8 halfword projections at `3ba0d8` and `3ba218`, and three original
caller literal words at `12bf7c..12bf88`. The original selector, producer,
queue and DMA execution remain guest work. Previous source owners and the
prior serialized representation remain valid.

Wine reader `e368-white-flowers-queued-readers-wine-xeokmdlp` verifies six interpreter/generated selector cases and
three original selector/queue/DMA cases for zero and nonzero flags, including
both destination projections. It checks all selected bytes through ROM mirrors
and mixed boundaries, 145 unchanged resources, source validation and snapshot
ownership. Wine scene `e368-white-flowers-queued-scene-wine-8n1stan0` verifies 700 retained White Flowers frames
with all selected backing bytes altered: state, guest clock, video, OAM,
dispatch and five nonempty audio streams match every frame, with the first
projection consumed. The second projection is covered by controlled readers,
not established in this ordinary scene. Wine CLI `e368-white-flowers-queued-cli-wine-oxrskpdo` creates, reuses and
verifies 147 matching files. All Wine servers drained with the local Bluetooth
overlay.

Static join `e368-white-flowers-queued-sample-residual-ce_e0lx3` leaves 6,437 E362 address-only sampled bytes outside
E277 declared sources: 3,600 are previously documented copied-code support
and 2,837 require classification. This sample lacks executing PCs and does
not establish caller ancestry, other scenes or full-ROM coverage. Other
selector paths, callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E369 White Flowers remaining source candidates

Protected grouping `e369-white-flowers-residual-groups-47pkvd93` reconciles
the E362 address-only sample against E277 without replay: 6,437 bytes remain
outside declared sources, in 323 sampled intervals. Of these, 3,600 are the
previously documented copied-IWRAM executable support and 2,837 need further
classification. A bounded bit-pattern scan finds 645 candidate lower-ROM
literal words; it does not establish executed instructions or ownership.

Focused read-only diagnosis identified a candidate indexed archive beginning
at `3c94bc`, whose independently parsed extent is `3c94bc..3cad91` and
includes 131 remaining sampled bytes. Original Wine disassembly
`e369-white-flowers-indexed-caller-dis-wine-gljcbz8v` identifies a caller at
`080cf3e0..080cf3f4`: it loads literal `0cf3f8`, projects its selector to
eight bits and calls indexed decoder `0800537c` for one program. The literal
word contributes four sampled bytes. These are conditional source leads; a
protected plan E370 below authenticates reconstruction, prior owners and
sample intersection before runtime publication. The address-only sample does
not establish that this caller executed. Other scenes, Linux, USA and native
Windows remain open. Focused Sol `/root/sol_e343_residual_diagnosis` was
requested with xhigh; effective model and effort were not independently
confirmed. No performance comparison was made.

## E370 White Flowers directly rooted indexed archive source plan

Protected plan `e370-white-flowers-direct-indexed-source-plan-0oy7mnn2`
authenticates the original wrapper `080cf3e0..080cf3f6`, its literal word
`0cf3f8`, and a directly rooted indexed archive at `3c94bc..3cad91`. An
independent static parser reconstructs 62 programs of 128 decoded bytes each;
program 16 spans the sampled `3c9d94..3c9df5` interval. E371 below verifies
production parsing and selected runtime consumption.

The 6,361 selected source bytes have no E277 owner. The prior indexed archive
starts at `3cad94`; the intervening three bytes are not included. The selected
sources cover 135 E362 residual bytes, leaving 6,302 if published, including
3,600 previously documented copied-code support. The original wrapper passes
an unsigned byte selector and requests one program; no new guest selector
cap is inferred. The address-only sample does not establish executing PCs or
caller ancestry. Other scenes, Linux, USA and native Windows remain open.
No performance comparison was made.

## E371 White Flowers directly rooted indexed archive runtime

EUR E278-d277 extends `graphics.indexed-archives` with the E370 62-program
archive at `3c94bc` and original literal at `0cf3f8`. The existing neighboring
archive keeps its owner and the intervening three bytes remain outside both
projections. The original wrapper and decoder retain their own selector
handling; the previous serialized representation remains valid.

Wine reader `e371-white-flowers-direct-readers-wine-ywvmnh46` verifies production reconstruction, program 16 and
boundary selectors in six original interpreter/generated cases, including
low-byte projection and out-of-range behavior. It checks 145 unchanged
resources, altered ROM mirrors, mixed boundaries, the adjacent existing
owner and gap, source authentication and snapshot ownership. Wine scene
`e371-white-flowers-direct-scene-wine-9pwn72wh` compares 700 retained White Flowers frames with both new backing
sources altered: full state, guest clock, video, OAM, dispatch and five
nonempty audio streams match each frame, with the archive and literal consumed.
Wine CLI `e371-white-flowers-direct-cli-wine-t_d5oash` creates, reuses and verifies 147 matching files. All Wine
servers drained with the local Bluetooth overlay.

Static join `e371-white-flowers-direct-sample-residual-2casb5p1` leaves 6,302 E362 address-only sampled bytes outside
E278 declared sources: 3,600 are previously documented copied-code support
and 2,702 require classification. This sample does not establish executing
PCs, caller ancestry, other scenes or full-ROM coverage. Other programs and
callers, source-save history, real BIOS, UI save/Load, revision-sensitive
older Wine fixtures, Linux, USA and native Windows remain open. No
performance comparison was made.

## E372 White Flowers four literal pools source plan

Wine disassemblies `e372-white-flowers-literal-pools-dis-wine-1cfeb1yt` and
`e372-white-flowers-literal-tail-dis-wine-dcgqeh_6` examine four original
Thumb contexts and their PC-relative literal loads. Protected plan
`e372-white-flowers-literal-pools-source-plan-bur61cim` authenticates 49
selected words in `023a74..023a9c`, `03d170..03d1a8`, `0bbc78..0bbc9c` and
`127b0c..127b4c`, including the original opcodes, four context fingerprints,
E278 owner disjointness and all 196 sampled bytes. If published, the E362
sample would leave 6,106 bytes outside declared sources, including 3,600
previously documented copied-code support.

The selected words retain their literal values without assigning semantics.
The sample lacks executing PCs and does not prove that the authenticated
readers caused those observations. Other candidate pools, callers, scenes,
Linux, USA and native Windows remain open. E373 below verifies selected
runtime publication. No performance comparison was made.

## E373 White Flowers four literal pools runtime

EUR E279-d278 extends `engine.thumb-pc-literals` with the E372 49 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e373-white-flowers-literals-readers-wine-6ot86e3c` verifies 98 original interpreter/generated reader
cases, all 49 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e373-white-flowers-literals-scene-wine-th4l8fim` compares 700 retained White Flowers frames with all 49 backing
words altered. All 49 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e373-white-flowers-literals-cli-wine-eixsrqj_`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e373-white-flowers-literals-sample-residual-wzleu30g` leaves 6,106 E362 address-only sampled bytes outside
E279 declared sources: 3,600 are previously documented copied-code support
and 2,506 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E374 White Flowers eight literal pools source plan

Wine disassembly `e374-white-flowers-literal-pools-dis-wine-x8vvx3in` and
protected plan `e374-white-flowers-literal-pools-source-plan-_dh5w4pb`
authenticate 62 selected PC-relative Thumb literal readers across eight
additional original code contexts. Their 248 source bytes have no E279 owner
and are all in the E362 address-only sample. Publication would leave 5,858
sampled bytes outside declared sources, including 3,600 previously documented
copied-code support. The selected offsets, opcodes and context fingerprints are
retained in the protected plan.

Literal values are preserved without assigning semantics. The sample lacks
executing PCs and does not prove that these readers caused the observations.
Other candidate pools, callers, scenes, Linux, USA and native Windows remain
open. E375 below verifies selected runtime publication. No performance
comparison was made.

## E375 White Flowers eight literal pools runtime

EUR E280-d279 extends `engine.thumb-pc-literals` with the E374 62 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e375-white-flowers-more-literals-readers-wine-g7ci2tnq` verifies 124 original interpreter/generated reader
cases, all 62 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e375-white-flowers-more-literals-scene-wine-0kdrgo3_` compares 700 retained White Flowers frames with all 62 backing
words altered. All 62 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e375-white-flowers-more-literals-cli-wine-ucmhxrmh`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e375-white-flowers-more-literals-sample-residual-cjcrgtzy` leaves 5,858 E362 address-only sampled bytes outside
E280 declared sources: 3,600 are previously documented copied-code support
and 2,258 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E376 White Flowers ten literal pools source plan

Wine disassembly `e376-white-flowers-literal-pools-dis-wine-q6818iy7` and
protected plan `e376-white-flowers-literal-pools-source-plan-ei2nr32i`
authenticate 62 selected PC-relative Thumb literal readers across ten
additional original code contexts. Their 248 source bytes have no E280 owner
and are all in the E362 address-only sample. Publication would leave 5,610
sampled bytes outside declared sources, including 3,600 previously documented
copied-code support. The selected offsets, opcodes and context fingerprints are
retained in the protected plan.

Literal values are preserved without assigning semantics. The sample lacks
executing PCs and does not prove that these readers caused the observations.
Other candidate pools, callers, scenes, Linux, USA and native Windows remain
open. E377 below verifies selected runtime publication. No performance
comparison was made.

## E377 White Flowers ten literal pools runtime

EUR E281-d280 extends `engine.thumb-pc-literals` with the E376 62 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e377-white-flowers-later-literals-readers-wine-sfa40qnd` verifies 124 original interpreter/generated reader
cases, all 62 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e377-white-flowers-later-literals-scene-wine-4aph05lq` compares 700 retained White Flowers frames with all 62 backing
words altered. All 62 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e377-white-flowers-later-literals-cli-wine-se9n4l2l`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e377-white-flowers-later-literals-sample-residual-kxz4n64i` leaves 5,610 E362 address-only sampled bytes outside
E281 declared sources: 3,600 are previously documented copied-code support
and 2,010 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E378 White Flowers thirteen literal pools source plan

Wine disassembly `e378-white-flowers-literal-pools-dis-wine-5yhjeio9` and
protected plan `e378-white-flowers-literal-pools-source-plan-ro9nq2gs`
authenticate 60 selected PC-relative Thumb literal readers across thirteen
additional original code contexts. Their 240 source bytes have no E281 owner
and are all in the E362 address-only sample. Publication would leave 5,370
sampled bytes outside declared sources, including 3,600 previously documented
copied-code support. The selected offsets, opcodes and context fingerprints are
retained in the protected plan.

Literal values are preserved without assigning semantics. The sample lacks
executing PCs and does not prove that these readers caused the observations.
Other candidate pools, callers, scenes, Linux, USA and native Windows remain
open. E379 below verifies selected runtime publication. No performance
comparison was made.

## E379 White Flowers thirteen literal pools runtime

EUR E282-d281 extends `engine.thumb-pc-literals` with the E378 60 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e379-white-flowers-thirteen-literals-readers-wine-xhl37t_m` verifies 120 original interpreter/generated reader
cases, all 60 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e379-white-flowers-thirteen-literals-scene-wine-ukju6ylf` compares 700 retained White Flowers frames with all 60 backing
words altered. All 60 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e379-white-flowers-thirteen-literals-cli-wine-hjw0548p`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e379-white-flowers-thirteen-literals-sample-residual-pte_4_ze` leaves 5,370 E362 address-only sampled bytes outside
E282 declared sources: 3,600 are previously documented copied-code support
and 1,770 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E380 White Flowers twenty-eight literal pools source plan

Wine disassembly `e380-white-flowers-literal-candidates-dis-wine-umy1ze7f` and
protected plan `e380-white-flowers-literal-pools-source-plan-ig_ne93b`
authenticate 92 selected PC-relative Thumb literal readers across 28
original code contexts. Their 368 source bytes have no E282 owner and are all
in the E362 address-only sample. Publication would leave 5,002 sampled bytes
outside declared sources, including 3,600 previously documented copied-code
support. The selected offsets, opcodes and context fingerprints are retained in
the protected plan.

Literal values are preserved without assigning semantics. The sample lacks
executing PCs and does not prove that these readers caused the observations.
Other candidate pools, callers, scenes, Linux, USA and native Windows remain
open. E381 below verifies selected runtime publication. No performance
comparison was made.

## E381 White Flowers twenty-eight literal pools runtime

EUR E283-d282 extends `engine.thumb-pc-literals` with the E380 92 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e381-white-flowers-many-literals-readers-wine-sn2kn037` verifies 184 original interpreter/generated reader
cases, all 92 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e381-white-flowers-many-literals-scene-wine-qnj53ytl` compares 700 retained White Flowers frames with all 92 backing
words altered. All 92 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e381-white-flowers-many-literals-cli-wine-n4ygbw8g`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e381-white-flowers-many-literals-sample-residual-l4g6pt_j` leaves 5,002 E362 address-only sampled bytes outside
E283 declared sources: 3,600 are previously documented copied-code support
and 1,402 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E382 White Flowers sixty-two literal pools source plan

Wine disassembly `e382-white-flowers-literal-candidates-dis-wine-hrfc36ve` and
protected plan `e382-white-flowers-literal-pools-source-plan-vh86eqhl`
authenticate 128 selected PC-relative Thumb literal readers across 62
original code contexts. Their 512 source bytes have no E283 owner and are all
in the E362 address-only sample. Publication would leave 4,490 sampled bytes
outside declared sources, including 3,600 previously documented copied-code
support. The selected offsets, opcodes and context fingerprints are retained in
the protected plan.

The first static selection `e382-white-flowers-literal-candidates-t3dztdqn`
failed because its inherited minimum range length excluded two-word ranges; the
corrected selection `e382-white-flowers-literal-candidates-oty9qqm6` passes.
No Wine process was involved in that failed selection. Literal values are
preserved without assigning semantics. The sample lacks executing PCs and does
not prove that these readers caused the observations. Other candidate pools,
callers, scenes, Linux, USA and native Windows remain open. E383 below verifies selected runtime publication. No performance
comparison was made.

## E383 White Flowers sixty-two literal pools runtime

EUR E284-d283 extends `engine.thumb-pc-literals` with the E382 128 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved.

Wine reader `e383-white-flowers-bulk-literals-readers-wine-ibbq6jvp` verifies 256 original interpreter/generated reader
cases, all 128 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e383-white-flowers-bulk-literals-scene-wine-8caa5f6p` compares 700 retained White Flowers frames with all 128 backing
words altered. All 128 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e383-white-flowers-bulk-literals-cli-wine-q0eg09u7`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e383-white-flowers-bulk-literals-sample-residual-fb5qs3bk` leaves 4,490 E362 address-only sampled bytes outside
E284 declared sources: 3,600 are previously documented copied-code support
and 890 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E384 White Flowers remaining sampled literal pools source plan

Wine disassembly `e384-white-flowers-literal-candidates-dis-wine-4lyi07_z` and
protected plan `e384-white-flowers-literal-pools-source-plan-6teqz1tq`
authenticate 182 selected PC-relative Thumb literal readers across 163
original code contexts. Their 728 source bytes have no E284 owner and are all
in the E362 address-only sample. Publication would leave 3,762 sampled bytes
outside declared sources, including 3,600 previously documented copied-code
support. The selected offsets, opcodes and context fingerprints are retained in
the protected plan.

Candidate source word `03320c` is excluded: its only bounded-scan candidate
reader `032eb8` is part of a jump-table pointer sequence and is not established
as an instruction. The true reader and source contract for `03320c` remain
open. Literal values are preserved without assigning semantics. The sample lacks
executing PCs and does not prove that the accepted readers caused observations.
Other candidate pools, callers, scenes, Linux, USA and native Windows remain
open. E385 below verifies selected runtime publication. No performance
comparison was made.

## E385 White Flowers remaining sampled literal pools runtime

EUR E285-d284 extends `engine.thumb-pc-literals` with the E384 182 selected
words and their original PC-relative readers. The original guest instructions
and fallback remain active; the previous representation and source owners are
preserved. The false candidate reader for `03320c` remains excluded.

Wine reader `e385-white-flowers-remaining-literals-readers-wine-fm569axt` verifies 364 original interpreter/generated reader
cases, all 182 source words through altered mirrors and mixed boundaries, 145
unchanged resources, source authentication and snapshot ownership. Wine scene
`e385-white-flowers-remaining-literals-scene-wine-tf_2k_48` compares 700 retained White Flowers frames with all 182 backing
words altered. All 182 are consumed; full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e385-white-flowers-remaining-literals-cli-wine-rm83cqb2`
creates, reuses and verifies 147 matching files. All Wine servers drained
with the local Bluetooth overlay.

Static join `e385-white-flowers-remaining-literals-sample-residual-x6f12no9` leaves 3,762 E362 address-only sampled bytes outside
E285 declared sources: 3,600 are previously documented copied-code support
and 162 require classification. This sample does not establish executing
PCs, specific reader ancestry, other scenes or full-ROM coverage. Other
literal pools and callers, source-save history, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E386 White Flowers byte-indexed switch source plan

Protected static plan `e386-white-flowers-032e-switch-source-plan-0x7b7450`
uses the retained Wine disassembly `e384-white-flowers-literal-candidates-dis-wine-4lyi07_z`
to authenticate the original byte-indexed dispatch at `032dfa..032e0e`. It
loads the root at `032e10`, which is already owned by `engine.thumb-pc-literals`,
then reads a destination at `032e14 + 4 * index` and branches through the guest
instruction. The byte load bounds the index to 0..255. The 256 destination
words span `032e14..033214`, reconstruct exactly from 61 distinct even ROM
targets, and do not overlap E285 source owners. The source SHA-256 is
`8b1cd081e8694bc98b608a3fce6b2f34449b907612e64b997d124e6103804e7f`.

The E362 address-only sample contains three unowned entries, indices 5, 45 and
254; the last is `03320c`, previously excluded because the apparent literal
reader `032eb8` is itself a jump-table pointer. Publication of the full
destination domain would remove 12 sampled bytes. The actual sampled PCs,
selector ancestry, other readers and whether each index occurs remain open.
E387 below verifies selected runtime publication. Other scenes, Linux, USA
and native Windows remain open. No performance comparison was made.

## E387 White Flowers byte-indexed switch runtime

EUR E286-d285 extends `engine.thumb-switch-tables` with all 256 E386
destinations. Their existing root remains owned by `engine.thumb-pc-literals`;
the original guest dispatch and fallback remain active. Previous switch
representations and all other resources remain preserved.

Wine reader `e387-white-flowers-switch-readers-wine-rzmpfvqn` verifies ten original interpreter/generated dispatch
cases at indices 0, 5, 45, 254 and 255, two out-of-domain guard cases, two
out-of-projection fallback cases, all 1,024 bytes through altered mirrors and
mixed boundaries, 145 unchanged resources and snapshot ownership. Wine scene
`e387-white-flowers-switch-scene-wine-dvb87b5n` consumes the selected table over 700 retained White Flowers frames
with its root and destinations altered. Full state, guest clock, video, OAM,
dispatch and five nonempty audio streams match each frame. Wine CLI `e387-white-flowers-switch-cli-wine-4zd4hs5l`
creates, reuses and verifies 147 matching files. All Wine servers drained with
the local Bluetooth overlay. The initial reader `7d7zef1n` failed after
stopping inside a generated chain at the guard's BL; Wine drained. The corrected
test stops at the original fused BL destination. A focused Sol consultation
was requested with xhigh; effective model and effort were not independently
confirmed.

Static join `e387-white-flowers-switch-sample-residual-3unoh3bm` leaves 3,750 E362 address-only sampled bytes outside
E286 declared sources: 3,600 are previously documented copied-code support
and 150 require classification. The sample does not establish executing PCs
or selector ancestry; other callers and scenes, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E388 White Flowers residual switch source plan

Protected static plan `e388-white-flowers-residual-switch-source-plan-c4c67ilr`
authenticates seven EUR original Thumb switches using retained Wine disassembly.
Their roots `031618`, `036320`, `03ce28`, `043f90`, `0465a8`, `08d344` and
`0d1f58` are already owned by `engine.thumb-pc-literals`. Their 373 destination
words occupy 1,492 bytes and have no E286 source overlap. The original guards,
shifts, root loads and indirect transfers determine each bounded source domain;
the exact code and destination hashes are retained in the plan.

The E362 address-only sample intersects 40 of these bytes; publication would
leave 3,710 sampled bytes outside declared sources, including 3,600 previously
documented copied-code support. At root `031618`, an out-of-range selector can
call a helper that may return: the guest path and fallback beyond the projected
255 entries must stay intact. Sampled PCs, selector ancestry, other readers
and reachability remain open. E389 below verifies selected runtime publication. Other scenes, Linux, USA
and native Windows remain open. No performance comparison was made.

## E389 White Flowers residual switches runtime

EUR E287-d286 extends `engine.thumb-switch-tables` with all 373 E388
destinations in seven arrays. Their existing roots remain owned by
`engine.thumb-pc-literals`; original guest dispatch and fallback stay active.
Previous switch representations and all other resources remain preserved.

Wine reader `e389-white-flowers-residual-switches-readers-wine-rf3njife` verifies 46 original interpreter/generated dispatch
cases including sampled indices and boundaries, 14 out-of-domain guard cases,
14 out-of-projection fallback cases, all 1,492 bytes through altered mirrors
and mixed boundaries, 145 unchanged resources and snapshot ownership. The
`031618` helper's possible return is preserved; its subsequent behavior remains
outside this controlled check. Wine scene `e389-white-flowers-residual-switches-scene-wine-6x87yzpb` consumes all seven tables
over 700 retained White Flowers frames with their roots and destinations
altered. Full state, guest clock, video, OAM, dispatch and five nonempty audio
streams match each frame. Wine CLI `e389-white-flowers-residual-switches-cli-wine-4yz9hng2` creates, reuses and verifies 147
matching files. All Wine servers drained with the local Bluetooth overlay.

Static join `e389-white-flowers-residual-switches-sample-residual-6uy1wu37` leaves 3,710 E362 address-only sampled bytes outside
E287 declared sources: 3,600 are previously documented copied-code support
and 110 require classification. The sample does not establish executing PCs
or selector ancestry; other callers and scenes, real BIOS, UI save/Load,
revision-sensitive older Wine fixtures, Linux, USA and native Windows remain
open. No performance comparison was made.

## E390 White Flowers mixed switch source plan

Protected static plan `e390-white-flowers-mixed-switch-source-plan-_u4vahgs`
authenticates five EUR Thumb switches at roots `04e038`, `08dfd0`, `0bbffc`,
`0cd950` and `125c88` using retained Wine disassembly. Their 89 destinations
occupy 356 bytes and do not overlap E287 source owners. Four roots and an
adjacent literal at `08dfcc` are also unowned; root `0cd950` already belongs
to `engine.thumb-pc-literals`. The five newly selected literal words occupy
20 bytes. Original guards, shifts, root loads, transfers and source hashes are
retained in the plan.

The E362 address-only sample intersects 80 selected bytes; publication would
leave 3,630 sampled bytes outside declared sources, including 3,600 previously
documented copied-code support. The `0bbffc` out-of-range helper may return,
so the original guest fallback must remain active beyond 37 destinations.
Sampled PCs, selector ancestry, other readers and reachability remain open.
E391 below validates the selected runtime publication. Other scenes, Linux,
USA and native Windows remain open. No performance comparison was made.


## E391 White Flowers mixed switches runtime

EUR E288-d287 extends `engine.thumb-switch-tables` with five selected
switch tables (89 destinations, 356 bytes) and `engine.thumb-pc-literals`
with five authenticated literal words (20 bytes). The existing root owner
at `0cd950` is retained, as are previous representations and other resource
files. Unowned bytes beyond each projection continue through their original
owner or ROM backing; the `0bbffc` helper can return and its post-return path
is not established by this check.

Wine reader validation `e391-white-flowers-mixed-switches-readers-wine-ge_wrdqh`
passes 42 original interpreter/generated dispatch cases, 10 guard cases,
10 out-of-projection cases, and two standalone literal-reader cases. It checks
mirrors, mixed-width boundaries, altered original sources, coherent-edit
rejection, missing-dataset handling and snapshot owner retention.

Wine scene `e391-white-flowers-mixed-switches-scene-wine-s_vunzae` passes 700
retained White Flowers battle-entry frames with all selected tables and
literals consumed while their original bytes are altered; state, guest clock,
video, OAM, dispatch and five nonempty audio streams match on each frame.
Wine CLI `e391-white-flowers-mixed-switches-cli-wine-ausnc_j0` creates, reuses and verifies 147 files;
all Wine servers drained. Static join
`e391-white-flowers-mixed-switches-sample-residual-y9chyeki` accounts for
80 E362 sampled bytes and leaves 3,630 unowned sampled bytes: 3,600 previously
documented copied-code support and 30 other bytes. The address-only sample
does not establish executing PCs, selector ancestry or full-ROM coverage.
Other readers, scenes and post-return behavior, Linux, USA, real BIOS and
native Windows remain open. No performance comparison was made.


## E392 White Flowers final sampled switch source plan

Protected candidate map `e392-white-flowers-final-sample-candidates-_bg_fj4z`
locates the 30 non-support bytes remaining in the E362 address-only sample.
Wine disassembly `e392-white-flowers-final-sample-dis-wine-7edn_8m0` and
protected plan `e392-white-flowers-final-switch-source-plan-5w_6ycut`
authenticate three EUR Thumb switch tables rooted at `02a7b0`, `125fe4` and
`1279b8`, with 11, 12 and 7 even destinations respectively. Their destination
sources occupy 120 disjoint bytes. Three new literal words occupy 12 bytes:
`02a7ac`, `02a7b0` and `125fe4`; the third switch root `1279b8` is already
owned by `engine.thumb-pc-literals`. The first dispatch subtracts one before
its guard; the second obtains a byte and subtracts three; the third guards
register `r5`. Preserve these original selectors and all fallback behavior.

The 132 selected source bytes include 24 sampled bytes. Publication would leave
3,606 sampled bytes outside declared sources: 3,600 previously documented
copied-code support bytes and six isolated bytes at `397b1e..397b21`,
`3c9460` and `6e89aa..6e89ac`. Their meanings and producers are not established
by proximity or linear decoding. E393 below validates selected runtime use.
Sampled PCs and ancestry, other readers and scenes remain open; Linux, USA and
native Windows remain pending.
No performance comparison was made.


## E393 White Flowers final sampled switches runtime

EUR E289-d288 serves three switch destination arrays (30 destinations,
120 bytes) and three literal words (12 bytes) from authenticated resources.
The already-owned root `1279b8` and earlier representations remain intact;
the original selector transformations and out-of-projection fallback remain
active. Wine reader `e393-white-flowers-final-switches-readers-wine-7kkhhjbu` passes 14 original
interpreter/generated dispatch cases, six guards, six fallback cases and two
standalone literal-reader cases, plus mirror, mixed-boundary, coherent-edit,
missing-dataset and snapshot-owner checks.

Wine scene `e393-white-flowers-final-switches-scene-wine-vwkx19aq` passes 700 retained
White Flowers battle-entry frames with all selected sources consumed while
their original bytes are altered. State, guest clock, video, OAM, dispatch
and five nonempty audio streams match on each frame. Wine CLI `e393-white-flowers-final-switches-cli-wine-wdmf8psu`
creates, reuses and verifies 147 files. All Wine servers drained. Protected
join `e393-white-flowers-final-switches-sample-residual-4a4mml6z` confirms 24 E362 sampled bytes
are covered and 3,606 remain outside declared sources, including 3,600
previously documented copied-code support bytes and six isolated bytes with
unknown producers. This address-only sample cannot establish executing PCs,
ancestry or full-ROM coverage. Other readers, scenes, real BIOS, Linux, USA
and native Windows remain open. No performance comparison was made.


## E394 White Flowers isolated sampled reader scopes

Wine observation `e394-white-flowers-isolated-sites-wine-9vq22s5t` reuses the
E289 dataset and first 600 frames of the retained White Flowers continuation.
All six E393 isolated sampled bytes occur: byte reads at `397b1e..397b21` have
no interpreter scope, the `3c9460` byte read occurs in Thumb scope `080cf406`,
and the halfword read at `6e89aa..6e89ac` occurs in Thumb scope `0802edcc`.
Retained Wine disassembly `e384-white-flowers-literal-candidates-dis-wine-4lyi07_z`
shows `LDRB` and `LDRH` at the respective scoped PCs, consistent with those
access widths. This scope may include nested DMA or HLE accesses; register
operands, exact attribution and source extents still need proof. The unscoped
reads may be native or host accesses, so no instruction is inferred.

This is one source-site observation, not a new scene-equivalence claim.
Native context, other scenes, real BIOS, Linux, USA and native Windows remain
open. The Wine server drained; no performance comparison was made.


## E395 White Flowers 397b1e native context

The test-only observer in `native_rom_data_residue_tests` follows the retained
EUR White Flowers continuation for 600 frames. Wine unit
`e395-white-flowers-397b1e-context-wine-95w54vo0` sees all three byte reads at
`397b1e..397b21` through the native read API. The filtered DMA capture records
three non-DMA requests and no DMA request. All three native records have outer
dispatch `08083bd1` and exported PC `08083bf2`; the latter may be stale and
the dispatch may include chained blocks, so neither proves the load instruction.

Retained disassembly shows a possible byte reader at `08083c00`, based on a
pointer loaded at `08083bf2`; its original selector domain and source extent
need independent authentication before use. The observer makes no new
scene-equivalence claim. Other readers, scenes, real BIOS, Linux, USA and native
Windows remain open. The Wine server drained; no performance comparison was
made.


## E396 White Flowers 397b1e bounded source plan

Wine disassemblies `e396-white-flowers-397b1e-dis-wine-o1ejpxi9` and
`e396-white-flowers-397b1e-helper-dis-wine-qvbss5me`, together with protected
plan `e396-white-flowers-397b1e-source-plan-g28z8ftk`, identify the original
`08083c00` signed-byte access sequence using base `08397b0a`. The existing
literal owner retains its root at `083c14`. The E395 native context is
consistent with indices 20, 21 and 22; it does not independently establish an
exact instruction.

The plan selects the non-overlapping 48-byte projection `397b0a..397b3a`,
matching the `0x30` argument passed to the subsequent cursor helper. The read
occurs *before* that helper, so an initially out-of-range cursor must continue
to use original fallback. This projection is not a proven asset boundary or
proof that every initial index is bounded. E397 below validates the selected
original-reader and helper execution and runtime publication. Producer ancestry
and other paths remain open. Linux, USA and native Windows remain pending; all Wine servers
drained and no performance comparison was made.


## E397 White Flowers signed-byte projection runtime

EUR E290-d289 adds `engine.white-flowers-signed-bytes` for the authenticated
48-byte projection at `397b0a..397b3a`. Original source bytes are represented
as signed values and reconstructed for guest reads. The existing literal root
remains separately owned; all addresses outside the projection retain their
prior owner or ROM fallback, including out-of-range initial cursors.

Wine reader `e397-white-flowers-signed-bytes-readers-wine-dagykkmv` passes 14 original interpreter/generated
reader cases, including sampled indices 20–22, both projection boundaries and
outside indices 48 and 255; the original helper is interpreted. Mixed-width
reads across boundaries, three mirrors, coherent-edit rejection, missing
dataset and snapshot owner retention also pass. The 146 earlier resources
and their files remain unchanged.

Wine scene `e397-white-flowers-signed-bytes-scene-wine-_s3ccy3e` passes 700 retained White Flowers battle-entry
frames with the 48 original source bytes altered and all three sampled values
consumed. State, guest clock, video, OAM, dispatch and five nonempty audio
streams match on each frame. Wine CLI `e397-white-flowers-signed-bytes-cli-wine-c2h7zruq` creates, reuses and
verifies 148 files. All Wine servers drained. Static join `e397-white-flowers-signed-bytes-sample-residual-9x1r8abz`
leaves 3,603 E362 sampled bytes outside declared sources: 3,600 previously
documented copied-code support and three isolated bytes. This address-only
sample does not establish full-ROM coverage. Source asset extent, producer
ancestry, other scenes, real BIOS, Linux, USA and native Windows remain open.
No performance comparison was made.


## E398 White Flowers isolated original operands

Wine unit `e398-white-flowers-operands-wine-s86speyz` observes the two
remaining isolated E362 accesses during 600 retained EUR White Flowers frames.
Immediately before the interpreter executes the original instructions,
`080cf406` has byte operand `083c9460` with base `083c943c`; `0802edcc` has
halfword operand `086e89aa`, base register `086e8994`, and already-incremented
cursor value 12. Both are normal missing-block interpreter steps, removing the
ambiguity that a nested access supplied these selected operands.

The retained original instruction sequence selects the observed first-row
halfword at index 11 and computes the observed byte from index 16. The complete
asset extents, caller ancestry and initial selector domains remain unproved;
no source ownership or additional scene equivalence is claimed. Linux, USA
and native Windows remain pending. The Wine server drained and no performance
comparison was made.


## E399 White Flowers isolated source plan

Protected plan `e399-white-flowers-isolated-source-plan-lnthon5v` combines
E398's direct Wine operands with retained original disassembly. It selects a
20-halfword first row at `6e8994..6e89bc` and the single byte `3c9460`,
covering the final three non-support bytes in the E362 sample. The original
row selector advances by 40 bytes per row; the observed halfword is index 11.
The other original reader obtains the selected byte at index 16 and extracts
its high nibble. Both pointer literals are already owned by
`engine.thumb-pc-literals`; the selected sources do not overlap prior owners.

The initial indices and other row selectors are not bounded by these source
projections. Original fallback outside the selected physical bytes is required.
The plan does not establish complete asset extents or caller ancestry. E400
below validates the selected runtime publication; 3,600 E362 sampled bytes
remain outside declared sources, all previously classified as copied-code support. Other
scenes, Linux, USA and native Windows remain open; no performance comparison
was made.


## E400 White Flowers isolated values runtime

EUR E291-d290 adds `engine.white-flowers-isolated-values`: a first row of 20
halfwords and the selected byte used by the original nibble reader. Both root
literals remain separately owned, while reads outside these physical sources
retain their prior owner or original ROM fallback. The 147 previous resources
and files are unchanged.

Wine reader `e400-white-flowers-isolated-values-readers-wine-i8jfpb3l` passes four original selector-setup, 12
row-reader and 12 byte-reader interpreter/generated cases, including selected
indices, boundary values, other rows and out-of-projection fallback. Mixed
reads, three mirrors, coherent-edit rejection, missing-dataset handling and
snapshot owner retention pass. Wine scene `e400-white-flowers-isolated-values-scene-wine-ohe96xlp` passes 700
retained White Flowers frames with all selected backing bytes altered and both
projections consumed; state, guest clock, video, OAM, dispatch and five nonempty
audio streams match per frame. Wine CLI `e400-white-flowers-isolated-values-cli-wine-05w416tw` creates, reuses and
verifies 149 files. All Wine servers drained.

Protected join `e400-white-flowers-isolated-values-sample-residual-urhx1g0d` leaves 3,600 unowned sampled bytes, all
previously classified as copied-code support, and zero other unclassified
bytes in the E362 address-only sample. This does not prove full-ROM coverage
or remove the copied-code dependency. Complete source extents, caller ancestry,
other scenes, real BIOS, Linux, USA and native Windows remain open. No
performance comparison was made.


## E401 White Flowers copied-code support reconciliation

Protected static join `e401-white-flowers-code-support-reconcile-7yeu1i9c`
confirms that the 3,600 E400 residual sample bytes are exactly the same
physical bytes left by E246 in the `d968e8..d97728` copied-IWRAM source block.
The separate 48-byte metadata alias `d976e8..d97718` is already owned by
`graphics.obj-dimensions`. The block contains executable support and remains
in the original ROM under the [inventory contract](development/ROM_DATA_INVENTORY.md);
the address join does not classify every individual byte as an instruction.

No byte outside that block remains unclassified in the selected E362 sample.
This does not establish other scene coverage or ROM-free startup. A separate
code-migration design would be needed to change the executable-code exclusion;
Linux, USA, native Windows and other consumers remain open. No game was
replayed and no performance comparison was made.


## E402 Roulette native sample reconciliation

Protected static join `e402-roulette-native-residual-join-ycaxc95s` compares
the retained E108 EUR Roulette native API inventory against E291. All 732
physical bytes that lacked E108 owners are now declared: 672 by Thumb literal
resources, 24 by switch tables, 20 by UI context templates, eight by DMA
directories and eight by IWRAM helper metadata. The join authenticates 175
intersecting source spans against the exact EUR ROM and performs no replay.

This is limited to native API reads in the retained 12-frame continuation;
it does not prove executing PCs, interpreter or host readers, other scenes or
full-ROM coverage. Linux, USA and native Windows remain open. No performance
comparison was made.


## E403 Star Cross ROM read inventory

Wine `e403-star-cross-audit-wine-us0nqlxx` inventories selected bus, audio and
OBJ readers during the previously recorded 480-frame European Star Cross
confirmation with E291-d290. It reuses the authenticated checkpoint, input,
Windows test executable and dataset. Raw and managed machines match complete
state, guest clock, video, OAM, dispatch and five nonempty audio streams at
every frame; Wine drained with the local Bluetooth overlay.

Of 243,297 distinct observed physical ROM bytes, 7,359 are outside declared
E291 sources: 136 below `300000` and 7,223 at or above it. The retained
`audit.json` records addresses, widths and selected reader classes but no
executing PCs. These are investigation leads, not proven data extents or a
claim that each byte is an asset; code and copied support require separate
classification. Unselected readers, other scenes, the source save's earlier
history and full-ROM coverage remain open. Linux, USA and native Windows are
unverified; no performance comparison was made.


## E404 Star Cross residual grouping

Protected static unit `e404-star-cross-residual-groups-7vnaqwjc` groups the
retained E403 E291 address inventory against the authenticated EUR ROM without
a replay. Its 7,359 sampled, unowned bytes form 37 contiguous intervals; none
overlap the earlier `d968e8..d97728` copied-IWRAM source block. Largest sampled
intervals are `affc70..b00d96` (4,390 bytes), `b00d9c..b00fd6` (570 bytes)
and `6bad16..6bb4f4` (2,014 bytes). Bus reads cover the first two; bus and
sprite-peek observations intersect the third. A bounded linear bit-pattern scan
also finds 19 potential Thumb literal words below `300000`.

The intervals are observed bytes, not authenticated resource extents. Their
reader classes do not establish executing PCs or data formats, and linear scan
candidates can be data; original control flow and ownership require individual
proof. Other scenes/readers, Linux, USA and native Windows remain open. No
performance comparison was made.


## E407 Star Cross sample reconciliation

Protected static join `e407-star-cross-bank-sample-residual-lgz7_nd7` compares
the retained E403 EUR address inventory with E292. All 2,116 E405 bank bytes
were among the 7,359 previously unowned sampled bytes and are now declared by
the compact catalog; the other resource metadata remains unchanged. The
sample has 5,243 remaining unowned bytes in 33 intervals. The largest are
`affc70..b00d96` (4,390 bytes) and `b00d9c..b00fd6` (570 bytes).

This is an address-only selected-reader sample, not proof of instruction sites,
source extents, other scenes or full-ROM coverage. Linux, USA and native
Windows remain open. No performance comparison was made.


## E410 Star Cross sample reconciliation

Protected static join `e410-star-cross-lz-sample-residual-0_a1p9o7` compares
the retained E403 EUR address inventory with E293. All 4,960 E408 LZ bytes
were among the 5,243 E407 unowned sampled bytes and are now declared by the
observed LZ resource; the other resource metadata remains unchanged. The sample
has 283 unowned bytes in 31 intervals, including 60 bytes at
`14d69c..14d6d8`, scattered bytes around `39dd98..39de38`, and 19 potential
Thumb literal words identified by the earlier bounded bit-pattern scan.

This address-only selected-reader sample and its candidate opcodes do not
prove instruction sites, data extents, other scenes or full-ROM coverage.
Linux, USA and native Windows remain open. No performance comparison was made.


## E411 Star Cross literal source plan

Wine disassembly and protected source plan
`e411-star-cross-literal-source-plan-wine-2u_db_rz` authenticate 19 original
EUR PC-relative Thumb load instructions and their corresponding 19 distinct
words. All 76 source bytes occur in the E403 sample and have no E293 owner.
Four bounded code windows and exact opcodes, values and source hashes are
retained in the plan; integrating these words would leave 207 E410 sampled
bytes outside declared sources. Wine drained with the local Bluetooth overlay.

These are authenticated static instruction/word relationships, not proof that
each instruction executed in the retained scene. Other readers, source-save
history, consumers and scenes remain open. Runtime publication and selected
altered-source checks were subsequently completed in E412 below. Linux, USA
and native Windows remain pending. No performance comparison was made.


## E412 Star Cross literal runtime

EUR extractor E294/decoder D293 extends `engine.thumb-pc-literals` with the
19 E411 words and their original PC-relative readers. The existing bus owner
serves the 76 source bytes; guest reads outside the authenticated projection
keep their previous routing. The prior literal representation and 147 other
resources remain unchanged.

Wine `e412-star-cross-literals-readers-wine-2_yy7brj` verifies source and
reader identities, prior representations, forged-file rejection, altered-source
mirrors and mixed boundaries, missing-dataset snapshot ownership and 38
original-interpreter/generated-reader complete-state/clock comparisons. Wine
`e412-star-cross-literals-scene-wine-lhr7hg1i` compares the 480 recorded
Star Cross frames while all 19 selected words are altered in the managed ROM.
Complete state, guest clock, central/wide video, OAM, dispatch and five
nonempty audio streams match at every frame; the resource serves all 19 words.
This demonstrates word consumption, not exact scene instruction attribution.

Production CLI `e412-star-cross-literals-cli-wine-da0uz463` creates, reuses
and verifies 149 files matching the tested E294 generation. All Wine servers
drained. Other readers/callers and scenes, earlier save history, real BIOS,
Linux, USA and native Windows remain open. No performance comparison was made.


## E413 Star Cross sample reconciliation

Protected static join `e413-star-cross-literals-sample-residual-6zwywcjb`
compares the retained E403 EUR address inventory with E294. All 76 E411 literal
bytes were among the 283 E410 unowned sampled bytes and are now declared by
the Thumb literal resource; the other resource metadata remains unchanged.
The sample has 207 unowned bytes in 22 intervals. They include 60 bytes at
`14d69c..14d6d8`, scattered bytes near `39dd98..39de38`, ten bytes at
`af2040..af204a` and four bytes preceding the E409 LZ streams. No E404
candidate Thumb literal words remain in this sample.

This address-only selected-reader sample does not establish instruction sites,
source extents, other scenes or full-ROM coverage. Linux, USA and native
Windows remain open. No performance comparison was made.


## E414 Star Cross copied tables source plan

Protected source plan `e414-star-cross-copied-tables-source-plan-3j15jgkp`
authenticates four contiguous EUR spans `14d69c..14d6d8`, totaling 60 bytes
observed in E403 without an E294 owner. Original Thumb readers reference
separately owned literal roots at `0eeb58`, `0eeca4`, `0eeca8` and `0eecac`.
The original code copies two 16-byte halfword groups directly to the stack,
then copies four bytes and twelve halfwords through the existing `147dc0`
helper. The code uses signed halfword coordinates from the first group and
compares the twelve copied halfwords on the later branch. The adjacent
`14d6d8..14d6e0` Soil Evidence table already has a separate owner.

The sampled addresses and static instructions do not attribute the exact
scene branch or prove all consumers and guest index domains. E415 below
subsequently integrates the selected sources and validates their Wine paths.
Linux, USA and native Windows remain open. No performance comparison was made.


## E415 Star Cross copied tables runtime

EUR extractor E295/decoder D294 extends the existing
`engine.copied-halfword-lookups` resource with the four E414 source groups,
represented as two signed-halfword groups, four bytes and twelve comparison
halfwords. Its existing guest bus path serves the 60 source bytes; accesses
outside the authenticated span retain their previous routing. The adjacent
Soil Evidence table, prior representation and 147 other resources remain
unchanged.

Wine `e415-star-cross-copied-tables-readers-wine-je7pmhcs` verifies source
and root identities, coherent edited-file rejection, altered-source mirrors
and mixed boundaries including the adjacent owner, snapshot ownership, and
eight original-interpreter/generated direct/helper copy prefix comparisons
with equal complete state and guest clock. Wine
`e415-star-cross-copied-tables-scene-wine-zui7wer8` compares 480 recorded
Star Cross frames while all four groups are altered in the managed ROM.
Complete state, guest clock, central/wide video, OAM, dispatch and five
nonempty audio streams match at every frame; all four groups are served.

Wine production CLI `e415-star-cross-copied-tables-cli-wine-u68dxkft`
creates, reuses and verifies 149 files matching the tested E295 generation.
All Wine servers drained. Source use does not establish exact scene PCs, all
callers, guest index domains, other scenes, earlier save history or real BIOS.
Linux, USA and native Windows remain open; no performance comparison was made.


## E416 Star Cross sample reconciliation

Protected static join `e416-star-cross-copied-tables-sample-residual-hplibhvt`
compares the retained E403 EUR address inventory with E295. All 60 E414 copied
table bytes were among the 207 E413 unowned sampled bytes and are now declared
by the copied-table resource; other resource metadata remains unchanged. The
sample has 147 unowned bytes in 21 intervals, predominantly scattered across
`39dd98..39de38`, plus ten bytes at `af2040..af204a` and four bytes preceding
the E409 LZ streams.

This address-only selected-reader sample does not establish instruction sites,
source extents, other scenes or full-ROM coverage. Linux, USA and native
Windows remain open. No performance comparison was made.


## E417 Star Cross indirect records source plan

Protected plan `e417-star-cross-indirect-records-source-plan-8evjfzvi`
authenticates three EUR eight-byte indirect records at `39de20..39de38`,
without E295 owners. Twenty-one of their 24 bytes are in the E403 sample.
The original caller at `080eedc8` loads their separately owned literal root,
passes count three and calls the already authenticated `080e389c` consumer.
Each record reconstructs the existing indirect-record field layout.

This static pointer/count relationship and address sample do not prove exact
scene instruction attribution or establish other consumers and guest
pointer/count domains. E418 below subsequently integrates these records and
validates the selected Wine paths. Linux, USA and native Windows remain open.
No performance comparison was made.


## E418 Star Cross indirect records runtime

EUR extractor E296/decoder D295 extends the existing
`engine.soil-indirect-records` resource with the three E417 records at
`39de20..39de38`. Their separately owned literal root and the guest's original
pointer/count arithmetic remain unchanged; accesses outside represented records
retain their previous routing. Earlier records, their observation mask and 147
other resources remain unchanged.

Wine `e418-star-cross-indirect-readers-wine-k4a2e7il` checks source/root
authentication, prior representation, coherent edited-file rejection,
altered-source mirrors and mixed boundaries, snapshot ownership, 44 original
interpreter/generated reader, fallback and caller-prefix cases, with equal
complete state and guest clock. Wine
`e418-star-cross-indirect-scene-wine-a47ngrrg` compares 480 recorded Star
Cross frames while all three records are altered in the managed ROM. Complete
state, guest clock, central/wide video, OAM, dispatch and five nonempty audio
streams match at every frame; all three records are served.

Wine production CLI `e418-star-cross-indirect-cli-wine-_c1pwxal` creates,
reuses and verifies 149 files matching the tested E296 generation. All Wine
servers drained. Record use does not establish exact scene PCs, all callers,
guest pointer/count domains, other scenes, earlier save history or real BIOS.
Linux, USA and native Windows remain open; no performance comparison was made.


## E419 Star Cross sample reconciliation

Protected static join `e419-star-cross-indirect-sample-residual-ux3w7c0x`
compares the retained E403 EUR address inventory with E296. Twenty-one of the
24 E417 indirect-record bytes were among the 147 E416 unowned sampled bytes
and are now declared by the indirect-record resource; other resource metadata
remains unchanged. The sample has 126 unowned bytes, predominantly near
`39dd98..39de20`, plus ten bytes at `af2040..af204a` and four bytes preceding
the E409 LZ streams.

This address-only selected-reader sample does not establish instruction sites,
source extents, other scenes or full-ROM coverage. Linux, USA and native
Windows remain open. No performance comparison was made.


## E420 Star Cross compact descriptor source plan

Protected plan `e420-star-cross-descriptors-source-plan-5inrm3__` authenticates
two EUR 24-byte compact descriptors at `39ddf0..39de20` without E296 owners.
Forty of their 48 bytes occur in the E403 address sample. Both point to the
existing bank `6bacd8` and use represented selectors 0 and 1. The original
caller at `080eedfa` passes count two, loads the separately owned root literal
at `0eee24` and calls `080d9ba8`; the retained original callback disassembly
confirms the existing descriptor stride and field layout. The authenticated
descriptor SHA-256 values are `7c4c8eeeefd39bb7772a6b0eb865f754d9239a57947817c6eb4863a62cc68549`
and `8368b81372d8270970d547faf30d13ec223dd3d45d44553f53679600fc97c9fa`.

Static code and sampled addresses do not prove that the selected scene executed
this exact caller or establish other consumers or guest index domains. E421
integrates the two descriptors and validates selected Wine paths; other scenes,
Linux, USA and native Windows remain open. No performance comparison was made.


## E422 Star Cross sample reconciliation

Protected static join `e422-star-cross-descriptors-sample-residual-8m719re6`
compares the retained E403 EUR address inventory with E297. Forty of the
48 E420 descriptor bytes were among the 126 E419 unowned sampled bytes
and are now declared by the compact catalog; other resource metadata
remains unchanged. The sample has 86 unowned bytes in 12 intervals, predominantly near
`39dd98..39ddf0`, plus ten bytes at `af2040..af204a` and four bytes preceding
the E409 LZ streams.

This address-only selected-reader sample does not establish instruction sites,
source extents, other scenes or full-ROM coverage. Linux, USA and native
Windows remain open. No performance comparison was made.


## E423 Star Cross row-list source plan

Protected plan `e423-star-cross-rows-source-plan-1u3y2pqu` authenticates four
EUR 12-byte rows at `39ddc0..39ddf0`, without E297 owners. Forty-two of the
48 bytes occur in the E403 sample. The original caller at `080eed4e` loads
a separately owned literal at `0eed64`, passes the row-list base and count four
to `080e3f08`, and the existing list task reads with a 12-byte stride. The
rows point to four nearby source records at `39dda0..39ddc0`, whose ownership
and linked data require separate integration. Their first bytes are 2, 3, 3
and 4; other fields retain their original guest interpretation.

This static caller and address sample do not prove exact scene PCs, all
consumers, guest index bounds or the extents of linked sources. Runtime
integration, other scenes, Linux, USA and native Windows remain open. No
performance comparison was made.


## E424 Star Cross linked-source plan

Protected plan `e424-star-cross-linked-source-plan-mrt9gl4w` authenticates 58
unowned EUR bytes linked to the E423 rows: one rectangle entry and its list
header, two eight-byte LZ descriptors and their four-byte wrappers, one direct
descriptor and ten bytes of a mode-7 header and first run. Forty-four are in
the E403 sample. The rectangle points to the already owned `ae6e14` rectangle;
the wrappers lead to E409 LZ streams at `affc70` and `b00d9c`. The original
list, type-2 and type-4 dispatch and nested readers are authenticated by the
retained Wine disassembly. Together with the 42 sampled E423 row bytes, these
selected sources account for the remaining 86 E422 sampled bytes. This plan
does not install source ownership.

Only the first sampled mode-7 run is represented in the plan; its total source
extent is not established. Static dispatch and sampled addresses do not prove
exact scene PCs, all callers, guest domains or other scenes. Runtime
integration, Linux, USA and native Windows remain open. No performance
comparison was made.


## E425 Star Cross linked-source runtime

EUR extractor E298/decoder D297 extends the existing tilemap, descriptor and
row resources with the 106 bytes planned in E423/E424, while keeping the
previous rectangle and LZ stream owners and ROM fallback beyond the selected
mode-7 prefix. No full guest index or source extent is implied.

Wine `e425-star-cross-transfer-readers-wine-wqepgepj` passes source and
dependency checks, previous fields and 145 other resources, coherent-edit
rejection, altered-source mirrors and mixed boundaries, retained snapshot
ownership and 44 original interpreter/generated reader and caller-prefix
cases. The next mode-7 byte remains ROM-backed. Wine
`e425-star-cross-transfer-scene-wine-st36aju0` matches state, guest clock,
central/wide video, OAM, dispatch and five nonempty audio streams across 480
recorded frames with all 106 backing bytes altered. All four rows and every
selected descriptor, wrapper and tilemap source group are observed served.
Both Wine servers drained.

Wine production CLI `e425-star-cross-transfer-cli-wine-zym9tqj7` creates,
reuses and verifies all 149 files matching the tested E298 dataset. The server
drained. Other readers, scenes, real BIOS, Linux, USA and native Windows remain
open. No performance comparison was made.


## E444 Star Cross sample reconciliation

Wine `e444-eur-star-cross-sample-ownership-wine-qv1g8xda` compares the complete
retained E403 selected-reader address inventory with the authenticated EUR
E297, E298 and E301 manifests. E297 leaves the same 86 bytes unowned as E422.
E298 declares owners for all 243,297 distinct sampled bytes. The 86 newly
owned bytes consist of 42 E423 row bytes and 44 E424 linked-source bytes.
Both E298 and E301 carry the matching source hashes and expected owners for
all 106 bytes in those plans. E301 preserves every E298 ownership assignment
in this sample, including the 903 bytes with multiple existing owners.

The Wine server drained and the protected inputs passed. This check reuses the
recorded 480-frame sample and opens only metadata. The E425 altered-source
comparison remains the proof of runtime use for its selected paths. Exact
executing PCs in E403, other readers and scenes, later mode-7 runs, real BIOS,
Linux, USA and native Windows remain open. No performance comparison was made.


## E445 European mode-7 source bounds

The E301 manifest declares 34 tilemap rows. Wine
`e445-eur-mode7-stream-extents-wine-u8bkowmo` authenticates those rows and
follows each of the ten type-4 rows through its original descriptor. Eight
lead to the existing mode-4 streams. The other two lead to the Soil Evidence
and Star Cross mode-7 streams.

The mode-7 arm at `080da758` passes the same stream start after the six-byte
header to four calls of `080da2a8`. Each call resets the stream cursor and
visits at most width × height cells. The first pass starts at coordinate zero
on both axes and fits inside the writer's 32 × 32 bounds for these streams.
Wine disassembly `x6mub2vs` authenticates the helper and its call to the
original per-cell reader at `080da14c`. The low 14 bits of each stream's first
ascending control word equal its full cell count. The reader advances past
that four-byte run after the final cell and makes no extra token read.

The complete selected intervals are `af270c..af2716` for Soil Evidence
(four cells) and `af2040..af204a` for Star Cross (256 cells). E301 declares
`graphics.observed-tilemap` as the sole owner of all 20 bytes. The two bytes
at `af2716..af2718` are beyond the mode-7 read extent and still have no
declared owner. Their purpose remains open. E445 bounds all mode-7 streams
referenced by the declared E301 rows. Additional roots, other readers and
platforms remain open. Both Wine servers drained. No replay or performance
comparison was made.


## E446 European tilemap transfer directories

Wine `e446-eur-tilemap-list-directories-wine-qs06dz3h` checks directory
metadata derived from the original EUR constructor and its lower-level list
entry. The signed Thumb `BL` census finds 224 encodings targeting `080e3f08`
and 30 targeting `080db580`. The 30 lower-level sites match the retained
E202 census. Local argument analysis resolves the base and positive row
count for 202 and 26 sites respectively, using up to 64 bytes before each
call. It recovers the E202 conditional count of zero or four and the E423
Star Cross count of four. The Wine run checks representative caller windows
at both entry points and compares the resulting intervals with E301.

The 230 distinct base/count pairs cover 937 distinct 12-byte records. E301
declares owners for 168 of the 11,244 row bytes. The 100 type-4 records point
through descriptors to 52 distinct tilemap starts. Within those records,
28 references select nine mode-7 streams. The header dimensions bound each
stream's cell count, and the original reader's ascending, repeated and
literal token sizes determine its encoded endpoint. The nine intervals
total 162 bytes. Ten belong to the E445 Star Cross stream and the other
152 bytes in eight streams have no E301 owner. The E445 Soil stream has
separate caller evidence.

The census establishes local argument witnesses and source geometry. Live
entry paths, the 26 unresolved direct setups, additional indirect callers,
and integration remain open. E447 below verifies the selected stream extents
for the other modes. Protected inputs passed and the Wine server drained.
Linux, USA and native Windows remain pending. No gameplay replay or
performance comparison was made.


## E447 European type-4 tilemap stream bounds

Wine `e447-eur-tilemap-dispatch-disassembly-wine-9xsy1p68` fills the missing
window in the original mode dispatcher. All 11 distinct call instructions
across the seven arms target `080da2a8`, which uses the same halfword reader
at `080da14c`. The first traversal starts at coordinate zero on both axes.
The 61 selected streams fit the writer's 32 × 32 bounds. Repeated and flipped
traversals reset the cursor to the same position after the six-byte header.
The low 14 control bits give the cell count. Ascending and repeated tokens
carry one encoded halfword, and a literal token carries a halfword per cell.
Every selected traversal ends at a token boundary after width × height cells.

Wine `e447-eur-type4-tilemap-extents-wine-78zzrsqb` joins 52 E446 roots and ten
E445 roots with one in common. The 109 distinct rows reference 107 distinct
descriptors. The 61 disjoint stream intervals contain 7,380 bytes. E301
declares owners for ten complete streams totaling 844 bytes. The other 51
streams account for 6,536 unowned bytes. Mode 1 accounts for 5,992 of them.
The E445 and E446 mode-7 lengths and hashes agree.

E301 owns 81 of 856 descriptor bytes. Ten descriptors have complete coverage.
For the descriptor at `3aba84`, `graphics.compact-primary-alias` already owns
its last byte at `3aba8b`. The other seven bytes of that descriptor and 96
complete descriptors remain outside declared coverage.

Both Wine servers drained and protected inputs passed. The evidence covers
the E445 and E446 type-4 projections in modes 1, 2 and 4 through 7. E457
extends the mode inventory to the complete contiguous bank. Unresolved
callers, ordinary use, integration, Linux, USA and native Windows remain
open. [E457 stream bank](#e457-european-tilemap-stream-bank)


## E448 European type-3 transfer source bounds

Wine `e448-eur-type3-transfer-extents-wine-mxjdpoph` follows all 617 type-3
rows from the E446 directories and all 21 declared E301 type-3 rows. Ten
rows occur in both inputs. The resulting 628 distinct rows select 627
eight-byte descriptors and 275 four-byte wrappers. The row pointer leads to
the descriptor, whose first word selects the wrapper. Multiple descriptors
can share a wrapper and its payload.

The original routine at `080db09c` uses wrapper flag `0x02` to choose the
payload reader at wrapper plus four. All 262 compressed paths lead to the
original SWI `0x11` wrapper. The type-`0x10` header provides the output count.
Walking the flag bits and token sizes, with valid history distances, proves
226,504 encoded bytes. Each stream reaches its exact declared size. All
20 existing E301 LZ77 source lengths and hashes agree with the audit.

The other 13 paths use the raw payload pointer. The original wrapper kind
and flag `0x01` determine the byte length from the row's halfword at offset
two. The selected lengths are even. The transfer loop divides each chunk
length by two for the halfword queue, advances the source pointer and
decrements the remaining bytes. The Wine disassembly authenticates the
queue's halfword control. The 13 bounded views contain 4,960 distinct bytes.

Across rows, descriptors, wrappers and payloads, E301 declares 15,820 of
245,116 distinct bytes. All 242 additional compressed streams and all 13 raw
views have no declared owner. Preserve the existing four-byte owner
`graphics.compact-primary-alias` at `3abd78..3abd7c` inside descriptor
`3abd74` when extending coverage.

The Wine server drained and protected inputs passed. The audit establishes
source extents and hashes for the selected references. Wider caller coverage,
live use, runtime integration, real BIOS, Linux, USA and native Windows
remain open. No payload output, gameplay replay or performance comparison
was produced.


## E449 European type-2 rectangle transfer bounds

The 212 E446 type-2 rows include all three rows already declared by
`graphics.observed-tilemap-row`. The independently authenticated E320 Firaga
program supplies one additional row. Together the 213 rows select 212 distinct
list headers and 429 distinct eight-byte entries. The type-2 arm of
`080db374` reads the header count and visits the entries at an eight-byte
stride. The original rectangle reader at `080d80e0` adds the two-byte header
to the source address, skips the starting row specified by the entry and
copies the requested number of rows, using twice the source width as the byte
stride. The original copy helper handles whole words and trailing bytes.

The 429 entries select 217 unique windows across 162 rectangles. The union
of windows covers all rows in 138 rectangles and a subset in 24. Another
1,220 bytes are within the nominal rectangles outside the selected row
windows. The audit covers 3,066 rectangle bytes including the two-byte
headers. The selected windows at `ae6db8`
cover two of its three rows. E301 owns the header and first row, and the
six-byte second row is unowned in E301.

The Wine CLI output from `2hz2xkoy` reports 356 owned bytes out of 10,750
distinct selected bytes across rows, headers, entries and rectangles. The
postprocessing script stopped after Wine had written its report and drained
because it mixed list and tuple intervals. Protected recovery `bftzb469`
normalizes the intervals and verifies the retained report against the exact
regenerated plan and E301 manifest. It confirms that 2,838 of the 3,066
selected rectangle bytes remain unowned.

Live use of the additional paths, further callers, runtime integration,
Linux, USA and native Windows remain open. No payload output, gameplay replay
or performance measurement was produced.


## E450 European type-5 byte-stream bounds

The eight E446 type-5 rows select eight descriptors and seven distinct
streams. The type-5 arm of `080db2a8` loads the pointer and halfword parameter
from the first six descriptor bytes. The initializer at `080da81c` passes the
stream at header offset six to the byte writer at `080da460`. Five streams use
initializer mode 1 and two use mode 4. The first traversal starts at zero on
both axes and covers the full width and height of each selected stream.

The byte reader at `080da200` uses a halfword control word and byte values.
Its token advance rounds the encoded values up to an even byte count. All
seven streams finish at a token boundary. Their occupied intervals total
1,730 bytes. The reader accesses 1,648 of those bytes and skips 14 header
positions and 68 token alignment bytes. The two bytes after each
descriptor's six accessed bytes remain outside this projection.

Wine `7h8krrgg` authenticates the initializer window. Wine `uhx_ek2y` checks
the byte reader and compares the source plan with E301. All 1,792 selected
bytes across the rows, descriptor fields and stream reads are unowned. The
audit also checks that all 937 E446 rows appear in the retained source
inventories for types 2 through 5. Both Wine servers drained and protected
inputs passed. E451 examines the remaining 26 direct setups. Indirect
callers, live use, runtime integration, Linux, USA and native Windows remain
open. The checks covered source metadata and performed no gameplay replay or
performance comparison.

## E451 European tilemap transfer call setup

Wine `hniuxufm` verifies the original instructions at all 26 call sites left
open by E446. Wine `fdhqvpxi` verifies the additional producer windows for
the two auxiliary callbacks and the stack-carried count at `1087fa`.
The task descriptor catalogs associate original callbacks with the task IDs
passed through constructor `080d7018`. The audit uses these IDs and the
original selector arithmetic to enumerate indexed lists, including both
branches where a helper changes the index. It recovers fixed counts of two
from `r8` at `0faa82` and stack offset `+0x4c` at `1087fa` on their
positive-count paths. The lower-level call at `0e3a68` forwards the
constructor's pointer and count when both are nonzero.

For auxiliary task 345, the relevant primary descriptor metadata and its
original remapping yield the range 0–22 under the catalog and initialization
conditions. Twenty slots have nonzero pointers. The audit preserves aliases
and verifies the previously identified Firaga list at `39a758`. The
separate Firaga program at `39cb9c` is one of three lists at `0ebb00`.
At `0e7352`, the two pointer tables and two count tables establish two
nine-slot physical projections. The selector loaded through `080d1e64`
comes from the live byte at offset `+0xe6` of the selected unit. E453
identifies the direct setter calls and bounds the separate registration
route. [E453 auxiliary selector](#e453-european-tilemap-auxiliary-selector)

The 24 bounded producer projections and the conditional auxiliary projection
add 105 base/count pairs and 324 distinct 12-byte rows to E446. The combined
inventory has 335 pairs and 1,261 distinct rows. The additions include 82
type-2, 207 type-3, 34 type-4 and one type-5 row. Thirty-six of the added
rows depend on the physical selector interval at `0e7352`.

Wine `qrfmjqvi` matches the exact E301 manifest ownership against an
independent interval join. E301 owns 264 of the 3,888 added row bytes and
35 of the 730 selected pointer, count and literal bytes. All three Wine
servers drained and protected inputs passed. The projections condition on
the catalog IDs and retained initializer fields. Linked source extents,
other producers and indirect paths, ordinary gameplay, runtime integration,
Linux, USA and native Windows remain open. The audit retained metadata and
performed no gameplay replay or performance comparison.

## E452 European tilemap linked source extents

E452 follows all 324 added E451 rows through the source-reader contracts
verified by E447 through E450. The type-2 path selects 118 entries through
49 headers, with 86 distinct row windows in 65 rectangles. The type-3 path
selects 176 descriptors, 148 LZ77 stream extents and four raw copy roots.
The type-4 path selects 28 descriptors and 26 halfword streams across
modes 1, 4 and 7. Six type-4 roots use mode 7, of which five already have
E447 bounds. The type-5 row points to an E450 byte stream. Common source
addresses reuse their existing bounds, and the ownership comparison counts
each physical byte once.

Wine `69eiy13_` verifies 152,473 distinct selected bytes against the exact
E301 manifest. Of these, 44,178 occur in the preceding source inventories
and 108,295 extend their coverage. E301 declares owners for 14,098 of the
selected bytes. The 36 rows conditioned on the auxiliary selector add
8,008 bytes disjoint from the other E452 selected reads. The unit records
the exact linked intervals and the two selector scopes in `extents.json`.
Protected inputs passed and Wine drained.

The E446 and E451 row sets now have reader bounds for all four observed row
types within their declared scopes. Mode 3, live auxiliary selector bounds,
other producers and indirect callers, ordinary gameplay, runtime integration,
Linux, USA and native Windows remain open. The audit retained metadata and
performed no gameplay replay or performance comparison.

## E453 European tilemap auxiliary selector

The original primary descriptor catalog maps IDs 199–207 to callback
`08104968`. Its initializer saves parameter `+0x14` and forwards it in the
message that registers auxiliary callback `080e7298` at `081049d0`.
The adapter at `08005bd0` forwards the message to `080059b8`, which copies
it and dispatches the initialized message with the same ID. The auxiliary
initializer stores `(ID + 0x39) & 0xff` in context byte `+7`, giving
indices 0–8. At `080e7352`, helper `080df798` selects the first table
for object byte `+0x1f` values 0 and 3 and the second table for other
values. This verifies every slot in both tables for the registered
ID domain, conditional on the mode and on the retained initializer
fields. [Task catalog source contract](rom_data_tasks.md#source-extent-and-original-readers)

The primary task 360 path instead dereferences its parameter at `+8` and
reads the resulting object's live byte at `+0xe6`. The setter at
`080d1edc` writes the low byte of its argument. Two direct callers pass
zero, two copy the byte at `+0x104` from their source object, and one
adds `0x36` to a context byte before narrowing. The writer census concerns
signed direct Thumb calls to that setter. E454 traces a dispatcher handle
to this read. The live input domains and writes through aliases remain open.
The secondary descriptor lookup also addresses the physical task 360
descriptor when given ID 361. E456 verifies the three direct callers and
their bounded sources. [E456 secondary caller audit](#e456-european-secondary-task-361-caller-audit)

Wine `w4fmebht` authenticates the accessors, direct setter sources and
registration call. Wine `64e2i50z` authenticates message forwarding and
the mode branch, reconciles all 18 base/count pairs with E451, and matches
the same 36 rows from the 8,008-byte E452 projection. Both servers drained
and protected inputs passed. Ordinary execution of the two modes,
runtime integration, Linux, USA and native Windows remain open.

## E454 European auxiliary task 360 actor source

The dispatcher at `08095ea0` receives the context read through
`0200f4a8`. Dispatcher state 113 passes context field `+4` to
`080ac23c`. The queue initializer at `080ab50c` stores that handle in
the first word of the new context and queues state 4. Dispatcher state 114
polls the resulting queue through `080ac22c`. In queue state 4, the
branch at `080ab5e2` loads task ID 360 and passes the saved handle as the
third argument to `080d7018`. The constructor stores it in parameter
`+8`. The initializer at `080e7298` reads the first word of the handle
and takes byte `+0xe6` from the resulting object.

The dispatcher can write the result of the `0809e150` template-byte lookup
to context field `+4`, clear the field, or store the result of
`080a2f68`. The first lookup compares a unit byte at `+0x104`. These
windows establish the handle passed to task 360 for the verified states.
The writer and value of that unit's byte `+0xe6` at the time of the
read still need proof. Establishing the bound requires distinguishing
the setter calls on the selected handle from writes to other units.

Wine `1t4505gy` authenticates ten instruction windows against the
European ROM and verifies ten call links using the retained E453
instructions. The server drained and protected inputs passed. Ordinary
reachability, the secondary ID 361 alias, runtime integration, Linux,
USA and native Windows remain open.

## E455 European auxiliary task 360 entry condition

In dispatcher state 37, a nonzero byte at context `+0xc8` selects
command reader `080266b4`. Return value 11 selects branch `08097878`,
which queues states 19 and 113 and sets state 33. E454 establishes the
subsequent state-113 allocation and state-114 polling path to auxiliary
task 360. The 17-entry command-reader switch uses a separate work pointer
at `0200f438` and selects the command from its mode and fields.

Wine `z5naxdzl` authenticates six original instruction windows. The
same unit checks the exact identities and RAM fields of the 345 retained
European snapshots. Forty-nine contain dispatcher state 37 with a
nonzero gate. Each supplies zero at byte `+0xe6` of the unit reached
through field `+4`. Their sampled command-reader inputs project commands
0, 8, 9 and 65,542. The retained cohort contains no command-11 entry
candidate and no state 113 or 114. Wine drained and protected inputs passed.

A new ordinary trace needs a source-backed input yielding command 11
with the relevant unit selected, or a verified later checkpoint. The
snapshot values establish the selector at those saved boundaries. The
writer, value at the task-360 read, secondary ID 361 reachability,
runtime integration, Linux, USA and native Windows remain open.

## E456 European secondary task 361 caller audit

The exact ROM contains three signed direct Thumb calls to secondary
constructor `080d7328`. At `08124f80`, event opcode 0 forwards the ID
halfword loaded from its 16-byte event record. Across the 75 declared
records, that opcode supplies 19 references to 15 distinct secondary IDs,
with extrema 368 and 413. The other two calls, at `08128d0e` and
`08128ed4`, share the secondary ID computed by the four-byte effect
selector entry. Its 51 declared rows project to IDs 363–413.
The original direct frontdoor for that entry starts with a byte
parameter. Across all 256 byte values, including the wrapping zero
case, the resulting IDs range from 362 to 617. ID 361 would require a
raw selector of 65,534. [Directory source contract](rom_data_events.md#independent-source-boundaries)

Wine `50vko1z1` authenticates six caller windows and joins both complete
declared directories to the original source spans. The 16-bit descriptor
address sweep confirms that ID 361 is the single physical selector for
callback `080e7299` through this constructor. The three direct call sites
exclude it within the verified bounds. Wine drained and protected inputs
passed. Indirect entries, live event indices outside the declared
directory, ordinary gameplay, Linux, USA and native Windows remain open.

## E457 European tilemap stream bank

E457 joins the E447, E450 and E452 source extents into one contiguous
interval at `af1818..af3ea0`. The 71 halfword streams and seven byte
streams occupy 9,722 bytes. Advancing from each verified end to the next
four-byte boundary accounts for the other 142 bytes. E452 independently
identifies `af3ea0` as the next type-3 wrapper, linked from row
`39cb54` through descriptor `39cb0c`.

Every one of the 302 aligned ROM words pointing into this interval targets
an exact stream start. Of those, 136 are verified descriptor words.
A further 153 have PC-relative Thumb literal load candidates, and 13
required classification in [E458](#e458-european-tilemap-bank-consumers).
The original halfword decoder dispatches mode 3 to
`080da608`. Of the 78 source headers in this bank, 41 use mode 1, two
use mode 2, 17 use mode 4, six use mode 5, one uses mode 6 and 11 use
mode 7. Other banks, constructed headers and indirect pointers remain open.

Wine `58n8155h` checks the complete interval and reference words against
the exact E301 ownership. E301 owns 844 of 9,864 bank bytes, with 8,878
unowned encoded bytes and 142 unowned alignment bytes. It owns 80 of the
1,208 reference bytes. The retained `bank-index.json` records starts,
ends, formats, hashes and reference addresses. The server drained and
protected inputs passed. Live use, integration, Linux, USA and native
Windows remain open.

## E458 European tilemap bank consumers

E458 traces 153 original literal loads to six functions at `080da940`,
`080dad30`, `080dadd8`, `080daebc`, `080daf78` and `080dafb4`. Each bounded
argument path passes the stream base in `r0`. These functions read origin
bytes `+2` and `+3` from that argument. The apparent load at `081099ec`
is part of a literal pool. The original load at `0810999a` uses the full
word at that address, and executable control flow skips the pool.

Three indexed lookup sites at `080e843a`, `080fef2e` and `081091e0` also
pass the selected pointer to `080dad30`. Their 12 physical bank references
form groups of two, two and eight at `39bcbc`, `3a1c58` and `3a4d84`. The
last group matches the complete E197 table. Together the literal loads
and indexed cells select 76 distinct roots. The index domains in live
execution remain open.

The remaining reference word at `39a6f0` is the pointer in the descriptor
for type-4 row `39a71c`. Two alternate paths into the constructor call
at `080e6168` supply the same two-row list at `39a710`. The second row
selects `af2a84`, which is already in the verified bank. This accounts
for all 302 aligned reference words as 137 descriptor words, 153 literal
words and 12 indexed cells.

Wine `zy0fdg0i` authenticates the six selected instruction windows and
checks E301 ownership. The manifest owns 20 of 152 origin bytes, eight of
612 literal pointer bytes and 32 of 48 indexed pointer bytes. All 20 bytes
of the additional row and descriptor are unowned. The origin pairs are
within E457's stream extents. The server drained and protected inputs
passed. Runtime consumption and integration, other banks, ordinary use,
Linux, USA and native Windows remain open.

## E459 European tilemap origin caller census

The signed Thumb branch scan for the six E458 origin readers finds 162
direct call sites. E458 accounts for 153 literal-driven calls and three
indexed call sites. The other six occur at `080e9668`, `080e9e5a`,
`080eb306`, `080ecaee`, `081231d0` and `0812335c`. Each call reuses the
source pointer loaded for an earlier origin-reader call. Across both
calls the pointer stays in `r4` or `r5` before being copied to `r0`.

Wine `4us_dpy7` authenticates the six original instruction windows and
verifies every call against the E457/E458 indexes. The added calls reach
six previously verified starts. The bounded direct census therefore
still covers 76 of the 78 bank roots for origin reads. The server drained
and protected inputs passed. Indirect calls, live indexed inputs,
ordinary use, integration and platform checks remain open.

### Bank coverage contract

Follow the original list rows to each descriptor and group the results by
physical source pointer. Use the corresponding halfword or byte reader to
bound each stream. Keep the encoded extents, alignment positions and
consumer addresses distinct. Include origin bytes in the stream extent so
geometry references and transfer rows retain the same source identity.
Compare the union of stream ranges with E301 ownership before adding new
ranges. The first verified start is `af1818` and the independent following
wrapper proves the upper boundary at `af3ea0`. Extending the inventory
below `af1818` or into another bank requires an independently linked
source and its reader bounds.

## E460 European tilemap bank reads in recorded scenes

E460 joins six retained Wine address inventories to the E457 stream extents
and E458 pointer roles. It authenticates the capture hashes, the complete
selected-reader address union for each window and the successful original
state comparisons. The Llednar capture uses E215's verified recovery of
the drained Wine child after a scratch-retention error.

Star Cross reads all 10 bytes of each mode-7 stream at `af2040` and
`af270c`. Soil Evidence and Firaga also read all 10 bytes at `af270c`.
The address inventories contain separate bus byte reads for both origin
bytes at the observed mode-7 roots. Blue Screw reads 100 of the 102 bytes
in the mode-4 stream at `af2780`. The White Flowers and Llednar windows
have no selected reads in this bank or its verified reference words.

Wine `gnjd_0y0` checks that E301 declares an owner for all 140 distinct
sampled bytes. These consist of 120 encoded bytes, 12 descriptor pointer
bytes and eight literal pointer bytes. The samples have no reads of
the alignment intervals, byte streams, indexed pointer words or the
alternate row and descriptor at `39a71c` and `39a6f0`. The first join
`rxzd10hx` stopped before the CLI when the Python range merge received
both JSON lists and tuples. The shared interval helper now normalizes the
endpoints before sorting. The corrected join passed, all 42 protected
inputs passed and Wine drained.

These captures establish a positive read baseline for three already-owned
streams. Evidence for the remaining extents needs a source-backed segment
that reaches an unowned stream or reference. Attributing a read to a
specific instruction or to managed serving needs separate runtime evidence.
Linux, USA and native Windows remain open.

## E461 European tilemap indexed producers

The primary constructor at `080d7018` stores the task ID in message field
`+0x14` and addresses the callback catalog with ID minus one. E461 combines
this contract with three original initializer paths and their retained
selector bytes.

- In IDs 9 and 10, callback `080e8524` subtracts nine and forwards slots 0
  and 1 through the child creation message to `080e8354`. The child uses
  the retained byte to index `39bcbc` on its timer-expiry path. Both slots
  select the last type-4 row of their four-row E451 list and the same
  stream for its origin bytes.
- In IDs 130 and 133, callback `080fee6c` initializes the byte at `+2` to
  zero and changes it to one for ID 133. It reads `3a1c58` after the
  transfer wait when its continuation predicate succeeds. Both slots
  select the last type-4 row of their three-row E451 list and the same
  stream for its origin bytes.
- In IDs 239–246, callback `08109440` adds 17 and stores the result as a
  byte, giving slots 0–7. E202 verifies the corresponding two-row list
  for each slot. The same byte is forwarded to `08109194`, which indexes
  `3a4d84` and selects the stream of that list's type-4 row.

Wine `8vqao49y` authenticates five original reader and producer windows.
Its `primary_callback_ids` field records zero-based catalog row indices.
The final audit at `mkryi11b` applies the original one-based ID conversion,
reuses the E202 and E453 source windows, and joins all 12 slots to their
rows, descriptors and encoded extents. E301 covers the complete selected
eight-slot group at `3a4d84`. The two smaller groups have 140 unowned
bytes across four streams (44 bytes), four rows (48 bytes), four
descriptors (32 bytes) and four geometry pointers (16 bytes). Both Wine
servers drained and protected inputs passed.

These source domains assume the verified initializers and retained values
until the original read. The primary IDs provide targets for an ordinary
input or checkpoint. Positive execution of the four unowned routes,
external writes, other callers, managed reads and Linux, USA and native
Windows checks remain open.

## E462 European tilemap action routes

The 347 authenticated metadata28 records contain exactly one selector-32
source for each of the 12 E461 task IDs. Actions 8 and 9 select tasks 9
and 10. Actions 133 and 136 select tasks 130 and 133. Actions 243–250
select tasks 239–246. The join uses the original halfword at `+0x14`
of the action record.

Wine `igckgky_` verifies the default dispatch path from `080a9f34` to
`080aa0b0`. The dispatcher loads the action at context `+0x10` and
passes selector 32 to `080d07a4`. When context byte `+9` is neither 10
nor 11 and the override bits `0xc000` in context halfword `+4` are clear
at the relevant checks, it forwards the result as the task ID to
`080d7018`. The override branches fetch other action records.

The 23 declared descriptor banks contain 48 matching action references
among their 684 records. Sixteen are kind 1. In those records, actions
8 and 9 appear at bank 1 slots 65/66, bank 3 slots 8/9 and bank 4 slots
40/41. Actions 133 and 136 appear at bank 5 slots 12 and 15.
Protected inputs passed and Wine drained. The declared references give
specific learned-slot candidates to look for in an original EUR input
or checkpoint. Availability, live dispatcher gates, ordinary tilemap
reads, integration, Linux, USA and native Windows remain open.

## E463 European tilemap action checkpoint candidates

E463 checks the 345 distinct EUR snapshots in the retained E455 cohort.
It uses the original bank byte, learned bit at `unit+0x40+slot` and
the inclusive metadata52 ranges of the saved primary and secondary job
rows. The job-range check uses rows with direct property routing.
The eight kind-1 slots from E462 produce 112 candidate checkpoints.

- Tasks 9 and 10 have 181 matching unit instances each in 63 snapshots.
  IN-08 includes bank 1 unit slot 0 with the two learned action slots
  and job 7. IN-19 includes a bank 4 candidate for both actions.
- Task 130 has 49 matching unit instances in 49 snapshots. IN-13 includes
  bank 5 unit slot 3 with a learned slot 12 and job 37.
- Task 133 has 67 unit instances with the learned bit for slot 15 and
  no match in a direct range of the currently equipped jobs.

The initial validation admission refused the symbolic `debug_saves`
ancestor. The corrected run protects the resolved ordinary save files and
authenticates each input by its retained hash. Wine `8srv4b7k` verifies the
original learned-bit instructions. The retained Windows state loader
independently agrees on the 24 roster records in IN-08 at guest frame
3020. All 372 protected inputs passed and Wine drained.

The candidates establish saved availability prerequisites. A controlled
ordinary continuation still needs the acting unit, menu option, target,
dispatcher predicates and positive reads of the E461 ranges. Task 133
needs a checkpoint with a suitable active job or a source-backed job
change path. Linux, USA and native Windows remain open.

## E464 European Shell and Protect menu

The IN-08 deployment checkpoint at guest frame 5441 has unit slot 15
selected. E463 authenticates its bank 4 learned slots 40 and 41. Wine
`fi8j7kzm` uses the retained placement and battle input sequence to deploy
that selection. At frame 7465 the turn menu identifies Wanda as the acting
unit. The input changes only the normal controller keys.

Wine `qacqfcxk` opens Action. Wine `7yv1vg9p` opens White Magic from
that exact endpoint. At frame 7593, the abilities menu shows Shell and
Protect, each costing three MP. The final authenticated checkpoint
retains that menu with Cure selected. All three Wine servers drained and
protected inputs passed.

The menu order places Shell three rows and Protect four rows below Cure.
[E465](#e465-european-white-magic-auxiliary-transfer) observes both effects
from this checkpoint. Managed equivalence, integration, Linux, USA and
native Windows remain open.

## E465 European White Magic auxiliary transfer

Wine `dfjyb0as` selects Shell and Protect through normal inputs. Both
previews target Wanda, at frames 7737 and 7753. The first filtered Wine
trace `u9gno321` observes the two action records at `080d08a6` and the
descriptor metadata at `080d70a2`. This load belongs to primary task
345's special constructor branch. The four requested E461 stream and
reference groups have zero reads in each initial 240-frame capture.

The updated Wine capture `lxd_65mg` verifies the caller at `080aa140`
reading the literal for task 345. Shell and Protect pass IDs 9 and 10
as the metadata arguments. Both source halfwords are zero, and the
original callback uses slot 0 of the E451 list tables. It reads pointer
`39a818` and count `39a874`, selecting the type-3 row at `39a740` and
descriptor at `39a668`.

Both 300-frame captures read **253 distinct bytes outside E301's declared
ownership**, including all 229 bytes of the LZ77 stream at
`b0e074..b0e159`. The stream reads occur in SWI `0x11` scope at `08145254`,
at frame 7952 for Shell and 7968 for Protect. The reads cover the four-byte
list pointer, one count byte, 12 row bytes, five of eight descriptor bytes
and two of four wrapper bytes. The full E452 source plan covers 258 bytes
across these six groups.

The first probe builds exposed a cached Rust version mismatch and missing
thin LTO. Build `pcnezi7k` uses the matching installed Rust 1.94 and thin
LTO. The auxiliary capture's final qualifier compared list values with
tuple scopes and returned FAIL after the successful traces and server
drain. Protected reconciliation `g1qq66op` validates both trace hashes,
the scope tuples, exact observed byte counts and all 41 original input
guards against the pinned E301 manifest. It reuses the captured Wine data.

This verifies ordinary ROM-backed use for slot 0. E466 maps the broader
E451 bank and E468 verifies managed control metadata on the Shell path.
The E476 continuation below reaches the later direct E461 reads for both
actions from the same protected previews.

## E466 European task-345 transfer bank

The call at `080e61dc` obtains the pointer and count directory addresses
from literals at `0e61e8` and `0e61ec`. The bounded primary metadata
domain selects all 23 slots. Slots 16, 19 and 20 have null pointers and
zero counts. The 20 non-null selections reference 23 rows with aliases.
These resolve to 18 distinct type-3 rows, 16 descriptors and 15 wrapped
LZ77 inputs. The index preserves every selection, including the three
two-row lists, rows shared across selectors and distinct descriptors that
share a wrapper.

Wine `xg51sldn` authenticates the E451 and E452 source indexes against the
exact European ROM and the pinned E301 manifest. The existing
`rom-data audit-ownership` CLI and independent interval accounting agree
on all ranges and owner IDs. The bank covers **4,358 distinct bytes**,
including 3,831 bytes of LZ77 input. E301 already declares 155 bytes under
five owners. Keep those owners and use the 23 uncovered intervals
(4,203 bytes) when planning adoption. In particular, the literal owners at
`0e61e8..0e61f0` and the existing scoped-value owners for pointer/count
slots 2 and 4 overlap the bank.

The six E465 slot-zero intervals match the bank index exactly. Their
258 planned bytes are all uncovered, of which 253 are confirmed by both
retained ordinary casts. The unit drains Wine and preserves 26 inputs.
Its evidence contains offsets, links and ownership counts. E468 adopts
the control metadata. Managed LZ77 input reads, ordinary nonzero slots,
other callers, Linux, USA and native Windows remain open.

### Generate and audit the task-345 bank plan

```sh
recomp rom-data plan-tilemap-bank "$eur_rom" > task-345-plan.json
recomp rom-data audit-ownership "$manifest" task-345-plan.json > task-345-ownership.json
```

Use the exact European ROM and a manifest for that source. The first
command follows all 23 list slots from the authenticated table layout
and emits a schema-1 ownership plan on standard output. Every slot gets a
group, including slots with a null pointer and a zero count. A second set
of groups collects the source ranges by role across the bank. This keeps
each selector's dependencies visible while the audit's `distinct` totals
count repeated rows and shared streams once.

For each type-3 row, the planner checks its descriptor and wrapper pointers
within the ROM. The bounded LZ77 parser reads tokens to determine the exact
encoded input length. The first command needs only the ROM and its JSON
contains source identity and ranges. The second command reports declared
owners and uncovered intervals using the selected manifest.

E467 builds the Windows CLI in `wxkz7e3r`. Wine `xkte2drk` matches all
23 slot plans and the seven role groups to E466, reproduces the E301 owner
splits, rejects a same-size synthetic image and passes four LZ77 tests.
The Wine server drained. The scratch guard retained the synthetic
all-zero file with a `.gba` suffix after the checks passed. Recovery
`kppbgu_4` verifies the captured JSON and logs against the pinned E466
inputs and confirms all 23 input guards from the Wine run.

## E468 European task-345 managed transfer metadata

EUR e302-d301 adds `graphics.tilemap-transfer-directory` in
`tilemap-transfer-directory.json`. Its model authenticates the exact ROM
and checks the pointer and count directory addresses, all 23 selectors,
18 distinct rows, 16 descriptors and 15 wrapper links. The plan command
uses this same model. Null slots, shared rows and shared wrappers retain
their original links.

The runtime adds 461 previously uncovered control bytes across the full
bank. These comprise 84 pointer bytes, 21 count bytes, 192 row bytes,
112 descriptor bytes and 52 wrapper bytes. Source binding preserves the
earlier owners, including slot 2 and 4 directory entries. Reads across a
field boundary compose the new bytes with earlier managed values and
the surrounding ROM backing. The E302 manifest now covers 616 of the
4,358 bytes in the E466 plan. The 3,742 uncovered bytes belong to the
LZ77 inputs.

Wine `fpyu2e7m` binds the new metadata through the ordinary dataset
loader with 148 E301 resource files reused as read-only inputs. The
check rejects an inconsistent null slot and a coherent file and hash
edit. It covers all new bytes and mixed halfword and word boundaries
through the cartridge mirrors and native read API. The plan remains
identical to E467.

The retained IN-08 Shell input runs 300 interpreter frames with altered
metadata backing on the managed side. The two runs match guest clock
and audio frame by frame, and match final video, OAM and complete saved
state. The added owner supplies 44 bytes across pointer, count, row,
descriptor and wrapper reads, including repeated accesses. Wine drains
and the runner preserves the 28 protected inputs. The first Windows
build `3imvzd94` found a missed budget argument in an existing test.
Corrected build `xdx2mspz` passes.

The loader and interpreter checks use an incrementally staged dataset.
Ordinary nonzero selectors, managed LZ77 inputs, native execution, Linux
and USA remain open.

### E469 E302 publication and reuse

Wine `azu4reeg` exercises the managed preparation transaction in an
empty resources root. The test writer checks each of the 148 previously
published resource hashes and links the existing E301 files. It writes
`tilemap-transfer-directory.json` and `manifest.toml` through the
standard file writer. The generated manifest matches the E468 reference
byte for byte, and the `current` pointer selects the sole generation.

A second preparation returns `reused` without calling the supplied
writer. The `rom-data verify` command handler also accepts that
generation under Wine. The runner preserves 20 inputs and removes the
temporary aliases after Wine drains. Fresh creation with the default
CLI writer, native Windows, Linux and USA remain open.

## E470 European task-345 action and selector routes

The complete source join projects the 341 primary descriptor halfwords
and all 347 action records onto the 23-slot table. The default action
lookup at `080aa140` provides the task-345 argument through property 32.
Eight action records point outside the bounded primary argument domain.
The two settings of message `+0x0c` bit 1 account for the selector-16
remaps, including slot 8 when the bit is set. The projection assumes the
default dispatcher, initializer and continuation predicates.

The 345 protected checkpoint rosters provide learned and active direct
job-range candidates for 114 actions across 17 slots. Two of these slots
have null directory entries. The candidate index records the original
action, roster entry and matching job ranges for each selector.

Wine `rxwzcitl` reuses the retained Firaga and Soil Evidence input
sequences. At frames 20865 and 8208, it sees the default task literal and
the action/primary metadata reads for action 25 with task argument 26,
and action 49 with task argument 49, respectively. The pointer/count
reader PCs then select slot 2 at frame 20873 and slot 4 at frame 8216.
The SWI reads all 55 and 34 input bytes of the respective streams on the
next frames. Both streams already have E302 owners. This resolves the
reader-PC ambiguity of the older address-only samples for these cases.

The initial `1z4cjccz` trace request exceeded the 32-range limit before
guest execution. The retry verifies its 391 input preimages and reuses
the full roster projection. Both retained inputs run with smaller
filters and Wine drains. Earlier replay ancestry, managed consumption
of further selectors, other dispatcher paths, Linux and USA remain
open.

## E471 Ordinary European task-345 slot-1 input

E470 links action 12 to primary argument 13 and slot 1 in both
initializer flag states. In IN-08, roster entry 10 has a kind-1 reference
to that action in bank 4, ability slot 22, with the learned bit set and
job 30 equipped as the secondary job. Wanda's job 32 has a kind-3
reference to action 10, which also projects to slot 1. Her kind-1
secondary actions select slot 5.

Wine `t8zwfqer` follows the original deployment input and verifies unit
10 in the two retained cursor pointer cells. The resulting turn menu
shows Marisa. From frame 7529, `l1xgftb2` opens the secondary menu and
reaches the first action's targeting screen. A single A press in
`97jkmpq9` reaches the valid target preview at frame 7769. The first
filtered `ore7y6l0` run sees zero selected reads. The single-confirmation
`m3od6xg3` run sees eleven preliminary action-12 reads and does not
reach the property-32 or slot readers.

Wine `jzl685ht` uses the earlier E465 two-confirmation pattern from
that preview and traces 300 interpreter frames. Action 12 and argument
13 are observed with the default task-345 literal at frame 7975. The
pointer and count readers select slot 1 at frame 7983. At frame 7984 the
SWI reads all **57 bytes** at `b0de70..b0dea9`. The raw trace also
observes 12 row, five descriptor and two wrapper bytes. The E302
manifest has no owner for the selected LZ77 input.

The final check preserves 36 inputs and Wine drains. This completes
the ordinary input prerequisite for that stream. E472 verifies the
managed controls on the same path. Managed LZ77 inputs, other slots,
Linux and USA remain open.

## E472 Managed European task-345 slot-1 controls

Wine `sx4zdj9n` replays the E471 action preview against the original
ROM and the authenticated E302 dataset. All 29 slot-1 control bytes
are altered in the managed machine's ROM backing. The staging
directory uses the 148 verified E301 files and the retained E302
manifest and catalog.

The audit attributes 24 distinct bytes to typed reads: four pointer,
one count, 12 row, five descriptor and two wrapper bytes. Both original
directory readers serve managed values at frame 7983. The SWI at
`08145254` obtains the full 57-byte LZ77 input from ROM. The capture
reports 75 matching requests and zero drops.

Across frames 7769 through 8069, both machines match guest clock and
audio frame by frame. Video, OAM and serialized state match at the
endpoint. The build `afpj05jx` and one focused Wine check pass, all 27
inputs are preserved and Wine drains. Managed stream ownership, other
selectors, native execution, Linux and USA remain open.

## E473 European task-345 LZ77 input binding

The bank plan and the runtime derive their input extents from the same
authenticated directory. Each type-3 descriptor links to a four-byte
wrapper. The bounded LZ77 token walk starts immediately after the
wrapper and stops at the declared output size. Shared wrappers resolve
to one input extent, with padding excluded.

Loading e302-d301 binds 13 encoded inputs from the supplied European ROM
in memory. The two streams already served by
`graphics.observed-lz-block` retain that path. SWI 11 and 12 resolve a
bound input at its exact physical start across cartridge mirrors and
use the existing decoder and destination writes. The runtime reports `bound_lz77_calls` and
`bound_lz77_input_bytes` alongside the directory control counters.
Binding requires the authentic ROM at load. The dataset manifest
continues to account for the control fields and prior owners, with
persistent storage for the 3,742 input bytes still open.

Wine `ulsona5y` matches the E467 bank plan and E466 ownership ranges for
all 15 inputs. With the live encoded ranges zeroed, both LZ77 SWIs
produce matching WRAM and VRAM in 90 controlled calls across the three
cartridge mirrors. The 13 new bindings cover 3,742 bytes. The two prior
streams use their existing owner, covering 89 bytes.

The retained E471 action input then runs the same 300 interpreter frames
as E472 with 29 control bytes altered and all 57 encoded bytes for slot 1
zeroed in the candidate. The trace confirms the same 24 typed control
bytes and no raw reads of the stream from the SWI site. One bound call
consumes 57 bytes. Clock and audio match on every frame. Video, OAM and
the serialized endpoint also match, including the E472 endpoint digest.

The Windows harness build `0iy9gyy_` uses the canonical Cargo output.
The sandbox attempt `xf68wnf3` stops before acquiring a lease because its
Wine runtime directory is read-only. The managed host check preserves
35 input paths, drains Wine and retires scratch. It reuses the retained
E302 manifest and catalog with 148 E301 resources. Remaining checks
cover native execution, Linux, USA, and ordinary paths for other slots.

## E474 European task-345 cartridge and DMA reads

The cartridge bus uses the same 13 immutable inputs as the HLE decoder.
Byte, halfword and word reads resolve by physical address, including
cross-page reads and all three mirrors. The runtime composes bytes from
the bound inputs, existing metadata and stream owners, and adjacent ROM
as needed. The native read ABI and DMA use these same bus accessors.
Activation checks that every bound input page is reachable through the
bus. The `bound_lz77_bus_bytes` counter attributes the encoded bytes
served there. The HLE counters account for decoder calls separately.

Wine `c5hp1xyf` verifies all 15 E466 extents with every encoded byte
inverted in the candidate backing. Direct byte reads and native ABI
reads of 8, 16 and 32 bits match across all three mirrors. The checks
span the stream at `b0df54..b0e06f`, whose input crosses a page, and
exercise mixed reads at every input boundary. The new bindings account
for 89,808 byte accesses and the two earlier owners for 2,136. An
altered byte outside the first extent verifies live access to adjacent ROM.
Halfword and word DMA3 copies across that boundary match destination data,
channel control and guest clock, with four bytes served by the binding
on each copy.

The controlled HLE outputs and the 300-frame E471 replay retain E473
equivalence. The replay selects the 57-byte bound HLE input with no
direct reads of the new input bindings. The Windows harness build
`rvt7j7za` passes. The host Wine run preserves 35 input paths, drains
Wine and retires scratch. Persistent input coverage, execution of
translated guest scenes, other ordinary selectors, Linux and USA remain open.

## E475 European halfword and byte tilemap bank binding

The runtime uses the contiguous E457 bank at `af1818..af3ea0` when
loading the authenticated EUR tilemap resource. The E450 type-5 rows
identify all seven byte streams, including the target shared by two rows.
For both stream formats, header dimensions bound the cell count. The
token walk computes the encoded endpoint, then aligns the next source
to four bytes. The resulting 78 extents retain the origin pair in each
header and exclude the byte tokens' internal padding and the gaps
between streams.

The bank binds uncovered ranges in memory from the supplied ROM.
The cartridge bus composes them with data from the existing manifest
owners and live adjacent ROM. `ROM_DATA_USE bank=tilemap-streams` counts
the data bytes served by this binding. Activation still requires the
authenticated source ROM.

Wine `7mu7rdhi` matches every extent and source digest in E457 and the
internal padding in E450. The bank includes 9,654 selected bytes and 210
padding bytes. It serves 8,810 bytes through the new binding and 844
through earlier owners. Inverting the selected backing bytes proves
direct and ABI reads across all three mirrors. Inverting the padding in
both machines verifies that those bytes still come from live ROM.

The test invokes the two original guest readers with cursors derived from
the verified streams. The halfword reader returns 10,685 matching cells
from 71 streams. The byte reader returns 1,733 from seven streams. Each
cursor reaches the verified endpoint with its token index reset. Clock
and final serialized state match, and the original readers consume
38,707 byte accesses through the new binding.

The Windows build `7y7qeoov` and the focused Wine check pass. Wine drains,
all 27 protected inputs pass, and scratch is retired. The proof covers
controlled original reader execution. Ordinary scenes, the additional
directory and pointer consumers, persistent data coverage, Linux, USA
and native Windows remain open.

## E476 European direct tilemap action continuations

Wine `ak9n6nmu` authenticates the 37 entries selected by the halfword at
dispatcher offset `+0x3450`. State 22 selects `080aa0f0`, and state 30
selects `080a9f34`. All 12 E461 action links resolve to their original
primary tasks through the E462 state-30 predicates. State 22 passes the
same action property as an argument to task 345 on the observed path.

On the state-10 branch at `080a999e`, the original code writes state 30.
With mask `0x0208` clear in context halfword `+4`, it pushes 30 to the
continuation stack at `+0x3454` and selects 22. State 22 installs the
task handle and selects 23. State 23 checks the handle at `+0x26c4`.
Once it clears, the code at `080aa42a` decrements the depth at
`+0x3452` and loads the most recently queued state. This allows the
auxiliary and direct constructors to run for one action.

Wine `qjyy7piy` extends the neutral hold in both retained IN-08 replays
to a 900-frame window. Shell visits states 22, 23 and 30 at frames
7943, 8048 and 8049. Protect visits them at 7959, 8064 and 8065.
Both direct constructor reads occur 12 frames after their earlier
300-frame windows end. The original catalog reader at `080d7062`
loads primary IDs 9 and 10 at the respective state-30 frames.

At frames 8072 and 8088, `080e8442` reads the two words at `39bcbc`
and `39bcc0`. The geometry routine reads both origin bytes from the
corresponding mode-7 streams. Each replay observes 33 source bytes
outside the E302 manifest. These comprise four pointer bytes, 12 row
bytes, seven descriptor bytes and ten stream bytes. The two streams are
within the E475 binding. E477 adds the controls for the other 46 observed
bytes.

The first qualifier used the lookup sequence's start at `080e843a`.
Reconciliation `dy80gsv8` authenticates the exact load at `080e8442`
against E458 and verifies both retained traces without replaying them.
Both Wine units drain and their protected inputs pass. Tasks 130 and 133
share the source dispatch conditions. Their live reads at `080fef36`,
managed equivalence, Linux, USA and native Windows remain open.

## E477 European indexed tilemap controls

The E475 bank loader resolves all 12 E461 geometry slots. For the
two-slot groups at `39bcbc` and `3a1c58`, it follows the original
transfer-list pointers and counts to the last row. For the eight-slot
group at `3a4d84`, it selects the second row of each verified E202
pair. Each type-4 descriptor must target the same halfword stream as
the geometry word.

The runtime reconstructs geometry words, rows and descriptors in memory
from the authenticated EUR ROM. It binds 48 uncovered control bytes in
each two-slot group and preserves the existing owners of 192 bytes in
the eight-slot group. Cartridge reads can combine these controls with
bytes from adjacent ROM and earlier resources. The runtime reports
`indexed_control_bytes` alongside the E475 stream count.

Wine `jlv4j9qj` verifies byte and boundary reads across all three
cartridge mirrors, including the read ABI and a neighboring raw byte.
The controlled checks alter the backing of all 288 selected control
bytes. Both IN-08 interpreter replay pairs use the original E476
confirmation input and 425 frames, through the observed dispatcher
state 36. Shell and Protect repeat the original dispatch, geometry and
state-36 reads at the same frames. Each case reads 23 distinct managed
control bytes and ten managed E475 stream bytes with those sources
altered in the test ROM view. Guest clocks and audio match per frame.
The framebuffer, OAM and full serialized state match at both endpoints.

The Windows build `3udkc2bn` and focused Wine test pass. Wine reuses
148 E301 files, preserves all 32 protected paths, drains its server and
retires scratch. The earlier 900-frame attempt `lsaq175e` timed out and
drained. Activation still requires the authenticated source ROM.
Persistent coverage, ordinary IDs 130 and 133, translated guest
execution, Linux, USA and native Windows remain open. Performance
measurement is deferred.

## E478 European indexed tilemap control resource

EUR preparation advances to `e303-d302` and writes
`indexed-tilemap-controls.json` for `graphics.indexed-tilemap-controls`.
The resource describes the complete 12-slot E461 projection. Its
manifest owns the 96 previously uncovered bytes of geometry pointers,
type-4 rows and linked descriptors. The original 192 bytes from the
eight-slot group keep their existing resource owners.

The normal loader authenticates the source ROM and compares all 12
slots with the expected projection. It builds the new bindings from the
parsed resource file. An explicitly selected `e302-d301` generation
remains readable and uses the E477 in-memory binding. Automatic
preparation selects the new generation. Cartridge reads retain the E477
width, mirror and mixed-ownership behavior.

Wine `urfx6b8z` published the new generation, reused 149 verified E302
files as read-only inputs and wrote the indexed controls and manifest
with the standard file writer. A second preparation reused the same
generation without a write. Both revisions passed CLI verification
and normal activation. The altered-backing read comparison covered
all 288 control bytes across the three mirrors and read widths. The
loader rejected a coherent file and manifest hash edit and a missing
control claim, preserving the active owner in both cases.

Windows harness build `q6ehr4lw` passes after fixing the budget test
call reported by `mnatrfna`. Wine preserves all 26 protected paths,
drains its server and retires scratch. Persistent stream backing and
ordinary IDs 130 and 133 remain open. Fresh publication with the
default writer for all files, Linux, USA and native Windows remain
unverified. Performance measurement is deferred.

## E479 European tilemap stream bank resource

EUR preparation advances to `e304-d303`. The `graphics.tilemap-stream-bank`
resource in `tilemap-stream-bank.bin` contains the unowned encoded ranges
from the complete E457 halfword and byte stream bank. The manifest lists
each range in file order with its original ROM offset, length, stream
kind and SHA-256. The stream headers retain their origin bytes. Both
the inter-stream alignment and the byte tokens' internal padding remain
in the cartridge backing. The 844 earlier owner bytes continue to come
from `graphics.observed-tilemap`.

The resource loader compares the file and manifest with the authenticated
ROM and builds the cartridge bindings from the file. Explicit
`e302-d301` and `e303-d302` datasets keep the verified in-memory stream
binding. The source ROM remains required for authentication and the
rest of the cartridge.

Wine `wd9v3o6q` creates all 151 resources with the standard preparation
writer and reuses the published generation. Its manifest adds 130
ranges totaling 8,810 bytes and preserves the complete E303 manifest
prefix. E302 and E303 attach with their original resource identities.
An edited stream file with a matching file digest is rejected, and the
active bindings stay available.

The single Wine test alters all selected stream bytes after attachment
and exercises byte, halfword and word reads on all three cartridge
mirrors. Padding reads still see the altered cartridge backing. The
two interpreter readers return matching values for 10,685 halfword
cells and 1,733 byte cells. Guest clocks, cursor endpoints and the
serialized state match E475. The file owner serves 38,707 bytes during
those calls, including repeated accesses.

Windows harness build `lnahlnsl` passes. Wine preserves all 18 protected
paths, drains its server and retires the generated resources. The
retained evidence contains the manifest and the check results.
Ordinary IDs 130 and 133, the separate task-345 LZ77 inputs, other
banks, translated guest execution, Linux, USA and native Windows
remain open. Performance measurement is deferred.

## E480 European task-345 LZ77 resource

EUR preparation advances to `e305-d304`. The manifest adds
`graphics.tilemap-transfer-lz77` in `tilemap-transfer-lz77.bin`.
Preparation follows the authenticated directory and its distinct
wrapper links. The LZ77 parser gives the encoded length of every input.
The file stores the 13 uncovered inputs in physical order, and the
manifest provides the source offset, length and SHA-256 for each one.
Two inputs totaling 89 bytes retain their earlier owner.

After source verification, the cartridge and HLE readers use the same
immutable file-backed inputs. SWI 11 and 12 still resolve the exact
physical start, and bus reads can cross the end of a stream into the
cartridge backing. Older tilemap datasets keep their ROM-derived LZ77
bindings. The ROM remains required for authentication and the rest of
the cartridge.

Wine `e5w8vr8k` publishes and reuses a fresh 152-resource generation
with the standard preparation writer. The earlier resource entries
and coverage match the E304 manifest. E304 still attaches and supplies
its original LZ77 view. The new generation rejects an
edited file even when its manifest digest is updated, and the previously
attached bindings stay active.

The single focused test matches the E474 bus, read ABI, DMA and SWI
results with the live ROM inputs altered. All 15 extents match across
the three cartridge mirrors. The 90 controlled SWI calls produce the
same WRAM and VRAM results. Both DMA widths match at the boundary
with adjacent ROM. The 300-frame IN-08 slot-1 replay uses 57 bytes
from the file-backed input and records zero raw SWI reads of that
stream. Clock and audio match per frame. The framebuffer, OAM and
serialized state match at the endpoint. Its state hash matches E474.

Windows harness build `5o8qopxw` and the Wine test pass. The runner
preserves 25 protected paths, drains Wine and retires scratch. Other
ordinary selectors and transfer banks, translated guest execution,
Linux, USA and native Windows remain open. Performance measurement
is deferred.

## E481 European type-3 transfer ownership

The combined E448 and E452 projection follows 825 distinct type-3 rows
through 793 descriptors and 372 wrappers. The 628 E448 and 207 E452 rows
share ten row addresses. The wrappers select 358 LZ77 inputs and 14
raw copy views. The audit rechecks each original row, linked pointer,
wrapper kind and flag. It matches the compressed lengths and hashes
against both retained indexes and derives the raw windows from each
row's original size field.

Wine `hs7i9i6k` audits all 349,097 distinct selected bytes against the
E305 manifest. The independent interval comparison matches every
owner and reproduces the earlier E301 totals for both projections.
E305 owns 23,247 bytes, up from 19,149 in E301. The task-345 metadata
covers 356 of the added bytes and its LZ77 resource covers 3,742.

The remaining 325,850 bytes include 785 rows, unowned descriptor and
wrapper fields, 323 complete LZ77 streams totaling 304,054 bytes and all
14 raw views totaling 4,992 bytes. The only partially owned extent is
descriptor `3abd74..3abd7c`. Its final four bytes belong to the compact
primary alias. The extraction plan selects the first four bytes of that
descriptor. It includes 7,744 bytes reached only by the 18 type-3 rows
in the conditional auxiliary-selector projection.

The retained `components.json` records the row links, exact source extents
and selector provenance. `unowned-plan.json` provides the ranges eligible
for new ownership. Both outputs contain metadata. Wine drains and 23
protected inputs pass. Extraction, runtime attribution, other producers,
ordinary auxiliary use, Linux and USA remain open.

## E482 European type-3 transfer resource

EUR preparation advances to `e306-d305` and creates
`graphics.tilemap-type3-transfers` in `tilemap-type3-transfers.bin`.
The extractor follows the verified E446 and E451 row windows and both
nine-slot E453 auxiliary tables. It selects the type-3 rows and follows
their descriptor and wrapper links. The wrapper flag selects the
bounded LZ77 token walk or the raw copy length computed from the row,
wrapper kind and layout bit. Repeated pointers share their source extent.

The resource stores the verified uncovered row, descriptor, wrapper,
LZ77 and raw-input bytes in manifest order. It subtracts existing
ownership at byte boundaries, including the four-byte compact primary
alias at `3abd78`. Each newly owned LZ77 stream retains its entire
encoded extent and start address.

The cartridge bus uses the selected file ranges across all mirrors.
Reads spanning multiple owners compose their byte lanes with prior
resources and live ROM. HLE decompression resolves a file-backed
stream by its exact physical start, using the existing decoder and
destination writes. Older tilemap generations keep their prior
bindings. The source ROM authenticates the dataset and supplies the
rest of the cartridge.

Wine `n9qirmvq` creates and reuses the standard 153-resource
generation. The new file covers all 325,850 bytes of the E481
extraction plan. E305 still attaches and its resource entries match
the E306 prefix. The loader rejects a changed file with an updated
manifest hash and keeps the active resource bindings.

With the backing bytes altered, the focused test matches the bus and
read ABI on all three cartridge mirrors. The split descriptor at
`3abd74` reads correctly from both resources. The 323 controlled
decompressions cover every new LZ77 extent and exercise both SWI
variants. All 14 raw views match through the halfword DMA3 path.

The ordinary 425-frame Shell replay reads both type-3 rows from the
file. It consumes 79 bytes on the bus and 255 LZ77 input bytes in
two HLE calls. Guest clocks and audio agree at each frame. Video,
OAM and serialized state agree at the endpoint, which retains the
E477 state hash.

Windows harness build `40gqbj31` and the Wine test pass. Wine
drains, 20 protected paths pass and scratch is retired. Other
producers, ordinary execution of the conditional selector,
translated guests, Linux, USA and native Windows remain open.
Performance measurement is deferred.

## E483 European type-2 transfer resource

EUR preparation advances to `e307-d306` and adds
`graphics.tilemap-type2-transfers` in `tilemap-type2-transfers.bin`.
It selects the type-2 rows of the E446/E451 projection, including the
18 conditional E453 rows, and follows the authenticated Firaga program.
Each 12-byte row locates an eight-byte list header. The count and
entry pointer in the header bound the eight-byte entries.

Each entry locates a halfword rectangle and gives its starting row and
row count. The extractor checks those against the two-byte height and
width header. It keeps the full header and `height × width` cells.
Shared list headers, entries and rectangle addresses are counted once.
The model covers 294 type-2 rows, 260 list headers, 545 entries and 205
rectangles. The 273 distinct copy windows select 13,962 bytes together
with their controls. Nineteen rectangles have cells outside those
windows. The full bank covers 15,120 distinct bytes.

The E306 manifest already owns 362 of those bytes. The new file
contains the remaining 14,758 bytes in manifest order. In particular,
the header and first row of the rectangle at `ae6db8` retain their
existing owner and the new file supplies its second and third rows.
Cartridge reads can combine those resources in the same word.
Activation checks the packed file against the authenticated source ROM.
The original ROM supplies other cartridge ranges.

The Windows build `eod6d6vx` and focused Wine test `rcph2aol` pass.
Preparation creates and reuses 154 resources while preserving the E306
manifest prefix. E306 still attaches, and the type-3 file supplies its
120-byte BIOS input at the original start address. Replacing the type-2
file and its manifest digest is rejected while the active binding stays
available.

The test compares every new range across all three cartridge mirrors
with altered ROM backing. It covers the split rectangle, adjacent raw
bytes and the byte, halfword and word read ABI. The interpreter enters
the original counted-list routine at `080db374` for all 294 rows. It
compares each resulting destination with the expected cells, including
unchanged bytes outside the copy windows. The routine copies 6,268
bytes and reads 15,993 file-backed bytes including repeated accesses.
The complete final guest state agrees with the unmodified ROM.

The first Wine attempt `cyov8p8f` passed the publication and bus checks
and stopped at the read count for row `39aed0`. The skip loop in
`080d80e0` loads the width once for a nonzero starting row and reuses
its computed stride. Correcting the expected count from 32 to 28 for
that row allowed the same Windows executable to pass the full proof.
Both Wine servers drained. The passing unit preserved 19 inputs and
removed scratch.

The six retained E460 gameplay captures have no reads in the new
intervals. An ordinary scene with a new file-backed read, ordinary use
of the conditional selector, other producers, translated execution,
Linux, USA and native Windows remain open. Performance measurement
is deferred.

## E601 European physical type-2 rectangle bank census

The full EUR ROM scan follows every word-aligned type-2 row
candidate. It checks the list link, every entry in the nonempty
counted list, the rectangle dimensions and the selected row
window. The scan allows the nonzero byte count and dimension
domain and all three physical ROM mirrors. The resulting 294
rows and each copy specification match the accepted E483 proof.
They link to 260 distinct headers, 545 entries and 205 full
rectangles, including all cells of the 19 rectangles with
partial copy windows.

Wine `l3jzyugd` compares the independently reconstructed ranges
to `e354-d353`. Every one of the 15,120 bytes has one owner.
The type-2 transfer file supplies 14,758 bytes, and four existing
resources supply the other 362. The packed byte ranges match the
ROM in manifest order. Original reader execution is covered
by E483. Ordinary and generated paths, RAM-built rows and
indirect producers remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E484 European direct tilemap transfer controls

EUR preparation advances to `e308-d307`. The new resource
`graphics.tilemap-direct-transfers` in `tilemap-direct-transfers.bin`
completes the direct controls for the 78 E457 streams. It follows the
type-4 and type-5 rows of the E446/E451 lists, the declared E445 row
at `39a704` and the E458 alternate row at `39a71c`. Each row
points to a descriptor that must address a stream start of the
matching kind. The links cover all 71 halfword and seven byte streams.

There are 136 type-4 rows and 128 distinct eight-byte descriptors.
The nine type-5 rows have nine descriptors, with six selected bytes
per descriptor. Together the controls select 2,818 distinct bytes.
E307 owns 281 of those bytes. The file stores the remaining 2,537
bytes, preserving the compact primary alias at `3aba8b` and the
earlier indexed controls. Shared descriptors retain a single copy.

The 9,654 encoded bytes and origin fields retain their E479 and
observed-tilemap owners. Byte-token padding and the trailing
type-5 descriptor bytes remain in the original ROM. On the cartridge
bus, a single read can combine lanes from the new file, earlier
resources and the original backing. Activation authenticates the
file and its ranges against the source ROM.

Windows build `5m1ymdyq` and Wine `drb46kuz` pass. Standard
preparation creates and reuses 155 resources. The E307 manifest
prefix is preserved and E307 still attaches. The loader rejects a
file edit accompanied by a matching manifest digest and preserves
the active binding.

The test alters the backing of every new control, the compact alias
byte and all 9,654 selected stream bytes. Mirrored bus and ABI reads
match, including the word split at `3aba88`. A skipped token byte
and a trailing type-5 descriptor byte remain readable from the
altered backing.

The original dispatcher at `080db2a8` runs for all 145 rows.
Its initializer arguments, output memory and final serialized
guest state match the original ROM. The new file supplies 2,196
type-4 control bytes and 153 type-5 control bytes, including repeated
reads. The earlier stream file serves 278,962 bytes during these calls.

The first Wine attempt `crs5ls20` passed publication and bus
checks and stopped at the test instruction limit for row `39bd3c`.
This mode-4 path repeats a 1-by-1 tile across a 32-by-32 map.
The successful build raises the test limit to accommodate that
writer. Both Wine servers drained. The passing unit preserved all
26 inputs and removed scratch.

The six E460 gameplay captures contain no reads of the newly owned
controls. Ordinary scene use, other producers, translated guest
execution, Linux, USA and native Windows remain open. Performance
measurement is deferred.

## E602 European physical type-4 and type-5 stream census

The full EUR ROM scan follows word-aligned type-4 and type-5 rows
through their descriptor pointers. It checks the verified header
modes and follows each encoded token until the declared number
of cells has been read. The results match all 136 type-4 and
nine type-5 rows in E484, including 137 descriptors and the 78
distinct E457 streams.

Wine `4izg3cjr` verifies all 2,818 control bytes and 9,654 encoded
stream bytes against `e354-d353`. Each of these 12,472 bytes has
one owner. The 210 raw bytes in the stream bank consist of 142
inter-stream alignment bytes and 68 byte-token alignment bytes.
Both packed files match the source ranges in manifest order.
The protected audit passes 25 input guards and the server drains.

E484 retains the original dispatcher proof for these rows.
Mode 3, other stream formats, indirect and RAM-built rows,
ordinary and generated execution and deferred platforms remain
in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E485 European tilemap origin references

EUR preparation advances to `e309-d308`. It extends
`tilemap-direct-transfers.bin` with 151 aligned origin pointers
totaling 604 bytes. The extractor scans the authenticated image for
aligned words that address the E457 stream bank. Each word must
address an exact stream start. The E484 descriptor links identify
137 words and the indexed geometry controls identify 12 more.
The remaining 153 are the literal words classified in E458.

Two literal words already belong to `engine.thumb-pc-literals`.
The 12 indexed references retain their observed-tilemap or
indexed-control owners. E459's six additional calls reuse words
from the same literal set. All 302 aligned references to the bank
now have declared owners. This covers the origin references to
76 roots within the 78-stream bank.

The direct control file contains 3,141 bytes, including the earlier
2,537 row and descriptor bytes. The range format and resource count
stay the same. Explicit E308 activation uses its original 2,537-byte
file. The source ROM continues to authenticate both revisions.

Windows build `kqw969ri` and Wine `yi8i51yu` pass. Standard
preparation creates and reuses 155 resources. The independent
E458/E459 projection matches every new interval and the earlier
control ranges stay intact. E308 still attaches. A changed literal
with a matching file digest is rejected without changing the active
binding.

The test alters the backing bytes of all 153 literals and checks
byte, halfword and word reads through the bus and ABI on all three
mirrors. The original Thumb loads return the expected pointers.
The extended file supplies 604 bytes to those instructions and
the previous literal resource supplies the other eight. The final
serialized states match. All 20 protected inputs pass and Wine
drains before scratch is removed.

Ordinary use of the new literals, live indexed and indirect calls,
other banks, translated guest execution, Linux, USA and native
Windows remain open. Performance measurement is deferred.

## E486 European transfer-list directory sources

For the authenticated EUR ROM, `e310-d309` extends the existing
`tilemap-direct-transfers.bin` with the remaining controls that
select the E446 and E451 transfer lists.

The 228 local E446 call witnesses use 230 Thumb literal loads for
their bases. Changing each sampled literal value in the source
model changes the original list argument. The immediate row counts
remain in the instructions. For E451, 23 bounded pointer tables
contain 107 distinct pointer words. The companion tables supply
102 count bytes. Other callers supply fixed counts or form lists
from a base and stride. Fifty original literal words supply the
table and list bases, including the alternate paths.

The 335 distinct positive-count base/count pairs select 1,261 rows.
E309 already owns all 15,132 row bytes and 156 of the 1,650 selected
control bytes. The extended file adds 1,494 control bytes and
retains the earlier owners and range records. Its size is 4,635
bytes. The three empty slots still return a zero pointer and count.
Both auxiliary projections remain bounded to their nine verified
physical slots. Ordinary selector reachability is a separate criterion.

Windows build `yyhnavbf` and Wine `fu029eck` pass. Standard
preparation creates and reuses 155 resources. E309 still attaches with
its original file, and a changed count byte with a matching file
digest is rejected while the active binding remains available.

With all selected control backing bytes altered, direct and ABI
reads match on all three cartridge mirrors. The original Thumb
instructions return the expected values for all 280 literals and
209 table cells, including the empty slots and both auxiliary
tables. The new file serves 1,084 literal bytes and 410 pointer
and count bytes in those calls. Final serialized guest state matches.
All 24 protected inputs pass and Wine drains before scratch is removed.

The proof covers the selected original memory loads. Live indices,
indirect setup and ordinary scene routes remain under the existing
directory and auxiliary criteria. The source ROM still supplies
authentication and other cartridge reads. Translated execution,
Linux, USA and native Windows are pending. Performance measurement
is deferred.

## E487 European tilemap source coverage

Wine `sgmwvtno` reconciles the verified European tilemap bank plans with
all 155 resources in `e310-d309`. The plans cover directory rows and
controls, Mode 7 bounds, the complete selected type-2 rectangles, type-3
rows and streams, direct type-4 and type-5 transfers and references, the
indexed direct controls, and all 23 task-345 directory slots. The audit
includes the documented conditional selector projections.

The manifest claims all **378,999 distinct required source bytes** once
each across 16 resources. Within the contiguous E457 stream bank, 9,654
bytes have a resource owner. The other 210 bytes are 142 alignment bytes
between streams and 68 alignment bytes inside byte-stream tokens, as
identified by E450 and E457. The complete task-345 bank contains 4,358
bytes, including 3,831 bytes for 15 LZ77 inputs. E480 published 3,742
bytes for 13 inputs. Earlier resources supply the other 616 bytes,
including both remaining inputs. E305 and E310 declare the same owners
for the entire task-345 bank.

The Windows `rom-data audit-ownership` command agrees with an independent
per-byte join for all 17 views and their distinct union. The manifest
digest is `be8999d1d1eac0b97055814d7fbebcdf62d0a635cddc8d4a5cded1ea9e848747`.
The combined plan digest is
`7ab2a3a72bbcfd44ba485e31402503c996a7aefd2a5eb7885905edd4f2fdd2c1`.
The unit retains the plan, ownership report and verifier, preserves all
69 protected inputs and drains Wine. The audit reuses the existing
manifest and plan evidence without opening the cartridge image.

This coverage applies to the verified bank boundaries. Ordinary scene
use, additional producers, translated execution, Linux, USA and native
Windows remain open. E470 establishes the original task-345 reads for
slots 2 and 4. E488 verifies their managed replays with altered backing.
Performance measurement is deferred.

## E488 European task-345 slot 2 and slot 4 replays

The standard preparation writer creates the 155-resource `e310-d309`
generation with a manifest matching E486. The normal loader attaches the
generation before the managed replay. Both machines use the retained E470
snapshots and input sequences. Firaga selects slot 2 over 606 frames. Soil
Evidence selects slot 4 over 600 frames.

The test alters 29 metadata bytes per replay in the managed cartridge
backing. It also alters the complete 55-byte and 34-byte LZ77 inputs. The
original task dispatch and selector loads occur at the E470 frames. The
managed bus traces attribute 24 distinct metadata bytes per scene to the
extracted resources, including the pointer, count, row, descriptor and
wrapper reads. `graphics.observed-lz-block` serves one LZ77 HLE call per
scene at the recorded frame. The managed trace has zero cartridge bus
reads of either compressed input.

Guest clock and audio match at every frame for both input sequences.
Serialized state, framebuffer and OAM match at both endpoints. Windows
test build `0ey4354t` and Wine `zmbcb0wa` pass. The test retains both read
traces and endpoint hashes, preserves all 25 protected inputs and drains
Wine.

The proof covers the original interpreter with HLE and the retained
continuations. Earlier snapshot ancestry, other selectors, translated
execution, real BIOS, Linux, USA and native Windows remain open.
Performance measurement is deferred.

## E489 European mode-3 dispatch and adjacent compact-cell bank

The original type-4 transfer chain reaches `080da544` from
`080db320`. That instruction belongs to the list dispatcher at
`080db2a8`, called once from callback `080db42c`. The registration
literal at `db5a8` identifies the callback. Its initialization copies
the base and count, and its active row selector advances in byte
field `+7`. The dispatcher reads the descriptor at row offset `+4`,
then the descriptor's stream pointer. The original load at `080da560`
selects an arm using the stream byte at `+4`.

The signed Thumb branch scan and aligned pointer index identify this
chain and its routing tables. All 254 direct setup call sites reconcile
with E446 and E451. Their bounded projections contain 335 list/count
pairs and 1,261 rows, including 134 type-4 rows. E484 has 136 verified
type-4 rows in total, selecting the same 71 halfword roots as E457.
None selects mode 3. The 99 distinct aligned pointer targets in the
preceding interval also yield zero mode-3 extent candidates under the
known halfword reader bounds.

Wine `tkbc4ljc` verifies all 64 bytes of the two routing tables, their
base literals and the callback literal against `e310-d309`. The bytes
already belong to `engine.thumb-switch-tables` and
`engine.thumb-pc-literals`. An additional mode-3 path requires a
source-backed witness of an original routine supplying such a stream.
Constructed pointers, live selectors outside the catalog projections
and indirect routes remain open.

The same Wine unit verifies the compact-cell constructors that serve
the preceding region and audits the resulting source plan. Its last
compact-cell record ends at `af1816`, followed by two alignment bytes
before the first E457 tilemap at `af1818`. Protected inputs pass and
Wine drains. Linux, USA and native Windows remain open. Performance
measurement is deferred. [Compact-cell plan](rom_data_graphics.md#e489-european-adjacent-compact-cell-source-plan)

## E543 European literal-rooted initializer copies

The authenticated E543 scan finds 17 aligned Thumb patterns in which a
PC-relative literal loads `r0`, followed immediately by one or more identical
three-word `ldmia r0!` / `stmia r1!` pairs. The Wine disassembler checks every
instruction against the exact European image. The original copied extents are
12 bytes for sixteen patterns and 48 bytes for one. Three extents already have
owners in `graphics.ui-raw-transfers` or `engine.copied-halfword-lookups`.
The other fourteen are disjoint from `e334-d333` ownership. These are exact
instruction-pattern candidates; the census does not establish reachability or
cover copies using other registers and instruction sequences.

EUR `e335-d334` extends `engine.copied-halfword-lookups` with those fourteen
word arrays and their literal roots: 204 source bytes and 56 pointer bytes.
Each source retains its physical identity, including identical content at
distinct addresses. The typed little-endian words reproduce the original bytes;
the original code retains the stack destination, subsequent interpretation and
control flow. Existing overlapping owners and unselected bytes keep their prior
routing. Dataset `e334-d333` remains attachable.

Census `e543-eur-initializer-copy-census-wine-m0qm_8f4` and Windows test build
`e543-initializer-bank-windows-build-retry-rt7n1bgc` pass. Wine
`e543-initializer-bank-wine-sdjtlh9f` runs all fourteen original literal and
copy sequences in interpreter and generated instruction blocks with altered
new ROM backing. Stack bytes, guest clock and complete state agree with the raw
machine; 32-bit reads across every source and root boundary also agree. The
new owner serves all fourteen records. Preparation/reuse, exact predecessor
attachment and transactional rejection of an incomplete current dataset pass;
the Wine server drains. Manifest SHA-256:
`52091fd25ec05e6d905fc3efa52b5e7bb1411d295dcf4775a2f65f1c8bc93675`.

The first build `0s3m7lqu` failed on a missing test-only `Bus` trait import;
the corrected retry passed. Ordinary scene entry, downstream use of the copied
stack data, other copy patterns, original BIOS, Linux, USA and native Windows
remain open. No performance measurements were made.

## E544 European initializer copy register variants

Wine census `e544-eur-initializer-variants-census-wine-m6gmfq5h` extends E543
beyond its particular register list. It authenticates 44 aligned patterns in
which a PC-relative Thumb literal loads the source base, followed immediately
by matching `ldmia`/`stmia` pairs. The source and destination base registers
must differ and must not occur in the transferred register list. Individual
pairs may use different register lists. This remains an instruction-pattern
census, not proof that every pattern is reached or that other copy sequences
have been identified.

Thirteen patterns include 212 source bytes absent from `e335-d334`. One of
those patterns continues an E543 array: its first 12 bytes and its literal
retain the existing owner, and only the next 12 bytes are added. Three further
patterns have completely owned source arrays but previously unowned literals;
two of these arrays belong to `graphics.palette-programs`, the other to the
existing copied lookup resource. All sixteen selected patterns add 60 literal
bytes. EUR `e336-d335` extends `engine.copied-halfword-lookups` with the new
word fragments and literal roots without duplicating those earlier owners.

Windows build `e544-initializer-variants-windows-build-tejjsew0` and Wine
`e544-initializer-variants-wine-hoduqk8r` pass. Each selected original sequence
runs in interpreter and generated instruction blocks with the new backing
bytes altered. The copied bytes, guest clock and complete state agree with the
raw path; 32-bit reads across every new source/root boundary also agree.
Preparation and reuse, exact `e335-d334` attachment and transactional rejection
of an incomplete current dataset pass, and Wine drains. Manifest SHA-256:
`5162d9d53b280ac64b7a1fd6d51331a27a3e0d26bcd87f5ef429710c87231e35`.

The wider census does not cover base registers in the transferred list,
non-literal roots, indirect copies or other instruction patterns. Ordinary
scene entry, downstream semantics, original BIOS, Linux, USA and native
Windows remain open. No performance measurements were made.

## E545 European two-word initializer pairs

Wine census `e545-eur-pair-initializers-census-wine-x24m4_a0` authenticates
16 aligned Thumb patterns that load a literal-rooted pair of words and store
them into adjacent stack slots. Thirteen readers and ten physical pairs were
unowned in `e336-d335`; four of those readers share one pair. Three other
patterns already belong to `engine.firaga-stack-initialization`. The scan is
limited to this specific immediate load/store sequence and does not establish
entrypoint reachability or the extent of other initializer formats.

Protected static census `e546-stack-copy-census-o5m6ysfv` additionally
allows up to 24 consecutive word loads and stack stores, including
interleaving, provided the original source and destination words are
contiguous and ordered. It finds exactly the same 16 two-word patterns; all
their source and literal bytes have `e337-d336` owners. Other instructions,
destinations, branches and indirect roots remain outside that census.

EUR `e337-d336` adds those ten two-word arrays and thirteen literal roots to
`engine.copied-halfword-lookups`: 80 source bytes and 52 pointer bytes. Shared
addresses have one physical owner; distinct addresses remain distinct even
when their contents match. Guest code continues to interpret the copied stack
values. All three pre-existing Firaga sources retain their original owner.

Windows build `e545-pair-chains-windows-build-bk4evfo2` and Wine
`e545-initializer-pairs-wine-retry-xtp6kbf3` pass. The thirteen original
readers run with altered new ROM backing in both interpreter and generated
instruction chains. Copied words, guest clock and complete state agree with
the raw path; 32-bit reads across every new source and root boundary agree.
Preparation/reuse, exact `e336-d335` predecessor attachment and transactional
rejection of an incomplete current dataset pass. All ten pair and thirteen
root masks are exercised, and Wine drains. Manifest SHA-256:
`0c1764b25baca3c3ec0d40a744c07cce1a6b62b7abaa9a0db7d0ea9aa183df96`.

The first Wine unit `qisp2u56` failed because its test attempted to stop
between adjacent generated blocks that can execute as one chain. The corrected
test compares both adjacent readers together and passes; both Wine servers
drained. Ordinary scene entry, downstream interpretation, other load/store
patterns, original BIOS, Linux, USA and native Windows remain open. No
performance measurements were made.

## E547 initializer reads in retained scene inventories

Protected join `e547-initializer-scene-join-nowdl2c0` compares all 79 E543,
E544 and E545 source/root spans (664 bytes) against the selected-reader
address inventories previously recorded on Wine for Star Cross, Soil Evidence,
White Flowers, Llednar, Blue Screw and Firaga. None of those captured windows
intersects the new spans. These are address-only samples of selected readers;
they do not exclude other readers or routes, identify executing PCs, or prove
current managed serving. Ordinary entry and downstream semantics remain open.

## E565 European ten-byte stack copy sources

The E560 census and e340 manifest expose two unowned data starts at
`14d360` and `14d36a`. Wine `o3ed4jee` reconciles all 372 world task
words and 381 original loaders. The selected literals at `035824`
and `035aa0` feed the source register of the existing byte-copy helper
`08147dc0` with the original count set to 10. Both literals already belong
to `engine.thumb-pc-literals`. World task slots 31 and 36 enter the
two copies, and the task later reads the copied bytes as signed offsets.

Wine `e565-ten-byte-copy-source-plan-g0_w9rrh` scans all 181 aligned
direct Thumb BL patterns to the authenticated helper. Exactly seven
put `movs r2, #10` immediately before the call. Each loads a separate
ROM literal into the source register and copies to a fixed stack
offset. The source extents are `14d360..14d37e`, three consecutive
ten-byte blocks, and four ten-byte blocks beginning at `14d92c`,
`14d938`, `14d944` and `14d94e`. Each start has one exact aligned
reference across the three ROM mirrors. Two middle sources have
equal contents and distinct physical addresses.

The plan authenticates 70 unowned payload bytes and five unowned
literal words against e340. Two earlier literals keep their owner.
It preserves the two-byte gaps at `14d936` and `14d942` as
unclassified. The plan SHA-256 is
`85b08267eec28fcf5e559be616452f3f49a27d673f8201006020549fd78ccef7`.
Twelve input guards pass and Wine drains. The first plan check
`tm0vlea8` expected a Thumb bit in the even world-table destinations.
The corrected check uses the original dispatch values.

The bounds apply to the exact immediate ten-byte pattern. Other
lengths, earlier or computed size arguments, indirect calls and
ordinary dispatch remain open. Linux and USA remain deferred.

## E566 European ten-byte stack copies at runtime

EUR `e341-d340` adds `stack_byte_copies` to
`engine.copied-halfword-lookups`. It preserves seven original
ten-byte arrays as fixed-size byte records with their physical
offsets. The packet publishes five literal roots and retains
the earlier Thumb literal owner for the two world-task copies.
The original helper supplies the destination and subsequent
guest code interprets the copied data.

Windows build `gcjjl32e` and Wine
`e566-stack-byte-copies-wine-r2a5swsy` pass. The seven original
interpreter paths make 77 typed requests with the 70 source
bytes and all seven literal words altered in memory. The stack
bytes, guest clock and full serialized state match the raw
reference after each copy. An additional 144 boundary checks
cover halfword and word reads across all three mirrors. Both
two-byte gaps still read the altered raw backing.

The exact `e340-d339` dataset attaches. Assigning e341 to the
old packet is rejected while the previous owner stays attached.
Removing the new group produces the exact earlier packet. Only
this resource differs among the 156 entries in the E566 manifest,
whose SHA-256 is
`dc77a74b0ebe493ced79370d8e25be11f4876e408af261fdbafc2395b275e59b`.
Thirteen input guards pass, Wine drains and the unit retains the
candidate manifest and changed packet.

E567 verifies the existing generation and reuse with the standard
CLI. First creation, generated copies, ordinary routes, real BIOS,
Linux, USA and native Windows remain open.

## E567 European stack copy dataset verification and reuse

Internal stage `e567-e341-internal-stage-zuua33cz` verifies all 157
files of the exact E566 candidate. It takes 151 unchanged resource
files from the verified e333 copy and four updated files from e340.
The E566 manifest and copied lookup packet supply the remaining files.

The default-feature Windows build `evwkootc` and Wine
`e567-stack-byte-cli-wine-xcsq9618` pass. The standard
`rom-data verify` command authenticates all 157 files. The standard
`rom-data prepare` command returns `reused` for the selected e341
generation. File and selector hashes agree before and after both
commands. Eleven input guards pass and Wine drains.

The stage retains the verified generation under
`out/e567-validation/staging/target/validation/units/e567-e341-internal-stage-zuua33cz/work/resources`.
The empty test-only fallback guard was removed after verification.
First creation from an empty resource root remains an independent
entry in the recovery checklist.

## E568 European fixed-size literal byte-copy census

Wine `e568-fixed-byte-copy-source-plan-uqy1fdv4` follows the same
181 aligned direct Thumb BL patterns to `08147dc0` as E565. It
selects every setup with a positive immediate byte count in `r2`
and a PC-relative literal source in `r1`, allowing one intervening
instruction that only sets destination register `r0`.
The 120 ROM-source calls copy 2, 3, 4, 5, 6, 8, 10, 24 or 64 bytes.
The other patterns include eight literal sources outside ROM,
40 different source setups and 13 different size setups.

Against the exact e341 manifest, 21 selected calls have both
arguments covered by existing resources, including all seven
E565 ten-byte copies. The other 99 calls leave **526 payload
bytes in 91 distinct source ranges** and **388 bytes in 97 literal
words**. The plan records shared sources and literals once per
physical range. The only new payload outside page `14d` is the
four-byte source at `6df49e`, shared by three calls.

The verified Windows CLI checks 590 halfwords in 12 original
instruction windows. All 16 input guards pass and Wine drains.
The plan SHA-256 is
`aa5ff835be13361562868356d7bf8b8b27a5059a8c2383efaaf6470c06a91560`.
E569 integrates this residual. Other argument setups, indirect branches,
ordinary dispatch, Linux and USA remain open.

## E569 European fixed-size byte copies at runtime

EUR `e342-d341` extends `engine.copied-halfword-lookups` with the
E568 residual. It extracts 91 distinct payload ranges (526 bytes)
and 97 distinct literal words (388 bytes) from the exact European ROM.
Calls sharing a source or a literal use the same physical span. The
runtime serves each covered byte through its original address and
Game Pak aliases. Unowned bytes in a mixed request use the original
backing or their existing owner.

Windows build `xwc6z8ye` and Wine
`e569-fixed-byte-copies-wine-77d8ytdh` pass. The original Thumb
instructions execute all 99 affected copies with every newly owned
byte altered in the test ROM buffer. The trace records 591 typed
requests, including byte and word payload reads. The destination,
full serialized state and guest clock match the raw reference after
each call. All 2,256 halfword and word checks at span boundaries
across the three cartridge windows and both raw gap checks pass.

The verified e341 dataset still attaches. Relabeling it as e342 without
the new bank is rejected before replacing the attached resources.
The e342 manifest changes only the copied lookup entry among the 156
resources. The candidate manifest and packet are retained in the Wine
evidence. Thirteen input guards pass and Wine drains.

E570 verifies standard CLI creation, verification and reuse. Ordinary
routes, generated copies, other source/count setups, indirect calls,
original BIOS, Linux, USA and native Windows remain open. See the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E570 European fixed-size byte-copy CLI publication

The default-feature Windows build `b_ocl3zy` creates the complete
`e342-d341` generation with `rom-data prepare` from an empty root under
Wine. Its manifest and copied lookup packet equal the E569 files byte
for byte. All 157 files match their expected hashes. The standard
`rom-data verify` command confirms the dataset, and a second
`rom-data prepare` selects the same generation. The file set, hashes,
modification times and selector match before and after verification
and reuse.

The initial Wine unit `nirq6vbi` timed out in filesystem I/O while
preparing on the external artifact volume. Its partial output remains
available for diagnosis. Wine drained and the runner released the lease.
The successful unit `e570-fixed-byte-cli-internal-wine-9c5u5ikm` uses
the same executable and managed Wine prefix with the resource root on
the internal disk. All 13 input guards pass and Wine drains.

The verified e342 predecessor is retained under
`out/e570-validation/staging/target/validation/units/e570-e342-internal-stage-qc09ozju/work/resources`.
The unit's `evidence/result.json` identifies the exact generation.

## E571 Remaining European direct byte-copy arguments

The default Windows CLI verifies 2,876 distinct Thumb halfwords in
12 disassembly windows on Wine. The census covers the 53 argument
setups left open by E568 and the eight earlier literal-source RAM
copies. The other 120 direct calls have complete payload and literal
ownership in e342.

The local argument slices resolve a positive count for all 61 calls.
The 13 previously unresolved count expressions all produce 264 bytes.
Twelve copies take 80 bytes from `03000940`, two use a fixed or bounded
EWRAM address, and three take a stack-relative pointer. Forty source
domains still depend on loaded pointers or registers from outside the
local slices. These include the two counted lists, 14 copies of
204-byte records, seven temporary-block restores and 11 indirect
264-byte copies. The copy at `0d84ee` takes a stack buffer filled from
IWRAM by an earlier setup. That producer and its literal at `0d8508`
have their own verification boundary.

Four paths resolve to two four-byte ROM payloads at `14d70c` and
`14d724`. Each pair shares a PC-relative literal through `r9`, which
the authenticated copy helper preserves. The e342 join covers 53
distinct source/count literal words. `engine.thumb-pc-literals` owns
the word at `12969c`. The residual is 52 words (208 bytes) plus the two
payloads (8 bytes), with no overlapping ranges.

Wine unit `e571-byte-copy-direct-windows-gr7xv4qr` passes all 16 input
guards and drains Wine. The ownership join `wq5etv7t` passes all 11
input guards and retains the complete per-call argument and source
plan under `out/e571-validation/projection/target/validation/units`.
It records local instruction definitions and candidate payload extents.
E572 integrates the two payloads and the 52 residual argument words.
Ordinary entry paths, dynamic producer domains, indirect calls to the
helper, original BIOS, Linux and USA remain open.

## E572 European direct byte-copy argument bank at runtime

EUR `e343-d342` extends `engine.copied-halfword-lookups` by 216 bytes
from the E571 ownership join. The extractor reads 52 argument words
from the attested Thumb literal sites and extracts the two four-byte
ROM payloads reached through those words. The runtime serves each
physical span through the existing copied lookup paths. The standard
Thumb literal resource still owns the word at `12969c`.

Windows build `gdyjs9j4` and Wine
`e572-direct-byte-copy-arguments-wine-0z1brzgc` pass. Each of the 53
original literal instructions uses its extracted word with the ROM
backing altered. Two original sequences execute both copies of each
shared payload, including the `r9` forwarding across intervening
helper calls. The four copy paths make 18 typed requests. The copied
bytes, serialized state and guest clock match their raw references.
All 648 mixed halfword and word mirror checks pass, including the
raw gap beside the span at `0088dc`.

The verified e342 dataset attaches. Relabeling it as e343 without the
new bank is rejected before replacing the attached resources. The
e343 manifest changes one entry among the 156 resources. The Wine
unit retains the candidate manifest and copied lookup packet. All 14
input guards pass and Wine drains.

E573 verifies standard CLI creation, verification and reuse. The
dynamic producers, ordinary and generated routes, original BIOS,
Linux, USA and native Windows remain open.

## E573 European direct byte-copy CLI publication

The default-feature Windows build `yf___ub5` creates the complete
`e343-d342` generation with `rom-data prepare` from an empty internal
resource root under Wine. The manifest and copied lookup packet match
the E572 reference byte for byte. All 157 files match their expected
hashes. `rom-data verify` accepts the generation, and a second
`rom-data prepare` reports `reused` for the same path. File set,
identity, hashes, modification times and the selector agree before and
after verification and reuse.

Wine `e573-direct-copy-cli-internal-wine-e3sb3wpw` passes all 13
input guards and drains the server. The current European generation
is retained under
`out/e573-validation/staging/target/validation/units/e573-e343-internal-stage-f4gl9ktu/work/resources`.
The unit's `evidence/result.json` identifies the exact dataset directory.

## E574 European dynamic byte-copy producer families

Three managed Wine units attest 14,233 distinct Thumb halfwords around
the 40 direct copy setups whose source pointers E571 left open. The
setups group into two counted message record paths, two cached-object
copies, seven scratch block restores, 14 context record copies, 11
indirect context record copies, one EWRAM record loop, two indexed
exchanges and one five-byte list projection.

The loop's call at `08c00a` copies 264 bytes from
`02000080 + 264 * slot`. The loop visits slots 1 through 23. A source
requires a previous empty slot, so the first possible source is slot
2. The half-open source envelope `[02000290, 02001940)` lies entirely
in EWRAM.

For the 11 copies that read the pointer field at context offset
`0x1d10`, nine local writes supply `02000080 + 264 * index` with an
unsigned byte index. These writes bound the 264-byte record within
`[02000080, 02010880)`. Seven other writes relay the pointer from
IWRAM slot `030027f4`. The seven confirmed local writes to that slot
include a fixed EWRAM address, two indexed EWRAM addresses, a preview
allocation offset, a helper result, a selected quest list pointer and
a forwarded register. The helper, selected pointer, forwarded register
and possible alias writers leave the live pointer domain open.

The local context and scratch allocation definitions narrow several
other source paths. The two counted message constructors forward an
incoming pointer and share a dispatcher. The five-byte projection
receives the first word of a list node through its one direct caller.
The 39 remaining domains still require live provenance. The earlier
stack copy at `0d84ee` draws from a buffer filled from IWRAM under the
attested setup. Other upstream stack definitions and indirect callers
remain open.

The guarded e343 ownership join yields 124 distinct aligned PC-relative
literal words at 125 original readers. The copied lookup bank owns
37 words and the standard Thumb literal bank owns three. The residual
is 84 complete words (336 bytes) across 85 readers. The plan records
exact span hashes and reader offsets without extending ownership.

Wine units `rd18kcew`, `e4k9r2zs` and `w7i6m4kp` pass 18, 22
and 26 input guards respectively and drain the server. The join
`d7fx1ud0` passes 21 input guards and retains the per-call plan under
`out/e574-validation/projection/target/validation/units`. E575
integrates the 84 residual words. Ordinary and generated dispatch,
original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E575 European byte-copy producer literals at runtime

EUR `e344-d343` extends `engine.copied-halfword-lookups` by 84
aligned producer literal words (336 bytes). The extractor follows the
original Thumb loads to the literal words in the authenticated ROM.
The two readers of the word at `07deb4` share a single source span.
The packet binds each word as a `byte-copy-producer-literal`. The
existing copied lookup path serves the bytes and preserves the 40
earlier owners.

Windows build `wbvwdsfb` and Wine unit
`e575-byte-copy-producer-literals-wine-unsgvy6q` pass. Each of the 85
original load instructions makes one typed four-byte read with its
ROM backing altered. The loaded values, full serialized state and
guest clock match the raw references. All 1,008 halfword and word
boundary checks pass across the three ROM mirrors. The adjacent raw
word at `006910` continues to reflect the altered backing.

The exact e343 dataset attaches. An e343 packet relabeled as e344 is
rejected before replacing the attached resources. The e344 manifest
changes a single resource among 156. The previous copied lookup packet
matches after removing the new field. All 16 input guards pass and
Wine drains. The unit retains the new manifest and packet.

The initial build `j3fkxz15` fails with E0599 because the test
omits the `Bus` trait import. The corrected build is the one used
for Wine validation. E576 verifies standard CLI publication. The 39
live source domains, ordinary and generated use, original BIOS,
Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E576 European byte-copy producer literal CLI publication

The default-feature Windows build `v5htx1gr` creates the European
`e344-d343` generation with `rom-data prepare` from an empty internal
resource root under Wine. The manifest and copied lookup packet match
the E575 reference byte for byte. All 157 file hashes match.
`rom-data verify` accepts the generation and a second
`rom-data prepare` reports `reused` for the same path. File set,
identities, hashes, modification times and the selector agree before
and after verification and reuse.

Wine unit `e576-producer-literal-cli-internal-wine-bdfakqb6` passes
all 13 input guards and drains the server. The European e345
reference generation is retained under
`out/e576-validation/staging/target/validation/units/e576-e344-internal-stage-3i4ioqmk/work/resources`.
The unit's `evidence/result.json` identifies the exact dataset
directory. The 39 live source domains and the deferred runtime and
platform checks remain in the recovery checklist.

## E577 European shared byte-copy pointer producers

E577 traces the seven direct writes to `030027f4` from E574. The
write at `032040` supplies fixed EWRAM address `02002fc4`. Two
writes supply `02000080 + 264 * u8(index)`, giving the half-open
264-byte envelope `[02000080, 02010880)`. The dispatcher at
`125c70` selects one of eight entries by the input byte at offset
`0x6`. Its branch at `125dd8` forms `02003b1c` in `r6`.
The call at `125e38` receives that pointer in `r1`. The write at
`125e3e` stores the fixed source on an `r6`-preserving return.

The write at `05cb42` supplies `[0200f428] + 0x3284`. The four
observed allocation sizes at that root are `0x1d7c`, `0x2fa0`,
`0x3404` and `0x9c08`. A complete 264-byte record at the observed
offset requires at least `0x338c` bytes. The two larger allocations
meet that size. The live allocation and reachability remain open.

The call at `024be2` makes the write at `024be8` load field `0x18`
of the object referenced by `0200f438`. Two local field writes
take a pointer from an indexed array reached through offset
`0x94`. The setter at `02a720` has six aligned direct callers and
stores its incoming pointer. At `062f26`, the source is a selected
word from `[0200f44c] + 0x20 + 4 * index`. The selection routine
adds a signed delta to the view offset and clamps to the available
count for a valid positive count. The list writers, live selections
and pointer contents still require evidence.

The two Wine units `rzp7_338` and `3y34xpii` attest 1,908
halfwords and 786 further halfwords respectively. They pass 30
and 34 input guards and drain Wine. The guarded e344 join
`ui3_rhik` passes all 32 input guards and finds 51 aligned literal
words at 51 original loads. The current dataset owns 24 words and
the dispatcher table. The remaining 27 words (108 bytes) form one
candidate extraction bank. The initial join `nahd38zl` had a
reversed register expectation at `032040` and the corrected join
uses the same Wine evidence.

E578 integrates the 27 literal words. Live aliases and ordering, the
other source families, ordinary and generated use, original BIOS,
Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E578 European shared-slot literals at runtime

EUR `e345-d344` extracts all 27 unowned literal words (108 bytes)
in the E577 shared-slot producer plan. Each original Thumb load
identifies its source word in the authenticated ROM. The existing
`engine.copied-halfword-lookups` packet binds each word as a
`byte-copy-shared-slot-literal`. Its earlier spans, the other
155 resource entries and the complete eight-entry dispatcher table
match e344.

Windows build `j981vha7` and Wine unit
`e578-shared-slot-literals-wine-fj32mu8e` pass. Each of the 27
original loads makes one typed four-byte read with the ROM backing
altered. Loaded values, serialized state and guest clock match the
raw references. All 324 halfword and word boundary checks across
the three ROM mirrors pass. The adjacent raw word at `0270b0`
continues to reflect the backing.

The exact e344 predecessor attaches. An e344 packet relabeled
as e345 is rejected before replacing the attached resources.
Wine drains after all 15 input guards pass. The unit retains
the e345 manifest and copied lookup packet.

E579 verifies standard CLI publication. The 39 live source domains,
ordinary and generated use, original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E579 European shared-slot literal CLI publication

The default-feature Windows build `l0eyeusr` creates the European
`e345-d344` generation with `rom-data prepare` from an empty internal
resource root under Wine. The manifest and copied lookup packet match
E578 byte for byte. All 157 file hashes agree. The standard
`rom-data verify` command accepts the generation and a second
`rom-data prepare` reports `reused` for the same path. The generation,
file set, identities, hashes, modification times and selector match
before and after verification and reuse.

Wine unit `e579-shared-slot-cli-internal-wine-mm0k4grq` passes
all 13 input guards and drains the server. The current European
generation is retained under
`out/e579-validation/staging/target/validation/units/e579-e345-internal-stage-7sglfxj4/work/resources`.
The unit's `evidence/result.json` identifies the exact dataset
directory. The 39 live source domains and the deferred runtime and
platform checks remain in the recovery checklist.

## E580 European helper field producers and dispatch tables

The setter at `02a720` stores its argument at offset `0x18` of
the object in `0200f438`. Three of its six direct calls take the
first word of the handle at primary context offset `0x4`. The
call at `097dfa` dereferences a handle from the filtered lookup
at `09cd5c`. Its callback at `09e2c8` selects a reference using
record bytes `0xf6` and `0xf7`. The nested dispatcher at `0bbc9c`
uses the same lookup to populate subcontext offset `0xa0` at
`0bd7c6`. The call at `0bda30` dereferences that field. The live
handles and intervening writes remain open.

The call at `0bce32` forwards a word from the list at subcontext
offset `0xc` through the wrapper at `02a6f8`. The observed
constructor at `0bb988` scans 24 input records with stride 264
and compacts pointers to accepted records. Its direct setup uses
base `02000080`. A live selector inside the compacted count in
the matching context gives a complete source envelope of
`[02000080, 02001940)` in EWRAM.

The three indexed stores at `0270a8`, `0297aa` and `0298fc`
select a four-byte list entry through context field `0x94`.
The constructor at `028fdc` copies a 20-byte template from
`394e44..394f20` and allocates four bytes per count at template
offset `0x4` for the list. All eleven templates already belong to
`ui.context-templates`. The separate setup at `029240` stores
the allocation for context field `0x9c`. The local writer scan
finds 14 other candidates that write indexed entries through
`0x94`. Mode-specific entries and alias writers still need live
evidence.

The e345 join checks 210 destination entries in six dispatch
tables reached by these paths. Four tables already have complete
owners. The seven-entry table at `096e54` and the eleven-entry
command table at `097644` leave 80 unowned bytes including the
two root words. The latter uses commands 7 through 17 and sends
out-of-range values through the original helper at `09a230`.
The join also checks 24 producer literal words at 25 original
loads. Ten unowned words add 40 bytes to the integration plan.

Wine `slfj198b`, `ehtmtokr` and `8ajybsq0` attest 8,837,
804 and 1,303 additional halfwords. Their 38, 42 and 46 input
guards pass and Wine drains after each unit. The e345 join
`mdd2l_wy` passes 44 input guards and retains the per-path plan,
table extents and source hashes under
`out/e580-validation/projection/target/validation/units`.
E581 integrates the 120 bytes. Live source domains, ordinary and
generated use, original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E581 European helper field runtime integration

The European `e346-d345` dataset adds 72 bytes of E580 destinations
to `engine.thumb-switch-tables` and 48 bytes of roots and producer
words to `engine.thumb-pc-literals`. The other 154 resource entries
match e345.

Wine unit `e581-helper-field-dispatch-wine-x1m7j60t` uses Windows
build `5i4bo5p_`. It changes backing ROM bytes for each controlled
interpreter path and verifies all 18 in-range dispatches, six
out-of-range guards and ten individual producer loads with identical
complete machine state and guest clock. There are 46 typed requests
and zero raw requests to the selected ranges across these paths.
Both adjacent raw words and 168 mixed-width boundary reads across
the three ROM mirrors pass.

The e345 predecessor attaches. Relabeling it as e346 without the
new groups is rejected before replacing the attached resources.
All 17 input guards pass, the Wine server drains and the managed
prefix lease is released. The unit retains the manifest and both
new packets under `evidence`.

E582 verifies standard CLI publication. The 39 live source domains,
ordinary and generated use, original BIOS, Linux and USA remain in
the [recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E582 European helper field CLI publication

The default-feature Windows build `lby9j1c3` creates the European
`e346-d345` generation with `rom-data prepare` from an empty resource
root under Wine. The manifest and both E581 packets match byte for
byte. All 157 file hashes match. The standard `rom-data verify`
command accepts the generation. The second `prepare` reports
`reused` for the same path, with the same file set, identities,
hashes, modification times and selector.

Wine unit `e582-helper-field-cli-internal-wine-5v5__08d` passes all
14 input guards and drains the server. The European e346 reference
dataset is retained under
`out/e582-validation/staging/target/validation/units/e582-e346-internal-stage-tjyv_m4d/work/resources`.
Its `evidence/result.json` identifies the exact generation. The
first staging attempt `d2vk9wez` stopped on a wrapper argument
before starting Wine. The corrected invocation reused the same build.

## E583 European 204-byte context record bank

Both branches of `0735b0` request `0x7264` bytes from the arena at
`03002778`. They publish the result at `03002818` and pass a fill of
`0x3932` halfwords to `145230`. Five direct calls enter this setup.
The allocator can return null. The arena slot has nearby constructors
that request `0xc624` or `0x22e0`, and one clear. The smaller arena
cannot supply this context. The last live arena and context writers
still need observation.

Eight direct copies take `0xcc` bytes from the work record at context
offset `0x2cac`. The other six use `context + 0xc6c + 0xcc * slot`.
The status halfwords start at `0xc60` and selection uses byte `0xc21`.
The forward and backward selectors at `07cf34` and `07cf9c` cycle over
five slots. The selectors at `0902c4` and `09032c` cycle over six slots.
These cycles require an active record. The backward path also requires
a valid incoming index. The matching mode initializers write five or
six status halfwords and the largest record array ends at `0x1134`.
The array and `[0x2cac, 0x2d78)` work record fit the matching allocation.

The five-slot setup at `07d1e4` forwards the alternate byte at `0xc22`
unless it equals `0xff`. The six-slot setup writes the first active
index or the empty-list sentinel at `090544`. Live writers, aliasing and the
guards before copying still need attribution. A read-only inspection
of seven retained Giza checkpoints finds zero arena and context roots.
An ordinary recording with the allocation active is needed.

Wine `1eob7sxe` attests 4,605 new halfwords, passes 30 input guards
and drains the server. The guarded e346 join `qtufl1m9` checks 63
literal words at 69 original loads. Seventeen already have owners.
E584 integrates the 46 residual words (184 bytes). The plan is under
`out/e583-validation/projection/target/validation/units`.
The live checks and deferred platforms are in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E584 European context record literals at runtime

The European `e347-d346` dataset adds the 46 E583 allocation,
selection and copy source words to `engine.copied-halfword-lookups`.
The extractor follows the authenticated Thumb literal loads. The
packet owns the 184 bytes as `byte-copy-context-record-literal`.
The earlier packet groups and the other 155 manifest entries match e346.

Windows build `1sqsyqmk` and Wine `e584-context-record-literals-wine-rdvwbe4l`
pass. Each of the 46 original instructions makes one typed read with
its ROM backing altered. Registers, complete serialized state and
guest clock match the reference for each load. The 552 mixed-width
boundary checks across three ROM mirrors pass. The adjacent raw word
at `025874` still reflects its altered backing.

The exact e346 dataset attaches. Relabeling it as e347 without the
new group is rejected before replacing the attached resources.
All 14 input guards pass and the Wine server drains. The unit retains
the new manifest and copied lookup packet. E585 verifies standard CLI
publication. Live pointer domains, ordinary and generated routes,
original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E585 European context record CLI publication

The default-feature Windows build `nqyflova` creates `e347-d346` with
`rom-data prepare` from an empty internal resource root under Wine.
The manifest and copied lookup packet match E584 byte for byte. All
157 file hashes match. The standard `rom-data verify` accepts the
generation, and a second `prepare` reports `reused` for the same path.
The file set, identities, hashes, modification times and selector match
before and after verification and reuse.

Wine unit `e585-context-record-cli-internal-wine-4u4irudf` passes all
13 input guards and drains the server. Internal stage `67w2lo4y`
retains the verified European generation under
`out/e585-validation/staging/target/validation/units/e585-e347-internal-stage-67w2lo4y/work/resources`.
The stage's `evidence/result.json` identifies the exact dataset.

## E586 European 80-byte scratch restore bank

All seven direct restore calls take 80 bytes from a context and write
them back to `[03000940, 03000990)` in IWRAM. Each has a save call
that copies the same IWRAM block to the same context offset. The
seven pairs use six distinct regions. At `0326e2` and `032706`,
the interval `[0xbb8, 0xc08)` fits the `0x7570` allocation published
in `03002810`. The original callers run the context constructor
before these helpers, so the block identity across phases still
needs live evidence.

Two preview pairs use `[0x2e10, 0x2e60)` at `[0200f428]`. The
matching allocations require at least `0x2e60` bytes. The third pair
uses `[0x3398, 0x33e8)` and fits the `0x3404` and `0x9c08` allocations.
The `0x1d7c` allocation fits neither interval. All four original
publication sizes and four corresponding slot clears are attested.

The quest view uses `[0x560, 0x5b0)` in the `0x3000` allocation
at `[0200f44c]`. One setup puts `[0200f454]` in the caller slot
used by `067732` and `067830`. In that path `[0xb54, 0xba4)` fits
the `0x3800` allocation. The final pair uses `[0x2c5c, 0x2cac)`
in the `0x7264` context at `[03002818]`. This interval ends where
the E583 work record begins.

Wine `_yk3qlx6` attests 372 new halfwords, passes 39 input guards
and drains the server. The guarded e347 join `1u8w8ztb` verifies the
14 copy sites and identifies 22 unowned words (88 bytes) among 57
original literal reads. The first two joins `1xhk63aq` and `iph3t5v3`
had incorrect register and destination load expectations. The final
join reuses the same Wine evidence. The plan is under
`out/e586-validation/projection/target/validation/units`.

E587 integrates the 22 unowned words. Live slot writers, save/restore
ordering, original BIOS and deferred platforms remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E587 European scratch restore literals and CLI

The European `e348-d347` profile adds the 22 E586 support words
(88 bytes) to `engine.copied-halfword-lookups`. The extractor
follows the authenticated original Thumb literal loads. The earlier
packet groups and the other 155 manifest entries match e347.

Windows build `iffyfyqx` supplies the audit test and default CLI.
Wine `e587-scratch-restore-literals-cli-wine-1ai5ud0i` verifies
all 22 original loads with the backing ROM words altered. Each load
uses its typed resource and matches the reference state and guest clock.
All 264 mixed-width boundary reads across the three ROM mirrors pass.
The raw word at `032580` reflects the altered backing. The e347
predecessor attaches, and relabeling it as e348 without the new group
is rejected before replacement.

Within the same Wine lease, default `rom-data prepare` creates
`e348-d347` from an empty internal resource root. The manifest,
copied lookup packet and all 157 file hashes match. `rom-data verify`
accepts the generation and a second `prepare` reports `reused`.
The generation, file set, identities, hashes, modification times and
selector match before and after verification and reuse. All 16 input
guards pass and Wine drains. The current generation is retained in
`out/e587-validation/staging/target/validation/units/e587-e348-internal-stage-8x8kiufl/work/resources`.

Live save and restore ordering, context identities, ordinary and
generated dispatch, original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E588 European counted message copy contract

Wrappers `0068dc` and `006938` store the incoming source at message
offset `0x4` and the low 16 bits of the count at offset `0x14`.
They request `u16(64 + stride * count)` bytes through `005bd0`.
The allocator path at `0059b8` adds `0x1c` and narrows again to u16.
It forwards the 28-byte message, sets event zero and dispatches at
`005a6c` through `145c14`, with the callback context at allocation
offset `0x18`.

The callback at `005f5c` copies 12-byte records to context offset
`0x34`. The `006270` callback copies 16-byte records to context
offset `0x30`. Both skip a zero count and iterate while the index is
below the message count. The allocation request is
`(92 + stride * count) mod 65536`. The largest nonwrapping counts
are 5,453 and 4,090. These ceilings describe the allocation arithmetic.
The original producer must establish the live source and count before
either can define an extractable bank.

The guarded plan `wm8ubq4h` reuses the three drained E574 Wine
attestations and passes 24 input guards. All four literal words and
the first IWRAM helper vector word at `370eb0` already have e348
owners. The census finds only the wrapper literals for the two
callbacks. It finds zero direct Thumb branches, matching aligned ARM
branch patterns, Thumb PC-derived address loads or stored absolute
words for either wrapper across the scanned encodings.

The read-only Sol consultation `/root/sol_e588_counted_message_entry`
also checked the paired setters and adjacent helpers for direct
Thumb calls and absolute pointers. The next prerequisite is an
authentic positive-count event-zero dispatch for each shape at
`08005a6c`, selected by callback `08005f5d` or `08006271` in `r3`.
The message source, count and their producer or last writer identify
the next bank to investigate. Ordinary reachability, Linux and USA
remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E589 European cached object copy bank

The initializer at `00887c` publishes the supplied pointer to
`02009188` and clears `0x1bd0` bytes through `145c10` and the
IWRAM helper vector at `370eb0`. The matched ROM body at `d96838`
clears exactly the requested number of bytes. The initializer copies
four bytes from `02002fc0` into the last field, at object offset
`0x1bcc`. The reset path clears the cached pointer at `0089ac`.
The scan found 26 Thumb PC-relative loads of this slot address. Its
nearby direct stores are the publish and reset. Other aliases remain
an open criterion.

The `0246a4` state machine requests `0x1bd0` bytes and stores the
pointer at `0200f390 + 0x5c` in state 0. The state 1 path reloads
that field and forwards it through `024718` and `0097de`. The
state 2 completion path frees the stored pointer and clears the field.
At `033de0`, the zero-bit branch reaches the second initializer
call at `033e58` with the pointer stored at `[03002810] + 0xc08`.
Both identified context constructors allocate `0x7570` bytes for
the parent and write the pointer to the same field. The `0324fe`
writer stores a successful `0x1c10` allocation that fits the
initializer. The `03264e` writer stores a `0x0f68` allocation,
which falls `0xc68` bytes short of the clear if it reaches that
call. The active constructor and actual field writer at `033de0`
still require a live observation.

The 11-entry table at `0089dc` dispatches selectors 9 and 10 to
the two four-byte copies from the object back to `02002fc0`.
Selector 9 first writes the fixed seed at `6df49e` to the object.
The initializer, the byte write at `0096f2` and the second fixed-seed
copy at `00970a` also supply that field. Selector 10 needs a live
last-writer check. The other guarded table at `009564` has 10
entries and dispatches through the field-editing routine.

Wine unit `m2fn_94l` verified 400 disassembled halfwords on the
exact EUR ROM with the retained e348 Windows CLI. All 31 guards
passed and the server drained. Source plan `fo895xwa` passed 32
guards and joined 52 support words to the e348 manifest. The four-byte
seed and helper vector word have existing owners. E589 identified
84 unowned table bytes and 132 bytes from 33 unowned literal words.
E590 integrates those sources. Live object identity and bounds, ordinary and
generated dispatch, original BIOS, Linux and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E590 European cached object tables and literals

The European `e349-d348` dataset serves the 21 switch destinations
through `engine.thumb-switch-tables` and the 33 support words
through `engine.thumb-pc-literals`. The table roots are among those
33 words. The source delta is 216 bytes across the two resources.
The other 154 resource entries match `e348-d347`.

Windows build `ctkdyxc5` and Wine unit `mdbpggu3` pass. With
altered backing ROM, one focused interpreter test executes the
original dispatch instructions for all 21 targets, both
out-of-range paths and the second table's disabled path. It
verifies all 33 original literal loads, 75 typed requests,
identical serialized state and guest clock, 420 mixed-width
boundary reads across the three ROM mirrors and both adjacent
raw gaps. The e348 predecessor attaches. Relabeling it as e349
without the two groups is rejected before replacing its resources.

In the same Wine lease, the default CLI prepares from an empty
resource root and verifies all 157 files. A second prepare
reuses the same generation and selector. File identities, hashes
and modification times match after verify and reuse. All 17
input guards pass and the server drains. The current dataset is
retained in
`out/e590-validation/staging/target/validation/units/e590-e349-internal-stage-ryyko0b0/work/resources`.

The live cached pointer and allocation check, selector 10's last
writer, ordinary and generated execution, original BIOS, Linux
and USA remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E591 European indexed exchange and direction lists

At `0c3ac6` the context initializer requests `0x400` bytes and
stores the returned pointer at context offset `0x204`. It can
return the callbacks at `0c27c0` and `0c305c`. Both selected
call sites pass the buffer and zero direction and mode inputs
to `0b82a4`. They store its count as a halfword at `0x20c` and
pass that count with signed extension to `0c12c8` along with the
same buffer pointer. The second caller enters the sorter only
when the count is positive.

The sorter skips counts below two. Its outer index runs from
zero through count minus two and the inner index from the next
entry through count minus one. Three four-byte copies perform
each exchange through a stack temporary. The builder advances
four bytes per emitted entry. Within the enumerated paths its
record writes target the first three bytes.

Source plan `8qonq3tc` joins every pair of the 347 linked metadata
records and 376 item records already owned in e349. For the
selected zero-mode input, the 130,472 pairs give ceilings of
8, 32, 4, 16 and 181 records for geometry modes `0x00` through
`0x40` in steps of `0x10`. The actor-flag override gives at most
41 records with the same flag result in both checks. These
bounds fit the 256-entry buffer. They depend on the intact
allocation and authenticated input domains. The original
catalog getters accept unchecked u16 IDs, so the physical
record counts do not establish live IDs or exclude another
writer.

The branch loops at `0b8336` and `0b83ba` read four bytes at
`399568` and two bytes at `39956c` respectively. The bytes at
`39956e` and `39956f` are outside those loops. The six-entry switch
at `0b7c28` and the fill vector word already have e349 owners.
The 13- and five-entry tables at `0bb5ac` and `0c3084` leave
72 destination bytes. Thirteen further support words leave
52 bytes. Together with the six direction bytes, the source
delta to integrate is 130 bytes.

The guarded source plan passed 28 input checks. It reuses E574
Wine evidence for the sorter and original caller edges and
authenticates the new paths against the exact EUR ROM. The first
Wine attempt `_25x267l` timed out at startup and drained. Retry
`odifpwrw` passes 30 input checks and drains. Its 1,916 halfwords
match the source plan's disassembly byte for byte. E592 serves the
identified sources. Live input provenance remains in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E592 European indexed exchange tables at runtime

The European `e350-d349` dataset adds the two direction lists
to `engine.adjacent-offset-tables`, the 18 switch destinations
to `engine.thumb-switch-tables` and the 13 support words to
`engine.thumb-pc-literals`. The delta is 130 bytes. The other
153 resource entries match `e349-d348`.

Windows build `glaijg0o` and Wine unit `bqipxuqu` pass. A focused
interpreter check uses altered backing bytes to verify the six
original direction reads, all 18 switch targets, five guard
paths and the 13 literal loads. The 61 typed requests have
matching complete serialized state and guest clock. The two
neighbor bytes at `39956e` remain raw, and all 204 mixed-width
boundary checks across the three ROM mirrors pass. The previous
e349 dataset attaches. Relabeling it as e350 without the three
groups is rejected before replacing its resources.

The default CLI prepares from an empty root, verifies all 157
files and reuses the same generation and selector in the same
Wine lease. The file set, identities, hashes and modification
times match after verify and reuse. All 17 input guards pass and
the server drains. The e350 generation is retained in
`out/e592-validation/staging/target/validation/units/e592-e350-internal-stage-mcafhefi/work/resources`.

The actual array last writer, active metadata inputs, ordinary
and generated exchange execution, original BIOS, Linux and USA
remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E593 European five-byte list and direction bank

The original `09a71c` path allocates one `0x98a`-byte context
and publishes the selected slot at owner offset `0x1c`. The only
encoded direct registration call passes that field to `14c224`.
The registry holds up to 16 nodes of 28 bytes at `03005f30`.
The dispatcher follows the node's context pointer when it calls
`14c524` at `14c3a6`.

The context reserves 256 seven-byte cells at `0x000..0x700` and
128 five-byte slots at `0x700..0x980`. Initialization clears the
list and both cursors at `0x988` and `0x989`. State 2 produces the
first record with kind 15. State 3 advances both cursors modulo 128
as it consumes active records and enqueues records of kinds 0–3.
The call at `14c76c` copies five bytes from the selected list slot
to cell offset 2. The cell index is `16 * row + column`. The seed
coordinates come from the source fields at `0xf6` and `0xf7` via
an actor's context pointer at `+0x3c`. A valid seed coordinate and
the alias between actor and owner slots remain conditions to
verify on a live path.

The six four-byte rows at `b946d4..b946ec` occupy the gap between
the palette-effect dispatch and script archive. The original
forward reader uses indices 1–4 for the seed and `[kind + 1,
kind, kind + 2]` for kinds 0–3. The reverse reader skips the seed
and uses `kind + 1`. Both fetch signed bytes at row offsets 0 and
1. The source plan preserves the entire physical rows, including
the two trailing bytes. The guard at `14c564` allows five
destinations rooted at `14c574`. The prior e350 owners already
serve the producer's `0x988` literal, allocator stride and related
words. The delta is 24 direction bytes, 20 destination bytes
and 30 literal words including the switch root, for 164 bytes.

Wine `18_slpld` captured 1,528 halfwords and drained with all
30 input guards intact. Its original verdict reflects an incorrect
expected register at `14cdce`. Corrected plan `1lt3a6fp` reuses
that log, matches 530 previously attested halfwords and passes all
18 input guards. The live last writers, original scene and
platform checks remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E594 European five-byte list tables at runtime

European `e351-d350` serves all six four-byte direction rows through
`engine.adjacent-offset-tables`, the five state destinations through
`engine.thumb-switch-tables`, and 30 support words including the
switch root through `engine.thumb-pc-literals`. These resources add
164 bytes. The other 153 resource entries match `e350-d349`.

Windows build `4wko239g` and Wine `tvct974j` pass. The original
Thumb interpreter instructions read all six forward rows and the
four reverse rows with altered backing bytes. All five switch
destinations and both bypasses pass with 69 typed requests and
matching full serialized state and guest clock. The mixed-width
checks cover 384 reads across the three ROM mirrors, both adjacent
resource owners and 36 trailing-byte reads. The e350 predecessor
attaches. Relabeling it as e351 with the older content is rejected
before replacing the attached resources.

The default CLI prepares from an empty directory, verifies all 157
files and reuses the same generation and selector under Wine. The
file identities, hashes and modification times match on reuse. All
17 input guards pass and the server drains. The e351 generation is
retained in `out/e594-validation/staging/target/validation/units/e594-e351-internal-stage-j6bvjbkq/work/resources`.

Live owner and actor aliasing, seed and cursor observations, ordinary
and generated projection and reverse walk, original BIOS, Linux, USA
and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E595 European scroll and palette stack producers

The scroll getter at `0d84ac` loads the eight halfwords at
`0300094a..0300095a` into `sp + 0x10` and uses `147dc0` to
copy the full 16-byte block to `sp`. It reads the signed halfword at index
`2 * u8(layer) + u8(axis)`. The three encoded direct calls select
indices 4, 5 and 7. The two setter functions at `00356c` and
`0035bc` write all eight shadow fields. The eight MMIO addresses
and both physical address-table copies have existing typed owners.
The IWRAM values are live and require last-writer evidence in a scene.

The palette task uses two already owned command switches with 12
setup and 14 update destinations. The update's opcode 12 arm at
`0d7ef8` loads three little-endian signed components from the
current program's relative parameter. At `14af68`, the original
callee stores them at `sp + 0x14`, `+0x16` and `+0x18`. It copies
the three pointers from `sp + 0x1c` to `sp + 0x8` through the
12-byte call at `14afa6`. The two processing modes cache or reuse
the three baseline bytes of a 12-byte row at `03003e68`. They
write the palette shadow at `03003860`. The original index limit
is 511 after the unsigned 16-bit end calculation. Task flag bit 1
selects the cached mode.

The plan verifies all 230 headers, 1,135 command records and 25
producer spans in the published European palette catalog. Across 51
programs it authenticates every one of the 503 distinct opcode 12
records, comprising 3,018 owned parameter bytes. Their start and
end indices lie within `1..415`. The three counted lists contain
3, 3 and 2 producer rows, eight bytes each. The e351 join identifies
15 unowned literal words. Eleven are used by the update, one selects
the task callback in the list constructor and three address the
already extracted producer lists. The known payloads and switch
destinations have owners in e351.

Wine `i952tupg` attests 1,276 new halfwords and matches 90
previously attested ones. All 35 input guards pass and the server
drains. E591 covers the third stack copy in the same census. The
live inputs, other program producers and platform checks remain
in the [recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E596 European palette task literals at runtime

European `e352-d351` adds all 15 E595 words to
`engine.thumb-pc-literals`. The 60 bytes cover the update's shadow
and copy vector operands, the task callback and the three counted
list roots. The other 155 resource entries and the prior literal
groups match e351.

Windows build `8otaxay3` and Wine `8894ngti` pass. All 15 original
Thumb loads make typed reads with altered backing ROM words and
match the full reference state and guest clock. All 180 mixed-width
boundary checks pass across the three ROM mirrors. The e351
predecessor attaches. Relabeling it as e352 with the old content is
rejected before replacing the attached resources.

The standard CLI prepares from an empty directory, verifies all 157
files and reuses the same generation and selector. All 16 input
guards pass in one Wine lease and the server drains. The verified
e352 generation is retained in
`out/e596-validation/staging/target/validation/units/e596-e352-internal-stage-ll6dkegp/work/resources`.

Live scroll writers, palette task flag order, dynamic program sources,
ordinary and generated paths, original BIOS, Linux, USA and native
Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E597 European tilemap transfer list bounds

The original constructor call at `0e6168` receives one of three
lists. Two paths select `39a710` with two rows. Another selects
`39a728` with one row, and another selects `39a734` with one row.
The type-4 row at `39a71c` and the type-3 row at `39a734` already
have owners. The type-3 rows at `39a710` and `39a728` use existing
wrappers and LZ77 streams with distinct, previously unowned
descriptors. Their streams produce 128 and 1,656 bytes respectively.

The callback at `0fde94` chooses between `3a1614` and `3a16a0`.
On state zero, original message field `+0x14` selects the latter
when its signed halfword equals `0x013d`. The caller at `0fdf14`
passes five rows for `3a1614` and six for `3a16a0`. A nonzero low
byte at message offset `+0x16` reduces the count to zero in either
branch. The earlier projection stopped after five rows in both
lists. The sixth row at `3a16dc` links through `3a1698` and `b34144`
to 246 LZ77 input bytes at `b34148` that produce 448 bytes.

The source join scans the cited physical row and wrapper format and
finds 828 structurally matching VRAM rows. All 825 E481 rows and
their linked inputs have coverage in e352. The three additional
rows, three descriptors, one wrapper and one stream contribute
310 bytes. Three list base words add 12 bytes. The E174 Red Spring
trace still maps to its already owned row and the original direct
decompression branch. The E509 bounded inputs give no aligned
type-1 row origin. The type-1 path and other source shapes require
separate producer evidence.

Wine `t889c308` authenticates 452 new original halfwords, passes 38
input guards and drains. The recovered list branches establish
the source extents. Live scene selection and recovery of other
computed sources remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E598 European counted type-3 transfers at runtime

European `e353-d352` reads the original list base and count pairs to
extend the type-3 row projection. `graphics.tilemap-type3-transfers`
adds the three rows, their descriptors and the wrapper and 246-byte
LZ77 input for the sixth row at `3a16dc`.
`graphics.tilemap-direct-transfers` adds the three missing base words.
The two packed files gain 322 source bytes in total. Existing source
spans in both files and the other 154 resource entries match e352.

Windows build `524srhho` and Wine `jf_6_agl` pass. The controlled
interpreter test follows six original list setup paths, including
both counts at `0fdf14`, and the zero-count guard. It observes the
three new base loads through typed resources with altered backing ROM.
All three recovered rows run through the original `0db09c` reader and
SWI wrapper with the emulator's BIOS HLE, producing 2,232 bytes through
the direct and staged paths. The output hashes, full states, guest
clocks and 132 mixed-width mirror reads match. The e352 predecessor
attaches, and relabeling it as e353 is rejected before replacing the
attached resources.

The standard CLI prepares from an empty root, verifies all 157
files and reuses the same generation and selector. Sixteen protected
inputs pass, the Wine server drains and its lease is released.
The current generation is retained in
`out/e598-validation/staging/target/validation/units/e598-e353-internal-stage-myupox0_/work/resources`.

The live message and list producers, queued writes, generated dispatch,
other calculated source banks, original BIOS and deferred platforms
remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E645 European save CRC32 table source

Original Thumb reader `0813eee4` XORs the next input byte with the low
byte of the complemented running state. It multiplies that byte index
by four, reads a word at `0813ef04` and combines it with the state shifted
right by eight. The instructions reach all 256 little-endian words at
`397180..397580`. Every word matches the independently generated reflected
CRC32 table for polynomial `edb88320`. Two literal words provide the
24-bit mask and the table base.

The bounded direct-call census through `146000` finds the calls at
`0813f456` and `0813f774`. The first stores the result at save header
offset `0x0c`. The second saves the expected checksum, clears the checksum
and header bytes `0x12` through `0x16`, and compares the calculated result
with the stored value. The next cartridge address, `397580`, has an
independent 16-byte copy at `08050c00`.

Wine source unit `d_089fkr` attests 59 original halfwords and confirms
1,032 unowned bytes in the e374 ownership join. All 16 protected inputs
pass, and the Wine server drains. Result SHA-256
`bf6ba89fd55b3bf6c0800c38c4fae84786adb89dd87373bc0b5555a35b1e4d83`.
E646 below verifies runtime publication. Ordinary save and load remain
in recovery.

The same audit ranks candidate intervals using the e374 manifest and the
retained e345 code atlas. It separates 2,519,968 residual bytes in uniform
runs of at least 1,024 bytes for prioritization. The large cartridge-tail
and post-transfer candidates still need rooted consumers. The 2,306-byte
interval before the E605 tile bank contains 2,270 repetitions of byte
`88` and requires its own reader. The CRC32 lookup establishes its exact
source bounds across the code atlas's candidate markings.

## E646 European save CRC32 table at runtime

`e375-d374` publishes all 256 CRC32 words and their two reader literals
as `engine.save-crc32-table` in `save-crc32-table.bin`. The original
checksum algorithm and guest addresses remain in use.

Windows build `v31npnju` and Wine unit `zsu3v1bs` pass. With altered ROM
backing, the original lookup exercises all 256 table indices and three
CRC cases covering empty input, the standard check vector and a nonzero
seed. The original prologues select 12,228- and 15,528-byte payloads.
For each length, the checksum writer stores the reference value, the
verifier accepts intact data and the mismatch branch rejects one
altered byte. The cases match the expected header writes, destination
guards, complete guest state and clock. The reader requests account for
336,280 managed bytes. All 192 mixed-width ROM mirror checks pass.

The CLI creates, verifies and reuses 177 files, with all 175 preceding
resources byte-identical. e374 attaches and two invalid candidates
preserve the active attachment. All 17 protected Wine inputs pass. The
server drains and releases its lease. Stage `tlf2q8ys` retains the dataset.
Result SHA-256 `98f57674add37719cd0eb992ae374522288a57033956cf56734bfe99e00744e8`.

The test supplies synthetic payloads in scratch guest RAM and enters
the original verification block after the signature check. Ordinary
save and load, translated execution, Linux, USA and native Windows
remain in recovery.
