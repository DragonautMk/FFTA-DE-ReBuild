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


## 4. Scheda Gioco: comportamento completo

### 4.1 Panoramica

**Ordine verticale fissato:**

1. **Torna al gioco** — dentro il suo pannello, allineato con gli altri.
2. **Salvataggi rapidi** — selettore immediato (senza entrare in Salvataggi).
3. **Volume** e **Fast Forward** — due riquadri affiancati su larghezze ampie, disposti nella stessa area interna compatta su larghezze minori.

La geometria dei primi due riquadri non deve dipendere dall'altezza dei due controlli inferiori. Se questi richiedono altro spazio, **scorrono nella sola zona a loro destinata**.

### 4.2 Salvataggi rapidi e salvataggio standard

Preservare **Save states**, **Load state**, **Save state**, **Replace state con conferma**, **Undo last operation** e relativi errori/stati di indisponibilità. Supportare i **9 slot standard** e i **99 slot** quando `Extended saves` è abilitato, senza nascondere i rimanenti o limitarli ai primi 9.

- Alla riapertura del menu, **l'ultimo slot rapido utilizzato/salvato deve rimanere selezionato**, pronto per **Sovrascrivi** o **Carica**. Conservare correttamente selezione, slot esistente, eventuale undo e stato corrente.
- Selezionare uno slot **non deve salvarlo**. Carica è disabilitato per slot vuoti. Sovrascrivere richiede conferma se già previsto dall'handler corrente.
- Mostrare timestamp reale dello slot quando reperibile, altrimenti «Data non disponibile». Non creare date di fantasia.
- Sotto il selettore mostrare **«Ultimo salvataggio standard del gioco»** con data/ora reali ricavate dalle informazioni attendibili sul salvataggio `.sav`. Se si usa la `mtime` del file, denominarla correttamente **«Ultima modifica file .sav»** anziché inventare un timestamp interno al gioco. Se non esiste alcun salvataggio, dire «Nessun salvataggio standard trovato».
- I salvataggi rapidi e quelli standard sono **due sistemi distinti**. Non leggere la data dell'uno dall'altro.

### 4.3 Salvataggi — gestione completa

Oltre all'accesso rapido in Panoramica:

- `Extended saves` ON/OFF, compatibilità con struttura su disco e 99 slot.
- `Game saves`: slot, disponibilità, slot cartuccia attivo, **Use and restart**, **Copy current save**; conferma esplicita per sostituzione/sovrascrittura.
- `Save backups`: **Copy 1 — latest**, **Copy 2 — previous**, **Restore and restart** con conferma; selezione di backup e gestione errori.
- Proteggere da perdita di dati in caso di annullamento, slot vuoto, operazione invalida, ROM cambiata o copia incompatibile.

### 4.4 Controlli globali di volume e Fast Forward

**Attivi sempre mentre il menu è aperto**, anche da Debug e quando il focus è su liste, titoli, altri controlli o dropdown dove l'interazione non deve essere consumata dal campo in editing.

| Combinazione proposta | Risultato |
| --- | --- |
| `L1` | Volume −10 punti percentuali; intervallo 0–100% (coerente con gli step 0.1 oggi presenti) |
| `R1` | Volume +10 punti percentuali |
| `L1 + R1` | Attiva/disattiva `Mute`, **preservando il valore di volume** |
| `L2` | Fast Forward −20 punti percentuali |
| `R2` | Fast Forward +20 punti percentuali |
| `L2 + R2` | Attiva/disattiva Fast Forward, **preservando la velocità selezionata** |

Le combinazioni vanno disambiguate: `L1+R1` non deve prima regolare il volume; `L2+R2` non deve prima alterare la velocità. Precedenza alle chord, con gestione di press/release/edge e ripetizione coerente. Il test deve includere la pressione quasi simultanea e il rilascio di un solo tasto. A menu chiuso i pulsanti rimangono assegnati al gioco come da binding esistente, senza sottrarre L/R o zoom al gameplay.

