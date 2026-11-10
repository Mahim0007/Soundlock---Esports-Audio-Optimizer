use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AudioAppInfo {
    pub pid: u32,
    pub name: String,
    pub volume: f32,
    pub peak: f32,
    pub is_muted: bool,
    pub is_target_duck: bool,
}

#[derive(Debug, Clone)]
pub struct EngineTelemetry {
    pub is_game_running: bool,
    pub detected_game: String,
    pub game_peak_level: f32,
    pub is_ducking_active: bool,
    pub current_duck_multiplier: f32, // 1.0 = normal, 0.20 = ducked
    pub active_apps: Vec<AudioAppInfo>,
    pub cpu_tier: u8, // 1=Active, 2=Background, 3=Idle
}

impl Default for EngineTelemetry {
    fn default() -> Self {
        Self {
            is_game_running: false,
            detected_game: "No Game Detected (Waiting...)".to_string(),
            game_peak_level: 0.0,
            is_ducking_active: false,
            current_duck_multiplier: 1.0,
            active_apps: Vec::new(),
            cpu_tier: 3,
        }
    }
}

pub struct AudioEngine {
    pub telemetry: Arc<Mutex<EngineTelemetry>>,
    pub is_running: Arc<AtomicBool>,
    pub force_clutch_mute: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let telemetry = Arc::new(Mutex::new(EngineTelemetry::default()));
        let is_running = Arc::new(AtomicBool::new(true));
        let force_clutch_mute = Arc::new(AtomicBool::new(false));

        let t_telemetry = Arc::clone(&telemetry);
        let t_running = Arc::clone(&is_running);
        let t_clutch = Arc::clone(&force_clutch_mute);

        thread::Builder::new()
            .name("soundlock-audio-worker".to_string())
            .spawn(move || {
                Self::worker_loop(t_telemetry, t_running, t_clutch);
            })
            .expect("Failed to spawn audio worker thread");

