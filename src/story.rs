use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use dioxus_fullstack::prelude::*;
use futures::future::join_all;
use reqwest::Result;
use serde::{Deserialize, Serialize};

use crate::comment::{get_comment, Comment, CommentComp};
use crate::fetch_hacker_news::fetch_hacker_news;

#[derive(PartialEq, Props, Debug, Clone, Deserialize, Serialize)]
pub struct StoryPageData {
    #[serde(flatten)]
    pub item: StoryItem,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct StoryItem {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    pub descendants: i64,
    pub score: i64,
    #[serde(default)]
    pub kids: Vec<i64>,
    pub r#type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
}

#[derive(Debug)]
pub enum StoryPreviewState {
    Unset,
    Loading,
    Loaded(StoryPageData),
}

#[allow(non_snake_case)]
#[inline_props]
pub fn StoryListing(cx: Scope, story: StoryItem) -> Element {
    let StoryItem {
        id, title, time, ..
    } = story;
    let preview_state = use_shared_state::<StoryPreviewState>(cx).unwrap();
    let cache_state = use_ref(cx, || None);
    let text = story.text.clone().unwrap_or_default();

    cx.render(rsx! {
        div {
            class: "w-full px-2 py-2 bg-gray-100 my-2",
            p {
                class: "text-slate-500 text-xl",
                onmouseenter: move |_evt| {
                    resolve_story(cache_state.clone(), preview_state.clone(), story.id)
                },
                format!("{:?}: {:}",  id,  title)
             },
             div{
                class: "py-1",
                dangerous_inner_html: "{text}"
             },
             div {
                class: "flex align-middle py-2",
                span {
                    class: "text-base text-black",
                    format!("time: {:}", time.with_timezone(&Local).format("%Y-%m-%d %H:%M"))
                }
                span {
                   class: "ml-3 text-base",
                   format!("kids: {:?}", story.kids.len())
                }
             }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Stories<'a>(
    cx: Scope<'a>,
    stories: std::cell::Ref<'a, std::result::Result<Vec<StoryItem>, ServerFnError>>,
) -> Element {
    cx.render(rsx! {
        div {
            class: "w-1/3 flex-grow-0 flex-shrink-0",

            match stories.as_deref() {
                Ok(list) => {
                    rsx!(
                        ol {
                            list.iter().map(|item| render!{
                                StoryListing{
                                    story: item.clone(),
                                }
                            })
                        }
                    )
                },
                Err(e) => {
                    rsx!(
                        format!("{:}", e)
                    )
                }
            }
        }
    })
}

#[allow(non_snake_case)]
pub fn StoryPreview(cx: Scope) -> Element {
    let preview_data = use_shared_state::<StoryPreviewState>(cx).unwrap();
    let preview_state = preview_data.read();

    let content = match &*preview_state {
        StoryPreviewState::Loading | StoryPreviewState::Unset => rsx!(div{"loading..."}),
        StoryPreviewState::Loaded(data) => {
            let comments = &data.comments;

            rsx!(
                div {
                    class: "flex flex-col",
                    h3 {
                        format!("{:}", data.item.title)
                    },
                    for comment in comments {
                        CommentComp{comment: comment.clone()}
                    }
                }
            )
        }
    };

    cx.render(rsx! {
        div {
            class: "w-2/3",
            content
        }
    })
}

async fn resolve_story(
    cache_state: UseRef<Option<StoryPageData>>,
    preview_state: UseSharedState<StoryPreviewState>,
    story_id: i64,
) {
    if let Some(story) = &*cache_state.read() {
        *preview_state.write() = StoryPreviewState::Loaded(story.clone());
        return;
    }

    *preview_state.write() = StoryPreviewState::Loading;
    let story = get_story(story_id).await;

    match story {
        Ok(story) => {
            *preview_state.write() = StoryPreviewState::Loaded(story.clone());
            *cache_state.write() = Some(story);
        }
        Err(_e) => {
            *preview_state.write() = StoryPreviewState::Unset;
        }
    }
}

pub async fn get_story_preview(id: i64) -> Result<StoryItem> {
    fetch_hacker_news(&format!("item/{}.json", id)).await
}

pub async fn get_stories(count: usize) -> std::result::Result<Vec<StoryItem>, ServerFnError> {
    let stories_ids = &fetch_hacker_news::<Vec<i64>>("topstories.json").await?[..count];

    let stories_futures = stories_ids.iter().map(|&id| get_story_preview(id));
    let stories = join_all(stories_futures)
        .await
        .into_iter()
        .filter_map(|it| it.ok())
        .collect();
    Ok(stories)
}

pub async fn get_story(id: i64) -> Result<StoryPageData> {
    let mut story: StoryPageData = fetch_hacker_news(&format!("item/{}.json", id)).await?;
    let comment_futures = story.item.kids.iter().map(|&id| get_comment(id));
    story.comments = join_all(comment_futures)
        .await
        .into_iter()
        .filter_map(|it| it.ok())
        .collect();
    Ok(story)
}