**Fast Forward a scatti del 20%:** impostazioni visive e runtime devono coincidere. Requisito progettato: `100%, 120%, 140%, …, 800%` (1.0×–8.0×); spegnendo il Fast Forward il gioco torna effettivamente a **100%** senza perdere il valore preferito. Il codice attuale supporta `2×`, `4×`, `8×` e audio `silent | fast | normal`: servirà estendere anche **scheduler/emulazione, audio-clock, serializzazione/configurazione e test**, non soltanto disegnare una percentuale nuova. Se questo sottocompito non è ancora funzionante **non visualizzare valori arbitrari come se fossero operativi**: mostra l'attuale comportamento e mantieni l'estensione come lavoro non completato.

La modalità di attivazione momentanea/toggle dei binding originali deve continuare a funzionare; non sostituire `ff_key`/`ff_pad` o le rispettive opzioni toggle senza integrare le due semantiche.

### 4.5 Fast Forward

- Velocità selezionata/effettiva, ON/OFF e controlli globali.
- **Audio mode**: `silent`, `fast`, `normal` (mantenere il comportamento implementato).
- Feedback separato per «velocità impostata» e «Fast Forward attivo».

### 4.6 Finestre di testo

Mantenere **Size** e **Length**, con intervalli e passi reali del progetto: `Size` 0.5–2.0 a passi 0.1, `Length` 1.0–2.0 a passi 0.1.

### 4.7 Audio

- Volume reale 0–100%; Mute persistente indipendente dal livello.
- **Enhanced Audio** ON/OFF.
- Cambiamenti applicati tramite i `Changed`/handler originali.

### 4.8 Video

Tutte le voci originali del menu:

- **Aspect**: `auto`, `3:2`, `16:10`, `16:9`, `20:9`, `21:9`, `32:9` dove confermate dalle costanti correnti.
- **Fullscreen (F11)**.
- **Screen**: `raw`, `unlit`, `frontlit`, `backlit`, `classic`.
- **Brightness** 0.5–1.5 a passi 0.1.
- **Darken**: `auto` oppure 0.0–1.0.
- **Response**: leggere esattamente `ResponseMode::ALL` dalla branch.
- **Grid**: `auto` oppure 0.0–1.0.
- **Scaling**: usare i valori esatti di `SCALING_MODES` dal codice; non inventare opzioni.
- **Vsync** e **FPS counter** (`off`, `internal`, `external`, `both`).

I parametri presenti **solo in configurazione** e non nel menu originale, come `display_gamut` e `response_keep`, possono essere raggruppati sotto «Avanzate» purché siano etichettati correttamente, con validazione effettiva e senza perdere la possibilità di impostarli dal file. Dove una modifica richiede il riavvio/nuovo lancio (es. certe proporzioni), dichiararlo invece di presentarla falsamente come immediata.

### 4.9 Controlli

Tre tab interni: **Tastiera**, **Gamepad**, **Mouse + tastiera**. Preservare:

- Binding `A`, `B`, `L`, `R`, `Start`, `Select`, `Up`, `Down`, `Left`, `Right`.
- Binding **Open menu**, **Fast Fwd**, **Toggle Fast Fwd**, **Zoom In**, **Zoom Out**.
- **Default layout** per gamepad.
- **Mouse & Keyboard** ON/OFF e logica di cambio tab.
- Rilevamento etichette gamepad, configurazione sorgente D-pad e deadzone dove presenti nella configurazione, senza inventare funzioni UI già esistenti.
- Cattura chord e persistenza `input.cfg` senza rompere press/release, annullamento e `Escape` di recupero.

Gli input di testo usati dal prototipo non devono diventare la nuova modalità principale di rimappatura: mantenere la **cattura nativa del dispositivo**.

### 4.10 Sistema

- **Restart ROM**.
- **Switch ROM**: `Return to game` / `Choose ROM…`.
- **Exit**: conferma `Yes, exit` / `No, go back`.
- Messaggi/annullamenti coerenti; nessuna chiusura imprevista del gioco.

---

## 5. Scheda Debug: inventario da mantenere

