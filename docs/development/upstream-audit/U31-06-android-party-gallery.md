# U31-06: Android Party gallery touch comparison

Status: **ADAPT candidate; a bounded host native run bypassed the upstream
Party hook, implementation and device validation pending**. This covers only the original
Party gallery, not the fork's Atlante menu or all Clan, Area and roster
screens. The pinned upstream target is
`6bdddbc559ac459c4a18325c15c28d040d87f407`, compared with fork HEAD
`83f245232c7dbd0206d4fea2a556e031e98fbf37`. The Android and desktop trees
are distinct. See the [Android overview](U31-06-android-touch.md) for gesture
delivery and the separate dialogue candidate.

## Existing and upstream paths

Both Android `Engine::tap` implementations convert normalized screen
coordinates to the relevant panel or world coordinates and dispatch to
`LanguageMouse::aim` or `ffta_battle_mouse::aim`. Unhandled taps can fall back
to an A pulse. In the fork, `LanguageScene::read` covers startup, frontend,
keyboard and name confirmation; the battle handler tries shop, world, facing,
generic menu and battle-cell paths. No Party gallery-specific recognizer or
roster module was found in these fork callers. A generic handler could still
match another live scene; the bounded EUR comparison below tests one instance.

The upstream `ffta_battle_mouse::aim` checks its roster scene ahead of shop
and generic battle menus, after higher-priority prompt/Notice handling. In
`ffta_roster_mouse::Scene::read`, active requires touch enabled, the Europe
profile and one of three guest owner values. The Party gallery path additionally
checks its owner, an EWRAM manager pointer, child state 3, a valid selected
index and a count from 1 to 24. It derives hit boxes from each live member's
coordinates and scroll offset. Overlapping hits choose the nearest center.
The recognizer and its exact addresses are version-specific; neither this
comparison nor synthetic memory proves correctness for another region.

A gallery tap queues a request with owner, target, confirmation and deadline.
The upstream original-owner entry at `0x08075466` checks the request, scene
and instruction signature before modifying gallery selection fields. Its key
handler waits for a committed and ready selection before queuing A. Existing
physical input cancels pending synthetic work. The gallery's top strip has
separate tool actions, so a proposed member-selection unit must not accidentally
claim those actions, other roster lists or scrolling.

The upstream file also defines `publish_party_click`, which uses a private CPU
and stack to invoke a guest publisher. A search of the pinned `AndroidPort/`
tree found its definition but no call; its existence is not evidence that the
Android tap path invokes it. Its failure path restores the temporary stack but
does not by itself establish rollback of all guest effects. Do not copy or rely
on that helper without a separate caller and safety analysis.

## Hook, cache and fork boundaries

Upstream roster state extends `ffta_battle_mouse::State::interprets`; Android
`Engine::frame_audio` declines a native block when the current PC matches that
predicate, and `Machine::step` reaches `ffta_battle_mouse_entry` and then
`ffta_roster_mouse::entry`. The fork already has an interpreter gate for
other mouse hooks, but it has no roster branch. A port must prove that native
block boundaries cannot bypass the intended hook when a block begins earlier;
the presence of a predicate alone is not that proof. Native and interpreter
execution must yield the same selected member and confirmation without extra
guest actions. Check pending-request cancellation and lifetime across scene
changes, touch disable, pause, back and app lifecycle.

The fork Android build tag hashes the Android Rust source trees, build script,
Android build script and target triple; the engine also includes the ROM digest
in native object cache names and checks object contents against a stored hash.
Those source facts are not a cache compatibility or invalidation test. Verify
cold and warm builds after any hook change, including actual native block
execution, and do not delete or reuse another task's cache. The desktop fork's
ROMDATA, recording and debug infrastructure is not evidence of equivalent
Android support; this comparison found no `rom_data`, `rom_resources` or
`dataset` references in the three inspected Android crate source trees. A
future cross-tree implementation requires its own analysis rather than copying
desktop assumptions. EUR-only gallery behavior must leave existing USA controls
and generic handling intact.

## Observed test evidence

On 2026-11-02 the pinned upstream Android workspace was extracted into an
isolated temporary directory and the following host-side command ran offline:

```text
cargo test --locked --offline -p gba-core gallery_ -- --test-threads=1
```

