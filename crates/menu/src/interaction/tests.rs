use super::*;

#[test]
fn directions_navigate_without_changing_values_and_sidebar_needs_no_extra_key() {
    for section in [0, 1, 2, 3, 4, 5, 6, 7] {
        let mut menu = Menu::new(true, true);
        let mut av = AvConfig::default();
        menu.atlas_section(clan_atlas::Domain::Game, section);
        let before = av.serialize();
        for input in [
            MenuInput::Down,
            MenuInput::Up,
            MenuInput::Right,
            MenuInput::Left,
        ] {
            assert_eq!(menu.handle(input, &mut av), None);
            assert_eq!(av.serialize(), before);
        }
    }
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.handle(MenuInput::Left, &mut av);
    assert!(menu.atlas.sidebar_focus);
    menu.handle(MenuInput::Down, &mut av);
    menu.handle(MenuInput::Right, &mut av);
    assert_eq!(menu.page, Page::Saves);
    assert!(!menu.atlas.sidebar_focus);
}

#[test]
fn confirmed_volume_and_transparency_choices_commit_or_cancel_without_early_changes() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    for (section, row, index) in [(4, Row::Volume, 5), (5, Row::MenuTransparency, 5)] {
        menu.atlas_section(clan_atlas::Domain::Game, section);
        menu.selected = menu.rows().iter().position(|r| *r == row).unwrap();
        let before = av.serialize();
        menu.handle(MenuInput::Confirm, &mut av);
        menu.dropdown.as_mut().unwrap().index = index;
        assert_eq!(av.serialize(), before);
        menu.handle(MenuInput::Cancel, &mut av);
        assert_eq!(av.serialize(), before);
        menu.handle(MenuInput::Confirm, &mut av);
        menu.dropdown.as_mut().unwrap().index = index;
        assert!(matches!(
            menu.handle(MenuInput::Confirm, &mut av),
            Some(MenuAction::Changed(_))
        ));
    }
    assert_eq!(av.audio_volume, 0.5);
    assert_eq!(av.menu_transparency, 0.5);
}

#[test]
fn slot_picker_is_pending_until_confirm_and_cancel_keeps_the_old_slot() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.state_slots = vec![StateSlot::default(); 99];
    menu.selected = 1;
    menu.handle(MenuInput::Confirm, &mut av);
    assert_eq!(menu.state_slot, 0);
    menu.dropdown.as_mut().unwrap().index = 98;
    menu.handle(MenuInput::Cancel, &mut av);
    assert_eq!(menu.state_slot, 0);
    menu.handle(MenuInput::Confirm, &mut av);
    menu.dropdown.as_mut().unwrap().index = 98;
    assert_eq!(menu.handle(MenuInput::Confirm, &mut av), None);
    assert_eq!(menu.state_slot, 98);
}

#[test]
fn input_device_tabs_are_navigation_until_confirm() {
    let mut menu = Menu::new(true, true);
    let mut av = AvConfig::default();
    menu.atlas_section(clan_atlas::Domain::Game, 6);
    assert_eq!(menu.handle(MenuInput::Up, &mut av), None);
    assert_eq!(menu.input_tabs_focus, Some(0));
    assert_eq!(menu.handle(MenuInput::Right, &mut av), None);
    assert_eq!(menu.page, Page::Inputs);
    assert_eq!(
        menu.handle(MenuInput::Confirm, &mut av),
        Some(MenuAction::InputDevice(Device::Gamepad))
    );
    assert_eq!(menu.page, Page::Gamepad);
    menu.handle(MenuInput::Up, &mut av);
    assert_eq!(menu.input_tabs_focus, Some(1));
    let r=menu.input_tab_rect(0,854,480);
    assert_eq!(menu.click(r.x+r.w/2,r.y+r.h/2,854,480,true,&mut av),Some(MenuAction::InputDevice(Device::Keyboard)));
    assert_eq!(menu.page,Page::Inputs);
    assert_eq!(menu.input_tabs_focus,None);
}

#[test]
fn transparency_reveals_game_pixels_and_preserves_foreground_and_scope() {
    let menu = Menu::new(true, true);
    let mut av = AvConfig {menu_transparency:0.,..Default::default()};
    let cfg = InputConfig::default();
    let mut opaque = vec![[110, 70, 190, 255]; 854 * 480];
    let mut transparent = opaque.clone();
    menu.draw_inputs(&av, &cfg, &mut opaque, 854, 480);
    av.menu_transparency = 1.;
    menu.draw_inputs(&av, &cfg, &mut transparent, 854, 480);
    assert_eq!(opaque[0], [9, 24, 39, 255]);
    assert_eq!(transparent[0], [110, 70, 190, 255]);
    assert!(
        transparent
            .iter()
            .any(|p| p[0] > 220 && p[1] > 220 && p[2] > 220)
    );
    assert!(transparent.iter().all(|p| p[3] == 255));
    assert_eq!(canvas::panel_alpha(), 255);
}

#[test]
fn tool_choices_require_ok_and_cancel_never_changes_the_committed_draft() {
    let mut menu = Menu::unlocked_for_test(true, true);
    let mut av = AvConfig::default();
    menu.open_tool(
        DebugTool::Inspect,
        vec![ToolField {
            key: "mode".into(),
            label: "Mode".into(),
            value: "off".into(),
            help: String::new(),
            kind: ToolFieldKind::Choice(vec!["off".into(), "on".into()]),
        }],
        String::new(),
    );
    let before = menu.tool_values();
    menu.handle(MenuInput::Confirm, &mut av);
    assert!(menu.content_text_focus());
    menu.handle(MenuInput::Right, &mut av);
    assert_eq!(menu.tool_values(), before);
    menu.handle(MenuInput::Cancel, &mut av);
    assert_eq!(menu.tool_values(), before);
    menu.handle(MenuInput::Confirm, &mut av);
    menu.handle(MenuInput::Right, &mut av);
    menu.handle(MenuInput::Confirm, &mut av);
    assert_eq!(menu.tool_values().1[0].1, "on");
}