Le voci sono raggruppate secondo l'interfaccia approvata; **gli handler restano quelli della repo**. Dove il comando è accessibile solo da CLI, contrassegnarlo «CLI» fino a un'integrazione effettiva nella UI. Ogni azione che richiede una copia isolata deve continuare a bloccare l'operazione in assenza di sessione.

### 5.1 Sessioni e casi

- `Extended saves`
- `Game saves`
- `Start debug session`
- `Close debug session`
- `Debug catalog`
- `Start input recording`
- `Stop input recording`
- `Copy diagnosis` (può anche essere visibile in Report)
- Debug catalog: **Region**, **Recorded status**, **Kind**, **DS case**, **Entry**, **Import to free snapshot slot**, **Open isolated case**, **Choose / reload catalog**, con anteprima e messaggi di errore.
- CLI collegate a sessioni, catalogo e registrazioni, se già supportate e validate.

### 5.2 Replay e confronti

- `Replay case`
- `Choose replay profile`
- `Cancel replay`
- `Audit replay`
- `Capture on trigger`
- `Compare first difference`
- Eventuali campagne e audit CLI **esistenti**, chiaramente etichettati quando non integrati nel menu.

### 5.3 Esecuzione e memoria

- `Step 1 frame (F7)`
- `Step 10 frames (F8)`
- `Resume (F6)`
- `Inspect PC / registers`
- Operazioni disponibili in `debug-control` (pause/status/inspect/break/watch) con distinzioni di supporto per esecuzione compiled/interpreter; non inventare capacità mancanti.
- I futuri trigger nominati e valori fissati a schermo sono *V2*, non funzionalità già presenti.

### 5.4 Tracce e ispettori

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

### 5.5 Console e script

- Mostrare gli strumenti locali/CLI già presenti e le loro capacità reali.
- Prevedere gli ingressi per **Console eventi**, **Console comandi**, **Script e ricette**. In baseline sono **estensioni V2 pianificate**: le finestre demo non costituiscono un'implementazione funzionante.
- Quando le console diventeranno operative, devono essere **apribili dalla barra superiore come overlay ancorato** e non modificare la geometria dei pannelli sottostanti. Console eventi sola lettura; console comandi con validazione e dispatcher condiviso con il menu, non shell arbitraria.

### 5.6 Cheat

- Catalogo locale CH e import **solo quando saranno completate le dipendenze DBG2**.
- Non includere o pubblicare il catalogo privato gitignored né i relativi codici nel repository.
- Non affermare che i **341 blocchi documentati** siano verificati, utilizzabili o decodificati; nella baseline risultano `not-runtime-tested`.
- Applicazione limitata a copia di debug isolata, con conferma, compatibilità e provenienza; `Disattiva` non equivale a rollback. Per annullare gli effetti richiedere checkpoint esplicito.

### 5.7 Report e archivio

- `Copy diagnosis`
- `Create issue report`
- Report UI/audio/codice/gameplay/recenti in continuità con gli handler già esistenti.
- Strumenti e script CLI disponibili per homepage e pulizia/preview degli artefatti, con protezione delle prove e dei salvataggi.

### 5.8 Debug Tools V2 — esattamente 28 attività pianificate

Mostrare un filtro o una vista separata «**Debug V2 — pianificato**», con i codici stabili e i nomi della checklist originale. La UI non deve far passare queste voci per completate.

| Gruppo nella nuova sidebar | Attività |
| --- | --- |
| Sessioni e casi | `DBG2-01` Inventario azioni; `02` Catalogo comune; `03` Selettore interno; `04` Migrazione delle scelte; `05` Parametri e preset; `06` Libreria script e strumenti; `07` Persistenza portable |
| Tracce e ispettori | `DBG2-08` Schema eventi; `09` Selezioni/azioni host; `10` Eventi runtime; `11` Console eventi live; `12` Limiti e prestazioni |
| Console e script | `DBG2-13` Linguaggio comandi; `14` Editor/completamento; `15` Risposte strutturate/job; `16` Ricette e batch |
| Cheat | `DBG2-17` Import catalogo; `18` Decoder; `19` Applicazione controllata; `20` Pagina cheat; `21` Primo sottoinsieme EUR; `22` Cheat avanzati; `23` Provenienza/checkpoint/replay |
| Esecuzione e memoria | `DBG2-24` Trigger e arresti nominati; `25` Valori e differenze fissati a schermo |
| Report e archivio | `DBG2-26` Timeline; `27` Verifiche di regressione; `28` Guida/distribuzione |

