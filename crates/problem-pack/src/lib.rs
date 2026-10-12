use judge_core::{Column, Dialect, OutputPolicy, QueryLimits, RowOrder};
use judge_postgres::ManagedPostgresSession;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_FILES: usize = 2_000;
const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PackData {
    files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ValidatedPack {
    files: BTreeMap<String, Vec<u8>>,
    manifest: PackManifest,
    digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedPack {
    pub digest: String,
    pub path: PathBuf,
    pub already_installed: bool,
}

#[derive(Debug)]
pub struct PackImporter {
    registry: PathBuf,
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("pack source must be a real directory")]
    SourceNotDirectory,
    #[error("pack contains a symlink: {0}")]
    Symlink(String),
    #[error("pack contains an unsafe path: {0}")]
    UnsafePath(String),
    #[error("pack contains a non-regular file: {0}")]
    NonRegularFile(String),
    #[error("pack contains duplicate or case-colliding path: {0}")]
    DuplicatePath(String),
    #[error("pack resource limits exceeded")]
    ResourceLimit,
    #[error("pack filesystem operation failed")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON in {file}")]
    InvalidJson { file: String },
    #[error("invalid pack manifest: {0}")]
    InvalidManifest(String),
    #[error("missing pack file: {0}")]
    MissingFile(String),
    #[error("undeclared pack file: {0}")]
    UndeclaredFile(String),
    #[error("invalid SQL file: {0}")]
    InvalidSql(String),
    #[error("pack dialect is not supported by the managed judge")]
    UnsupportedDialect,
    #[error("runtime validation failed for {problem}/{test}")]
    RuntimeValidation { problem: String, test: String },
    #[error("immutable pack version conflict for {pack_id} {version}")]
    ImmutableVersionConflict { pack_id: String, version: String },
    #[error("published pack destination is not a directory")]
    InvalidPublishedDestination,
    #[error("pack publication failed")]
    Publication,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackManifest {
    spec_version: String,
    id: String,
    version: String,
    title: String,
    dialect: Dialect,
    #[serde(default)]
    min_engine_version: Option<String>,
    schema: Vec<String>,
    problems: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProblemManifest {
    id: String,
    title: String,
    difficulty: Difficulty,
    topics: Vec<String>,
    statement: String,
    dialect: Dialect,
    reference: String,
    output: OutputManifest,
    limits: QueryLimits,
    tests: Vec<TestManifest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputManifest {
    row_order: RowOrder,
    strict_column_names: bool,
    strict_column_types: bool,
    columns: Vec<Column>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestManifest {
    id: String,
    file: String,
    hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct InstalledIdentity {
    id: String,
    version: String,
}

impl PackData {
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self, PackError> {
        let root = path.as_ref();
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() {
            return Err(PackError::SourceNotDirectory);
        }

        let mut files = BTreeMap::new();
        collect_directory(root, root, &mut files)?;
        Ok(Self { files })
    }

    pub fn from_entries<I>(entries: I) -> Result<Self, PackError>
    where
        I: IntoIterator<Item = (String, Vec<u8>)>,
    {
        let mut files = BTreeMap::new();
        let mut folded = BTreeSet::new();
        for (name, content) in entries {
            validate_path(&name)?;
            if !folded.insert(name.to_lowercase()) {
                return Err(PackError::DuplicatePath(name));
            }
            if content.len() > MAX_FILE_BYTES {
                return Err(PackError::ResourceLimit);
            }
            files.insert(name, content);
            if files.len() > MAX_FILES {
                return Err(PackError::ResourceLimit);
            }
        }
        if files.values().map(Vec::len).sum::<usize>() > MAX_TOTAL_BYTES {
            return Err(PackError::ResourceLimit);
        }
        Ok(Self { files })
    }

    pub fn validate(&self) -> Result<ValidatedPack, PackError> {
        if self.files.is_empty()
            || self.files.len() > MAX_FILES
            || self.files.values().map(Vec::len).sum::<usize>() > MAX_TOTAL_BYTES
        {
            return Err(PackError::ResourceLimit);
        }
        for (name, content) in &self.files {
            validate_path(name)?;
            if content.len() > MAX_FILE_BYTES {
                return Err(PackError::ResourceLimit);
            }
        }
        let mut folded = BTreeSet::new();
        for name in self.files.keys() {
            if !folded.insert(name.to_lowercase()) {
                return Err(PackError::DuplicatePath(name.clone()));
            }
        }

        let manifest: PackManifest = json_file(&self.files, "pack.json")?;
        if manifest.spec_version != "sqljudge.pack/v1"
            || !is_id(&manifest.id)
            || !is_semver(&manifest.version)
            || !(3..=140).contains(&manifest.title.chars().count())
            || manifest.schema.is_empty()
            || manifest.schema.len() > 30
            || manifest.problems.is_empty()
            || manifest.problems.len() > 500
        {
            return Err(PackError::InvalidManifest(
                "pack.json contract violation".to_owned(),
            ));
        }
        if manifest
            .min_engine_version
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.chars().count() > 30)
        {
            return Err(PackError::InvalidManifest(
                "invalid minimum engine version".to_owned(),
            ));
        }

        let mut expected = BTreeSet::from(["pack.json".to_owned()]);
        let mut schema_paths = BTreeSet::new();
        for path in &manifest.schema {
            if !is_schema_path(path) || !schema_paths.insert(path.clone()) {
                return Err(PackError::InvalidManifest(
                    "invalid schema path list".to_owned(),
                ));
            }
            expected.insert(path.clone());
            check_sql(&self.files, path)?;
        }

        let mut problem_paths = BTreeSet::new();
        let mut problem_ids = BTreeSet::new();
        for path in &manifest.problems {
            if !is_problem_path(path) || !problem_paths.insert(path.clone()) {
                return Err(PackError::InvalidManifest(
                    "invalid problem path list".to_owned(),
                ));
            }
            expected.insert(path.clone());
            let problem: ProblemManifest = json_file(&self.files, path)?;
            validate_problem(
                &self.files,
                &manifest,
                path,
                &problem,
                &mut expected,
                &mut problem_ids,
            )?;
        }

        for path in self.files.keys() {
            if !expected.contains(path) && path != "README.md" && path != "LICENSE" {
                return Err(PackError::UndeclaredFile(path.clone()));
            }
        }

        Ok(ValidatedPack {
            files: self.files.clone(),
            manifest,
            digest: content_digest(&self.files),
        })
    }
}

impl PackImporter {
    pub fn new(registry: impl Into<PathBuf>) -> Self {
        Self {
            registry: registry.into(),
        }
    }

    pub fn import(
        &self,
        pack: &ValidatedPack,
        session: &mut ManagedPostgresSession,
        runner_role: &str,
    ) -> Result<PublishedPack, PackError> {
        if pack.manifest.dialect != Dialect::Postgresql {
            return Err(PackError::UnsupportedDialect);
        }
        validate_runtime(pack, session, runner_role)?;
        self.publish(pack)
    }

    fn publish(&self, pack: &ValidatedPack) -> Result<PublishedPack, PackError> {
        let digest_root = self.registry.join("sha256");
        let destination = digest_root.join(&pack.digest);
        if let Some(metadata) = symlink_metadata_if_present(&destination)? {
            if !metadata.is_dir() {
                return Err(PackError::InvalidPublishedDestination);
            }
            let installed = PackData::from_directory(&destination)
                .map_err(|_| PackError::InvalidPublishedDestination)?;
            if content_digest(&installed.files) != pack.digest {
                return Err(PackError::InvalidPublishedDestination);
            }
            return Ok(PublishedPack {
                digest: pack.digest.clone(),
                path: destination,
                already_installed: true,
            });
        }

        if digest_root.is_dir() {
            for entry in fs::read_dir(&digest_root)? {
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                let identity_path = entry.path().join("pack.json");
                if !identity_path.is_file() {
                    continue;
                }
                let identity: InstalledIdentity =
                    match serde_json::from_slice(&fs::read(identity_path)?) {
                        Ok(identity) => identity,
                        Err(_) => continue,
                    };
                if identity.id == pack.manifest.id && identity.version == pack.manifest.version {
                    return Err(PackError::ImmutableVersionConflict {
                        pack_id: identity.id,
                        version: identity.version,
                    });
                }
            }
        }

        let root_existed = self.registry.exists();
        let digest_root_existed = digest_root.exists();
        let stage = self.registry.join(format!(
            ".staging-{}-{}",
            std::process::id(),
            unique_suffix()
        ));
        let result = (|| {
            fs::create_dir_all(&digest_root)?;
            fs::create_dir(&stage)?;
            for (name, content) in &pack.files {
                let target = stage.join(name);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(target, content)?;
            }
            fs::rename(&stage, &destination).map_err(|_| PackError::Publication)
        })();

        if result.is_err() {
            let _ = fs::remove_dir_all(&stage);
            if !digest_root_existed {
                let _ = fs::remove_dir(&digest_root);
            }
            if !root_existed {
                let _ = fs::remove_dir(&self.registry);
            }
        }
        result.map(|_| PublishedPack {
            digest: pack.digest.clone(),
            path: destination,
            already_installed: false,
        })
    }
}

fn validate_runtime(
    pack: &ValidatedPack,
    session: &mut ManagedPostgresSession,
    runner_role: &str,
) -> Result<(), PackError> {
    let schema_sql = pack
        .manifest
        .schema
        .iter()
        .map(|path| sql_file(&pack.files, path))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    for problem_path in &pack.manifest.problems {
        let problem: ProblemManifest = json_file(&pack.files, problem_path)?;
        let root = Path::new(problem_path)
            .parent()
            .ok_or_else(|| PackError::InvalidManifest("problem path has no parent".to_owned()))?;
        let reference_path = root.join(&problem.reference).to_string_lossy().into_owned();
        let reference = sql_file(&pack.files, &reference_path)?;
        for test in &problem.tests {
            let test_path = root.join(&test.file).to_string_lossy().into_owned();
            let fixture = sql_file(&pack.files, &test_path)?;
            let result =
                session.with_managed_fixture(runner_role, &schema_sql, &fixture, |session| {
                    session.execute(&reference, problem.limits)
                });
            let result = result.map_err(|_| PackError::RuntimeValidation {
                problem: problem.id.clone(),
                test: test.id.clone(),
            })?;
            if result.columns != problem.output.columns || result.validate().is_err() {
                return Err(PackError::RuntimeValidation {
                    problem: problem.id.clone(),
                    test: test.id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_problem(
    files: &BTreeMap<String, Vec<u8>>,
    pack: &PackManifest,
    path: &str,
    problem: &ProblemManifest,
    expected: &mut BTreeSet<String>,
    problem_ids: &mut BTreeSet<String>,
) -> Result<(), PackError> {
    let root = Path::new(path)
        .parent()
        .ok_or_else(|| PackError::InvalidManifest("problem path has no parent".to_owned()))?;
    if root.file_name().and_then(|name| name.to_str()) != Some(problem.id.as_str())
        || !is_id(&problem.id)
        || !problem_ids.insert(problem.id.clone())
        || !(3..=140).contains(&problem.title.chars().count())
        || problem.topics.is_empty()
        || problem.topics.len() > 15
        || problem.statement.chars().count() < 20
        || problem.statement.chars().count() > 12_000
        || problem.dialect != pack.dialect
        || problem.reference != "reference.sql"
        || problem.output.columns.is_empty()
        || problem.output.columns.len() > 128
        || problem.tests.len() < 3
        || problem.tests.len() > 50
    {
        return Err(PackError::InvalidManifest(format!(
            "problem contract violation: {path}"
        )));
    }
    if !problem.topics.iter().all(|topic| is_id(topic))
        || problem.topics.iter().collect::<BTreeSet<_>>().len() != problem.topics.len()
        || problem
            .output
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != problem.output.columns.len()
    {
        return Err(PackError::InvalidManifest(format!(
            "problem uniqueness violation: {path}"
        )));
    }
    match problem.difficulty {
        Difficulty::Easy | Difficulty::Medium | Difficulty::Hard => {}
    }
    let output_policy = OutputPolicy::new(
        problem.output.row_order,
        problem.output.strict_column_names,
        problem.output.strict_column_types,
        problem.output.columns.clone(),
    );
    output_policy
        .validate()
        .map_err(|error| PackError::InvalidManifest(error.to_string()))?;
    problem
        .output
        .columns
        .iter()
        .try_for_each(validate_column)?;
    problem
        .limits
        .validate()
        .map_err(|error| PackError::InvalidManifest(error.to_string()))?;

    let reference_path = root.join(&problem.reference).to_string_lossy().into_owned();
    expected.insert(reference_path.clone());
    check_sql(files, &reference_path)?;

    let mut test_ids = BTreeSet::new();
    let mut test_paths = BTreeSet::new();
    let mut visible = 0;
    let mut hidden = 0;
    for test in &problem.tests {
        if !is_id(&test.id)
            || !is_test_path(&test.file)
            || !test_ids.insert(test.id.clone())
            || !test_paths.insert(test.file.clone())
        {
            return Err(PackError::InvalidManifest(format!(
                "test contract violation: {path}"
            )));
        }
        if test.hidden {
            hidden += 1;
        } else {
            visible += 1;
        }
        let test_path = root.join(&test.file).to_string_lossy().into_owned();
        expected.insert(test_path.clone());
        check_sql(files, &test_path)?;
    }
    if visible < 1 || hidden < 2 {
        return Err(PackError::InvalidManifest(format!(
            "fixture visibility violation: {path}"
        )));
    }
    Ok(())
}

fn validate_column(column: &Column) -> Result<(), PackError> {
    if column.name.is_empty()
        || !column.name.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character.is_ascii_alphanumeric()
                    && (index > 0 || character.is_ascii_alphabetic())
        })
    {
        return Err(PackError::InvalidManifest(
            "invalid output column".to_owned(),
        ));
    }
    Ok(())
}

fn collect_directory(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), PackError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| PackError::UnsafePath(path.display().to_string()))?;
        let name = relative
            .to_str()
            .ok_or_else(|| PackError::UnsafePath(relative.display().to_string()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(PackError::Symlink(name));
        }
        if metadata.is_dir() {
            collect_directory(root, &path, files)?;
        } else if metadata.is_file() {
            validate_path(&name)?;
            if metadata.len() > MAX_FILE_BYTES as u64 {
                return Err(PackError::ResourceLimit);
            }
            if files.len() >= MAX_FILES {
                return Err(PackError::ResourceLimit);
            }
            let current_size = files.values().map(Vec::len).sum::<usize>();
            if metadata.len() > (MAX_TOTAL_BYTES - current_size) as u64 {
                return Err(PackError::ResourceLimit);
            }
            files.insert(name, fs::read(path)?);
        } else {
            return Err(PackError::NonRegularFile(name));
        }
    }
    if files.values().map(Vec::len).sum::<usize>() > MAX_TOTAL_BYTES {
        return Err(PackError::ResourceLimit);
    }
    Ok(())
}

fn json_file<T: DeserializeOwned>(
    files: &BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<T, PackError> {
    let bytes = files
        .get(name)
        .ok_or_else(|| PackError::MissingFile(name.to_owned()))?;
    serde_json::from_slice(bytes).map_err(|_| PackError::InvalidJson {
        file: name.to_owned(),
    })
}

fn sql_file(files: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<String, PackError> {
    let bytes = files
        .get(name)
        .ok_or_else(|| PackError::MissingFile(name.to_owned()))?;
    let value = std::str::from_utf8(bytes).map_err(|_| PackError::InvalidSql(name.to_owned()))?;
    if value.trim().is_empty() || value.contains('\0') {
        return Err(PackError::InvalidSql(name.to_owned()));
    }
    Ok(value.to_owned())
}

fn check_sql(files: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<(), PackError> {
    sql_file(files, name).map(|_| ())
}

fn validate_path(name: &str) -> Result<(), PackError> {
    if name.is_empty()
        || name.contains('\\')
        || name.contains('\0')
        || name.starts_with('/')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || (name.as_bytes().get(1) == Some(&b':'))
    {
        return Err(PackError::UnsafePath(name.to_owned()));
    }
    Ok(())
}

fn is_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase())
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && !value.contains("--")
        && !value.ends_with('-')
}

fn is_semver(value: &str) -> bool {
    let parts: Vec<_> = value.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part == &"0" || !part.starts_with('0'))
                && part.chars().all(|c| c.is_ascii_digit())
        })
}

fn is_schema_path(value: &str) -> bool {
    value.strip_prefix("schema/").is_some_and(|name| {
        name.ends_with(".sql") && name[..name.len() - 4].chars().all(path_character)
    })
}

fn is_problem_path(value: &str) -> bool {
    value
        .strip_prefix("problems/")
        .and_then(|value| value.strip_suffix("/problem.json"))
        .is_some_and(is_id)
}

fn is_test_path(value: &str) -> bool {
    value
        .strip_prefix("tests/")
        .and_then(|value| value.strip_suffix(".sql"))
        .is_some_and(is_id)
}

fn path_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

fn content_digest(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    for (name, content) in files {
        hasher.update((name.len() as u64).to_be_bytes());
        hasher.update(name.as_bytes());
        hasher.update((content.len() as u64).to_be_bytes());
        hasher.update(content);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn symlink_metadata_if_present(path: &Path) -> Result<Option<std::fs::Metadata>, std::io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
