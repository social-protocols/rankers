create table if not exists User (
  id text not null primary key
  , email text not null
  , username text not null
  , createdAt integer not null default (unixepoch('subsec')*1000)
  , isAdmin integer not null default false
) strict;

create table if not exists VoteEvent (
    voteEventId integer not null primary key autoincrement
  , userId text not null
  , postId integer not null
  , vote integer not null
  , voteEventTime integer not null default (unixepoch('subsec')*1000)
  , parentId integer
  , foreign key(userId) references User(id)
) strict;
