//! Horizon Replay native trait mod.
//!
//! First implementation milestone: record and replay the camera pose. The camera
//! pose hook is a supported Horizon API; it is deliberately used instead of
//! guessing at private player fields. Samples are bounded and stored in a small
//! versioned binary file under the mod's config directory.

use horizon_client::mod_api::{
    CameraPoseEvent, FrameEvent, Host, Hooks, HudEvent, KeyEvent,
};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const SAMPLE_INTERVAL: f64 = 0.05; // 20 samples per second
const MAX_SAMPLES: usize = 60_000; // 50 minutes at the configured sample rate
const MAX_REPLAY_BYTES: usize = 8 * 1024 * 1024;
const REPLAY_FILE: &str = "latest.hrp";
const MAGIC: &[u8; 4] = b"HRP1";

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sample {
    time: f64,
    position: [f64; 3],
    yaw: f32,
    pitch: f32,
}

#[derive(Debug)]
struct ReplayState {
    recording: bool,
    playing: bool,
    paused: bool,
    samples: Vec<Sample>,
    started: Option<Instant>,
    last_sample: f64,
    playhead: f64,
    speed: f64,
    dimension: String,
    status: String,
}

impl Default for ReplayState {
    fn default() -> Self {
        Self {
            recording: false,
            playing: false,
            paused: false,
            samples: Vec::new(),
            started: None,
            last_sample: -SAMPLE_INTERVAL,
            playhead: 0.0,
            speed: 1.0,
            dimension: String::new(),
            status: "Ready | F8 record | F9 play | F7 pause | F10 stop".into(),
        }
    }
}

static STATE: OnceLock<Mutex<ReplayState>> = OnceLock::new();

fn state() -> &'static Mutex<ReplayState> {
    STATE.get_or_init(|| Mutex::new(ReplayState::default()))
}

fn lock_state() -> std::sync::MutexGuard<'static, ReplayState> {
    state().lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct HorizonReplay;

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(HorizonReplay)
}

impl Hooks for HorizonReplay {
    fn frame(&self, _host: &mut dyn Host, event: &mut FrameEvent) {
        let mut state = lock_state();
        if !state.playing || state.paused {
            return;
        }

        // A stalled frame must not jump a replay several seconds forward.
        state.playhead += event.elapsed.as_secs_f64().min(0.25) * state.speed;
        if let Some(last) = state.samples.last() {
            if state.playhead >= last.time {
                state.playhead = last.time;
                state.playing = false;
                state.status = "Replay finished".into();
            }
        }
    }

    fn key(&self, host: &mut dyn Host, event: &mut KeyEvent<'_>) {
        if !event.pressed {
            return;
        }

        // Mouse buttons in the replay panel map to the same actions as hotkeys.
        let key = if event.key == "key.mouse.left" {
            let [x, y] = host.cursor();
            if (x >= 16.0 && x <= 102.0 && y >= 88.0 && y <= 108.0) {
                "key.keyboard.f8"
            } else if (x >= 108.0 && x <= 190.0 && y >= 88.0 && y <= 108.0) {
                "key.keyboard.f9"
            } else if (x >= 16.0 && x <= 102.0 && y >= 114.0 && y <= 134.0) {
                "key.keyboard.f7"
            } else if (x >= 108.0 && x <= 190.0 && y >= 114.0 && y <= 134.0) {
                "key.keyboard.f10"
            } else {
                ""
            }
        } else {
            event.key
        };

        match key {
            "key.keyboard.f8" => {
                let save = {
                    let mut state = lock_state();
                    if state.recording {
                        state.recording = false;
                        state.started = None;
                        state.playing = false;
                        state.status = "Saving replay…".into();
                        Some((state.samples.clone(), state.dimension.clone()))
                    } else {
                        state.samples.clear();
                        state.dimension.clear();
                        state.started = Some(Instant::now());
                        state.last_sample = -SAMPLE_INTERVAL;
                        state.recording = true;
                        state.playing = false;
                        state.paused = false;
                        state.status = "RECORDING".into();
                        None
                    }
                };

                if let Some((samples, dimension)) = save {
                    let result = save_replay(host, &dimension, &samples);
                    let mut state = lock_state();
                    state.status = match result {
                        Ok(()) if samples.len() >= 2 => {
                            format!("Saved {} camera samples", samples.len())
                        }
                        Ok(()) => "Recording too short; nothing saved".into(),
                        Err(error) => format!("Save failed: {error}"),
                    };
                }
            }
            "key.keyboard.f9" => {
                match load_replay(host) {
                    Ok((dimension, samples)) if samples.len() >= 2 => {
                        let current_dimension = host.camera().map(|camera| camera.dimension);
                        let mut state = lock_state();
                        if current_dimension.as_deref().is_some_and(|d| d != dimension.as_str()) {
                            state.status = "Replay belongs to another dimension".into();
                            state.playing = false;
                        } else {
                            state.dimension = dimension;
                            state.samples = samples;
                            state.playhead = 0.0;
                            state.playing = true;
                            state.paused = false;
                            state.recording = false;
                            state.speed = 1.0;
                            state.status = "PLAYING | F7 pause | F10 stop".into();
                        }
                    }
                    Ok(_) => lock_state().status = "Replay has too few samples".into(),
                    Err(error) => lock_state().status = format!("Load failed: {error}"),
                }
            }
            "key.keyboard.f7" => {
                let mut state = lock_state();
                if state.playing {
                    state.paused = !state.paused;
                    state.status = if state.paused { "PAUSED" } else { "PLAYING" }.into();
                }
            }
            "key.keyboard.f5" => {
                let mut state = lock_state();
                if state.playing {
                    state.speed = (state.speed / 2.0).max(0.25);
                }
            }
            "key.keyboard.f6" => {
                let mut state = lock_state();
                if state.playing {
                    state.speed = (state.speed * 2.0).min(4.0);
                }
            }
            "key.keyboard.f10" => {
                let mut state = lock_state();
                state.recording = false;
                state.playing = false;
                state.paused = false;
                state.started = None;
                state.status = "Stopped".into();
            }
            _ => {}
        }
    }

