use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn AudioPlayer(playing: Signal<bool>, audio_url: Signal<Option<String>>) -> Element {
    if let Some(src) = audio_url() {
        render_audio(&src, playing.clone(), audio_url.clone())
    } else {
        rsx!(div { id: "audio-player" })
    }
}

fn render_audio(src: &str, mut playing: Signal<bool>, mut audio_url: Signal<Option<String>>) -> Element {
    rsx! {
        div { id: "audio-player",
            audio {
                id: "my_audio",
                controls: false,
                style: "display:none",
                autoplay: true,
                src: "{src}",
                onplay: move |_| playing.set(true),
                onpause: move |_| playing.set(false),
                onended: move |_| {
                    playing.set(false);
                    audio_url.set(None);
                },
            }
        }
    }
}
