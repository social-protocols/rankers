# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

dev:
  process-compose up -t=false

migrate:
  sqlx migrate run

db:
  litecli $DATABASE_PATH

# TODO: configure in editor config (on save)
format:
  cargo fmt
