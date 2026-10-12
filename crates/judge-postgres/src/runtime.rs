use super::{ManagedPostgresSession, PostgresSession, QueryExecutionError};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct ManagedPostgresRuntime {
    root: PathBuf,
    socket: PathBuf,
    user: String,
    stop_on_drop: bool,
}

impl ManagedPostgresRuntime {
    pub fn start(root: impl AsRef<Path>, instance_id: &str) -> Result<Self, QueryExecutionError> {
        if instance_id.trim().is_empty() {
            return Err(QueryExecutionError::PolicyViolation);
        }

        let root = root.as_ref().to_path_buf();
        if !root.is_absolute() {
            return Err(QueryExecutionError::PolicyViolation);
        }
        let fresh = !root.exists();
        if !fresh && !root.join("PG_VERSION").is_file() {
            return Err(QueryExecutionError::PolicyViolation);
        }
        let socket = root.join("socket");
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .map_err(|_| QueryExecutionError::ConnectionError)?;
        if user.trim().is_empty() {
            return Err(QueryExecutionError::ConnectionError);
        }

        let parent = root.parent().ok_or(QueryExecutionError::PolicyViolation)?;
        fs::create_dir_all(parent).map_err(|_| QueryExecutionError::ConnectionError)?;
        if fresh {
            fs::create_dir(&root).map_err(|_| QueryExecutionError::ConnectionError)?;
            let initialized = Command::new("initdb")
                .args([
                    "-D",
                    root.to_str().ok_or(QueryExecutionError::PolicyViolation)?,
                    "--username",
                    &user,
                    "--auth-local=peer",
                    "--auth-host=reject",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|_| QueryExecutionError::ConnectionError)?;
            if !initialized.success() {
                let _ = fs::remove_dir_all(&root);
                return Err(QueryExecutionError::ConnectionError);
            }
        }
        if socket.exists() {
            if !socket.is_dir() {
                return Err(QueryExecutionError::PolicyViolation);
            }
        } else if fs::create_dir(&socket).is_err() {
            if fresh {
                let _ = fs::remove_dir_all(&root);
            }
            return Err(QueryExecutionError::ConnectionError);
        }

        let root_string = root.to_str().ok_or(QueryExecutionError::PolicyViolation)?;
        let running = Command::new("pg_ctl")
            .args(["-D", root_string, "status"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !running {
            let options = format!(
                "-k {} -p 5432 -c listen_addresses=''",
                socket
                    .to_str()
                    .ok_or(QueryExecutionError::PolicyViolation)?
            );
            let started = Command::new("pg_ctl")
                .args(["-D", root_string, "-o", &options, "-w", "start"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|_| QueryExecutionError::ConnectionError)?;
            if !started.success() {
                if fresh {
                    let _ = fs::remove_dir_all(&root);
                }
                return Err(QueryExecutionError::ConnectionError);
            }
        }

        let mut runtime = Self {
            root,
            socket,
            user,
            stop_on_drop: !running,
        };
        let initialized = if fresh {
            runtime.initialize(instance_id)
        } else {
            ManagedPostgresSession::connect(&runtime.connection_string(), instance_id).map(|_| ())
        };
        if let Err(error) = initialized {
            drop(runtime);
            return Err(error);
        }
        runtime.stop_on_drop = true;
        Ok(runtime)
    }

    pub fn connection_string(&self) -> String {
        format!(
            "host={} user={} dbname=postgres",
            self.socket.to_string_lossy(),
            self.user
        )
    }

    fn initialize(&self, instance_id: &str) -> Result<(), QueryExecutionError> {
        let mut session = PostgresSession::connect(&self.connection_string())?;
        session
            .client
            .batch_execute(
                "CREATE ROLE sqljudge_runner NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS NOREPLICATION NOLOGIN;
                 REVOKE CREATE ON SCHEMA public FROM PUBLIC;
                 CREATE TABLE public.__sql_judge_runtime (instance_id text PRIMARY KEY)",
            )
            .map_err(|_| QueryExecutionError::RuntimeError)?;
        session
            .client
            .execute(
                "INSERT INTO public.__sql_judge_runtime (instance_id) VALUES ($1)",
                &[&instance_id],
            )
            .map_err(|_| QueryExecutionError::RuntimeError)?;
        session
            .client
            .batch_execute("REVOKE ALL ON TABLE public.__sql_judge_runtime FROM PUBLIC")
            .map_err(|_| QueryExecutionError::RuntimeError)
    }
}

impl Drop for ManagedPostgresRuntime {
    fn drop(&mut self) {
        if !self.stop_on_drop {
            return;
        }
        let _ = Command::new("pg_ctl")
            .args([
                "-D",
                self.root.to_string_lossy().as_ref(),
                "-w",
                "stop",
                "-m",
                "fast",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
