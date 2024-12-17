use crate::common::model::Score;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct QnSampleInterval {
    pub interval_id: i32,
    pub start_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QnSample {
    pub item_id: i32,
    pub interval: QnSampleInterval,
    pub sample_time: i64,
    pub submission_time: i64,
    pub rank_top: Option<i32>,
    pub rank_new: Option<i32>,
    pub upvotes: i32,
    pub upvote_share: f32,
}

impl<'r> FromRow<'r, SqliteRow> for QnSample {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(QnSample {
            item_id: row.try_get("item_id")?,
            interval: QnSampleInterval {
                interval_id: row.try_get("interval_id")?,
                start_time: row.try_get("start_time")?,
            },
            sample_time: row.try_get("sample_time")?,
            submission_time: row.try_get("submission_time")?,
            rank_top: row.try_get("rank_top")?,
            rank_new: row.try_get("rank_new")?,
            upvotes: row.try_get("upvotes")?,
            upvote_share: row.try_get("upvote_share")?,
        })
    }
}

pub struct QnSampleWithPrediction {
    pub sample: QnSample,
    pub expected_upvotes: f32,
    pub expected_upvote_share: f32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct QnStats {
    pub item_id: i32,
    pub updated_at: i64,
    pub sample_time: i64,
    pub submission_time: i64,
    pub cumulative_upvotes: i32,
    pub cumulative_expected_upvotes: f32,
}

impl Score for QnStats {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.submission_time) as f32 / 1000.0 / 60.0 / 60.0;
        let estimated_upvote_rate: f32 = if self.cumulative_expected_upvotes == 0.0 {
            // TODO: is this a sane default for 0.0 expected upvotes?
            // TODO: use a global prior with bayesian averaging for initial guess of upvote rate
            1.0
        } else {
            self.cumulative_upvotes as f32 / self.cumulative_expected_upvotes
        };
        (age_hours * estimated_upvote_rate).powf(0.8) / (age_hours + 2.0).powf(1.8)
    }
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct ItemWithRanks {
    pub item_id: i32,
    pub rank_top: i32,
    pub rank_new: i32,
}
