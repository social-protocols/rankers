use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VoteEvent {
    pub vote_event_id: i32,
    pub post_id: i32,
    pub vote: i32,
    pub vote_event_time: i64,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct NewsAggregatorPost {
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub created_at: i64,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct HNPost {
    pub post_id: i32,
    pub upvotes: i32,
    pub age_hours: f32,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct HNScoredPost {
    pub post_id: i32,
    pub score: f32,
}

impl HNScoredPost {
    pub fn from_hn_post(post: HNPost) -> HNScoredPost {
        HNScoredPost {
            post_id: post.post_id,
            score: (post.upvotes as f32).powf(0.8) / (post.age_hours + 2.0).powf(1.8),
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct PostWithStats {
    pub post_id: i32,
    pub submission_time: i64,
    pub upvote_count: i32,
}
