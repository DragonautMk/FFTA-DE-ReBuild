<a id="todo-ottimizzazioni-desktop-fftarecomp"></a>

# FFTARecomp desktop optimization TODO

Plan from October 20, 2026, based on code at revision `e39cdb6`.
Priorities: stability, game correctness, Linux/Windows compatibility and code
simplicity. The reported problem concerns Ubuntu 26 and possible slowdowns during
enemy turns. Checkboxes indicate completion of every criterion for each task;
partial progress is recorded below.

The recommended sequence is to **measure the actual slowdown, eliminate
redundant work, address the dominant subsystem, then assess further GPU work**.
Increased GPU utilization alone is not a useful result.

<a id="stato-dellesecuzione"></a>

## Execution status

Status reconciled with current criteria and evidence on October 21. Read
[TASK_HANDOFF.md](TASK_HANDOFF.md) before selecting work: an open parent does
not reopen completed substeps or rejected experiments. The early test sections
below are historical; their counts/exports apply to that phase.

- **PERF-01 — partial:** enemy replay, worldmap and zoom6 already available and
  measured; the rare original slowdown is not reproduced/isolated. Do not
  recreate fixtures and baselines to reconstruct context.
- **PERF-02 — partial:** CPU/PPU/GPU profiles, audio callback, frame gate, raw
  enhanced requests, PipeWire observation and presentation cadence are
  implemented. Attribution of the actually slow scene, controlled realtime
  overhead and hardware/native Windows coverage remain. Empty requests do not
  equal audible defects; ERR PipeWire does not cover every hardware XRUN.
- **PERF-03 — completed:** A/B procedure with hash/configuration, verification
  separate from timing, state/frame/audio comparison and rollback; incomplete
  guard stopped before temporal history and frontend counters. Reuse
  [dispatch checks](../changes/2026-10-20-preserve-native-dispatch-in-cost-profiling.json)
  and complete candidate checks without reimplementing them.
- **PERF-04 — partial:** recovery, skipped uploads, controlled device
  invalidation and X11/Wine lifecycle verified; Wayland/native Windows and the
  physical display/DPI/GPU matrix specified in the entry remain.
- **PERF-05/06 — completed:** LUT/FPS revisions, UI input-packing cache,
  call-local shared world upload and FIFO reuse. Rejected prototypes stay rejected.
- **PERF-07 — partial:** CPU inventory and sampling completed; no new
  translation accepted. Guest counts/cycles do not isolate host cost.
- **PERF-08 — completed:** scanline/UI reductions retained with CPU reference,
  full state and headless/GUI benefits; worldmap/zoom6/lifecycle verified.
  [Latest retained reduction](../changes/2026-10-21-evaluate-world-only-ui-pixels.json).
  Remaining CPU share does not automatically reopen this item.
- **PERF-09 — partial:** diagnostics/gates/sources/backend already available;
  current samples show no contention that justifies SPSC or more buffering.
- **PERF-10 — partial:** observer and cadence matrix completed; composition
  before waiting rejected (worse p99 in 4/5 pairs). Criteria for physical
  60/high-Hz monitors, audio drift and input responsiveness remain.
- **PERF-11/12/13 — conditional, not implemented/accepted:** available costs do
  not yet justify new GPU filters or a GPU PPU. The optional zoom pool already
  exists; do not replace it with another or create threads per frame.

Current exports and hashes are in the handoff: 344 Windows/Wine tests and 345
Linux tests, 15 optional tests excluded per target. These are compatibility/
export results, not native Windows performance or proof that the original lag
is resolved. The [measurement workflow](PERFORMANCE_WORKFLOW.md) and
`docs/changes/` records retain details and limits. This audit repeats no builds
or benchmarks.


## Fixture locali e prima verifica della battaglia

Il nuovo archivio locale `saves.zip` e la sua eventuale estrazione `saves/`
sono ignorati da Git; `.sav` e `.ffstate` sono già esclusi anche altrove.
Le prove usano copie in `target/`: nessun progresso personale entra nei
record di progetto, nel contesto Docker o nei pacchetti.

Il caricamento effettivo dello slot fornito è riuscito con il runtime Linux
esportato dal container. Il salvataggio cartuccia permette inoltre di aprire
la world map dalla voce Saved Game > Load. Entrambe le scene sono state
riconosciute visivamente, senza dedurle dal solo nome del file.

Una sessione esplorativa della battaglia, ripresa esplicitamente dal menu
host e lasciata avanzare per 20 secondi, ha prodotto 1.074 campioni core,
171.840 scanline e nessun intervallo PPU invalido. Il profilo include anche
l'avvio: mediana core 23,084 ms e scanline 17,545 ms; p95 rispettivamente
25,520 e 19,434 ms. Lo zoom non ha eseguito righe aggiuntive.
Questa è una **misura preliminare Xvfb/llvmpipe**, non una baseline Radeon né
un replay identificato del turno nemico. I tentativi rimasti nel menu host
sono esclusi da questa valutazione.

La PPU CPU è quindi una priorità da approfondire in PERF-02/08: separare
rendering BG/OBJ, composizione ed estrazione dei pannelli; verificare in
particolare i passaggi ripetuti per riga in `capture_dialogue_part`.
Quella funzione ordina i candidati BG/OBJ per ogni pixel, pur avendo priorità
BG costanti per riga: misurare separatamente questo lavoro e confrontare una
selezione dei primi due livelli con il riferimento prima di cambiarlo.
Non è ancora dimostrato quale funzione domini né un beneficio dello
spostamento sulla GPU. Il runner ora supporta `--state` anche nel confronto
Python: 600 frame aggiuntivi, 120 di warm-up e cinque coppie native sulla stessa
build hanno prodotto framebuffer/OAM, stato finale e conteggi identici.
Le mediane di lavoro sono 20,950/20,856 ms; le due catture diagnostiche attribuiscono
circa il 76% del core alle scanline. Il caricamento e l'avvio sono esclusi dal
campione. Un controllo separato verifica anche lo zoom relativo allo snapshot.
Questa è una baseline della battaglia inattiva, senza ancora un turno nemico
identificato: non è un guadagno di prestazioni e PERF-01/02/03 restano aperti.

## Correzione delle condizioni di misura e selettore UI valutato

