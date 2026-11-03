//! Mineral-blue clan ledger, composed by the native canvas.
use super::{
    canvas::Canvas,
    clan_atlas::{self as atlas, Rect},
    *,
};
const BG: [u8; 3] = [9, 24, 39];
const PANEL: [u8; 3] = [28, 56, 78];
const SELECTED: [u8; 3] = [45, 78, 100];
const TEXT: [u8; 3] = [233, 239, 236];
const MUTED: [u8; 3] = [168, 189, 199];
const BRASS: [u8; 3] = [209, 175, 117];
const EDGE: [u8; 3] = [89, 121, 139];

fn brand_logo(c: &mut Canvas, sidebar: Rect, scale: f32) {
    static LOGOS: std::sync::OnceLock<Vec<tiny_skia::Pixmap>> = std::sync::OnceLock::new();
    let logos = LOGOS.get_or_init(|| {
        let master = image::load_from_memory(include_bytes!("../assets/de-rebuild-logo.png"))
            .expect("embedded menu logo must be a valid PNG")
            .to_rgba8();
        [32, 64, 128, 256].into_iter().map(|side| {
            let image = image::imageops::resize(&master, side, side, image::imageops::FilterType::Lanczos3);
            let size = tiny_skia::IntSize::from_wh(side, side).unwrap();
            let mut pixels = image.into_raw();
            // tiny-skia requires premultiplied alpha.
            for pixel in pixels.chunks_exact_mut(4) {
                let alpha = u16::from(pixel[3]);
                for channel in &mut pixel[..3] {
                    *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
                }
            }
            tiny_skia::Pixmap::from_vec(pixels, size).expect("valid menu logo dimensions")
        }).collect()
    });
    let size = 44. * scale;
    let x = sidebar.x as f32 + 4. * scale;
    let y = sidebar.y as f32 + 4. * scale;
    let logo = logos.iter().find(|logo| logo.width() as f32 >= size).unwrap_or(&logos[3]);
    let factor = size / logo.width() as f32;
    let paint = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Bilinear,
        ..Default::default()
    };
    c.pix.draw_pixmap(
        0, 0, logo.as_ref(), &paint,
        tiny_skia::Transform::from_row(factor, 0., 0., factor, x, y), None,
    );
}
fn focused_panel(c: &mut Canvas, r: Rect) {
    // Focus must remain distinguishable from the active page, even at100% transparency.
    canvas::with_panel_alpha(255,||c.rect(r.x as f32,r.y as f32,r.w as f32,r.h as f32,2.,BRASS,BRASS,TEXT));
}
pub(super) fn panel(c: &mut Canvas, r: Rect, on: bool) {
    c.rect(
        r.x as f32,
        r.y as f32,
        r.w as f32,
        r.h as f32,
        2.,
        if on { SELECTED } else { PANEL },
        if on { SELECTED } else { PANEL },
        if on { BRASS } else { EDGE },
    );
}
pub(super) fn text(c: &mut Canvas, r: Rect, s: &str, size: f32, title: bool, color: [u8; 3]) {
    c.text(
        r.x as f32 + 10.,
        r.y as f32 + ((r.h as f32 - size) / 2.).max(2.),
        s,
        color,
        size,
        title,
        r.w.saturating_sub(20) as f32,
    );
}
fn title(menu: &Menu) -> &str {
    match menu.page {
        Page::Information => menu.tr("Details · read only"),
        Page::CliReference => menu.tr("Menu / CLI reference · read only"),
        Page::Exit => menu.tr("Confirm exit"),
        Page::SwitchRom => menu.tr("Switch ROM"),
        Page::ReplaceState => menu.tr("Overwrite this state?"),
        Page::RestoreBackup => menu.tr("Restore this backup?"),
        Page::SwitchGameSave => menu.tr("Use save and restart"),
        Page::ReplaceGameSave => menu.tr("Replace game save"),
        Page::DebugCatalog => menu.tr("DS catalog"),
        Page::CheatCatalog => menu.tr("Cheat catalog"),
        Page::ContentSelector => menu.tr("Content selection"),
        Page::DebugTool => menu.tr("Tool parameters"),
        Page::ToolInputs => menu.tr("Inputs and current selections"),
        Page::DebugTools => menu.tr("Parameters and profiles"),
        Page::ScriptLibrary => menu.tr("Scripts and tools"),
        Page::SaveStates => menu.tr("Save states"),
        Page::GameSaves => menu.tr("Game saves"),
        Page::SaveBackups => menu.tr("Backups"),
        _ => menu.tr(menu.atlas.title()),
    }
}
pub(super) fn overview_rects(w: usize, h: usize) -> [Rect; 9] {
    let g = atlas::geometry(w, h);
    let b = g.body;
    let s = g.scale_milli;
    let gap = (s * 8 / 1000).max(3);
    let rh = (s * 42 / 1000).max(24);
    let resume = Rect {
        x: b.x,
        y: b.y,
        w: b.w,
        h: rh,
    };
    let quick_y = resume.y + rh + gap;
    let slot = Rect {
        x: b.x,
        y: quick_y,
        w: b.w,
        h: rh,
    };
    let bw = b.w.saturating_sub(gap * 2) / 3;
    let actions = std::array::from_fn::<_, 3, _>(|i| Rect {
        x: b.x + i * (bw + gap),
        y: slot.y + rh + gap,
        w: bw,
        h: rh,
    });
    let y = actions[0].y + rh + gap + (s * 17 / 1000).max(12);
    let half = b.w.saturating_sub(gap) / 2;
    let row_h =
        b.h.saturating_sub(y - b.y)
            .min(rh * 2 + gap)
            .saturating_sub(gap)
            / 2;
    [
        resume,
        slot,
        actions[0],
        actions[1],
        actions[2],
        Rect {
            x: b.x,
            y,
            w: half,
            h: row_h,
        },
        Rect {
            x: b.x,
            y: y + row_h + gap,
            w: half,
            h: row_h,
        },
        Rect {
            x: b.x + half + gap,
            y,
            w: b.w.saturating_sub(half + gap),
            h: row_h,
        },
        Rect {
            x: b.x + half + gap,
            y: y + row_h + gap,
            w: b.w.saturating_sub(half + gap),
            h: row_h,
        },
    ]
}
pub(super) fn draw(
    menu: &Menu,
    av: &AvConfig,
    keys: &[String; 10],
    ff: &str,
    toggle: &str,
    zin: &str,
    zout: &str,
    px: &mut [[u8; 4]],
    w: usize,
    h: usize,
) {
    if w < 32 || h < 32 || px.len() < w * h {
        return;
    }
    if w < 480 || h < 360 {
        let mut c = Canvas::new(px, w, h);
        panel(&mut c, Rect { x: 0, y: 0, w, h }, false);
        c.text(
            10.,
            10.,
            menu.tr("CLAN ATLAS"),
            TEXT,
            14.,
            true,
            w.saturating_sub(20) as f32,
        );
        c.text(
            10.,
            40.,
            menu.tr("Menu minimum: 480 x 360. Resize or press Esc."),
            MUTED,
            11.,
            false,
            w.saturating_sub(20) as f32,
        );
        c.finish(px);
        return;
    }
    for p in px.iter_mut().take(w * h) {
        let alpha = super::canvas::panel_alpha() as u32;
        for channel in 0..3 {
            p[channel] = ((BG[channel] as u32 * alpha + p[channel] as u32 * (255-alpha) + 127)/255) as u8;
        }
        p[3] = 255;
    }
    let g = atlas::geometry(w, h);
    let s = g.scale_milli as f32 / 1000.;
    let fs = (13. * s).max(10.);
    let mut c = Canvas::new(px, w, h);
    panel(&mut c, g.header, false);
    text(
        &mut c,
        g.header,
        "FFTA DE-ReBuild",
        (17. * s).max(12.),
        true,
        TEXT,
    );
    if g.header.w > 600 {
        c.text(
            (g.header.x + g.header.w - 210) as f32,
            (g.header.y + 10) as f32,
            menu.tr("THE CLAN ARCHIVE"),
            BRASS,
            12. * s,
            true,
            200.,
        );
    }
    panel(&mut c, g.sidebar, false);
    brand_logo(&mut c, g.sidebar, s);
    c.text(
        g.sidebar.x as f32 + 54. * s,
        g.sidebar.y as f32 + 15. * s,
        menu.tr("ATLAS"),
        TEXT,
        (13. * s).max(9.),
        true,
        g.sidebar.w as f32 - 65. * s,
    );
    c.text(
        g.sidebar.x as f32 + 10.,
        g.sidebar.y as f32 + 48. * s,
        menu.tr("Clan ledger"),
        MUTED,
        (11. * s).max(9.),
        false,
        g.sidebar.w as f32 - 20.,
    );
    let domains=if menu.debug_enabled(){2}else{1};
    let half = g.domain.w / domains;
    for (i, label) in [menu.tr("Game"), "Debug"].iter().take(domains).enumerate() {
        let r = Rect {
            x: g.domain.x + i * half,
            w: if i == 0 { half } else { g.domain.w - half },
            ..g.domain
        };
        let on = menu.atlas.index() == i;
        let focused=menu.atlas.sidebar_focus && menu.atlas.nav_cursor==0 && on;
        if focused {focused_panel(&mut c,r);} else {panel(&mut c, r, on);}
        text(&mut c, r, label, fs, true, if focused {BG} else if on { BRASS } else { TEXT });
    }
    let labels = if menu.atlas.domain == atlas::Domain::Game {
        atlas::GAME_SECTIONS_LABELS
    } else {
        atlas::DEBUG_SECTIONS_LABELS
    };
    for (i, r) in g.nav.iter().copied().enumerate() {
        let on = menu.atlas.selected_section() == i;
        let focused=menu.atlas.sidebar_focus && menu.atlas.nav_cursor==i+1;
        if focused {focused_panel(&mut c,r);} else {panel(&mut c,r,on);}
        let label = format!("{}{:02}  {}",if focused {"› "}else{""}, i + 1, menu.tr(labels[i]));
        text(
            &mut c,
            r,
            &label,
            fs * 0.90,
            false,
            if focused {BG} else if on { BRASS } else { TEXT },
        );
    }
    panel(&mut c, g.binding, false);
    c.text(
        (g.binding.x + 8) as f32,
        (g.binding.y + 7) as f32,
        menu.tr("OPEN / CLOSE MENU"),
        MUTED,
        (10. * s).max(8.),
        true,
        g.binding.w.saturating_sub(16) as f32,
    );
    let binding = menu.menu_binding.borrow();
    let shown = binding
        .split('+')
        .map(|v| {
            if menu.hints_gamepad() {
                gamepad_label(v, menu.controller_layout.get())
            } else {
                v.to_uppercase()
            }
        })
        .collect::<Vec<_>>()
        .join(" + ");
    c.text(
        (g.binding.x + 8) as f32,
        (g.binding.y + (27. * s) as usize) as f32,
        &shown,
        BRASS,
        fs,
        false,
        g.binding.w.saturating_sub(16) as f32,
    );
    c.text(
        (g.binding.x + 8) as f32,
        (g.binding.y + (47. * s) as usize) as f32,
        if menu.hints_gamepad() {
            "Gamepad"
        } else {
            menu.tr("Keyboard · Esc recovery")
        },
        MUTED,
        (9. * s).max(8.),
        false,
        g.binding.w.saturating_sub(16) as f32,
    );
    let back_focused=menu.atlas.sidebar_focus && menu.atlas.nav_cursor==9;
    if back_focused {focused_panel(&mut c,g.back);}else{panel(&mut c,g.back,false);}
    let back=menu.hint_button("East","Esc");let focus=menu.hint_button("West","Tab");
    let back_hint=if g.back.w<160 {menu.tr_format("{back}: back", &[("back", (back).to_string())])}else{menu.tr_format("{back} back · {focus} focus", &[("back", (back).to_string()), ("focus", (focus).to_string())])};
    text(
        &mut c,
        g.back,
        &back_hint,
        fs * 0.8,
        false,
        if back_focused {BG}else{TEXT},
    );
    text(
        &mut c,
        g.breadcrumb,
        &format!(
            "{} / {}",
            if menu.atlas.domain == atlas::Domain::Game {
                menu.tr("Game")
            } else {
                "Debug"
            },
            menu.tr(menu.atlas.title())
        ),
        fs * 0.85,
        false,
        MUTED,
    );
    let status = format!(
        "VOL {}%{}   |   FAST {}% · {}   |   FRAME {}{}",
        (av.audio_volume * 100.).round() as u32,
        if av.audio_mute { " MUTE" } else { "" },
        input_config::fast_forward::selected_percent(av.ff_speed),
        if menu.atlas.fast_active { menu.tr("ON") } else { menu.tr("OFF") },
        menu.atlas
            .frame
            .map_or_else(|| menu.tr("unavailable").into(), |v| v.to_string()),
        if menu.debug_session {
            menu.tr("  ·  ISOLATED DEBUG COPY")
        } else {
            ""
        }
    );
    text(&mut c, g.status, &status, fs * 0.85, false, MUTED);
    text(&mut c, g.title, title(menu), (22. * s).max(15.), true, TEXT);
    panel(&mut c, g.body, false);
    panel(&mut c, g.shortcuts, false);
    let hints=menu.control_hints();
    c.text(
        (g.shortcuts.x + 10) as f32,
        (g.shortcuts.y + 8) as f32,
        &hints[0],
        TEXT,
        fs * 0.9,
        false,
        g.shortcuts.w.saturating_sub(20) as f32,
    );
    c.text(
        (g.shortcuts.x + 10) as f32,
        (g.shortcuts.y + (30. * s) as usize) as f32,
        &hints[1],
        MUTED,
        fs * 0.9,
        false,
        g.shortcuts.w.saturating_sub(20) as f32,
    );
    let feedback = [
        menu.state_message.as_str(),
        menu.game_save_message.as_str(),
        menu.backup_message.as_str(),
        menu.debug_message.as_str(),
        menu.session_message.as_str(),
    ]
    .into_iter()
    .find(|v| !v.is_empty())
    .unwrap_or(menu.tr("Arrows: navigate · OK: edit/confirm · Cancel: back"));
    text(&mut c, Rect{w:g.feedback.w.saturating_sub(g.information.w+5),..g.feedback}, menu.tr(feedback), fs * 0.85, false, MUTED);
    panel(&mut c,g.information,menu.atlas.information_hover&&menu.information_available());
    let info=if menu.page==Page::Information {menu.tr_format("{back} Close", &[("back", (back).to_string())])}else{format!("{} Info",menu.hint_button("RightThumb","F1"))};
    text(&mut c,g.information,&info,fs*0.85,false,if menu.information_available(){TEXT}else{MUTED});
    if menu.debug_enabled() && g.breadcrumb.w > 300 {
        let bw = (90. * s) as usize;
        for (i, label) in [menu.tr("Events · planned"), menu.tr("CMD · planned")].iter().enumerate() {
            let r = Rect {
                x: g.breadcrumb.x + g.breadcrumb.w - (2 - i) * bw,
                y: g.breadcrumb.y,
                w: bw,
                h: g.breadcrumb.h,
            };
            panel(&mut c, r, false);
            text(&mut c, r, label, (9. * s).max(8.), false, MUTED);
        }
    }
    c.finish(px);
    if menu.page==Page::Information{information::draw(menu,px,w,h);}
    else if menu.atlas_reference_active() {
        draw_references(menu, px, w, h);
    } else if menu.page == Page::Main {
        draw_overview(menu, av, px, w, h);
    } else if atlas::specialty(menu.page) {
        let b = g.body;
        let mut body = Vec::with_capacity(b.w*b.h);
        for y in 0..b.h {
            body.extend_from_slice(&px[(b.y+y)*w+b.x..(b.y+y)*w+b.x+b.w]);
        }
        match menu.page {
            Page::ContentSelector => content_selector::draw(&menu.content, &mut body, b.w, b.h, menu.language()),
            Page::CheatCatalog => content_selector::draw(&menu.cheats, &mut body, b.w, b.h, menu.language()),
            Page::DebugTool => debug_tool::draw(&menu.tool, &mut body, b.w, b.h, menu.language()),
            Page::DebugCatalog => debug_catalog::draw(menu, &mut body, b.w, b.h),
            Page::SaveStates | Page::ReplaceState => save_states::draw(menu, &mut body, b.w, b.h),
            Page::GameSaves | Page::SwitchGameSave | Page::ReplaceGameSave => {
                game_saves::draw(menu, &mut body, b.w, b.h)
            }
            Page::SaveBackups | Page::RestoreBackup => {
                save_backups::draw(menu, &mut body, b.w, b.h)
            }
            _ => {}
        }
        for y in 0..b.h {
            px[(b.y + y) * w + b.x..(b.y + y) * w + b.x + b.w]
                .copy_from_slice(&body[y * b.w..(y + 1) * b.w]);
        }
    } else {
        theme::draw_rows(menu, av, keys, ff, toggle, zin, zout, px, w, h);
    }
    if menu.page == Page::Debug && !menu.atlas.planned {
        draw_debug_details(menu, px, w, h);
    }
    if matches!(menu.page, Page::Debug | Page::CliReference) {
        draw_debug_tabs(menu, px, w, h);
    }
    if let Some(dd) = menu.dropdown.as_ref() {
        let mut c = Canvas::new(px, w, h);
        theme::draw_dropdown(menu, dd, &mut c, w, h);
        c.finish(px);
    }
}
fn draw_overview(menu: &Menu, av: &AvConfig, px: &mut [[u8; 4]], w: usize, h: usize) {
    let mut c = Canvas::new(px, w, h);
    let rects = overview_rects(w, h);
    let fs = (atlas::geometry(w, h).scale_milli as f32 / 1000. * 14.).max(10.);
    let slot = &menu.state_slots[menu.state_slot];
    let stamp = menu
        .state_times
        .get(menu.state_slot)
        .copied()
        .flatten()
        .and_then(atlas::timestamp_utc)
        .unwrap_or_else(|| menu.tr("Date unavailable").into());
    let labels = [
        menu.tr("Return to game").into(),
        menu.tr_format("Quick state {0} / {1} · {2} · {3}", &[("0", (menu.state_slot + 1).to_string()), ("1", (menu.state_slots.len()).to_string()), ("2", menu.tr(&slot.status).to_string()), ("3", (stamp).to_string())]),
        if slot.occupied {
            menu.tr("Overwrite")
        } else {
            menu.tr("Save")
        }
        .into(),
        menu.tr("Load").into(),
        menu.tr("Undo last operation").into(),
        format!("Volume  {}%", (av.audio_volume * 100.).round() as u32),
        menu.tr_format("Mute  {value}", &[("value", if av.audio_mute {menu.tr("ON")} else {menu.tr("OFF")}.into())]),
        menu.tr_format("Fast Forward  {value}%", &[("value", input_config::fast_forward::selected_percent(av.ff_speed).to_string())]),
        menu.tr_format("Fast Forward  {value}", &[("value", if menu.atlas.fast_active {menu.tr("ON")} else {menu.tr("OFF")}.into())]),
    ];
    for (i, (r, label)) in rects.into_iter().zip(labels).enumerate() {
        panel(&mut c, r, menu.selected == i && !menu.atlas.sidebar_focus);
        text(
            &mut c,
            r,
            &label,
            fs,
            i == 0,
            if i == 3 && !slot.occupied {
                MUTED
            } else {
                TEXT
            },
        );
    }
    let r = atlas::geometry(w, h).body;
    let y = rects[4].y + rects[4].h + 1;
    c.text(
        (r.x + 4) as f32,
        y as f32,
        &menu.standard_save_text(),
        MUTED,
        (fs * 0.66).max(8.),
        false,
        r.w.saturating_sub(8) as f32,
    );
    c.finish(px);
}
fn draw_debug_details(menu: &Menu, px: &mut [[u8; 4]], w: usize, h: usize) {
    let g = atlas::geometry(w, h);
    let mut c = Canvas::new(px, w, h);
    let fs = (g.scale_milli as f32 / 1000. * 13.).max(10.);
    panel(&mut c, g.detail, false);
    let row = menu.rows()[menu.selected.min(menu.rows().len() - 1)];
    let r = g.detail;
    c.text(
        (r.x + 12) as f32,
        (r.y + 12) as f32,
        row.localized_label(menu.language()),
        TEXT,
        fs,
        true,
        r.w.saturating_sub(24) as f32,
    );
    let description = if menu.atlas.selected_section() == 6 {
        menu.tr("Local catalog for read-only browsing. Entries disabled; decoding and application unavailable. No private codes are included.")
    } else if menu.debug_session {
        menu.tr("Isolated copy active. Confirm invokes existing handlers; validation and prerequisites are checked again before the operation.")
    } else {
        menu.tr("Guest operations require an isolated debug copy. Selecting an entry does not run commands. Open a session from Sessions and cases.")
    };
    let max_lines = r.h.saturating_sub(80) / (fs as usize + 5).max(1);
    for (i, line) in Canvas::wrap_text(menu.tr(description), fs, r.w.saturating_sub(24) as f32)
        .iter()
        .take(max_lines)
        .enumerate()
    {
        c.text(
            (r.x + 12) as f32,
            (r.y + 45 + i * (fs as usize + 5)) as f32,
            line,
            MUTED,
            fs,
            false,
            r.w.saturating_sub(24) as f32,
        );
    }
    c.finish(px);
}
fn draw_debug_tabs(menu: &Menu, px: &mut [[u8; 4]], w: usize, h: usize) {
    let g = atlas::geometry(w, h);
    let mut c = Canvas::new(px, w, h);
    let fs = (g.scale_milli as f32 / 1000. * 12.).max(9.);
    let half = g.debug_tabs.w / 2;
    for (i, label) in [format!("Menu/CLI · {}",menu.hint_button("North","Shift+Tab")), menu.tr("Debug V2 · reference").into()]
        .iter()
        .enumerate()
    {
        let rr = Rect {
            x: g.debug_tabs.x + i * half,
            w: half,
            ..g.debug_tabs
        };
        panel(&mut c, rr, usize::from(menu.atlas.planned) == i);
        text(&mut c, rr, label, fs * 0.9, false, TEXT);
    }
    c.finish(px);
}

