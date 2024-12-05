create table if not exists item (
    item_id    integer not null primary key autoincrement
  , parent_id  integer          references item(item_id)
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , item_id       integer not null references item(item_id)
  , vote          integer not null
  , created_at    integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists stats_history (
    item_id                     integer not null references item(item_id)
  , sample_time                 integer not null
  , upvotes          integer not null
  , expected_upvotes real    not null
) strict;

create table if not exists rank_history (
    item_id     integer not null
  , sample_time integer not null
  , rank_top    integer
) strict;

create view if not exists upvotes_at_rank_history as
with upvotes_at_sample_time as (
  select
      item_id
    , sample_time
    , coalesce(
      upvotes - lag(upvotes) over (
        partition by item_id
        order by sample_time
      ),
      0
    ) as upvotes_at_sample_time
  from stats_history
)
, sitewide_upvotes_at_sample_time as (
  select
      sample_time
    , sum(upvotes_at_sample_time) as sitewide_upvotes
  from upvotes_at_sample_time
  group by sample_time
)
select
    rh.item_id
  , rh.sample_time
  , rh.rank_top
  , coalesce(uast.upvotes_at_sample_time, 0) as upvotes
  , suat.sitewide_upvotes
  , coalesce(cast(uast.upvotes_at_sample_time as real) / suat.sitewide_upvotes, 0) as upvotes_share
from rank_history rh
left outer join upvotes_at_sample_time uast
on rh.item_id = uast.item_id
and rh.sample_time = uast.sample_time
join sitewide_upvotes_at_sample_time suat
on rh.sample_time = suat.sample_time;

create view if not exists upvote_share as
select
    rank_top
  , avg(upvotes_share) as upvote_share_at_rank
from upvotes_at_rank_history
group by rank_top;
