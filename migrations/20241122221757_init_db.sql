create table if not exists item (
    item_id    integer not null primary key autoincrement
  , parent_id  integer          references item(item_id)
  , author_id  text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , item_id       integer not null references item(item_id)
  , user_id       text    not null
  , vote          integer not null
  , rank          integer
  , page          text
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
  values (
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

create table if not exists qn_sample_interval (
    interval_id integer not null primary key autoincrement
  , start_time  integer not null
);

create table if not exists stats_history (
    item_id               integer not null references item(item_id)
  , interval_id           integer not null references qn_sample_interval(interval_id)
  , upvotes               integer not null
  , upvote_share          real    not null
  , expected_upvotes      real    not null
  , expected_upvote_share real    not null
) strict;

create table if not exists stats (
    item_id                     integer not null primary key references item(item_id)
  , updated_at                  integer not null
  , cumulative_upvotes          integer not null
  , cumulative_expected_upvotes real    not null
) strict;

create trigger after_insert_stats_history
after insert on stats_history
begin
  insert into stats (
      item_id
    , updated_at
    , cumulative_upvotes
    , cumulative_expected_upvotes
  )
  values (
      new.item_id
    , unixepoch('subsec') * 1000
    , new.upvotes
    , new.expected_upvotes
  )
  on conflict (item_id) do update set
      updated_at                  = unixepoch('subsec') * 1000
    , cumulative_upvotes          = stats.cumulative_upvotes + new.upvotes
    , cumulative_expected_upvotes = stats.cumulative_expected_upvotes + new.expected_upvotes;
end;

create table if not exists rank_history (
    item_id     integer not null
  , interval_id integer not null references qn_sample_interval(interval_id)
  , rank_top    integer
  , rank_new    integer
) strict;