**Sono attività trasversali:** questa collocazione serve alla navigazione, non sostituisce la sequenza di dipendenze della checklist `DEBUG_TOOLS_V2_TODO.md`.

---

## 6. Pattern di interazione da implementare

### 6.1 Pannello a due aree per gli strumenti Debug

La lista e i dettagli devono essere due aree **con geometria indipendente**:

```text
┌───────────────────────────────┬────────────────────────────────────────┐
│ [Menu + CLI] [V2 pianificati] │ Titolo strumento + disponibilità      │
├───────────────────────────────┤────────────────────────────────────────┤
│ [01 Strumento A]              │ Parametri / filtri / info            │
│ [02 Strumento B]              │ Scroll interno SOLO se necessario    │
│ [03 Strumento C]              │                                        │
│     SCROLL INTERNO ↓          │                                        │
├───────────────────────────────┤────────────────────────────────────────┤
│                               │ [stato/prerequisiti] [Avvia / Conferma]│
└───────────────────────────────┴────────────────────────────────────────┘
```

Il titolo e il pulsante di avvio rimangono fermi anche con descrizioni lunghe. La selezione della voce non porta il pannello dettagli a un'altra coordinata. Quando la larghezza non è sufficiente, **ridisporre soltanto queste aree interne** in una divisione verticale, assegnando un'altezza fissa relativa a ciascuna.

### 6.2 Focus, mouse e controller

- Navigazione per focus/tab e D-pad, tasti di conferma/annulla, mouse hover/click, wheel e scrollbar quando presenti.
- Focus e selezione sono stati distinti dal semplice mouse hover, salvo le regole già implementate. Non introdurre doppia esecuzione per input tenuto.
- Non cambiare scroll offset di una lista quando si aggiorna un valore o si seleziona un elemento. Resettarlo solo quando si entra volutamente in una sezione diversa o cambia la fonte dei dati.
- Modali di conferma sovrapposte all'area corrente senza modificarne la geometria; `Cancel` torna al contesto precedente, focus ripristinato.
- Prima di eseguire un comando distruttivo mostrare target, slot, profilo o sessione e il suo effetto reale. Non accoppiare selezione ed esecuzione.

### 6.3 Stato e persistenza

Separare chiaramente:

- Preferenze persistenti (`AvConfig`, input cfg, opzioni video, velocità preferita quando supportata).
- Stato della sessione corrente (Fast Forward ON/OFF, debug copy attiva, registrazioni, job).
- Stato della UI (scheda corrente, voce selezionata, scroll offset, filtri, dropdown aperti).
- Metadati letti dal disco (save timestamp, file disponibili, backup, cataloghi).

Non sovrascrivere file di configurazione per operazioni puramente visive. Non far dipendere il gameplay dai dati temporanei della UI.

---

## 7. Architettura tecnica consigliata

**Riutilizza** il renderer presente in `crates/menu/src/theme.rs` e gli handler di `crates/menu/src/lib.rs`. Possibile refactor incrementale:

