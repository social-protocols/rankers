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

create view if not exists upvotes_at_rank_history as
with upvotes_at_sample_time as (
  select
      post_id
    , sample_time
    , coalesce(
      cumulative_upvotes - lag(cumulative_upvotes) over (
        partition by post_id
        order by sample_time
      ),
      0
    ) as upvotes_at_sample_time
  from stats_history
)
, sitewide_upvotes_at_tick as (
  select
      sample_time
    , sum(upvotes_at_sample_time) as sitewide_upvotes
  from upvotes_at_sample_time
  group by sample_time
)
, with_sitewide as (
  select
      rh.post_id
    , rh.sample_time
    , rh.rank_top
    , coalesce(uast.upvotes_at_sample_time, 0) as upvotes
    , suat.sitewide_upvotes
  from rank_history rh
  left outer join upvotes_at_sample_time uast
  on rh.post_id = uast.post_id
  and rh.sample_time = uast.sample_time
  join sitewide_upvotes_at_tick suat
  on rh.sample_time = suat.sample_time
)
select
      post_id
    , sample_time
    , rank_top
    , upvotes
    , sitewide_upvotes
    , coalesce(cast(upvotes as real) / sitewide_upvotes, 0) as upvotes_share
from with_sitewide;

create view if not exists upvote_share as
select
    rank_top
  , avg(upvotes_share) as upvote_share_at_rank
from upvotes_at_rank_history
group by rank_top;
