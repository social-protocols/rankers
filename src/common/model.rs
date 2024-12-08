use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct VoteEvent {
    pub vote_event_id: i32,
    pub item_id: i32,
    pub user_id: String,
    pub vote: i32,
    pub rank: Option<i32>,
    pub page: Option<RankingPage>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RankingPage {
    Newest,
    QualityNews,
    HackerNews,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Item {
    pub item_id: i32,
    pub parent_id: Option<i32>,
    pub author_id: String,
    pub created_at: i64,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct ScoredItem {
    pub item_id: i32,
    pub rank: i32,
    pub page: RankingPage,
    pub score: f32,
}

pub struct Observation<T> {
    pub sample_time: i64,
    pub data: T,
}

pub trait Score {
    fn score(&self) -> f32;
}
