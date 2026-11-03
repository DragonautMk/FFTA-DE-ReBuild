//! Bounded, explicit guest/producer/backend-buffer observation. No callback I/O.
use gba_core::Machine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const RAW_LIMIT: usize = 2 * 1024 * 1024;
const OUTPUT_VALUES: usize = 512 * 1024;
pub(crate) const STREAMS: [&str; 5] = ["mixed", "psg", "hle", "fifo0", "fifo1"];
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub frames: u32,
    pub streams: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            frames: 60,
            streams: STREAMS.iter().map(|s| s.to_string()).collect(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=180).contains(&self.frames) || self.streams.is_empty() || self.streams.len() > 5 {
            return Err("Audio capture needs1..180frames and1..5streams".into());
        }
        for (i, s) in self.streams.iter().enumerate() {
            if !STREAMS.contains(&s.as_str()) || self.streams[..i].contains(s) {
                return Err("Choose distinct mixed,psg,hle,fifo0,fifo1 streams".into());
            }
        }
        Ok(())
    }
}
pub(crate) trait OutputSample: Copy {
    const FORMAT: &'static str;
    fn normalized(self) -> f32;
}
impl OutputSample for f32 {
    const FORMAT: &'static str = "f32";
    fn normalized(self) -> f32 {
        self
    }
}
impl OutputSample for i16 {
    const FORMAT: &'static str = "i16";
    fn normalized(self) -> f32 {
        self as f32 / 32768.
    }
}
impl OutputSample for u16 {
    const FORMAT: &'static str = "u16";
    fn normalized(self) -> f32 {
        (self as f32 - 32768.) / 32768.
    }
}