    fn camera_pose(&self, host: &dyn Host, event: &mut CameraPoseEvent) {
        let current_dimension = host.camera().map(|camera| camera.dimension);

        let mut state = lock_state();

        if state.recording {
            let Some(started) = state.started else {
                state.recording = false;
                state.status = "Recording stopped: missing timer".into();
                return;
            };

            let elapsed = started.elapsed().as_secs_f64();
            if elapsed - state.last_sample >= SAMPLE_INTERVAL {
                if let Some(dimension) = current_dimension {
                    if !state.dimension.is_empty() && state.dimension != dimension {
                        // Stop adding samples from the new dimension, but keep
                        // F8 available to save the recording already captured.
                        state.status = "Dimension changed; press F8 to save".into();
                        return;
                    }
                    state.dimension = dimension;
                }

                if state.samples.len() >= MAX_SAMPLES {
                    // Keep the recording state armed so F8 still saves the full buffer.
                    state.status = "Recording limit reached; press F8 to save".into();
                    return;
                }
                state.samples.push(Sample {
                    time: elapsed,
                    position: event.position,
                    yaw: event.yaw,
                    pitch: event.pitch,
                });
                state.last_sample = elapsed;
            }
            return;
        }

        if !state.playing || state.samples.len() < 2 {
            return;
        }

        if current_dimension
            .as_deref()
            .is_some_and(|dimension| dimension != state.dimension.as_str())
        {
            state.playing = false;
            state.status = "Playback stopped: dimension changed".into();
            return;
        }

        if let Some(sample) = sample_at(&state.samples, state.playhead) {
            event.position = sample.position;
            event.yaw = sample.yaw;
            event.pitch = sample.pitch;
        }
    }

