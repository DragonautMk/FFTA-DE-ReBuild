# FFTARecomp — Analisi preliminare di integrazione upstream v3.1

**Versione:** 0.2 — 2 novembre 2026 (decisioni di indirizzo approvate)**Tipo:** documento decisionale e piano di audit; scelte di indirizzo approvate dal proprietario, **non** autorizzazione a modificare il codice o eseguire merge**Ambito:** upstream `Jorge262007/FFTARecomp` → fork `DragonautMk/FFTARecomp`, branch `decomp`**Autore dell'analisi:** ChatGPT, su lettura delle repository GitHub e della release, **senza build, merge o esecuzione del gioco**.

> **Decisione approvata:** mantenere `decomp` come base autorevole, integrare le **funzionalità**, non necessariamente i commit upstream. Non eseguire `git merge upstream/main` né cherry-pick massivi senza audit e prove. Il progetto del fork è soggetto a una policy _English-first_: se questo documento verrà inserito nella repository, produrne una traduzione inglese revisionata, senza modificare ID, link o risultati.

## 0. Decisioni confermate dal proprietario — 2 novembre 2026

Il proprietario ha confermato di concordare con **le raccomandazioni strategiche** presentate nell'analisi v0.1 e nel riepilogo successivo. Questa conferma è **autorizzazione a registrare le decisioni e preparare l'audit**, non approvazione implicita di un merge, di una modifica del codice, dell'esecuzione di test costosi o di un nuovo workflow asincrono.

Le decisioni approvate sono:

1. **Porting selettivo anziché merge generale.** La branch `decomp` resta la fonte autorevole; ogni funzione upstream va confrontata con le implementazioni già presenti.
2. **Non regredire sulle ROM.** Preservare il supporto EUR e gli eventuali percorsi USA realmente già funzionanti; la restrizione USA della v3.1 non viene adottata automaticamente.
3. **Menu Atlante del Clan invariato come architettura.** Valutare solo miglioramenti rilevanti delle finestre _in-game_, scrollbar, animazioni e correzioni visive; non importare il menu upstream per sostituire quello del fork.
4. **Mouse/touch e scrollbar: porting per piccoli ambiti verificabili.** Priorità all'effettivo miglioramento di una scena alla volta, Android e desktop considerati separatamente.
5. **Autosave dello stato macchina: sì alla progettazione, separato e configurabile.** Nessuna sostituzione dei salvataggi manuali, dei 9/99 snapshot, dei 99 cartridge saves, di Undo o dei backup. Non adottare automaticamente il periodo di 5 secondi né il formato upstream: prima una specifica e prove di compatibilità/recovery. **L'attivazione predefinita, l'intervallo, la rotazione e l'esperienza di ripresa devono essere definiti in specifica prima di implementare.**
6. **Correzioni di rendering e transizioni: solo dopo confronto.** I miglioramenti host possono essere adattati, preferibilmente disattivabili, senza cambiare indebitamente la logica guest.
7. **Fast Forward al 150%: non prioritario.** Valutare un preset compatibile solo se desiderato; non sostituire la scala percentuale e il clock razionale del fork.
8. **Steam Deck Y: soltanto contestuale/configurabile**, senza collisioni con l'uso del tasto nell'Atlante e nei binding esistenti.
9. **Installer, packaging, colorimetria e grandi refactor: differire** salvo difetti concreti o benefici dimostrabili; niente adozione automatica.

**Significato degli stati:** `APPROVED_POLICY` = indirizzo autorizzato; `PENDING_TECHNICAL_AUDIT` = presenza, vantaggi e rischi da provare nel codice; `PENDING_DESIGN` = dettagli UX e formato non ancora approvati; `APPROVED_IMPLEMENTATION` = non assegnato ad alcuna funzione in questa versione. Non trasformare `APPROVED_POLICY` in `VERIFIED`.

**Prossimo passo autorizzato da questo documento:** audit documentale/read-only delle 12 famiglie, con proposta di piccoli task separati e criteri di accettazione. Nessuna modifica al runtime o fusione diretta finché non viene scelta e approvata una specifica unità d'integrazione.

