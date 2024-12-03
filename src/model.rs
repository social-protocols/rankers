use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VoteEvent {
    pub vote_event_id: i32,
    pub post_id: i32,
    pub vote: i32,
    pub created_at: i64,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct Post {
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub created_at: i64,
}

pub trait Score {
    fn score(&self) -> f32;
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ScoredPost {
    pub post_id: i32,
    pub score: f32,
}

// Hacker News

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct HnStatsObservation {
    pub post_id: i32,
    pub submission_time: i64,
    pub sample_time: i64,
    pub upvotes: i32,
}

impl Score for HnStatsObservation {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.submission_time) as f32 / 60.0 / 60.0;
        (self.upvotes as f32).powf(0.8) / (age_hours + 2.0).powf(1.8)
    }
}

// Quality News

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct StatsObservation {
    pub post_id: i32,
    pub sample_time: i64,
    pub cumulative_upvotes: i32,
    pub cumulative_expected_upvotes: f32,
    pub score: f32,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct PostWithRanks {
    pub post_id: i32,
    pub sample_time: i64,
    pub rank_top: i32,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct UpvotesByRank {
    pub rank_top: i32,
    pub avg_upvotes: f32,
}
