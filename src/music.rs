use std::{fs::File, sync::mpsc::Receiver, time::Duration};

use dioxus::signals::{ReadableExt, Signal, SyncStorage, WritableExt};
use rodio::Source;

use crate::MusicMessage;

pub fn music_backend(
    rx: Receiver<MusicMessage>,
    mut playing: Signal<bool, SyncStorage>,
    mut slider_dragging: Signal<bool, SyncStorage>,
    mut slider_value: Signal<i32, SyncStorage>,
    mut slider_length: Signal<i32, SyncStorage>,
    mut now: Signal<String, SyncStorage>,
    mut end: Signal<String, SyncStorage>,
    mut current_index: Signal<i32, SyncStorage>
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::Player::connect_new(&handle.mixer());

    loop {
        if playing() {
            std::thread::sleep(Duration::from_secs_f32(0.2));

            if player.empty() {
                playing.set(false);

                *current_index.write() += 1;

                continue;
            }

            match rx.try_recv() {
                Ok(MusicMessage::Pause) => {
                    player.pause();
                    playing.set(false);
                }
                Ok(MusicMessage::Stop) => {
                    player.clear();
                    playing.set(false);
                }
                Ok(MusicMessage::Seek) => {
                    slider_dragging.set(false);
                    let _ = player.try_seek(Duration::from_secs(*slider_value.read() as u64));
                }
                Ok(MusicMessage::Play(music)) => {
                    player.clear();
                    let file = File::open(music.path().to_path_buf());
                    if let Ok(file) = file
                        && let Ok(source) = rodio::Decoder::try_from(file)
                    {
                        let d = source
                            .total_duration()
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs();
                        end.set(format_duration(d));
                        slider_length.set(d as i32);
                        player.append(source);
                        player.play();
                        playing.set(true);
                    }
                }
                _ => {}
            }

            if !slider_dragging() {
                let n = player.get_pos().as_secs();
                slider_value.set(n as i32);
                now.set(format_duration(n));
            }
        } else {
            match rx.recv() {
                Ok(MusicMessage::Play(music)) => {
                    let file = File::open(music.path().to_path_buf());
                    if let Ok(file) = file
                        && let Ok(source) = rodio::Decoder::try_from(file)
                    {
                        let d = source
                            .total_duration()
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs();
                        end.set(format_duration(d));
                        slider_length.set(d as i32);
                        player.append(source);
                        player.play();
                        playing.set(true);
                    }
                }
                Ok(MusicMessage::Continue) => {
                    if !player.empty() {
                        player.play();
                        playing.set(true);
                    }
                }
                _ => {}
            }
        }
    }
}

fn format_duration(d: u64) -> String {
    format!("{:02}:{:02}:{:02}", d / 3600, (d % 3600) / 60, d % 60)
}
