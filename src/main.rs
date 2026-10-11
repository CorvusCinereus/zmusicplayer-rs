// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use zmusicplayer::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = ZMusicPlayer::new()?;
    app.bind_events();
    app.run()?;

    Ok(())
}
