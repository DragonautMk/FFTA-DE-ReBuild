use super::*;

fn text(menu: &Menu, av: &AvConfig) -> String {
    let mut pixels = vec![[60, 80, 100, 255]; 854 * 480];
    diagnostics::capture(|| menu.draw_inputs(av, &InputConfig::default(), &mut pixels, 854, 480))
        .unwrap()
        .runs
        .into_iter()
        .map(|r| r.text)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn locked_menu_hides_debug_entries_and_rejects_mouse_and_programmatic_routes() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    let labels = text(&menu, &av);
    assert!(!labels.contains("Debug") && !labels.contains("previsto"));
    menu.atlas_section(clan_atlas::Domain::Debug, 0);
    assert_eq!(menu.atlas.domain, clan_atlas::Domain::Game);
    let domain = clan_atlas::geometry(854, 480).domain;
    menu.click(
        domain.x + domain.w - 5,
        domain.y + 5,
        854,
        480,
        true,
        &mut av,
    );
    assert_eq!(menu.atlas.domain, clan_atlas::Domain::Game);
    menu.atlas_section(clan_atlas::Domain::Game, 1);
    assert!(
        !menu.rows().contains(&Row::ExtendedSaves) && !menu.rows().contains(&Row::GameSavesMenu)
    );
    for page in [
        Page::Debug,
        Page::CliReference,
        Page::DebugTools,
        Page::GameSaves,
        Page::ContentSelector,
        Page::DebugTool,
    ] {
        menu.open(page);
        assert_eq!(menu.page, Page::Saves);
    }
    menu.atlas_reference_tab();
    assert!(!menu.atlas.planned);
}

#[test]
fn exact_sequence_emits_one_save_action_and_persists_across_a_new_menu() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    let original_volume = av.audio_volume;
    for (i, input) in SEQUENCE.into_iter().enumerate() {
        let action = menu.handle(input, &mut av);
        if i + 1 == SEQUENCE.len() {
            assert_eq!(action, Some(MenuAction::Changed(Changed::Overlay)));
        } else {
            assert_eq!(action, None);
            assert!(!av.debug);
        }
    }
    assert!(menu.debug_enabled() && av.debug);
    assert_eq!(av.audio_volume, original_volume);
    let saved = av.serialize();
    menu.reset();
    assert!(menu.debug_enabled());
    let loaded = AvConfig::parse(&saved);
    let mut reopened = Menu::new(true, true);
    reopened.set_debug_enabled(loaded.debug);
    assert!(text(&reopened, &loaded).contains("Debug"));
    reopened.atlas_section(clan_atlas::Domain::Debug, 0);
    assert_eq!(reopened.page, Page::Debug);
    reopened.set_debug_enabled(false);
    assert_eq!(reopened.atlas.domain, clan_atlas::Domain::Game);
    assert!(!text(&reopened, &AvConfig::default()).contains("Debug"));
}

#[test]
fn mismatch_reset_capture_dropdown_and_pointer_cannot_complete_a_partial_sequence() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.handle(MenuInput::Up, &mut av);
    menu.reset();
    for input in SEQUENCE.into_iter().skip(1) {
        menu.handle(input, &mut av);
    }
    assert!(!av.debug);
    menu.reset();
    menu.capture = Some(Bind::Menu);
    for input in SEQUENCE {
        menu.handle(input, &mut av);
    }
    assert!(!av.debug);
    assert_eq!(menu.capturing(), Some(Bind::Menu));
    menu.end_capture();
    menu.atlas_section(clan_atlas::Domain::Game, 4);
    menu.selected = 0;
    menu.handle(MenuInput::Confirm, &mut av);
    assert!(menu.dropdown.is_some());
    for input in SEQUENCE {
        menu.handle(input, &mut av);
    }
    assert!(!av.debug);
    menu.handle(MenuInput::Cancel, &mut av);
    menu.reset();
    menu.handle(MenuInput::Up, &mut av);
    menu.handle(MenuInput::Up, &mut av);
    menu.click(0, 0, 854, 480, true, &mut av);
    for input in SEQUENCE.into_iter().skip(2) {
        menu.handle(input, &mut av);
    }
    assert!(!av.debug);
    menu.reset();
    menu.handle(MenuInput::Up, &mut av);
    // Overlapping UUU suffix still contains the required final UU prefix.
    for input in SEQUENCE {
        menu.handle(input, &mut av);
    }
    assert!(av.debug);
}
