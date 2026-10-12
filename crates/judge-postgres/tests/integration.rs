use judge_core::{Column, LogicalType, OutputPolicy, QueryLimits, RowOrder, SqlValue, Verdict};
use judge_postgres::{
    JudgeCase, ManagedEnvironment, PostgresSession, QueryExecutionError, judge_submission,
};
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct TestServer {
    data_dir: PathBuf,
    socket_dir: PathBuf,
    port: u16,
    password: String,
}

impl TestServer {
    fn start() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("sql-judge-pg-rust-{}-{nonce}", std::process::id()));
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        std::fs::create_dir_all(&socket_dir).expect("create PostgreSQL socket directory");
        let port = TcpListener::bind(("127.0.0.1", 0))
            .expect("reserve PostgreSQL port")
            .local_addr()
            .expect("read reserved port")
            .port();
        let admin = std::env::var("USER").expect("USER must identify the local test owner");

        assert!(
            Command::new("initdb")
                .args([
                    "-D",
                    data_dir.to_str().expect("data path"),
                    "--username",
                    &admin,
                    "--auth-local=peer",
                    "--auth-host=scram-sha-256",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("run initdb")
                .success()
        );
        assert!(
            Command::new("pg_ctl")
                .args([
                    "-D",
                    data_dir.to_str().expect("data path"),
                    "-o",
                    &format!(
                        "-h 127.0.0.1 -p {port} -k {} -c listen_addresses=127.0.0.1",
                        socket_dir.to_str().expect("socket path")
                    ),
                    "-w",
                    "start",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("start PostgreSQL")
                .success()
        );
        assert!(
            Command::new("createdb")
                .args([
                    "-h",
                    socket_dir.to_str().expect("socket path"),
                    "-p",
                    &port.to_string(),
                    "sqljudge_test",
                ])
                .status()
                .expect("create test database")
                .success()
        );

        let password = format!("sqljudge_test_{}", std::process::id());
        let setup = format!(
            "CREATE ROLE sqljudge_runner LOGIN PASSWORD '{password}' NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;\
             REVOKE CREATE ON SCHEMA public FROM PUBLIC;\
             GRANT USAGE ON SCHEMA public TO sqljudge_runner;\
             CREATE TABLE public.sample (id integer NOT NULL, label text);\
             INSERT INTO public.sample VALUES (1, 'one'), (2, NULL);\
             GRANT SELECT ON public.sample TO sqljudge_runner;"
        );
        let mut setup_process = Command::new("psql")
            .args([
                "-h",
                socket_dir.to_str().expect("socket path"),
                "-p",
                &port.to_string(),
                "-d",
                "sqljudge_test",
                "-v",
                "ON_ERROR_STOP=1",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start psql setup");
        setup_process
            .stdin
            .take()
            .expect("open psql stdin")
            .write_all(setup.as_bytes())
            .expect("write setup SQL");
        assert!(setup_process.wait().expect("wait for setup SQL").success());

        Self {
            data_dir,
            socket_dir,
            port,
            password,
        }
    }

    fn connection_string(&self) -> String {
        format!(
            "host=127.0.0.1 port={} user=sqljudge_runner password={} dbname=sqljudge_test",
            self.port, self.password
        )
    }

    fn admin_connection_string(&self) -> String {
        format!(
            "host={} port={} user={} dbname=sqljudge_test",
            self.socket_dir.to_str().expect("socket path"),
            self.port,
            std::env::var("USER").expect("USER must identify the local test owner")
        )
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = Command::new("pg_ctl")
            .args([
                "-D",
                self.data_dir.to_str().expect("data path"),
                "-w",
                "stop",
                "-m",
                "fast",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = std::fs::remove_dir_all(self.data_dir.parent().expect("test root"));
        let _ = &self.socket_dir;
    }
}

#[test]
fn session_decodes_typed_rows_and_nulls() {
    let server = TestServer::start();
    let mut session =
        PostgresSession::connect(&server.connection_string()).expect("connect runner");

    let result = session
        .execute(
            "SELECT id, label FROM public.sample ORDER BY id",
            QueryLimits::new(2_000, 10, 64 * 1024),
        )
        .expect("execute typed query");

    assert_eq!(result.columns[0].logical_type, LogicalType::Integer);
    assert_eq!(result.columns[1].logical_type, LogicalType::Text);
    assert_eq!(result.rows[0][0], SqlValue::Signed(1));
    assert_eq!(result.rows[1][1], SqlValue::Null);
}

#[test]
fn session_distinguishes_syntax_timeout_and_result_limits() {
    let server = TestServer::start();
    let mut session =
        PostgresSession::connect(&server.connection_string()).expect("connect runner");

    assert_eq!(
        session.execute("SELEC 1", QueryLimits::new(2_000, 10, 64 * 1024)),
        Err(QueryExecutionError::SyntaxError)
    );
    assert_eq!(
        session.execute(
            "SELECT generate_series(1, 4)",
            QueryLimits::new(2_000, 2, 64 * 1024),
        ),
        Err(QueryExecutionError::ResultLimitExceeded)
    );
    assert_eq!(
        session.execute(
            "SELECT count(*) FROM generate_series(1, 1000000000)",
            QueryLimits::new(100, 10, 64 * 1024),
        ),
        Err(QueryExecutionError::TimeLimitExceeded)
    );
}

#[test]
fn orchestrator_accepts_equivalent_window_query_and_rejects_hardcoded_rows() {
    let server = TestServer::start();
    let mut admin =
        PostgresSession::connect(&server.admin_connection_string()).expect("connect admin");
    let output = OutputPolicy::new(
        RowOrder::Unordered,
        true,
        true,
        vec![
            Column::new("employee_id", LogicalType::Integer),
            Column::new("employee_name", LogicalType::Text),
        ],
    );
    let case = JudgeCase {
        id: "basic",
        schema_sql: "CREATE TABLE employees (id integer, name text, department_id integer, salary integer);",
        fixture_sql: "INSERT INTO employees VALUES (1, 'Alice', 1, 150), (2, 'Bob', 1, 100), (3, 'Carol', 1, 50);",
        reference_sql: "SELECT e.id AS employee_id, e.name AS employee_name FROM employees e WHERE e.salary > (SELECT AVG(peer.salary) FROM employees peer WHERE peer.department_id = e.department_id);",
        output,
        limits: QueryLimits::new(2_000, 100, 64 * 1024),
    };

    let accepted = judge_submission(
        &mut admin,
        "sqljudge_runner",
        std::slice::from_ref(&case),
        "SELECT id AS employee_id, name AS employee_name FROM (SELECT e.*, AVG(e.salary) OVER (PARTITION BY department_id) AS average_salary FROM employees e) e WHERE salary > average_salary",
    );
    assert_eq!(accepted.verdict, Verdict::Accepted);

    let rejected = judge_submission(
        &mut admin,
        "sqljudge_runner",
        &[case],
        "SELECT 999 AS employee_id, 'Not Alice' AS employee_name",
    );
    assert_eq!(rejected.verdict, Verdict::WrongAnswer);
}

#[test]
fn orchestrator_maps_reference_failure_to_judge_error() {
    let server = TestServer::start();
    let mut admin =
        PostgresSession::connect(&server.admin_connection_string()).expect("connect admin");
    let report = judge_submission(
        &mut admin,
        "sqljudge_runner",
        &[JudgeCase {
            id: "broken",
            schema_sql: "CREATE TABLE values_table (id integer);",
            fixture_sql: "INSERT INTO values_table VALUES (1);",
            reference_sql: "SELEC 1",
            output: OutputPolicy::new(
                RowOrder::Ordered,
                true,
                true,
                vec![Column::new("id", LogicalType::Integer)],
            ),
            limits: QueryLimits::new(2_000, 100, 64 * 1024),
        }],
        "SELECT id FROM values_table",
    );

    assert_eq!(report.verdict, Verdict::JudgeError);
}

#[test]
fn managed_namespace_provisioning_isolated_and_marker_verified() {
    let server = TestServer::start();
    let mut admin =
        PostgresSession::connect(&server.admin_connection_string()).expect("connect admin");
    let first = ManagedEnvironment::new("env-a", "instance-a", "sj_env_a", "nonce-a")
        .expect("valid managed environment");
    let second = ManagedEnvironment::new("env-b", "instance-b", "sj_env_b", "nonce-b")
        .expect("valid managed environment");

    admin
        .provision_managed_namespace(&first, "sqljudge_runner")
        .expect("provision first namespace");
    admin
        .provision_managed_namespace(&second, "sqljudge_runner")
        .expect("provision second namespace");

    let mut runner = PostgresSession::connect(&server.connection_string()).expect("connect runner");
    let marker = runner
        .execute(
            "SELECT environment_id, instance_id, ownership_nonce FROM \"sj_env_a\".\"__sql_judge_ownership\"",
            QueryLimits::new(2_000, 10, 64 * 1024),
        );
    assert_eq!(marker, Err(QueryExecutionError::RuntimeError));
    let second_marker = admin
        .execute(
            "SELECT environment_id FROM \"sj_env_b\".\"__sql_judge_ownership\"",
            QueryLimits::new(2_000, 10, 64 * 1024),
        )
        .expect("read second marker");
    assert_eq!(second_marker.rows[0][0], SqlValue::Text("env-b".to_owned()));

    let forged =
        ManagedEnvironment::new("env-forged", "instance-forged", "sj_env_a", "nonce-forged")
            .expect("valid forged shape");
    assert_eq!(
        admin.provision_managed_namespace(&forged, "sqljudge_runner"),
        Err(QueryExecutionError::OwnershipVerificationFailed)
    );
}

#[test]
fn managed_namespace_schema_setup_isolated_between_environments() {
    let server = TestServer::start();
    let mut admin =
        PostgresSession::connect(&server.admin_connection_string()).expect("connect admin");
    let first = ManagedEnvironment::new("env-a", "instance-a", "sj_env_a", "nonce-a")
        .expect("valid first environment");
    let second = ManagedEnvironment::new("env-b", "instance-b", "sj_env_b", "nonce-b")
        .expect("valid second environment");
    let schema_sql = "CREATE TABLE employees (name text); INSERT INTO employees VALUES ('first');";

    admin
        .provision_managed_namespace_with_schema(&first, "sqljudge_runner", schema_sql)
        .expect("provision first schema");
    admin
        .provision_managed_namespace_with_schema(&second, "sqljudge_runner", schema_sql)
        .expect("provision second schema");

    let first_rows = admin
        .execute(
            "SELECT name FROM \"sj_env_a\".employees",
            QueryLimits::new(2_000, 10, 64 * 1024),
        )
        .expect("read first schema");
    let second_rows = admin
        .execute(
            "SELECT name FROM \"sj_env_b\".employees",
            QueryLimits::new(2_000, 10, 64 * 1024),
        )
        .expect("read second schema");

    assert_eq!(first_rows.rows, second_rows.rows);
    assert_eq!(first_rows.rows[0][0], SqlValue::Text("first".to_owned()));
}

#[test]
fn fixture_setup_cannot_write_to_public_schema_with_admin_session() {
    let server = TestServer::start();
    let mut admin =
        PostgresSession::connect(&server.admin_connection_string()).expect("connect admin");
    let report = judge_submission(
        &mut admin,
        "sqljudge_runner",
        &[JudgeCase {
            id: "escape",
            schema_sql: "CREATE TABLE public.should_not_exist (id integer);",
            fixture_sql: "INSERT INTO public.should_not_exist VALUES (1);",
            reference_sql: "SELECT id FROM public.should_not_exist",
            output: OutputPolicy::new(
                RowOrder::Ordered,
                true,
                true,
                vec![Column::new("id", LogicalType::Integer)],
            ),
            limits: QueryLimits::new(2_000, 10, 64 * 1024),
        }],
        "SELECT id FROM public.should_not_exist",
    );
    assert_eq!(report.verdict, Verdict::JudgeError);
    let table = admin
        .execute(
            "SELECT count(*)::int4 FROM pg_catalog.pg_class WHERE relname = 'should_not_exist'",
            QueryLimits::new(2_000, 1, 64 * 1024),
        )
        .expect("inspect escaped table");
    assert_eq!(table.rows[0][0], SqlValue::Signed(0));
}
