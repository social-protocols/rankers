create table if not exists post (
    post_id    integer not null primary key autoincrement
  , parent_id  integer          references post(post_id)
  , content    text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists vote_event (
    vote_event_id   integer not null primary key autoincrement
  , post_id         integer not null references post(post_id)
  , vote            integer not null
  , vote_event_time integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists rank_samples (
    post_id         integer not null references post(post_id)
  , sample_time     integer not null
  , submission_time integer not null -- to calculate age
  -- , rank_top integer not null
  -- -- TODO: devise scheme to add other ranks if there are several pages
  , upvote_count    integer not null
  -- , cumulative_expected_upvotes real not null
  -- , upvote_rate real not null
)
