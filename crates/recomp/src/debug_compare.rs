//! First differing completed frame, using independent guest Machines in one worker.
//! Reuses the established raw/typed/native paired-validation execution contract.
use gba_core::Machine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub frames: u64,
    pub backend: String,
    pub audio: String,
}
pub(crate) struct Request {
    pub settings: Settings,
    pub dataset: Option<PathBuf>,
}
pub(crate) fn validate(s: &Settings, frames: u64, native: bool) -> Result<(), String> {
    if s.frames == 0
        || s.frames > frames
        || s.frames > 3600
        || !["interp", "native"].contains(&s.backend.as_str())
        || !["preserve", "arm-hle"].contains(&s.audio.as_str())
        || (s.backend == "native") != native
    {
        return Err("Comparison needs 1..min(recorded frames,3600), interp/native with matching library, and preserve/arm-hle audio".into());
    }
    Ok(())
}
pub(crate) fn prepare(r: &Request, frames: u64, native: bool) -> Result<Settings, String> {
    let mut s = r.settings.clone();
    if s.frames == u64::MAX {
        s.frames = frames.min(3600);
    }
    validate(&s, frames, native)?;
    Ok(s)
}
fn hash(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
fn u16s(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn i16s(values: &[i16]) -> Vec<u8> {
    values.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn f32s(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|x| x.to_bits().to_le_bytes())
        .collect()
}
fn fifos(values: &[(i8, u32)]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&(s, p)| std::iter::once(s as u8).chain(p.to_le_bytes()))
        .collect()
}
#[derive(Clone, Serialize)]
struct Observation {
    clock: u64,
    guest_frame: u64,
    frame_ready: bool,
    hashes: BTreeMap<String, String>,
    bytes: BTreeMap<String, usize>,
    audio_counts: BTreeMap<String, usize>,
    channels_enabled: bool,
    hle_present: bool,
}
fn observe(m: &Machine) -> Result<(Observation, Vec<u8>), String> {
    let state = m.save_state()?;
    let components = BTreeMap::from([
        (
            "cpu",
            serde_json::to_vec(&m.cpu).map_err(|e| e.to_string())?,
        ),
        ("central", u16s(&m.bus.framebuffer)),
        ("wide_allocated", u16s(&m.bus.wide_framebuffer)),
        ("oam", m.bus.oam.clone()),
        ("mixed", i16s(&m.bus.audio_buf)),
        ("psg", i16s(&m.bus.psg_tap)),
        ("hle", f32s(&m.bus.hle_tap)),
        ("fifo0", fifos(&m.bus.fifo_tap[0])),
        ("fifo1", fifos(&m.bus.fifo_tap[1])),
    ]);
    let mut hashes: BTreeMap<_, _> = components
        .iter()
        .map(|(&k, v)| (k.into(), hash(v)))
        .collect();
    hashes.insert("full_state".into(), hash(&state));
    let mut bytes: BTreeMap<_, _> = components
        .iter()
        .map(|(&k, v)| (k.into(), v.len()))
        .collect();
    bytes.insert("full_state".into(), state.len());
    let audio_counts = BTreeMap::from([
        ("mixed_i16_values".into(), m.bus.audio_buf.len()),
        ("psg_i16_values".into(), m.bus.psg_tap.len()),
        ("hle_f32_values".into(), m.bus.hle_tap.len()),
        ("fifo0_events".into(), m.bus.fifo_tap[0].len()),
        ("fifo1_events".into(), m.bus.fifo_tap[1].len()),
    ]);
    Ok((
        Observation {
            clock: m.bus.clock,
            guest_frame: m.bus.frames,
            frame_ready: m.bus.frame_ready,
            hashes,
            bytes,
            audio_counts,
            channels_enabled: m.bus.tap_channels,
            hle_present: m.bus.mp2k.is_some() || m.bus.gax.is_some() || m.bus.rdrv.is_some(),
        },
        state,
    ))
}
fn changed(a: &Observation, b: &Observation) -> Vec<String> {
    let mut names: Vec<_> = a
        .hashes
        .keys()
        .chain(b.hashes.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|k| a.hashes.get(*k) != b.hashes.get(*k))
        .cloned()
        .collect();
    if a.clock != b.clock {
        names.push("clock".into());
    }
    if a.guest_frame != b.guest_frame {
        names.push("guest_frame".into());
    }
    if a.frame_ready != b.frame_ready {
        names.push("frame_ready".into());
    }
    names
}
fn drain(m: &mut Machine) {
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.hle_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
}
struct Decision {
    outcome: &'static str,
    message: String,
    completed: u64,
    previous: Option<(Observation, Vec<u8>)>,
    left: Observation,
    right: Observation,
    audio_totals: [BTreeMap<String, usize>; 2],
    native_dispatches: u64,
    native_fallbacks: u64,
}
fn loop_compare(
    left: &mut Machine,
    right: &mut Machine,
    s: &Settings,
    input: &crate::InputScript,
    mut candidate: impl FnMut(&mut Machine) -> Result<(u64, u64), String>,
    mut control: impl FnMut(u64) -> Option<(&'static str, String)>,
) -> Result<Decision, String> {
    let mut completed = 0;
    let mut previous = None;
    let mut totals: [BTreeMap<String, usize>; 2] = [BTreeMap::new(), BTreeMap::new()];
    let (mut dispatches, mut fallbacks) = (0, 0);
    let (mut a, mut state) = observe(left)?;
    let (mut b, _) = observe(right)?;
    loop {
        let difference = changed(&a, &b);
        let finish = if !difference.is_empty() {
            Some((
                "diverged",
                format!(
                    "First difference at completed frame {completed}: {}",
                    difference.join(", ")
                ),
            ))
        } else if let Some(result) = control(completed) {
            Some(result)
        } else if completed == s.frames {
            Some(("matched",format!("All declared components match through {completed} completed frames; recorded endpoint not certified.")))
        } else {
            None
        };
        if let Some((outcome, message)) = finish {
            return Ok(Decision {
                outcome,
                message,
                completed,
                previous,
                left: a,
                right: b,
                audio_totals: totals,
                native_dispatches: dispatches,
                native_fallbacks: fallbacks,
            });
        }
        previous = Some((a, state));
        // Observe origin/current queues first, then drain both frontend streams
        // before the NEXT frame. Never carry samples across compared windows.
        drain(left);
        drain(right);
        for m in [&mut *left, &mut *right] {
            m.bus.keys = input.keys_at(completed);
            input.apply_mouse(&mut m.bus, completed)?;
        }
        let before = [left.bus.frames, right.bus.frames];
        left.run_frame(5_000_000);
        let (d, f) = candidate(right)?;
        dispatches += d;
        fallbacks += f;
        if left.bus.frames != before[0] + 1
            || right.bus.frames != before[1] + 1
            || !left.bus.frame_ready
            || !right.bus.frame_ready
        {
            let (a, _) = observe(left)?;
            let (b, _) = observe(right)?;
            return Ok(Decision {
                outcome: "error",
                message:
                    "Incomplete frame: step guard exceeded; no aligned first-difference claim."
                        .into(),
                completed,
                previous,
                left: a,
                right: b,
                audio_totals: totals,
                native_dispatches: dispatches,
                native_fallbacks: fallbacks,
            });
        }
        completed += 1;
        (a, state) = observe(left)?;
        (b, _) = observe(right)?;
        for (index, o) in [&a, &b].into_iter().enumerate() {
            for (k, v) in &o.audio_counts {
                *totals[index].entry(k.clone()).or_default() += v;
            }
        }
    }
}

pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let p = root.join("comparisons");
    match fs::symlink_metadata(&p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
        Ok(m) if !m.is_dir() || m.file_type().is_symlink() => {
            return Err("Comparison output must be a real local directory".into())
        }
        _ => {}
    }
    if fs::read_dir(p).map_err(|e| e.to_string())?.count() >= 8 {
        return Err(
            "Eight comparison reports retained; archive explicitly or choose a new profile".into(),
        );
    }
    Ok(())
}
#[derive(Serialize)]
pub(crate) struct Outcome {
    pub outcome: String,
    pub message: String,
    pub completed: u64,
    pub report: String,
    pub state_sha256: String,
    pub clock: u64,
}
pub(crate) fn run(
    root: &Path,
    run: &str,
    s: &Settings,
    origin: Value,
    right: &mut Machine,
    start: &[u8],
    input: &crate::InputScript,
    table: Option<&crate::BlockTable>,
    control: impl FnMut(u64) -> Option<(&'static str, String)>,
) -> Result<Outcome, String> {
    let mut left = crate::make_machine(
        right.bus.rom.clone(),
        right.bus.real_bios.then_some(right.bus.bios.as_slice()),
    );
    left.load_state(start)?;
    let initial = changed(&observe(&left)?.0, &observe(right)?.0);
    if !initial.is_empty() {
        return Err("Comparison origins differ before audio setup".into());
    }
    let setup = if s.audio == "arm-hle" {
        left.bus.tap_channels = true;
        right.bus.tap_channels = true;
        json!({"left":crate::arm_audio_hle(&mut left,None),"right":crate::arm_audio_hle(right,None)})
    } else {
        json!({"left":"source settings retained","right":"source settings retained"})
    };
    let decision = loop_compare(
        &mut left,
        right,
        s,
        input,
        |m| {
            if s.backend == "native" {
                let table = table.ok_or("Native comparison needs an existing library")?;
                let ptr = m as *mut Machine as *mut std::ffi::c_void;
                Ok(crate::run_frame_native(m, table, ptr, 5_000_000))
            } else {
                m.run_frame(5_000_000);
                Ok((0, 0))
            }
        },
        control,
    )?;
    let report = publish(root, run, s, origin, setup, &decision, &left, right)?;
    Ok(Outcome {
        outcome: decision.outcome.into(),
        message: decision.message,
        completed: decision.completed,
        report,
        state_sha256: decision.right.hashes["full_state"].clone(),
        clock: decision.right.clock,
    })
}

fn byte_difference(a: &[u8], b: &[u8]) -> Value {
    let mut count = 0;
    let mut first = Vec::new();
    for i in 0..a.len().max(b.len()) {
        if a.get(i) != b.get(i) {
            count += 1;
            if first.len() < 16 {
                first.push(json!({"encoded_offset":i,"left":a.get(i),"right":b.get(i)}));
            }
        }
    }
    json!({"left_bytes":a.len(),"right_bytes":b.len(),"left_sha256":hash(a),"right_sha256":hash(b),"different_encoded_bytes":count,"first":first})
}
fn indexed<T: PartialEq + Serialize>(a: &[T], b: &[T]) -> Value {
    let mut count = 0;
    let mut first = Vec::new();
    for i in 0..a.len().max(b.len()) {
        if a.get(i) != b.get(i) {
            count += 1;
            if first.len() < 16 {
                first.push(json!({"index":i,"left":a.get(i),"right":b.get(i)}));
            }
        }
    }
    json!({"left_values":a.len(),"right_values":b.len(),"different_values":count,"first":first})
}
fn field_differences(left: &Machine, right: &Machine) -> Result<Value, String> {
    let a = gba_core::state_diagnostics::fields(left)?;
    let b = gba_core::state_diagnostics::fields(right)?;
    let mut different = Vec::new();
    let mut equal = 0;
    for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
        if a.get(key) == b.get(key) {
            equal += 1;
        } else {
            different.push(json!({"field":key,"difference":byte_difference(a.get(key).map_or(&[],Vec::as_slice),b.get(key).map_or(&[],Vec::as_slice))}));
        }
    }
    let registers: Vec<_> = left
        .cpu
        .regs
        .iter()
        .zip(&right.cpu.regs)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| json!({"register":format!("r{i}"),"left":a,"right":b}))
        .collect();
    Ok(
        json!({"encoding":"Named current-schema CPU/Bus fields, postcard value bytes; exact float bits. Encoded offsets are NOT guest addresses or whole-snapshot offsets.","equal_fields":equal,"different":different,"registers":registers}),
    )
}
fn png(path: &Path, pixels: &[u16], width: usize, height: usize) -> Result<(), String> {
    if width.checked_mul(height).is_none_or(|n| n > pixels.len()) {
        return Err("Framebuffer shape exceeds buffer".into());
    }
    let rgb: Vec<u8> = pixels[..width * height]
        .iter()
        .flat_map(|&v| {
            [
                ((v & 31) * 255 / 31) as u8,
                (((v >> 5) & 31) * 255 / 31) as u8,
                (((v >> 10) & 31) * 255 / 31) as u8,
            ]
        })
        .collect();
    image::save_buffer(
        path,
        &rgb,
        width as u32,
        height as u32,
        image::ColorType::Rgb8,
    )
    .map_err(|e| e.to_string())
}
fn dump(dir: &Path, label: &str, m: &Machine) -> Result<Value, String> {
    fs::write(dir.join(format!("{label}.ffstate")), m.save_state()?).map_err(|e| e.to_string())?;
    let width = gba_core::mem::render_width();
    let height = gba_core::mem::render_height();
    png(
        &dir.join(format!("{label}-central.png")),
        &m.bus.framebuffer,
        240,
        160,
    )?;
    png(
        &dir.join(format!("{label}-wide.png")),
        &m.bus.wide_framebuffer,
        width,
        height,
    )?;
    let mut files = BTreeMap::new();
    for (name, bytes) in [
        ("mixed.pcm16le", i16s(&m.bus.audio_buf)),
        ("psg.pcm16le", i16s(&m.bus.psg_tap)),
        ("hle.f32le", f32s(&m.bus.hle_tap)),
        ("fifo0.events", fifos(&m.bus.fifo_tap[0])),
        ("fifo1.events", fifos(&m.bus.fifo_tap[1])),
    ] {
        if !bytes.is_empty() {
            let file = format!("{label}-{name}");
            fs::write(dir.join(&file), bytes).map_err(|e| e.to_string())?;
            files.insert(name, file);
        }
    }
    Ok(
        json!({"snapshot":format!("{label}.ffstate"),"central":format!("{label}-central.png"),"wide":format!("{label}-wide.png"),"wide_width":width,"wide_height":height,"audio":files}),
    )
}
fn publish(
    root: &Path,
    run: &str,
    s: &Settings,
    origin: Value,
    audio_setup: Value,
    d: &Decision,
    left: &Machine,
    right: &Machine,
) -> Result<String, String> {
    if run.is_empty()
        || run.len() > 64
        || !run.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid comparison run".into());
    }
    capacity(root)?;
    let parent = root.join("comparisons");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let destination = parent.join(run);
    if fs::symlink_metadata(&destination).is_ok() {
        return Err("Comparison run already exists".into());
    }
    let staging = tempfile::Builder::new()
        .prefix(".comparison-")
        .tempdir_in(&parent)
        .map_err(|e| e.to_string())?;
    let (artifact, fields) = if d.outcome == "diverged" {
        let mut artifacts = json!({"left":dump(staging.path(),"left",left)?,"right":dump(staging.path(),"right",right)?});
        if let Some((previous, state)) = &d.previous {
            let mut m = crate::make_machine(
                left.bus.rom.clone(),
                left.bus.real_bios.then_some(left.bus.bios.as_slice()),
            );
            m.load_state(state)?;
            artifacts["previous"] = dump(staging.path(), "previous", &m)?;
            artifacts["previous"]["guest_frame"] = previous.guest_frame.into();
        }
        (artifacts, field_differences(left, right)?)
    } else {
        (Value::Null, Value::Null)
    };
    let report = json!({"schema":1,"kind":"ffta-first-divergence","run":run,"outcome":d.outcome,"message":d.message,"origin":origin,
        "settings":s,"reference":"interpreter/raw","candidate_backend":s.backend,"completed":d.completed,"limit":s.frames,
        "first_difference":if d.outcome=="diverged"{Some(d.completed)}else{None},"last_matching":if d.outcome!="error" && changed(&d.left,&d.right).is_empty(){Some(&d.left)}else{d.previous.as_ref().map(|(m,_)|m)},
        "left":d.left,"right":d.right,"different_components":changed(&d.left,&d.right),"audio_setup":audio_setup,"audio_totals":d.audio_totals,
        "value_differences":if d.outcome=="diverged"{json!({"ewram":indexed(&left.bus.ewram,&right.bus.ewram),"iwram":indexed(&left.bus.iwram,&right.bus.iwram),"oam":indexed(&left.bus.oam,&right.bus.oam),
            "central_rgb555":indexed(&left.bus.framebuffer,&right.bus.framebuffer),"wide_allocated_rgb555":indexed(&left.bus.wide_framebuffer,&right.bus.wide_framebuffer),
            "mixed_i16":indexed(&left.bus.audio_buf,&right.bus.audio_buf),"psg_i16":indexed(&left.bus.psg_tap,&right.bus.psg_tap),
            "hle_f32_bits":indexed(&left.bus.hle_tap.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),&right.bus.hle_tap.iter().map(|v|v.to_bits()).collect::<Vec<_>>()),
            "fifo0_events":indexed(&left.bus.fifo_tap[0],&right.bus.fifo_tap[0]),"fifo1_events":indexed(&left.bus.fifo_tap[1],&right.bus.fifo_tap[1])})}else{Value::Null},
        "candidate_native_dispatches":d.native_dispatches,"candidate_native_fallbacks":d.native_fallbacks,"fields":fields,"artifacts":artifact,
        "scope":["Every aligned completed frame from common origin; first differing boundary, not first instruction.","Full snapshot bytes (ROM/BIOS-bound), clock/frame/ready, complete CPU, central RGB555, entire allocated wide buffer, OAM, mixed/PSG/HLE/FIFO exact stream bits/events before frontend drain.","RGB555 images show the active region; inactive wide allocation remains covered by state/component hashes.","Audio counts are values/events, not bytes or audible quality. Empty/disabled taps are not proof of audio coverage.","Named fields follow current snapshot Serialize and postcard values, including private members and float bits; encoded offsets are not guest addresses.","Only selected modes/dataset/input/view/audio setup and bounded frames. No recorded-endpoint, full-ROM or new DS/native-parity acceptance.","All snapshots, images, byte deltas and audio payloads are private local evidence."]});
    let payload = serde_json::to_string(&report).map_err(|e| e.to_string())?;
    fs::write(staging.path().join("comparison.json"), payload.as_bytes())
        .map_err(|e| e.to_string())?;
    fs::write(
        staging.path().join("index.html"),
        include_str!("debug_compare/report.html")
            .replace("__COMPARISON__", &payload.replace('<', "\\u003c")),
    )
    .map_err(|e| e.to_string())?;
    let path = staging.keep();
    fs::rename(&path, &destination).map_err(|e| {
        format!(
            "Comparison publication failed, retained {}: {e}",
            path.display()
        )
    })?;
    Ok(format!("comparisons/{run}/index.html"))
}