#[derive(Clone, Copy, Serialize, Default)]
pub(crate) struct Delivery {
    pub epoch: u64,
    pub consumed: u64,
    pub holds: u64,
    pub drops: u64,
    pub temporary_mute: bool,
    pub user_mute: bool,
    pub target_gain: f32,
    pub current_gain: f32,
    pub enhanced: bool,
    pub hle: bool,
    #[serde(rename = "requested_speed")]
    pub speed: f64,
    pub normal: bool,
    pub fast_estimate: Option<f64>,
    pub fast_speed: Option<f64>,
    pub fast_ready: Option<bool>,
    pub queue_values: [usize; 5],
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct OutputSpec {
    rate: u32,
    channels: usize,
    format: &'static str,
}
#[derive(Serialize)]
struct Block {
    offset: usize,
    values: usize,
    published_guest_frame: u64,
    context: Delivery,
}
#[derive(Serialize)]
struct Reset {
    epoch: u64,
    published_guest_frame: u64,
    output_offset: usize,
}
struct Raw {
    name: String,
    data: Vec<u8>,
    unit: usize,
    truncated: bool,
    observed_values: u64,
}
impl Raw {
    fn new(name: &str) -> Result<Self, String> {
        let mut data = Vec::new();
        data.try_reserve_exact(RAW_LIMIT)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            name: name.into(),
            data,
            unit: unit(name),
            truncated: false,
            observed_values: 0,
        })
    }
    fn append(&mut self, bytes: impl IntoIterator<Item = u8>, values: usize) {
        self.observed_values = self.observed_values.saturating_add(values as u64);
        let remaining = (RAW_LIMIT - self.data.len()) / self.unit * self.unit;
        self.truncated |= values.saturating_mul(self.unit) > remaining;
        self.data.extend(bytes.into_iter().take(remaining));
    }
}
fn unit(name: &str) -> usize {
    match name {
        "mixed" | "psg" => 2,
        "hle" | "callback" => 4,
        _ => 5,
    }
}
pub(crate) struct Capture {
    settings: Settings,
    origin: Value,
    origin_frame: u64,
    started: Instant,
    active: bool,
    reason: &'static str,
    raw: Vec<Raw>,
    frames: Vec<Value>,
    published: u64,
    output: Vec<f32>,
    spec: Option<OutputSpec>,
    blocks: Vec<Block>,
    resets: Vec<Reset>,
    output_stop: Option<&'static str>,
    omitted_resets: u64,
}
impl Capture {
    pub fn enabled(&self) -> bool {
        self.active
    }
    pub fn start(settings: Settings, origin: Value, frame: u64) -> Result<Self, String> {
        settings.validate()?;
        let raw = settings
            .streams
            .iter()
            .map(|s| Raw::new(s))
            .collect::<Result<Vec<_>, _>>()?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(OUTPUT_VALUES)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            settings,
            origin,
            origin_frame: frame,
            started: Instant::now(),
            active: true,
            reason: "collecting",
            raw,
            frames: Vec::with_capacity(180),
            published: frame,
            output,
            spec: None,
            blocks: Vec::with_capacity(512),
            resets: Vec::with_capacity(64),
            output_stop: None,
            omitted_resets: 0,
        })
    }
    pub fn status(&self) -> Value {
        json!({"collecting":self.active,"frames":self.frames.len(),"selected_frames":self.settings.frames,"output_values":self.output.len(),"callback_blocks":self.blocks.len(),"reason":self.reason,"output_stop":self.output_stop,"source_limits":self.raw.iter().map(|r|json!({"name":r.name,"bytes":r.data.len(),"truncated":r.truncated})).collect::<Vec<_>>()})
    }
    pub fn freeze(&mut self, reason: &'static str) {
        if self.active {
            self.active = false;
            self.reason = reason;
        }
    }
    /// Main/UI thread only; the callback never reads the host clock.
    pub fn tick(&mut self) {
        if self.started.elapsed().as_secs() >= 30 {
            self.freeze("30-second-limit");
        }
    }
    pub fn generated(&mut self, m: &Machine, decision: &str) {
        self.tick();
        if !self.active {
            return;
        }
        self.published = m.bus.frames;
        if m.bus.frames <= self.origin_frame {
            self.freeze("timeline-not-after-origin");
            return;
        }
        if m.bus.frames > self.origin_frame + u64::from(self.settings.frames) {
            self.freeze("selected-guest-window-ended");
            return;
        }
        if self
            .frames
            .last()
            .is_some_and(|f| f["frame"] == m.bus.frames)
        {
            return;
        }
        let mut counts = serde_json::Map::new();
        for raw in &mut self.raw {
            let before = raw.data.len();
            match raw.name.as_str() {
                "mixed" => raw.append(
                    m.bus.audio_buf.iter().flat_map(|x| x.to_le_bytes()),
                    m.bus.audio_buf.len(),
                ),
                "psg" => raw.append(
                    m.bus.psg_tap.iter().flat_map(|x| x.to_le_bytes()),
                    m.bus.psg_tap.len(),
                ),
                "hle" => raw.append(
                    m.bus.hle_tap.iter().flat_map(|x| x.to_bits().to_le_bytes()),
                    m.bus.hle_tap.len(),
                ),
                name => {
                    let index = usize::from(name == "fifo1");
                    let values = &m.bus.fifo_tap[index];
                    raw.append(
                        values
                            .iter()
                            .flat_map(|(s, p)| std::iter::once(*s as u8).chain(p.to_le_bytes())),
                        values.len(),
                    );
                }
            }
            counts.insert(raw.name.clone(),json!({"raw_offset":before,"retained_bytes":raw.data.len()-before,"truncated":raw.truncated}));
        }
        self.frames.push(json!({"frame":m.bus.frames,"clock":m.bus.clock,"keys":m.bus.keys,"ready":m.bus.frame_ready,"handoff_decision":decision,"streams":counts,"tap_channels":m.bus.tap_channels,"hle_present":m.bus.mp2k.is_some()||m.bus.gax.is_some()||m.bus.rdrv.is_some()}));
    }
    /// Existing callback mutex is held. Storage is reserved at Start; no I/O,
    /// clock, allocation, logging or additional lock occurs in this method.
    pub fn callback<T: OutputSample>(
        &mut self,
        out: &[T],
        rate: u32,
        channels: usize,
        context: Delivery,
    ) {
        if !self.active || self.frames.is_empty() || self.output_stop.is_some() {
            return;
        }
        if rate == 0 || channels == 0 || channels > 32 {
            self.output_stop = Some("unsupported-layout");
            return;
        }
        let spec = OutputSpec {
            rate,
            channels,
            format: T::FORMAT,
        };
        if self.spec.is_some_and(|old| old != spec) {
            self.output_stop = Some("callback-format-changed");
            return;
        }
        self.spec = Some(spec);
        if self.blocks.len() == 512 {
            self.output_stop = Some("512-block-limit");
            return;
        }
        let time_limit = (rate as usize).saturating_mul(channels).saturating_mul(3);
        let limit = OUTPUT_VALUES.min(time_limit);
        let take = out.len().min(limit.saturating_sub(self.output.len())) / channels * channels;
        if take == 0 {
            self.output_stop = Some("3-second-or-value-limit");
            return;
        }
        let offset = self.output.len();
        self.output
            .extend(out[..take].iter().map(|s| s.normalized()));
        self.blocks.push(Block {
            offset,
            values: take,
            published_guest_frame: self.published,
            context,
        });
        if take < out.len() || self.output.len() == limit {
            self.output_stop = Some("3-second-or-value-limit");
        }
    }
    pub fn reset(&mut self, epoch: u64) {
        if !self.active {
            return;
        }
        if self.resets.len() < 64 {
            self.resets.push(Reset {
                epoch,
                published_guest_frame: self.published,
                output_offset: self.output.len(),
            });
        } else {
            self.omitted_resets = self.omitted_resets.saturating_add(1);
        }
    }
    pub fn save(&mut self, root: &Path, mut metadata: Value) -> Result<String, String> {
        capacity(root)?;
        self.freeze("report-request");
        let mut rows = Vec::new();
        let base = root.join("Data/config/audio");
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
        let name = format!(
            "{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos(),
            std::process::id()
        );
        let dir = base.join(&name);
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        for raw in &self.raw {
            rows.push(row(
                &dir,
                &raw.name,
                &raw.data,
                None,
                0,
                raw.truncated,
                Some(raw.observed_values),
                65536,
                2,
            )?);
        }
        let callback: Vec<_> = self
            .output
            .iter()
            .flat_map(|f| f.to_bits().to_le_bytes())
            .collect();
        let spec = self.spec.unwrap_or(OutputSpec {
            rate: 0,
            channels: 0,
            format: "unobserved",
        });
        rows.push(row(
            &dir,
            "callback",
            &callback,
            None,
            0,
            self.output_stop.is_some(),
            None,
            spec.rate,
            spec.channels,
        )?);
        metadata["kind"] = json!("ffta-audio-inspection");
        metadata["schema"] = json!(1);
        metadata["mode"] = json!("live");
        metadata["origin"] = self.origin.clone();
        metadata["settings"] = json!(self.settings);
        metadata["capture"] = self.status();
        metadata["frames"] = json!(self.frames);
        metadata["callback"] = json!({"spec":spec,"blocks":self.blocks,"resets":self.resets,"omitted_resets":self.omitted_resets});
        metadata["streams"] = json!(rows);
        publish(&dir, metadata)?;
        Ok(format!("Data/config/audio/{name}/index.html"))
    }
}
pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let base = root.join("Data/config/audio");
    if base.exists() && fs::read_dir(base).map_err(|e| e.to_string())?.count() >= 8 {
        return Err(
            "Eight audio reports already exist; retain/archive before another capture".into(),
        );
    }
    Ok(())
}
fn hash(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
fn values(name: &str, bytes: &[u8]) -> Vec<f32> {
    match name {
        "mixed" | "psg" => bytes
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes(c.try_into().unwrap()) as f32 / 32768.)
            .collect(),
        "hle" | "callback" => bytes
            .chunks_exact(4)
            .map(|c| f32::from_bits(u32::from_le_bytes(c.try_into().unwrap())))
            .collect(),
        _ => bytes
            .chunks_exact(5)
            .map(|c| c[0] as i8 as f32 / 128.)
            .collect(),
    }
}
fn preview(name: &str, bytes: &[u8], rate: u32, channels: usize) -> (Vec<f32>, u32, usize, bool) {
    if name.starts_with("fifo") {
        let mut output = Vec::new();
        let mut clocks = 0u64;
        let mut limited = false;
        for event in bytes.chunks_exact(5) {
            clocks =
                clocks.saturating_add(u32::from_le_bytes(event[1..].try_into().unwrap()) as u64);
            let end = (clocks / 256).min(65536 * 3) as usize;
            output.resize(end, event[0] as i8 as f32 / 128.);
            if clocks / 256 > 65536 * 3 {
                limited = true;
                break;
            }
        }
        (output, 65536, 1, limited)
    } else {
        let mut pcm = values(name, bytes);
        let limit = (rate as usize).saturating_mul(channels).saturating_mul(3);
        let limited = pcm.len() > limit;
        pcm.truncate(limit);
        (pcm, rate, channels, limited)
    }
}
fn wav(values: &[f32], rate: u32, channels: usize) -> Vec<u8> {
    let length = (values.len() * 2) as u32;
    let ch = channels as u16;
    let mut bytes = Vec::with_capacity(44 + length as usize);
    bytes.extend(b"RIFF");
    bytes.extend((36 + length).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(ch.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend(
        rate.saturating_mul(ch as u32)
            .saturating_mul(2)
            .to_le_bytes(),
    );
    bytes.extend((ch * 2).to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(length.to_le_bytes());
    for &value in values {
        let value = if value.is_finite() {
            value.clamp(-1., 1.)
        } else {
            0.
        };
        bytes.extend(((value * 32767.).round() as i16).to_le_bytes());
    }
    bytes
}
fn decoded(name: &str, bytes: &[u8]) -> Value {
    match name {
        "mixed" | "psg" => json!(i16::from_le_bytes(bytes.try_into().unwrap())),
        "hle" | "callback" => {
            let bits = u32::from_le_bytes(bytes.try_into().unwrap());
            json!({"bits":bits,"value":f32::from_bits(bits)})
        }
        _ => {
            json!({"sample":bytes[0] as i8,"period":u32::from_le_bytes(bytes[1..].try_into().unwrap())})
        }
    }
}
fn difference(name: &str, left: &[u8], right: &[u8], start: usize) -> Value {
    let unit = unit(name);
    let a: Vec<_> = left.chunks_exact(unit).collect();
    let b: Vec<_> = right.chunks_exact(unit).collect();
    let first = a
        .iter()
        .zip(&b)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(16)
        .map(|(i, (a, b))| json!({"index":start+i,"left_bytes":a,"right_bytes":b,"left":decoded(name,a),"right":decoded(name,b)}))
        .collect::<Vec<_>>();
    json!({"different_values_or_events":a.iter().zip(&b).filter(|(a,b)|a!=b).count()+a.len().abs_diff(b.len()),"left_values_or_events":a.len(),"right_values_or_events":b.len(),"first":first,"alignment":"Raw index comparison only; no inferred delay/resampling or common acquisition origin"})
}
#[allow(clippy::too_many_arguments)]
fn row(
    dir: &Path,
    name: &str,
    left: &[u8],
    right: Option<&[u8]>,
    start: usize,
    truncated: bool,
    observed: Option<u64>,
    rate: u32,
    channels: usize,
) -> Result<Value, String> {
    let mut sides = Vec::new();
    for (side, raw) in std::iter::once(("left", left)).chain(right.map(|r| ("right", r))) {
        let file = format!("{side}-{name}.bin");
        fs::write(dir.join(&file), raw).map_err(|e| e.to_string())?;
        let (pcm, rate, channels, limited) = preview(name, raw, rate, channels);
        let preview_name = format!("{side}-{name}.wav");
        let has_audio = !pcm.is_empty() && rate > 0 && channels > 0;
        if has_audio {
            fs::write(dir.join(&preview_name), wav(&pcm, rate, channels))
                .map_err(|e| e.to_string())?;
        }
        let bins = pcm
            .chunks(pcm.len().div_ceil(512).max(1))
            .map(|c| {
                c.iter()
                    .filter(|v| v.is_finite())
                    .fold([0f32, 0f32], |[lo, hi], &v| [lo.min(v), hi.max(v)])
            })
            .collect::<Vec<_>>();
        sides.push(json!({"side":side,"raw":file,"sha256":hash(raw),"bytes":raw.len(),"values_or_events":raw.len()/unit(name),"preview":if has_audio{Some(preview_name)}else{None},"preview_rate":rate,"preview_channels":channels,"preview_values":pcm.len(),"preview_limited":limited,"nonfinite":pcm.iter().filter(|v|!v.is_finite()).count(),"peak":pcm.iter().filter(|v|v.is_finite()).map(|v|v.abs()).fold(0f32,f32::max),"bins":bins}));
    }
    Ok(
        json!({"name":name,"unit_bytes":unit(name),"start_index":start,"truncated":truncated,"observed_values_or_events":observed,"sides":sides,"difference":right.map(|r|difference(name,left,r,start)),"preview_scope":if name.starts_with("fifo"){ "Isolated FIFO sample hold using period/256 at65536Hz; first phase/routing/filter unknown, max3s"}else{"Lossy PCM16 audition; raw i16/IEEE-f32 bits retained for exact comparison"}}),
    )
}
fn publish(dir: &Path, mut report: Value) -> Result<(), String> {
    report["scope"] = json!({"generated":"Guest buffers observed before normal frontend handoff/discard; first buffer may include pending origin data, declared in origin counts", "submitted":"Actual application callback buffer after DSP/gain and native sample conversion, normalized to f32; not physical device delivery/listening", "callbacks":"Preallocated bounded samples/blocks under existing mutex; no new callback IO/clock/allocation/lock. Published guest frame is only a producer watermark, not a per-sample origin", "listener":"Physical listening not performed; backend/device buffering, XRUNs and speaker output unknown", "preview":"PCM16 audition and bounded wave summaries; exact differences use raw bytes. FIFO preview excludes routing/filter/initial phase", "acceptance":"No new native/ROM/DS correctness or performance acceptance"});
    let json = serde_json::to_vec(&report).map_err(|e| e.to_string())?;
    if json.len() > 2 * 1024 * 1024 {
        return Err("Audio metadata exceeds2MiB".into());
    }
    let embedded = String::from_utf8(json.clone())
        .unwrap()
        .replace('<', "\\u003c");
    fs::write(
        dir.join("index.html"),
        include_str!("debug_audio/report.html").replacen("/*REPORT*/", &embedded, 1),
    )
    .map_err(|e| e.to_string())?;
    fs::write(dir.join("audio.json"), json).map_err(|e| e.to_string())
}
fn raw(m: &Machine, name: &str) -> Vec<u8> {
    match name {
        "mixed" => m
            .bus
            .audio_buf
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect(),
        "psg" => m.bus.psg_tap.iter().flat_map(|x| x.to_le_bytes()).collect(),
        "hle" => m
            .bus
            .hle_tap
            .iter()
            .flat_map(|x| x.to_bits().to_le_bytes())
            .collect(),
        _ => m.bus.fifo_tap[usize::from(name == "fifo1")]
            .iter()
            .flat_map(|(s, p)| std::iter::once(*s as u8).chain(p.to_le_bytes()))
            .collect(),
    }
}
pub(crate) fn cmd(args: &[String]) -> u8 {
    match states(args) {
        Ok(code) => code,
        Err(message) => {
            println!("{}", json!({"schema":1,"kind":"ffta-audio-result","outcome":"incompatible","message":message}));
            3
        }
    }
}
fn states(args: &[String]) -> Result<u8, String> {
    if args.first().map(String::as_str) != Some("states") {
        return Err("Use debug-audio states --rom ROM --left STATE --right STATE --dir NEW [--streams mixed,psg,hle,fifo0,fifo1] [--start EVEN --count EVEN] [--bios FILE]".into());
    }
    let mut options = std::collections::BTreeMap::new();
    let mut it = args[1..].iter();
    while let Some(key) = it.next() {
        if ![
            "--rom",
            "--left",
            "--right",
            "--dir",
            "--streams",
            "--start",
            "--count",
            "--bios",
        ]
        .contains(&key.as_str())
            || options.contains_key(key)
        {
            return Err(format!("Unknown/duplicate audio option {key}"));
        }
        options.insert(
            key.clone(),
            it.next().ok_or("Missing audio option value")?.clone(),
        );
    }
    let get = |key: &str| options.get(key).ok_or_else(|| format!("Missing {key}"));
    let start: usize = options
        .get("--start")
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| "Invalid start index")?
        .unwrap_or(0);
    let count: usize = options
        .get("--count")
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| "Invalid count")?
        .unwrap_or(262144);
    if start % 2 != 0
        || count == 0
        || count % 2 != 0
        || count > 262144
        || start.checked_add(count).is_none()
    {
        return Err("State audio selection needs even start/count,1..262144values/events".into());
    }
    let selected = Settings {
        frames: 1,
        streams: options
            .get("--streams")
            .map(|s| s.split(',').map(str::to_owned).collect())
            .unwrap_or_else(|| Settings::default().streams),
    };
    selected.validate()?;
    let rom = crate::diagnostics::read(Path::new(get("--rom")?), 32 * 1024 * 1024)?;
    let bios = options
        .get("--bios")
        .map(|p| crate::diagnostics::read(Path::new(p), 16384))
        .transpose()?;
    let mut left = crate::make_machine(rom.clone(), bios.as_deref());
    let mut right = crate::make_machine(rom, bios.as_deref());
    let lbytes = crate::diagnostics::read(Path::new(get("--left")?), 65 * 1024 * 1024)?;
    let rbytes = crate::diagnostics::read(Path::new(get("--right")?), 65 * 1024 * 1024)?;
    crate::save_state::load_bytes(&mut left, &lbytes)?;
    crate::save_state::load_bytes(&mut right, &rbytes)?;
    if left.bus.frames != right.bus.frames {
        return Err("State audio comparison requires the same declared guest frame; clock differences remain visible".into());
    }
    let dir = Path::new(get("--dir")?);
    if dir.exists() {
        return Err("Audio output directory must be new".into());
    }
    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for name in &selected.streams {
        let (a, b) = (raw(&left, name), raw(&right, name));
        let u = unit(name);
        let begin = start.checked_mul(u).ok_or("Audio index overflow")?;
        let end = begin.saturating_add(count * u);
        let a = &a[begin.min(a.len())..end.min(a.len())];
        let b = &b[begin.min(b.len())..end.min(b.len())];
        rows.push(row(dir, name, a, Some(b), start, false, None, 65536, 2)?);
    }
    let identity = |m: &Machine, bytes: &[u8]| json!({"snapshot_sha256":hash(bytes),"frame":m.bus.frames,"clock":m.bus.clock,"keys":m.bus.keys,"bios":{"real":m.bus.real_bios,"sha256":hash(&m.bus.bios)},"tap_channels":m.bus.tap_channels,"hle_present":m.bus.mp2k.is_some()||m.bus.gax.is_some()||m.bus.rdrv.is_some(),"pending_audio":[m.bus.audio_buf.len(),m.bus.psg_tap.len(),m.bus.hle_tap.len(),m.bus.fifo_tap[0].len(),m.bus.fifo_tap[1].len()]});
    let runtime_sha=hash(&crate::diagnostics::read(&std::env::current_exe().map_err(|e|e.to_string())?,256*1024*1024)?);
    let different=rows.iter().any(|r|r["difference"]["different_values_or_events"].as_u64().is_some_and(|n|n>0));
    let outcome=if different{"diverged"}else{"equal"};
    publish(
        dir,
        json!({"schema":1,"kind":"ffta-audio-inspection","outcome":outcome,"mode":"frozen-state-comparison","runtime_sha256":runtime_sha,"rom_sha256":hash(&left.bus.rom),"left":identity(&left,&lbytes),"right":identity(&right,&rbytes),"selection":{"start":start,"count":count},"streams":rows,"callback":{"observed":false},"frames":[],"capture":{"reason":"existing buffers only; no guest step, input replay or attached dataset"}}),
    )?;
    println!(
        "{}",
        json!({"schema":1,"kind":"ffta-audio-result","outcome":outcome,"report":dir.join("index.html").to_string_lossy(),"guest_execution":false})
    );
    Ok(if different{2}else{0})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_and_exact_float_fifo_comparisons_are_bounded() {
        assert!(
            Settings {
                frames: 181,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                streams: vec!["mixed".into(), "mixed".into()],
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        let a = [f32::from_bits(0x7fc00001), 0.];
        let b = [f32::from_bits(0x7fc00002), -0.];
        let bytes = |v: &[f32]| {
            v.iter()
                .flat_map(|x| x.to_bits().to_le_bytes())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            difference("hle", &bytes(&a), &bytes(&b), 30)["different_values_or_events"],
            2
        );
        let (w, rate, ch, limited) = preview("fifo0", &[64, 0, 1, 0, 0], 0, 0);
        assert_eq!((w.len(), rate, ch, limited), (1, 65536, 1, false));
        assert_eq!(wav(&[0., 1., f32::NAN], 48000, 1).len(), 50);
    }
    #[test]
    fn callback_storage_never_grows_and_is_observational() {
        let mut c = Capture::start(
            Settings {
                frames: 1,
                streams: vec!["mixed".into()],
            },
            json!({}),
            0,
        )
        .unwrap();
        let mut m = Machine::new(vec![0; 256]);
        m.bus.frames = 1;
        m.bus.audio_buf = vec![400, -400];
        let before = m.save_state().unwrap();
        c.generated(&m, "test");
        assert_eq!(m.save_state().unwrap(), before);
        let delivery = Delivery {
            epoch: 1,
            target_gain: 1.,
            current_gain: 1.,
            speed: 1.,
            ..Default::default()
        };
        let capacity = c.output.capacity();
        let blocks = c.blocks.capacity();
        for _ in 0..700 {
            c.callback(&[0i16; 2048], 48000, 2, delivery);
        }
        assert_eq!(c.output.capacity(), capacity);
        assert_eq!(c.blocks.capacity(), blocks);
        assert!(c.output.len() <= 48000 * 2 * 3);
        assert!(c.output_stop.is_some());
        for e in 0..100 {
            c.reset(e);
        }
        assert_eq!(c.resets.len(), 64);
        assert_eq!(c.omitted_resets, 36);
        assert_eq!(<u16 as OutputSample>::normalized(32768), 0.);
    }
}