    fn hud(&self, host: &mut dyn Host, event: &mut HudEvent<'_>) {
        let state = lock_state();

        // Compact clickable control panel. Coordinates are GUI pixels.
        host.fill(event.batch, [8.0, 34.0, 194.0, 112.0], 0xD9161920);
        host.fill(event.batch, [8.0, 34.0, 194.0, 20.0], 0xFF252B36);
        host.text(event.batch, "HORIZON REPLAY", [16.0, 40.0], 0xFFFFFF, true);

        let status = if state.recording {
            format!("RECORDING  •  {} samples", state.samples.len())
        } else if state.playing && state.paused {
            format!("PAUSED  •  {:.1}s  •  {:.2}x", state.playhead, state.speed)
        } else if state.playing {
            format!("PLAYING  •  {:.1}s  •  {:.2}x", state.playhead, state.speed)
        } else {
            state.status.clone()
        };
        host.text(event.batch, &status, [16.0, 62.0], 0xE8E8E8, true);

        let record_color = if state.recording { 0xFF8B3030 } else { 0xFF315B43 };
        host.fill(event.batch, [16.0, 88.0, 86.0, 20.0], record_color);
        host.text(event.batch, if state.recording { "SAVE" } else { "RECORD" }, [38.0, 94.0], 0xFFFFFF, true);

        host.fill(event.batch, [108.0, 88.0, 82.0, 20.0], 0xFF31506F);
        host.text(event.batch, "PLAY", [137.0, 94.0], 0xFFFFFF, true);

        host.fill(event.batch, [16.0, 114.0, 86.0, 20.0], 0xFF55505F);
        host.text(event.batch, if state.paused { "RESUME" } else { "PAUSE" }, [37.0, 120.0], 0xFFFFFF, true);

        host.fill(event.batch, [108.0, 114.0, 82.0, 20.0], 0xFF684040);
        host.text(event.batch, "STOP", [136.0, 120.0], 0xFFFFFF, true);

        let line = if state.recording {
            format!("Horizon Replay  •  REC  •  {} samples", state.samples.len())
        } else if state.playing && state.paused {
            format!("Horizon Replay  •  PAUSED  •  {:.1}s  •  {:.2}x", state.playhead, state.speed)
        } else if state.playing {
            let duration = state.samples.last().map(|sample| sample.time).unwrap_or(0.0);
            format!("Horizon Replay  •  PLAY  •  {:.1}/{:.1}s  •  {:.2}x", state.playhead, duration, state.speed)
        } else {
            format!("Horizon Replay  •  {}", state.status)
        };

        host.text(event.batch, &line, [10.0, 10.0], 0xFFFFFF, true);
        host.text(
            event.batch,
            "F8 Record/Save   F9 Play   F7 Pause   F5/F6 Speed   F10 Stop",
            [10.0, 22.0],
            0xD0D0D0,
            true,
        );
    }
}

fn sample_at(samples: &[Sample], time: f64) -> Option<Sample> {
    let first = *samples.first()?;
    let last = *samples.last()?;
    if time <= first.time {
        return Some(first);
    }
    if time >= last.time {
        return Some(last);
    }

    let right = samples.partition_point(|sample| sample.time < time);
    let a = samples[right.saturating_sub(1)];
    let b = samples[right];
    let span = (b.time - a.time).max(f64::EPSILON);
    let t = ((time - a.time) / span).clamp(0.0, 1.0);

    Some(Sample {
        time,
        position: std::array::from_fn(|i| a.position[i] + (b.position[i] - a.position[i]) * t),
        yaw: lerp_angle(a.yaw, b.yaw, t as f32),
        pitch: a.pitch + (b.pitch - a.pitch) * t as f32,
    })
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let delta = (b - a + 180.0).rem_euclid(360.0) - 180.0;
    a + delta * t
}

fn replay_path(host: &dyn Host) -> Result<PathBuf, String> {
    host.config_dir()
        .map(|directory| directory.join("horizon-replay").join(REPLAY_FILE))
}

