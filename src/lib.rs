use std::{cell::RefCell, fs::File, path::PathBuf, rc::Rc, sync::{Arc, mpsc::Sender}, time::Duration, vec};

use rodio::Source;
use slint::{ModelRc, SharedString, ToSharedString, VecModel};

slint::include_modules!();

enum MusicMessage {
    Play(PathBuf),
    Stop,
    Pause,
    Seek(i32),
    Continue
}

struct AppState {
    musics: Rc<VecModel<SharedString>>,
    music_paths: Vec<PathBuf>,
    tx: Sender<MusicMessage>
}

pub struct ZMusicPlayer {
    ui: AppWindow,
    state: Rc<RefCell<AppState>>
}

// UI
impl ZMusicPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ui = AppWindow::new()?;

        let musics = Rc::new(VecModel::from(vec![]));
        ui.set_musics(ModelRc::from(musics.clone()));

        let ui_ = Arc::new(ui.as_weak());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut playing = false;
            let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
            let player = rodio::Player::connect_new(&handle.mixer());

            loop {
                if playing {
                    std::thread::sleep(Duration::from_secs_f32(0.2));
                    if !player.empty() {
                        match rx.try_recv() {
                            Ok(MusicMessage::Pause) => {
                                playing = false;
                                player.pause();
                            }
                            Ok(MusicMessage::Stop) => {
                                playing = false;
                                player.stop();
                            }
                            Ok(MusicMessage::Play(music)) => {
                                player.clear();
                                let file = File::open(music);
                                if let Ok(file) = file {
                                    let source = rodio::Decoder::try_from(file).unwrap();
                                    let total = source.total_duration().unwrap().as_secs();

                                    let _ = ui_.upgrade_in_event_loop(move |ui| {
                                        ui.set_end(format_duration(total).to_shared_string());
                                        ui.set_length(total as i32);
                                    });

                                    player.append(source);
                                    player.play();
                                }
                            }
                            Ok(MusicMessage::Seek(pos)) => {
                                match player.try_seek(Duration::from_secs(pos as u64)) {
                                    Err(e) => {
                                        std::thread::spawn(move || {
                                            rfd::MessageDialog::new().set_description(e.to_string()).show();
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            Ok(_) => {}
                            Err(_) => {}
                        }

                        let pos = player.get_pos().as_secs();
                        let _ = ui_.upgrade_in_event_loop(move |ui| {
                            ui.set_slider_value(pos as i32);
                            ui.set_now(format_duration(pos).to_shared_string());
                        });
                    } else {
                        let _ = ui_.upgrade_in_event_loop(move |ui| {
                            ui.invoke_next();
                        });
                    }
                } else {
                    match rx.recv() {
                        Ok(MusicMessage::Play(music)) => {
                            playing = true;
                            let file = File::open(music);
                            if let Ok(file) = file {
                                let source = rodio::Decoder::try_from(file).unwrap();
                                let total = source.total_duration().unwrap().as_secs();

                                let _ = ui_.upgrade_in_event_loop(move |ui| {
                                    ui.set_end(format_duration(total).to_shared_string());
                                    ui.set_length(total as i32);
                                });

                                player.append(source);
                                player.play();
                            }
                        }
                        Ok(MusicMessage::Continue) => {
                            if !player.empty() {
                                playing = true;
                                player.play();
                            } else {
                                let _ = ui_.upgrade_in_event_loop(move |ui| {
                                    ui.set_playing(false);
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        let state = Rc::new(RefCell::new(AppState {
            musics,
            music_paths: vec![],
            tx
        }));

        Ok(Self { ui, state })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.run()?;
        Ok(())
    }

    fn bind_choose_file(&self) -> &Self {
        let state = self.state.clone();
        self.ui.on_choose_file(move || {
            let state = state.clone();
            let _ = slint::spawn_local(async move {
                let files = rfd::AsyncFileDialog::new().add_filter("Music", &["mp3", "wav", "ogg", "flac", "m4a"]).pick_files().await;
                if let Some(files) = files {
                    for file in files {
                        let file_name = file.file_name();
                        state.borrow_mut().musics.push(file_name.to_shared_string());
                        state.borrow_mut().music_paths.push(file.path().to_path_buf());
                    }
                }
            });
        });
        self
    }

    fn bind_clear(&self) -> &Self {
        let ui = self.ui.as_weak();
        let state = self.state.clone();
        self.ui.on_clear(move || {
            state.borrow_mut().music_paths.clear();
            state.borrow_mut().musics.clear();

            let ui = ui.unwrap();
            if ui.get_playing() {
                let _ = state.borrow().tx.send(MusicMessage::Stop);
            }
        });
        self
    }

    fn bind_play_music(&self) -> &Self {
        let state = self.state.clone();
        self.ui.on_play_music(move |index| {
            let _ = state.borrow().tx.send(MusicMessage::Play(state.borrow().music_paths[index as usize].clone()));
        });
        self
    }

    fn bind_clicked_play_button(&self) -> &Self {
        let ui = self.ui.as_weak();
        let state = self.state.clone();
        self.ui.on_clicked_play_button(move || {
            let ui = ui.unwrap();
            if ui.get_playing() {
                let _ = state.borrow().tx.send(MusicMessage::Continue);
            } else {
                let _ = state.borrow().tx.send(MusicMessage::Pause);
            }
        });
        self
    }

    fn bind_remove(&self) -> &Self {
        let state = self.state.clone();
        self.ui.on_remove(move |index| {
            let _ = state.borrow().tx.send(MusicMessage::Stop);
            state.borrow_mut().music_paths.remove(index as usize);
            state.borrow_mut().musics.remove(index as usize);
        });
        self
    }

    fn bind_seek(&self) -> &Self {
        let state = self.state.clone();
        self.ui.on_seek(move |pos| {
            let _ = state.borrow().tx.send(MusicMessage::Seek(pos));
        });
        self
    }

    pub fn bind_events(&self) {
        self.bind_choose_file()
            .bind_clear()
            .bind_play_music()
            .bind_remove()
            .bind_seek()
            .bind_clicked_play_button();
    }
}

fn format_duration(d: u64) -> String {
    format!("{:02}:{:02}:{:02}", d / 3600, (d % 3600) / 60, d % 60)
}