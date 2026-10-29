use std::{fs::File, sync::Mutex};

use rodio::{Decoder, MixerDeviceSink, Player};
use tauri::{Manager, State};

struct AppState {
    _handler: MixerDeviceSink,
    player: Player,
    playing: bool,
}

#[tauri::command]
fn play_music(state: State<'_, Mutex<AppState>>, path: String) -> Result<(), String> {
    let source = Decoder::try_from(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.player.clear();
    state.player.append(source);
    state.player.play();
    state.playing = true;

    Ok(())
}

#[tauri::command]
fn stop(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.player.clear();
    state.playing = false;

    Ok(())
}

#[tauri::command]
fn switch(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if !state.player.empty() {
        if state.playing {
            state.player.pause();
            state.playing = false;
        } else {
            state.player.play();
            state.playing = true;
        }
    }

    Ok(())
}

#[tauri::command]
fn set_volume(state: State<'_, Mutex<AppState>>, volume: f32) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.player.set_volume(volume);

    Ok(())
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
            app.manage(Mutex::new(AppState {
                _handler,
                player,
                playing: false,
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            play_music, stop, switch, set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
