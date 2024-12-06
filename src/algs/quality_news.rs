use crate::common::{
    error::AppError,
    model::{Observation, Score, ScoredItem},
};
use crate::util::now_utc_millis;
use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar, FromRow, Sqlite, Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QnStats {
    pub item_id: i32,
    pub submission_time: i64,
    pub upvotes: i32,
    pub expected_upvotes: f32,
}

impl Score for Observation<QnStats> {
    fn score(&self) -> f32 {
        // TODO: sane default for 0.0 expected upvotes
        let age_hours =
            (self.sample_time - self.data.submission_time) as f32 / 1000.0 / 60.0 / 60.0;
        let estimated_upvote_rate: f32 = if self.data.expected_upvotes != 0.0 {
            self.data.upvotes as f32 / self.data.expected_upvotes
        } else {
            0.0
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

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let latest_sample_time: i64 = query_scalar("select max(sample_time) from stats_history")
        .fetch_one(&mut **tx)
        .await?;

    let current_stats = get_stats_observations(tx, latest_sample_time).await?;

    let scored_items: Vec<ScoredItem> = current_stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .map(|item| ScoredItem {
            item_id: item.data.item_id,
            score: item.score(),
        })
        .collect();

    Ok(scored_items)
}

pub async fn sample_ranks(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    // TODO: come up with a better initialization logic

    let sample_time = now_utc_millis();
    println!("Sampling stats at: {:?}", sample_time);

    let any_items_exist: bool = sqlx::query_scalar("select exists (select 1 from item)")
        .fetch_one(&mut **tx)
        .await?;
    if !any_items_exist {
        println!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let any_ranks_recorded: bool = sqlx::query_scalar("select exists (select 1 from rank_history)")
        .fetch_one(&mut **tx)
        .await?;
    if !any_ranks_recorded {
        println!("No ranks recorded yet - Initializing...");
        sqlx::query(
            "
            with items_in_pool as (
                select
                      item_id
                    , unixepoch('subsec') * 1000 as sample_time
                    , row_number() over (order by created_at desc) as rank_top
                    , row_number() over (order by created_at desc) as rank_new
                from item
                limit 1500
            )
            insert into rank_history
            select * from items_in_pool
            ",
        )
        .fetch_all(&mut **tx)
        .await?;
        return Ok(axum::http::StatusCode::OK);
    }

    let previous_sample_time: i64 =
        sqlx::query_scalar("select max(sample_time) from stats_history")
            .fetch_one(&mut **tx)
            .await?;

    let new_stats: Vec<Observation<QnStats>> =
        calc_and_insert_newest_stats(tx, sample_time, previous_sample_time).await?;

    calc_and_insert_newest_ranks(tx, &new_stats).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn calc_and_insert_newest_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_sample_time: i64,
) -> Result<Vec<Observation<QnStats>>, AppError> {
    let previous_stats: Vec<Observation<QnStats>> =
        get_stats_observations(&mut *tx, previous_sample_time).await?;

    let sitewide_upvotes = get_sitewide_new_upvotes(&mut *tx, previous_sample_time).await?;

    let mut new_stats = Vec::<Observation<QnStats>>::new();

    for s in &previous_stats {
        let expected_upvote_share = get_expected_upvote_share(&mut *tx, s.data.item_id).await?;
        let new_upvotes = get_current_upvote_count(&mut *tx, s.data.item_id).await?;
        let new_expected_upvotes =
            s.data.expected_upvotes + (expected_upvote_share * (sitewide_upvotes as f32));

        let new_stat = Observation {
            sample_time,
            data: QnStats {
                item_id: s.data.item_id,
                submission_time: s.data.submission_time,
                upvotes: new_upvotes,
                expected_upvotes: new_expected_upvotes,
            },
        };

        query(
            "
            insert into stats_history (
                  item_id
                , sample_time
                , upvotes
                , expected_upvotes
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(new_stat.data.item_id)
        .bind(new_stat.sample_time)
        .bind(new_stat.data.upvotes)
        .bind(new_stat.data.expected_upvotes)
        .execute(&mut **tx)
        .await?;

        new_stats.push(new_stat);
    }

    Ok(new_stats)
}

async fn get_current_upvote_count(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i32,
) -> Result<i32, AppError> {
    let upvote_count: i32 = sqlx::query_scalar(
        "
        select count(*)
        from item i
        join vote v
        on i.item_id = v.item_id
        where vote = 1
        and i.item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(upvote_count)
}

async fn calc_and_insert_newest_ranks(
    tx: &mut Transaction<'_, Sqlite>,
    stats: &Vec<Observation<QnStats>>,
) -> Result<axum::http::StatusCode, AppError> {
    let newest_ranks: Vec<Observation<ItemWithRanks>> = stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .sorted_by(|(_, a), (_, b)| {
            a.data
                .submission_time
                .partial_cmp(&b.data.submission_time)
                .unwrap()
        })
        .enumerate()
        .map(|(rank_new, (rank_top, stat))| Observation {
            sample_time: stat.sample_time,
            data: ItemWithRanks {
                item_id: stat.data.item_id,
                rank_top: rank_top as i32 + 1,
                rank_new: rank_new as i32 + 1,
            },
        })
        .collect();

    for r in &newest_ranks {
        query(
            "
            insert into rank_history (
                  item_id
                , sample_time
                , rank_top
                , rank_new
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(r.data.item_id)
        .bind(r.sample_time)
        .bind(r.data.rank_top)
        .bind(r.data.rank_new)
        .execute(&mut **tx)
        .await?;
    }

    Ok(axum::http::StatusCode::OK)
}

async fn get_stats_observations(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<Observation<QnStats>>, AppError> {
    let stats_observations = sqlx::query_as::<_, QnStats>(
        "
        with items_in_pool as (
            select
                  item_id
                , created_at
            from item
            order by created_at desc
            limit 1500
        )
        , stats_at_sample_time as (
            select *
            from stats_history
            where sample_time = ?
        )
        , stats as (
            select
                  iip.item_id
                , iip.created_at as submission_time
                , coalesce(sast.upvotes, 0) as upvotes
                , coalesce(sast.expected_upvotes, 0.0) as expected_upvotes
            from items_in_pool iip
            left outer join stats_at_sample_time sast
            on iip.item_id = sast.item_id
        )
        select
              item_id
            , submission_time
            , upvotes
            , expected_upvotes
        from stats
        order by upvotes desc
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|stat| Observation {
        sample_time,
        data: QnStats {
            item_id: stat.item_id,
            submission_time: stat.submission_time,
            upvotes: stat.upvotes,
            expected_upvotes: stat.expected_upvotes,
        },
    })
    .collect();

    Ok(stats_observations)
}

async fn get_sitewide_new_upvotes(
    tx: &mut Transaction<'_, Sqlite>,
    previous_sample_time: i64,
) -> Result<i32, AppError> {
    let sitewide_upvotes: i32 = sqlx::query_scalar(
        "
        select count(*)
        from vote
        where vote = 1
        and created_at > ?
        ",
    )
    .bind(previous_sample_time)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sitewide_upvotes)
}

// TODO: replace with an actual model of expected upvotes by rank combination (and other factors)
async fn get_expected_upvote_share(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i32,
) -> Result<f32, AppError> {
    let ranking_entries: i32 = sqlx::query_scalar(
        "
        select count(*)
        from rank_history
        where item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    // previously unseen items
    if ranking_entries == 0 {
        return Ok(0.0);
    }

    let expected_upvotes: f32 = sqlx::query_scalar(
        "
        select us.upvote_share_at_rank
        from rank_history rh
        left outer join upvote_share us
        on rh.rank_top = us.rank_top
        where sample_time = (
            select max(sample_time)
            from rank_history
        )
        and item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(expected_upvotes)
}
