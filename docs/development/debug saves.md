# TODO — Debug saves

Updated October 26, 2026. Stable numbering **DS-01…DS-26**, in the same order as
the original list. Each entry includes its short name.

For tools to catalog, record and replay these cases, see the
[debug tools TODO](DEBUG_TOOLS_TODO.md): **DBG** IDs identify tools; **DS** IDs
continue to identify scenes and their evidence.

Updated inventory: **18 validated cases, 6 partial, 2 uncovered**. The resumed
work adds **DS-04/05/06/09/10/11/13/14/23/24/25** to the earlier
DS-01/02/16/18/19/22. DS-26 now has a partial postgame point. The new user EUR
file completes **DS-20** on October 26. The initial inventory remains preserved
in its record: 3 validated, 10 partial, 13 uncovered. A check applies only to
the stated scenario and evidence. Completed checks are reused; builds,
benchmarks and complete comparisons were not repeated to reconstruct context.
New inspections: Windows/Wine, keyboard, interpreter and HLE BIOS, on isolated
copies.

- **Validated**: requested event or path observed with retained evidence.
- **Partial**: candidate snapshot and some checks present; explicit criteria remain.
- **Uncovered**: no targeted point identified in the sample. This does not mean
  it is impossible to reach through play from the available saves.


## Raccolta numerata locale