1. **Catalogo di navigazione:** per ogni voce mostrare `id`, `label`, `group`, stato supporto, azione o pagina di destinazione, eventuali prerequisiti. Fare riferimento alle vere `Row`/`MenuAction`, senza duplicare la logica del debugger.
2. **Layout nativo:** funzione deterministica che calcola rettangoli di `sidebar`, `header`, `body`, `footer`, `tool_list`, `detail`, `modal` a partire da larghezza/altezza effettive; unico risultato per rendering e hit-testing.
3. **Rendering:** palette e componenti riutilizzabili (`Panel`, `NavRow`, `SettingRow`, `SlotButton`, `KeyCap`, `ConsolePanel`, `ConfirmDialog`). Nessun asset esterno non necessario.
4. **Input:** routing globale delle combinazioni L/R **solo quando il menu è aperto**; priorità alla combo e cattura/consumo dei singoli eventi. Gestire conflitti con rimappatura dei tasti e i controlli di gameplay.
5. **Sottomenu:** le funzioni reali restano collegate agli handler; conservare le transizioni `Changed` perché il runtime applichi immediatamente gli aggiornamenti che già supporta.
6. **Menu Debug:** riutilizzare selettori reali/catalogo DS esistenti; funzioni solo CLI con integrazione reale solo se è progettata/validata. Evitare falsi pulsanti «Esegui» che non fanno nulla.
7. **Prestazioni:** evitare rebuild, allocazioni e invalidazioni di framebuffer quando non cambiano pagina, stato, animazione o selezione. Testare entrambe le vie di presentazione GPU/CPU dove supportate.

Se il refactor richiede nuovi moduli, mantenerli dentro `crates/menu/src/` con responsabilità precise (es. `layout`, `navigation`, `widgets`), evitando di trasformare tutto in un unico file enorme.

---

## 8. Piano di lavoro raccomandato

### Fase A — Sicurezza / inventario

- Leggere e mappare tutte le righe di `MAIN_ROWS`, `rows_of(Page::...)`, `MenuAction`, tutte le opzioni configurabili e i shortcut.
- Preparare un **inventario prima/dopo** e una piccola matrice «voce originale → sezione nuova → handler» da conservare nei documenti di sviluppo.
- Eseguire i test attuali e registrare l'esito. Non rimuovere gli esistenti.

### Fase B — Struttura e stile nativo

- Implementare navbar sinistra e selettore `Gioco/Debug`.
- Aggiungere layout fluido con limiti minimi e scroll confinati.
- Implementare pannelli e stile «Atlante del Clan», incluso `Torna al gioco` coerente.
- Collegare tutte le azioni esistenti preservando conferme e state handling.

### Fase C — Funzioni approvate oltre la baseline

- Save state rapido in Panoramica con ultimo slot ricordato, sovrascrittura/caricamento e metadata veri.
- Data dell'ultimo salvataggio standard senza fonti fittizie.
- Footer scorciatoia di apertura/chiusura basata sui binding effettivi.
- Volume L1/R1 e mute chord; Fast Forward L2/R2 e toggle chord con consumo corretto.
- Estensione della scala Fast Forward 100–800%, passi 20%, **solo dopo** aver integrato il motore e i test di timing/audio.

### Fase D — Debug e validazione completa

- Migrare tutte le voci Debug attuali nella sidebar organizzata in 8 gruppi, mantenendo la distinzione `Menu / CLI / V2`.
- Collegare il catalogo DS, il debug session guard e gli altri handler reali.
- Mettere le V2 come *pianificate* oppure implementarle **separatamente** seguendo i task DBG2 e i loro criteri di chiusura, senza proclamare funzionalità future come già funzionanti.
- Eseguire test e smoke visuali del renderer con diversi aspect ratio e lunghezze di testo.

---

## 9. Test di accettazione obbligatori

### Layout e navigazione

- [x] A 1280×720, 1024×768, 854×480 e un viewport stretto supportato: **nessuno scroll orizzontale della finestra principale**.
- [x] Side menu, switch Gioco/Debug, barra superiore, titolo, footer shortcut, pulsanti L/R e status finale rimangono alle stesse coordinate durante scroll.
- [x] Le liste Debug da 3, 9 e 20+ voci scorrono **solo all'interno** del loro riquadro; il pannello dettagli non salta.
- [x] Cambiando strumento o modificando un parametro, scroll dell'elenco e selezione rimangono stabili.
- [x] A larghezza stretta, solo il contenuto interno si dispone verticalmente; sidebar, tabs e footer rimangono in posizione.
- [x] `Torna al gioco` condivide cornice/padding/focus/hover dei pannelli e non esce dalla griglia.
- [x] Nessun testo importante viene troncato senza possibilità di aprire il dettaglio o leggere il valore intero.

