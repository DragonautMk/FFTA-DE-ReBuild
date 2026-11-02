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


<a id="raccolta-numerata-locale"></a>

## Local numbered collection

The new **`debug_saves/ready/`** collection contains `README.local.md` and
`catalog.json`, regional **USA/** and **EUR/** folders, the 27 extracted standard
saves, the earlier EUR base and **32 numbered checkpoints by case** (14 EUR,
18 USA), three new standard saves GEN-30/31/33 and the empty NEW-32 cartridge
for naming scenes. All of `debug_saves/` is excluded from Git via
`/debug_saves/`; no file in the folder is tracked. No push was performed.

The two numbering schemes are independent:

- **IN-04 → Game saves, slot 4**, file `ffta-slot-04.sav`: identifies the
  imported original. USA `ffta.sav` is IN-01; EUR `ffta.sav` retains F-01.
  The new EUR files are in slots 7, 8, 13 and 19; IN-19 is now verified on EUR.
- **DS-18 → Save states, slot 18**, file `slot-18.ffstate`: identifies the shop
  case. The catalog specifies the point's region, source and scope. A new USA
  validation does not automatically promote the old EUR point.

In addition, `loaded/` retains **31 load checkpoints** (including the new
IN-03/IN-05 File No.2, IN-19 on EUR, GEN-30/31 and Resume Battle GEN-33), while
`USA/variants/` has **25 points**, including before KO/after revival, law Help,
the replaced earlier points and Decision Time deployment.
The **14 variants in slots 51…64** are already selectable from the USA menu:
51 before KO, 52 after revival, 53 before keyboard, 54 name confirmation,
55 three Prison Break laws, 56 Decision Time deployment, 57 short dialogue,
58 two active missions, 59 White Magic assigned, 60 Firaga target,
61 before Auto-Life, 62 rewards, 63 full inventory, 64 missing requirement.
Also, **EUR slot 65** is the DS-20 variant after escape, before the failed
return. `EUR/variants/` retains its descriptive copy; the 25 earlier points
under `USA/variants/` remain unchanged. Other files must be imported into a free
slot. Standard copies are exact; snapshots come from ordinary inputs without
changing RAM, flags or inventory. Previews added to replay checkpoints do not
alter the core payload. Originals remain unchanged.

The ignored **`saves/debug-cases/`** folder contains a copy of `ffta.sav`,
`catalog.json`, `README.local.md` and **13 slots** under
`SaveStates/<SHA-256-ROM-EUR>/`. The slot number matches the case:
DS-01 → `slot-1.ffstate`, DS-19 → `slot-19.ffstate`, etc. Uncovered cases stay
empty; partial slots are candidates, not checked cases. These are exact copies
of **six distinct snapshots**: one point can cover multiple cases. Descriptive
name and status are in the catalog; the menu retains generic Slot N labels. No
original has been renamed or overwritten.

To use slots beyond 9, enable **Debug > Extended saves**. The slots folder sits
beside the selected `.sav`: use the collection copy, or import only desired
files into free positions in your own folder. Preserve any personal slots first.
Game data stay out of Git and packages. See [Extended saves](../extended_saves.md).

<a id="fonti-disponibili"></a>

## Available sources

- **F-01 — Cartridge progress**: `saves/saves.zip` → `saves/ffta.sav`, working
  copy `target/desktop-perf/user-fixture/ffta.sav`. Equality verified.
- **F-02 — Supplied battle**: slot 1 in the same ZIP, verified copy at
  `target/desktop-perf/user-fixture/slot-1.ffstate`.
- **F-03 — Italian deployment**: `saves/slot-3.ffstate`.
- **F-04 — Map and menu**: `target/desktop-perf/user-fixture/worldmap.ffstate`.
- **F-05 — English map/pub**: slot 5 in
  `target/rom-data-bg-alternate-callers/native-color-options/Data/saves/SaveStates/<EUR>/`.
- **F-06 — Accepted dispatch, day 9**: slot 2 in
  `target/rom-data-progression-gates/windows-dispatch/Data/saves/SaveStates/<EUR>/`.
- **F-07 — Before return, day 18**: slot 3 in the F-06 folder.
- **F-08 — Mission Cleared, day 19**: slot 4 in the F-06 folder.
- **F-09 — Before enemy turn**:
  `target/desktop-perf/ai-turn-replay/start.ffstate` and `input.input`.

Eight ordinary snapshots and one cartridge save were selected. SHA-256, preview
and payload checksums, version 1, ROM identity and exact paths are in the local
manifest `target/debug-saves-audit/source-manifest.json`. The old inventory of
49 payloads includes synthetic states/rejected experiments: these are not new
ordinary cases and are not automatically promoted to evidence.

<a id="nuovi-file-in-01in-28"></a>

### New IN-01…IN-28 files

Received 5 ZIP and 23 SPS: **27 extracted 65536-byte SRAMs**, **25 distinct
payloads** booted and loaded onto the map with Windows/Wine, interpreter and
HLE BIOS. **IN-18**, `final-fantasy-tactics-advance.3442.sps`, has an unsupported
BGBA container: no extraction assumed.

IN-06/IN-11 are USA duplicates. IN-01/IN-19 also have identical SRAM, but
IN-19 has a EUR header. After the first inventory, a new boot/load **with EUR**
reached the map: both regions are now directly tested with the same SRAM bytes.
The four SPS files IN-14/21/22/28 have mismatched container checksums; unaltered
SRAM copies load correctly. No checksum repaired. The five ZIP files without a
region header were tested with USA. The SPS parser follows framing and the
historical signed-char checksum variant; the reference consulted is
[mGBA's SharkPort code](https://github.com/mgba-emu/mgba/blob/master/src/gba/sharkport.c).

Useful sources identified:

- **IN-04** — `fftafinal1.zip`, USA: clan level 42, 206 missions; advanced
  party and inventory, 62/64 mission items. Source of new DS-04/12/16/17/18/
  21/24/25 checkpoints and the Swimming Meet battle.
- **IN-01** — `Final Fantasy Tactics Advance.zip`, tested USA: 300 missions,
  source of populated DS-22 bank.
- **IN-08** — `final-fantasy-tactics-advance.21006.sps`, EUR: File No.1 shows
  300 missions, clan level 86. Loaded onto the map; the attempt to reach the
  bank encounters a clan and does not verify it.
- **IN-05** — `fftafinal2.zip`, USA: both files show 300 missions. File No.2
  has now been loaded: Decision Time is active at Sprohm, dialogue and postgame
  battle reached. Victory/ending remain unverified.

All imported files retain both internal saves where present: only File No.1 was
played in the first inventory. On resumption **IN-03 and IN-05 File No.2** were
also loaded; the others remain unplayed. IN-03 File No.2 actually shows
**300 missions**, not a 299 point. The selection screen alone does not prove
loading or actual content. Full inventory of names, SHA-256, duplicates,
checksums, region and results is in the local catalog; original evidence under
`target/debug-saves-expansion/`.

<a id="nuovo-salvataggio-standard-gen-30"></a>

### New GEN-30 standard save

**USA, Game saves slot 30**, `ffta-slot-30.sav`: derived from **IN-05 File No.2**.
The normal in-game System → Save command saved progress to **File No.1** of an
isolated copy, at Sprohm before Decision Time. The existing Windows frontend
loaded the checkpoint after Save and wrote SRAM on normal exit (**exit0**).
New startup, loading File No.1 and availability of Decision Time at Sprohm
verified. File No.2 remains present; no original or progression byte was changed
outside the game. GEN-30 numbering distinguishes this derivation from originals
IN-01…28.

**GEN-31, USA Game saves slot 31**, File No.1: new Save at Sprohm with both
**Decision Time and Prison Break active** and **White Magic** assigned to Marche
as a secondary ability. Export with normal exit0; new startup, loading and
persistence of ability and both missions verified.

**GEN-33, USA Game saves slot 33**: created with **System → Save Now** during
Prison Break, before acting. It is a suspended save: choose **Saved Game → Resume
Battle**, confirm the summary with A, proceed past the notice and choose **Yes**
at the final confirmation (initially on No). Load/File No.1 instead loads map
progress. New startup and actual battle loading verified; the game warns that
resuming deletes the suspended point. GEN-30/31/33 masters remain separate in
`debug_saves/generated/`, in addition to numbered working copies.

**NEW-32, USA Game saves slot 32**: factory-empty cartridge memory, not a progress
save. Select it before loading DS-23 or variants 53/54. Snapshots also restore
SRAM, which closing can write to the active standard slot; use collection
working copies. A NEW-32 master remains in the `generated/` folder.


<a id="casistiche"></a>

## Cases

<a id="ds-01--crescita-clan"></a>

### DS-01 — Clan growth

- [x] **VALIDATED** — `crescita_clan`.

Request: A mission that increases a clan ability/statistic: before the final blow or result confirmation. This is the most useful transition to check now.

Source: F-07; evidence F-08. Collection: **slot 1**.

Evidence: Mad Alchemist completed with Sigmund: two clan abilities rise from 1 to 2 and 3 to 4. RESULTS screen inspected; existing 1000-frame check with identical full state, images and audio. Resume travel on day 18, wait for day 19 and confirm results with A.

Remaining scope: Valid for this increment; other parameters and thresholds remain outside the sample.

<a id="ds-02--rientro-da-spedizione-riuscita"></a>

### DS-02 — Return from a successful dispatch

- [x] **VALIDATED** — `rientro_spedizione`.

Request: Return from a successful dispatch mission: on the map shortly before the final days elapse, with instructions to finish it.

Source: F-07; evidence F-08, start F-06. Collection: **slot 2**.

Evidence: Mission Cleared and Dispatched unit returned messages observed on day 19. Sigmund's return and rewards confirmed. The same completed full check as DS-01 was reused without repeating it.

Remaining scope: Valid for Mad Alchemist, duration 10 days; failure is DS-20.

<a id="ds-03--premio-sbloccato-dalla-progressione-del-clan"></a>

### DS-03 — Reward unlocked by clan progress

- [ ] **UNCOVERED** — `premio_progressione_clan`.

Request: Unlocking a reward tied to clan growth: before the result that awards it, if a case can be identified.

Source: No ordinary checkpoint identified.

Evidence: The Mad Alchemist result awards mission rewards. It does not demonstrate an additional reward unlocked by a clan threshold: the corresponding return/output fields have zero ordinary use in the available check.

Further search: IN-06 File No.1 shows a clan ability entry at **28**, compared with author metadata “Diplomacy at level 30”. No in-game label/prize confirmed. Girl In Love is a possible dispatch but requires missing White Thread: the pub refuses acceptance. This is neither a new reward nor a failed dispatch.

Remaining scope: A point before the actual unlock is needed; isolated table tests and author metadata do not close this case.

<a id="ds-04--schieramento-della-squadra"></a>

### DS-04 — Party deployment

- [x] **VALIDATED** — `schieramento_squadra`.

Request: Deployment before battle, with a varied enough party to change characters and positions.

Source: IN-04 USA, slot 4 at Begin battle confirmation; F-03 and old EUR points retained.

Evidence: Two members deployed (2/4), Montblanc repositioned to another tile, removal/re-entry already visible in retained steps, and a new Start → Yes confirmation that actually starts battle. Replay through Marche's turn and then Montblanc's. Evidence: `deploy-multi/`, `resume-deploy-two/`, `resume-deploy-start/` and `resume-montblanc-turn/`.

Remaining scope: Valid for this two-member sequence, positioning and start; other formations remain outside the sample.

<a id="ds-05--più-leggi-attive-e-relativo-aiuto"></a>

### DS-05 — Multiple active laws and their help

- [x] **VALIDATED** — `leggi_multiple`.

Request: A battle with multiple active laws from which both the list and law help can be opened.

Source: IN-04 USA, slot 5 with the Swimming Meet law list.

Evidence: System → Law Cards shows two simultaneous prohibitions: **Law Cards** and **Target All**, with **Target Area** recommended. Down selects Target All; Select opens Help and A advances to the second page, which describes A-abilities across the entire field. Evidence `resume-lawcards/`, `resume-law-help/`, `resume-law-help-complete/`. This is not just the layout with empty rows.

Remaining scope: Valid for this pair and its help; other combinations have not all been inventoried.

<a id="ds-06--infrazione-e-intervento-del-giudice"></a>

### DS-06 — Law violation and judge intervention

- [x] **VALIDATED** — `infrazione_legge`.

Request: A law violation immediately before the action, to observe judge intervention and penalty.

Source: IN-04 USA, slot 6 before confirming Totema/Mateus.

Evidence: Confirm with A and wait: Mateus affects the entire field, **Target All** is violated, a red card appears and a message sends Marche to prison. The only deployed member is then absent and Game Over follows. Evidence `resume-cast-totema/`, `resume-totema-end/`, `resume-prison-end/`.

Remaining scope: This violation with expulsion is validated; not every penalty or the yellow card.

<a id="ds-07--battaglia-affollata-e-unità-differenti"></a>

### DS-07 — Crowded battle and different units

- [ ] **PARTIAL** — `battaglia_affollata`.

Request: A crowded battle with many different units, races and monsters visible at the same time.

Source: F-02; derived replay F-09. Collection: **slot 7**.

Evidence: Ordinary battle and publication of multiple bodies already observed. Sprite, HUD and enemy-turn evidence exists.

Remaining scope: A sufficiently crowded scene, variety of races/monsters and simultaneous observation of the requested groups remain. Publishing a sprite count alone is not equivalent.

<a id="ds-08--turno-nemico-lento-o-problematico"></a>

### DS-08 — Slow or problematic enemy turn

- [ ] **PARTIAL** — `turno_nemico_lento`.

Request: A particularly long or problematic enemy turn: before passing the turn, with reproduction instructions.

Source: F-09 and target/desktop-perf/ai-turn-replay/input.input. Collection: **slot 8**.

Evidence: Ordinary 3530-frame replay: activation and movement of an enemy, then return to the player, already verified.

Remaining scope: The original slowdown was not reproduced. A concrete case still needs identification; no new benchmarks were started.

<a id="ds-09--magie-evocazioni-ed-effetti-ad-area"></a>

### DS-09 — Magic, summons and area effects

- [x] **VALIDATED** — `magie_effetti_area`.

Request: Magic or an ability with striking effects before target confirmation. Different examples are useful, especially area effects and summons.

Source: IN-04 USA, slot 9 before Firaga Do it; slot 6 also offers Mateus.

Evidence: Selected magic, area and target and confirmed the **Firaga** effect with HP change (72 in the first self-target example). Separately performed the **Mateus** summon, with field animation and damage to enemies. Evidence `resume-firaga/`, `resume-firaga-cast/`, `resume-cast-totema/`. Checks use copies and deliberately include friendly fire to prepare DS-11.

Remaining scope: Valid for these two effects and their confirmations; other spells/summons remain outside the sample.

<a id="ds-10--alterazioni-di-stato"></a>

### DS-10 — Status effects

- [x] **VALIDATED** — `alterazioni_stato`.

Request: Units with status effects: visible icons and, if possible, effects changing movement, actions or turn order.

Source: IN-05 USA File No.2, slot 10 on the field; slot 61 variant before confirming Auto-Life.

Evidence: Marche shows alternating icons above the unit, also visible in Status. White Magic → Auto-Life offers 100% on the target; confirmation, animation and MP **85 → 69** observed. Evidence `resume-autolife-cast/`, `resume-autolife-status/`, `resume-status-pages/`. Icons are verified; not every icon is attributed to the new cast, since some already appear beforehand.

Remaining scope: The KO that triggers automatic revival, each icon's identity and further effects changing movement/actions/order were not verified. These effects were additional, not necessary to the requested scene with icons.

<a id="ds-11--ko-e-rianimazione"></a>

### DS-11 — KO and revival

- [x] **VALIDATED** — `ko_rianimazione`.

Request: KO and revival, before the blow causing KO or before revival confirmation.

Source: IN-04 USA, slot 11 before confirming Phoenix Down; variant `DS-11-before-KO.ffstate`.

Evidence: The ordinary Firaga/Return Magic sequence brings Marche to **HP0**, verified in item selection and absence from turn order. Montblanc approaches with Move; selects Phoenix Down and Marche, with preview **Revive KO**, 100%, +221 HP. Confirmation and effect observed: Marche stands again. Points before KO, before Do it and after revival retained. Evidence `resume-third-firaga/`, `resume-ko-observe/`, `resume-phoenix-adjacent/`, `resume-phoenix-cast/`.

Remaining scope: Valid for this KO and revival; HP/status were not changed in files or RAM. Initial out-of-range attempts are not evidence of failed revival.

<a id="ds-12--terreno-dislivelli-e-ostacoli"></a>

### DS-12 — Terrain, elevation and obstacles

- [ ] **PARTIAL** — `terreno_complesso`.

Request: Complex terrain: elevation changes, water, obstacles and characters partly hidden by scenery.

Source: F-03; another map F-02; IN-04 USA Swimming Meet. Collection: **slot 12**.

Evidence: The deployment scene shows marked elevation changes, walls and vegetation; loading and presentation inspected.

IN-04 extension: water, platforms, walls and palms visible in the new battle; checkpoint also at the first player turn.

Remaining scope: Movement between elevations and occlusion during the path remain. Blue deployment tiles in the old scene were not evidence of water.

<a id="ds-13--vittoria-in-battaglia-e-ricompense"></a>

### DS-13 — Battle victory and rewards

- [x] **VALIDATED** — `vittoria_ricompense`.

Request: End of battle with rewards before the final blow, preferably with a level increase or ability learning.

Source: IN-05 USA File No.2, slot 13 before Totema/Mateus Do it in Prison Break; slot 62 variant at RESULTS screen. GEN-33 restarts from battle before the action.

Evidence: Mateus eliminates the remaining guards; Julian's dialogue, **MISSION CLEARED** and **Prison Break** results follow: **7000 Gil, 40 AP, Helje Key, Perseus Bow, Chivalry**, then **20 clan points**. Final attack and screens observed in `resume-prison-mateus/`, `resume-prison-victory/` replays. These are not merely dispatch rewards.

Remaining scope: No level increase/ability learning attributed. **Too many items** follows, with a choice of item to discard (slot 63 variant): resolving that menu and completing the return remain to be done.


<a id="ds-14--sconfitta-o-fallimento-dellobiettivo"></a>

### DS-14 — Defeat or objective failure

- [x] **VALIDATED** — `sconfitta_obiettivo`.

Request: Defeat or objective failure shortly before the concluding event.

Source: IN-04 USA, slot 14 at the prison message, before Game Over.

Evidence: The Target All violation with Marche as the only deployed unit leads to his expulsion and **Game Over**, observed first on the field and then on the black background. The checkpoint precedes the outcome, reproduced by waiting. Evidence `resume-totema-end/`, `resume-prison-end/`.

Remaining scope: Valid for this concluding outcome; no other objective failure is inferred.

<a id="ds-15--obiettivo-di-battaglia-alternativo"></a>

### DS-15 — Alternative battle objective

- [ ] **UNCOVERED** — `obiettivo_alternativo`.

Request: Battle with an objective other than eliminating enemies, before meeting or failing it, with the objective identified.

Source: No alternative objective identified.

Evidence: The selected sources do not document a victory/failure with a requirement other than eliminating enemies. Swimming Meet describes special laws, but the observed objective is DEFEAT ALL ENEMIES: this does not validate an alternative objective.

Remaining scope: Identify mission and requirement; save before completing or failing it.

<a id="ds-16--squadra-e-classi-avanzate"></a>

### DS-16 — Advanced party and jobs

- [x] **VALIDATED** — `squadra_classi_avanzate`.

Request: An advanced party with many unlocked jobs, different races and numerous abilities, from which to explore job changes, statistics and equipment.

Source: **IN-04 USA**, slot 16; old F-04/F-05 EUR points remain initial candidates.

Evidence: Party of different races with units at levels 40–50, Change Jobs ring with numerous available jobs. Actual Soldier → Thief change observed, with new job and statistics on the sheet. Several ability sets consulted, including Battle Tech, Black Magic, Chivalry, Fighter Tech and Steal, as well as reaction/support/combo entries. Advanced equipment and inventory inspected. Checkpoint at the job ring before the change; captures `jobs-advanced.png`, `job-changed.png`, `abilities-advanced.png`.

Remaining scope: Requested exploration of this party validated; the author's description «tutte le abilità padroneggiate» (“all abilities mastered”) was not verified.

<a id="ds-17--inventario-categorie-e-scorrimento"></a>

### DS-17 — Inventory, categories and scrolling

- [ ] **PARTIAL** — `inventario_categorie`.

Request: A rich inventory with different categories, items with long descriptions and lists requiring scrolling.

Source: **IN-04 USA**, slot 17; earlier F-04/F-05 EUR retained.

Evidence: Existing checks of the headgear list and its properties. New GUI: Party → unit → Equip → item list, with five visible headgear items and a scroll indicator. A second short session rechecked Party without reaching the general inventory.

IN-04 extension: reached general inventory with six category icons; headgear, armor and consumables inspected. Actual scrolling to Barrette/Bangaa Helm/Acacia Hat/Parade Helm. **Select** opens Bronze Helm Help with a two-line description; R shows information/abilities and another view shows equipment compatibility. Captures named weapon-long-help show the latter, not a long description.

Remaining scope: Long descriptions and verification of all categories. The first SIGTERM143 session and the new interrupted GUI session still lack evidence of normal closure; retained observations do not close the whole case.

<a id="ds-18--negozio-avanzato-e-transazioni"></a>

### DS-18 — Advanced shop and transactions

- [x] **VALIDATED** — `negozio_avanzato`.

Request: An advanced shop with many categories and available items, and enough money to try purchasing, selling and confirmations.

Source: **IN-04 USA**, slot 18 at the Sprohm Shop entrance; old F-05/F-04 EUR retained.

Evidence: Stock/categories, plentiful money, confirmed Potion purchase for **42 gil** (99399199 → 99399157), explicit cancellation of a second purchase without a funds change, sale of a Bronze Helm for **250 gil** (99399157 → 99399407). Confirmations and shopkeeper response inspected. Evidence `sell-quantity.png` (response after Buy), `buy-explicit-cancel.png`, `sell-final-confirmation.png`, `sell-committed.png`.

Remaining scope: Valid for these transactions, not every item, quantity limit or availability.

<a id="ds-19--pub-e-accettazione-di-una-missione"></a>

### DS-19 — Pub and mission acceptance

- [x] **VALIDATED** — `pub_accettazione_missione`.

Request: A pub with many available missions: accessible details, requirements, participant selection and dispatch confirmation.

Source: F-05; accepted start F-06. Collection: **slot 19**.

Evidence: Navigation between missions, details/requirements pages, Mad Alchemist, selection of Sigmund and dispatch confirmation already observed in the ordinary GUI. Cost of 600 gil applied. Captures and terminal result were rechecked.

Remaining scope: Valid for this pub path, not every mission or job/item requirement.

<a id="ds-20--spedizione-fallita"></a>

### DS-20 — Failed dispatch

- [x] **VALIDATED** — `spedizione_fallita`.

Request: Failed dispatch shortly before return or notification of the result.

Source: new `slot-4.ffstate` supplied in the root, **EUR** ROM, format 1 and HLE BIOS. Renamed and moved with unchanged bytes and SHA-256 to **`debug_saves/DS-20-spedizione-fallita-EUR.ffstate`**. Ready copy in **Save states slot 20 EUR**.

Reproduction: the confirmation «Vuoi fuggire dalla battaglia?» (“Do you want to flee the battle?”) starts on **No**. Press **Left → A** to choose **Sì** (Yes) and wait for **SCONFITTA** (defeat), the fade and return to the map. **Polly** says «Mi dispiace, davvero. Ho fatto un pasticcio». Confirm with A: the report **«Rientrata unità in trasferta»** appears. A negative Sigmund report follows too. The generic heading «Missione Completata» in the summary was not used as evidence of success: the failed return is verified by Polly's dialogue and the dispatch notification.

Evidence: loading and reloading checkpoints in the existing Windows helper; No branch cancelled, Sì branch followed through map and reports. The Windows/Wine GUI also actually loads **slot 20**, shows Polly's message and the report, then exits normally **exit0**. No build or change to original payload, RAM or cartridge saves. Evidence in `target/debug-saves-expansion/ds20-user-{yes,after-defeat,frontend}/` and `target/debug-saves-ds20/`.

**EUR slot 65** variant: already after escape/defeat, before transition to the map and Polly's message; just wait. Local master `EUR/variants/DS-20-prima-del-rientro.ffstate`. The user's original remains fully preserved.

Remaining scope: This failed return is validated; other participants/missions and all possible causes of failure remain outside the sample. The SCONFITTA banner alone would not have sufficed to close DS-20.

<a id="ds-21--oggetti-missione-e-utilizzo"></a>

### DS-21 — Mission items and use

- [ ] **PARTIAL** — `oggetti_missione`.

Request: Mission items: a populated list and a mission requiring their selection or use.

Source: F-04 EUR; **IN-04 USA**, slot 21 before choosing the item to take.

Evidence: Clan → Mission Item shows Chocobo Egg, list 1/64; existing 520-frame check with positive use of display data.

IN-04 extension: **62/64** list, with Gold Vessel, Clock Gear, medals and other items. In the pub **Gold Vessel** was selected and equipped for Swimming Meet, then the mission accepted and **850 gil** charged. Slot 21 precedes selection; the mission-items variant retains the list.

Remaining scope: The selected equipment does not prove a mandatory requirement or item consumption. Further IN-06 check: Girl In Love shows **White Thread / used** in requirements; acceptance is refused with “This mission requires … White Thread”. Slot 64 variant. This is a mandatory requirement observed negatively; successful consumption remains to be proven. The case therefore remains partial.

<a id="ds-22--banca-mostri-popolata-e-cattura"></a>

### DS-22 — Populated monster bank and capture

- [x] **VALIDATED** — `banca_mostri_popolata`.

Request: A populated monster bank with multiple monsters to inspect; a point from which to try a capture is also useful.

Source: **IN-01 tested USA**, slot 22 in Cyril's bank.

Evidence: **Red Cap Lv23, Jelly Lv24 and Ice Flan Lv25** inspected with portraits; Ice Flan's sheet also opened (HP/MP, movement and statistics). This is not only the species catalog: the three entries have present monsters with actual levels, while the ???? entry shows Lv--. Replay completed exit0; evidence `bank-IN01-owned/contact.png` and checkpoint reloaded by helper.

Remaining scope: Inspection of the populated bank validated; the additional point before a capture is missing.

<a id="ds-23--reclutamento-o-inserimento-del-nome"></a>

### DS-23 — Recruitment or name entry

- [x] **VALIDATED** — `reclutamento_nome`.

Request: Recruitment or name entry immediately before selection or opening the keyboard.

Source: New Game on empty NEW-32 USA memory; slot 23 actually saved by the frontend at the keyboard, slot 53 variant before opening and 54 at confirmation.

Evidence: Continued the courtyard dialogue normally to the name keyboard. Character entry and deletion, Start → confirmation initially No, cancellation returning to keyboard and Yes observed. Essential sequence repeated in Windows/Wine GUI: MarcheA, addition/removal of A, cancellation and confirmation; native screenshot and slot 23 snapshot retained. Session closed normally exit0, sources unchanged. Evidence `resume-school-dialogue2/`, `resume-name-edit/`, `resume-name-confirm/`, `resume-name-frontend/`.

Remaining scope: The name-entry alternative is validated; no recruitment attributed. Use Game Save32 before loading these new-game snapshots.

<a id="ds-24--mappa-avanzata-ed-eventi"></a>

### DS-24 — Advanced map and events

- [x] **VALIDATED** — `mappa_avanzata`.

Request: An advanced map with many unlocked locations, available events and access to different towns and battles.

Source: IN-05 USA File No.2, slot 24 in the Sprohm menu with two active missions; GEN-31 as equivalent standard save on the map. Old IN-04 point near Ambervale retained among variants.

Evidence: On IN-04, **29 destinations** inventoried and travel to town, pub/shop and Swimming Meet entry verified. On IN-05 File No.2, **Decision Time and Prison Break** reached, two different scenes and battles from the same advanced progress: Decision Time already active, Prison Break accepted for 500 Gil. GEN-31 retains both and White Magic after a new start. Evidence `resume-IN05-decision-enter/`, `resume-prison-enter/`, `resume-GEN31-quests/`. The count 29 belongs to the IN-04 inventory and is not automatically extended to IN-05.

Remaining scope: These events and entries are validated, not every event/location. IN-04 has no active missions in the Quit Mission menu; Ambervale advances the day and opens Save, without an ending observed.

<a id="ds-25--scene-narrative-differenti"></a>

### DS-25 — Different narrative scenes

- [x] **VALIDATED** — `scene_narrative`.

Request: Different narrative scenes with long dialogue, multiple speakers, background changes, fades or special animations. A few representative examples suffice.

Source: IN-05 USA File No.2, slot 25 before the long Decision Time dialogue.

Evidence: Entry from Sprohm, fade/change from map to room, extended dialogue among **Cid, Marche and judges**, portraits and camera change; dialogue completed through deployment and battle start. Evidence `resume-IN05-decision-next/`, `resume-IN05-dialogue/`, `resume-IN05-dialogue2/`, `resume-IN05-battle/`. The short opening Swimming Meet dialogue remains a second example in variants.

Remaining scope: These narrative sequences and transitions are valid; they are not game credits or an ending.

<a id="ds-26--boss-o-finale"></a>

### DS-26 — Boss or ending

- [ ] **PARTIAL** — `boss_finale`.

Request: Boss, final battle or game ending, if progress is already close to these points.

Source: IN-05 USA File No.2; slot 26 at Cid's turn in Decision Time, variants before entry and at deployment.

Evidence: Postgame mission **Decision Time** is already accepted. Details indicate Sprohm and the person responsible for corruption; entry, long dialogue with Cid and judges, three-member party and actual battle reached. This is a concrete advanced point beyond the author's description alone. GEN-30 allows normal re-entry from Sprohm.

Remaining scope: Mission victory, concluding scene and game ending/credits were not observed. Its nature as the last battle remains a lead from metadata, not evidence of the epilogue. IN-04 near Ambervale is a negative attempt: no active mission and only day advancement/Save, without a final event.

<a id="prove-e-prossimi-controlli"></a>

## Evidence and next checks

Evidence for dispatch return/growth and the pub path is described in the
[ordinary dispatch record](../changes/2026-10-23-consume-european-progression-gates-and-receipt-records.json).
The 1000-frame pair is complete and was not repeated. RESULTS, Mission Cleared
and acceptance images were rechecked. Mission rewards and clan-progression
rewards remain separate cases, DS-02 and DS-03.

For other subcases reuse evidence for [laws and deployment](../ffta_laws_layout.md),
[shop/bank](../ffta_shop_bank_mouse.md), the
[enemy replay](../changes/2026-10-20-record-observed-enemy-activation-replay.json)
and contracts/scenarios already recorded in [game data](../rom_data_gameplay.md).

The first inventory's observations and terminal results are in
`target/debug-saves-audit/{DS-04,DS-17-global}/`, with captures and guards on
originals. The intermediate `DS-17/` session is incomplete (SIGTERM143), retains
only observations up to Equip and does not prove normal closure. That inventory
did not create new game snapshots.

The new-file expansion is retained in **`target/debug-saves-expansion/`**:
`input-manifest.json`, `boot-results.json`, `helper-build.json`,
`package-result.json`, replay directories and `USA-final/` with input/captures.
A private Windows helper linked to the already built core executed ordinary
inputs and exported checkpoints, verifying reload in a new Machine. The
application was not rebuilt. Initial failed link attempts and a wrong helper
path are retained as tool errors, without attributing them to the game.

The 25 boots/loads and cited replays are terminal exit0. The GUI session **was
interrupted**: the log ends with X connection broken and `final.json` is missing.
Intermediate captures document observed actions; no normal closure, full
equivalence, audio quality or performance is inferred from that session. No new
native Windows, physical controller, Linux or Android evidence. Final checks
retain hashes of originals and sources, preview/payload checksums, ROM/version
and absence of tracked Git files.

Overall inventory **partial**: **18 validated, 6 partial, 2 uncovered**.
DS-07 crowd/variety, DS-08 slow-turn reproduction, DS-12 elevation/occlusion
path, DS-17 long descriptions/categories, DS-21 required-item consumption and
DS-26 conclusion/ending remain partial. DS-03 threshold reward and DS-15
alternative objective remain uncovered. DS-20 is now validated by the new EUR
point and Polly's observed failed return. The Girl In Love attempt stops at the
missing requirement; Prison Break has objective DEFEAT ALL ENEMIES and no
mandatory item. Bind previews at 0% remain negative attempts, distinct from the
DS-10 icon scene. Do not repeat completed validations to reconstruct context.

Also verified **26 Resume Battle menus** from existing checkpoints: none loads a
battle with A. New **GEN-33**, produced by normal Save Now, serves as a positive
control: summary, notice, Yes confirmation and the field at Marche's turn were
actually reached after a new start. No original SRAM is rewritten by replay.
The three standard-export GUIs and the naming GUI exit normally with exit0; the
first interrupted GUI remains separate. Generated masters, catalog and
instructions are in ignored `debug_saves/`; terminal results and captures remain
under `target/debug-saves-expansion/`.
