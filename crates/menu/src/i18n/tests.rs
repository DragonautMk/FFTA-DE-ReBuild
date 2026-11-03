use super::*;
use crate::{
    clan_atlas::Domain, diagnostics, AvConfig, Changed, Device, InputConfig, Menu, MenuAction,
    MenuInput, Page, Row,
};
use std::collections::BTreeSet;

fn texts(menu: &Menu, av: &AvConfig, w: usize, h: usize) -> Vec<diagnostics::TextRun> {
    let mut px = vec![[30, 70, 100, 255]; w * h];
    diagnostics::capture(|| menu.draw_inputs(av, &InputConfig::default(), &mut px, w, h))
        .unwrap()
        .runs
}

#[test]
fn standard_save_status_uses_english_keys_and_italian_translations() {
    let statuses = [
        ("No standard save found", "Nessun salvataggio standard trovato"),
        ("Last .sav modification: date unavailable", "Ultima modifica file .sav: data non disponibile"),
        (".sav file: metadata unavailable", "File .sav: metadati non disponibili"),
    ];
    let mut menu = Menu::new(true, true);
    assert_eq!(menu.standard_save_text(), statuses[0].0);
    for (key, italian) in statuses {
        menu.set_standard_save_status(key.into());
        assert_eq!(menu.standard_save_text(), key);
        let _ = texts(&menu, &AvConfig { menu_language: Language::parse("it"), ..AvConfig::default() }, 854, 480);
        assert_eq!(menu.standard_save_text(), italian);
        let _ = texts(&menu, &AvConfig::default(), 854, 480);
    }
    assert_eq!(text(Language::default(), "Nessun salvataggio standard trovato"), statuses[0].0);
}

#[test]
fn save_form_english_keys_preserve_legacy_translations_and_rendering() {
    let pairs = [
        (" · active", " · attivo"),
        ("Saves unavailable", "Salvataggi non disponibili"),
        ("Cartridge backup · two automatic copies", "Backup cartuccia · due copie automatiche"),
        ("Preview unavailable", "Anteprima non disponibile"),
        ("Empty slot", "Slot vuoto"),
        ("Date unavailable", "Data non disponibile"),
        ("The game restarts from this copy. Confirm replaces the active cartridge save and the session's progress.", "Il gioco riparte da questa copia. Confermare sostituisce il salvataggio cartuccia attivo; il progresso della sessione viene sostituito."),
        ("Automatic copies created at startup. Selecting a copy requires confirmation before restoring.", "Copie automatiche create all'avvio. Selezionare una copia richiede conferma prima del ripristino."),
        ("Undo last operation", "Annulla ultima operazione"),
        ("Cancel", "Annulla"),
        ("Arrows: navigate · OK: choose/confirm · Cancel: back", "Frecce: naviga · OK: scegli/conferma · Annulla: indietro"),
        ("{name}\nCopies contain progress saved in the game. Using a file restarts the game. Replacing requires confirmation; Cancel is selected by default.", "{name}\nLe copie contengono il progresso salvato nel gioco. Usare un file riavvia il gioco. Sostituire richiede conferma; Annulla è preselezionato."),
    ];
    for (english, legacy) in pairs {
        assert_eq!(text(Language::default(), english), text(Language::default(), legacy));
        assert_eq!(text(Language::parse("it"), english), text(Language::parse("it"), legacy));
    }
    for (language, expected) in [(Language::default(), "Empty slot"), (Language::parse("it"), "Slot vuoto")] {
        let mut menu = Menu::new(true, true);
        menu.open(Page::SaveStates);
        let av = AvConfig { menu_language: language, ..AvConfig::default() };
        assert!(texts(&menu, &av, 854, 480).iter().any(|run| run.text == expected));
    }
}

