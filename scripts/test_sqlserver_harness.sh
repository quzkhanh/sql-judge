#!/usr/bin/env bash
set -euo pipefail

for command in docker openssl python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Missing required SQL Server harness command: %s\n' "$command" >&2
    exit 2
  fi
done

if ! docker info >/dev/null 2>&1; then
  printf 'SQL Server harness blocked: Docker daemon is unavailable.\n' >&2
  exit 2
fi

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sql-judge-sqlserver.XXXXXX")"
CONTAINER_NAME="sql-judge-mssql-$$"
IMAGE="${SQLSERVER_IMAGE:-mcr.microsoft.com/mssql/server:2022-latest}"
PORT="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
SA_PASSWORD="SqlJudge_Sa_$(python3 -c 'import secrets; print(secrets.token_hex(12))')!Aa9"
READER_PASSWORD="SqlJudge_Reader_$(python3 -c 'import secrets; print(secrets.token_hex(12))')!Aa9"
CA_CERT="$WORK_DIR/ca.crt"
CA_KEY="$WORK_DIR/ca.key"
SERVER_CERT="$WORK_DIR/server.crt"
SERVER_KEY="$WORK_DIR/server.key"
SERVER_CSR="$WORK_DIR/server.csr"
SERVER_CONFIG="$WORK_DIR/server.cnf"

