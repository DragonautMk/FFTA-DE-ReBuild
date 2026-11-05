# FFTA DE-ReBuild — Save Manager 2D, cronologia e titoli

**Tipo:** piano di implementazione, non modifica del codice.  
**Repository analizzato:** `DragonautMk/FFTARecomp`, branch `main`, commit `8d276a06ec6f5f43e660557a51ffca7c15eb3fc0` (05/11/2026).  
**Obiettivo:** mantenere compatibili i salvataggi esistenti, aggiungendo cronologia orizzontale di Save State per slot, navigazione verticale fra slot, titolo personalizzabile per ogni Save State e per ogni normale salvataggio cartuccia `.sav`, scrollbar a scorrimento rapido.

## 1. Stato del codice verificato

| Componente | File | Stato effettivo | Intervento |
| --- | --- | --- | --- |
| Storage Save State | [`crates/recomp/src/save_state.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/recomp/src/save_state.rs) | `Slots`: 9 o 99 slot; un `slot-N.ffstate` per slot; intestazione `FFTA-SL1`; anteprima incorporata; Undo di una sola operazione | Aggiungere versioni/indice, mantenere lettura dei legacy |
| Storage cartuccia | [`crates/recomp/src/game_saves.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/recomp/src/game_saves.rs) | `GameSaves`: `ffta.sav`, `ffta-slot-02.sav` … `-99.sav`; `choices()` usa il nome del file, non un titolo utente | Metadati esterni associati ai file, senza cambiare contenuti `.sav` |
| Stato menu Save State | [`crates/menu/src/save_states.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/save_states.rs) | `StateSlot` contiene solo `occupied`, `preview`, `status` | ID versione, titolo, data, protezione, stato |
| Stato menu `.sav` | [`crates/menu/src/game_saves.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/game_saves.rs) | `GameSaveSlot` contiene `name`, `status`, `occupied`, `available`, `current` | Separare `filename` da `display_title` |
| Layout | [`crates/menu/src/save_forms.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/save_forms.rs) | Form verticale, un solo slot selezionato e screenshot | Griglia/visualizzatore 2D, barra e pannello dettagli |
| Navigazione | [`crates/menu/src/lib.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/lib.rs) e [`crates/menu/src/interaction.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/interaction.rs) | `Row::StateSlot`/`Row::GameSaveSlot` aprono dropdown; frecce avanzano di 1; il focus generico usa Up/Down | Modalità di navigazione dedicata al Save Manager, con indice riga e indice versione |
| Dropdown | [`crates/menu/src/theme.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/menu/src/theme.rs) | `dropdown_geometry` e `dropdown_hit` selezionano la singola voce sotto il mouse. Non esiste scrollbar trascinabile nel picker dei salvataggi | Nuova scrollbar con traccia, thumb e hit test indipendente; non simulare ripetuti clic |
| Collegamento menu/runtime | [`crates/recomp/src/play.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/recomp/src/play.rs) | `refresh_state_slots` rilegge tutti gli slot e le preview; `state_operation` salva/carica con solo `slot: u8`; `MouseEv::Wheel(bool)` perde l'entità dello scorrimento | API con version ID, metadata/lazy loading, eventi pointer drag e wheel delta |
| Backup cartuccia | [`crates/recomp/src/save_backups.rs`](https://github.com/DragonautMk/FFTARecomp/blob/main/crates/recomp/src/save_backups.rs) | Due backup della cartuccia | Non alterarne logica né path |

**Vincolo UI:** `Page::Saves`, quando Debug è bloccato, mostra solo Save States e Save Backups (`crates/menu/src/lib.rs`). Pertanto il nome del salvataggio cartuccia **attivo** deve essere visibile e modificabile nella sezione normale Game > Saves, anche senza abilitare Extended saves; la navigazione fra 99 `.sav` può invece continuare a dipendere dall'opzione esistente.

## 2. Comportamento approvato

### Save State

- **Asse verticale:** una riga per slot Save State numerico esistente (1–9, o 1–99 con Extended saves). Eventuali etichette `Rapido`, `Manuale`, `Automatico` sono **gruppi UI proposti**, non tipi di salvataggio già implementati: non inventare autosalvataggi finché non esiste una sorgente reale.
- **Asse orizzontale:** versioni successive dello **stesso slot**. Ogni nuova operazione Salva aggiunge una versione, senza sostituire la precedente; caricare una versione storica non cancella le versioni più recenti.
- **Titolo:** personalizzabile per ciascuna versione, con valore predefinito; editabile dopo il salvataggio e indipendente dallo slot. Titoli duplicati ammessi, ID univoci obbligatori.
- **Dettagli:** titolo, tipo/slot, numero versione, data e ora, dimensione, screenshot se disponibile, stato leggibile/non leggibile, protezione contro cancellazione.
- **Gestione:** carica versione selezionata; crea nuova versione; rinomina; proteggi/sblocca; elimina versione con conferma. Nessuna cancellazione implicita delle versioni storiche; eventuale politica di retention futura deve essere esplicitamente opt-in.
- **Navigazione:** passando da una riga all'altra, preservare il cursore della versione preferita di quella riga; se manca, scegliere l'ultima disponibile. Elementi laterali, header e footer stabili durante lo scroll.

### Salvataggi cartuccia `.sav`

- Ogni `.sav` numerato (incluso il file standard **slot 1**) deve poter ricevere un titolo personalizzato, senza rinominare il file e **senza scrivere il titolo nella SRAM**.
- Visualizzare il titolo come etichetta primaria e il filename reale come informazione secondaria, conservando l'indicazione del file attivo.
- `Game > Saves` deve consentire di rinominare il `.sav` attivo anche quando Extended saves è OFF; se ON, mostrare i titoli per gli slot 1–99 esistenti.
- Una copia `.sav` su slot diverso non eredita automaticamente il titolo del file sorgente: il destinatario mantiene il suo titolo o riceve un titolo predefinito. Eventuali cambi di titolo sono espliciti.
- Non introdurre una cronologia delle *versioni `.sav`* in questa feature: i backup cartuccia esistenti continuano a essere indipendenti. La cronologia 2D richiesta riguarda i Save State.

### Scorrimento rapido: requisito centrale

- **Trascinamento del pollice della scrollbar**: posizione relativa sull'intero intervallo (non +1 per evento). Il cursore deve seguire il movimento in tempo reale, senza confermare/caricare il salvataggio.
- **Clic sulla traccia**: salto di una pagina visibile oppure posizionamento proporzionale; fissare una regola comune, consigliato **salto di pagina**, con **Shift+clic** per posizionamento esatto (se implementabile senza conflitti).
- **Rotellina/trackpad**: conservare il delta, consentendo scorrimenti multipli proporzionali e con una soglia/debounce prevedibile; non ridurre tutto a `bool`.
- **Tastiera**: Up/Down una riga; Left/Right una versione; PageUp/PageDown una pagina; Home/End primo/ultimo; Shift+rotella o trackpad orizzontale per le versioni.
- **Controller**: D-pad per movimenti singoli, ripetizione accelerata tenendo premuta una direzione sul Save Manager; eventuali salti multipli contestuali non devono interferire con i tasti globali L1/R1 dedicati al volume.
- **Focalizzazione**: le azioni di scroll **non devono** caricare, salvare, rinominare o cancellare nulla.
- **Accessibilità**: la stessa destinazione deve essere raggiungibile senza mouse, con feedback `Riga N / M`, `Versione X / Y`.

## 3. Modello dati suggerito

Separare **file macchina** e **metadati utente**.

### Save State

**Compatibilità:** non modificare il formato interno delle snapshot né la busta `FFTA-SL1` senza necessità. I file `slot-N.ffstate` preesistenti vanno letti come una prima versione virtuale dell'archivio del relativo slot; nessuna migrazione distruttiva all'avvio.

Layout indicativo:

```text
<save-dir>/SaveStates/<ROM-SHA-256>/
  slot-1.ffstate                     # file legacy, intatto
  slot-2.ffstate                     # file legacy, intatto
  index-v2.json                      # metadati e riferimenti per le versioni
  history/
    slot-001/
      <version-uuid>.ffstate
      <version-uuid>.ffstate
    slot-002/
      <version-uuid>.ffstate
```

Esempio `index-v2.json` (nomi e hash puramente illustrativi):

```json
{
  "schema": 2,
  "slots": {
    "1": [
      {
        "id": "legacy-slot-1",
        "file": "slot-1.ffstate",
        "title": "Prima di Famfrit",
        "created_at": null,
        "protected": true
      },
      {
        "id": "a04d4f62-6f8f-460d-9dd7-000000000001",
        "file": "history/slot-001/a04d4f62-6f8f-460d-9dd7-000000000001.ffstate",
        "title": "Dopo la battaglia",
        "created_at": "2026-11-05T16:50:00+01:00",
        "protected": false
      }
    ]
  }
}
```

**Sicurezza dei path:** costruire il path solo da slot e UUID validati, senza interpretare nomi forniti dall'utente come path; limiti di numero/lunghezza e verifica ROM/BIOS invariati. Non affidarsi esclusivamente a un `index.json` esterno: rilevare file legacy anche se l'indice manca o è corrotto.

**Scrittura transazionale:** (1) serializzare/validare macchina e preview, (2) creare un file versione temporaneo e `sync_all`, (3) pubblicarlo senza sovrascrivere versioni, (4) aggiornare l'indice atomico, (5) riportare il risultato. In caso di crash fra 3 e 4, recuperare o segnalare il file orfano, senza cancellarlo. Se si annulla l'ultima operazione Save, eliminare la nuova versione e ripristinare l'indice; l'Undo Load deve mantenere identico il comportamento precedente. Non rimuovere o sovrascrivere file precedenti.

**Prestazioni:** l'attuale `refresh_state_slots` costruisce tutte le `preview` per 9–99 slot. Non estendere questo comportamento a tutte le versioni; leggere indice e metadati, poi caricare solo anteprima selezionata e vicine, con cache LRU di dimensioni limitate. Non decodificare gli stati macchina per la lista.

### Cartuccia

Utilizzare un sidecar associato alla *famiglia* di file `.sav`, per esempio:

```text
Data/saves/
  ffta.sav
  ffta-slot-02.sav
  SaveTitles/
    ffta.sav.json
```

Esempio:

```json
{
  "schema": 1,
  "titles": {
    "1": "La prima avventura",
    "2": "Nuovo clan"
  }
}
```

Il manifest va identificato con il **nome base completo** (`ffta.sav`, `my.progress.sav`), non solo con lo stem, per evitare collisioni; deve restare valido spostando la cartella portatile. Metadati scritti atomicamente; validare Unicode, lunghezza massima suggerita 80 caratteri, input vuoto come ripristino del titolo predefinito; i file `.sav` e i due backup rimangono bit-per-bit invariati dalla rinomina.

## 4. Fasi operative (task da assegnare a Codex)

| ID | Fase | Attività / output | Dipende da |
| --- | --- | --- | --- |
| SM-01 | Contratti e test legacy | Fotografia test, invarianti di storage, undo, menu, controller e file `.sav`; casi di migrazione | — |
| SM-02 | Metadati `.sav` | Sidecar e test di scrittura atomica, path, portabilità, Unicode, file mancanti | SM-01 |
| SM-03 | Titoli `.sav` nell'UI | Esporre nome per file attivo senza Debug; rinomina e visualizzazione degli slot 1–99 | SM-02 |
| SM-04 | Storage versionato Save State | Indice per ROM, version ID, enumerazione legacy, append versioni, load per ID, delete/protect | SM-01 |
| SM-05 | Undo e sicurezza versioni | Adattare Undo al version ID; proteggere contro race/concorrenza, crash e indice danneggiato | SM-04 |
| SM-06 | Modello UI 2D | Due cursori (`slot`, `version_id`), memorizzazione focus per riga, selezione stabile, pannello anteprima | SM-04 |
| SM-07 | Titoli versioni | Dialogo di rinomina via IME/tastiera e tastiera virtuale/controller; titoli e protezione | SM-04, SM-06 |
| SM-08 | Scrollbar reale | Geometria, hit test, drag/press/release, page jump, delta rotella, navigazione rapida, trackpad | SM-06 |
| SM-09 | Runtime e prestazioni | `play.rs`: routing nuove azioni, lazy preview e cache, feedback e refresh incrementale | SM-03–08 |
| SM-10 | QA e documentazione | Test automatici, prove desktop mouse+tastiera+pad, migration, docs aggiornati, audit release | SM-09 |

**Ordine di sviluppo consigliato:** il naming cartuccia (SM-02/03) può procedere in parallelo al versionamento (SM-04/05); la nuova UI (SM-06/07/08) deve consumare le API definitive, non implementare una cronologia fittizia basata su indici. PR separate per backend, UI, controlli e test, evitando una riscrittura di `play.rs` in un unico commit.

## 5. Modifiche API suggerite

- `Slots::versions(slot: u8) -> Result<Vec<StateVersionInfo>, Error>`
- `Slots::save_new_version(machine, slot, preview, optional_title) -> Result<VersionId, Error>`
- `Slots::load_version(machine, slot, version_id) -> Result<(), Error>`
- `Slots::rename_version(slot, version_id, title)` e `Slots::set_protected(...)`
- `Slots::delete_version(slot, version_id)` solo con conferma esplicita e rifiuto se protetto.
- `GameSaves::title(slot)`, `GameSaves::rename(slot, title)` senza alterare `GameSaves::path`, `switch`, `copy_current`.
- Eventi `MenuAction` con `slot + version_id`, anziché solo `slot`; tipizzare gli identificativi per evitare scambi.
- `SaveManagerViewState { row, version_selection_by_row, vertical_scroll_px, horizontal_scroll_by_row, editing_title, drag }`.
- Input pointer con `Press/Release/Move` e movimento durante drag; `Wheel` deve contenere delta verticali e orizzontali (unità e scala definite), non booleani.

## 6. Test di accettazione obbligatori

1. Il progetto carica invariati tutti i vecchi `slot-N.ffstate` e `.sav`; nessuna migrazione distruttiva. Slot 10–99 nascosti ma non persi se Extended saves OFF.
2. Salvare tre volte nello stesso slot crea tre versioni distinte; caricare la prima lascia tutte e tre accessibili; creare una quarta dopo il caricamento non cancella le altre.
3. Rinominare la seconda versione non modifica la prima o la terza, né il file stato. Riavviare e ritrovare i nomi. Duplicati consentiti.
4. Rinominare `ffta.sav` e `ffta-slot-99.sav` non cambia alcun byte delle cartucce; riavvio e spostamento cartella portatile mantengono i titoli.
5. È possibile rinominare il `.sav` attivo anche senza sbloccare Debug/Extended saves.
6. Trascinando la scrollbar dal primo all'ultimo elemento si raggiunge direttamente il fondo, senza 98 clic; clic sulla traccia salta di pagina. Il drag non esegue mai un load/save.
7. Rotella ad alta risoluzione, PageUp/PageDown, Home/End e controller con accelerazione raggiungono gli stessi slot senza conflitti con volume e fast-forward.
8. La navigazione Up/Down mantiene la posizione orizzontale specifica della riga, evitando salti imprevisti.
9. Uno stato non valido o appartenente a ROM/BIOS differente non altera la macchina. Una voce protetta non viene cancellata per errore.
10. Un indice mancante, malformato o obsoleto non elimina file: recovery/rescan non distruttivo e messaggio comprensibile.
11. Undo Save e Undo Load mantengono le garanzie preesistenti, incluse verifiche contro modifiche esterne. Le conferme di sovrascrittura cartuccia restano attive.
12. Con centinaia di versioni il menu non carica tutte le macchine o tutte le preview contemporaneamente; tempi e memoria restano limitati.
13. UI e hit-test verificati almeno a 480×640, 640×480, 1280×720, 3840×2160; niente spostamenti di elementi fissi durante lo scroll.
14. La nuova UI funziona con mouse, tastiera e controller; inserimento titoli possibile anche senza tastiera fisica.
15. Gli stati e i sidecar dell'utente non finiscono in commit/release (`.gitignore`, audit pacchetti).

## 7. Rischi tecnici da presidiare

- **Undo vecchio vs cronologia:** l'Undo esistente è una singola operazione *in RAM*, non una cronologia; non sostituirlo implicitamente con lo storico permanente. Storico e Undo hanno semantiche diverse.
- **`.sav` vs Save State:** un load snapshot ripristina anche SRAM, poi autosave/exit può scriverla sul `.sav` attivo. Conservare avviso, guardrail e backup esistenti.
- **Crash consistency:** indice e file macchina sono due scritture; predisporre recupero dei file creati ma non indicizzati, e fallimenti atomici dimostrabili con test.
- **Identità del file `.sav`:** non rinominare per riflettere il titolo, altrimenti si rompe la logica `-slot-NN` e la selezione del file condiviso.
- **Input text:** il runtime inoltra già IME al `content_selector`, ma non alle pagine Save; il nuovo editor del titolo necessita di proprio focus e degli eventi di testo (comprese lingue accentate), senza confondere input di gioco, rimappatura o menu.
- **Mouse drag:** l'attuale `WindowEvent::MouseInput` considera solo la pressione e `MouseEv::Wheel(bool)` perde il delta; implementare il trascinamento attraverso lo stato press/release, senza cambiare il comportamento delle altre pagine.
- **Uso del disco:** versioni numerose possono essere grandi. Mostrare dimensioni, spazio disponibile e cancellazione esplicita; niente pruning silenzioso.
- **Portabilità:** testare sia portable `Data/saves` sia installazione con `.sav` personalizzato o percorso passato con `FFTA_SAVE_PATH`; evitare percorsi assoluti nel manifest.
- **Compatibilità e regressioni:** conservare `FFTA-SL1`, i vecchi formati raw e le garanzie di mismatch ROM/BIOS, registrazione input e provenienza cheat già presenti.

## 8. Definizione di completamento

La feature è completa quando un utente può selezionare uno slot in verticale, scegliere una delle sue versioni in orizzontale, attribuirle o cambiarle un titolo, scorrere direttamente fra centinaia di elementi con una vera barra drag-and-drop, caricare quella versione senza perdere le altre, e rinominare separatamente un qualsiasi file cartuccia `.sav` (incluso il file standard), conservando ogni formato e backup preesistente. **Nessun codice è stato modificato nell'ambito di questa analisi.**