#[test]
fn atlas_information_and_tool_english_keys_preserve_existing_translations() {
    let pairs = [
        ("Dettagli · sola lettura", "Details · read only"),
        ("Riferimenti Menu / CLI · sola lettura", "Menu / CLI reference · read only"),
        ("Conferma uscita", "Confirm exit"),
        ("Cambia ROM", "Switch ROM"),
        ("Sovrascrivi lo stato?", "Overwrite this state?"),
        ("Ripristina il backup?", "Restore this backup?"),
        ("Usa salvataggio e riavvia", "Use save and restart"),
        ("Sostituisci salvataggio cartuccia", "Replace game save"),
        ("Catalogo DS", "DS catalog"),
        ("Catalogo cheat", "Cheat catalog"),
        ("Selezione contenuti", "Content selection"),
        ("Parametri dello strumento", "Tool parameters"),
        ("Input e selezioni correnti", "Inputs and current selections"),
        ("Parametri e profili", "Parameters and profiles"),
        ("Script e strumenti", "Scripts and tools"),
        ("Salvataggi cartuccia", "Game saves"),
        ("Copie di sicurezza", "Backups"),
        ("ATLANTE DEL CLAN", "CLAN ATLAS"),
        ("Minimo menu: 480 x 360. Ridimensiona o premi Esc.", "Menu minimum: 480 x 360. Resize or press Esc."),
        ("ATLANTE", "ATLAS"),
        ("Registro del Clan", "Clan ledger"),
        ("Gioco", "Game"),
        ("APRI / CHIUDI MENU", "OPEN / CLOSE MENU"),
        ("Tastiera · Esc recupero", "Keyboard · Esc recovery"),
        ("{back}: indietro", "{back}: back"),
        ("{back} indietro · {focus} focus", "{back} back · {focus} focus"),
        ("non disponibile", "unavailable"),
        ("  ·  COPIA DEBUG ISOLATA", "  ·  ISOLATED DEBUG COPY"),
        ("Frecce: naviga · OK: modifica/conferma · Annulla: indietro", "Arrows: navigate · OK: edit/confirm · Cancel: back"),
        ("{back} Chiudi", "{back} Close"),
        ("Eventi · previsto", "Events · planned"),
        ("CMD · previsto", "CMD · planned"),
        ("Data non disponibile", "Date unavailable"),
        ("Torna al gioco", "Return to game"),
        ("Stato rapido {} / {} · {} · {}", "Quick state {0} / {1} · {2} · {3}"),
        ("Sovrascrivi", "Overwrite"),
        ("Salva", "Save"),
        ("Carica", "Load"),
        ("Annulla ultima operazione", "Undo last operation"),
        ("Catalogo locale consultabile in sola lettura. Voci disattivate; decoder e applicazione non disponibili. Nessun codice privato viene incluso.", "Local catalog for read-only browsing. Entries disabled; decoding and application unavailable. No private codes are included."),
        ("Copia isolata attiva. Conferma richiama gli handler esistenti; validazioni e prerequisiti vengono ricontrollati prima dell'operazione.", "Isolated copy active. Confirm invokes existing handlers; validation and prerequisites are checked again before the operation."),
        ("Le operazioni guest richiedono una copia di debug isolata. Selezionare una voce non esegue comandi. Apri una sessione dagli strumenti Sessioni e casi.", "Guest operations require an isolated debug copy. Selecting an entry does not run commands. Open a session from Sessions and cases."),
        ("Debug V2 · riferimenti", "Debug V2 · reference"),
        ("Nessuna voce in questo gruppo", "No entries in this group"),
        ("Nessuna capacità disponibile. Questa vista non esegue operazioni.", "No capability available. This view does not execute operations."),
        ("↑↓ elenco · ←→ dettagli · {} indietro", "↑↓ list · ←→ details · {0} back"),
        ("Dettagli della selezione all’apertura", "Selection details at opening"),
        ("Valore: {value}", "Value: {value}"),
        ("{}\n{}\nPosizione: {}\nIdentità: {}\nStato: {}\nCompatibilità: {}\n{}", "{0}\n{1}\nLocation: {2}\nIdentity: {3}\nStatus: {4}\nCompatibility: {5}\n{6}"),
        ("Menu tastiera: {}", "Keyboard menu: {0}"),
        ("Menu gamepad: {}", "Gamepad menu: {0}"),
        ("Stato {} / {}: {}", "State {0} / {1}: {2}"),
        ("Data stato: {}", "State date: {0}"),
        ("Stato", "Status"),
        ("Sessione", "Session"),
        ("File: {}\nStato: {}", "File: {0}\nStatus: {1}"),
        ("↑↓ / rotella: scorri · F1 / R3 / Esc: chiudi", "↑↓ / wheel: scroll · F1 / R3 / Esc: close"),
        ("Scelta: frecce regolano · OK applica · Annulla scarta", "Choice: arrows adjust · OK applies · Cancel discards"),
        ("Frecce: naviga · OK modifica · usa i pulsanti per eseguire", "Arrows: navigate · OK edits · use buttons to execute"),
    ];
    for (legacy, english) in pairs {
        for language in [Language::default(), Language::parse("it")] {
            assert_eq!(text(language, legacy), text(language, english), "{english:?}");
        }
    }
    for language in [Language::default(), Language::parse("it")] {
        let args = [("0", "{1} <private>".into()), ("1", "next".into())];
        assert_eq!(
            format(language, "File: {}\nStato: {}", &args),
            format(language, "File: {0}\nStatus: {1}", &args),
        );
    }
}