pub(crate) fn cmd(args: &[String]) -> u8 {
    let error = |message: String| {
        println!(
            "{}",
            json!({"schema":1,"kind":"ffta-debug-compare-result","outcome":"incompatible","message":message})
        );
        3
    };
    if args.first().map(String::as_str) == Some("create") {
        return match create_cmd(&args[1..]) {
            Ok(p) => {
                println!(
                    "{}",
                    json!({"schema":1,"kind":"ffta-debug-compare-created","profile":p})
                );
                0
            }
            Err(e) => error(e),
        };
    }
    if matches!(args.first().map(String::as_str), Some("run" | "cancel")) && args.len() >= 2 {
        if let Err(e) = crate::debug_replay::Job::require_comparison(Path::new(&args[1])) {
            return error(e);
        }
        return crate::debug_replay::cmd(args);
    }
    error("Use debug-compare create [replay create options] [--frames N] [--backend interp|native] [--dataset DIR] [--audio preserve|arm-hle], then run/cancel replay.json".into())
}
fn create_cmd(args: &[String]) -> Result<PathBuf, String> {
    create_from_menu(args,false)
}
pub(crate) fn create_from_menu(args:&[String],validate_only:bool)->Result<PathBuf,String>{
    let (mut frames, mut backend, mut dataset, mut audio) =
        (u64::MAX, None, None, "preserve".to_owned());
    let mut rest = Vec::new();
    let mut library = false;
    let mut it = args.iter();
    while let Some(k) = it.next() {
        let v = it.next().ok_or_else(|| format!("{k} needs a value"))?;
        match k.as_str() {
            "--frames" => frames = v.parse().map_err(|_| "Invalid comparison frames")?,
            "--backend" => backend = Some(v.clone()),
            "--dataset" => dataset = Some(PathBuf::from(v)),
            "--audio" => audio = v.clone(),
            _ => {
                library |= k == "--library";
                rest.push(k.clone());
                rest.push(v.clone());
            }
        }
    }
    crate::debug_replay::create_dispatch(
        &rest,None,None,
        Some(&Request {
            settings: Settings {
                frames,
                backend: backend
                    .unwrap_or_else(|| if library { "native" } else { "interp" }.into()),
                audio,
            },
            dataset,
        }),validate_only,
    )
}
#[cfg(test)]
mod tests;
