//! Private offline PPU inspector. No emulation or guest-state writes.
use gba_core::Machine;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
const MAX_REPORTS: usize = 8;
fn hash(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
fn rgba(values: &[u16], transparent: bool) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| {
            if transparent && *v == 0xffff {
                [0, 0, 0, 0]
            } else {
                [
                    ((v & 31) * 255 / 31) as u8,
                    (((v >> 5) & 31) * 255 / 31) as u8,
                    (((v >> 10) & 31) * 255 / 31) as u8,
                    255,
                ]
            }
        })
        .collect()
}
pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let base = root.join("Data/config/graphics");
    if base.exists() && fs::read_dir(base).map_err(|e| e.to_string())?.count() >= MAX_REPORTS {
        return Err(
            "Eight graphical reports already exist. Retain or archive them before another capture."
                .into(),
        );
    }
    Ok(())
}
pub(crate) struct Frontend {
    pub width: usize,
    pub height: usize,
    pub world: Vec<[u8; 4]>,
    pub composed: Vec<[u8; 4]>,
    pub settings: Value,
}
fn prior(root: &Path, rom: &str) -> Result<Option<(String, Value)>, String> {
    let base = root.join("Data/config/graphics");
    if !base.exists() {
        return Ok(None);
    }
    let mut paths = fs::read_dir(&base)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths.into_iter().rev() {
        let file = path.join("graphics.json");
        if !file.is_file() {
            continue;
        }
        let raw = crate::diagnostics::read(&file, 2 * 1024 * 1024)?;
        let report: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        if report["schema"] != 1
            || report["kind"] != "ffta-graphics-inspection"
            || report["rom_sha256"] != rom
        {
            return Err("Previous graphics report ROM/schema mismatch".into());
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid prior report name")?;
        if !name.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return Err("Invalid prior graphics report identity".into());
        }
        return Ok(Some((name.into(), report)));
    }
    Ok(None)
}
fn difference(a: &[u8], b: &[u8]) -> Value {
    let first: Vec<_> = a
        .iter()
        .zip(b)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(16)
        .map(|(offset, (a, b))| json!({"offset":offset,"previous":a,"current":b}))
        .collect();
    json!({"previous_bytes":a.len(),"current_bytes":b.len(),"different_bytes":a.iter().zip(b).filter(|(a,b)|a!=b).count()+a.len().abs_diff(b.len()),"first":first})
}
fn compare(root: &Path, rom: &str, current: &BTreeMap<String, Vec<u8>>) -> Result<Value, String> {
    let Some((name, previous)) = prior(root, rom)? else {
        return Err("No prior complete graphics report to compare".into());
    };
    let mut diffs = BTreeMap::new();
    for (file, bytes) in current {
        let old = crate::diagnostics::read(
            &root.join("Data/config/graphics").join(&name).join(file),
            8 * 1024 * 1024,
        )?;
        if previous["raw_hashes"][file] != hash(&old) {
            return Err(format!("Previous graphics payload changed: {file}"));
        }
        diffs.insert(file.clone(), difference(&old, bytes));
    }
    for file in ["central.png", "wide.png", "frontend.png"] {
        let old = crate::diagnostics::read(
            &root.join("Data/config/graphics").join(&name).join(file),
            12 * 1024 * 1024,
        )?;
        if previous["image_hashes"][file] != hash(&old) {
            return Err(format!("Previous graphics image changed: {file}"));
        }
    }
    Ok(
        json!({"report":name,"frame":previous["frame"],"clock":previous["clock"],"state_sha256":previous["state_sha256"],"width":previous["width"],"height":previous["height"],"frontend":previous["frontend"],"raw_differences":diffs,"alignment":"Images comparable only with matching view/settings; offsets are buffer bytes, not ROM addresses"}),
    )
}
fn write_png(dir: &Path, name: &str, bytes: &[u8], w: usize, h: usize) -> Result<(), String> {
    if w == 0
        || h == 0
        || w.checked_mul(h).and_then(|n| n.checked_mul(4)) != Some(bytes.len())
        || w > 1600
        || h > 1024
    {
        return Err("Invalid graphics image dimensions".into());
    }
    image::save_buffer(
        dir.join(name),
        bytes,
        w as u32,
        h as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}
/// Publishes metadata last. Failed bundles are retained and count toward capacity.
pub(crate) fn capture(
    root: &Path,
    m: &Machine,
    frontend: Frontend,
    compare_last: bool,
) -> Result<String, String> {
    capacity(root)?;
    if frontend.width == 0
        || frontend.height == 0
        || frontend.width > 1600
        || frontend.height > 1000
        || frontend.world.len() != frontend.width * frontend.height
        || frontend.composed.len() != frontend.world.len()
    {
        return Err("Frontend capture dimensions out of bounds".into());
    }
    let rom = hash(&m.bus.rom);
    let state = hash(&m.save_state()?);
    let w = gba_core::mem::render_width();
    let h = gba_core::mem::render_height();
    let wide = m
        .bus
        .wide_framebuffer
        .get(..w * h)
        .ok_or("Wide buffer shape mismatch")?;
    let frozen = gba_core::ppu::inspect::capture(&m.bus)?;
    let raw = BTreeMap::from([
        ("vram.bin".into(), m.bus.vram.clone()),
        ("oam.bin".into(), m.bus.oam.clone()),
        ("palette.bin".into(), m.bus.palette.clone()),
        ("io.bin".into(), m.bus.io.clone()),
        ("central.rgb555".into(), words(&m.bus.framebuffer)),
        ("wide.rgb555".into(), words(wide)),
    ]);
    let comparison = if compare_last {
        compare(root, &rom, &raw)?
    } else {
        Value::Null
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let name = format!("{stamp}-{}", std::process::id());
    let dir = root.join("Data/config/graphics").join(&name);
    fs::create_dir_all(dir.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    for (file, bytes) in &raw {
        fs::write(dir.join(file), bytes).map_err(|e| e.to_string())?;
    }
    write_png(
        &dir,
        "central.png",
        &rgba(&m.bus.framebuffer, false),
        240,
        160,
    )?;
    write_png(&dir, "wide.png", &rgba(wide, false), w, h)?;
    let front: Vec<u8> = frontend.composed.iter().flatten().copied().collect();
    let world: Vec<u8> = frontend.world.iter().flatten().copied().collect();
    write_png(
        &dir,
        "frontend.png",
        &front,
        frontend.width,
        frontend.height,
    )?;
    write_png(
        &dir,
        "frontend-world.png",
        &world,
        frontend.width,
        frontend.height,
    )?;
    let mut image_hashes = BTreeMap::new();
    for file in [
        "central.png",
        "wide.png",
        "frontend.png",
        "frontend-world.png",
    ] {
        image_hashes.insert(
            file.to_string(),
            hash(&fs::read(dir.join(file)).map_err(|e| e.to_string())?),
        );
    }
    for layer in &frozen.layers {
        let file = format!("{}.png", layer.name);
        write_png(
            &dir,
            &file,
            &rgba(&layer.pixels, true),
            layer.width,
            layer.height,
        )?;
        image_hashes.insert(
            file,
            hash(&fs::read(dir.join(format!("{}.png", layer.name))).map_err(|e| e.to_string())?),
        );
    }
    let mut count = 0;
    let mut bbox = [frontend.width, frontend.height, 0, 0];
    for (i, (a, b)) in frontend.world.iter().zip(&frontend.composed).enumerate() {
        if a != b {
            count += 1;
            let (x, y) = (i % frontend.width, i / frontend.width);
            bbox[0] = bbox[0].min(x);
            bbox[1] = bbox[1].min(y);
            bbox[2] = bbox[2].max(x + 1);
            bbox[3] = bbox[3].max(y + 1);
        }
    }
    if hash(&m.save_state()?) != state {
        return Err("Graphics inspection changed guest state".into());
    }
    let report = json!({"schema":1,"kind":"ffta-graphics-inspection","report":name,"frame":m.bus.frames,"clock":m.bus.clock,"frame_ready":m.bus.frame_ready,"state_sha256":state,"rom_sha256":rom,
        "runtime_sha256":hash(&crate::diagnostics::read(&std::env::current_exe().map_err(|e|e.to_string())?,256*1024*1024)?),
        "width":w,"height":h,"extra":gba_core::mem::extra(),"extra_y":gba_core::mem::extra_y(),"frontend":{"width":frontend.width,"height":frontend.height,"settings":frontend.settings,"ui_changed_pixels":count,"ui_changed_bounds":if count==0{Value::Null}else{json!(bbox)},"scope":"CPU preview sampler + color/temporal + native UI at captured window dimensions; excludes pause/FPS and GPU grid/filter/display effects"},
        "frozen":frozen,"vram":m.bus.vram,"palette":m.bus.palette,"oam":m.bus.oam,"io":m.bus.io,
        "raw_hashes":raw.iter().map(|(n,b)|(n.clone(),hash(b))).collect::<BTreeMap<_,_>>(),"image_hashes":image_hashes,"comparison":comparison,
        "scope":["Central/wide buffers are saved outputs, possibly containing a partial frame; no scanline register history is stored.","Layer/object/window projections use current frozen registers and existing readonly helpers. They do not rerender the guest or prove which layer won in an earlier output pixel.","Affine views show raw maps, without inventing earlier affine references. OAM textures in browser omit screen transforms.","Pixel provenance is exact for decoded backing data only. VRAM-to-ROM transfer provenance and final pixel ownership are unknown; no guessed ROM link.","Private copyrighted pixel/buffer data; never commit/package. At most eight reports; no automatic deletion or overwrite."]});
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Graphics metadata exceeds 2 MiB".into());
    }
    let html = include_str!("debug_graphics/report.html").replace(
        "__GRAPHICS__",
        &String::from_utf8(bytes.clone())
            .map_err(|e| e.to_string())?
            .replace('<', "\\u003c"),
    );
    fs::write(dir.join("index.html"), html).map_err(|e| e.to_string())?;
    fs::write(dir.join("graphics.json"), bytes).map_err(|e| e.to_string())?;
    Ok(format!("Data/config/graphics/{name}/index.html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn front() -> Frontend {
        Frontend {
            width: 2,
            height: 2,
            world: vec![[0, 0, 0, 255]; 4],
            composed: vec![[1, 0, 0, 255]; 4],
            settings: json!({"view":[0,0,240,160]}),
        }
    }
    #[test]
    fn saved_rgb555_and_layer_transparency_have_distinct_encoding() {
        assert_eq!(rgba(&[0xffff], false), [255, 255, 255, 255]);
        assert_eq!(rgba(&[0xffff], true), [0, 0, 0, 0]);
    }
    #[test]
    fn readonly_bundle_and_known_palette_difference_are_exact() {
        gba_core::mem::set_extra(0);
        gba_core::mem::set_extra_y(0);
        let root = tempfile::tempdir().unwrap();
        let mut m = Machine::new(vec![0; 192]);
        let before = m.save_state().unwrap();
        let first = capture(root.path(), &m, front(), false).unwrap();
        assert_eq!(before, m.save_state().unwrap());
        let file = root.path().join(first).with_file_name("graphics.json");
        let initial: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(initial["frontend"]["ui_changed_pixels"], 4);
        m.bus.palette[2] = 31;
        let before = m.save_state().unwrap();
        let next = capture(root.path(), &m, front(), true).unwrap();
        assert_eq!(before, m.save_state().unwrap());
        let second: Value = serde_json::from_slice(
            &fs::read(root.path().join(next).with_file_name("graphics.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            second["comparison"]["raw_differences"]["palette.bin"]["different_bytes"],
            1
        );
        assert_eq!(
            second["comparison"]["raw_differences"]["vram.bin"]["different_bytes"],
            0
        );
        assert!(file.exists());
        assert!(
            root.path()
                .join("Data/config/graphics")
                .read_dir()
                .unwrap()
                .count()
                == 2
        );
    }
    #[test]
    fn capacity_and_modified_previous_payload_refuse_without_loss() {
        gba_core::mem::set_extra(0);
        gba_core::mem::set_extra_y(0);
        let root = tempfile::tempdir().unwrap();
        let m = Machine::new(vec![0; 192]);
        assert!(capture(root.path(), &m, front(), true).is_err());
        let path = capture(root.path(), &m, front(), false).unwrap();
        let parent = root.path().join(path).parent().unwrap().to_owned();
        fs::write(parent.join("vram.bin"), [0]).unwrap();
        assert!(capture(root.path(), &m, front(), true).is_err());
        for i in 0..7 {
            fs::create_dir(root.path().join(format!("Data/config/graphics/{i}"))).unwrap();
        }
        assert!(capacity(root.path()).is_err());
        assert!(parent.join("graphics.json").is_file());
    }
}