#[test]
fn menu_presentation_keys_keep_localized_text_and_legacy_routes() {
    let pairs = [
        ("Panoramica", "Overview"),
        ("Salvataggi", "Saves"),
        ("Finestre di testo", "Text windows"),
        ("Controlli", "Controls"),
        ("Sistema", "System"),
        ("Sessioni e casi", "Sessions and cases"),
        ("Replay e confronti", "Replays and comparisons"),
        ("Esecuzione e memoria", "Execution and memory"),
        ("Tracce e ispettori", "Traces and inspectors"),
        ("Console e script", "Consoles and scripts"),
        ("Report e archivio", "Reports and archive"),
        ("Riferimenti Menu / CLI", "Menu / CLI reference"),
        ("Trasparenza menu", "Menu transparency"),
        ("Console eventi", "Event console"),
        ("Console comandi", "Command console"),
        ("Catalogo cheat (sola lettura)", "Cheat catalog (read only)"),
        ("D-pad · {} OK · {} indietro · {} focus · {} dettagli", "D-pad · {0} OK · {1} back · {2} focus · {3} details"),
        ("VOL {}− / {}+ / {}+{} mute · FAST {}− / {}+ / {}+{} ON", "VOL {0}− / {1}+ / {2}+{3} mute · FAST {4}− / {5}+ / {6}+{7} ON"),
        ("Frecce · Invio OK · Esc indietro · Tab focus · F1 dettagli", "Arrows · Enter OK · Esc back · Tab focus · F1 details"),
        ("Volume / Fast: voce + Invio · Shift+Tab: vista Debug", "Volume / Fast: entry + Enter · Shift+Tab: Debug view"),
        ("Volume / Fast: scegli la voce + Invio", "Volume / Fast: choose an entry + Enter"),
    ];
    for (legacy, english) in pairs {
        for language in [Language::default(), Language::parse("it")] {
            assert_eq!(text(language, legacy), text(language, english), "{english:?}");
        }
    }
    let legacy_sections = [
        "Panoramica", "Sessioni e casi", "Replay e confronti", "Esecuzione e memoria",
        "Tracce e ispettori", "Console e script", "Cheat", "Report e archivio",
    ];
    assert_eq!(crate::clan_atlas::DEBUG_SECTIONS, legacy_sections);
    for (index, legacy) in legacy_sections.into_iter().enumerate() {
        assert_eq!(crate::clan_atlas::reference_group(&format!("Debug / {legacy} / action")), index);
        for language in [Language::default(), Language::parse("it")] {
            assert_eq!(text(language, legacy), text(language, crate::clan_atlas::DEBUG_SECTIONS_LABELS[index]));
        }
    }
    for index in 0..crate::clan_atlas::GAME_SECTIONS.len() {
        for language in [Language::default(), Language::parse("it")] {
            assert_eq!(text(language, crate::clan_atlas::GAME_SECTIONS[index]), text(language, crate::clan_atlas::GAME_SECTIONS_LABELS[index]));
        }
    }
}