fn save_replay(host: &dyn Host, dimension: &str, samples: &[Sample]) -> Result<(), String> {
    if samples.len() < 2 {
        return Err("at least two samples are required".into());
    }

    let path = replay_path(host)?;
    let parent = path.parent().ok_or("invalid replay path")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let bytes = encode_replay(dimension, samples)?;
    let temporary = path.with_extension("hrp.tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(())
}

fn load_replay(host: &dyn Host) -> Result<(String, Vec<Sample>), String> {
    let path = replay_path(host)?;
    let size = fs::metadata(&path).map_err(|error| error.to_string())?.len();
    if size > MAX_REPLAY_BYTES as u64 {
        return Err("replay file exceeds the size limit".into());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    decode_replay(&bytes)
}

fn encode_replay(dimension: &str, samples: &[Sample]) -> Result<Vec<u8>, String> {
    if samples.len() < 2 {
        return Err("at least two samples are required".into());
    }
    if samples.len() > MAX_SAMPLES {
        return Err("too many samples".into());
    }
    if dimension.len() > 128 {
        return Err("dimension identifier is too long".into());
    }

    let mut output = Vec::with_capacity(10 + dimension.len() + samples.len() * 40);
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&(dimension.len() as u16).to_le_bytes());
    output.extend_from_slice(dimension.as_bytes());
    output.extend_from_slice(&(samples.len() as u32).to_le_bytes());

    let mut previous_time = -1.0;
    for sample in samples {
        if !sample.time.is_finite()
            || sample.time < 0.0
            || sample.time < previous_time
            || sample.position.iter().any(|value| !value.is_finite())
            || !sample.yaw.is_finite()
            || !sample.pitch.is_finite()
        {
            return Err("sample contains invalid values".into());
        }
        previous_time = sample.time;
        output.extend_from_slice(&sample.time.to_le_bytes());
        for coordinate in sample.position {
            output.extend_from_slice(&coordinate.to_le_bytes());
        }
        output.extend_from_slice(&sample.yaw.to_le_bytes());
        output.extend_from_slice(&sample.pitch.to_le_bytes());
    }

    if output.len() > MAX_REPLAY_BYTES {
        return Err("encoded replay exceeds the size limit".into());
    }
    Ok(output)
}

fn decode_replay(bytes: &[u8]) -> Result<(String, Vec<Sample>), String> {
    if bytes.len() > MAX_REPLAY_BYTES || bytes.len() < 10 || &bytes[..4] != MAGIC {
        return Err("invalid replay header".into());
    }

    let mut cursor = 4;
    let dimension_len = take_u16(bytes, &mut cursor)? as usize;
    if dimension_len > 128 {
        return Err("invalid dimension length".into());
    }
    let dimension_end = cursor.checked_add(dimension_len).ok_or("invalid length")?;
    let dimension = std::str::from_utf8(
        bytes.get(cursor..dimension_end).ok_or("truncated dimension")?,
    )
    .map_err(|_| "dimension is not UTF-8")?
    .to_owned();
    cursor = dimension_end;

    let count = take_u32(bytes, &mut cursor)? as usize;
    if count > MAX_SAMPLES || count < 2 {
        return Err("invalid sample count".into());
    }
    let expected = cursor
        .checked_add(count.checked_mul(40).ok_or("invalid sample size")?)
        .ok_or("invalid replay length")?;
    if expected != bytes.len() {
        return Err("replay length does not match its sample count".into());
    }

    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let time = take_f64(bytes, &mut cursor)?;
        let position = [
            take_f64(bytes, &mut cursor)?,
            take_f64(bytes, &mut cursor)?,
            take_f64(bytes, &mut cursor)?,
        ];
        let yaw = take_f32(bytes, &mut cursor)?;
        let pitch = take_f32(bytes, &mut cursor)?;
        samples.push(Sample { time, position, yaw, pitch });
    }

    // Reuse the encoder's validation without retaining a second encoded copy.
    let mut previous_time = -1.0;
    for sample in &samples {
        if !sample.time.is_finite()
            || sample.time < 0.0
            || sample.time < previous_time
            || sample.position.iter().any(|value| !value.is_finite())
            || !sample.yaw.is_finite()
            || !sample.pitch.is_finite()
        {
            return Err("replay contains invalid samples".into());
        }
        previous_time = sample.time;
    }

    Ok((dimension, samples))
}

fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> Result<[u8; N], String> {
    let end = cursor.checked_add(N).ok_or("invalid offset")?;
    let slice = bytes.get(*cursor..end).ok_or("truncated replay")?;
    let mut value = [0u8; N];
    value.copy_from_slice(slice);
    *cursor = end;
    Ok(value)
}

fn take_u16(bytes: &[u8], cursor: &mut usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(take(bytes, cursor)?))
}
fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(take(bytes, cursor)?))
}
fn take_f64(bytes: &[u8], cursor: &mut usize) -> Result<f64, String> {
    Ok(f64::from_le_bytes(take(bytes, cursor)?))
}
fn take_f32(bytes: &[u8], cursor: &mut usize) -> Result<f32, String> {
    Ok(f32::from_le_bytes(take(bytes, cursor)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<Sample> {
        vec![
            Sample { time: 0.0, position: [0.0, 64.0, 0.0], yaw: 170.0, pitch: 0.0 },
            Sample { time: 1.0, position: [10.0, 66.0, -4.0], yaw: -170.0, pitch: 20.0 },
        ]
    }

    #[test]
    fn replay_round_trips() {
        let samples = fixture();
        let bytes = encode_replay("minecraft:overworld", &samples).unwrap();
        let (dimension, decoded) = decode_replay(&bytes).unwrap();
        assert_eq!(dimension, "minecraft:overworld");
        assert_eq!(decoded, samples);
    }

    #[test]
    fn interpolation_wraps_yaw_the_short_way() {
        let sample = sample_at(&fixture(), 0.5).unwrap();
        assert!((sample.position[0] - 5.0).abs() < 1e-9);
        assert!(sample.yaw.abs() > 179.0);
        assert!((sample.pitch - 10.0).abs() < 1e-5);
    }

    #[test]
    fn rejects_truncated_and_wrong_magic_files() {
        assert!(decode_replay(b"nope").is_err());
        let bytes = encode_replay("minecraft:overworld", &fixture()).unwrap();
        assert!(decode_replay(&bytes[..bytes.len() - 1]).is_err());
    }
}
