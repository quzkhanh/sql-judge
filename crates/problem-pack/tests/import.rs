use problem_pack::PackData;
use problem_pack::PackImporter;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn managed_pack_session_rejects_database_without_runtime_marker() {
    let server = LocalPostgres::start();
    let error = judge_postgres::ManagedPostgresSession::connect(
        &server.connection_string(),
        "test-instance",
    )
    .err()
    .expect("unmarked database must not become a managed pack target");
    assert_eq!(
        error,
        judge_postgres::QueryExecutionError::OwnershipVerificationFailed
    );
}

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/packs/hr-sql-101")
}

fn copy_sample() -> (tempfile_like::TempDir, PathBuf) {
    let temp = tempfile_like::TempDir::new();
    let destination = temp.path().join("pack");
    copy_tree(&sample_path(), &destination);
    (temp, destination)
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source") {
        let entry = entry.expect("read entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

#[test]
fn sample_pack_passes_strict_structural_validation() {
    let (_temp, pack) = copy_sample();
    let data = PackData::from_directory(pack).expect("load sample pack");
    data.validate().expect("validate sample pack");
}

#[test]
fn unknown_manifest_fields_are_rejected() {
    let (_temp, pack) = copy_sample();
    let manifest = pack.join("pack.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).expect("read manifest"))
            .expect("parse manifest");
    value["unexpected"] = serde_json::Value::Bool(true);
    fs::write(
        &manifest,
        serde_json::to_vec(&value).expect("encode manifest"),
    )
    .expect("write manifest");

    let data = PackData::from_directory(pack).expect("load pack");
    assert!(data.validate().is_err());
}

#[test]
fn mismatched_dialect_is_rejected() {
    let (_temp, pack) = copy_sample();
    let problem = pack.join("problems/above-average/problem.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&problem).expect("read problem"))
            .expect("parse problem");
    value["dialect"] = serde_json::Value::String("sqlserver".to_owned());
    fs::write(
        &problem,
        serde_json::to_vec(&value).expect("encode problem"),
    )
    .expect("write problem");

    let data = PackData::from_directory(pack).expect("load pack");
    assert!(data.validate().is_err());
}

#[test]
fn traversal_and_symlink_sources_are_rejected() {
    let (_temp, pack) = copy_sample();
    std::os::unix::fs::symlink(pack.join("pack.json"), pack.join("escape.json"))
        .expect("create symlink");
    assert!(PackData::from_directory(pack).is_err());
}

#[test]
fn case_colliding_files_are_rejected() {
    let (_temp, pack) = copy_sample();
    fs::write(pack.join("README.MD"), "collision").expect("write colliding file");
    let data = PackData::from_directory(pack).expect("load pack");
    assert!(data.validate().is_err());
}

#[test]
fn duplicate_problem_ids_are_rejected() {
    let (_temp, pack) = copy_sample();
    let duplicate = pack.join("problems/duplicate/problem.json");
    fs::create_dir_all(duplicate.parent().expect("duplicate parent")).expect("create problem");
    fs::copy(pack.join("problems/above-average/problem.json"), &duplicate).expect("copy problem");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(pack.join("pack.json")).expect("read manifest"))
            .expect("parse manifest");
    manifest["problems"] = serde_json::json!([
        "problems/above-average/problem.json",
        "problems/duplicate/problem.json"
    ]);
    fs::write(
        pack.join("pack.json"),
        serde_json::to_vec(&manifest).expect("encode manifest"),
    )
    .expect("write manifest");

    let data = PackData::from_directory(pack).expect("load pack");
    assert!(data.validate().is_err());
}

#[test]
fn failed_runtime_validation_leaves_no_published_or_staged_artifact() {
    let (_temp, pack) = copy_sample();
    fs::write(
        pack.join("problems/above-average/reference.sql"),
        "SELEC broken",
    )
    .expect("break reference");
    let data = PackData::from_directory(&pack)
        .expect("load pack")
        .validate()
        .expect("structural validation");
    let registry = pack.join("registry");
    let server = LocalPostgres::start();
    let mut session = server.admin_session();

    assert!(
        PackImporter::new(&registry)
            .import(&data, &mut session, "sqljudge_runner")
            .is_err()
    );
    assert!(!registry.exists());
}

#[test]
fn successful_runtime_validation_publishes_immutable_digest_and_is_idempotent() {
    let (_temp, pack) = copy_sample();
    let data = PackData::from_directory(&pack)
        .expect("load pack")
        .validate()
        .expect("structural validation");
    let registry = pack.join("registry");
    let server = LocalPostgres::start();
    let mut session = server.admin_session();
    let importer = PackImporter::new(&registry);

    let first = importer
        .import(&data, &mut session, "sqljudge_runner")
        .expect("runtime validation and publish");
    assert!(!first.already_installed);
    assert!(first.path.is_dir());
    assert!(first.path.join("pack.json").is_file());

    let second = importer
        .import(&data, &mut session, "sqljudge_runner")
        .expect("same content is an idempotent import");
    assert!(second.already_installed);
    assert_eq!(first.digest, second.digest);
    assert_eq!(first.path, second.path);

    fs::write(first.path.join("README.md"), "tampered installed content")
        .expect("tamper installed content");
    assert!(
        importer
            .import(&data, &mut session, "sqljudge_runner")
            .is_err()
    );
}

#[test]
fn different_content_cannot_replace_an_installed_pack_version() {
    let (_temp, pack) = copy_sample();
    let data = PackData::from_directory(&pack)
        .expect("load pack")
        .validate()
        .expect("structural validation");
    let registry = pack.parent().expect("pack parent").join("registry");
    let server = LocalPostgres::start();
    let mut session = server.admin_session();
    let importer = PackImporter::new(&registry);
    importer
        .import(&data, &mut session, "sqljudge_runner")
        .expect("first import");

    fs::write(pack.join("README.md"), "different immutable content")
        .expect("change optional content");
    let changed = PackData::from_directory(&pack)
        .expect("load changed pack")
        .validate()
        .expect("changed pack remains structurally valid");
    assert!(
        importer
            .import(&changed, &mut session, "sqljudge_runner")
            .is_err()
    );
    assert_eq!(
        fs::read_dir(registry.join("sha256"))
            .expect("read registry")
            .count(),
        1
    );
}

struct LocalPostgres {
    root: PathBuf,
    socket: PathBuf,
    port: u16,
}

impl LocalPostgres {
    fn start() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sql-judge-pack-pg-{}-{}",
            std::process::id(),
            unique_suffix()
        ));
        let data = root.join("data");
        let socket = root.join("socket");
        fs::create_dir_all(&socket).expect("create socket directory");
        let port = TcpListener::bind(("127.0.0.1", 0))
            .expect("reserve port")
            .local_addr()
            .expect("read port")
            .port();
        let admin = std::env::var("USER").expect("USER identifies test owner");

        assert!(
            Command::new("initdb")
                .args([
                    "-D",
                    data.to_str().expect("data path"),
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
                    data.to_str().expect("data path"),
                    "-o",
                    &format!(
                        "-h 127.0.0.1 -p {port} -k {} -c listen_addresses=127.0.0.1",
                        socket.to_str().expect("socket path")
                    ),
                    "-w",
                    "start",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("start postgres")
                .success()
        );
        assert!(
            Command::new("createdb")
                .args([
                    "-h",
                    socket.to_str().expect("socket path"),
                    "-p",
                    &port.to_string(),
                    "sqljudge_test",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("create database")
                .success()
        );

        let mut psql = Command::new("psql")
            .args([
                "-h",
                socket.to_str().expect("socket path"),
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
            .expect("start setup psql");
        psql.stdin
            .take()
            .expect("open psql stdin")
            .write_all(
                b"CREATE ROLE sqljudge_runner LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT; REVOKE CREATE ON SCHEMA public FROM PUBLIC;",
            )
            .expect("write setup");
        assert!(psql.wait().expect("wait setup").success());
        Self { root, socket, port }
    }

    fn connection_string(&self) -> String {
        format!(
            "host={} port={} user={} dbname=sqljudge_test",
            self.socket.to_str().expect("socket path"),
            self.port,
            std::env::var("USER").expect("USER identifies test owner")
        )
    }

    fn mark_runtime(&self) {
        let sql = "CREATE TABLE public.__sql_judge_runtime (instance_id text PRIMARY KEY); INSERT INTO public.__sql_judge_runtime (instance_id) VALUES ('test-instance'); REVOKE ALL ON TABLE public.__sql_judge_runtime FROM PUBLIC;";
        assert!(
            Command::new("psql")
                .args([
                    "-h",
                    self.socket.to_str().expect("socket path"),
                    "-p",
                    &self.port.to_string(),
                    "-d",
                    "sqljudge_test",
                    "-v",
                    "ON_ERROR_STOP=1",
                    "-c",
                    sql,
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("mark managed runtime")
                .success()
        );
    }

    fn admin_session(&self) -> judge_postgres::ManagedPostgresSession {
        self.mark_runtime();
        judge_postgres::ManagedPostgresSession::connect(&self.connection_string(), "test-instance")
            .expect("connect managed test target")
    }
}

impl Drop for LocalPostgres {
    fn drop(&mut self) {
        let _ = Command::new("pg_ctl")
            .args([
                "-D",
                self.root.join("data").to_str().expect("data path"),
                "-w",
                "stop",
                "-m",
                "fast",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}

mod tempfile_like {
    use std::path::{Path, PathBuf};

    pub struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        pub fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "sql-judge-pack-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock")
                    .as_nanos()
            ));
            std::fs::create_dir_all(&path).expect("create temp directory");
            Self { path }
        }

        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
