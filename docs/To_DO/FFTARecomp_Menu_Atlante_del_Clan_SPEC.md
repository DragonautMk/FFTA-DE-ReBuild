<a id="fftarecomp--specifica-di-implementazione-del-menu-atlante-del-clan"></a>

# FFTARecomp — Clan Atlas menu implementation specification

> **Operational document for Codex / development agent**
> **Status:** implemented in the native desktop menu; EUR validation on Wine completed October 30, 2026, within the limits below.
> **Project:** `DragonautMk/FFTARecomp`, branch `decomp`
> **Verified baseline:** `999d0a617296cae3b2ad9b585f1df03faa45a134` (October 29, 2026)
> **Objective:** implement the approved interface **in the native Rust menu**, preserving all available functionality and clearly distinguishing still-planned Debug V2 features.

**Migration closure (October 30, 2026).** Implementation starts from the actual
branch at commit `945966c`, after the historical baseline cited here: DBG2-01…09
were already available and remain so; DBG2-10…28 are planned. Acceptance checks
below refer to Windows GNU release builds and **Wine** tests, with mouse/keyboard
and simulated XInput, handler inventory and targeted tests. They do not attest
native Windows, Linux, physical controllers, hardware GPU drivers or the USA ROM,
which remain to be verified. At the user's request, performance was not measured.
Preserved operations reuse existing backends and checks; not every CLI procedure
or ROM-data test was rerun. Documentation, startup commands and results:
[native menu](../menu_clan_atlas.md),
[before/after inventory](../development/MENU_ATLAS_INVENTORY.json),
[changed files and validation](../changes/2026-10-30-implement-native-clan-atlas-menu.json).

<a id="0-istruzione-principale-per-lagente"></a>

## 0. Main instruction for the agent

Implement a new native pause/debug menu in FFTARecomp called, at the *design*
level, **“Clan Atlas”**. The objective is a new graphical interface and feature
organization, **not** replacement of the game engine, save system or debugger.
Preserve public APIs, file formats, handlers and existing semantics; change
routing only where necessary and with tests. Do not convert the application to
HTML/Electron/WebView/Godot: the HTML prototype was a visual and behavioral
reference, not the target technology.

**Non-negotiable constraints:**

1. **Zero regressions:** no existing item may disappear, become unreachable or lose effects/validation. Keep the original menu as a comparison reference, not a second divergent implementation.
2. **Permanent side navigation** with the `Gioco | Debug` selector (Game | Debug) at the top on the **left**, not as two horizontal tabs above the content.
3. **Stable container dimensions:** headings, sidebar, tabs, footer, page title, global controls and list/detail areas must not move during scrolling or selection changes. Scroll **only within internal lists**.
4. **Adaptive width:** no window fixed at 1048 px and no horizontal page scroll. Adapt to the game window, rearranging **only internal content** when space is insufficient.
5. **Return to game** must visually be **a panel in the same design system**: same frame, material, margins, padding, hover and focus. No differently decorated or out-of-grid button.
6. New volume and speed shortcuts must work **anywhere in the open menu**, without highlighting a control. The two simultaneous combinations take priority over individual buttons.
7. **Real data in the real build:** no fictional dates, artificial save-state counts, example debug sessions or simulated diagnostic results. Identify missing values as such.
8. **Current Debug ≠ Debug V2:** do not make a V2 item operational until its backend exists and is verified. V2 tasks shown in the UI must clearly be *Planned* or *Unavailable*.
9. Preserve access to operations by **keyboard, mouse and gamepad** with visible focus and correct binding capture.
10. Do not declare an implementation complete if it does not compile, is not connected to real handlers or has not passed the planned tests.

Before changing code, **reread the workspace branch**: files may have changed
since the indicated baseline.

---

<a id="1-sorgenti-da-esaminare-prima-di-intervenire"></a>

## 1. Sources to inspect before making changes

