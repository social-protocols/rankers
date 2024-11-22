#!/usr/bin/env bash
# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail
set -Eeuo pipefail

json_data='{
  "vote_event_id": 1,
  "vote": 1
}'

curl -X POST http://localhost:3000/send_vote_event \
     -H "Content-Type: application/json" \
     -d "$json_data"
