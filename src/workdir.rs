use std::path::PathBuf;

/// Determine the application working directory.
///
/// Priority:
///   1. `%APPDATA%\rusty-suite\rusty_qr`  — installation via suite_install
///   2. `%USERPROFILE%\rusty_qr`           — mode autonome (sans installeur)
pub fn work_dir() -> PathBuf {
    // 1. Installation propre : %APPDATA%\rusty-suite\ existe
    if let Ok(appdata) = std::env::var("APPDATA") {
        let suite = PathBuf::from(&appdata).join("rusty-suite");
        if suite.exists() {
            let dir = suite.join("rusty_qr");
            let _ = std::fs::create_dir_all(&dir);
            return dir;
        }
    }

    // 2. Mode autonome : %USERPROFILE%\rusty_qr
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir  = home.join("rusty_qr");
    let _    = std::fs::create_dir_all(&dir);
    dir
}
