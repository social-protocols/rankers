#!/usr/bin/env bash
# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail
set -Eeuo pipefail

API_BASE_URL="http://localhost:3000"
API_POST_ENDPOINT="$API_BASE_URL/create_post"
API_VOTE_ENDPOINT="$API_BASE_URL/send_vote_event"

TOTAL_POSTS=20
TOTAL_VOTE_EVENTS=500

generate_post() {
    local post_id=$1
    local content_options=(
        "Hello, world!"
        "Just another day in paradise"
        "Coding is fun!"
        "Learning something new today"
        "Check out this cool thing I found"
        "Random thoughts..."
        "Weekend plans?"
        "Music recommendation time"
        "Feeling grateful"
        "Deep philosophical musing"
    )

    # Randomly decide if this is a top-level or reply post
    local parent_id="null"
    if [ $((RANDOM % 5)) -eq 0 ] && [ $post_id -gt 10 ]; then
        # 20% chance of being a reply to an earlier post
        parent_id=$((RANDOM % (post_id - 1) + 1))
    fi

    local timestamp=$(date +%s%3N)
    local content=${content_options[$((RANDOM % ${#content_options[@]}))]}
    local post_json=$(cat <<EOF
{
  "post_id": $post_id,
  "parent_id": $parent_id,
  "content": "$content",
  "created_at": $timestamp
}
EOF
)

    curl -X POST "$API_POST_ENDPOINT" \
         -H "Content-Type: application/json" \
         -d "$post_json"

    sleep 0.1
}

generate_vote_event() {
    local vote_event_id=$1

    # Randomly select a post to vote on
    local post_id=$((RANDOM % TOTAL_POSTS + 1))

    # Randomize vote (1 for upvote, -1 for downvote)
    local vote
    if [ $((RANDOM % 2)) -eq 0 ]; then
        vote=1
    else
        vote=-1
    fi

    local timestamp=$(date +%s%3N)

    local vote_event_json=$(cat <<EOF
{
  "vote_event_id": $vote_event_id,
  "post_id": $post_id,
  "vote": $vote,
  "vote_event_time": $timestamp
}
EOF
)

    curl -X POST "$API_VOTE_ENDPOINT" \
         -H "Content-Type: application/json" \
         -d "$vote_event_json"

    # Add a small delay to prevent overwhelming the server
    sleep 0.05
}


# Seed 20 posts
echo "Starting to seed 20 posts..."
for ((i=1; i<=TOTAL_POSTS; i++)); do
    generate_post $i
    echo "Posted post $i"
done
echo "Finished seeding posts!"

echo "Starting to seed $TOTAL_VOTE_EVENTS vote events..."
for ((i=1; i<=TOTAL_VOTE_EVENTS; i++)); do
    generate_vote_event $i
    echo "Posted vote event $i"
done
echo "Finished seeding vote events!"