| Path | Role |
| --- | --- |
| `crates/menu/src/lib.rs` | `Menu`, `Page`, `Row`, `MenuAction`, page mapping and input handling |
| `crates/menu/src/theme.rs` | Native pixel/RGBA UI rendering, panel geometry, dropdowns, hit testing |
| `crates/menu/src/menu_icons.rs` | Menu resources and icons |
| `crates/menu/src/debug_catalog.rs` | DS catalog, filters, selection and preview |
| `crates/recomp/src/play.rs` | Menu integration with game, ROM, state, debugger and input |
| `crates/input-config/src/lib.rs` and `storage.rs` | Dynamic keyboard/gamepad bindings, `key.menu` / `pad.menu`, persistence |
| `crates/screen/src/profile.rs` and related modules | Screen models and video configuration |
| `docs/menu_shortcuts.md` | Actual menu opening/closing semantics and custom combinations |
| `docs/debug_tools.md` | Existing debugger tools, CLI and integrations |
| `docs/development/DEBUG_TOOLS_V2_TODO.md` | 28 V2 extensions: **planning, not implementation** |
| `docs/portable.md` | Files, folders, saves, migration and portable mode |

Online references:

- https://github.com/DragonautMk/FFTARecomp/tree/decomp
- https://github.com/DragonautMk/FFTARecomp/blob/decomp/crates/menu/src/lib.rs
- https://github.com/DragonautMk/FFTARecomp/blob/decomp/docs/menu_shortcuts.md
- https://github.com/DragonautMk/FFTARecomp/blob/decomp/docs/development/DEBUG_TOOLS_V2_TODO.md

**Important:** before making changes, generate a programmatic inventory of
current actions and items (including subpages, dropdowns, confirmation dialogs
and CLI commands). Map each action to its destination in the new menu. Prototype
names do not replace actual code identifiers.

---

<a id="2-design-visivo--atlante-del-clan"></a>

## 2. Visual design — “Clan Atlas”

<a id="21-carattere-e-direzione-artistica"></a>

### 2.1 Character and art direction

An interface for a **GBA fantasy tactical RPG**, not a SaaS dashboard or generic
template fantasy. Identity inspired by a **clan ledger / mission atlas**: small
geometric ornaments, simple original crest, restrained serif headings, side
numbering, dark mineral blue and worn brass. Avoid excessive gilding, conspicuous
gradients, floating shadows, rounded cards everywhere and arbitrary decorative icons.

Indicative initial palette (harmonize with the renderer and actual contrast):

| Role | Color |
| --- | --- |
| Deep background | `#091827` |
| Sidebar | `#112B40` → `#1B374D` |
| Panels | `#1C384E` |
| Selected panels | `#2D4E64` |
| Primary text | `#E9EFEC` |
| Secondary text | `#A8BDC7` |
| Brass / important elements | `#D1AF75` |
| Light gold / focus | `#F0D7A4` |
| Borders | `#59798B` |
| Confirmation / valid state | `#8BC6AD` |
| Warning / destructive effect | `#C78B77` |

- Use a serif font existing/available in the project for headings; use a readable, consistent font for option text. **Do not embed external fonts unless authorized and legally distributable.**
- Use a **minimal original** crest built with native geometric primitives; avoid logos or artwork copied from the game.
- Highlight active rows with a brass border and change of blue tone; retain clear contrast on hover/gamepad focus.
- Panel borders must be thin and consistent. A small metallic corner is enough; intricate frames everywhere are unnecessary.
- Each section needs a readable hierarchy: overline (optional), title, short description, **stable work area** and optional action footer.

<a id="22-layout-di-riferimento"></a>

