create table if not exists item (
    item_id    integer not null primary key autoincrement
  , parent_id  integer          references item(item_id)
  , author_id  text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create table lineage (
    ancestor_id   integer not null references item(item_id)
  , descendant_id integer not null references item(item_id)
  , separation    integer not null
  , primary key(ancestor_id, descendant_id)
) strict;

create trigger after_insert_item
after insert on item when new.parent_id is not null
begin
    -- Insert a lineage record for parent
    insert into lineage (
          ancestor_id
        , descendant_id
        , separation
    )
    values(
          new.parent_id
        , new.item_id
        , 1
    )
    on conflict do nothing;

    -- Insert a lineage record for all ancestors of this parent
    insert into lineage
    select
          ancestor_id
        , new.item_id as descendant_id
        , 1 + separation as separation
    from lineage ancestor
    where ancestor.descendant_id = new.parent_id;

    -- It is possible that this item is inserted *after* some of its children
    -- In that case, the ancestry for those children (and their children) will
    -- be incomplete.

    -- Insert a lineage record for all children joined to all ancestors
    insert into lineage
    select
            ancestor.ancestor_id
          , child.item_id descendant_id
          , ancestor.separation + 1 as separation
    from item child
    join lineage ancestor
         on ancestor.descendant_id = new.item_id
    where child.parent_id = new.item_id;

    -- Insert a lineage record for all descendants joined to all ancestors
    insert into Lineage
    select
            ancestor.ancestor_id
          , descendant.descendant_id
          , ancestor.separation + descendant.separation + 1 as separation
    from item child
    join Lineage descendant
        on descendant.ancestor_id = child.item_id
    join Lineage ancestor
        on ancestor.descendant_id = new.item_id
    where child.parent_id = new.item_id;
end;

create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , item_id       integer not null references item(item_id)
  , user_id       text    not null
  , vote          integer not null
  , created_at    integer not null default (unixepoch('subsec') * 1000)
) strict;

create table vote (
    vote_event_id integer not null references vote_event(vote_event_id)
  , item_id       integer not null references item(item_id)
  , user_id       text    not null
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