#[test]
fn catalog_keys_are_unique_and_translated_placeholders_match() {
    for language in Language::ALL {
        let mut keys = BTreeSet::new();
        for &(key, value) in language.messages() {
            assert!(keys.insert(key), "Duplicate catalog key: {key}");
            assert!(!value.is_empty(), "Empty translation: {key}");
        }
    }
    assert_eq!(
        text(Language::default(), "unknown future key"),
        "unknown future key"
    );
    assert_eq!(
        text(Language::parse("it"), "unknown future key"),
        "unknown future key"
    );
    assert_eq!(
        Row::Key(crate::Button::Select).localized_label(Language::parse("it")),
        "Select"
    );
    let args = [("stamp", "{stamp} <private> àèìòù".into())];
    assert_eq!(
        format(
            Language::parse("it"),
            "Last .sav modification: {stamp}",
            &args
        ),
        "Ultima modifica file .sav: {stamp} <private> àèìòù"
    );
}

#[test]
fn language_requires_confirmation_cancel_preserves_preference_and_commit_updates_immediately() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.atlas_section(Domain::Game, 7);
    assert_eq!(menu.rows()[menu.selected], Row::Language);
    menu.handle(MenuInput::Confirm, &mut av);
    for _ in 0..Language::ALL
        .iter()
        .position(|l| l.token() == "it")
        .unwrap()
    {
        menu.handle(MenuInput::Down, &mut av);
    }
    assert_eq!(av.menu_language, Language::default());
    menu.handle(MenuInput::Cancel, &mut av);
    assert_eq!(av.menu_language, Language::default());
    menu.handle(MenuInput::Confirm, &mut av);
    for _ in 0..Language::ALL
        .iter()
        .position(|l| l.token() == "it")
        .unwrap()
    {
        menu.handle(MenuInput::Down, &mut av);
    }
    assert_eq!(
        menu.handle(MenuInput::Confirm, &mut av),
        Some(MenuAction::Changed(Changed::Overlay))
    );
    assert_eq!(av.menu_language, Language::parse("it"));
    assert_eq!(menu.language(), Language::parse("it"));
    assert_eq!(menu.tr("Panoramica"), "Panoramica");
    let loaded = AvConfig::parse(&av.serialize());
    let reopened = Menu::new(true, true);
    let runs = texts(&reopened, &loaded, 854, 480);
    assert!(runs.iter().any(|r| r.text == "Torna al gioco"));
    assert_eq!(reopened.language(), Language::parse("it"));
    menu.handle(MenuInput::Confirm, &mut av);
    for _ in 0..Language::ALL
        .iter()
        .position(|l| l.token() == "it")
        .unwrap()
    {
        menu.handle(MenuInput::Up, &mut av);
    }
    assert_eq!(
        menu.handle(MenuInput::Confirm, &mut av),
        Some(MenuAction::Changed(Changed::Overlay))
    );
    assert_eq!(menu.language(), Language::default());
    assert_eq!(menu.tr("Panoramica"), "Overview");
}

#[test]
fn mouse_language_choice_and_translated_video_options_keep_canonical_values() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.atlas_section(Domain::Game, 7);
    let r = crate::clan_atlas::row_rect(&menu, 0, 854, 480).unwrap();
    menu.click(r.x + 3, r.y + 3, 854, 480, true, &mut av);
    let target = Language::ALL
        .iter()
        .position(|l| l.token() == "it")
        .unwrap();
    for _ in 0..target {
        menu.handle(MenuInput::Down, &mut av);
    }
    let dd = menu.dropdown.as_ref().unwrap();
    let (x, y, _, _, _, first) = crate::theme::dropdown_geometry(dd, 854, 480);
    let pitch = (crate::clan_atlas::geometry(854, 480).body.h / 15).clamp(14, 40);
    assert_eq!(
        menu.click(
            x + 10,
            y + 6 + pitch * (target - first) + 2,
            854,
            480,
            true,
            &mut av
        ),
        Some(MenuAction::Changed(Changed::Overlay))
    );
    assert_eq!(av.menu_language, Language::parse("it"));
    menu.atlas_section(Domain::Game, 5);
    menu.selected = menu.rows().iter().position(|r| *r == Row::Fps).unwrap();
    menu.handle(MenuInput::Confirm, &mut av);
    menu.handle(MenuInput::Down, &mut av);
    assert!(texts(&menu, &av, 854, 480)
        .iter()
        .any(|r| r.text == "INTERNO"));
    menu.handle(MenuInput::Confirm, &mut av);
    assert_eq!(av.video_fps, "internal");
}

