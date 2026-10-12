#!/usr/bin/env bash
set -euo pipefail

for command in initdb pg_ctl psql createdb python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Missing required PostgreSQL harness command: %s\n' "$command" >&2
    exit 2
  fi
done

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sql-judge-pg.XXXXXX")"
DATA_DIR="$WORK_DIR/data"
SOCKET_DIR="$WORK_DIR/socket"
PASSFILE="$WORK_DIR/pgpass"
DATABASE="sqljudge_harness"
RUNNER_ROLE="sqljudge_runner"
RUNNER_PASSWORD="$(python3 -c 'import secrets; print(secrets.token_hex(24))')"
PORT="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"

cleanup() {
  if [ -f "$DATA_DIR/postmaster.pid" ]; then
    pg_ctl -D "$DATA_DIR" -w stop -m fast >/dev/null
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

mkdir -m 700 "$SOCKET_DIR"
initdb -D "$DATA_DIR" --username="$(id -un)" --auth-local=peer --auth-host=scram-sha-256 >/dev/null
pg_ctl -D "$DATA_DIR" -o "-h 127.0.0.1 -p $PORT -k $SOCKET_DIR -c listen_addresses=127.0.0.1" -w start >/dev/null

createdb -h "$SOCKET_DIR" -p "$PORT" "$DATABASE"

psql -h "$SOCKET_DIR" -p "$PORT" -d "$DATABASE" -v ON_ERROR_STOP=1 <<SQL >/dev/null
CREATE ROLE $RUNNER_ROLE LOGIN PASSWORD '$RUNNER_PASSWORD' NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO $RUNNER_ROLE;
CREATE TABLE public.runner_guard (id integer NOT NULL);
GRANT SELECT ON public.runner_guard TO $RUNNER_ROLE;
SQL

printf '*:*:*:%s:%s\n' "$RUNNER_ROLE" "$RUNNER_PASSWORD" >"$PASSFILE"
chmod 600 "$PASSFILE"

listen_addresses="$(psql -h "$SOCKET_DIR" -p "$PORT" -d "$DATABASE" -Atqc 'SHOW listen_addresses')"
if [ "$listen_addresses" != "127.0.0.1" ]; then
  printf 'Expected loopback-only listen_addresses, got: %s\n' "$listen_addresses" >&2
  exit 1
fi

runner_psql=(env PGPASSFILE="$PASSFILE" psql -h 127.0.0.1 -p "$PORT" -U "$RUNNER_ROLE" -d "$DATABASE" -v ON_ERROR_STOP=1)
if [ "$("${runner_psql[@]}" -Atqc 'SELECT count(*) FROM public.runner_guard')" != "0" ]; then
  printf 'Runner could not read its guard table\n' >&2
  exit 1
fi

if "${runner_psql[@]}" -c 'CREATE TABLE public.runner_must_fail (id integer)' >/dev/null 2>&1; then
  printf 'Runner unexpectedly created a table\n' >&2
  exit 1
fi

if "${runner_psql[@]}" -c 'DROP TABLE public.runner_guard' >/dev/null 2>&1; then
  printf 'Runner unexpectedly dropped a table\n' >&2
  exit 1
fi

printf 'PostgreSQL harness passed: loopback-only server and low-privilege runner enforced.\n'