## 1. Riferimenti verificati e limiti

| Ruolo                                       | Riferimento                                                                                                                                                       | SHA fissato                                          |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| Upstream                                    | [Jorge262007/FFTARecomp](https://github.com/Jorge262007/FFTARecomp) — `main`                                                                                      | `6bdddbc559ac459c4a18325c15c28d040d87f407`           |
| Fork                                        | [DragonautMk/FFTARecomp](https://github.com/DragonautMk/FFTARecomp/tree/decomp) — `decomp`                                                                        | `bd66b8763623847af9f093a03fd416e4a10e2029`           |
| Antenato comune (dalla comparazione GitHub) | commit condiviso                                                                                                                                                  | `fcc38cc46084793f29136419a8a15e6914a570ce`           |
| Novità dichiarate                           | [Release upstream 3.1](https://github.com/Jorge262007/FFTARecomp/releases/tag/3.1)                                                                                | 1 novembre 2026                                      |
| Modifiche upstream                          | [Diff dal comune antenato](https://github.com/Jorge262007/FFTARecomp/compare/fcc38cc46084793f29136419a8a15e6914a570ce...6bdddbc559ac459c4a18325c15c28d040d87f407) | 5 commit                                             |
| Storico fork                                | [Branch decomp](https://github.com/DragonautMk/FFTARecomp/tree/decomp)                                                                                            | 117 commit oltre l'antenato nel confronto incrociato |

**Rilevazione:** 173 file modificati nell'upstream rispetto all'antenato; 30 percorsi modificati **da entrambi** i rami (nessun file identico fra i 30 per SHA). I 30 sono **potenziali conflitti logici**, non 30 conflitti Git già dimostrati. Fra essi: `crates/recomp/src/play.rs`, `main.rs`, `crates/gba-core/src/mem.rs`, `ppu.rs`, `crates/menu/src/lib.rs`, `crates/input-config/src/lib.rs`, `crates/screen/src/ui_layer.rs`.

**Attenzione:** un file nuovo nell'upstream può comunque creare conflitti funzionali tramite API, formato di salvataggio, input o assunzioni sulla ROM. Il numero di commit non è un indicatore affidabile di complessità: la v3.1 contiene un aggiornamento cumulativo e commit di merge.

**Non verificato in questa fase:** build della v3.1, esecuzione reale, corretta applicazione delle modifiche su `decomp`, test Windows/Linux/Android, conflitti Git prodotti da un merge di prova, effettiva superiorità delle nuove funzionalità, modifiche non ancora committate o processi locali. Le note della release descrivono intenzioni e comportamenti dichiarati, non accettazione indipendente.

## 2. Principi da preservare nella branch decomp

1. **ROM EUR come priorità**, ma non cancellare o degradare intenzionalmente la compatibilità USA esistente senza una decisione esplicita. La v3.1 upstream dichiara temporaneamente la ROM USA non supportata.
2. **Separazione guest/host:** modifiche visive o di input non devono alterare stato, frame o clock del gioco, salvo funzionalità che richiedano deliberatamente un cambiamento.
3. **Salvataggi esistenti:** mantenere 9/99 slot snapshot, Undo, backup, file standard numerati e compatibilità con i file esistenti; l'upstream usa un sistema differente di salvataggi macchina automatici e schede.
4. **Menu Atlante del Clan:** navigazione e layout personalizzati, localizzazione `en`/`it`, Debug, profili e scorciatoie non vanno sostituiti con il vecchio menu upstream.
5. **Debug e ROMDATA:** preservare audit, recorder, cache native, provenance e test già accettati; non fare test storici completi solo per ricostruire il contesto.
6. **Doppia linea Android:** `AndroidPort/` è un albero separato dalla root; nessuna modifica del desktop si assume automaticamente applicata ad Android, o viceversa.
7. **Lavori asincroni:** prima di mutare la working tree controllare stato, owner e commit degli altri task; lavorare su branch/worktree isolati e non sovrascrivere file non propri.

## 3. Matrice decisionale — indirizzi approvati, applicabilità da verificare

Legenda: **ADOTTA** = integrare la funzione dopo prove; **ADATTA** = ricostruire solo il comportamento compatibile; **CONFRONTA** = valutare prima se c'è davvero un vantaggio; **RIMANDA** = non bloccare lo sviluppo attuale; **NON IMPORTARE** = non prendere l'implementazione upstream così com'è.

| ID     | Novità upstream                                       | Stato rilevato nel fork                                                                                           | Decisione di indirizzo approvata                                                                                                 | Difficoltà                        | Priorità                             |
| ------ | ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- | ------------------------------------ |
| U31-01 | Fast Forward **1,5×**                                 | `ff.percent` da 100% a 800% con incrementi di 20; **150% non previsto**                                           | **ADATTA:** eventuale preset 150% nel sistema esistente, senza sostituire il clock razionale                                     | Bassa–media                       | P2                                   |
| U31-02 | Barre di scorrimento nei menu di gioco                | Atlante gestisce già scroll interno, ma non equivale a scrollbar _nelle liste del gioco_                          | **ADOTTA selettivamente:** geometria/drag per liste originali realmente non servite                                              | Media                             | P1                                   |
| U31-03 | Correzioni di finestre e fumetti                      | Fork possiede proprie estensioni grafiche e strumenti diagnostici                                                 | **CONFRONTA** scena per scena; trasferire solo i bugfix riproducibili                                                            | Medio–alta                        | P1                                   |
| U31-04 | Animazioni entrata/uscita e zoom fluido               | Implementazione upstream host-only; renderer e sincronizzazione fork divergenti                                   | **ADATTA:** isolare UI motion dalle transizioni guest, con opzione disabilitabile                                                | Medio–alta                        | P2                                   |
| U31-05 | Menù widescreen senza bande nere                      | Atlante è già un menu nativo adattativo; il rendering di gioco è distinto                                         | **NON IMPORTARE** l'intero menu; **CONFRONTA** solo il rendering delle finestre originali                                        | Alta se merge; media se selettivo | P2                                   |
| U31-06 | Mouse e touch migliorati (soprattutto Android)        | Esistono percorsi mouse/touch, Android separato; il nuovo upstream aggiunge parecchie routine dedicate            | **ADOTTA per aree:** Android prima; desktop solo dopo verifica dei controlli non ancora gestiti                                  | Alta                              | P1 Android / P2 desktop              |
| U31-07 | Autosave macchina ogni 5 secondi                      | Fork ha `.sav` automatico e snapshot manuali 9/99 con Undo: **non sono lo stesso tipo di salvataggio**            | **CONFRONTA/PROTOTIPA:** autosave macchina opt-in in file dedicato e con recupero sicuro, senza sostituire le funzioni esistenti | Alta                              | P1 di analisi; P2 di implementazione |
| U31-08 | Più di 2 schede/slot di salvataggio                   | Fork supporta 99 slot snapshot e fino a 99 file cartridge; upstream modifica anche la UI originale di salvataggio | **NON IMPORTARE** il sottosistema completo; verificare se la UI interna del gioco offre qualcosa di diverso e utile              | Alta se importato                 | P3                                   |
| U31-09 | Steam Deck: menu con tasto Y                          | Nel fork Y/North è già usato per cambiare vista Debug in alcuni contesti                                          | **ADATTA** come binding contestuale/configurabile, senza sovrascrivere Y globale                                                 | Bassa–media                       | P2                                   |
| U31-10 | Rendering video/colorimetria e cache grafiche         | Modifiche estese al renderer e all'AndroidPort; alcune non sono evidenziate nella release                         | **RIMANDA/CONFRONTA** per componenti isolati e risultati visivi, senza import massivo                                            | Alta                              | P3                                   |
| U31-11 | Script di installazione/build e pacchetti             | Fork dispone già di workflow Docker/portable differenti                                                           | **CONFRONTA** difetti specifici; mantenere packaging fork se funzionante                                                         | Media                             | P3                                   |
| U31-12 | Cambio dei requisiti ROM, USA temporaneamente esclusa | Obiettivo fork include percorsi EUR e USA, con prove distinte                                                     | **NON IMPORTARE** l'esclusione USA come policy del fork                                                                          | Alta (rischio regressione)        | Vincolo P0                           |

**P1/P2/P3 restano priorità consigliate, non risultati di test né autorizzazioni a eseguire un pacchetto.** Se un miglioramento è già presente con qualità equivalente, marcarlo `SKIP_ALREADY_PRESENT` indicando file e prova.

## 4. Decisioni tecniche motivate e criteri di accettazione

### U31-01 — Fast Forward 150%

- **Evidenza:** nell'upstream `ff.speed` ammette `[1.5, 2.0, 3.0, 4.0]`; nel fork `crates/input-config/src/fast_forward.rs` usa `PERCENTS` 100,120,…,800. [Upstream](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/input-config/src/lib.rs) · [Fork](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/crates/input-config/src/fast_forward.rs).
- **Scelta:** offrire 150% come passo di 10 in una futura scala: _non_ forzare la conversione a 140/160 né migrare `ff.speed` float integralmente.
- **Accettazione:** parsing e serializzazione vecchie configurazioni invariati, pacing/clock corrispondenti al 150%, audio coerente, spalla ±20 e opzioni menu con comportamento dichiarato. La semantica della sequenza ±20 partendo da 150% va definita prima del codice.

### U31-02 — Scrollbar e input mouse

- **Evidenza:** upstream ha `crates/screen/src/ui_scrollbar.rs` con track/thumb/hit/drag/test, e chiama scrollbar da `crates/recomp/src/play.rs` per elenchi (`roster_scrollbar`, `pub_scrollbar`). [Scrollbar](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/screen/src/ui_scrollbar.rs).
- **Scelta:** prima identificare i menu originali **non ancora scrollabili via mouse** nel fork. Adattare il modello di geometria se i test dimostrano utilità; non aggiungere un secondo sistema di scroll nell'Atlante.
- **Accettazione:** drag, rotella, controller e selettori originali coerenti; niente doppia azione né selezione del dato errato; equivalenza stato guest a parità di scelta.

### U31-03/04 — Correzioni visive e transizioni

- **Evidenza:** upstream aggiunge `crates/recomp/src/ui_motion.rs` (transizioni `Instant`, durata ~0,15 s; chiusura dialogo ~0,22 s) e `crates/gba-core/src/ffta_ui.rs`; cambia significativamente `ffta_dialogue.rs`, `mem.rs`, `ppu.rs`, `play.rs` e presentazione. [UI motion](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/recomp/src/ui_motion.rs).
- **Scelta:** separare i bugfix oggettivi (immagini errate, geometria, pop, focus) dagli effetti estetici opzionali (slide/ease). Non introdurre modifiche al guest solo per effetti host.
- **Accettazione:** test immagini e scena su casi mirati, comparazione prima/dopo, frame/clock invariati quando l'animazione è solo host; controlli continui durante apertura e chiusura.

### U31-05 — Widescreen

- **Evidenza:** la release afferma che lo sfondo del menu originale è un'immagine allargata; il fork usa l'Atlante con sidebar e layout adattativo. [Atlante](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/docs/menu_clan_atlas.md).
- **Scelta:** non trasferire aspetto e layout del menu upstream nell'Atlante; valutare separatamente bugfix di scene/finestre del gioco originale e comportamento ultrawide.
- **Accettazione:** nessuna regressione nel menu a 480×360, 640×480, 854×480, 1280×720 e widescreen; no hitbox fuoriposto.

### U31-06 — Android, mouse e touch

- **Evidenza:** upstream aggiunge `ffta_roster_mouse.rs`, modifica `ffta_battle_mouse.rs`, `MainActivity.kt`, `GameSurface.kt`, `engine.rs`, `touch.rs`, aggiunge `host_layout.rs` e altre routine; la release dichiara una copertura touch significativamente maggiore. [Android upstream](https://github.com/Jorge262007/FFTARecomp/tree/6bdddbc559ac459c4a18325c15c28d040d87f407/AndroidPort).
- **Scelta:** audit per scene: Clan/Party, Area, Store, Battle, finestre e selezione. Implementare prima una sola famiglia Android nella relativa branch isolata. Non copiare assieme renderer, salvataggi e input.
- **Accettazione:** scelta touch corrispondente alle stesse azioni tramite pad, nessun cambio selezione involontario, UI host e guest sincronizzate, prove su dispositivo o emulatori dichiarati (distinte dalla sola compilazione APK).

### U31-07/08 — Autosave, salvataggi e ripresa (principio approvato)

- **Evidenza:** `upstream/crates/recomp/src/autosave.rs` contiene un trigger `Duration::from_secs(5)`, cattura `m.save_state()` e accoda il risultato a un worker; ha file `QuickSave` e `SaveN`/schede del gioco. Il fork dispone già di snapshot 9/99 con Undo e file cartridge 1–99, _non necessariamente dello stesso flusso automatico di ripresa_. [Autosave upstream](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/recomp/src/autosave.rs) · [Salvataggi fork](https://github.com/DragonautMk/FFTARecomp/blob/bd66b8763623847af9f093a03fd416e4a10e2029/docs/extended_saves.md).
- **Scelta approvata:** non portare l'intero modulo da ~2.000 righe. Preparare una specifica di **autosave macchina in file dedicato**, configurabile e indipendente da `.sav`, snapshot manuali, Undo e backup. Sono ancora **da definire e validare** attivazione predefinita, intervallo (5 s non approvati), offerta di ripresa al lancio, politica di retention e fallimenti/rotazione.
- **Accettazione obbligatoria:** interruzione durante scrittura, crash/ripristino, ROM/region/versione diverse, save in corso, cache nativa, registrazioni/replay/debug, spazio disco, limite scritture, riapertura senza corrompere file manuali. Verificare se ogni 5 secondi è desiderabile o eccessivo; non imporlo come valore predefinito senza scelta.
- **Nota:** l'esistenza di 99 slot snapshot non prova che la UI _in-game_ della v3.1 sia già equivalente: sono concetti diversi. Verificare prima di scartare tutte le UX upstream.

### U31-09 — Steam Deck Y

- **Evidenza:** upstream dichiara Y per il menu port su Steam Deck; nel fork `docs/menu_clan_atlas.md` documenta Y/North per il cambio vista nella pagina Debug.
- **Scelta:** non assegnare Y globalmente. Valutare mapping per tipo dispositivo solo con esclusività di contesto e preferenze persistenti; mantenere le associazioni dell'utente.
- **Accettazione:** nessuna collisione con tasti di gioco, menu, cattura binding, Debug e Steam Input.

### U31-10 — Rendering/colorimetria

- **Evidenza:** upstream aggiunge nel ramo Android `video/color.rs`, `lut.rs`, `profile.rs` e nuove routine grafiche/asset cache; la sola presenza di un modello di colore non garantisce che sia più fedele al pannello originale.
- **Scelta:** valutazione tecnica separata solo quando prioritario; verificare output neutro, gamma, palette, prestazioni e reversibilità. Non far transitare queste modifiche attraverso il task ROMDATA per confondere presentazione con dati estratti.

### U31-11/12 — Installer e ROM

- **Scelta:** mantenere gli script di build/portable già in uso nel fork salvo bug specifico dimostrato; importare eventuali fix minimi. **Non** ereditare la restrizione USA introdotta dall'upstream v3.1. La compatibilità USA va provata, non semplicemente assunta.

## 5. Pacchetti operativi suggeriti a Codex

### P0 — Audit, nessuna modifica

1. Controllare lo stato del checkout (`git status --short`), remoti, SHA fissati e merge-base; evitare di toccare processi o modifiche altrui.
2. Produrre un file Markdown/JSON per ogni `U31-xx` con: intent upstream; API; file; chiamanti; differenze fork; test disponibili; decisione `PORT/ADAPT/SKIP/DEFER`; rischio; prova di accettazione.
3. Non leggere storici di megabyte. Usare `TASK_HANDOFF.md` breve, `git diff --stat` e parti mirate dei file.
4. Per elementi già presenti, cercare evidenza di comportamento prima di ricrearli. Segnalare esplicitamente i dubbi.
5. Fermarsi alla pubblicazione del rapporto; **nessun merge automatico**.

### P1 — Prime integrazioni candidate

1. **Scrollbar nelle liste del gioco**: modulo e un solo elenco campione; verificare hitbox e controller. Eseguire solo se il fork ne è realmente privo.
2. **Correzioni visive riproducibili**: selezionare 1–2 bugfix dalla v3.1, isolando i cambiamenti host.
3. **Android touch**: una scena piccola e verificabile, distinta dal desktop.
4. **Progetto autosave**: prima solo specifica e verifica dei formati, senza adottare silenziosamente il trigger da 5 secondi.

### P2 — Integrazioni opzionali

- 150% Fast Forward come preset coerente con la scala del fork.
- Animazioni host e zoom fluido, con disattivazione e confronto visivo.
- Nuovo mapping Steam Deck solo configurabile e contestuale.
- Autosave implementato dopo approvazione della specifica, compatibilità dei formati e test di recovery.

### P3 — Non urgenti

- Colorimetria video e pipeline Android completa.
- Aspetto grafico del menu upstream, se non offre un vantaggio distinto dal menu Atlante.
- Rifacimento installer/build senza regressione concreta.

**Per ciascuna unità:** sviluppare in branch/worktree isolato, annotare SHA upstream e fork, commit minima, test positivi/negativi mirati, risultato e limiti. Fare integrazione nella branch principale soltanto dopo revisione delle prove. Evitare una nuova copia dell'intera checklist nell'handoff operativo.

## 6. Registro delle decisioni: approvate e ancora aperte

**Conferma del proprietario ricevuta il 2 novembre 2026.** Le scelte di **indirizzo** D1–D7 sono approvate; l'implementazione resta subordinata all'audit di applicabilità, alle prove e a una selezione esplicita del prossimo task.

| ID  | Decisione di indirizzo                                                                                                                       | Stato                                | Dettagli ancora aperti                                                                     |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------ |
| D1  | Porting selettivo; `decomp` base autorevole; nessun merge massivo                                                                            | **APPROVED_POLICY**                  | Scelta e ordine delle prime unità dopo audit                                               |
| D2  | Non importare l'esclusione USA; preservare le compatibilità dimostrate                                                                       | **APPROVED_POLICY**                  | Matrice di regressione per regione e piattaforma                                           |
| D3  | Progettare un autosave macchina **separato, configurabile e recuperabile**; mantenere manuali, Undo, backup, 9/99 snapshot e cartridge saves | **APPROVED_POLICY + PENDING_DESIGN** | Default on/off, intervallo, recovery prompt, formato, retention, condizioni di sospensione |
| D4  | Mantenere Atlante; adattare soltanto miglioramenti visivi utili e, dove possibile, opzionali                                                 | **APPROVED_POLICY**                  | Confronto scena per scena; default e opzioni di animazione                                 |
| D5  | Mouse/touch Android per scene in worktree separato, senza bloccare desktop o ROMDATA                                                         | **APPROVED_POLICY**                  | Prima scena e piattaforma di prova                                                         |
| D6  | Steam Deck Y solo contestuale/configurabile                                                                                                  | **APPROVED_POLICY**                  | Priorità dei binding e contesti effettivi                                                  |
| D7  | 150% Fast Forward non prioritario; possibile preset speciale, preservando 100–800%                                                           | **APPROVED_POLICY**                  | Comportamento di ±20 partendo da 150%, soltanto se implementato                            |

**Decisioni rimaste volutamente non prese:** nessuna attivazione automatica di autosave ogni 5 secondi, nessun ripristino automatico che sovrascriva una sessione senza conferma, nessuna rinuncia agli slot esistenti e nessuna modifica dell'architettura degli strumenti di debug o di ROMDATA. Questi aspetti richiedono una specifica verificata e, quando incidono sul comportamento utente, ulteriore approvazione.

### Ordine consigliato dopo l'audit

1. `U31-02` **scrollbar dei menu originali**: un solo elenco con differenza comprovata rispetto a `decomp`.
2. `U31-03` **un bugfix grafico riproducibile**: prova di prima/dopo senza regressione guest.
3. `U31-06` **una scena mouse/touch Android**: isolamento da desktop e dal renderer generale.
4. `U31-07` **specifica tecnica dell'autosave**: design e analisi della compatibilità, **non** implementazione automatica.

Per qualunque task successivo all'audit, produrre prima una scheda proposta con diff minimo, owner, file concorrenti, costo dei test e criteri di accettazione. Attendere una selezione esplicita prima di modificare codice o effettuare merge.

## 7. Template per i successivi aggiornamenti

```md
### U31-XX — Nome della modifica

- Upstream source SHA: ...
- Fork baseline SHA: ...
- Fonte e file: ...
- Comportamento upstream dimostrato / solo dichiarato: ...
- Equivalente fork: ...
- Differenza reale: ...
- Dipendenze / conflitti API: ...
- Decisione: PORT | ADAPT | SKIP_ALREADY_PRESENT | DEFER | REJECT
- Motivazione e alternativa scartata: ...
- Verifica minima e criteri di accettazione: ...
- Esito: NOT_STARTED | INVESTIGATED | IMPLEMENTED_UNVERIFIED | VERIFIED | REJECTED
- Evidence links / test output: ...
- Rischi residui: ...
```

## 8. Prossimo controllo e criteri di completamento dell'audit

- [ ] Confermare localmente SHA e merge-base in un worktree aggiornato.
- [ ] Per ciascuna delle 12 famiglie, verificare anche i **chiamanti effettivi** e l'eventuale implementazione equivalente nel fork.
- [ ] Separare chiaramente i cambiamenti dell'albero `AndroidPort/` da quelli desktop.
- [ ] Verificare il formato upstream delle copie automatiche e la compatibilità con i `.ffstate` del fork **prima** di scrivere un adapter.
- [ ] Controllare se i fix upstream interferiscono con hook/debug/ROMDATA e cache nativa.
- [x] Ottenere l’accordo sugli indirizzi D1–D7 (2 novembre 2026); restano **aperti i dettagli di specifica** D3 e le verifiche di applicabilità.
- [ ] Verificare i risultati delle build e dei test come evidenza distinta dalla semplice pubblicazione della release.
- [ ] Trasformare gli indirizzi approvati in **proposte** di singoli task con criteri di accettazione e arresto dopo ciascuna unità; ottenere approvazione della specifica unità prima di mutare il codice.

---

**Conclusione e decisione di indirizzo approvata:** la v3.1 contiene miglioramenti interessanti soprattutto per la qualità d'uso, ma una fusione indiscriminata presenta un rischio alto di regressioni. La scelta razionale è una sincronizzazione **per comportamento**, preservando il lavoro del fork su ROMDATA, Debug, menu e salvataggi. Il documento è un punto di partenza versionato: nessuna voce è marcata `VERIFIED_IN_FORK` finché non viene eseguita e accettata la prova specifica.

## 9. Registro revisioni

| Versione | Data       | Modifica                                                                                                                                                         |
| -------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.1      | 2026-11-02 | Audit iniziale upstream v3.1 vs fork `decomp`; raccomandazioni e limiti senza esecuzione                                                                         |
| 0.2      | 2026-11-02 | Confermati dal proprietario gli indirizzi strategici; distinto consenso al progetto da autorizzazione a modificare il runtime; dettagli autosave lasciati aperti |

**Nota di integrità:** le prove e gli SHA delle due repository restano quelli della v0.1; questa revisione registra decisioni del proprietario e non afferma nuovi risultati di test o nuovi commit controllati.
