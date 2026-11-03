<a id="fftarecomp--analisi-preliminare-di-integrazione-upstream-v31"></a>

# FFTARecomp — Preliminary upstream v3.1 integration analysis

**Version:** 0.2 — 2 November 2026 (policy decisions approved)

**Type:** decision document and audit plan; policy choices approved by the owner, **not** authorization to change code or merge

**Scope:** upstream `Jorge262007/FFTARecomp` → fork `DragonautMk/FFTARecomp`, branch `decomp`

**Analysis author:** ChatGPT, based on reading the GitHub repositories and release, **without building, merging or running the game**.

> **Approved decision:** keep `decomp` as the authoritative base and integrate **features**, not necessarily upstream commits. Do not run `git merge upstream/main` or bulk cherry-picks without an audit and evidence. The fork follows an _English-first_ policy: if this document is added to the repository, produce a reviewed English translation without changing IDs, links or results.

<a id="0-decisioni-confermate-dal-proprietario--2-novembre-2026"></a>

## 0. Decisions confirmed by the owner — 2 November 2026

The owner confirmed agreement with the **strategic recommendations** in the v0.1 analysis and subsequent summary. This confirmation **authorizes recording the decisions and preparing the audit**, not implicit approval of a merge, code changes, expensive tests or a new asynchronous workflow.

The approved decisions are:

1. **Selective porting instead of a general merge.** The `decomp` branch remains authoritative; compare each upstream feature with existing implementations.
2. **Avoid ROM regressions.** Preserve EUR support and any USA paths already working in practice; do not automatically adopt the v3.1 USA restriction.
3. **Keep the Clan Atlas menu architecture.** Evaluate only relevant improvements to _in-game_ windows, scrollbars, animations and visual fixes; do not replace the fork's menu with upstream's menu.
4. **Mouse/touch and scrollbars: port in small, verifiable scopes.** Prioritize actual improvements to one scene at a time, considering Android and desktop separately.
5. **Machine-state autosave: yes to a separate, configurable design.** Do not replace manual saves, 9/99 snapshots, 99 cartridge saves, Undo or backups. Do not automatically adopt upstream's 5-second period or format: first specify the feature and prove compatibility and recovery. **Default enablement, interval, rotation and the resume experience must be specified before implementation.**
6. **Rendering and transition fixes: only after comparison.** Host improvements may be adapted, preferably with an off switch, without improperly changing guest logic.
7. **150% Fast Forward: not a priority.** Consider a compatible preset only if wanted; do not replace the fork's percentage scale and rational clock.
8. **Steam Deck Y: context-specific/configurable only**, without collisions with Atlas use of the button or existing bindings.
9. **Installer, packaging, color and large refactors: defer** unless there are specific defects or demonstrated benefits; no automatic adoption.

**Status meanings:** `APPROVED_POLICY` = authorized direction; `PENDING_TECHNICAL_AUDIT` = presence, benefits and risks to be established in code; `PENDING_DESIGN` = UX and format details not yet approved; `APPROVED_IMPLEMENTATION` = not assigned to any feature in this version. Do not turn `APPROVED_POLICY` into `VERIFIED`.

**Next step authorized by this document:** a read-only documentation audit of the 12 families, proposing small, separate tasks and acceptance criteria. No runtime changes or direct merge until a specific integration unit is selected and approved.

<a id="1-riferimenti-verificati-e-limiti"></a>

## 1. Verified references and limits