La nuova raccolta **`debug_saves/ready/`** contiene `README.local.md` e
`catalog.json`, cartelle regionali **USA/** e **EUR/**, i 27 salvataggi standard
estratti, la precedente base EUR e **32 checkpoint numerati per casistica**
(14 EUR, 18 USA), tre nuovi standard GEN-30/31/33 e la cartuccia
vuota NEW-32 per le scene del nome. L'intera `debug_saves/` è esclusa da Git con
`/debug_saves/`; nessun file della cartella è tracciato. Nessun push eseguito.

Le due numerazioni sono indipendenti:

- **IN-04 → Game saves, slot 4**, file `ffta-slot-04.sav`: identifica
  l'originale importato. USA `ffta.sav` è IN-01; EUR `ffta.sav` conserva F-01.
  I nuovi file EUR sono negli slot 7, 8, 13 e 19; IN-19 è ora verificato su EUR.
- **DS-18 → Save states, slot 18**, file `slot-18.ffstate`: identifica la
  casistica negozio. Il catalogo specifica regione, fonte e ambito del punto.
  Una nuova validazione USA non promuove automaticamente il vecchio punto EUR.

Inoltre `loaded/` conserva **31 checkpoint di caricamento** (inclusi i nuovi
File No.2 di IN-03/IN-05, IN-19 su EUR, GEN-30/31 e Resume Battle GEN-33), mentre `USA/variants/` ha
**25 punti**, compresi prima del KO/dopo la rianimazione, Help leggi, i punti
precedenti sostituiti e lo schieramento di Decision Time.
Le **14 varianti negli slot 51…64** sono già selezionabili dal menu USA:
51 prima KO, 52 dopo rianimazione, 53 prima tastiera, 54 conferma nome,
55 tre leggi Prison Break, 56 schieramento Decision Time, 57 dialogo breve,
58 due missioni attive, 59 White Magic assegnata, 60 bersaglio Firaga,
61 prima Auto-Life, 62 ricompense, 63 inventario pieno, 64 requisito mancante.
Inoltre **EUR slot 65** è la variante DS-20 già dopo la fuga, prima del
rientro fallito. `EUR/variants/` ne conserva la copia descrittiva; i 25 punti
precedenti sotto `USA/variants/` restano invariati. Gli altri file vanno
importati in uno slot libero.
Le copie standard sono esatte; gli snapshot derivano da input ordinari,
senza modificare RAM, flag o inventario. Le anteprime aggiunte ai checkpoint
del replay non alterano il payload core. Gli originali restano invariati.

La cartella ignorata **`saves/debug-cases/`** contiene una copia di `ffta.sav`,
`catalog.json`, `README.local.md` e **13 slot** sotto
`SaveStates/<SHA-256-ROM-EUR>/`. Il numero dello slot corrisponde al caso:
DS-01 → `slot-1.ffstate`, DS-19 → `slot-19.ffstate`, ecc.
I casi non coperti restano vuoti; gli slot parziali sono candidati, non spunte.
Sono copie esatte di **sei snapshot distinti**: lo stesso punto può coprire più
casi. Il nome descrittivo e lo stato sono nel catalogo; il menu conserva le
etichette generiche Slot N. Nessun originale è stato rinominato o sovrascritto.

Per usare gli slot oltre 9 abilitare **Debug > Extended saves**. La cartella degli
slot è affiancata al `.sav` selezionato: usare la copia della raccolta, oppure
importare soltanto i file desiderati in posizioni libere della propria cartella.
Preservare prima eventuali slot personali. I dati di gioco restano fuori da Git
e dai pacchetti. Vedi [Extended saves](../extended_saves.md).

## Fonti disponibili

- **F-01 — Progresso cartuccia**: `saves/saves.zip` → `saves/ffta.sav`, copia di
  lavoro `target/desktop-perf/user-fixture/ffta.sav`. Uguaglianza verificata.
- **F-02 — Battaglia fornita**: slot 1 nello stesso ZIP, copia verificata in
  `target/desktop-perf/user-fixture/slot-1.ffstate`.
- **F-03 — Schieramento italiano**: `saves/slot-3.ffstate`.
- **F-04 — Mappa e menu**: `target/desktop-perf/user-fixture/worldmap.ffstate`.
- **F-05 — Mappa/pub inglese**: slot 5 in
  `target/rom-data-bg-alternate-callers/native-color-options/Data/saves/SaveStates/<EUR>/`.
- **F-06 — Spedizione accettata, giorno 9**: slot 2 in
  `target/rom-data-progression-gates/windows-dispatch/Data/saves/SaveStates/<EUR>/`.
- **F-07 — Prima del rientro, giorno 18**: slot 3 nella cartella di F-06.
- **F-08 — Mission Cleared, giorno 19**: slot 4 nella cartella di F-06.
- **F-09 — Prima del turno nemico**:
  `target/desktop-perf/ai-turn-replay/start.ffstate` e `input.input`.

Sono otto snapshot ordinari e un salvataggio cartuccia selezionati. SHA-256,
checksum delle anteprime e dei payload, versione 1, identità ROM e percorsi esatti sono nel manifest locale
`target/debug-saves-audit/source-manifest.json`. Il vecchio inventario di 49
payload include stati sintetici/esperimenti scartati: non sono nuove casistiche
ordinarie e non vengono promossi automaticamente a prove.

### Nuovi file IN-01…IN-28

Ricevuti 5 ZIP e 23 SPS: **27 SRAM da 65536 byte estratte**, **25 payload
distinti** avviati e caricati sulla mappa con Windows/Wine, interprete e BIOS
HLE. **IN-18**, `final-fantasy-tactics-advance.3442.sps`, ha un contenitore
BGBA non supportato: nessuna estrazione ipotizzata.

IN-06/IN-11 sono duplicati USA. Anche IN-01/IN-19 hanno SRAM identica, ma
IN-19 ha intestazione EUR. Dopo la prima ricognizione è stato eseguito un
nuovo avvio/caricamento **con EUR**, arrivando sulla mappa: entrambe le regioni
sono ora provate direttamente, con gli stessi byte SRAM.
I quattro SPS IN-14/21/22/28 hanno checksum del contenitore non corrispondente;
le copie SRAM inalterate caricano correttamente. Nessun checksum riparato.
I cinque ZIP privi di intestazione regionale sono stati provati con USA.
Il parser SPS segue il framing e la variante storica signed-char del checksum;
il riferimento consultato è il [codice SharkPort di mGBA](https://github.com/mgba-emu/mgba/blob/master/src/gba/sharkport.c).

Fonti utili individuate:

- **IN-04** — `fftafinal1.zip`, USA: clan livello 42, 206 missioni;
  squadra e inventario avanzati, 62/64 oggetti missione. Fonte dei nuovi
  checkpoint DS-04/12/16/17/18/21/24/25 e della battaglia Swimming Meet.
- **IN-01** — `Final Fantasy Tactics Advance.zip`, provato USA:
  300 missioni, fonte della banca popolata DS-22.
- **IN-08** — `final-fantasy-tactics-advance.21006.sps`, EUR:
  File No.1 mostra 300 missioni, clan livello 86. Caricato sulla mappa;
  il tentativo di raggiungere la banca incontra un clan e non la verifica.
- **IN-05** — `fftafinal2.zip`, USA: entrambi i file mostrano 300 missioni.
  File No.2 è ora caricato: Decision Time è attiva a Sprohm, dialogo e
  battaglia postgame sono raggiunti. Vittoria/finale restano da verificare.

Tutti i file importati conservano entrambi gli eventuali salvataggi interni:
nella prima ricognizione fu giocato soltanto File No.1. Nella ripresa sono
stati caricati anche **File No.2 di IN-03 e IN-05**; gli altri restano non giocati.
IN-03 File No.2 mostra effettivamente **300 missioni**, non un punto a 299.
La sola schermata di scelta non dimostra caricamento o contenuto effettivo. Inventario completo dei
nomi, SHA-256, duplicati, checksum, regione e risultati nel catalogo locale;
prove originali sotto `target/debug-saves-expansion/`.

### Nuovo salvataggio standard GEN-30

**USA, Game saves slot 30**, `ffta-slot-30.sav`: deriva da **IN-05 File No.2**.
Il normale comando System → Save del gioco ha salvato il progresso nel
**File No.1** di una copia isolata, a Sprohm prima di Decision Time. Il frontend
Windows già disponibile ha caricato il checkpoint dopo Save e ha scritto la
SRAM uscendo normalmente (**exit0**). Nuovo avvio, caricamento di File No.1 e
disponibilità di Decision Time a Sprohm verificati. Il File No.2 rimane presente;
nessun originale o byte di progressione è stato modificato fuori dal gioco.
La numerazione GEN-30 distingue questa derivazione dagli originali IN-01…28.

**GEN-31, USA Game saves slot 31**, File No.1: nuovo Save a Sprohm con
**Decision Time e Prison Break entrambe attive** e **White Magic** secondaria
assegnata a Marche. Esportazione con normale chiusura exit0; nuovo avvio,
caricamento e persistenza di abilità e due missioni verificati.

**GEN-33, USA Game saves slot 33**: creato tramite **System → Save Now**
durante Prison Break, prima di agire. È un salvataggio sospeso: scegliere
**Saved Game → Resume Battle**, confermare il riepilogo con A, proseguire
l'avviso e scegliere **Yes** alla conferma finale (inizialmente su No).
Load/File No.1 carica invece il progresso sulla mappa. Il nuovo avvio e
caricamento effettivo della battaglia sono verificati; il gioco avvisa che la
ripresa cancella il punto sospeso. I master GEN-30/31/33 restano separati in
`debug_saves/generated/`, oltre alle copie di lavoro numerate.

**NEW-32, USA Game saves slot 32**: memoria cartuccia vuota di fabbrica,
non un salvataggio di progresso. Selezionarla prima di caricare DS-23 o
varianti 53/54. Gli snapshot ripristinano anche SRAM, che la chiusura può
scrivere nello slot standard attivo; usare le copie di lavoro della raccolta.
Un master NEW-32 resta nella cartella `generated/`.


## Casistiche


### DS-01 — Crescita clan

- [x] **VALIDATA** — `crescita_clan`.

Richiesta: Missione che aumenta un’abilità/statistica del clan: prima dell’ultimo colpo o della conferma del risultato. È il passaggio più utile da verificare adesso.

Fonte: F-07; riscontro F-08. Raccolta: **slot 1**.

Riscontro: Mad Alchemist conclusa con Sigmund: due abilità del clan salgono da 1 a 2 e da 3 a 4. Schermata RESULTS ispezionata; prova esistente di 1000 frame con stato completo, immagini e audio uguali. Riprendere il viaggio al giorno 18, attendere il giorno 19 e confermare i risultati con A.

Ambito restante: Valida per questo incremento; altri parametri e soglie restano fuori dal campione.

### DS-02 — Rientro da spedizione riuscita

- [x] **VALIDATA** — `rientro_spedizione`.

Richiesta: Rientro da una missione di spedizione riuscita: sulla mappa, poco prima che trascorrano gli ultimi giorni necessari, con indicazioni su come farla terminare.

Fonte: F-07; riscontro F-08, avvio F-06. Raccolta: **slot 2**.

Riscontro: Messaggi Mission Cleared e Dispatched unit returned osservati al giorno 19. Confermati ritorno di Sigmund e ricompense. Stessa prova completa già conclusa di DS-01, riutilizzata senza ripeterla.

Ambito restante: Valida per Mad Alchemist, durata 10 giorni; il fallimento è DS-20.

### DS-03 — Premio sbloccato dalla progressione del clan

- [ ] **NON COPERTA** — `premio_progressione_clan`.

Richiesta: Sblocco di un premio legato alla crescita del clan: prima del risultato che lo assegna, se riesci a individuare un caso.

Fonte: Nessun checkpoint ordinario individuato.

Riscontro: Il risultato di Mad Alchemist assegna ricompense della missione. Non dimostra un premio aggiuntivo sbloccato da una soglia del clan: i relativi campi return/output hanno zero uso ordinario nella prova disponibile.

Nuova ricerca: IN-06 File No.1 mostra una voce di abilità clan a **28**, a fronte del metadato autore “Diplomacy at level 30”. Nessuna etichetta/prize in gioco confermata. Girl In Love è una possibile spedizione ma richiede White Thread assente: il pub rifiuta l'accettazione. Non è un nuovo premio né un fallimento di spedizione.

Ambito restante: Serve un punto prima dello sblocco effettivo; i test isolati sulle tabelle e il metadato autore non chiudono questa casistica.

### DS-04 — Schieramento della squadra

- [x] **VALIDATA** — `schieramento_squadra`.

Richiesta: Schieramento prima della battaglia: con una squadra abbastanza varia da poter cambiare personaggi e posizioni.

Fonte: IN-04 USA, slot 4 alla conferma Begin battle; F-03 e vecchi punti EUR conservati.

Riscontro: Due membri schierati (2/4), Montblanc riposizionato su un’altra casella, rimozione/rientro già visibili nei passaggi conservati e nuova conferma Start → Yes che avvia effettivamente la battaglia. Replay fino al turno di Marche e poi Montblanc. Prove: `deploy-multi/`, `resume-deploy-two/`, `resume-deploy-start/` e `resume-montblanc-turn/`.

Ambito restante: Valida questa sequenza di due membri, posizionamento e avvio; altre formazioni restano fuori campione.

### DS-05 — Più leggi attive e relativo aiuto

- [x] **VALIDATA** — `leggi_multiple`.

Richiesta: Battaglia con più leggi attive: da cui aprire sia l’elenco sia l’aiuto delle leggi.

Fonte: IN-04 USA, slot 5 con elenco leggi di Swimming Meet.

Riscontro: System → Law Cards mostra due divieti contemporanei: **Law Cards** e **Target All**, con **Target Area** consigliato. Down seleziona Target All; Select apre Help e A passa alla seconda pagina, che descrive le A-abilities sull’intero campo. Prove `resume-lawcards/`, `resume-law-help/`, `resume-law-help-complete/`. Non è il solo layout con righe vuote.

Ambito restante: Valida questa coppia e il relativo aiuto; altre combinazioni non sono tutte censite.

### DS-06 — Infrazione e intervento del giudice

- [x] **VALIDATA** — `infrazione_legge`.

Richiesta: Infrazione di una legge: immediatamente prima dell’azione, per vedere intervento del giudice e penalità.

Fonte: IN-04 USA, slot 6 prima della conferma di Totema/Mateus.

Riscontro: Confermare con A e attendere: effetto di Mateus sull’intero campo, infrazione di **Target All**, cartellino rosso e messaggio che manda Marche in prigione. Seguono assenza dell’unico membro schierato e Game Over. Prove `resume-cast-totema/`, `resume-totema-end/`, `resume-prison-end/`.

Ambito restante: Validata questa infrazione con espulsione; non tutte le penalità o il cartellino giallo.

### DS-07 — Battaglia affollata e unità differenti

- [ ] **PARZIALE** — `battaglia_affollata`.

Richiesta: Battaglia affollata: molte unità, razze e mostri diversi contemporaneamente visibili.

Fonte: F-02; replay derivato F-09. Raccolta: **slot 7**.

Riscontro: Battaglia ordinaria e pubblicazione di più corpi già osservate. Esistono prove su sprite, HUD e turno nemico.

Ambito restante: Restano una scena sufficientemente affollata, varietà di razze/mostri e osservazione contemporanea dei gruppi richiesti. Non equivalgono al solo conteggio di sprite pubblicati.

### DS-08 — Turno nemico lento o problematico

- [ ] **PARZIALE** — `turno_nemico_lento`.

Richiesta: Turno nemico particolarmente lungo o problematico: prima di passare il turno, indicando come riprodurlo.

Fonte: F-09 e target/desktop-perf/ai-turn-replay/input.input. Raccolta: **slot 8**.

Riscontro: Replay ordinario di 3530 frame: attivazione e movimento di un nemico, poi ritorno al giocatore, già verificati.

Ambito restante: Il rallentamento originale non è riprodotto. Serve identificarne un caso concreto; non sono stati avviati nuovi benchmark.

### DS-09 — Magie, evocazioni ed effetti ad area

- [x] **VALIDATA** — `magie_effetti_area`.

Richiesta: Magia o abilità con effetti vistosi: prima della conferma del bersaglio. Sono utili esempi diversi, soprattutto effetti ad area ed evocazioni.

Fonte: IN-04 USA, slot 9 prima di Do it di Firaga; slot 6 offre anche Mateus.

Riscontro: Selezionati magia, area e bersaglio, confermato l’effetto di **Firaga** con variazione di HP (72 nel primo esempio su sé stessi). Separatamente eseguita l’evocazione **Mateus**, con animazione sul campo e danni ai nemici. Prove `resume-firaga/`, `resume-firaga-cast/`, `resume-cast-totema/`. Le prove sono su copie e includono deliberatamente fuoco amico per preparare DS-11.

Ambito restante: Valida questi due effetti e le rispettive conferme; altre magie/evocazioni restano fuori campione.

### DS-10 — Alterazioni di stato

- [x] **VALIDATA** — `alterazioni_stato`.

Richiesta: Unità con alterazioni di stato: icone visibili e, possibilmente, effetti che modificano movimento, azioni o ordine dei turni.

Fonte: IN-05 USA File No.2, slot 10 sul campo; variante slot 61 prima di confermare Auto-Life.

Riscontro: Marche mostra icone alternate sopra l'unità, visibili anche in Status. White Magic → Auto-Life offre 100% sul bersaglio; conferma, animazione e MP **85 → 69** osservati. Prove `resume-autolife-cast/`, `resume-autolife-status/`, `resume-status-pages/`. La presenza delle icone è verificata; non si attribuisce ogni icona al nuovo lancio, dato che ne compaiono già prima.

Ambito restante: Non verificati il KO che attiva la rianimazione automatica, l'identità di ciascuna icona o ulteriori alterazioni che cambiano movimento/azioni/ordine. Questi effetti erano aggiuntivi, non necessari alla scena con icone richiesta.

### DS-11 — KO e rianimazione

- [x] **VALIDATA** — `ko_rianimazione`.

Richiesta: KO e rianimazione: prima del colpo che manda KO oppure prima della conferma della rianimazione.

Fonte: IN-04 USA, slot 11 prima di confermare Phoenix Down; variante `DS-11-before-KO.ffstate`.

Riscontro: La sequenza ordinaria Firaga/Return Magic porta Marche a **HP0**, verificati nella selezione dell’oggetto e nell’assenza dall’ordine dei turni. Montblanc si avvicina con Move; seleziona Phoenix Down e Marche, con anteprima **Revive KO**, 100%, +221 HP. Conferma ed effetto osservati: Marche torna in piedi. Conservati prima del KO, prima di Do it e dopo la rianimazione. Prove `resume-third-firaga/`, `resume-ko-observe/`, `resume-phoenix-adjacent/`, `resume-phoenix-cast/`.

Ambito restante: Valida questo KO e questa rianimazione; non sono stati modificati HP/status nei file o nella RAM. I primi tentativi fuori portata non sono prove di rianimazione fallita.

### DS-12 — Terreno, dislivelli e ostacoli

- [ ] **PARZIALE** — `terreno_complesso`.

Richiesta: Terreno complesso: dislivelli, acqua, ostacoli e personaggi parzialmente coperti dallo scenario.

Fonte: F-03; altra mappa F-02; IN-04 USA Swimming Meet. Raccolta: **slot 12**.

Riscontro: La scena di schieramento mostra dislivelli marcati, pareti e vegetazione; caricamento e rappresentazione ispezionati.

Estensione IN-04: acqua, piattaforme, pareti e palme visibili nella nuova
battaglia; checkpoint anche al primo turno giocatore.

Ambito restante: Restano movimento tra quote e occlusioni durante il percorso.
Le caselle blu di schieramento della vecchia scena non erano prova di acqua.

### DS-13 — Vittoria in battaglia e ricompense

- [x] **VALIDATA** — `vittoria_ricompense`.

Richiesta: Fine battaglia con ricompense: prima dell'ultimo colpo, possibilmente con aumento di livello o apprendimento di un'abilità.

Fonte: IN-05 USA File No.2, slot 13 prima di Do it di Totema/Mateus in Prison Break; variante slot 62 alla schermata RESULTS. GEN-33 riparte dalla battaglia prima dell'azione.

Riscontro: Mateus elimina le guardie rimanenti; seguono dialogo di Julian, **MISSION CLEARED** e risultati di **Prison Break**: **7000 Gil, 40 AP, Helje Key, Perseus Bow, Chivalry**, poi **20 punti clan**. Ultimo attacco e schermate osservati nei replay `resume-prison-mateus/`, `resume-prison-victory/`. Non sono le sole ricompense di una spedizione.

Ambito restante: Nessun aumento di livello/apprendimento di abilità attribuito. Segue **Too many items**, con scelta dell'oggetto da scartare (variante slot 63): la risoluzione di quel menu e la conclusione completa del rientro restano da fare.

### DS-14 — Sconfitta o fallimento dell’obiettivo

- [x] **VALIDATA** — `sconfitta_obiettivo`.

Richiesta: Sconfitta o fallimento dell’obiettivo: poco prima dell’evento conclusivo.

Fonte: IN-04 USA, slot 14 al messaggio di prigione, prima di Game Over.

Riscontro: La violazione Target All con Marche unico schierato conduce alla sua espulsione e a **Game Over**, osservato prima sul campo e poi sullo sfondo nero. Il checkpoint precede l’esito, riprodotto attendendo. Prove `resume-totema-end/`, `resume-prison-end/`.

Ambito restante: Valido questo esito conclusivo; nessun altro fallimento di obiettivo viene dedotto.

### DS-15 — Obiettivo di battaglia alternativo

- [ ] **NON COPERTA** — `obiettivo_alternativo`.

Richiesta: Battaglia con obiettivo diverso dall’eliminazione dei nemici: prima di soddisfarlo o fallirlo, indicando l’obiettivo.

Fonte: Nessun obiettivo alternativo identificato.

Riscontro: Nelle fonti selezionate non è documentata una vittoria/fallimento con requisito diverso dall’eliminazione dei nemici.
Swimming Meet descrive leggi speciali, ma l'obiettivo osservato è DEFEAT ALL
ENEMIES: non valida un obiettivo alternativo.

Ambito restante: Indicare missione e requisito; salvare prima di completarlo o fallirlo.

### DS-16 — Squadra e classi avanzate

- [x] **VALIDATA** — `squadra_classi_avanzate`.

Richiesta: Squadra avanzata: molte classi sbloccate, razze diverse e numerose abilità; da cui esplorare cambio classe, statistiche ed equipaggiamento.

Fonte: **IN-04 USA**, slot 16; i vecchi punti F-04/F-05 EUR restano candidati iniziali.

Riscontro: squadra di razze diverse con unità ai livelli 40–50, anello Change
Jobs con numerose classi disponibili. Cambio effettivo Soldier → Thief
osservato, con nuova classe e statistiche nella scheda. Consultati diversi
set di abilità, tra cui Battle Tech, Black Magic, Chivalry, Fighter Tech e
Steal, oltre alle voci reazione/supporto/combo. Equipaggiamento e inventario
avanzati consultati. Checkpoint all'anello delle classi prima del cambio;
catture `jobs-advanced.png`, `job-changed.png`, `abilities-advanced.png`.

Ambito restante: validata l'esplorazione richiesta di questa squadra; la
descrizione dell'autore «tutte le abilità padroneggiate» non è stata verificata.

### DS-17 — Inventario, categorie e scorrimento

- [ ] **PARZIALE** — `inventario_categorie`.

Richiesta: Inventario ricco: categorie diverse, oggetti con descrizioni lunghe e liste che richiedono scorrimento.

Fonte: **IN-04 USA**, slot 17; precedenti F-04/F-05 EUR conservati.

Riscontro: Prove esistenti sull’elenco copricapi e relative proprietà. Nuova GUI: Party → unità → Equip → elenco oggetti, con cinque copricapi visibili e indicatore di scorrimento. Seconda sessione breve ha ricontrollato Party, senza raggiungere l’inventario generale.

Estensione IN-04: raggiunto l'inventario generale con sei icone di categoria;
ispezionati copricapi, armature e consumabili. Scorrimento effettivo fino a
Barrette/Bangaa Helm/Acacia Hat/Parade Helm. **Select** apre Help di Bronze
Helm, con descrizione su due righe; R mostra informazioni/abilità e un'altra
vista mostra la compatibilità dell'equipaggiamento. Le catture chiamate
weapon-long-help mostrano quest'ultima, non una descrizione lunga.

Ambito restante: descrizioni lunghe e verifica di tutte le categorie. La prima
sessione SIGTERM143 e la nuova sessione GUI interrotta restano prive di prova
di chiusura normale; le osservazioni conservate non chiudono l'intero caso.

### DS-18 — Negozio avanzato e transazioni

- [x] **VALIDATA** — `negozio_avanzato`.

Richiesta: Negozio avanzato: molte categorie e oggetti disponibili, con denaro sufficiente per provare acquisto, vendita e conferme.

Fonte: **IN-04 USA**, slot 18 all'ingresso Shop di Sprohm; vecchi F-05/F-04 EUR conservati.

Riscontro: assortimento/categorie, denaro abbondante, acquisto di una Potion
per **42 gil** confermato (99399199 → 99399157), annullamento esplicito di
un secondo acquisto senza variazione di fondi, vendita di un Bronze Helm
per **250 gil** (99399157 → 99399407). Conferme e risposta del negoziante
ispezionate. Riscontri `sell-quantity.png` (risposta dopo Buy),
`buy-explicit-cancel.png`, `sell-final-confirmation.png`, `sell-committed.png`.

Ambito restante: valida per queste transazioni; non per ogni oggetto, limite di quantità o disponibilità.

### DS-19 — Pub e accettazione di una missione

- [x] **VALIDATA** — `pub_accettazione_missione`.

Richiesta: Pub con molte missioni disponibili: dettagli, requisiti, selezione del partecipante e conferma della spedizione accessibili.

Fonte: F-05; avvio accettato F-06. Raccolta: **slot 19**.

Riscontro: Navigazione tra missioni, pagine dettagli/requisiti, Mad Alchemist, scelta di Sigmund e conferma della spedizione già osservate nella GUI ordinaria. Costo di 600 gil applicato. Le catture e il risultato terminale sono stati ricontrollati.

Ambito restante: Valida per questo percorso del pub; non per ogni missione o requisito di classe/oggetto.

### DS-20 — Spedizione fallita

- [x] **VALIDATA** — `spedizione_fallita`.

Richiesta: Spedizione fallita: poco prima del rientro o della comunicazione del risultato.

Fonte: nuovo `slot-4.ffstate` fornito nella root, ROM **EUR**, formato 1 e BIOS HLE. Rinominato e spostato, con byte e SHA-256 invariati, in **`debug_saves/DS-20-spedizione-fallita-EUR.ffstate`**. Copia pronta in **Save states slot 20 EUR**.

Riproduzione: la conferma «Vuoi fuggire dalla battaglia?» parte su **No**. Premere **Sinistra → A** per scegliere Sì e attendere la sequenza **SCONFITTA**, la dissolvenza e il ritorno alla mappa. **Polly** comunica «Mi dispiace, davvero. Ho fatto un pasticcio». Confermare con A: compare il rapporto **«Rientrata unità in trasferta»**. Segue anche un rapporto negativo di Sigmund. La generica intestazione «Missione Completata» nel riepilogo non è stata usata come prova di successo: il rientro fallito è verificato dal dialogo di Polly e dalla notifica di trasferta.

Riscontro: caricamento e ricaricamento dei checkpoint nel helper Windows già disponibile; ramo No annullato, ramo Sì seguito fino alla mappa e ai rapporti. Anche la GUI Windows/Wine carica effettivamente lo **slot 20**, mostra il messaggio di Polly e il rapporto, poi termina normalmente **exit0**. Nessun build o modifica a payload, RAM o salvataggi cartuccia originali. Prove in `target/debug-saves-expansion/ds20-user-{yes,after-defeat,frontend}/` e `target/debug-saves-ds20/`.

Variante **slot 65 EUR**: già dopo la fuga/sconfitta, prima del passaggio alla mappa e del messaggio di Polly; basta attendere. Master locale `EUR/variants/DS-20-prima-del-rientro.ffstate`. L'originale utente resta conservato integralmente.

Ambito restante: validato questo rientro fallito; altri partecipanti/missioni e tutte le possibili cause di fallimento restano fuori campione. Il solo banner SCONFITTA non sarebbe bastato a chiudere DS-20.

### DS-21 — Oggetti missione e utilizzo

- [ ] **PARZIALE** — `oggetti_missione`.

Richiesta: Oggetti missione: elenco popolato e una missione che ne richiede la selezione o l’utilizzo.

Fonte: F-04 EUR; **IN-04 USA**, slot 21 prima della scelta dell'oggetto da portare.

Riscontro: Clan → Mission Item mostra Chocobo Egg, elenco 1/64; prova esistente di 520 frame con uso positivo dei dati di visualizzazione.

Estensione IN-04: elenco **62/64**, con Gold Vessel, Clock Gear, medaglie e
altri oggetti. Nel pub selezionato ed equipaggiato **Gold Vessel** per
Swimming Meet, quindi accettata la missione e addebitati **850 gil**.
Lo slot 21 precede la scelta; la variante mission-items conserva l'elenco.

Ambito restante: l'equipaggiamento scelto non prova un requisito obbligatorio
o il consumo dell'oggetto. Ulteriore prova IN-06: Girl In Love mostra
**White Thread / used** nei requisiti; l'accettazione viene negata con
“This mission requires … White Thread”. Variante slot 64. È un requisito
obbligatorio osservato in negativo; il consumo riuscito resta da provare.
Per questo il caso resta parziale.

### DS-22 — Banca mostri popolata e cattura

- [x] **VALIDATA** — `banca_mostri_popolata`.

Richiesta: Banca mostri popolata: con più mostri consultabili; utile anche un punto da cui provare una cattura.

Fonte: **IN-01 provato USA**, slot 22 nella banca di Cyril.

Riscontro: consultati **Red Cap Lv23, Jelly Lv24 e Ice Flan Lv25**, con
ritratti; aperta anche la scheda di Ice Flan (HP/MP, movimento e statistiche).
Non è il solo catalogo delle specie: le tre voci hanno mostri presenti con
livelli effettivi, mentre la voce ???? mostra Lv--. Replay completato exit0;
riscontro `bank-IN01-owned/contact.png` e checkpoint ricaricato dal helper.

Ambito restante: validata la consultazione della banca popolata; manca il punto aggiuntivo prima di una cattura.

### DS-23 — Reclutamento o inserimento del nome

- [x] **VALIDATA** — `reclutamento_nome`.

Richiesta: Reclutamento o inserimento di un nome: subito prima della scelta o dell'apertura della tastiera.

Fonte: New Game su memoria vuota NEW-32 USA; slot 23 salvato realmente dal frontend alla tastiera, varianti slot 53 prima dell'apertura e 54 alla conferma.

Riscontro: Proseguito normalmente il dialogo nel cortile fino alla tastiera del nome. Inserimento e cancellazione di caratteri, Start → conferma inizialmente No, annullamento con ritorno alla tastiera e Yes osservati. Il dialogo successivo usa il nome scelto. Ripetuta la sequenza essenziale nella GUI Windows/Wine: MarcheA, aggiunta/rimozione di A, annullamento e conferma; screenshot nativo e snapshot slot 23 conservati. Sessione chiusa normalmente exit0, sorgenti invariati. Prove `resume-school-dialogue2/`, `resume-name-edit/`, `resume-name-confirm/`, `resume-name-frontend/`.

Ambito restante: Validata l'alternativa nome; nessun reclutamento attribuito. Usare Game Save32 prima di caricare questi snapshot di nuova partita.

### DS-24 — Mappa avanzata ed eventi

- [x] **VALIDATA** — `mappa_avanzata`.

Richiesta: Mappa avanzata: molte località sbloccate, eventi disponibili e possibilità di entrare in città e battaglie diverse.

Fonte: IN-05 USA File No.2, slot 24 nel menu di Sprohm con due missioni attive; GEN-31 come standard equivalente sulla mappa. Vecchio punto IN-04 presso Ambervale conservato nelle varianti.

Riscontro: Su IN-04 censite **29 destinazioni** e verificati viaggio a città, pub/negozio e ingresso Swimming Meet. Su IN-05 File No.2 raggiunti **Decision Time e Prison Break**, due scene e battaglie diverse da uno stesso progresso avanzato: Decision Time già attiva, Prison Break accettata pagando 500 Gil. GEN-31 conserva entrambe e White Magic dopo nuovo avvio. Prove `resume-IN05-decision-enter/`, `resume-prison-enter/`, `resume-GEN31-quests/`. Il numero 29 appartiene al censimento IN-04 e non viene esteso automaticamente a IN-05.

Ambito restante: Validati questi eventi e accessi, non tutti gli eventi/località. IN-04 non ha missioni attive nel menu Quit Mission; Ambervale fa avanzare il giorno e apre Save, senza finale osservato.

### DS-25 — Scene narrative differenti

- [x] **VALIDATA** — `scene_narrative`.

Richiesta: Scene narrative differenti: dialoghi lunghi, più interlocutori, cambi di sfondo, dissolvenze o animazioni particolari. Bastano alcuni esempi rappresentativi.

Fonte: IN-05 USA File No.2, slot 25 prima del lungo dialogo di Decision Time.

Riscontro: Ingresso da Sprohm, dissolvenza/cambio dalla mappa alla stanza, dialogo prolungato fra **Cid, Marche e giudici**, ritratti e cambio d’inquadratura; conclusione del dialogo fino allo schieramento e avvio della battaglia. Prove `resume-IN05-decision-next/`, `resume-IN05-dialogue/`, `resume-IN05-dialogue2/`, `resume-IN05-battle/`. Il breve dialogo iniziale di Swimming Meet resta come secondo esempio nelle varianti.

Ambito restante: Valide queste sequenze narrative e transizioni; non sono crediti o finale del gioco.

### DS-26 — Boss o finale

- [ ] **PARZIALE** — `boss_finale`.

Richiesta: Boss, battaglia finale o finale del gioco: se hai già progressi vicini a questi punti.

Fonte: IN-05 USA File No.2; slot 26 al turno di Cid in Decision Time, varianti prima dell’ingresso e allo schieramento.

Riscontro: La missione postgame **Decision Time** risulta già accettata. I dettagli indicano Sprohm e il responsabile della corruzione; ingresso, lungo dialogo con Cid e giudici, squadra di tre membri e battaglia effettiva sono raggiunti. È un punto avanzato concreto, oltre alla sola descrizione dell’autore. GEN-30 consente di rientrare normalmente da Sprohm.

Ambito restante: Vittoria della missione, scena conclusiva e finale/crediti del gioco non osservati. La natura di ultimo scontro è ancora un indizio dai metadati, non una prova dell’epilogo. IN-04 presso Ambervale è un tentativo negativo: nessuna missione attiva e soltanto avanzamento giorno/Save, senza evento finale.

## Prove e prossimi controlli

Le prove del rientro/crescita e del percorso al pub sono descritte nel
[record della spedizione ordinaria](../changes/2026-10-23-consume-european-progression-gates-and-receipt-records.json).
La coppia di 1000 frame è conclusa e non è stata ripetuta. Le immagini RESULTS,
Mission Cleared e accettazione sono state ricontrollate. Premi di missione e
premi di progressione del clan rimangono casi separati, DS-02 e DS-03.

Per gli altri sottocasi riutilizzare le prove di [leggi e schieramento](../ffta_laws_layout.md),
[negozio/banca](../ffta_shop_bank_mouse.md), il
[replay nemico](../changes/2026-10-20-record-observed-enemy-activation-replay.json)
e i contratti/scenari già registrati in [dati di gioco](../rom_data_gameplay.md).

Le osservazioni della prima ricognizione e i risultati terminali sono in
`target/debug-saves-audit/{DS-04,DS-17-global}/`, con catture e guardie sugli
originali. La sessione intermedia `DS-17/` è incompleta (SIGTERM143), conserva
solo le osservazioni fino a Equip e non dà una prova di chiusura normale.
Quella ricognizione non creò nuovi snapshot di gioco.

L'espansione sui nuovi file è conservata in **`target/debug-saves-expansion/`**:
`input-manifest.json`, `boot-results.json`, `helper-build.json`,
`package-result.json`, directory dei replay e `USA-final/` con input/catture.
Un helper Windows privato, collegato al core già costruito, ha eseguito input
ordinari ed esportato checkpoint, verificandone il ricaricamento in una nuova
Machine. Non è stata ricostruita l'applicazione. I tentativi iniziali di link
falliti e un percorso errato del helper sono conservati come errori degli
strumenti, senza attribuirli al gioco.

I 25 boot/caricamenti e i replay citati sono terminali exit0. La sessione GUI
**è stata interrotta**: il log termina con X connection broken e manca
`final.json`. Le catture intermedie documentano le azioni osservate; nessuna
chiusura normale, equivalenza completa, qualità audio o prestazione è dedotta
da quella sessione. Nessuna nuova prova su Windows nativo, controller fisico,
Linux o Android. Le verifiche finali conservano hash degli originali e fonti,
checksum anteprime/payload, ROM/versione e assenza di file tracciati in Git.

Ricognizione complessiva **parziale**: **18 validati, 6 parziali, 2 non coperti**.
Restano parziali DS-07 folla/varietà, DS-08 riproduzione del turno lento,
DS-12 percorso tra quote/occlusioni, DS-17 descrizioni lunghe/categorie,
DS-21 consumo di un oggetto richiesto e DS-26 conclusione/finale.
Non coperti DS-03 premio di soglia e DS-15 obiettivo alternativo. DS-20 è ora
validato dal nuovo punto EUR e dal rientro fallito osservato di Polly. Il tentativo Girl In Love si ferma al requisito mancante;
Prison Break ha obiettivo DEFEAT ALL ENEMIES e nessun oggetto obbligatorio.
Le anteprime Bind a 0% restano tentativi negativi, distinti dalla scena con
icone DS-10. Non ripetere le validazioni concluse per ricostruire il contesto.

Verificati anche **26 menu Resume Battle** da checkpoint esistenti: nessuno
carica uno scontro con A. Il nuovo **GEN-33**, prodotto dal normale Save Now,
serve da controllo positivo: riepilogo, avviso, conferma Yes e campo al turno
di Marche effettivamente raggiunti con un nuovo avvio. Nessuna SRAM originale
viene riscritta dal replay. Le tre GUI di esportazione standard e la GUI del
nome terminano normalmente exit0; resta separata la prima GUI interrotta.
I master generati, il catalogo e le istruzioni sono in `debug_saves/` ignorata;
i risultati terminali e le catture rimangono sotto `target/debug-saves-expansion/`.
