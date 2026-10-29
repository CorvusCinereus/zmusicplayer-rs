use std::{fs::File, sync::Mutex, time::Duration};

use rodio::{Decoder, MixerDeviceSink, Player, Source};
use tauri::{Manager, State};

struct AppState {
    _handler: MixerDeviceSink,
    player: Player,
}

#[tauri::command]
fn play_music(state: State<'_, Mutex<AppState>>, path: String) -> Result<u64, String> {
    let source = Decoder::try_from(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let d = source.total_duration().unwrap_or_default().as_secs();

    let state = state.lock().map_err(|e| e.to_string())?;
    state.player.clear();
    state.player.append(source);
    state.player.play();

    Ok(d)
}

#[tauri::command]
fn stop(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.player.clear();

    Ok(())
}

#[tauri::command]
fn switch(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    if !state.player.empty() {
        if state.player.is_paused() {
            state.player.play();
        } else {
            state.player.pause();
        }
    }

    Ok(())
}

#[tauri::command]
fn set_volume(state: State<'_, Mutex<AppState>>, volume: f32) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.player.set_volume(volume.clamp(0.0, 1.0));

    Ok(())
}

#[tauri::command]
fn seek(state: State<'_, Mutex<AppState>>, pos: u64) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .player
        .try_seek(Duration::from_secs(pos))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_pos(state: State<'_, Mutex<AppState>>) -> Result<(u64, bool), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok((state.player.get_pos().as_secs(), state.player.empty()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _handler = rodio::DeviceSinkBuilder::open_default_sink().expect("open default sink");
    let player = Player::connect_new(&_handler.mixer());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState { _handler, player }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            play_music, stop, switch, set_volume, seek, get_pos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
