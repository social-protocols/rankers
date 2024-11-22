create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , vote integer not null
) strict;