#[test]
fn both_languages_render_navigation_saves_and_hints_at_supported_sizes() {
    for language in [Language::default(), Language::parse("it")] {
        let av = AvConfig {
            menu_language: language,
            ..AvConfig::default()
        };
        for (w, h) in [(480, 640), (854, 480), (1280, 720)] {
            let mut menu = Menu::new(true, true);
            menu.set_standard_save_timestamp(0);
            let runs = texts(&menu, &av, w, h);
            for key in ["Panoramica", "Salvataggi", "Sistema", "Torna al gioco"] {
                assert!(
                    runs.iter().any(|r| r.text.contains(text(language, key))),
                    "Missing {key:?} at {w}x{h}"
                );
            }
            let expected = if language == Language::default() {
                "Last .sav modification: 1970-01-01 00:00 UTC"
            } else {
                "Ultima modifica file .sav: 1970-01-01 00:00 UTC"
            };
            assert!(runs.iter().any(|r| r.text == expected));
            menu.atlas_section(Domain::Game, 7);
            for key in ["Language", "Restart ROM", "Exit"] {
                menu.selected = menu.rows().iter().position(|r| r.label() == key).unwrap();
                let runs = texts(&menu, &av, w, h);
                let run = runs
                    .iter()
                    .find(|r| r.text == text(language, key))
                    .unwrap_or_else(|| {
                        panic!(
                            "Missing {key} for {language:?} at {w}x{h}: {:?}",
                            runs.iter().map(|r| &r.text).collect::<Vec<_>>()
                        )
                    });
                assert!(
                    !run.width_stopped && !run.canvas_clipped,
                    "Clipped {key} at {w}x{h}"
                );
            }
            menu.open(Page::SaveStates);
            assert!(texts(&menu, &av, w, h)
                .iter()
                .any(|r| r.text == text(language, "Load state")));
            menu.atlas_section(Domain::Game, 0);
        }
        let menu = Menu::new(true, true);
        menu.note_input(Device::Gamepad);
        let _ = texts(&menu, &av, 854, 480);
        assert!(
            menu.control_hints()[0].contains(if language == Language::default() {
                "back"
            } else {
                "indietro"
            })
        );
    }
}

#[test]
fn independent_menus_and_retained_undo_details_use_the_current_language() {
    let mut italian = Menu::new(true, true);
    italian.open(Page::SaveStates);
    italian.set_state_undo_description(Some(("Undo load (slot {slot})", 9)));
    let italian_config = AvConfig {
        menu_language: Language::parse("it"),
        ..AvConfig::default()
    };
    assert!(texts(&italian, &italian_config, 854, 480)
        .iter()
        .any(|r| r.text == "Annulla caricamento (slot 9)"));
    let english = Menu::new(true, true);
    assert!(texts(&english, &AvConfig::default(), 854, 480)
        .iter()
        .any(|r| r.text == "Return to game"));
    assert_eq!(italian.language(), Language::parse("it"));
    assert_eq!(english.language(), Language::default());
    assert!(texts(&italian, &AvConfig::default(), 854, 480)
        .iter()
        .any(|r| r.text == "Undo load (slot 9)"));
    assert_eq!(
        italian.state_undo_description,
        Some(("Undo load (slot {slot})", 9))
    );
}

#[test]
fn every_discovered_language_is_listed_selectable_and_persisted_by_code() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.atlas_section(Domain::Game, 7);
    assert_eq!(
        menu.row_choices(Row::Language),
        Language::ALL
            .iter()
            .map(|l| l.name().to_string())
            .collect::<Vec<_>>()
    );
    for (index, language) in Language::ALL.into_iter().enumerate() {
        menu.handle(MenuInput::Confirm, &mut av);
        for _ in 0..Language::ALL.len() {
            if menu.dropdown.as_ref().unwrap().index == index {
                break;
            }
            menu.handle(MenuInput::Down, &mut av);
        }
        menu.handle(MenuInput::Confirm, &mut av);
        assert_eq!(av.menu_language, language);
        assert_eq!(
            AvConfig::parse(&av.serialize()).menu_language.token(),
            language.token()
        );
    }
}
