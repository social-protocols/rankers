#!/usr/bin/env bash
# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail
set -Eeuo pipefail

post='{
  "post_id": 1,
  "parent_id": null,
  "content": "Halo Weld, i bims lol",
  "created_at": 1732529304459
}'

vote_event_1='{
  "vote_event_id": 1,
  "post_id": 1,
  "vote": 1,
  "vote_event_time": 1732529304459
}'
vote_event_2='{
  "vote_event_id": 2,
  "post_id": 1,
  "vote": 1,
  "vote_event_time": 1732529304459
}'
vote_event_3='{
  "vote_event_id": 3,
  "post_id": 1,
  "vote": 1,
  "vote_event_time": 1732529304459
}'

curl -X POST http://localhost:3000/create_post \
     -H "Content-Type: application/json" \
     -d "$post"

curl -X POST http://localhost:3000/send_vote_event \
     -H "Content-Type: application/json" \
     -d "$vote_event_1"
curl -X POST http://localhost:3000/send_vote_event \
     -H "Content-Type: application/json" \
     -d "$vote_event_2"
curl -X POST http://localhost:3000/send_vote_event \
     -H "Content-Type: application/json" \
     -d "$vote_event_3"
