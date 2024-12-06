create table if not exists item (
    item_id    integer not null primary key autoincrement
  , parent_id  integer          references item(item_id)
  , author_id  integer not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , item_id       integer not null references item(item_id)
  , user_id       integer not null
  , vote          integer not null
  , created_at    integer not null default (unixepoch('subsec') * 1000)
) strict;

create table vote (
    vote_event_id integer not null references vote_event(vote_event_id)
  , item_id       integer not null references item(item_id)
  , user_id       integer not null
  , vote          integer not null
  , created_at    integer not null
  , primary key(user_id, item_id)
) strict;

create trigger after_insert_vote_event
after insert on vote_event
begin
  insert into vote (
      vote_event_id
    , item_id
    , user_id
    , vote
    , created_at
  )
  values(
      new.vote_event_id
    , new.item_id
    , new.user_id
    , new.vote
    , new.created_at
  ) on conflict(user_id, item_id) do update set
      vote          = new.vote
    , vote_event_id = new.vote_event_id
    , created_at    = new.created_at;
end;

create table if not exists stats_history (
    item_id          integer not null references item(item_id)
  , sample_time      integer not null
  , upvotes          integer not null
  , expected_upvotes real    not null
) strict;

create table if not exists rank_history (
    item_id     integer not null
  , sample_time integer not null
  , rank_top    integer
  , rank_new    integer
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
  , rh.rank_new
  , coalesce(uast.upvotes_at_sample_time, 0) as upvotes
  , suat.sitewide_upvotes
  , coalesce(cast(uast.upvotes_at_sample_time as real) / suat.sitewide_upvotes, 0) as upvotes_share
from rank_history rh
left outer join upvotes_at_sample_time uast
on rh.item_id = uast.item_id
and rh.sample_time = uast.sample_time
join sitewide_upvotes_at_sample_time suat
on rh.sample_time = suat.sample_time;

-- TODO: This model only takes rank_top into account. We need a model that incorporates all ranks
create view if not exists upvote_share as
select
    rank_top
  , avg(upvotes_share) as upvote_share_at_rank
from upvotes_at_rank_history
group by rank_top;
