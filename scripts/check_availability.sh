#!/usr/bin/env bash
# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail
set -Eeuo pipefail

ENDPOINT="http://localhost:3000/health_check"
TIMEOUT=5

while ! curl -s --max-time $TIMEOUT $ENDPOINT > /dev/null; do
    echo "Waiting for the HTTP server to be available..."
    sleep 2
done

echo "HTTP server is available."

exit 1