All three matching upstream synthetic tests passed: live roster hit geometry,
hover followed by click, and delayed confirmation. The first models counts
1, 7, 20 and 24; the latter tests manually arrange publisher completion or
selector readiness. They do not run a real guest publisher, execute an Android
tap through JNI, compare pad behavior, exercise native block boundaries or
prove physical-device behavior. Compilation emitted two unused-function
warnings in `ppu.rs`. Those three tests did not exercise the fork. The
subsequent comparisons add host-side fork and one real-scene result; neither
unit exercised an APK build, Android device, emulator or native cache.

### Paired synthetic dispatcher and hook comparison

On 2026-11-02, isolated copies of the current fork Android crates and the
pinned upstream Android crates received the same test modules, without changing
the project runtime. Each was tested on host Linux with `cargo test --locked
--offline -p gba-core`, first selecting `same_gallery_dispatch_fixture` and then
`same_gallery_hook_fixture`; both tests passed in each copy. Separate build
directories were used. Upstream emitted the same two unused-function warnings;
the fork test compilation emitted none.

The common fixture models 20 gallery members, selection 0, the Party owner,
child state 3, live manager coordinates and a touch at member 1. It manually
constructs the Europe-shaped battle/menu profiles and sets the Europe flag;
its ROM initially contains zero bytes and is not an authenticated game. With
released physical keys, direct `battle::aim` returned **handled** upstream and
**unhandled** in the fork. The immediate key value remained `0x3ff` and EWRAM
remained unchanged in both. A blank-space point returned unhandled in both
direct dispatchers. `Engine::tap` itself was not invoked: its subsequent A
fallback is a source observation, not a result of this test.

The second test extends that same synthetic ROM with the upstream hook's
eight-byte guard, calls `aim`, sets Thumb PC `0x08075466`, and calls the mouse
entry method directly. Upstream then sets gallery column 1 and its update flag;
the fork leaves column 0 and the flag clear. Neither call advances the frame.
This invokes a hook method with fabricated inputs: it does not run the guest
instruction, its original publisher, frame scheduling or the native dispatch
loop, and does not establish real-game or pad equivalence. The manually set
profile, lack of other real scene state, direct coordinates and missing ROM
authentication could all change which handler wins in an actual scene.

Temporary fixture modules and four complete logs remain under the isolated
comparison directory while available. The fixture-module SHA-256 values are
`c5a031fe0129e2a75c3d429b9eb15e4bbcd894144ed0df855e7a538ebb413fd5`
and `ffdaf36dcda1a96d1d3b0efe52e41fbf23cfb616d29ae3d137078b1cffa0d56c`;
the first module's gallery helper was made visible to its sibling before the
second test. This evidence supports a difference in the constructed core
model; the subsequent real-scene check independently narrows its applicability.

### One EUR Party gallery through the Android engine on host

On 2026-11-02, fresh isolated copies of the fork and pinned upstream Android
crates were built offline in release mode on Linux. A local test booted the
authenticated EUR ROM with the previously catalogued IN-08 isolated cartridge
save and its existing 3,020-frame boot input, then used the first 93 frames of
the existing world-map-to-Party input. At frame 3,113 both runs reached the
Party gallery with 24 members and selection 0. The 240×160 native framebuffer
captures were byte-identical and visually inspected. They establish the same
captured image, not complete machine-state equivalence.

In each `gba-android` test, touch was enabled, the viewport and world rectangle
were 240×160, and the actual `Engine::tap` received the normalized coordinate
corresponding to native point `(113, 29)`, inside member 1. Both tests ran the
normal frame method with no native blocks loaded. Six frames later, upstream
had selection 1 and remained in the gallery; the fork had selection 0 and had
entered the member screen. After 24 frames, upstream had selection 1 in the
member screen while the fork still had selection 0 there. Thus this one host
interpreter run demonstrates a meaningful difference in the selected member;
it does not exercise JNI, Kotlin gesture delivery, scaled viewports, a compiled
native block, an Android emulator or a physical device.

A separate direct core probe reached the same gallery: upstream's scene reader
recognized the target and `aim` returned handled; the fork's language and battle
dispatch returned unhandled, and its modeled A fallback opened the original
selection. The engine comparison above uses the actual tap method rather than
that modeled fallback. Both engine tests passed; release builds produced
unused-code warnings. ROM, source save and both input scripts matched their
pre-run SHA-256 values after all tests. The first upstream test invocation
selected zero tests because its exact filter omitted the module path; it was
corrected and the one selected probe passed. Local scripts, captures, full
logs and pre/post protection evidence are in `target/u31-party-scene-N2ow4j/`;
the private ROM, save and input contents are not included in this report.