### Gioco e input

- [x] Tutte le 12 opzioni originali del menu sono raggiungibili nella nuova navigazione.
- [x] Save/load/replace/undo, 9/99 slot, game saves, copia e restore backup funzionano come prima, con conferme.
- [x] Ultimo slot rapido correttamente preselezionato alla riapertura; selezione non scrive sul disco.
- [x] Timestamp standard reale o assenza esplicita; data del `.sav` non confusa con quella degli save states.
- [x] L1/R1 regolano il volume da **ogni pagina** del menu; combo simultanea muta senza variare il valore.
- [x] L2/R2 regolano la velocità da **ogni pagina**; combo simultanea toggla senza variarla.
- [x] A menu chiuso non ci sono regressioni sui comandi L/R, zoom, fast toggle e input GBA.
- [x] Se l'FF 20% è implementato: prove su 100, 120, 140, 200, 400, 800%, on/off, audio modes, serializzazione, clock e runtime. Altrimenti evidenziare espressamente il mancato completamento.
- [x] Rimapparsi la combinazione di menu aggiorna subito i keycap in basso a sinistra; `Escape` di recupero e combo press/release funzionano.
- [x] Navigazione e conferme mouse/tastiera/gamepad testate, inclusa cancellazione dropdown e modali.

### Debug e sicurezza

- [x] Confronto **uno a uno** tra tutte le righe di `Page::Debug` e l'inventario delle nuove sezioni: nessuna azione orfana.
- [x] Catalogo DS conserva filtri, anteprima, importazione e segnalazione degli errori.
- [x] Le funzioni che richiedono copia isolata non si avviano senza sessione.
- [x] Step frame, resume, trace, report, replay e diagnosi richiamano gli handler esistenti.
- [x] Le CLI non diventano automaticamente «azioni Menu» se manca un'integrazione verificata.
- [x] Le 28 DBG2 seguono lo stato reale della checklist: 01…09 completate, 10…28 pianificate; nessun cheat non verificato compare come attivo.
- [x] Nessun file `.sav`, stato, log/prova o artefatto viene sovrascritto senza le condizioni e conferme previste.

### Qualità del progetto

- [x] Compilazione e test dei crate interessati secondo i comandi documentati nella repository.
- [x] Test unitari per geometria/hit-testing, clamp/scroll, switch scheda, combo controller, persistenza e conferme.
- [x] Smoke test grafici per canvas piccolo/grande, renderer CPU/GPU dove disponibili.
- [x] Documentazione aggiornata quando cambiano shortcut e opzioni.

---

## 10. Output richiesto all'agente che implementa

Consegnare:

1. **Codice reale nel repository** (non un nuovo mockup HTML) e lista dei file cambiati.
2. **Inventario «prima → dopo»** completo delle voci menu/debug con relativo handler.
3. Breve documentazione della struttura/sidebar e della logica del layout adattativo.
4. Risultati dei test e istruzioni per avviare il menu modificato.
5. Elenco preciso delle funzionalità **completate**, **rimaste operative come prima**, **ancora da realizzare** e degli eventuali limiti incontrati.
6. Se il Fast Forward 20% non è stato implementato nel motore, dirlo esplicitamente e non far apparire la UI come se applicasse velocità inesistenti.

### Definizione di completamento

Il compito è completato **solo** quando il menu nativo presenta lo stile «Atlante del Clan», la barra laterale approvata, il layout orizzontale adattativo con geometrie stabili, il pulsante `Torna al gioco` integrato, i controlli rapidi desiderati e **tutte le funzioni preesistenti ancora raggiungibili e operative**, con test superati. Ogni funzionalità Debug V2 non implementata deve restare dichiarata come futura, non simulata come reale.

---

*Specifica redatta a partire dalle richieste di progettazione del menu e dall'inventario della branch `decomp` consultato il 29/10/2026. Prima di procedere, verificare eventuali commit successivi alla baseline e adattare la mappa senza perdere le funzioni introdotte nel frattempo.*
