use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use futures::future::join_all;
use reqwest::Result;
use serde::{Deserialize, Serialize};

use crate::fetch_hacker_news::fetch_hacker_news;

const MAX_COMMENT_DEPTH: usize = 4;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    #[serde(default)]
    pub by: String,
    pub text: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    #[serde(default)]
    pub sub_comments: Vec<Comment>,
    pub r#type: String,
}

#[async_recursion::async_recursion(?Send)]
async fn get_comment_with_depth(id: i64, depth: usize) -> Result<Comment> {
    let mut comment: Comment = fetch_hacker_news(&format!("item/{}.json", id)).await?;

    if depth > 0 {
        let comment_futures = comment
            .kids
            .iter()
            .map(|&id| get_comment_with_depth(id, depth - 1));
        comment.sub_comments = join_all(comment_futures)
            .await
            .into_iter()
            .filter_map(|it| it.ok())
            .collect();
    }

    Ok(comment)
}

pub async fn get_comment(id: i64) -> Result<Comment> {
    get_comment_with_depth(id, MAX_COMMENT_DEPTH).await
}

#[allow(non_snake_case)]
#[inline_props]
pub fn CommentComp(cx: Scope, comment: Comment) -> Element {
    let Comment { text, .. } = comment;

    cx.render(rsx! {
        div {
            class: "text-red-500 text-xl bg-slate-400 px-2 py-2",

            "{text}"
        }
    })
}
