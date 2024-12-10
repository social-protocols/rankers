# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

# Setup the development environment. Should only be run once
setup-dev-env:
  scripts/dev_setup.sh

# Start the server and a simulated that sends random posts and vote events to the api
dev:
  process-compose up -t=false

# Enter an interactive sqlite session
db:
  litecli $DATABASE_PATH

# Run migrations that are not yet applied to the database
db-migrate:
  sqlx migrate run

# Drop the database and recreate it, running all migrations
db-reset:
  sqlx database reset -y
