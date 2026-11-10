use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::Command;

const REPO_OWNER: &str = "Mahim0007";
const REPO_NAME: &str = "Soundlock---Esports-Audio-Optimizer";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate(String),
    Available {
        version: String,
        download_url: String,
        notes: String,
    },
    Downloading(f32), // 0.0 to 1.0
    ReadyToRestart {
        new_exe: PathBuf,
        current_exe: PathBuf,
    },
    Failed(String),
}

#[derive(Clone)]
pub struct UpdateManager {
    pub status: Arc<Mutex<UpdateStatus>>,
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }
}

impl UpdateManager {
    pub fn new() -> Self {
        let mgr = Self::default();
        // Automatically check for updates on startup in background
        mgr.check_for_updates();
        mgr
    }

    pub fn current_version(&self) -> &'static str {
        CURRENT_VERSION
    }

    pub fn check_for_updates(&self) {
        let status = Arc::clone(&self.status);
        if let Ok(mut s) = status.lock() {
            *s = UpdateStatus::Checking;
        }

        thread::Builder::new()
            .name("soundlock-update-checker".to_string())
            .spawn(move || {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    REPO_OWNER, REPO_NAME
                );

                #[cfg(windows)]
                let mut cmd = Command::new("curl.exe");
                #[cfg(not(windows))]
                let mut cmd = Command::new("curl");

                cmd.args(&[
                    "-s",
                    "--max-time", "6",
                    "-H", "User-Agent: SoundLock-Suite",
                    "-H", "Accept: application/vnd.github.v3+json",
                    &url,
                ]);

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }

                let output = match cmd.output() {
                    Ok(o) if o.status.success() => o.stdout,
                    _ => {
                        if let Ok(mut s) = status.lock() {
                            *s = UpdateStatus::UpToDate(CURRENT_VERSION.to_string());
                        }
                        return;
                    }
                };

                let Ok(json_str) = String::from_utf8(output) else {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::UpToDate(CURRENT_VERSION.to_string());
                    }
                    return;
                };

                let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) else {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::UpToDate(CURRENT_VERSION.to_string());
                    }
                    return;
                };

                let tag_name = val["tag_name"].as_str().unwrap_or("").trim();
                let clean_remote = tag_name.trim_start_matches(|c| c == 'v' || c == 'V');

                if is_newer_version(clean_remote, CURRENT_VERSION) {
                    // Locate downloadable .exe asset
                    let mut download_url = String::new();
                    if let Some(assets) = val["assets"].as_array() {
                        for asset in assets {
                            if let Some(name) = asset["name"].as_str() {
                                if name.ends_with(".exe") {
                                    if let Some(dl) = asset["browser_download_url"].as_str() {
                                        download_url = dl.to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Fallback to release page if no direct .exe asset
                    if download_url.is_empty() {
                        download_url = val["html_url"].as_str().unwrap_or("").to_string();
                    }

                    let notes = val["body"].as_str().unwrap_or("").trim().to_string();

                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Available {
                            version: clean_remote.to_string(),
                            download_url,
                            notes,
                        };
                    }
                } else {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::UpToDate(CURRENT_VERSION.to_string());
                    }
                }
            })
            .expect("Failed to spawn update checker thread");
    }

    pub fn download_and_install(&self, download_url: String) {
        let status = Arc::clone(&self.status);
        if let Ok(mut s) = status.lock() {
            *s = UpdateStatus::Downloading(0.2);
        }

        thread::Builder::new()
            .name("soundlock-update-downloader".to_string())
            .spawn(move || {
                let temp_dir = std::env::temp_dir();
                let new_exe = temp_dir.join("SoundLock_New.exe");

                #[cfg(windows)]
                let mut cmd = Command::new("curl.exe");
                #[cfg(not(windows))]
                let mut cmd = Command::new("curl");

                cmd.args(&[
                    "-L",
                    "-s",
                    "--max-time", "60",
                    "-H", "User-Agent: SoundLock-Suite",
                    "-o", new_exe.to_string_lossy().as_ref(),
                    &download_url,
                ]);

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }

                let success = match cmd.status() {
                    Ok(s) => s.success(),
                    Err(_) => false,
                };

                if success && new_exe.exists() {
                    if let Ok(current_exe) = std::env::current_exe() {
                        if let Ok(mut s) = status.lock() {
                            *s = UpdateStatus::ReadyToRestart {
                                new_exe,
                                current_exe,
                            };
                        }
                        return;
                    }
                }

                if let Ok(mut s) = status.lock() {
                    *s = UpdateStatus::Failed("Download failed or executable verification error.".to_string());
                }
            })
            .expect("Failed to spawn update downloader thread");
    }

    pub fn apply_and_restart(&self, new_exe: &Path, current_exe: &Path) -> Result<(), String> {
        let temp_dir = std::env::temp_dir();
        let bat_path = temp_dir.join("soundlock_updater.bat");

        let bat_content = format!(
            "@echo off\r\n\
            timeout /t 1 /nobreak > nul\r\n\
            copy /y \"{}\" \"{}\" > nul\r\n\
            del \"{}\" > nul\r\n\
            start \"\" \"{}\"\r\n\
            del \"%~f0\"\r\n",
            new_exe.to_string_lossy(),
            current_exe.to_string_lossy(),
            new_exe.to_string_lossy(),
            current_exe.to_string_lossy(),
        );

        std::fs::write(&bat_path, bat_content).map_err(|e| e.to_string())?;

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let mut cmd = Command::new("cmd.exe");
            cmd.args(&["/c", bat_path.to_string_lossy().as_ref()]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.spawn();
        }

        Ok(())
    }
}

pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_nums = |s: &str| -> Vec<u32> {
        s.split('.')
            .map(|part| part.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let r = parse_nums(remote);
    let c = parse_nums(current);

    for i in 0..r.len().max(c.len()) {
        let r_val = *r.get(i).unwrap_or(&0);
        let c_val = *c.get(i).unwrap_or(&0);
        if r_val > c_val {
            return true;
        } else if r_val < c_val {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("0.1.1", "0.1.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.0.9", "0.1.0"));
    }
}

