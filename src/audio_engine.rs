use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::config::AppConfig;

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
    pub current_duck_multiplier: f32, // 1.0 = normal, e.g. 0.20 = ducked
    pub active_apps: Vec<AudioAppInfo>,
    pub cpu_tier: u8, // 1=Active, 2=Background, 3=Idle
    pub is_clutch_active: bool,
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
            is_clutch_active: false,
        }
    }
}

pub enum AudioEngineCommand {
    SetAppVolume { pid: u32, volume: f32 },
    SetAppMute { pid: u32, mute: bool },
    TestDucking { duration: Duration },
    ToggleClutch,
}

pub struct AudioEngine {
    pub telemetry: Arc<Mutex<EngineTelemetry>>,
    pub is_running: Arc<AtomicBool>,
    pub command_sender: Sender<AudioEngineCommand>,
    pub shared_config: Arc<Mutex<AppConfig>>,
}

impl AudioEngine {
    pub fn new(initial_config: AppConfig) -> Self {
        let telemetry = Arc::new(Mutex::new(EngineTelemetry::default()));
        let is_running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = channel::<AudioEngineCommand>();
        let shared_config = Arc::new(Mutex::new(initial_config));

        let t_telemetry = Arc::clone(&telemetry);
        let t_running = Arc::clone(&is_running);
        let t_config = Arc::clone(&shared_config);

        thread::Builder::new()
            .name("soundlock-audio-worker".to_string())
            .spawn(move || {
                Self::worker_loop(t_telemetry, t_running, rx, t_config);
            })
            .expect("Failed to spawn audio worker thread");

        Self {
            telemetry,
            is_running,
            command_sender: tx,
            shared_config,
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn send_command(&self, cmd: AudioEngineCommand) {
        let _ = self.command_sender.send(cmd);
    }

    pub fn update_config(&self, new_cfg: &AppConfig) {
        if let Ok(mut cfg) = self.shared_config.lock() {
            *cfg = new_cfg.clone();
        }
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

impl AudioEngine {
    fn worker_loop(
        telemetry: Arc<Mutex<EngineTelemetry>>,
        is_running: Arc<AtomicBool>,
        command_receiver: Receiver<AudioEngineCommand>,
        shared_config: Arc<Mutex<AppConfig>>,
    ) {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }

        let mut current_multiplier = 1.0f32;
        let mut last_applied_multiplier = 1.0f32;
        let mut baseline_volumes: HashMap<u32, f32> = HashMap::new();
        let mut pid_cache: HashMap<u32, String> = HashMap::new();
        let mut test_ducking_until: Option<Instant> = None;
        let mut is_clutch_toggled = false;
        let mut was_clutch_key_down = false;
        let mut last_target_duck_apps: Vec<String> = Vec::new();

        while is_running.load(Ordering::Relaxed) {
            let loop_start = Instant::now();

            // 1. Process any incoming user commands from UI
            while let Ok(cmd) = command_receiver.try_recv() {
                match cmd {
                    AudioEngineCommand::SetAppVolume { pid, volume } => {
                        baseline_volumes.insert(pid, volume);
                        #[cfg(windows)]
                        Self::raw_set_session_volume(pid, volume);
                    }
                    AudioEngineCommand::SetAppMute { pid, mute } => {
                        #[cfg(windows)]
                        Self::raw_set_session_mute(pid, mute);
                    }
                    AudioEngineCommand::TestDucking { duration } => {
                        test_ducking_until = Some(Instant::now() + duration);
                    }
                    AudioEngineCommand::ToggleClutch => {
                        is_clutch_toggled = !is_clutch_toggled;
                    }
                }
            }

            // 2. Read snapshot of config
            let config = if let Ok(c) = shared_config.lock() {
                c.clone()
            } else {
                AppConfig::default()
            };
            last_target_duck_apps = config.target_duck_apps.clone();

            // 3. Check Emergency Clutch Hotkey state (Win32 GetAsyncKeyState)
            let is_clutch_key_pressed = check_hotkey_pressed(&config.clutch_hotkey);
            let is_clutch_active = if config.clutch_hold_mode {
                is_clutch_key_pressed || is_clutch_toggled
            } else {
                if is_clutch_key_pressed && !was_clutch_key_down {
                    is_clutch_toggled = !is_clutch_toggled;
                }
                was_clutch_key_down = is_clutch_key_pressed;
                is_clutch_toggled
            };

            // 4. Scan active audio sessions from Windows WASAPI (cached PIDs to eliminate CPU spikes)
            let current_apps = Self::scan_active_audio_sessions(&config.target_duck_apps, &mut pid_cache);

            // 5. Identify active target game and sound level
            let (is_game_active, game_name, game_peak) = identify_target_game(&current_apps, &config.target_game);

            // 6. Decide whether audio ducking should trigger right now
            let is_test_active = test_ducking_until.map_or(false, |until| Instant::now() < until);
            let is_sound_trigger = config.ducking_enabled && is_game_active && (game_peak > config.trigger_threshold);
            let should_duck = is_test_active || is_clutch_active || is_sound_trigger;

            // Keep track of baseline volumes for apps ONLY when resting at 100% normal volume (prevents downward volume ratcheting)
            if !should_duck && (current_multiplier - 1.0).abs() < 1e-4 {
                for app in &current_apps {
                    baseline_volumes.insert(app.pid, app.volume);
                }
            }

            // 7. Calculate duck multiplier with smooth easing
            let duck_floor = (config.duck_level_percent / 100.0).clamp(0.0, 1.0);
            let target_multiplier = if should_duck { duck_floor } else { 1.0f32 };

            if (current_multiplier - target_multiplier).abs() > 0.01 {
                let speed = if should_duck { 0.20 } else { 0.08 }; // Fast attack, smooth decay
                current_multiplier += (target_multiplier - current_multiplier) * speed;
            } else {
                current_multiplier = target_multiplier;
            }

            // 8. Apply real attenuation to all target duck applications via WASAPI (only when multiplier changes)
            let should_apply = (current_multiplier - last_applied_multiplier).abs() > 0.002
                || (current_multiplier == 1.0 && (last_applied_multiplier - 1.0).abs() > 0.001);

            if should_apply {
                #[cfg(windows)]
                Self::apply_ducking_attenuation(&current_apps, &baseline_volumes, current_multiplier);
                last_applied_multiplier = current_multiplier;
            }

            // 9. Update telemetry for UI display
            let cpu_tier = if is_game_active {
                1
            } else if !current_apps.is_empty() {
                2
            } else {
                3
            };

            if let Ok(mut t) = telemetry.lock() {
                t.is_game_running = is_game_active;
                t.detected_game = game_name;
                t.game_peak_level = game_peak;
                t.is_ducking_active = should_duck;
                t.current_duck_multiplier = current_multiplier;
                t.active_apps = current_apps;
                t.cpu_tier = cpu_tier;
                t.is_clutch_active = is_clutch_active;
            }

            // 10. Adaptive sleep pacing for sub-0.10% CPU usage
            let sleep_duration = match cpu_tier {
                1 => Duration::from_millis(16),  // In-Game Battle: 60 Hz smooth tracking
                2 => Duration::from_millis(60),  // Background Music/Lobby: ~16 Hz
                _ => Duration::from_millis(250), // Idle: 4 Hz
            };

            let elapsed = loop_start.elapsed();
            if elapsed < sleep_duration {
                thread::sleep(sleep_duration - elapsed);
            }
        }

        // Clean termination: restore any ducked applications to 100% baseline volume
        #[cfg(windows)]
        {
            let exit_apps = Self::scan_active_audio_sessions(&last_target_duck_apps, &mut pid_cache);
            Self::apply_ducking_attenuation(&exit_apps, &baseline_volumes, 1.0f32);
        }

        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Com::CoUninitialize;
            CoUninitialize();
        }
    }

    #[cfg(windows)]
    fn scan_active_audio_sessions(
        target_duck_apps: &[String],
        pid_cache: &mut HashMap<u32, String>,
    ) -> Vec<AudioAppInfo> {
        use windows::core::Interface;
        use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            IAudioSessionControl2, IAudioSessionManager2, ISimpleAudioVolume,
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

                        let name = pid_cache.entry(pid).or_insert_with(|| {
                            get_process_name_by_pid(pid).unwrap_or_else(|| format!("PID {}", pid))
                        }).clone();

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
                        let clean_lower = lower.replace(".exe", "").replace(" ", "").replace("-", "");
                        let is_target = target_duck_apps.iter().any(|target| {
                            let t = target.to_lowercase().replace(".exe", "").replace(" ", "").replace("-", "");
                            clean_lower == t
                                || clean_lower.contains(&t)
                                || t.contains(&clean_lower)
                                || (t.contains("applemusic") && (clean_lower.contains("itunes") || clean_lower.contains("apple")))
                                || (t.contains("amazonmusic") && clean_lower.contains("amazon"))
                        });

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

            // Periodic cache hygiene: prune dead PIDs if cache exceeds 128 items
            if pid_cache.len() > 128 {
                let active_pids: std::collections::HashSet<u32> = results.iter().map(|a| a.pid).collect();
                pid_cache.retain(|k, _| active_pids.contains(k));
            }
        }

        results
    }

    #[cfg(not(windows))]
    fn scan_active_audio_sessions(
        _target_duck_apps: &[String],
        _pid_cache: &mut HashMap<u32, String>,
    ) -> Vec<AudioAppInfo> {
        Vec::new()
    }

    #[cfg(windows)]
    fn apply_ducking_attenuation(
        apps: &[AudioAppInfo],
        baseline_volumes: &HashMap<u32, f32>,
        multiplier: f32,
    ) {
        use windows::core::Interface;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            IAudioSessionControl2, IAudioSessionManager2, ISimpleAudioVolume,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

        let target_pids: Vec<u32> = apps
            .iter()
            .filter(|a| a.is_target_duck)
            .map(|a| a.pid)
            .collect();

        if target_pids.is_empty() {
            return;
        }

        unsafe {
            let enumerator: Result<IMMDeviceEnumerator, _> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            let Ok(enumerator) = enumerator else { return; };
            let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else { return; };
            let session_manager: Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None);
            let Ok(session_manager) = session_manager else { return; };
            let Ok(session_list) = session_manager.GetSessionEnumerator() else { return; };
            let count = session_list.GetCount().unwrap_or(0);

            for i in 0..count {
                if let Ok(session) = session_list.GetSession(i) {
                    if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                        let pid = control2.GetProcessId().unwrap_or(0);
                        if target_pids.contains(&pid) {
                            if let Ok(sav) = session.cast::<ISimpleAudioVolume>() {
                                let base = baseline_volumes.get(&pid).copied().unwrap_or(1.0);
                                let attenuated_vol = (base * multiplier).clamp(0.0, 1.0);
                                if let Ok(current_vol) = sav.GetMasterVolume() {
                                    if (current_vol - attenuated_vol).abs() > 0.005 {
                                        let _ = sav.SetMasterVolume(attenuated_vol, std::ptr::null());
                                    }
                                } else {
                                    let _ = sav.SetMasterVolume(attenuated_vol, std::ptr::null());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    fn raw_set_session_volume(target_pid: u32, volume: f32) {
        use windows::core::Interface;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            IAudioSessionControl2, IAudioSessionManager2, ISimpleAudioVolume,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

        unsafe {
            let enumerator: Result<IMMDeviceEnumerator, _> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            let Ok(enumerator) = enumerator else { return; };
            let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else { return; };
            let session_manager: Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None);
            let Ok(session_manager) = session_manager else { return; };
            let Ok(session_list) = session_manager.GetSessionEnumerator() else { return; };
            let count = session_list.GetCount().unwrap_or(0);

            for i in 0..count {
                if let Ok(session) = session_list.GetSession(i) {
                    if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                        if control2.GetProcessId().unwrap_or(0) == target_pid {
                            if let Ok(sav) = session.cast::<ISimpleAudioVolume>() {
                                let _ = sav.SetMasterVolume(volume.clamp(0.0, 1.0), std::ptr::null());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    fn raw_set_session_mute(target_pid: u32, mute: bool) {
        use windows::core::Interface;
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            IAudioSessionControl2, IAudioSessionManager2, ISimpleAudioVolume,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

        unsafe {
            let enumerator: Result<IMMDeviceEnumerator, _> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            let Ok(enumerator) = enumerator else { return; };
            let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else { return; };
            let session_manager: Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None);
            let Ok(session_manager) = session_manager else { return; };
            let Ok(session_list) = session_manager.GetSessionEnumerator() else { return; };
            let count = session_list.GetCount().unwrap_or(0);

            for i in 0..count {
                if let Ok(session) = session_list.GetSession(i) {
                    if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                        if control2.GetProcessId().unwrap_or(0) == target_pid {
                            if let Ok(sav) = session.cast::<ISimpleAudioVolume>() {
                                let _ = sav.SetMute(BOOL::from(mute), std::ptr::null());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn identify_target_game(apps: &[AudioAppInfo], target_game_setting: &str) -> (bool, String, f32) {
    let known_esports = [
        "cs2", "csgo", "valorant", "r5apex", "apex", "cod", "warzone", "fortnite",
        "overwatch", "rainbowsix", "pubg", "deadlock", "destiny2", "halo",
    ];

    // Explicit specific game match
    if !target_game_setting.starts_with("Auto") {
        let search_keys: &[&str] = if target_game_setting.contains("Counter-Strike") {
            &["cs2", "csgo"]
        } else if target_game_setting.contains("VALORANT") {
            &["valorant"]
        } else if target_game_setting.contains("Apex") {
            &["r5apex", "apex"]
        } else if target_game_setting.contains("Call of Duty") {
            &["cod", "warzone"]
        } else if target_game_setting.contains("Fortnite") {
            &["fortnite"]
        } else if target_game_setting.contains("Overwatch") {
            &["overwatch"]
        } else if target_game_setting.contains("Rainbow Six") {
            &["rainbowsix"]
        } else {
            &[]
        };

        if !search_keys.is_empty() {
            if let Some(game) = apps.iter().find(|a| {
                let low = a.name.to_lowercase();
                search_keys.iter().any(|&k| low.contains(k))
            }) {
                return (true, game.name.clone(), game.peak);
            }
        } else {
            let needle = target_game_setting.to_lowercase().replace(".exe", "");
            if let Some(game) = apps.iter().find(|a| a.name.to_lowercase().contains(&needle)) {
                return (true, game.name.clone(), game.peak);
            }
        }
    }

    // Auto-detection: First check recognized esports games
    if let Some(game) = apps.iter().find(|a| {
        let low = a.name.to_lowercase();
        known_esports.iter().any(|g| low.contains(g))
    }) {
        return (true, game.name.clone(), game.peak);
    }

    // Auto-detection fallback: Any audio session producing sound that is NOT in duck targets
    if let Some(active_session) = apps.iter().filter(|a| !a.is_target_duck && a.peak > 0.02).max_by(|a, b| a.peak.total_cmp(&b.peak)) {
        return (true, format!("{} (Active Audio)", active_session.name), active_session.peak);
    }

    (false, "No Active Game (Standing by)".to_string(), 0.0)
}

pub fn check_hotkey_pressed(hotkey_name: &str) -> bool {
    #[cfg(windows)]
    {
        if let Some(vk) = hotkey_to_vk(hotkey_name) {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
                let state = GetAsyncKeyState(vk);
                return (state as u16 & 0x8000) != 0;
            }
        }
    }
    false
}

#[cfg(windows)]
pub fn hotkey_to_vk(name: &str) -> Option<i32> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let trimmed = name.trim().to_lowercase();
    match trimmed.as_str() {
        "caps lock" | "capslock" | "caps" => Some(VK_CAPITAL.0 as i32),
        "f1" => Some(VK_F1.0 as i32),
        "f2" => Some(VK_F2.0 as i32),
        "f3" => Some(VK_F3.0 as i32),
        "f4" => Some(VK_F4.0 as i32),
        "f5" => Some(VK_F5.0 as i32),
        "f6" => Some(VK_F6.0 as i32),
        "f7" => Some(VK_F7.0 as i32),
        "f8" => Some(VK_F8.0 as i32),
        "f9" => Some(VK_F9.0 as i32),
        "f10" => Some(VK_F10.0 as i32),
        "f11" => Some(VK_F11.0 as i32),
        "f12" => Some(VK_F12.0 as i32),
        "space" => Some(VK_SPACE.0 as i32),
        "tab" => Some(VK_TAB.0 as i32),
        "left alt" | "alt" => Some(VK_LMENU.0 as i32),
        "left ctrl" | "ctrl" => Some(VK_LCONTROL.0 as i32),
        "left shift" | "shift" => Some(VK_LSHIFT.0 as i32),
        "mouse 4" | "xbutton1" => Some(VK_XBUTTON1.0 as i32),
        "mouse 5" | "xbutton2" => Some(VK_XBUTTON2.0 as i32),
        "middle click" | "mbutton" => Some(VK_MBUTTON.0 as i32),
        "c" => Some(b'C' as i32),
        "v" => Some(b'V' as i32),
        "x" => Some(b'X' as i32),
        "z" => Some(b'Z' as i32),
        "t" => Some(b'T' as i32),
        "q" => Some(b'Q' as i32),
        _ => {
            // Single char check
            if trimmed.len() == 1 {
                let ch = trimmed.chars().next().unwrap();
                if ch.is_ascii_alphabetic() {
                    return Some(ch.to_ascii_uppercase() as i32);
                }
                if ch.is_ascii_digit() {
                    return Some(ch as i32);
                }
            }
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_matching() {
        let apps = vec![
            AudioAppInfo {
                pid: 100,
                name: "cs2.exe".to_string(),
                volume: 1.0,
                peak: 0.45,
                is_muted: false,
                is_target_duck: false,
            },
            AudioAppInfo {
                pid: 200,
                name: "Spotify.exe".to_string(),
                volume: 0.8,
                peak: 0.1,
                is_muted: false,
                is_target_duck: true,
            },
        ];

        // Specific dropdown selection
        let (active, name, peak) = identify_target_game(&apps, "Counter-Strike 2 (cs2)");
        assert!(active);
        assert_eq!(name, "cs2.exe");
        assert_eq!(peak, 0.45);

        // Auto detection
        let (active_auto, name_auto, _) = identify_target_game(&apps, "Auto (Any Active Game)");
        assert!(active_auto);
        assert_eq!(name_auto, "cs2.exe");
    }

    #[test]
    #[cfg(windows)]
    fn test_hotkey_resolution() {
        assert!(hotkey_to_vk("Caps Lock").is_some());
        assert!(hotkey_to_vk("F5").is_some());
        assert!(hotkey_to_vk("Space").is_some());
        assert!(hotkey_to_vk("Mouse 4").is_some());
        assert!(hotkey_to_vk("NonexistentKey999").is_none());
    }
}