cleanup() {
  docker rm --force "$CONTAINER_NAME" >/dev/null 2>&1 || true
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

cat >"$SERVER_CONFIG" <<'EOF'
[req]
distinguished_name = req_distinguished_name
req_extensions = req_ext
prompt = no

[req_distinguished_name]
CN = localhost

[req_ext]
subjectAltName = DNS:localhost, IP:127.0.0.1

[server_cert]
subjectAltName = DNS:localhost, IP:127.0.0.1
extendedKeyUsage = serverAuth
keyUsage = digitalSignature, keyEncipherment
EOF

openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
  -subj "/CN=sql-judge-test-ca" \
  -keyout "$CA_KEY" -out "$CA_CERT" >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes \
  -keyout "$SERVER_KEY" -out "$SERVER_CSR" \
  -config "$SERVER_CONFIG" >/dev/null 2>&1
openssl x509 -req -days 2 \
  -in "$SERVER_CSR" -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial \
  -out "$SERVER_CERT" -extfile "$SERVER_CONFIG" -extensions server_cert \
  >/dev/null 2>&1

docker run --detach --name "$CONTAINER_NAME" --hostname localhost \
  --env ACCEPT_EULA=Y --env MSSQL_PID=Developer \
  --env "MSSQL_SA_PASSWORD=$SA_PASSWORD" \
  --publish "127.0.0.1:$PORT:1433" "$IMAGE" >/dev/null

docker cp "$SERVER_CERT" "$CONTAINER_NAME:/tmp/sqljudge-server.crt" >/dev/null
docker cp "$SERVER_KEY" "$CONTAINER_NAME:/tmp/sqljudge-server.key" >/dev/null
docker exec --user 0 "$CONTAINER_NAME" bash -eu -c '
  install -d -m 700 /var/opt/mssql/tls
  install -o mssql -g mssql -m 600 /tmp/sqljudge-server.crt /var/opt/mssql/tls/server.crt
  install -o mssql -g mssql -m 600 /tmp/sqljudge-server.key /var/opt/mssql/tls/server.key
  /opt/mssql/bin/mssql-conf set network.tlscert /var/opt/mssql/tls/server.crt
  /opt/mssql/bin/mssql-conf set network.tlskey /var/opt/mssql/tls/server.key
  /opt/mssql/bin/mssql-conf set network.forceencryption 1
' >/dev/null

if docker exec "$CONTAINER_NAME" test -x /opt/mssql-tools18/bin/sqlcmd; then
  SQLCMD=/opt/mssql-tools18/bin/sqlcmd
else
  SQLCMD=/opt/mssql-tools/bin/sqlcmd
fi

for attempt in $(seq 1 60); do
  if docker exec "$CONTAINER_NAME" "$SQLCMD" -S localhost -U sa -P "$SA_PASSWORD" -C -b -Q "SELECT 1" \
    >/dev/null 2>&1; then
    break
  fi
  if [ "$attempt" -eq 60 ]; then
    printf 'SQL Server did not become ready.\n' >&2
    exit 1
  fi
  sleep 2
done

docker exec -i "$CONTAINER_NAME" \
  "$SQLCMD" -S localhost -U sa -P "$SA_PASSWORD" -C -b \
  -v "READER_PASSWORD=$READER_PASSWORD" <<'SQL'
IF DB_ID(N'sqljudge_fixture') IS NULL CREATE DATABASE sqljudge_fixture;
GO
IF DB_ID(N'sqljudge_other') IS NULL CREATE DATABASE sqljudge_other;
GO
IF SUSER_ID(N'sqljudge_reader') IS NULL
  CREATE LOGIN [sqljudge_reader] WITH PASSWORD = '$(READER_PASSWORD)', CHECK_POLICY = OFF, CHECK_EXPIRATION = OFF;
GO
USE sqljudge_fixture;
GO
IF USER_ID(N'sqljudge_reader') IS NULL
  CREATE USER [sqljudge_reader] FOR LOGIN [sqljudge_reader];
USE sqljudge_other;
GO
IF OBJECT_ID(N'dbo.remote_sample', N'U') IS NOT NULL DROP TABLE dbo.remote_sample;
CREATE TABLE dbo.remote_sample (id int NOT NULL);
INSERT INTO dbo.remote_sample (id) VALUES (7);
USE sqljudge_fixture;
GO
IF OBJECT_ID(N'dbo.sample', N'U') IS NOT NULL DROP TABLE dbo.sample;
CREATE TABLE dbo.sample (
  id int NOT NULL,
  label nvarchar(50) NULL,
  amount decimal(10, 2) NOT NULL,
  active bit NOT NULL,
  item_id uniqueidentifier NOT NULL,
  payload varbinary(16) NOT NULL
);
INSERT INTO dbo.sample (id, label, amount, active, item_id, payload)
VALUES
  (1, N'one', 10.50, 1, '11111111-1111-1111-1111-111111111111', 0x010203),
  (2, NULL, 20.25, 0, '22222222-2222-2222-2222-222222222222', 0x040506);
IF OBJECT_ID(N'dbo.indirect_other', N'V') IS NOT NULL DROP VIEW dbo.indirect_other;
EXEC(N'CREATE VIEW dbo.indirect_other AS SELECT id FROM sqljudge_other.dbo.remote_sample');
GRANT CONNECT TO [sqljudge_reader];
GRANT SELECT ON OBJECT::dbo.sample TO [sqljudge_reader];
GRANT SELECT ON OBJECT::dbo.indirect_other TO [sqljudge_reader];
DENY INSERT, UPDATE, DELETE, ALTER, TAKE OWNERSHIP ON OBJECT::dbo.sample TO [sqljudge_reader];
GO
SQL

if ! SELECT_OUTPUT=$(docker exec "$CONTAINER_NAME" "$SQLCMD" -S localhost -U sqljudge_reader -P "$READER_PASSWORD" -C -b \
  -d sqljudge_fixture -Q "SELECT count(*) FROM dbo.sample" 2>&1); then
  printf 'SELECT-only login could not read the fixture table.\n' >&2
  printf '%s\n' "$SELECT_OUTPUT" >&2
  exit 1
fi

if docker exec "$CONTAINER_NAME" "$SQLCMD" -S localhost -U sqljudge_reader -P "$READER_PASSWORD" -C -b \
  -d sqljudge_fixture -Q "INSERT INTO dbo.sample (id, label, amount, active, item_id, payload) VALUES (99, N'blocked', 1, 1, '99999999-9999-9999-9999-999999999999', 0x00)" \
  >/dev/null 2>&1; then
  printf 'SELECT-only login unexpectedly executed INSERT.\n' >&2
  exit 1
fi

if docker exec "$CONTAINER_NAME" "$SQLCMD" -S localhost -U sqljudge_reader -P "$READER_PASSWORD" -C -b \
  -d sqljudge_fixture -Q "DROP TABLE dbo.sample" >/dev/null 2>&1; then
  printf 'SELECT-only login unexpectedly executed DROP.\n' >&2
  exit 1
fi

SQLSERVER_TEST_HOST=localhost \
SQLSERVER_TEST_PORT="$PORT" \
SQLSERVER_TEST_DATABASE=sqljudge_fixture \
SQLSERVER_TEST_USERNAME=sqljudge_reader \
SQLSERVER_TEST_PASSWORD="$READER_PASSWORD" \
SSL_CERT_FILE="$CA_CERT" \
cargo test --manifest-path "$ROOT_DIR/Cargo.toml" \
  -p judge-sqlserver --features sqlserver-integration --test sqlserver_integration -- --ignored --nocapture

printf 'SQL Server harness passed: TLS, metadata, bounded SELECT, and server-side write denial verified.\n'