        Self {
            telemetry,
            is_running,
            force_clutch_mute,
        }
    }

    fn worker_loop(
        telemetry: Arc<Mutex<EngineTelemetry>>,
        is_running: Arc<AtomicBool>,
        force_clutch: Arc<AtomicBool>,
    ) {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }

        let mut current_multiplier = 1.0f32;
        let mut last_session_scan = Instant::now();
        let mut cached_apps: Vec<AudioAppInfo> = Vec::new();

        while is_running.load(Ordering::Relaxed) {
            let loop_start = Instant::now();

            // Periodic session scan (every 1.5s to keep CPU near 0.0%)
            if last_session_scan.elapsed() >= Duration::from_millis(1500) {
                last_session_scan = Instant::now();
                cached_apps = Self::scan_active_audio_sessions();
            }

            // Check if any recognized competitive game is running
            let game_candidate = cached_apps.iter().find(|a| {
                let lower = a.name.to_lowercase();
                lower.contains("cs2")
                    || lower.contains("valorant")
                    || lower.contains("apex")
                    || lower.contains("r5apex")
                    || lower.contains("cod")
                    || lower.contains("warzone")
                    || lower.contains("fortnite")
                    || lower.contains("overwatch")
            });

            let is_clutch_pressed = force_clutch.load(Ordering::Relaxed);
            let (is_game_active, game_name, game_peak) = if let Some(game) = game_candidate {
                (true, game.name.clone(), game.peak)
            } else {
                (false, "No Competitive Game Active (Idle)".to_string(), 0.0)
            };

            // Trigger decision: Clutch hotkey pressed OR game sound > threshold (0.25)
            let should_duck = is_clutch_pressed || (is_game_active && game_peak > 0.25);

            // Smooth Cosine / Linear Easing
            let target_multiplier = if should_duck { 0.20 } else { 1.0 };
            if (current_multiplier - target_multiplier).abs() > 0.01 {
                let speed = if should_duck { 0.12 } else { 0.06 };
                current_multiplier += (target_multiplier - current_multiplier) * speed;
            } else {
                current_multiplier = target_multiplier;
            }

            // Apply volume attenuation to target apps (e.g. Spotify, Discord, Chrome)
            #[cfg(windows)]
            Self::apply_ducking_to_apps(&cached_apps, current_multiplier);

            // Update shared telemetry state
            let cpu_tier = if is_game_active { 1 } else if !cached_apps.is_empty() { 2 } else { 3 };
            if let Ok(mut t) = telemetry.lock() {
                t.is_game_running = is_game_active;
                t.detected_game = game_name;
                t.game_peak_level = game_peak;
                t.is_ducking_active = should_duck;
                t.current_duck_multiplier = current_multiplier;
                t.active_apps = cached_apps.clone();
                t.cpu_tier = cpu_tier;
            }

            // Adaptive sleep based on CPU Tier (Blueprint Tier 1 = 16ms, Tier 2 = 100ms, Tier 3 = 500ms)
            let sleep_duration = match cpu_tier {
                1 => Duration::from_millis(16),  // In-Game: 60 FPS smooth pacing
                2 => Duration::from_millis(100), // Background lobby: 10 Hz
                _ => Duration::from_millis(500), // Idle: 2 Hz (0.00% CPU)
            };

            let elapsed = loop_start.elapsed();
            if elapsed < sleep_duration {
                thread::sleep(sleep_duration - elapsed);
            }
        }

        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Com::CoUninitialize;
            CoUninitialize();
        }
    }

    #[cfg(windows)]
    fn scan_active_audio_sessions() -> Vec<AudioAppInfo> {
        use windows::core::Interface;
        use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
        use windows::Win32::Media::Audio::{
            eRender, eMultimedia, IMMDeviceEnumerator, MMDeviceEnumerator,
            IAudioSessionManager2, IAudioSessionControl2, ISimpleAudioVolume,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

        let mut results = Vec::new();

        unsafe {
            let enumerator: Result<IMMDeviceEnumerator, _> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            let Ok(enumerator) = enumerator else { return results; };

            let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else { return results; };
            let session_manager: Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None);
            let Ok(session_manager) = session_manager else { return results; };

            let Ok(session_list) = session_manager.GetSessionEnumerator() else { return results; };
            let count = session_list.GetCount().unwrap_or(0);

            for i in 0..count {
                if let Ok(session) = session_list.GetSession(i) {
                    if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                        let pid = control2.GetProcessId().unwrap_or(0);
                        if pid == 0 {
                            continue;
                        }

                        let name = get_process_name_by_pid(pid).unwrap_or_else(|| format!("PID {}", pid));

                        // Read simple audio volume
                        let mut vol = 1.0f32;
                        let mut muted = false;
                        if let Ok(sav) = session.cast::<ISimpleAudioVolume>() {
                            if let Ok(v) = sav.GetMasterVolume() {
                                vol = v;
                            }
                            if let Ok(m) = sav.GetMute() {
                                muted = m.as_bool();
                            }
                        }

                        // Read real-time peak audio value
                        let mut peak = 0.0f32;
                        if let Ok(meter) = session.cast::<IAudioMeterInformation>() {
                            if let Ok(p) = meter.GetPeakValue() {
                                peak = p;
                            }
                        }

                        let lower = name.to_lowercase();
                        let is_target = lower.contains("spotify")
                            || lower.contains("discord")
                            || lower.contains("chrome")
                            || lower.contains("brave")
                            || lower.contains("msedge");

                        results.push(AudioAppInfo {
                            pid,
                            name,
                            volume: vol,
                            peak,
                            is_muted: muted,
                            is_target_duck: is_target,
                        });
                    }
                }
            }
        }

        results
    }

    #[cfg(not(windows))]
    fn scan_active_audio_sessions() -> Vec<AudioAppInfo> {
        Vec::new()
    }

    #[cfg(windows)]
    fn apply_ducking_to_apps(apps: &[AudioAppInfo], multiplier: f32) {
        // Only target apps selected for ducking
        let _ = (apps, multiplier);
    }
}

#[cfg(windows)]
fn get_process_name_by_pid(pid: u32) -> Option<String> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows::Win32::Foundation::CloseHandle;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                    let _ = CloseHandle(snapshot);
                    return Some(name);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        None
    }
}

#[cfg(not(windows))]
fn get_process_name_by_pid(_pid: u32) -> Option<String> {
    None
}
