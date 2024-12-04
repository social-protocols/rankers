use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct VoteEvent {
    pub vote_event_id: i32,
    pub item_id: i32,
    pub vote: i32,
    pub created_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Item {
    pub item_id: i32,
    pub parent_id: Option<i32>,
    pub created_at: i64,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct ScoredItem {
    pub item_id: i32,
    pub score: f32,
}

pub trait Score {
    fn score(&self) -> f32;
}

// Hacker News

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct HnStatsObservation {
    pub item_id: i32,
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

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QnStatsObservation {
    pub item_id: i32,
    pub submission_time: i64,
    pub sample_time: i64,
    pub cumulative_upvotes: i32,
    pub cumulative_expected_upvotes: f32,
}

impl Score for QnStatsObservation {
    fn score(&self) -> f32 {
        // TODO: sane default for 0.0 expected upvotes
        let age_hours = (self.sample_time - self.submission_time) as f32 / 60.0 / 60.0;
        let estimated_upvote_rate: f32 = if self.cumulative_expected_upvotes != 0.0 {
            self.cumulative_upvotes as f32 / self.cumulative_expected_upvotes
        } else {
            0.0
        };
        let numerator = (age_hours * estimated_upvote_rate).powf(0.8);
        let denominator = (age_hours + 2.0).powf(1.8);
        numerator / denominator
    }
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct ItemWithRanks {
    pub item_id: i32,
    pub sample_time: i64,
    pub rank_top: i32,
}
