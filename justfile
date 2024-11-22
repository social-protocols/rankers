# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

run:
  cargo run

migrate:
  sqlx migrate run

db:
  sqlite3 $DATABASE_PATH