| Role | Reference | Pinned SHA |
| --- | --- | --- |
| Upstream | [Jorge262007/FFTARecomp](https://github.com/Jorge262007/FFTARecomp) — `main` | `6bdddbc559ac459c4a18325c15c28d040d87f407` |
| Fork | [DragonautMk/FFTARecomp](https://github.com/DragonautMk/FFTARecomp/tree/decomp) — `decomp` | `bd66b8763623847af9f093a03fd416e4a10e2029` |
| Common ancestor (from the GitHub comparison) | shared commit | `fcc38cc46084793f29136419a8a15e6914a570ce` |
| Announced changes | [Upstream release 3.1](https://github.com/Jorge262007/FFTARecomp/releases/tag/3.1) | 1 November 2026 |
| Upstream changes | [Diff from the common ancestor](https://github.com/Jorge262007/FFTARecomp/compare/fcc38cc46084793f29136419a8a15e6914a570ce...6bdddbc559ac459c4a18325c15c28d040d87f407) | 5 commits |
| Fork history | [decomp branch](https://github.com/DragonautMk/FFTARecomp/tree/decomp) | 117 commits beyond the ancestor in the cross-comparison |

**Observation:** 173 files changed upstream relative to the ancestor; 30 paths changed **in both** branches (none of the 30 files has the same SHA). The 30 are **potential logical conflicts**, not 30 demonstrated Git conflicts. They include `crates/recomp/src/play.rs`, `main.rs`, `crates/gba-core/src/mem.rs`, `ppu.rs`, `crates/menu/src/lib.rs`, `crates/input-config/src/lib.rs`, `crates/screen/src/ui_layer.rs`.

**Caution:** even a new upstream file can cause functional conflicts through APIs, save formats, input or ROM assumptions. Commit count is not a reliable measure of complexity: v3.1 contains a cumulative update and merge commits.

**Not verified at this stage:** building v3.1, running it, correct application of changes to `decomp`, Windows/Linux/Android tests, Git conflicts produced by a trial merge, actual superiority of the new features, uncommitted changes or local processes. Release notes describe intended and claimed behavior, not independent acceptance.

<a id="2-principi-da-preservare-nella-branch-decomp"></a>

## 2. Principles to preserve on the decomp branch

1. **Prioritize EUR ROMs**, but do not deliberately remove or degrade existing USA compatibility without an explicit decision. Upstream v3.1 states that the USA ROM is temporarily unsupported.
2. **Guest/host separation:** visual or input changes must not alter game state, frames or clock unless a feature deliberately requires a change.
3. **Existing saves:** retain 9/99 snapshot slots, Undo, backups, standard numbered files and compatibility with existing files; upstream uses a different system of automatic machine saves and cards.
4. **Clan Atlas menu:** do not replace custom navigation and layout, `en`/`it` localization, Debug, profiles and shortcuts with the old upstream menu.
5. **Debug and ROMDATA:** preserve accepted audits, recorder, native caches, provenance and tests; do not rerun complete historical tests just to reconstruct context.
6. **Separate Android line:** `AndroidPort/` is a separate tree from the root; do not assume a desktop change also applies to Android, or vice versa.
7. **Asynchronous work:** before changing the working tree, check the status, owner and commits of other tasks; use isolated branches/worktrees and do not overwrite others' files.

<a id="3-matrice-decisionale--indirizzi-approvati-applicabilità-da-verificare"></a>

## 3. Decision matrix — approved directions, applicability to verify

Key: **ADOPT** = integrate the feature after testing; **ADAPT** = reconstruct only compatible behavior; **COMPARE** = first evaluate whether there is an actual benefit; **DEFER** = do not block current development; **DO NOT IMPORT** = do not take the upstream implementation as-is.

| ID | Upstream change | State observed in the fork | Approved policy decision | Difficulty | Priority |
| --- | --- | --- | --- | --- | --- |
| U31-01 | Fast Forward **1.5×** | `ff.percent` from 100% to 800% in steps of 20; **150% unavailable** | **ADAPT:** possible 150% preset in the existing system, without replacing the rational clock | Low–medium | P2 |
| U31-02 | Scrollbars in game menus | Atlas already handles internal scrolling, but this is not the same as scrollbars _in game lists_ | **ADOPT selectively:** geometry/drag for original lists that are actually unsupported | Medium | P1 |
| U31-03 | Window and dialogue bubble fixes | Fork has its own graphics extensions and diagnostic tools | **COMPARE** scene by scene; transfer only reproducible bug fixes | Medium–high | P1 |
| U31-04 | Entrance/exit animations and smooth zoom | Upstream implementation is host-only; fork renderer and synchronization differ | **ADAPT:** isolate UI motion from guest transitions, with an off switch | Medium–high | P2 |
| U31-05 | Widescreen menus without black bars | Atlas is already a native adaptive menu; game rendering is separate | **DO NOT IMPORT** the whole menu; **COMPARE** only original window rendering | High if merged; medium if selective | P2 |
| U31-06 | Improved mouse and touch (especially Android) | Mouse/touch paths exist, Android is separate; new upstream adds many dedicated routines | **ADOPT by area:** Android first; desktop only after checking controls not yet handled | High | P1 Android / P2 desktop |
| U31-07 | Machine autosave every 5 seconds | Fork has automatic `.sav` and manual 9/99 snapshots: **these are different kinds of save** | **COMPARE/PROTOTYPE:** opt-in machine autosave in a dedicated file with safe recovery, without replacing existing features | High | P1 analysis; P2 implementation |
| U31-08 | More than 2 save cards/slots | Fork supports 99 snapshot slots and up to 99 cartridge files; upstream also changes the original save UI | **DO NOT IMPORT** the whole subsystem; check whether the in-game UI offers something different and useful | High if imported | P3 |
| U31-09 | Steam Deck: Y button for menu | In the fork Y/North already changes the Debug view in some contexts | **ADAPT** as a contextual/configurable binding without overriding Y globally | Low–medium | P2 |
| U31-10 | Video rendering/color and graphics caches | Extensive renderer and AndroidPort changes, some not highlighted in the release | **DEFER/COMPARE** isolated components and visual results, without bulk imports | High | P3 |
| U31-11 | Installation/build scripts and packages | Fork already has different Docker/portable workflows | **COMPARE** specific defects; retain fork packaging if it works | Medium | P3 |
| U31-12 | Changed ROM requirements, USA temporarily excluded | Fork targets EUR and USA paths, with separate evidence | **DO NOT IMPORT** the USA exclusion as fork policy | High (regression risk) | P0 constraint |

**P1/P2/P3 remain suggested priorities, not test results or authorization to execute a package.** If an improvement is already present at equivalent quality, mark it `SKIP_ALREADY_PRESENT` with file and evidence.

<a id="4-decisioni-tecniche-motivate-e-criteri-di-accettazione"></a>

## 4. Technical decisions and acceptance criteria

### U31-01 — Fast Forward 150%

- **Evidence:** upstream `ff.speed` allows `[1.5, 2.0, 3.0, 4.0]`; fork `crates/input-config/src/fast_forward.rs` uses `PERCENTS` 100,120,…,800. [Upstream](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/input-config/src/lib.rs) · [Fork](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/crates/input-config/src/fast_forward.rs).
- **Choice:** offer 150% as a step of 10 in a future scale; do _not_ force conversion to 140/160 or fully migrate float `ff.speed`.
- **Acceptance:** unchanged parsing and serialization of old configurations, pacing/clock corresponding to 150%, consistent audio, shoulder ±20 and menu options with stated behavior. Define the semantics of the ±20 sequence starting from 150% before coding.

<a id="u31-02--scrollbar-e-input-mouse"></a>

### U31-02 — Scrollbars and mouse input

- **Evidence:** upstream has `crates/screen/src/ui_scrollbar.rs` with track/thumb/hit/drag/tests and calls scrollbars from `crates/recomp/src/play.rs` for lists (`roster_scrollbar`, `pub_scrollbar`). [Scrollbar](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/screen/src/ui_scrollbar.rs).
- **Choice:** first identify original menus **not yet mouse-scrollable** in the fork. Adapt the geometry model if tests show it is useful; do not add a second Atlas scrolling system.
- **Acceptance:** consistent drag, wheel, controller and original selectors; no double action or wrong data selection; equivalent guest state for the same choice.

<a id="u31-0304--correzioni-visive-e-transizioni"></a>

### U31-03/04 — Visual fixes and transitions

- **Evidence:** upstream adds `crates/recomp/src/ui_motion.rs` (`Instant` transitions, duration ~0.15 s; dialogue closing ~0.22 s) and `crates/gba-core/src/ffta_ui.rs`; it significantly changes `ffta_dialogue.rs`, `mem.rs`, `ppu.rs`, `play.rs` and presentation. [UI motion](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/recomp/src/ui_motion.rs).
- **Choice:** separate objective bug fixes (incorrect images, geometry, pop, focus) from optional aesthetic effects (slide/ease). Do not change the guest solely for host effects.
- **Acceptance:** image and scene tests for targeted cases, before/after comparison, unchanged frames/clock for host-only animation; continuous controls while opening and closing.

### U31-05 — Widescreen

- **Evidence:** the release says the original menu background is a widened image; the fork uses Atlas with a sidebar and adaptive layout. [Atlas](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/docs/menu_clan_atlas.md).
- **Choice:** do not transfer upstream menu appearance or layout into Atlas; separately assess fixes for original game scenes/windows and ultrawide behavior.
- **Acceptance:** no menu regression at 480×360, 640×480, 854×480, 1280×720 and widescreen; no misplaced hitboxes.

<a id="u31-06--android-mouse-e-touch"></a>

### U31-06 — Android, mouse and touch

- **Evidence:** upstream adds `ffta_roster_mouse.rs`, changes `ffta_battle_mouse.rs`, `MainActivity.kt`, `GameSurface.kt`, `engine.rs`, `touch.rs`, adds `host_layout.rs` and other routines; the release claims significantly greater touch coverage. [Upstream Android](https://github.com/Jorge262007/FFTARecomp/tree/6bdddbc559ac459c4a18325c15c28d040d87f407/AndroidPort).
- **Choice:** audit by scene: Clan/Party, Area, Store, Battle, windows and selection. First implement only one Android family in its isolated branch. Do not copy renderer, saves and input together.
- **Acceptance:** touch selections correspond to the same actions on a pad, no unintended selection changes, synchronized host and guest UI, tests on stated devices or emulators (distinct from APK compilation alone).

<a id="u31-0708--autosave-salvataggi-e-ripresa-principio-approvato"></a>

### U31-07/08 — Autosave, saves and resume (approved principle)

- **Evidence:** `upstream/crates/recomp/src/autosave.rs` contains a `Duration::from_secs(5)` trigger, captures `m.save_state()` and queues the result to a worker; it has `QuickSave` and `SaveN` files/game cards. The fork already has 9/99 snapshots with Undo and cartridge files 1–99, _not necessarily the same automatic resume flow_. [Upstream autosave](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/recomp/src/autosave.rs) · [Fork saves](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/docs/extended_saves.md).
- **Approved choice:** do not port the entire ~2,000-line module. Prepare a specification for **machine autosave in a dedicated file**, configurable and independent of `.sav`, manual snapshots, Undo and backups. Default enablement, interval (5 s not approved), a launch resume offer, retention policy and failures/rotation are **still to be defined and validated**.
- **Required acceptance:** interruption during writing, crash/recovery, different ROM/region/version, save in progress, native cache, recordings/replay/debug, disk space, write limits and reopening without corrupting manual files. Check whether every 5 seconds is desirable or excessive; do not impose it as a default without a decision.
- **Note:** 99 snapshot slots do not prove that the v3.1 _in-game_ UI is already equivalent: these are different concepts. Check before dismissing all upstream UX.

### U31-09 — Steam Deck Y

- **Evidence:** upstream specifies Y for the port menu on Steam Deck; in the fork `docs/menu_clan_atlas.md` documents Y/North for changing the Debug page view.
- **Choice:** do not assign Y globally. Consider per-device mapping only with exclusive contexts and persistent preferences; retain user bindings.
- **Acceptance:** no collisions with game buttons, menu, binding capture, Debug or Steam Input.

<a id="u31-10--renderingcolorimetria"></a>

### U31-10 — Rendering/color

- **Evidence:** upstream adds `video/color.rs`, `lut.rs`, `profile.rs` and new graphics/asset cache routines in the Android branch; the presence of a color model alone does not guarantee greater fidelity to the original panel.
- **Choice:** perform a separate technical evaluation only when prioritized; check neutral output, gamma, palette, performance and reversibility. Do not route these changes through ROMDATA and confuse presentation with extracted data.

<a id="u31-1112--installer-e-rom"></a>

### U31-11/12 — Installer and ROM

- **Choice:** retain the fork's existing build/portable scripts unless a specific bug is demonstrated; import only minimal fixes where appropriate. Do **not** inherit the USA restriction introduced by upstream v3.1. USA compatibility must be demonstrated, not assumed.

<a id="5-pacchetti-operativi-suggeriti-a-codex"></a>

## 5. Suggested work packages for Codex

<a id="p0--audit-nessuna-modifica"></a>

### P0 — Audit, no changes

1. Check checkout status (`git status --short`), remotes, pinned SHAs and merge-base; avoid touching other people's processes or changes.
2. Produce a Markdown/JSON file for each `U31-xx` with: upstream intent; API; files; callers; fork differences; available tests; `PORT/ADAPT/SKIP/DEFER` decision; risk; acceptance evidence.
3. Do not read megabyte histories. Use the short `TASK_HANDOFF.md`, `git diff --stat` and targeted file sections.
4. For items already present, find evidence of behavior before recreating them. State uncertainties explicitly.
5. Stop after publishing the report; **no automatic merge**.

<a id="p1--prime-integrazioni-candidate"></a>

### P1 — Initial candidate integrations

1. **Scrollbars in game lists:** module and one sample list; verify hitboxes and controller. Proceed only if the fork actually lacks them.
2. **Reproducible visual fixes:** select 1–2 v3.1 bug fixes, isolating host changes.
3. **Android touch:** one small, verifiable scene, separate from desktop.
4. **Autosave design:** specification and format checks first, without silently adopting the 5-second trigger.

<a id="p2--integrazioni-opzionali"></a>

### P2 — Optional integrations

- 150% Fast Forward as a preset consistent with the fork's scale.
- Host animations and smooth zoom, with a switch to disable and visual comparison.
- New Steam Deck mapping only if configurable and contextual.
- Implement autosave after specification approval, format compatibility and recovery tests.

<a id="p3--non-urgenti"></a>

### P3 — Not urgent

- Video color and the full Android pipeline.
- Upstream menu appearance, if it offers no distinct benefit over the Atlas menu.
- Installer/build overhaul without a concrete regression.

**For each unit:** develop in an isolated branch/worktree, record upstream and fork SHAs, minimal commit, targeted positive/negative tests, result and limits. Integrate into the main branch only after evidence review. Avoid another copy of the entire checklist in the operational handoff.

<a id="6-registro-delle-decisioni-approvate-e-ancora-aperte"></a>

## 6. Decision register: approved and still open

**Owner confirmation received on 2 November 2026.** The D1–D7 **policy** choices are approved; implementation remains subject to an applicability audit, evidence and explicit selection of the next task.

| ID | Policy decision | Status | Details still open |
| --- | --- | --- | --- |
| D1 | Selective porting; `decomp` authoritative base; no bulk merge | **APPROVED_POLICY** | Selection and order of initial units after audit |
| D2 | Do not import USA exclusion; preserve demonstrated compatibility | **APPROVED_POLICY** | Regression matrix by region and platform |
| D3 | Design **separate, configurable, recoverable** machine autosave; retain manual saves, Undo, backups, 9/99 snapshots and cartridge saves | **APPROVED_POLICY + PENDING_DESIGN** | Default on/off, interval, recovery prompt, format, retention, suspension conditions |
| D4 | Retain Atlas; adapt only useful visual improvements and, where possible, make them optional | **APPROVED_POLICY** | Scene-by-scene comparison; animation defaults and options |
| D5 | Android mouse/touch by scene in a separate worktree, without blocking desktop or ROMDATA | **APPROVED_POLICY** | First scene and test platform |
| D6 | Steam Deck Y contextual/configurable only | **APPROVED_POLICY** | Binding priority and actual contexts |
| D7 | 150% Fast Forward not a priority; possible special preset, preserving 100–800% | **APPROVED_POLICY** | ±20 behavior starting at 150%, only if implemented |

**Decisions deliberately left open:** no automatic autosave every 5 seconds, no automatic restore overwriting a session without confirmation, no abandonment of existing slots and no changes to debug tools or ROMDATA architecture. These matters require a verified specification and, when they affect user behavior, further approval.

<a id="ordine-consigliato-dopo-laudit"></a>

### Recommended order after the audit

1. `U31-02` **original menu scrollbars**: a single list with a proven difference from `decomp`.
2. `U31-03` **one reproducible graphics bug fix**: before/after evidence without guest regression.
3. `U31-06` **one Android mouse/touch scene**: isolate from desktop and the general renderer.
4. `U31-07` **autosave technical specification**: design and compatibility analysis, **not** automatic implementation.

For any task after the audit, first produce a proposal sheet with minimal diff, owner, concurrent files, test cost and acceptance criteria. Await explicit selection before changing code or merging.

<a id="7-template-per-i-successivi-aggiornamenti"></a>

## 7. Template for subsequent updates

```md
### U31-XX — Change name

- Upstream source SHA: ...
- Fork baseline SHA: ...
- Source and files: ...
- Upstream behavior demonstrated / only claimed: ...
- Fork equivalent: ...
- Actual difference: ...
- Dependencies / API conflicts: ...
- Decision: PORT | ADAPT | SKIP_ALREADY_PRESENT | DEFER | REJECT
- Rationale and rejected alternative: ...
- Minimum verification and acceptance criteria: ...
- Outcome: NOT_STARTED | INVESTIGATED | IMPLEMENTED_UNVERIFIED | VERIFIED | REJECTED
- Evidence links / test output: ...
- Remaining risks: ...
```

<a id="8-prossimo-controllo-e-criteri-di-completamento-dellaudit"></a>

## 8. Next checks and audit completion criteria

- [ ] Confirm SHAs and merge-base locally in an up-to-date worktree.
- [ ] For each of the 12 families, also check **actual callers** and any equivalent fork implementation.
- [ ] Clearly separate changes in the `AndroidPort/` tree from desktop changes.
- [ ] Check the upstream automatic-copy format and compatibility with the fork's `.ffstate` files **before** writing an adapter.
- [ ] Check whether upstream fixes interfere with hook/debug/ROMDATA and the native cache.
- [x] Obtain agreement on directions D1–D7 (2 November 2026); D3 **specification details remain open**, as do applicability checks.
- [ ] Verify build and test results as evidence separate from the release's publication.
- [ ] Turn approved directions into **proposals** for individual tasks with acceptance criteria and a stop after each unit; obtain approval of the specific unit before changing code.

---

**Conclusion and approved policy decision:** v3.1 contains interesting improvements, especially for usability, but an indiscriminate merge poses a high regression risk. The rational choice is synchronization **by behavior**, preserving the fork's ROMDATA, Debug, menu and save work. This document is a versioned starting point: no item is marked `VERIFIED_IN_FORK` until its specific evidence has been run and accepted.

<a id="9-registro-revisioni"></a>

## 9. Revision history

| Version | Date | Change |
| --- | --- | --- |
| 0.1 | 2026-11-02 | Initial upstream v3.1 vs fork `decomp` audit; recommendations and limitations without execution |
| 0.2 | 2026-11-02 | Owner confirmed strategic directions; distinguished design agreement from authorization to change runtime; autosave details left open |

**Integrity note:** evidence and SHAs for both repositories remain those of v0.1; this revision records owner decisions and does not assert new test results or newly checked commits.
