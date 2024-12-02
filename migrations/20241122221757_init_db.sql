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

create table if not exists stats_history (
    post_id                     integer not null references post(post_id)
  , sample_time                 integer not null
  , cumulative_upvotes          integer not null
  , cumulative_expected_upvotes real    not null
  -- , upvote_rate real not null -- TODO
) strict;

-- TODO: devise scheme to add other ranks if there are several pages
create table if not exists rank_history (
    post_id     integer not null
  , sample_time integer not null
  , rank_top    integer
) strict;

create table upvotes_at_rank_history (
    sample_time integer not null
  , rank_top    integer not null
  , upvotes     real not null
) strict;

create view if not exists upvotes_by_rank as
with upvotes_in_time_window as (
  select
      post_id
    , sample_time
    , cumulative_upvotes - lag(cumulative_upvotes) over (
        partition by post_id
        order by sample_time
    ) as upvotes_in_time_window
  from stats_history
)
, upvote_window as (
  select
      post_id
    , sample_time
    , coalesce(upvotes_in_time_window, 0) as upvotes_in_time_window
  from upvotes_in_time_window
)
, ranks_with_upvote_count as (
  select
      rh.post_id
    , rh.sample_time
    , rh.rank_top
    , uw.upvotes_in_time_window
  from rank_history rh
  join upvote_window uw
  on rh.post_id = uw.post_id
  and rh.sample_time = uw.sample_time
)
select
    rank_top
  , avg(upvotes_in_time_window) as avg_upvotes
from ranks_with_upvote_count
group by rank_top;
