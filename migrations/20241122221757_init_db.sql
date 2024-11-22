create table if not exists vote_event (
    vote_event_id integer not null primary key autoincrement
  , post_id integer not null references post(post_id)
  , vote integer not null
) strict;

create table if not exists post (
    post_id integer not null primary key autoincrement
  , content text not null
) strict;
