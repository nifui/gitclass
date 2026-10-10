use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use time::OffsetDateTime;

/// A setting that the user may explicitly constrain or delegate
/// to the scheduler.
///
/// For the current schema this is used for numeric resource limits
/// and step timeouts.
///
/// TOML:
///     memory_mb = 512
///     memory_mb = "auto"
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Setting<T> {
    Value(T),
    Auto(AutoValue),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoValue {
    Auto,
}

impl<T> Default for Setting<T> {
    fn default() -> Self {
        Self::Auto(AutoValue::Auto)
    }
}

impl<T> Setting<T> {
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto(_))
    }

    pub const fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Auto(_) => None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Workflow {
    #[serde(default)]
    pub metadata: Metadata,

    /// The runtime is required to execute the task, but the individual
    /// dependency settings are optional.
    pub runtime: Runtime,

    #[serde(default)]
    pub network: Network,

    #[serde(default)]
    pub filesystem: Filesystem,

    #[serde(default)]
    pub limits: Limits,

    #[serde(default)]
    pub environment: Environment,

    #[serde(default)]
    pub artifacts: Artifacts,

    #[serde(default)]
    pub cleanup: Cleanup,

    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    #[serde(default)]
    pub name: String,

    #[serde(default = "default_metadata_version")]
    pub version: u32,

    #[serde(default = "default_metadata_date")]
    pub date: OffsetDateTime,

    /// `workflow` means the document is a complete workflow.
    /// `step` means the document represents one reusable execution step.
    /// Validation should require exactly one entry in `steps` for this type.
    #[serde(rename = "type", default)]
    pub workflow_type: WorkflowType,

    /// Other workflow documents that must successfully execute before
    /// this document may execute.
    ///
    /// The referenced document declares its own type.
    #[serde(default)]
    pub requires: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_metadata_version(),
            date: default_metadata_date(),
            workflow_type: WorkflowType::default(),
            requires: Vec::default(),
        }
    }
}

const fn default_metadata_version() -> u32 {
    1
}

const fn default_metadata_date() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    #[default]
    Workflow,

    Step,
}

/// Runtime requirements are resolved by the scheduler into a concrete
/// immutable container image/digest.
#[derive(Deserialize, Debug)]
pub struct Runtime {
    pub language: String,
    pub version: String,

    #[serde(default)]
    pub dependencies: RuntimeDependencies,
}

#[derive(Deserialize, Debug, Default)]
pub struct RuntimeDependencies {
    /// Path relative to the workflow/project workspace.
    pub file: Option<String>,

    /// SHA-256 of the dependency lockfile.
    ///
    /// The scheduler can use this to identify an immutable cached
    /// runtime image/environment.
    pub sha256: Option<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for Runtime {
    fn default() -> Self {
        Self {
            language: String::new(),
            version: String::new(),
            dependencies: RuntimeDependencies::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Network {
    /// Networking is intentionally a plain bool rather than `auto`.
    /// Enabling network access is a security-sensitive user decision.
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub protocols: Vec<NetworkProtocol>,

    #[serde(default)]
    pub ports: Vec<u16>,
}
#[allow(clippy::derivable_impls)]
impl Default for Network {
    fn default() -> Self {
        Self {
            enabled: false,
            protocols: Vec::new(),
            ports: Vec::new(),
        }
    }
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum NetworkProtocol {
    #[default]
    Tcp,
    Udp,
}

#[derive(Deserialize, Debug, Default)]
pub struct Filesystem {
    #[serde(default)]
    pub mode: FilesystemMode,

    /// Paths inside the execution environment which should be exposed
    /// read-only.
    #[serde(default)]
    pub read_only: Vec<String>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemMode {
    #[default]
    Workspace,
}

#[derive(Deserialize, Debug, Default)]
pub struct Limits {
    #[serde(default)]
    pub timeout_seconds: Setting<u64>,

    #[serde(default)]
    pub cpu_seconds: Setting<u64>,

    #[serde(default)]
    pub memory_mb: Setting<u64>,

    #[serde(default)]
    pub disk_mb: Setting<u64>,

    #[serde(default)]
    pub processes: Setting<u32>,

    #[serde(default)]
    pub output_mb: Setting<u64>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Environment {
    /// Ordinary non-secret environment variables.
    #[serde(default)]
    pub variables: HashMap<String, String>,

    /// Environment variable name -> secret reference.
    ///
    /// Example:
    ///
    ///     API_KEY = "secret://api-key"
    #[serde(default)]
    pub secrets: HashMap<String, SecretRef>,
}

#[derive(Debug)]
pub struct SecretRef {
    pub reference: String,
}

impl<'de> Deserialize<'de> for SecretRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let reference = String::deserialize(deserializer)?;

        Ok(Self { reference })
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Artifacts {
    /// Paths relative to the workspace that should be preserved as
    /// workflow artifacts.
    #[serde(default)]
    pub include: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Cleanup {
    /// Cleanup should happen by default.
    #[serde(default = "default_cleanup")]
    pub workspace: bool,

    #[serde(default = "default_cleanup")]
    pub processes: bool,
}

impl Default for Cleanup {
    fn default() -> Self {
        Self {
            workspace: true,
            processes: true,
        }
    }
}

const fn default_cleanup() -> bool {
    true
}

#[derive(Deserialize, Debug, Default)]
pub struct Step {
    pub name: String,

    #[serde(rename = "type", default)]
    pub step_type: StepType,

    /// Required for `command` steps.
    ///
    /// Empty for scheduler-defined steps such as
    /// `install_dependencies`.
    #[serde(default)]
    pub command: Vec<String>,

    /// `auto` means the scheduler/workflow policy decides.
    #[serde(default)]
    pub timeout_seconds: Setting<u64>,

    /// Dependencies on other steps in this same document.
    #[serde(default)]
    pub requires: Vec<String>,

    /// Environment variable names made available to this step.
    ///
    /// These names can refer to either `[environment.variables]`
    /// or `[environment.secrets]`.
    #[serde(default)]
    pub environment: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    #[default]
    Command,

    InstallDependencies,
}