fn draw_references(menu: &Menu, px: &mut [[u8; 4]], w: usize, h: usize) {
    let g = atlas::geometry(w, h);
    let mut c = Canvas::new(px, w, h);
    let fs = (g.scale_milli as f32 / 1000. * 13.).max(10.);
    panel(&mut c, g.list, false);
    panel(&mut c, g.detail, false);
    let items = menu.atlas_reference_items();
    let cursor = menu
        .atlas_reference_cursor()
        .min(items.len().saturating_sub(1));
    let (first, visible, pitch) = menu.atlas_reference_first(w, h);
    for (i, (label, _)) in items.iter().enumerate().skip(first).take(visible) {
        let r = Rect {
            x: g.list.x + 5,
            y: g.list.y + 6 + (i - first) * pitch,
            w: g.list.w.saturating_sub(16),
            h: pitch - 3,
        };
        panel(&mut c, r, i == cursor && !menu.atlas.sidebar_focus);
        text(&mut c, r, label, fs, false, TEXT);
    }
    if items.is_empty() {
        text(
            &mut c,
            g.list,
            menu.tr("No entries in this group"),
            fs,
            false,
            MUTED,
        );
    }
    if items.len() > visible && visible > 0 {
        let height = g.list.h.saturating_sub(12);
        let thumb = (height * visible / items.len()).max(8);
        let y =
            g.list.y + 6 + (height - thumb) * first / items.len().saturating_sub(visible).max(1);
        c.rect(
            (g.list.x + g.list.w - 7) as f32,
            y as f32,
            3.,
            thumb as f32,
            1.,
            BRASS,
            BRASS,
            BRASS,
        );
    }
    let r = g.detail;
    let line_h = fs as usize + 5;
    let description = items
        .get(cursor)
        .map(|(_, details)| details.as_str())
        .unwrap_or(menu.tr("No capability available. This view does not execute operations."));
    let lines: Vec<_> = description
        .split('\n')
        .flat_map(|p| Canvas::wrap_text(p, fs, r.w.saturating_sub(24) as f32))
        .collect();
    let capacity = r.h.saturating_sub(48) / line_h;
    let limit=lines.len().saturating_sub(capacity);
    menu.atlas.detail_limit.set(limit);
    let start=menu.atlas.detail_scroll.min(limit);
    for (i, line) in lines.iter().skip(start).take(capacity).enumerate() {
        c.text(
            (r.x + 12) as f32,
            (r.y + 10 + i * line_h) as f32,
            line,
            if i == 0 { TEXT } else { MUTED },
            fs,
            false,
            r.w.saturating_sub(24) as f32,
        );
    }
    c.text(
        (r.x + 10) as f32,
        (r.y + r.h.saturating_sub(25)) as f32,
        &menu.tr_format("↑↓ list · ←→ details · {0} back", &[("0", (menu.hint_button("East","Esc")).to_string())]),
        MUTED,
        fs * 0.72,
        false,
        r.w.saturating_sub(20) as f32,
    );
    c.finish(px);
}