### Pad, scaled viewport and blank area

On 2026-11-02, six new host interpreter scenarios used fresh isolated fork and
upstream Android engine builds and the same protected EUR inputs. Each scenario
replayed the established boot only to reach the new input condition; the prior
native-size touch result was not rerun as acceptance. All six tests passed
their stated observational assertions, including assertions that expose the
current undesirable result. Builds emitted unused-code warnings.

In both trees, holding Right for four frames and releasing for 20 selected
member 1 while staying in the gallery; holding A for four frames and releasing
for 20 opened that member. The selected index, unit identifier, portrait
pointer and spotlight coordinates agreed between trees at both observations.
At the first observation the guest frame was 3,137 and clock 881,086,978; at
the second they were 3,161 and 887,828,481. This establishes the pad outcome
for this scene, not equivalence of complete machine state or timing to touch.

For a second tap scenario the test explicitly configured a 1,280×720 host
viewport and a centered `[100, 0, 1080, 720]` world rectangle, then mapped the
same native point `(113, 29)` through it. At six frames upstream selected member
1 and updated its unit and portrait, while the fork had opened member 0; at
24 frames both had entered a member screen, with their respective selections.
The 24-frame clock was 881,086,976 in both touch runs. The upstream selection,
unit, portrait and final spotlight at that point match the pad's corresponding
selected values, but the pad was still in the gallery at its 24-frame
observation and its clock differed by two cycles. No complete state or exact
timing equivalence is claimed. This tests an explicitly supplied world
rectangle, not the Android UI's calculation or physical touch coordinates.

Finally, both trees received a tap at native `(239, 159)`, below the gallery
hit boxes and tool strip. Both opened the previously selected member 0 after
24 frames. This is the observed existing fallback, not acceptance of that
behavior for a future gallery implementation; a safe empty-area policy needs
to be decided and tested. The ROM, source save and both input scripts retained
their pre-run SHA-256 values. The six logs, test source and protection evidence
are retained locally in `target/u31-party-controls-F3jMeG/`. No JNI, native
block, emulator or device was exercised in those six scenarios.

### Targeted native-dispatch check on host

On 2026-11-02, separate instrumented copies of the fork and upstream Android
engines replayed the same EUR inputs in the interpreter to the existing Party
scene. At that point the test analyzed ROM entry points in the Party routine,
kept 92 blocks within `0x08075194..0x08075780`, and used the project's local
Linux x86-64 TinyCC library to compile them in memory. The compiled table
included the `0x08075466` hook entry. The test then installed this deliberately
limited table into the engine and called the actual `Engine::tap` at the same
native-size coordinate as the earlier comparison. No object cache was written.

Both tests passed. In the upstream run, counters in the isolated test copy
observed the hook once, gated to the interpreter once, and two native Party
block dispatches over the following 24 frames; member 1 was opened. In the fork
run, one Party block was dispatched natively and member 0 was opened; the
instrumented hook counter was zero. This checks only the observed path and
selected compiled blocks: it does not prove every block-entry boundary, the
normal startup compiler/cache, full machine-state equivalence or correctness of
an adapted fork implementation. The initial 3,113 frames were interpreted, not
native. It is Linux host evidence, not Android ARM64, JNI or device evidence.

The authenticated ROM, isolated source save, two input scripts and local
TinyCC library all retained their pre-run SHA-256 values. Both builds emitted
unused-code warnings. Local test source, complete logs and pre/post protection
record are in `target/u31-party-native-FjbEUv/`; no generated ROM code or
private input is included in this report. The required local-artifact
`complete --apply` command refused this unit because its helper only accepts
`target/rom-data-*` directories; the unit and evidence were retained, and no
cleanup is claimed.

### Normal cold and warm native cache on host

On 2026-11-02, new isolated fork and upstream engine copies used their normal
`Engine::prepare` path and the local Linux x86-64 TinyCC library. Each started
with its own empty cache directory, built a 21,298-block base object, then
created a fresh engine and successfully loaded that object from its warm cache.
Before replay, both tests verified that preparation had not advanced the live
machine or changed its in-memory save, that the warm object matched the cold
object's digest, and that its table included the Party hook entry. Host adaptive
background compilation was disabled; this is the host default, not evidence of
Android's adaptive compiler behavior.

