use dioxus::html::FileData;

pub mod music;

pub enum MusicMessage {
    Play(FileData),
    Pause,
    Continue,
    Stop,
    Seek,
    Quit
}