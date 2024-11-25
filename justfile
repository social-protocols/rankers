# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

run:
  cargo run

migrate:
  sqlx migrate run

db:
  sqlite3 $DATABASE_PATH

reset-db:
  rm -f $DATABASE_PATH
  sqlite3 $DATABASE_PATH ".exit"
  sqlx migrate run

seed:
  ./scripts/seed.sh

# TODO: configure in editor config (on save)
format:
  cargo fmt