Both warm engines replayed the protected EUR inputs to the same gallery with
native dispatch active. The fork test passed its existing observation: after
the member-1 tap it opened member 0. The **upstream test failed** its assertion
that selection would be 1 after six frames: it observed selection 0 and the
gallery still open. The assertion stopped that test at six frames, so the
24-frame outcome and hook/dispatch counters after the tap were not recorded.
This failed test alone does not establish why it differs from the earlier
interpreter and 92-block targeted run. The separate bounded observation below
tests whether the requested hook was reached.

The ROM, source save, input scripts and TinyCC library retained their pre-run
hashes. The isolated cache objects, test source, logs and protection evidence
are retained in `target/u31-party-warm-CMVO0U/`; the failed unit must not be
cleaned as a successful validation. Builds emitted unused-code warnings. No
Android ARM64, JNI, emulator or device was tested.

### Six-frame hook observation

A read-only Sol consultation was requested as `gpt-6.1-sol` with `xhigh`
effort; the agent handle did not independently confirm the effective settings.
It identified direct transfers within compiled groups as a possible route
around the Rust dispatcher. In the retained object, the Party hook entry and
an earlier owner entry are members of the same native group, and disassembly
shows a direct jump to the hook body. This static finding alone did not prove
which path the failed run took.

One new isolated upstream host test explicitly loaded the **same retained
cache object** and repeated the original boot solely to observe the first six
frames after the tap. Test-only counters were added in the isolated Rust copy;
the object was not rebuilt or changed. The live scene reported hit target 1,
and immediately after the tap a valid, uncommitted request for target 1 was
pending. After each of the six frames, selection was still 0, that request
remained uncommitted, and the dispatcher-level hook and gate counters were
zero. Meanwhile a counter in the native 32-bit read callback observed the
hook body's literal read with guest PC `0x08075466` 24 times. This demonstrates
that, in this observed path, native code executed the hook instruction without
the dispatcher invoking the hook; it is consistent with the direct group
transfer found in the object. The exact caller of each execution was not traced.

The observation test passed, and all six protected inputs, including the
retained object, matched their pre-run hashes. Its code, log and protection
record are in `target/u31-party-observe-37LUnX/`. This is one EUR scene on
Linux x86-64, with the prior cache loaded explicitly and adaptive compilation
disabled. It is not a fix or an Android ARM64/device result, and it does not
prove the behavior of every block entry or other hook. The project's required
Terra primary model/effort could not be independently verified in this session.
The required `local_artifacts.py complete --apply` attempt refused this unit
because it only accepts `target/rom-data-*`; the evidence remains in place.

## Candidate unit and stop conditions

A [reviewable Party-only implementation proposal](U31-06-party-implementation-proposal.md)
now specifies a possible pad-driven route, its boundaries and acceptance gates.
It remains subject to the TODO's specific-unit approval requirement.

Before changing code, design and validate a hook-boundary solution that also
handles compiled group transfers, resolve the blank-area policy and verify the
actual device transform. The host evidence compares one different-member target with
pad and scaled tap, including selected unit, portrait, spotlight, guest frames
and clocks; it does not establish complete state or timing equivalence. Define
and validate the desired empty-area behavior instead of implicitly adopting
the existing A fallback.

If a gap is demonstrated and the specific unit is approved, implement only
member selection and confirmation in Android, retaining existing controls and
USA behavior. Explicitly choose between a safe existing pad-action sequence and
a narrowly gated guest-owner hook; do not import the whole roster, rendering,
scrolling or autosave systems. Require tests of valid and invalid owner/ROM
signatures, count and pointers, nonfinite and empty coordinates, overlapping
targets, stale requests, repeated taps, cancel/back/physical controls, touch
disable and scene switches. Include host-layout/zoom mapping and no accidental
underlying or duplicate confirmation. Prove interpreter/native equivalence and
cache behavior, then exercise the named scene on a declared Android device or
emulator with isolated data. Report synthetic, host, emulator and physical
results separately. Stop before runtime mutation if the scene, hook ownership,
native boundary or cartridge/input preservation cannot be established.
