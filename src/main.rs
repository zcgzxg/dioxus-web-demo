mod comment;
mod fetch_hacker_news;
mod story;

use dioxus::prelude::*;
use dioxus_fullstack::{launch::LaunchBuilder, prelude::*};
use story::{get_stories, Stories, StoryItem, StoryPreview, StoryPreviewState};

fn main() {
    LaunchBuilder::new(App).launch();
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    use_shared_state_provider(cx, || StoryPreviewState::Unset);
    let stories = use_server_future(cx, (), |()| async { get_first_10_stories().await })?.value();

    cx.render(rsx! {
        div {
            class: "w-full flex-row flex",
            Stories{
                stories: stories
            }
            StoryPreview{}
        }
    })
}

#[server]
async fn get_first_10_stories() -> Result<Vec<StoryItem>, ServerFnError> {
    get_stories(5).await
}
