#!/usr/bin/env bash
# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail
set -Eeuo pipefail

BASE_URL="http://localhost:3000"

# Create 10 items
for i in {1..10}; do
    ITEM_PAYLOAD=$(cat <<EOF
{
    "item_id": $i,
    "parent_id": null,
    "author_id": "author_$i",
    "created_at": $(date +%s%N | cut -b1-13)
}
EOF
)
    echo "Creating item $i"
    curl -X POST "$BASE_URL/items" \
         -H "Content-Type: application/json" \
         -d "$ITEM_PAYLOAD"
done

# Create 100 vote events
for i in {1..100}; do
    VOTE_EVENT_PAYLOAD=$(cat <<EOF
{
    "vote_event_id": $i,
    "item_id": $(( (i % 10) + 1 )),
    "user_id": "user_$(( (i % 10) + 1 ))",
    "vote": 1,
    "rank": null,
    "page": null,
    "created_at": $(date +%s%N | cut -b1-13)
}
EOF
)
    echo "Creating vote event $i"
    curl -X POST "$BASE_URL/vote_events" \
         -H "Content-Type: application/json" \
         -d "$VOTE_EVENT_PAYLOAD"
done

echo "Seeding complete!"
