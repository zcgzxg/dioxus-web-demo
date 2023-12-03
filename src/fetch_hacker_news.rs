use serde::de::DeserializeOwned;

pub const BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";

pub async fn fetch_hacker_news<T: DeserializeOwned>(path: &str) -> reqwest::Result<T> {
    reqwest::get(format!("{}/{}", BASE_URL, path))
        .await?
        .json::<T>()
        .await
}