Una ricerca automatica npm di VS Code nella workspace seguiva anche gli alberi
ignorati ed era attiva da oltre mezz'ora, occupando circa dieci core. È stata
interrotta dopo averne identificato l'origine automatica; `npm.autoDetect` è
stato disattivato soltanto nelle impostazioni locali ignorate della workspace.
Nessuna impostazione globale o preferenza preesistente è stata cambiata.
Il portable con renderer originale ha poi superato caricamento, Annulla,
ricaricamento e chiusura in Xvfb/llvmpipe: nella battaglia inattiva il contatore
osservato è circa 59–60 FPS INT/EXT. Non è una misura Radeon né del turno AI.
Il [workflow](PERFORMANCE_WORKFLOW.md#host-load-before-timing) descrive il controllo.

Con **lo stesso intervallo da 1.200 frame**, snapshot, binario baseline,
traduzione, input, audio e affinità CPU, la mediana baseline è 18,655 ms con
quel carico e 10,777 ms dopo la sua rimozione. Le precedenti prove da 600 frame
intorno ai 21 ms restano dati storici sotto contesa, non una baseline senza
interferenze; non confrontare direttamente lunghezze diverse. Questo è un
cambiamento dell'ambiente di sviluppo, non un'accelerazione del codice di gioco.

Sono stati valutati due selettori dei soli due livelli UI frontali al posto
dei due ordinamenti per pixel. Entrambi conservano risultati esatti su 196.608
combinazioni di priorità/presenza e sui replay. Sul sistema senza la scansione:
la variante a chiave compatta peggiora la mediana dell'1,64%; quella semplice
migliora la mediana dell'1,56%, ma la cattura diagnostica non mostra un vantaggio
sui picchi (p95/p99 maggiori). Il beneficio complessivo non giustifica sostituire
l'ordinamento standard: **nessuna variante è stata mantenuta nel runtime**.
I singoli profili dei picchi non provano una regressione causale universale;
sono comunque insufficienti per accettare un'ottimizzazione del pacing.

La world map è stata raggiunta di nuovo partendo da una copia del `.sav`,
registrando gli input e creando uno snapshot locale classificato visivamente.
Il replay da quello snapshot passa cinque coppie da 1.200 frame sul renderer
originale: mediana circa 2,1 ms/frame, stato finale e immagini identici,
nessun frame oltre budget nei due profili. È una calibrazione sullo stesso
binario, non un miglioramento. I file restano sotto `target/`, fuori da Git
e dai pacchetti. Prima di altri
interventi PPU/GPU va identificato il turno nemico lento sul sistema senza
questa interferenza; PERF-01/02/03/08 restano aperti.

## Replay con attivazione nemica identificata

È stata preparata una nuova sequenza locale dalla copia dello snapshot fornito:
salvataggio sospeso tramite il comando del gioco, nuovo avvio con registrazione,
ripresa della battaglia e attesa delle unità del giocatore. La sequenza comprende
l'attivazione e il movimento di un mostro nemico, poi il ritorno al menu di
un personaggio del giocatore. Questi passaggi sono stati osservati nella finestra;
non si tratta più del solo replay della battaglia inattiva.

Uno snapshot intermedio e gli input successivi ribasati delimitano 3.530 frame.
Il replay headless raggiunge lo stesso frame finale e lo stesso hash del
framebuffer della sessione grafica; anche l'immagine finale è stata ispezionata.
Le due verifiche A/B coincidono su framebuffer e OAM di tutti i frame.
Il [workflow](PERFORMANCE_WORKFLOW.md#record-a-battle-from-a-supplied-snapshot)
descrive la procedura senza rimuovere la protezione che vieta di caricare uno
snapshot durante la registrazione. Tutte le fixture rimangono ignorate.

Il confronto completo passa cinque coppie alternate sullo stesso codice, con
120 frame di warm-up: mediane 8,346/8,290 ms per i 3.410 frame misurati,
stato macchina finale e conteggi di esecuzione identici. I due profili separati
hanno zero frame oltre 16,74 ms, p99 10,055/10,129 ms e massimi
12,535/16,518 ms. Le scanline occupano circa il 73% del core (6,07/6,08 ms
medi); il resto circa 2,25/2,24 ms, senza righe zoom aggiuntive. È una nuova
baseline, non uno speedup: la differenza fra binari identici calibra il rumore.
La prima esecuzione interrotta con errore di output resta conservata come
parziale; questi risultati provengono dal successivo confronto completato.

Questo intervallo include input del giocatore, animazioni e attese: non misura
solo il calcolo AI e non dimostra che sia il turno lento originariamente
segnalato. PERF-01 resta aperto per gli altri scenari e la caratterizzazione del
rallentamento; nessuna accelerazione del runtime è ancora attribuita al replay.

## Attribuzione delle fasi PPU completata

`FFTA_PPU_STAGES=1` aggiunge intervalli disgiunti per BG, OBJ, composizione,
estrazione UI, setup UI e residuo. Il confronto del nuovo codice diagnostico
con la baseline passa cinque coppie sul replay esistente, immagini/OAM per
frame, stato macchina e conteggi. Mediane con raccolta spenta 8,289/8,317 ms
(differenza +0,34% in questa prova). La cattura dettagliata usa 8,373 ms/frame,
circa +0,67% rispetto alla mediana dello stesso candidato: è una stima singola
soggetta al rumore, non una garanzia universale di overhead.

Medie per frame: BG 0,246 ms, OBJ 0,288 ms, composizione 0,531 ms,
UI capture 4,912 ms, setup UI 0,071 ms, residuo scanline 0,078 ms.
L'estrazione UI occupa circa l'80% delle scanline e il 59% del core.
Nessun intervallo invalido o frame oltre budget nei due profili; p99 core
baseline/candidato 10,237/10,116 ms. Questo chiude il sottopasso diagnostico
PERF-02-PPU-STAGES su Linux, non l'intero PERF-02 o una nuova ottimizzazione.
Il profilo ha guidato il precalcolo delle finestre per riga di
`capture_dialogue_part`, già concluso nella sezione seguente; i selettori frontali
scartati non sono stati ripetuti.

## Prima ottimizzazione dell'estrazione UI mantenuta

PERF-08-UI-ROW-INVARIANTS è concluso: registri finestra e appartenenza verticale,
origine nativa e flag modale sono calcolati una volta per chiamata. La precedenza
WIN0/WIN1/OBJ/esterno e gli intervalli circolari restano identici; 2.457.600 casi
confrontati con il riferimento precedente, 169 test core e 48 runtime passati.
Nessuna cache persistente, modifica del timing guest o versione stato nuova.

Cinque coppie sul replay esistente: mediana 8,412 → 8,229 ms/frame, circa
**2,18% di lavoro host in meno**, immagini/OAM per frame, stato finale e conteggi
identici. Il profilo separato attribuisce UI capture 4,941 → 4,779 ms medi.
p95/p99 core migliorano (10,117/10,540 → 9,890/10,346 ms); i rari frame oltre
budget sono 2/4 nelle singole catture, con massimi 18,489/18,634 ms. Non è una
prova di eliminazione dei picchi, né del lag originariamente segnalato.

Il nuovo codice passa anche cinque coppie sulla world map: mediane
1,958/1,967 ms (+0,43%), stato/immagini/conteggi identici e zero frame oltre
budget nei profili. Una sessione Xvfb/llvmpipe con il nuovo runtime passa
load, Annulla, reload, ripresa e chiusura; anteprime e messaggi ispezionati,
profilazione dettagliata senza intervalli invalidi, slot e ZIP originali invariati.
Windows nativo, Radeon e controller fisico restano fuori da questa validazione.
Non ripetere questi task senza una nuova modifica o un criterio scoperto.

## Riuso dei BG nativi UI mantenuto

PERF-08-UI-PREPARED-BG-ROWS è concluso: BG2/BG3 nativi sono preparati una volta
per scanline e condivisi fra pannelli/passaggi. Il backing filtra una copia
privata del testo; Stats conserva la sua maschera nulla. Il riferimento con
preparazione fresca per passaggio coincide nello stato macchina finale su 48
configurazioni e tutte le 160 righe, con piani non vuoti, filtri e BG abilitati
in combinazioni diverse. Passati 170 test core e 48 runtime.

Cinque coppie sul replay esistente contro la versione con finestre precalcolate:
**8,244 → 7,903 ms/frame, 4,14% di lavoro host in meno**. Immagini/OAM per
frame, stato finale e conteggi identici. UI capture media 4,781 → 4,476 ms;
p95/p99 core 9,895/10,328 → 9,487/9,909 ms. Nessun frame oltre budget nei due
profili, ma il massimo isolato cresce da 10,862 a 12,277 ms: nessuna promessa
universale di eliminazione dei picchi.

Cinque coppie world map: mediane 1,962/1,950 ms, differenza piccola, immagini,
stato e conteggi identici, zero frame oltre budget. Nuovo runtime verificato
anche su load/Annulla/reload/ripresa/chiusura in Xvfb/llvmpipe, con ispezione
delle anteprime e del gioco; 989 frame e zero intervalli PPU invalidi. Slot e
ZIP originali invariati. Nessun nuovo dato persistente, ABI o formato stato;
nessuna misura Windows nativa o Radeon è dedotta da questi risultati.

## Righe UI vuote: ordinamento e blending evitati

PERF-08-UI-EMPTY-ROWS è concluso. Dopo preparazione, cattura sprite e reset dei
flag, una riga con tutte le sorgenti UI vuote viene azzerata esplicitamente e
non ordina/fonde i candidati del mondo: questi non possono diventare pixel UI.
Non vengono saltati frame, scanline hardware, cicli o decisioni guest. Il
vecchio ciclo completo resta il riferimento di test; le righe non vuote lo
mantengono invariato. Passati 171 test core e 48 runtime, inclusi contenuti
precedenti da cancellare, maschere Stats, modali e blending di oggetti mondo.

Cinque coppie sul replay esistente contro la versione con BG condivisi:
**7,891 → 6,194 ms/frame, 21,50% di lavoro host in meno**. Immagini/OAM per
frame, stato finale e conteggi identici. UI capture media 4,467 → 2,770 ms;
p95/p99 core 9,481/9,906 → 7,497/7,793 ms, zero frame oltre budget nei due
profili. Il massimo isolato cresce 10,770 → 14,502 ms: non dedurre una garanzia
sui picchi rari o la risoluzione del lag originariamente segnalato.

Cinque coppie world map: 1,978/1,942 ms, uguaglianza immagini/stato/conteggi e
zero frame oltre budget. Il nuovo runtime passa load/Annulla/reload/ripresa/
chiusura in Xvfb/llvmpipe, con ispezione di anteprime e gioco; 987 frame e zero
intervalli PPU invalidi. Slot e archivio originale restano invariati. Il passo
di validazione delle build correnti Windows/Linux è stato completato nella
sezione seguente; gli archivi Docker aggiornati incorporano il renderer corrente.

## Build e archivi desktop aggiornati

PERF-03-CURRENT-DESKTOP-BUILDS è concluso per il suo ambito: Windows GNU/MinGW
in Docker con **317 test Wine**, Linux Bookworm con **318 test**; 9 casi opzionali
ignorati per piattaforma. Integrità, percorsi e contenuti degli archivi verificati:
solo lo scaffold Data vuoto, nessuna ROM/save/registrazione/cache gioco o DLL di
test, TinyCC e licenze inclusi. PE con risorse e import invariati; ELF eseguibili
con requisito massimo GLIBC 2.35. Hash degli input registrati invariati.

Il nuovo runtime Windows esportato passa il replay esistente da 3.530 frame:
framebuffer/OAM per frame, stato finale e conteggi coincidono sia col precedente
runtime Windows sia con la build Linux già validata. È una prova di correttezza
sotto Wine, non un confronto di prestazioni Windows nativo. Il nuovo archivio
Linux passa avvio a cache vuota, load/Annulla/reload/ripresa/chiusura in
Xvfb/llvmpipe: anteprime e gioco ispezionati, 1.001 frame e zero intervalli PPU
invalidi; slot e ZIP originali invariati.

Gli archivi e BUILD-INFO correnti sono aggiornati in `dist/docker-windows/` e
`dist/docker-linux/`; i precedenti restano conservati nella directory locale di
validazione. Nessuna pubblicazione remota. Restano aperti Windows nativo,
controller fisici, GPU hardware/pacing e gli scenari non ancora caratterizzati,
fra cui zoom elevato: non ripetere la baseline ordinaria per coprirli.

## Osservazioni sulla revisione iniziale (storiche)

- La GPU è già usata per scalatura, griglia e composizione dei livelli UI:
  [present.rs](../../crates/screen/src/present.rs), `Presenter::present`, e
  [ui_layer.rs](../../crates/screen/src/ui_layer.rs), `Gpu::draw`.
  Rendering GBA, preparazione dei livelli e filtri temporali restano sulla CPU.
- `Gpu::update` ricarica texture UI, mondo e LUT a ogni aggiornamento del
  livello. La LUT contiene 32.768 colori; anche il contatore
  [FPS](../../crates/recomp/src/fps.rs), ridisegnato circa una volta al secondo,
  percorre questo caricamento a ogni presentazione.
  I livelli possono avere mondi/LUT differenti: non vanno unificati alla cieca.
- Nella baseline di [play.rs](../../crates/recomp/src/play.rs), `emu_ema_ms` comprende esecuzione,
  aggiornamento UI e `Temporal::push`, ma esclude `Temporal::compose`,
  presentazione, attese e parte del lavoro audio/persistenza. Il valore
  `native %` confronta blocchi nativi con singole istruzioni interpretate:
  non misura la percentuale di tempo CPU. La prima tranche sostituisce questa
  percentuale con conteggi separati e aggiunge un profilo CPU per fase.
- Un errore restituito da `Presenter::present` fa abbandonare il presenter GPU
  per il resto della sessione. Anche il timeout nell'acquisizione del frame
  passa oggi da questa strada. È una possibile causa di peggioramento
  improvviso, da verificare nel log; non è ancora la diagnosi del lag segnalato.
- [Temporal](../../crates/screen/src/blend.rs) elabora la storia dei frame
  emulati, con sei frame per la modalità Smart. La fusione usa colori già
  codificati per il display: spostarla in spazio lineare cambierebbe l'immagine.
- La [PPU](../../crates/gba-core/src/ppu.rs) dispone già di un percorso parallelo
  per le sole righe aggiuntive dello zoom, con test di equivalenza. È
  disabilitato per default su desktop; il pool persistente richiede la feature
  `parallel-ppu`, altrimenti vengono creati thread scoped durante il rendering.
- Il [benchmark precedente](../windows_runtime_performance.md) misura una
  sequenza introduttiva headless sotto Wine. Non dimostra il costo di un turno
  nemico, la fluidità della finestra o il comportamento di Windows nativo.

<a id="p0--rendere-il-problema-riproducibile"></a>

## P0 — Make the problem reproducible

- [ ] **PERF-01 — Record representative scenes and baselines.**
  Low risk; first step, no dependency. Prepare separate replays for a slow enemy
  turn, battle movement/effects, dialogue/menus and high zoom. Keep ROM, saves
  and recordings only in ignored local directories. Record build, backend,
  resolution, zoom, filters, VSync, audio mode, driver and cold/warm cache.
  Extend [benchmark_windows_runtime.py](../../scripts/benchmark_windows_runtime.py)
  for correct platform detection and reuse deterministic comparison.
  **Complete when:** the same turn repeats from controlled input/initial state
  and there are at least five alternating runs for comparison. Distinguish host
  slowdown from a wait already present in the game when emulation maintains
  approximately 59.73 frames per second.

- [ ] **PERF-02 — Measure times, spikes and waits separately.**
  Low risk; depends on PERF-01. Add opt-in diagnostics in `play.rs`, `main.rs`,
  PPU and presenter: guest execution, PPU, UI preparation, filters, audio handoff,
  save, upload, surface acquisition, command submission and clock wait. Avoid
  per-instruction timers and synchronous per-frame logs. Do not add nested
  intervals as if independent. Count native blocks, fallback, intentionally
  interpreted hooks and incomplete frames separately.
  **Progress:** the `RECOMP_COST_FROM` inventory now uses the runtime's same
  native dispatch, preserving audio/sprite/mouse hooks. It distinguishes native
  blocks, missing translations, system steps and interpreted hooks; it attributes
  guest cycles, not host CPU time, and remains separate from timing comparisons.
  Per-frame CPU times for GBA scanlines, zoom rows and the rest of the core
  (`--ppu-profile` / `FFTA_PROFILE=1`) are also available, with count checks,
  bounded samples and no guest clock/state change.
  **New completed substep:** opt-in callback profiling separates mutex wait,
  rendering, total and arrival interval, with faithful hold/drop counters for
  play, priming, pause and fast-forward. Bounded storage and report after
  teardown; DSP/sample/queue parity verified for different formats and channels.
  The Linux CPAL 44.1 kHz check measures total p99 0.500 ms and mutex wait
  0.000912 ms during play, with no callback exceeding its period. The 1790
  faithful holds in 6 callbacks include transitions/resumption: they are not
  evidence of audible underrun or isolated analysis of the AI turn. Median
  synthetic overhead is about 129 ns/callback (+0.34%). Later substeps
  (PERF-09/10) added frame gate, enhanced counters, backend observation and cadence.
  Original slow scene, controlled realtime overhead, audible meaning of source
  data and full hardware/native Windows coverage remain. Exports pass 328
  Windows/Wine and 329 Linux tests; profile on/off, loads/Undo/FF and benchmark
  audio bypass pass on portable Linux.
  **Complete when:** report contains median, p95, p99, maximum, frames over
  16.74/33.49 ms, turn duration and audio underruns; diagnostic overhead measured.
  GPU time uses optional timestamps and deferred read where supported: `submit`
  time is not GPU execution time. [wgpu 26 Queue APIs](https://docs.rs/wgpu/26.0.1/wgpu/struct.Queue.html)
  describe submission, transfers and timestamps.

- [x] **PERF-03 — Establish correctness checks and A/B procedure.**
  **PERF-03-FRACTIONAL-UI-TEXELS — completed.** Integer RG32Uint maps computed
  using existing CPU formulas eliminate fractional-crop differences without
  changing zoom/FOV. Full keys and device limits; conservative transparency
  bounding box and scissor avoid unnecessary work. Radeon matrix of 32
  zoom/resize/DPI states and shared-resource/invalidation tests pass with exact
  CPU parity. 3530-frame replay and worldmap preserve images/state/counters;
  GUI load/Undo/reload/close and Wayland pass. Five pairs improve 8.230 ->
  7.775 ms (-5.53%); the previous prototype without scissor (+4.17%) remains
  rejected. This result does not prove AI acceleration or native Windows
  performance. Docker export: 332 Windows/Wine tests, 333 Linux (14 optional
  excluded each); integrity, privacy, PE imports and GLIBC 2.35 verified.
  Portable Linux with cold cache: load/Undo/reload/resume/closure and screens
  checked.
  Low risk; depends on PERF-01. Reuse replay, interpreter/native comparison,
  framebuffer/OAM and existing savestate tests. For core changes also compare
  registers, RAM, IO, VRAM, guest cycles, saves and deterministic audio samples
  before the device. Block counters can change with a different translation:
  do not confuse them with game state.
  **Complete when:** each comparison has configuration and binary hashes,
  verification separate from timing and a procedure to return to baseline.
  Check that the `max_steps` limit does not advance temporal history and
  completed-frame counters when `run_frame_native` does not complete a frame.

<a id="p1--correzioni-circoscritte-e-lavoro-ridondante"></a>

## P1 — Bounded fixes and redundant work

- [ ] **PERF-04 — Recover from temporary graphics errors.**
  Medium risk; depends on PERF-02/03. In `Presenter::present` and
  `PlayApp::present_rgba`, distinguish timeout, surface needing reconfiguration,
  device loss and permanent failure. An isolated timeout should be able to skip
  that presentation and retry without permanently disabling the GPU. Define
  bounded retries, counters and a working software fallback; avoid recreation
  loops or continuous logs.
  **Progress:** timeouts implemented with skipped frame without losing GPU, one
  reconfiguration attempt per frame on Lost/Outdated and fallback after three
  consecutive failed recoveries. Out-of-memory/unknown errors trigger immediate
  fallback; only successful presentations count in FPS EXT. Tests simulate
  prolonged timeouts, recovery, budget and permanent failures. The initial
  Xvfb/Radeon-GL Invalid surface case is now handled with balanced error scopes:
  failed configuration returns to software renderer without panic. The same
  check covers resize, VSync and recovery reconfiguration; VSync returns a Result
  to the caller. Verified actual fallback case, resize/VSync ON-OFF on valid
  Vulkan and replay/closure on Radeon Wayland.
  **PERF-04-SKIPPED-UPLOADS:** fixed pending uploads on skipped frames: submit
  transfers only, without draw/present/timestamp or GPU waits. The frontend
  avoids new preparation for zero-size windows and preserves the benchmark
  error when a presentation is skipped. Batch consumption tests and Presenter
  check on 1200 zero-size batches pass; 492 MB logical traffic, stable RSS
  between batches 600 and 1200. Normal GUI load/Undo/reload/close passes; final
  exports verified (333 Windows/Wine tests, 334 Linux, 14 optional excluded
  each), with cold-cache GUI check on portable Linux and screen inspection.
  This is not a measurement of AI-turn acceleration or compositor minimization.
  **PERF-04-MINIMIZE-RESTORE — completed for Linux X11 and Wine/Vulkan.**
  New ROM-free diagnostic: three minimizations/restores observed by the WM,
  three resizes and VSync ON/OFF, successful rendering after every restore and
  correct teardown. Independent Iconic/Normal state verification. Linux
  Openbox/llvmpipe: 27 Presented, zero Skipped; Windows GNU under Wine with
  Vulkan: 18 Presented, 9 Skipped, zero dimensions during minimization. The
  first Wine D3D12 attempt fails device creation: it remains a negative result,
  not native Windows coverage. Current portable Linux also passes three cycles
  in battle followed by Undo/reload/resume/closure. No runtime change or package
  regeneration required.
  **PERF-04-MONITOR-TRANSITIONS — completed for virtual X11/Wine outputs.**
  Diagnostic `--monitors` mode: two distinct monitor handles, A/B/A/B/A sequence
  observed and confirmed by X positions, actual resizes between 640x360 and
  640x480, VSync ON/OFF and correct teardown. Xorg/dummy with 1280x720/1280x960
  outputs: 20 Presented, zero Skipped on Linux and Windows GNU/Wine/Vulkan.
  Portable Linux unchanged: battle, four moves, Undo/reload/resume/closure
  verified with screens. No runtime change or new benchmark. The first scenario
  with unconfigured keyboard was not accepted.
  **PERF-04-DEVICE-LOSS — completed for controlled device destruction.**
  Private test with actual device destruction: existing frames/UI returned an
  error, but creating a new UI caused panic on invalid textures. Presenter now
  retains the first uncaptured/loss error from its Device and passes it to
  software fallback; no per-frame GPU allocations or waits. Tested plain/cached/
  new UI and resize, error-scope isolation, persistent error, refusal of new
  writes/VSync and software presentation. Native and Windows/Wine/Vulkan pass;
  normal GUI without unexpected fallback and Radeon/Wayland 3530-frame replay/
  state/counters unchanged with correct closure. Docker export: 333 Windows/Wine
  tests and 334 Linux (15 optional excluded), archive/privacy/DLL import/GLIBC
  2.35 checks and cold-cache portable Linux GUI pass; previous copy retained.
  Controlled destruction does not prove every asynchronous driver reset or
  physical fault. Wayland and native Windows minimize/restore, physical screens,
  mixed-DPI/GPU migration/hotplug and device loss during execution remain.
  **Complete when:** resize, minimize/restore, monitor change and simulated
  transient error recover without stalls. Also verify Wayland closure after
  recovery and fallback, preserving release order in `exiting`.

- [x] **PERF-05 — Update UI textures and LUTs only when they change.**
  Low/medium risk; depends on PERF-02/03. In `ui_layer.rs`, `fps.rs` and
  `refresh_ui_layers`, introduce explicit content revisions: LUT when colors
  change, FPS when text changes, UI when it actually changes. Separate layout
  changes from pixel changes. Share world upload only for layers with the same
  source, dimensions and generation; the statistics panel has its own background.
  **Outcome:** LUT/FPS revisions, UI packing input cache and world reuse within
  a single presentation are implemented and verified. Worlds with the same
  source and dimensions share texture/upload; distinct sources, including Stats,
  remain separate. All unique worlds update on every call: no persistent
  address-based cache. Payload 530 -> 316 MB in replay, five pairs 8.251 ->
  8.219 ms (-0.39%) with better p95/p99. Independent/shared GPU, mutations,
  source graphs, dimensions, recreation and GUI load/Undo/zoom pass; export 329
  Windows/Wine and 330 Linux tests, including GUI on cold-cache portable Linux.
  The later fractional-coordinate change is described in PERF-03.
  **Complete when:** byte/upload counters demonstrate reduction and images,
  HUD, colors, menu, zoom and load/Undo remain equivalent. Invalidate revisions
  on ROM change, state load and GPU recreation. Assess reusable staging only if
  justified by profile: uploads also have copy/allocation costs
  ([wgpu documentation](https://docs.rs/wgpu/26.0.1/wgpu/struct.Queue.html#method.write_texture)).

- [x] **PERF-06 — Reuse buffers and intermediate results measured as costly.**
  Low/medium risk; depends on PERF-02/03. Check `handoff_audio`: `mem::take`
  consumption of `fifo_tap` abandons the buffer after transfer. Assess draining
  that retains capacity. Also measure repeated panel packing/bounds and retain
  them by content revision. Avoid caches based only on addresses or allocation
  optimizations present solely in tests.
  **Experiment evaluated:** scanning only visible extremes per row reduces
  function cost on synthetic panels, but two 3,000-frame end-to-end comparisons
  showed a candidate regression. Not enabled in runtime: retained only in tests
  with equivalence and a five-alternating-pair release microbenchmark.
  **Final outcome:** FIFO reuse already verified; UI packing cache retained with
  complete keys and memory bounded to current buffers, invalidation on restore
  and tests against direct packing. Five battle pairs measure -1.41% in GUI loop,
  UI preparation 0.157 -> 0.122 ms. Replay, world map, GUI/Undo/style and
  Linux/Windows-Wine builds pass; no audio event flow change. Evidence: October 21
  UI input cache record.
  **Complete when:** allocations or time decrease in the measured path without
  continuous memory growth, lost audio events or stale panels.


<a id="p2--ottimizzare-il-collo-di-bottiglia-dimostrato"></a>

## P2 — Optimize the demonstrated bottleneck

- [ ] **PERF-07 — Reduce expensive fallback during enemy turns.**
  Medium/high risk; depends on PERF-01/02/03. Profile `run_frame_native`,
  `BlockTable`, interpreter and generated code in
  [recomp-core](../../crates/recomp-core/src). ROM/IWRAM lookup is already
  direct: do not replace it without evidence. Use the fallback inventory to find
  missing entries, distinguishing them from intentionally interpreted hooks.
  **PERF-07-FALLBACK-CANDIDATES — triage completed, no translation changed.**
  New inventory on the already recorded enemy replay: after 120 warm-up frames,
  22.63M native blocks, 79.14M missing-translation steps, 55.91M hooks and 6820
  system steps; 3804 distinct missing entries over the entire replay. Parity of
  3530 frames/state/counters verified. Guest cycles are not host cost.
  Optimized native CPU sampling with symbols, 3928 samples/zero lost, indicates
  capture_dialogue_part 31.52%, sprites 8.63%, composition 7.76%; interpreter
  step_inner 6.21% and Machine::step 4.56%. It does not isolate the cost of
  individual missing entries. Prioritize remaining UI extraction work (PERF-08),
  without repeating rejected selectors or rewriting lookup/ABI based only on counts.
  **PERF-07-POST-PPU-HOST-TRIAGE — completed.** New sample after retained PPU
  changes, without recreating fixtures/inventory/baseline: 3798 samples, zero
  lost, identical final state/counters. UI capture 31.20%, sprites 9.19%, compose
  8.85%, step_inner 7.64% self; cost percentages, not speedup or cost of
  individual missing entries. Isolated binary with symbols, normal runtime
  restored by hash. Rejected selectors and caches remain closed.
  **PERF-07-IMMUTABLE-ROM-SEEDS — rejected for production.** 18 already observed
  ROM entries, no new RAM seeds: 28859->29196 blocks. Baseline reconstructed
  byte-for-byte identical to previous translation. All 3530 frames/OAM,
  initial state and warm-up scene equal, but final full-state hash different.
  Test stopped before timing; runtime/cache/gamedb/archives unchanged.
  [Evidence](../changes/2026-10-21-evaluate-immutable-rom-entry-seeds.json).
  Reduced fallback is not evidence of speed or correctness. Before expanding
  coverage, fix and verify audio/timer order without weakening checks.
  **PERF-07-ROM-SEED-STATE-DIVERGENCE — diagnosis completed:** first difference
  at relative frame 38 (guest 8514), equal CPU/clock; two mixer samples and HLE
  verifier state differ. Diagnostic reproduces both original final hashes.
  ROM-free fixture: tick600 and 600*tick1 have equal final clock and DAC but
  different samples and timer bookkeeping. This sensitivity supports the event
  ordering hypothesis without yet proving the entire causal chain in battle.
  No core fix or new translation accepted; next criterion PERF-07-TIMER-AUDIO-ORDER,
  checking DMA, waits, cycles and audio before resuming performance measurements.
  [Diagnosis](../changes/2026-10-21-isolate-rom-coverage-audio-divergence.json).
  **PERF-07-TIMER-AUDIO-ORDER — completed and delivered.** Events at their own
  timestamp, audio/timer order equal to the single-cycle reference, DMA without
  video re-entry and subtracted cycles added to CPU work; halt/IntrWait uses the
  same path. Five new regressions, 179 native/Wine core and 66 runtime tests
  pass; native 3530-frame/OAM/counter replay identical to baseline. Correct state
  verified separately: only final audio/HLE/timer bookkeeping changes intentionally,
  not CPU/clock. Five pairs measure median +0.066607 ms (+1.157%): correctness
  fix, not speedup. Slot v1 and GUI load/zoom/Undo/reload/closure pass. Docker
  export 349 Windows/Wine and 350 Linux tests (15 optional excluded), integrity/
  privacy/compatibility verified; extracted Linux archive passes 3530 frames/
  corrected state and cold-cache GUI with slot v1/load/zoom/Undo/reload/closure.
  Dist archives updated only after checks, with previous versions retained. No
  commit or public release. New seeds still excluded: after the fix the first
  divergence is CPU/clock at frame 212; fixing audio alone does not justify
  expanding translation.
  [Fix and limits](../changes/2026-10-21-order-timer-audio-events.json).
  **PERF-07-NATIVE-FRAME-BOUNDARY — correctness-verified candidate.** Native
  code continued to the back-edge with IRQ already pending: reproduced without
  ROM too, with differing IRQ stack bytes. Budget guards at each entry and after
  store/shell eliminate the problem, retaining native batching away from events
  and separation from RAM-content checks. Translation cache 7, same savestate/
  ABI schema. Native 181 core/66 runtime tests and 11-case TinyCC regression
  pass; also Windows/Wine. Both ROM coverages now match in all state at every
  one of 3530 frames, including continuous queues with 7.7M samples per mixer/
  PSG/HLE and 931920 events per FIFO. Real CLI and previous images verified.
  Cost/GUI/export still to be accepted; timing triplets underway. No automatic
  seed expansion or new portable published.
  [Candidate and evidence](../changes/2026-10-21-preserve-native-event-boundaries.json).
  Extend translation only for code with guaranteed validity; mutable RAM and
  overlays require checks and invalidation.
  **Complete when:** AI replay takes less host time beyond measurement noise and
  preserves decisions, RNG, cycles, interrupts and final state. Update cache/ABI
  revision when the generated-code contract changes.

- [x] **PERF-08 — Reduce CPU PPU cost without changing timing.**
  Medium risk; depends on PERF-02/03. Profile `render_scanline`, tiles, sprites
  and `compose_into`, separating the original 240×160 pixels from widescreen/
  zoom extension. Reuse invariant data within the same scanline and limit
  processing to necessary areas. Assess decoded-tile caching only with complete
  invalidation for VRAM/palette writes, DMA, loading and restoration.
  **Complete when:** raw images and guest state are identical with windows,
  blending, semitransparent sprites and scanline effects too.
  **PERF-08-UI-PANEL-ROW-CLASSIFICATION — completed and retained.** Banner/
  bounds rules cached per call and membership computed only for the first
  eligible BG of each pixel. Lazy bounds, windows, blending, portraits, sort and
  state/ABI preserved. Native and Windows/Wine 173 core tests/5 optional excluded;
  parity of 3530 frames/state/counters even at zoom6. Five headless pairs:
  6.002->5.841 ms (-2.69%); five full GUI pairs: 7.767->7.591 ms (-2.26%), all
  better, including p95/p99. Worldmap and GUI load/Undo/zoom/reload/close pass.
  Distinct from already completed WIN0/WIN1 cache and rejected selectors; not
  evidence of resolving rare original lag or native Windows performance.
  Export 335 Windows/Wine and 336 Linux tests (15 optional excluded each),
  archive/privacy/DLL import/GLIBC 2.35 checks and cold-cache portable Linux
  load/Undo/reload/closure and screens verified. Previous versions retained;
  saves and translation cache compatible.
  **PERF-08-UI-WORLD-ONLY-PIXELS — completed and retained.** Skip sort/blend only
  when no candidate can enter the UI texture, still writing UiPixel::default to
  clear previous data. Distinct from already completed fully empty rows and
  rejected front selectors; implemented with full-state evidence for portraits,
  blending onto the world and previous data to clear. Native/Wine 174 core tests
  pass; 3530 identical headless frames with state/counters, five pairs
  5.846->5.600 ms (-4.20%). GUI 3530 identical frames and five pairs
  7.584->7.382 ms (-2.67%), all better. Worldmap 240 frames/state/counters and
  two diagnostic pairs pass; zoom6 identical over 3530 frames. Normal GUI
  load/Undo/reload/zoom/closure and screens pass without fallback. Docker export
  336 Windows/Wine and 337 Linux tests (15 optional excluded each); archives/
  privacy/DLL/GLIBC 2.35 and extracted cold-cache portable Linux pass. Local dist
  updated with previous archives retained.
  **PERF-08-COMPACT-UI-LAYER-ID — rejected, baseline restored.** Local u8 tags
  instead of usize, same algorithms. Native/Wine 174 core tests and all 3530 GUI/
  headless frames/state/counters identical. Five headless pairs: -1.348%; one
  diagnostic profile opposite (+1.367%). GUI with CPU profile: -0.454%, without
  diagnostics: -0.774%; both with one unfavorable pair and worse maxima. Modest/
  inconsistent GUI benefit relative to variability: no robust useful advantage
  demonstrated for changing already validated code. Previous source/binary
  restored byte-for-byte, evidence retained; archives unchanged. Do not repeat
  the experiment for context or merely because the parent is open. Off-screen
  effects affecting GBA state must remain active.

- [ ] **PERF-09 — Reduce contention and blocking work in audio.**
  Medium/high risk; depends on PERF-02/03. Measure the mutex shared between
  `build_stream` in [main.rs](../../crates/recomp/src/main.rs) and
  `handoff_audio`: the callback holds it while processing samples. Also check
  fast-forward path in [ff_audio.rs](../../crates/recomp/src/ff_audio.rs).
  The first measurement with the new observer shows mutex wait p99 below
  1 microsecond in the Linux sample: this alone does not justify an SPSC
  refactor. Reuse these data and the testable callback; do not recreate diagnostics.
  **PERF-09-INTERACTIVE-GATING — completed.** Optional guest-frame window for
  one traversal, pause/priming/fast-forward separated; load/Undo during capture
  interrupts it. Same idle battle, frames 8596:9196 after load 8476: 236/235
  faithful/enhanced callbacks, zero faithful source hold/drop/over-period; total
  p99 0.548/0.665 ms, mutex 0.000912/0.000711 ms. These are not AI times,
  inter-mode performance comparison or enhanced starvation/XRUN measurement.
  GUI with zoom/Undo instead observes 1309 holds in 7 playing callbacks
  (transitions and concurrent compilation): not a steady-state comparison or
  evidence of an audible defect. Native 60 runtime tests and container 338
  Wine/339 Linux pass, 15 optional excluded in exports; cold-cache portable
  Linux and off/benchmark behavior verified. Reuse the gate without repeating
  this implementation.
  **PERF-09-ENHANCED-SOURCE-EVENTS — completed.** Raw counters and configured
  destination for PSG/HLE/FIFO, with existing gate/context and per-request
  counters excluded from normal DSP specialization. Sample/queue/interpolator
  state parity verified with partial pairs, DAC hold, routing, inactive HLE and
  backlog too; 63 runtime tests pass. In idle window 8596:9196, 236 callbacks
  per mode and no empty reads from any source. In enhanced mode PSG/HLE are
  configured destinations, FIFO only warm; no enhanced destination in faithful
  mode. GUI with transitions instead observes 2741 empty PSG/HLE and 660 per-FIFO
  requests in one playing callback, without enhanced destination: data distinct
  from idle measurement, not audible defects inferred from counters. Export 341
  Wine/342 Linux tests (15 optional excluded each), archive/privacy/compatibility
  and cold-cache portable Linux load/Undo/off/benchmark pass.
  **PERF-09-BACKEND-OBSERVABILITY — completed for local PipeWire.** One pw-top
  with line output, PID->client->stream->link->sink association and stable
  serials. In running samples of idle battle (~9s/10 samples), pause (~2s/3)
  and resume (~5s/6), stream and sink ERR deltas are 0; cadence verified with
  maximum gap <1.001s. Sink already has ERR 1 at first idle sample: initial zeros
  in creating state are not a running baseline. This does not prove zero ALSA/
  hardware XRUNs or the cause of that count. The first buffered-output trace is
  excluded from phase attribution and retained as negative. Full app gate 235
  playing callbacks without hold/drop/over-period/device errors, battle and
  resume displayed and clean closure; private data unchanged. No runtime/CPAL/
  configuration change or rebuild required. Tested scenes do not justify SPSC
  queues/larger buffering; native Windows, hardware and genuinely slow scenes
  remain. Reassess CPU cost with PPU now optimized before adding audio changes.
  Do not confuse counts with cost. First reduce critical sections and
  allocations; consider bounded SPSC rings only if contention remains dominant.
  Preserve event order, capacity, reset, crossfade and device recovery.
  **Complete when:** spikes/underruns improve without increasing latency, with
  checks for pause, mode change, fast-forward, load/Undo and closure.

- [ ] **PERF-10 — Fix pacing only after distinguishing work and waits.**
  Medium risk; depends on PERF-02/03.
  **PERF-10-PRESENTATION-CADENCE — completed.** Optional observer on existing
  FPS timestamp: intervals of successful presentations and guest advances,
  maximum 18000 samples, frame window and lifecycle anchors. Surface skips visible
  as longer intervals; no gap across pause/priming/FF/zero-size, restore/zoom/
  resize or different backends. No pacing or clock changes when off.
  CPAL/Xvfb/llvmpipe matrix, frames 8596:9196: clock 600 presentations/599
  intervals, mean 16.7445/16.7427 ms, p99 ~17.96 ms; audio 237/236, multiple
  guest frames in every interval, mean 42.6843/42.6692 ms, p99 52.77/52.31 ms.
  Requested VSync is not hardware 60/high-Hz coverage; clock is already the
  Linux/Windows default. Silent fallback with ALSA invalid only in the process,
  66 runtime tests and 344 Wine/345 Linux tests pass (15 optional excluded in
  exports). Archive/privacy/compatibility and cold-cache portable Linux load/
  Undo/off/benchmark pass; archives updated. Do not repeat tool/matrix to
  reconstruct context.
  **PERF-10-POST-WAIT-COMPOSE — rejected, baseline restored.** Purity verified;
  five realtime pairs have worse p99 in 4/5 cases despite small p95 improvement.
  Previous source/runtime restored exactly, archives unchanged; no candidate
  verification remains pending.
  [Decision and measurements](../changes/2026-10-21-evaluate-composition-before-deadline.json).
  Do not reopen the attempt merely because the parent is open.
  Compare `FFTA_FRAME_PACING=clock`/`audio` modes, VSync and surface acquisition
  times. Verify interaction between sleep and presentation blocking before
  changing scheduling. Assess an event-loop-managed wait instead of sleep in the
  callback while retaining input handling. Separately measure any pauses caused
  by autosave every 300 frames: moving it to a worker requires write ordering
  and reliable flush.
  **Complete when:** cadence p99 improves on 60 and high-frequency monitors
  without audio drift, guest slowdown or less responsive input. Do not tie the
  GBA clock to monitor refresh or skip simulation.


## P3 — GPU e parallelismo incrementali

- [ ] **PERF-11 — Prototipare LUT e filtri temporali sulla GPU.**
  Rischio medio/alto; dipende da PERF-02/03/05. Procedere solo se questi passaggi
  occupano una parte significativa del budget CPU.
  Conservare core e framebuffer BGR555 sulla CPU; caricare il frame raw e usare
  LUT/storia in texture. Partire da Off/Simple, poi Smart e infine Persistence.
  Preferire un pass grafico semplice; un compute shader non è obbligatorio.
  Preservare arrotondamenti e fusione nello spazio dei colori già codificati.
  **Completato quando:** costo totale CPU+trasferimenti+GPU e p99 migliorano,
  con confronti CPU/GPU e fallback CPU. La storia avanza per ogni frame guest,
  anche se una presentazione viene saltata o il fast-forward produce più frame.
  Pausa, caricamento, Undo, zoom e cambio LUT devono mantenere il comportamento
  attuale. Niente readback sincrono per frame; anteprime e salvataggi devono
  funzionare senza introdurre attese GPU nel normale ciclo del gioco.

- [ ] **PERF-12 — Valutare sul desktop il parallelismo già presente per lo zoom.**
  Rischio medio; dipende da PERF-02/03/08.
  Provare `render_zoom_parallel` solo sulle righe aggiuntive, a stato guest
  congelato nel VBlank. Usare un pool persistente, una soglia misurata e il
  percorso seriale per lavori piccoli. Non aggiungere thread per ogni frame.
  **Completato quando:** equivalenza raw anche per pannelli e blending, nessuna
  mutazione concorrente del guest e vantaggio su CPU con pochi e molti core,
  senza peggiorare audio o p99. Il test `parallel_rows_match_serial...` è una
  base, non una prova di prestazioni desktop.

## P4 — Esperimenti da rimandare

- [ ] **PERF-13 — PPU estesa sulla GPU, solo con evidenza residua forte.**
  Rischio alto; dipende dalle misure dopo PERF-08/11/12.
  Valutare prima una porzione isolata delle estensioni host a snapshot stabile.
  Una PPU GPU completa deve rappresentare correttamente cambi di registri,
  VRAM, palette, finestre e DMA durante il frame: una fotografia a fine frame
  non basta in generale. Servono confronto col riferimento CPU e percorso
  sperimentale disattivato per default.
  **Completato quando:** vantaggio sostanziale e ripetibile anche con costi di
  sincronizzazione, equivalenza e recupero su backend differenti. Scartare
  l'esperimento se la complessità supera il beneficio.

Non metterei l'IA guest sulla GPU nel piano iniziale: il percorso attuale
esegue istruzioni ARM/Thumb con dipendenze da memoria, RNG e interrupt.
Trasformarlo in kernel o sostituire l'IA con una riscrittura host richiederebbe
una nuova prova di equivalenza. Anche parallelizzare arbitrariamente CPU,
PPU e audio, usare fast-math o eliminare controlli di sicurezza per aumentare
gli FPS introduce rischi sproporzionati rispetto alle opzioni precedenti.

## Regole per accettare ogni intervento

Una modifica alla volta, con diff piccolo e una responsabilità chiara. Estrarre
moduli solo quando aiuta l'intervento corrente; evitare refactoring massivi di
`play.rs`/PPU insieme alla modifica algoritmica. Riutilizzare `wgpu`, strumenti
e test esistenti; nuove dipendenze o `unsafe` richiedono una necessità concreta.

- Correttezza: replay e controlli PERF-03 passano; test core seriali per i
  globali di display condivisi. Per filtri GPU, specificare prima eventuali
  tolleranze motivate; nessuna tolleranza sullo stato guest o sui save.
- Prestazioni: beneficio ripetibile oltre la variabilità fra run nella scena
  bersaglio, senza regressioni significative nelle altre. Conservare anche
  misure sfavorevoli; scartare modifiche senza beneficio utile.
- Compatibilità: Ubuntu Wayland/RADV e X11, fallback software, Windows nativo
  quando disponibile; Wine è una verifica distinta. Backend alternativi e
  hardware non provati devono restare esplicitamente non verificati.
- Persistenza e lifecycle: caricamento/Undo, backup/ripristino, cambio ROM,
  spostamento portable, resize e chiusura pulita secondo l'area toccata.
  Cache e risorse host devono essere ricostruibili e invalidabili. Cambiare
  versione dei savestate solo se cambia il loro contratto, non per un filtro.
- Memoria di progetto: registrare risultati reali, limitazioni e decisione
  tenere/scartare in `docs/changes`, poi rigenerare il changelog. Un TODO resta
  aperto fino a implementazione e verifica; AndroidPort richiede lavoro separato.

**Prima tranche consigliata:** PERF-01/02/03, quindi PERF-04/05/06.
Le priorità successive vanno riordinate con i dati: PERF-07 se domina
l'esecuzione dei turni, PERF-08/12 se domina lo zoom, PERF-09 se domina
l'audio, PERF-10 se dominano le attese, PERF-11 se dominano LUT e filtri.