### 2.2 Reference layout

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ ✥ FFTARecomp                                        THE CLAN ARCHIVE  │
├────────────────────┬────────────────────────────────────────────────────┤
│ [ original crest ] │ Path / section                         [Events][CMD]│
│ FFTARecomp          ├────────────────────────────────────────────────────┤
│ PAUSE & DEBUG MENU  │ VOL n% | FAST n% | SAVE/SESSION | FRAME ...       │
│ [ Game | Debug ]    ├────────────────────────────────────────────────────┤
│                    │ Page title / fixed subtitle                       │
│ 01 Overview        │ ┌────────────────────────────────────────────────┐ │
│ 02 ...             │ │                                                │ │
│ 03 ...             │ │    CONTENT — changes, with internal scrolling │ │
│ 04 ...             │ │                                                │ │
│ ...                │ └────────────────────────────────────────────────┘ │
│                    ├────────────────────────────────────────────────────┤
│ OPEN / CLOSE MENU  │ VOLUME L1[-] L1+R1[MUTE] R1[+]                   │
│ [SELECT]+[START]   │ FAST   L2[-] L2+R2[ON/OFF] R2[+]                 │
│ dynamic binding    ├────────────────────────────────────────────────────┤
│ ◂ Back             │ Operation status/result                           │
└────────────────────┴────────────────────────────────────────────────────┘
```

<a id="23-adattività-senza-spostamenti-dovuti-allo-scroll"></a>

### 2.3 Adaptability **without scroll-induced movement**

The final prototype showed an error **not to reproduce**: fixed horizontal
sizes requiring the entire interface to scroll. The final UI must respect the
available window.

Rules:

- **Height and position** of sidebar, top bar, title, status bar, shortcut footer and L/R bar are determined by window layout and remain fixed while the user scrolls.
- **Flexible width:** proportional sidebar bounded by a sensible minimum/maximum; remaining width to the main panel. Measure real text and hit boxes in the native renderer.
- At wide widths, long settings and Debug tools use **two internal columns**. The Debug list scrolls in its own column; details, title and confirmation actions **remain in the adjacent column**.
- At narrow widths the **content columns** may become two vertically stacked internal areas, each with its own bounds and scrolling. The sidebar does not turn into a top bar. Do not let content grow the window or push footers off-screen.
- On extremely small screens, provide a readable, navigable compact mode or state a real technical renderer minimum. Do not use horizontal overflow of the whole UI.
- When changing a value or selecting a row, retain the **relevant list's** scroll position, focus and detail-area rectangle. Content changes, not geometry.
- Only lists or text fields that need it should scroll. No global scrollbar that moves `Torna al gioco` (Return to game), the sidebar or controls.
- The home page's `Torna al gioco` (Return to game) is **inside a fixed-height panel in the grid**, using the same colors, border, focus, inner margins and typography as other items. It is a primary action, not a banner outside the grid.

Derive numerical width/height values from canvas/frame dimensions; **do not
hard-code 1048×720 px as the only supported resolution**. For lists use clipping
and scroll index/offset consistent with hit-testing calculations.

---

<a id="3-sidebar-struttura-da-preservare-esattamente"></a>

## 3. Sidebar: structure to preserve exactly

The **Game / Debug** selector is **at the top of the left column**. Each tab has
its own list. Switching tabs must not arbitrarily reset the previous section or
the selected quick save.

<a id="scheda-gioco--8-voci"></a>

### **Game** tab — 8 items

1. **Overview**
2. **Saves**
3. **Fast Forward**
4. **Text windows**
5. **Audio**
6. **Video**
7. **Controls**
8. **System**

<a id="scheda-debug--8-voci"></a>

### **Debug** tab — 8 items

1. **Overview**
2. **Sessions and cases**
3. **Replay and comparisons**
4. **Execution and memory**
5. **Traces and inspectors**
6. **Console and scripts**
7. **Cheats**
8. **Reports and archive**

The Debug tab **must not be reduced to a few decorative cards**: the user must
be able to access the real option inventory. Each Debug group supports a
**“Current Menu/CLI”** view and, where relevant, a **“Planned Debug V2”** view.

At the bottom of the sidebar, always visible: **“OPEN / CLOSE MENU”** + current
buttons (e.g. `SELECT + START`) + device indication. Read the binding from the
actual configuration (`InputConfig`, `menu_key`, `menu_pad`) and controller
display mapping. Verified defaults are `Escape` on keyboard and
`Start + Back/Select` on controller; the command can be remapped. `Guide/Mode`
remains an additional path where supported. Combinations trigger once per
complete press and must not repeat on partial release; `Escape` retains the
documented recovery path.

---


<a id="4-scheda-gioco-comportamento-completo"></a>

## 4. Game tab: complete behavior

<a id="41-panoramica"></a>

### 4.1 Overview

**Fixed vertical order:**

1. **Return to game** — inside its panel, aligned with the others.
2. **Quick saves** — immediate selector (without entering Saves).
3. **Volume** and **Fast Forward** — two side-by-side panels at wide widths, arranged in the same compact internal area at narrower widths.

The geometry of the first two panels must not depend on the height of the two
lower controls. If these need more space, **they scroll only in their own area**.

<a id="42-salvataggi-rapidi-e-salvataggio-standard"></a>

### 4.2 Quick saves and standard saving

Preserve **Save states**, **Load state**, **Save state**, **Replace state with
confirmation**, **Undo last operation** and related errors/unavailable states.
Support **9 standard slots** and **99 slots** when `Extended saves` is enabled,
without hiding the remainder or limiting them to the first 9.

- When reopening the menu, **the last quick slot used/saved must remain selected**, ready to **Overwrite** or **Load**. Preserve selection, existing slot, any undo and current state correctly.
- Selecting a slot **must not save it**. Load is disabled for empty slots. Overwriting requires confirmation if already required by the current handler.
- Show the real slot timestamp when available, otherwise “Date unavailable”. Do not invent dates.
- Below the selector show **“Last standard game save”** with actual date/time from reliable `.sav` save information. If file `mtime` is used, label it **“Last .sav file modification”** rather than inventing an internal game timestamp. If no save exists, say “No standard save found”.
- Quick saves and standard saves are **two separate systems**. Do not read one's date from the other.

<a id="43-salvataggi--gestione-completa"></a>

### 4.3 Saves — complete management

In addition to quick access in Overview:

- `Extended saves` ON/OFF, compatibility with the on-disk structure and 99 slots.
- `Game saves`: slots, availability, active cartridge slot, **Use and restart**, **Copy current save**; explicit confirmation for replacement/overwrite.
- `Save backups`: **Copy 1 — latest**, **Copy 2 — previous**, **Restore and restart** with confirmation; backup selection and error handling.
- Protect against data loss on cancellation, empty slot, invalid operation, changed ROM or incompatible copy.

<a id="44-controlli-globali-di-volume-e-fast-forward"></a>

### 4.4 Global volume and Fast Forward controls

**Always active while the menu is open**, including in Debug and when focus is
on lists, headings, other controls or dropdowns where interaction must not be
consumed by the field being edited.

| Proposed combination | Result |
| --- | --- |
| `L1` | Volume −10 percentage points; range 0–100% (consistent with the current 0.1 steps) |
| `R1` | Volume +10 percentage points |
| `L1 + R1` | Toggle `Mute`, **preserving the volume value** |
| `L2` | Fast Forward −20 percentage points |
| `R2` | Fast Forward +20 percentage points |
| `L2 + R2` | Toggle Fast Forward, **preserving the selected speed** |

Disambiguate the combinations: `L1+R1` must not first adjust volume; `L2+R2`
must not first change speed. Give chords precedence, with consistent press/
release/edge handling and repeat. Tests must include nearly simultaneous presses
and release of just one button. When the menu is closed, buttons remain assigned
to the game according to existing bindings, without taking L/R or zoom from gameplay.

**Fast Forward in 20% steps:** visual settings and runtime must agree. Designed
requirement: `100%, 120%, 140%, …, 800%` (1.0×–8.0×); turning Fast Forward off
actually returns the game to **100%** without losing the preferred value. The
current code supports `2×`, `4×`, `8×` and audio `silent | fast | normal`:
**scheduler/emulation, audio clock, serialization/configuration and tests**
would also need extension, not just drawing a new percentage. If this subtask
is not yet working **do not display arbitrary values as operational**: show the
current behavior and keep the extension as incomplete work.

Momentary/toggle activation for the original bindings must keep working; do not
replace `ff_key`/`ff_pad` or their toggle options without integrating both semantics.

### 4.5 Fast Forward

- Selected/effective speed, ON/OFF and global controls.
- **Audio mode**: `silent`, `fast`, `normal` (retain implemented behavior).
- Separate feedback for “configured speed” and “Fast Forward active”.

<a id="46-finestre-di-testo"></a>

### 4.6 Text windows

Keep **Size** and **Length**, with the project's actual ranges and steps: `Size`
0.5–2.0 in 0.1 steps, `Length` 1.0–2.0 in 0.1 steps.

### 4.7 Audio

- Real volume 0–100%; persistent Mute independent of level.
- **Enhanced Audio** ON/OFF.
- Changes applied through the original `Changed`/handlers.

### 4.8 Video

All original menu items:

- **Aspect**: `auto`, `3:2`, `16:10`, `16:9`, `20:9`, `21:9`, `32:9` where confirmed by current constants.
- **Fullscreen (F11)**.
- **Screen**: `raw`, `unlit`, `frontlit`, `backlit`, `classic`.
- **Brightness** 0.5–1.5 in 0.1 steps.
- **Darken**: `auto` or 0.0–1.0.
- **Response**: read `ResponseMode::ALL` exactly from the branch.
- **Grid**: `auto` or 0.0–1.0.
- **Scaling**: use exact `SCALING_MODES` values from code; do not invent options.
- **Vsync** and **FPS counter** (`off`, `internal`, `external`, `both`).

Parameters present **only in configuration** and not in the original menu, such
as `display_gamut` and `response_keep`, may be grouped under “Advanced” if
correctly labeled and actually validated, without losing file configuration.
Where a change requires restart/relaunch (e.g. some aspect ratios), state that
instead of falsely presenting it as immediate.

<a id="49-controlli"></a>

### 4.9 Controls

Three internal tabs: **Keyboard**, **Gamepad**, **Mouse + keyboard**. Preserve:

- Bindings `A`, `B`, `L`, `R`, `Start`, `Select`, `Up`, `Down`, `Left`, `Right`.
- **Open menu**, **Fast Fwd**, **Toggle Fast Fwd**, **Zoom In**, **Zoom Out** bindings.
- **Default layout** for gamepad.
- **Mouse & Keyboard** ON/OFF and tab-switching logic.
- Gamepad label detection, D-pad source configuration and deadzone where present in configuration, without inventing existing UI features.
- Chord capture and `input.cfg` persistence without breaking press/release, cancellation or recovery `Escape`.

The prototype's text inputs must not become the main remapping method: retain
**native device capture**.

<a id="410-sistema"></a>

### 4.10 System

- **Restart ROM**.
- **Switch ROM**: `Return to game` / `Choose ROM…`.
- **Exit**: confirmation `Yes, exit` / `No, go back`.
- Consistent messages/cancellation; no unexpected game closure.

---

<a id="5-scheda-debug-inventario-da-mantenere"></a>

## 5. Debug tab: inventory to retain

Items are grouped according to the approved interface; **handlers remain those
in the repository**. Where a command is available only through CLI, label it
“CLI” until actual UI integration. Every action requiring an isolated copy
must continue to block the operation when there is no session.

<a id="51-sessioni-e-casi"></a>

### 5.1 Sessions and cases

- `Extended saves`
- `Game saves`
- `Start debug session`
- `Close debug session`
- `Debug catalog`
- `Start input recording`
- `Stop input recording`
- `Copy diagnosis` (may also be visible in Reports)
- Debug catalog: **Region**, **Recorded status**, **Kind**, **DS case**, **Entry**, **Import to free snapshot slot**, **Open isolated case**, **Choose / reload catalog**, with preview and error messages.
- CLIs linked to sessions, catalog and recordings, if already supported and validated.

<a id="52-replay-e-confronti"></a>

### 5.2 Replay and comparisons

- `Replay case`
- `Choose replay profile`
- `Cancel replay`
- `Audit replay`
- `Capture on trigger`
- `Compare first difference`
- Any **existing** CLI campaigns and audits, clearly labeled when not integrated in the menu.

<a id="53-esecuzione-e-memoria"></a>

### 5.3 Execution and memory

- `Step 1 frame (F7)`
- `Step 10 frames (F8)`
- `Resume (F6)`
- `Inspect PC / registers`
- Operations available in `debug-control` (pause/status/inspect/break/watch), distinguishing compiled/interpreter support; do not invent missing capabilities.
- Future named triggers and on-screen pinned values are *V2*, not existing features.

<a id="54-tracce-e-ispettori"></a>

### 5.4 Traces and inspectors

- `Capture graphics report`
- `Start UI / input trace`
- `Save UI / input report`
- `Start audio capture`
- `Save audio report`
- `Start code trace`
- `Save code report`
- `Start task / script trace`
- `Save gameplay report`
- `Start recent events`
- `Freeze / save recent events`

<a id="55-console-e-script"></a>

### 5.5 Console and scripts

- Show existing local/CLI tools and their actual capabilities.
- Provide entry points for **Event console**, **Command console**, **Scripts and recipes**. At the baseline these are **planned V2 extensions**: demo windows are not a working implementation.
- When consoles become operational, they must be **openable from the top bar as an anchored overlay** without changing underlying panel geometry. Event console read-only; command console with validation and a dispatcher shared with the menu, not an arbitrary shell.

<a id="56-cheat"></a>

### 5.6 Cheats

- Local CH catalog and import **only when DBG2 dependencies have been completed**.
- Do not include or publish the gitignored private catalog or its codes in the repository.
- Do not claim the **341 documented blocks** are verified, usable or decoded; at the baseline they are `not-runtime-tested`.
- Apply only to an isolated debug copy, with confirmation, compatibility and provenance; `Disattiva` (Disable) is not rollback. Require an explicit checkpoint to undo effects.

<a id="57-report-e-archivio"></a>

### 5.7 Reports and archive

- `Copy diagnosis`
- `Create issue report`
- UI/audio/code/gameplay/recent reports in continuity with existing handlers.
- CLI tools and scripts available for the homepage and artifact cleanup/preview, protecting evidence and saves.

<a id="58-debug-tools-v2--esattamente-28-attività-pianificate"></a>

### 5.8 Debug Tools V2 — exactly 28 planned tasks

Show a filter or separate **“Debug V2 — planned”** view with stable codes and
names from the original checklist. The UI must not present these as completed.

| Group in new sidebar | Tasks |
| --- | --- |
| Sessions and cases | `DBG2-01` Action inventory; `02` Shared catalog; `03` Internal selector; `04` Choice migration; `05` Parameters and presets; `06` Script and tool library; `07` Portable persistence |
| Traces and inspectors | `DBG2-08` Event schema; `09` Host selections/actions; `10` Runtime events; `11` Live event console; `12` Limits and performance |
| Console and scripts | `DBG2-13` Command language; `14` Editor/completion; `15` Structured responses/jobs; `16` Recipes and batches |
| Cheats | `DBG2-17` Catalog import; `18` Decoder; `19` Controlled application; `20` Cheats page; `21` First EUR subset; `22` Advanced cheats; `23` Provenance/checkpoint/replay |
| Execution and memory | `DBG2-24` Named triggers and stops; `25` Pinned values and differences |
| Reports and archive | `DBG2-26` Timeline; `27` Regression checks; `28` Guide/distribution |

**These tasks cut across groups:** this placement supports navigation; it does
not replace the dependency sequence in `DEBUG_TOOLS_V2_TODO.md`.

---

<a id="6-pattern-di-interazione-da-implementare"></a>

## 6. Interaction patterns to implement

<a id="61-pannello-a-due-aree-per-gli-strumenti-debug"></a>

### 6.1 Two-area panel for Debug tools

The list and details must be two areas **with independent geometry**:

```text
┌───────────────────────────────┬────────────────────────────────────────┐
│ [Menu + CLI] [Planned V2]     │ Tool title + availability              │
├───────────────────────────────┤────────────────────────────────────────┤
│ [01 Tool A]                   │ Parameters / filters / info            │
│ [02 Tool B]                   │ Internal scroll ONLY if needed         │
│ [03 Tool C]                   │                                        │
│     INTERNAL SCROLL ↓         │                                        │
├───────────────────────────────┤────────────────────────────────────────┤
│                               │ [status/prerequisites] [Start / Confirm]│
└───────────────────────────────┴────────────────────────────────────────┘
```

The title and start button remain fixed even with long descriptions. Selecting
an item does not move the detail panel to another coordinate. When width is
insufficient, **rearrange only these internal areas** in a vertical split,
assigning a relative fixed height to each.

<a id="62-focus-mouse-e-controller"></a>

### 6.2 Focus, mouse and controller

- Focus/tab and D-pad navigation, confirm/cancel buttons, mouse hover/click, wheel and scrollbar where present.
- Focus and selection are distinct states from mouse hover alone, subject to existing implemented rules. Do not introduce double execution for held input.
- Do not change a list's scroll offset when updating a value or selecting an item. Reset it only when deliberately entering another section or changing the data source.
- Confirmation modals overlay the current area without changing geometry; `Cancel` returns to the previous context with focus restored.
- Before executing a destructive command show target, slot, profile or session and its real effect. Do not couple selection and execution.

<a id="63-stato-e-persistenza"></a>

### 6.3 State and persistence

Clearly separate:

- Persistent preferences (`AvConfig`, input cfg, video options, preferred speed when supported).
- Current session state (Fast Forward ON/OFF, active debug copy, recordings, jobs).
- UI state (current tab, selected item, scroll offset, filters, open dropdowns).
- Metadata read from disk (save timestamp, available files, backups, catalogs).

Do not overwrite configuration files for purely visual operations. Do not make
gameplay depend on temporary UI data.

---

<a id="7-architettura-tecnica-consigliata"></a>

## 7. Recommended technical architecture

**Reuse** the renderer in `crates/menu/src/theme.rs` and handlers in
`crates/menu/src/lib.rs`. Possible incremental refactor:

1. **Navigation catalog:** for each item show `id`, `label`, `group`, support status, action or destination page and any prerequisites. Refer to real `Row`/`MenuAction` values without duplicating debugger logic.
2. **Native layout:** deterministic function calculating `sidebar`, `header`, `body`, `footer`, `tool_list`, `detail`, `modal` rectangles from actual width/height; one result for rendering and hit-testing.
3. **Rendering:** palette and reusable components (`Panel`, `NavRow`, `SettingRow`, `SlotButton`, `KeyCap`, `ConsolePanel`, `ConfirmDialog`). No unnecessary external assets.
4. **Input:** global routing of L/R combinations **only while the menu is open**; combination priority and capture/consumption of individual events. Handle conflicts with remapping and gameplay controls.
5. **Submenus:** real functions remain connected to handlers; retain `Changed` transitions so the runtime immediately applies updates it already supports.
6. **Debug menu:** reuse real selectors/existing DS catalog; integrate CLI-only functions only if designed/validated. Avoid fake “Execute” buttons that do nothing.
7. **Performance:** avoid rebuilds, allocations and framebuffer invalidation when page, state, animation and selection do not change. Test both GPU/CPU presentation paths where supported.

If the refactor needs new modules, keep them inside `crates/menu/src/` with
precise responsibilities (e.g. `layout`, `navigation`, `widgets`), avoiding
turning everything into one enormous file.

---

<a id="8-piano-di-lavoro-raccomandato"></a>

## 8. Recommended work plan

<a id="fase-a--sicurezza--inventario"></a>

### Phase A — Safety / inventory

- Read and map all `MAIN_ROWS`, `rows_of(Page::...)`, `MenuAction` rows, all configurable options and shortcuts.
- Prepare a **before/after inventory** and a small “original item → new section → handler” matrix to retain in development documentation.
- Run current tests and record the outcome. Do not remove existing tests.

<a id="fase-b--struttura-e-stile-nativo"></a>

### Phase B — Native structure and style

- Implement left navbar and `Gioco/Debug` selector (Game/Debug).
- Add fluid layout with minimum limits and confined scrolling.
- Implement “Clan Atlas” panels and style, including consistent `Torna al gioco` (Return to game).
- Connect all existing actions while preserving confirmations and state handling.

<a id="fase-c--funzioni-approvate-oltre-la-baseline"></a>

### Phase C — Approved features beyond the baseline

- Quick save state in Overview with last slot remembered, overwrite/load and real metadata.
- Last standard save date without fictional sources.
- Open/close shortcut footer based on actual bindings.
- L1/R1 volume and mute chord; L2/R2 Fast Forward and toggle chord with correct event consumption.
- Extend the Fast Forward scale to 100–800% in 20% steps **only after** integrating the engine and timing/audio tests.

<a id="fase-d--debug-e-validazione-completa"></a>

### Phase D — Debug and full validation

- Migrate all current Debug items to the sidebar organized into 8 groups, retaining the `Menu / CLI / V2` distinction.
- Connect the DS catalog, debug session guard and other real handlers.
- Mark V2 items as *planned* or implement them **separately** following DBG2 tasks and their closure criteria, without claiming future features already work.
- Run tests and visual renderer smoke checks at different aspect ratios and text lengths.

---

<a id="9-test-di-accettazione-obbligatori"></a>

## 9. Mandatory acceptance tests

<a id="layout-e-navigazione"></a>

### Layout and navigation

- [x] At 1280×720, 1024×768, 854×480 and a supported narrow viewport: **no horizontal scroll of the main window**.
- [x] Side menu, Game/Debug switch, top bar, title, shortcut footer, L/R buttons and final status remain at the same coordinates while scrolling.
- [x] Debug lists of 3, 9 and 20+ items scroll **only inside** their panel; detail panel does not jump.
- [x] Changing tools or modifying a parameter keeps list scroll and selection stable.
- [x] At narrow width, only internal content is arranged vertically; sidebar, tabs and footer remain in position.
- [x] `Torna al gioco` (Return to game) shares panel frame/padding/focus/hover and stays in the grid.
- [x] No important text is truncated without a way to open details or read the full value.

<a id="gioco-e-input"></a>

### Game and input

- [x] All 12 original menu options are reachable in the new navigation.
- [x] Save/load/replace/undo, 9/99 slots, game saves, copy and backup restore work as before, with confirmations.
- [x] Last quick slot correctly preselected on reopening; selection does not write to disk.
- [x] Real standard timestamp or explicit absence; `.sav` date not confused with save-state dates.
- [x] L1/R1 adjust volume from **every menu page**; simultaneous combo mutes without changing its value.
- [x] L2/R2 adjust speed from **every page**; simultaneous combo toggles without changing it.
- [x] With menu closed there are no regressions in L/R, zoom, fast toggle and GBA input.
- [x] If 20% FF is implemented: tests at 100, 120, 140, 200, 400, 800%, on/off, audio modes, serialization, clock and runtime. Otherwise explicitly identify the incomplete work.
- [x] Remapping the menu combination immediately updates bottom-left keycaps; recovery `Escape` and combo press/release work.
- [x] Mouse/keyboard/gamepad navigation and confirmations tested, including dropdown and modal cancellation.

<a id="debug-e-sicurezza"></a>

### Debug and safety

- [x] **One-to-one** comparison of all `Page::Debug` rows with the new section inventory: no orphan actions.
- [x] DS catalog retains filters, preview, import and error reporting.
- [x] Functions requiring an isolated copy do not start without a session.
- [x] Step frame, resume, trace, report, replay and diagnosis call existing handlers.
- [x] CLIs do not automatically become “Menu actions” without verified integration.
- [x] All 28 DBG2 tasks follow the actual checklist status: 01…09 complete, 10…28 planned; no unverified cheat appears active.
- [x] No `.sav` file, state, log/evidence or artifact is overwritten without the prescribed conditions and confirmations.

<a id="qualità-del-progetto"></a>

### Project quality

- [x] Build and tests of affected crates according to commands documented in the repository.
- [x] Unit tests for geometry/hit-testing, clamp/scroll, tab switch, controller combinations, persistence and confirmations.
- [x] Graphics smoke tests for small/large canvas, CPU/GPU renderer where available.
- [x] Updated documentation when shortcuts and options change.

---

<a id="10-output-richiesto-allagente-che-implementa"></a>

## 10. Output required from the implementing agent

Deliver:

1. **Real code in the repository** (not a new HTML mockup) and a list of changed files.
2. Complete **“before → after” inventory** of menu/debug items with corresponding handler.
3. Brief documentation of the sidebar structure and adaptive layout logic.
4. Test results and instructions for starting the changed menu.
5. Precise list of features **completed**, **still working as before**, **still to be implemented** and any limits encountered.
6. If 20% Fast Forward was not implemented in the engine, explicitly say so and do not make the UI appear to apply nonexistent speeds.

<a id="definizione-di-completamento"></a>

### Definition of completion

The task is complete **only** when the native menu presents “Clan Atlas” style,
the approved sidebar, adaptive horizontal layout with stable geometry, integrated
`Torna al gioco` (Return to game), desired shortcuts and **all preexisting
functions still reachable and operational**, with passing tests. Every Debug V2
feature not implemented must remain declared future work, not simulated as real.

---

*Specification written from menu design requests and the `decomp` branch
inventory consulted October 29, 2026. Before proceeding, check for commits after
the baseline and adapt the map without losing functions introduced meanwhile.*
