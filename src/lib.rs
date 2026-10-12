pub mod music;

pub enum MusicMessage {
    Play(rfd::FileHandle),
    Pause,
    Continue,
    Stop,
    Seek,
    Quit
}