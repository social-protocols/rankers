# QN Sampling

```
                  sample stats
                  for interval 1 /
                  start inteval 2 /
                  sample ranks
                  for interval 2
                  ↓

     | interval 1 | interval 2 | interval 3 | ...
time ---------------------------------------------->
     ↑
     sample ranks
     for interval 1
```

Interval data:

- `interval_id`
- `start_time` / (`end_time` is `start_time` of next interval)
- `item_id`
- `rank_top`
- `rank_new`
- (TODO: add custom ranks from client side)
- `upvotes`
- `upvote_share`
- `expected_upvotes`
- `expected_upvote_share`

For convenience and efficiency, a trigger maintains the current stats of each item:

```
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
```