#[cfg(test)]
mod brand_tests {
    use super::*;

    #[test]
    fn emblem_renders_within_sidebar_at_supported_sizes_and_transparency() {
        for (w, h) in [(480, 360), (640, 480), (854, 480), (1920, 1080)] {
            let geometry = atlas::geometry(w, h);
            let scale = geometry.scale_milli as f32 / 1000.;
            let background = [28, 56, 78, 255];
            let pixels = vec![background; w * h];
            let mut opaque = Canvas::new(&pixels, w, h);
            let mut transparent = Canvas::new(&pixels, w, h);
            canvas::with_panel_alpha(255, || brand_logo(&mut opaque, geometry.sidebar, scale));
            canvas::with_panel_alpha(0, || brand_logo(&mut transparent, geometry.sidebar, scale));
            assert_eq!(opaque.pix.data(), transparent.pix.data());
            let mut changed = 0;
            for (i, pixel) in opaque.pix.data().chunks_exact(4).enumerate() {
                if pixel != background {
                    let (x, y) = (i % w, i / w);
                    assert!(x >= geometry.sidebar.x && x < geometry.sidebar.x + geometry.sidebar.w);
                    assert!(y >= geometry.sidebar.y && y < geometry.domain.y);
                    changed += 1;
                }
            }
            assert!(changed > 50, "logo missing at {w}x{h}");
        }
    }
}
