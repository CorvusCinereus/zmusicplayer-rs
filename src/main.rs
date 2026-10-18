use std::{
    cell::RefCell,
    sync::{Arc, mpsc::Sender},
};

use dioxus::prelude::*;
use zmusicplayer::{MusicMessage, music::music_backend};

const CSS: Asset = asset!("/assets/md3.css");
static mut TX: Option<Arc<Sender<MusicMessage>>> = None;

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop!(
            use dioxus::desktop::{Config, WindowBuilder};
            Config::new().with_window(WindowBuilder::new().with_decorations(false))
        ))
        .launch(App);
    unsafe {
        if let Some(Some(tx)) = (&raw const TX).as_ref() {
            let _ = tx.send(MusicMessage::Quit);
        }
    }
}

#[component]
fn App() -> Element {
    let playing = use_signal_sync(|| false);
    let mut slider_dragging = use_signal_sync(|| false);
    let mut slider_value = use_signal_sync(|| 0);
    let now = use_signal_sync(|| "00:00:00".to_string());
    let end = use_signal_sync(|| "00:00:00".to_string());
    let length = use_signal_sync(|| 10);
    let mut current_index = use_signal_sync(|| -2);
    let mut musics: Signal<Arc<RefCell<Vec<dioxus_elements::FileData>>>> =
        use_signal(|| Arc::new(RefCell::new(vec![])));

    let tx = use_hook(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = music_backend(
                rx,
                playing,
                slider_dragging,
                slider_value,
                length,
                now,
                end,
                current_index,
            );
        });

        let tx = Arc::new(tx);

        unsafe {
            TX = Some(tx.clone());
        }

        tx
    });

    use_effect({
        let tx = tx.clone();
        move || {
            if current_index() > -2 {
                if current_index() < 0 {
                    current_index.set(musics.peek().borrow().len() as i32 - 1);
                } else if current_index() as usize == musics.peek().borrow().len() {
                    current_index.set(0);
                }

                let _ = tx.send(MusicMessage::Play(
                    musics.peek().borrow()[current_index() as usize].clone(),
                ));
            }
        }
    });

    rsx! {
        Stylesheet { href: CSS }

        div { class: "main",
            my_decorations {}
            div { class: "appbar",
                input {
                    class: "file_input",
                    r#type: "file",
                    accept: "music/*.mp3,*.wav,*.flac",
                    multiple: true,
                    onchange: move |evt| async move {
                        musics.write().borrow_mut().append(&mut evt.files());
                    },
                }
                button {
                    class: "clear_button",
                    onclick: {
                        let tx = tx.clone();
                        move |_| {
                            musics.write().borrow_mut().clear();
                            current_index.set(-2);
                            let _ = tx.send(MusicMessage::Stop);
                        }
                    },
                    "清空"
                }
            }
            div { class: "list_view",
                for (index , item) in musics.read().borrow().iter().enumerate() {
                    div {
                        ondoubleclick: move |_| {
                            current_index.set(index as i32);
                        },
                        label { class: if index == current_index() as usize { "playing_item" } else { "item_label" },
                            {item.name()}
                        }
                        button {
                            class: "delete_item",
                            onclick: {
                                let tx = tx.clone();
                                move |_| {
                                    current_index.set(-2);
                                    let _ = tx.send(MusicMessage::Stop);
                                    musics.write().borrow_mut().remove(index);
                                }
                            },
                            "☓"
                        }
                    }
                }
            }
            div { class: "process_div",
                label { "{now}" }
                input {
                    class: "slider",
                    r#type: "range",
                    min: "0",
                    max: "{length}",
                    value: "{slider_value}",
                    onpointerdown: move |_| {
                        slider_dragging.set(true);
                    },
                    onpointerup: {
                        let tx = tx.clone();
                        move |_| {
                            let _ = tx.send(MusicMessage::Seek);
                        }
                    },
                    oninput: move |e| {
                        if let Ok(value) = e.value().parse() {
                            slider_value.set(value);
                        }
                    },
                }
                label { "{end}" }
            }
            div { class: "bottom_div",
                button {
                    class: "previous_button",
                    onclick: move |_| {
                        if musics.read().borrow().len() > 0 {
                            *current_index.write() -= 1;
                        }
                    },
                    "⏮"
                }
                button {
                    class: if playing() { "play_button playing" } else { "play_button" },
                    onclick: {
                        let tx = tx.clone();
                        move |_| {
                            if playing() {
                                let _ = tx.send(MusicMessage::Pause);
                            } else {
                                let _ = tx.send(MusicMessage::Continue);
                            }
                        }
                    },
                    {if playing() { "⏸" } else { "▶" }}
                }
                button {
                    class: "next_button",
                    onclick: move |_| {
                        if musics.read().borrow().len() > 0 {
                            *current_index.write() += 1;
                        }
                    },
                    "⏭"
                }
            }
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn my_decorations() -> Element {
    use dioxus::desktop::use_window;
    rsx! {
        div { class: "decoration",
            label { class: "title", "ZMusicPlayer" }
            button {
                class: "min_button",
                onclick: move |_| {
                    use_window().set_minimized(true);
                },
                "−"
            }
            button {
                class: "max_button",
                onclick: move |_| {
                    let window = use_window();
                    window.set_maximized(!window.is_maximized());
                },
                "□"
            }
            button {
                class: "close_button",
                onclick: move |_| {
                    use_window().close();
                },
                "✕"
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn my_decorations() -> Element {
    rsx! {
        Title { "ZMusicPlayer" }
    }
}