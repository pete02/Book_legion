use dioxus::prelude::*;
pub const FAVICON: Asset = asset!("/assets/favicon.ico");
pub const MAIN_CSS: Asset = asset!("/assets/main.css");
pub const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
pub const PAUSE: Asset = asset!("/assets/pause.png");
pub const PLAY: Asset = asset!("/assets/play.png");
pub const FORWARD: Asset = asset!("/assets/forward.png");

#[cfg(feature = "mock")]
pub const MOCK_COVER: Asset = asset!("/test_assets/test.jpg");

#[cfg(feature = "mock")]
pub const MOCK_MP3: Asset = asset!("/test_assets/test.mp3");