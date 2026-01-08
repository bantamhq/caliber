use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::get_config_dir;

/// Entry in the registry file - stores only the path
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisteredProject {
    pub path: PathBuf,
}

/// Registry file format
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectRegistryFile {
    #[serde(default)]
    pub project: Vec<RegisteredProject>,
}

/// Resolved project info combining registry path with project config
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Path to .caliber/ directory
    pub path: PathBuf,
    /// Git root or parent of .caliber/
    pub root: PathBuf,
    /// Display name (from config or derived from folder)
    pub name: String,
    /// Unique identifier (from config or derived from folder)
    pub id: String,
    /// Whether the journal file exists
    pub available: bool,
    /// Whether the project is hidden from picker
    pub hidden: bool,
}

impl ProjectInfo {
    #[must_use]
    pub fn journal_path(&self) -> PathBuf {
        self.path.join("journal.md")
    }
}

/// Project registry with resolved project info
#[derive(Debug, Clone, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectInfo>,
}

impl ProjectRegistry {
    /// Load registry from disk and resolve all project info
    #[must_use]
    pub fn load() -> Self {
        let file = load_registry_file().unwrap_or_default();
        let mut projects = Vec::new();
        let mut seen_ids = Vec::new();

        for reg in file.project {
            let caliber_path = normalize_to_caliber_dir(&reg.path);
            if let Some(mut info) = resolve_project_info(&caliber_path) {
                let base_id = info.id.clone();
                let mut final_id = base_id.clone();
                let mut counter = 2;
                while seen_ids.iter().any(|id: &String| id.eq_ignore_ascii_case(&final_id)) {
                    final_id = format!("{}-{}", base_id, counter);
                    counter += 1;
                }
                info.id = final_id.clone();
                seen_ids.push(final_id);
                projects.push(info);
            }
        }

        Self { projects }
    }

    /// Save registry to disk (persists paths only)
    pub fn save(&self) -> io::Result<()> {
        let file = ProjectRegistryFile {
            project: self
                .projects
                .iter()
                .map(|p| RegisteredProject {
                    path: p.path.clone(),
                })
                .collect(),
        };

        let path = get_registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, content)
    }

    pub fn register(&mut self, path: PathBuf) -> io::Result<ProjectInfo> {
        let caliber_path = normalize_to_caliber_dir(&path);

        if self.find_by_path(&caliber_path).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Project already registered",
            ));
        }

        let Some(mut info) = resolve_project_info(&caliber_path) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Invalid project path - must be a .caliber/ directory",
            ));
        };

        let unique_id = self.generate_unique_id(&info.id);
        info.id = unique_id;

        write_project_identity(&caliber_path, &info.name, &info.id)?;

        self.projects.push(info.clone());
        Ok(info)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.projects.len();
        self.projects
            .retain(|p| !p.id.eq_ignore_ascii_case(id));
        self.projects.len() < len_before
    }

    /// Find project by ID (case-insensitive)
    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<&ProjectInfo> {
        self.projects
            .iter()
            .find(|p| p.id.eq_ignore_ascii_case(id))
    }

    /// Find project by path (accepts either .caliber/ or .caliber/journal.md)
    #[must_use]
    pub fn find_by_path(&self, path: &Path) -> Option<&ProjectInfo> {
        let caliber_path = normalize_to_caliber_dir(path);
        self.projects.iter().find(|p| p.path == caliber_path)
    }

    /// Generate a unique ID from a base, adding suffix for collisions
    #[must_use]
    pub fn generate_unique_id(&self, base: &str) -> String {
        let base_id = sanitize_id(base);

        if !self.id_exists(&base_id) {
            return base_id;
        }

        for n in 2..=99 {
            let candidate = format!("{}-{}", base_id, n);
            if !self.id_exists(&candidate) {
                return candidate;
            }
        }

        // Extremely unlikely fallback - use timestamp
        format!(
            "{}-{}",
            base_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        )
    }

    fn id_exists(&self, id: &str) -> bool {
        self.projects
            .iter()
            .any(|p| p.id.eq_ignore_ascii_case(id))
    }
}

#[must_use]
pub fn get_registry_path() -> PathBuf {
    get_config_dir().join("projects.toml")
}

fn load_registry_file() -> io::Result<ProjectRegistryFile> {
    let path = get_registry_path();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(ProjectRegistryFile::default())
    }
}

/// Accepts either /path/.caliber/ or /path/.caliber/journal.md
fn normalize_to_caliber_dir(path: &Path) -> PathBuf {
    if let Some(parent) = path.parent()
        && parent.file_name().and_then(|n| n.to_str()) == Some(".caliber")
    {
        return parent.to_path_buf();
    }
    path.to_path_buf()
}

fn resolve_project_info(caliber_path: &Path) -> Option<ProjectInfo> {
    if caliber_path.file_name()?.to_str()? != ".caliber" {
        return None;
    }
    let root = caliber_path.parent()?;

    let journal_path = caliber_path.join("journal.md");
    let available = journal_path.exists();

    let config_path = caliber_path.join("config.toml");
    let (name, id, hidden) = if config_path.exists() {
        load_project_identity(&config_path).unwrap_or_else(|| {
            let (n, i) = derive_identity(root);
            (n, i, false)
        })
    } else {
        let (n, i) = derive_identity(root);
        (n, i, false)
    };

    Some(ProjectInfo {
        path: caliber_path.to_path_buf(),
        root: root.to_path_buf(),
        name,
        id,
        available,
        hidden,
    })
}

fn load_project_identity(config_path: &Path) -> Option<(String, String, bool)> {
    #[derive(Deserialize)]
    struct ProjectConfig {
        name: Option<String>,
        id: Option<String>,
        #[serde(default)]
        hidden: bool,
    }

    let content = fs::read_to_string(config_path).ok()?;
    let config: ProjectConfig = toml::from_str(&content).ok()?;

    let root = config_path.parent()?.parent()?;
    let derived = derive_identity(root);

    Some((
        config.name.unwrap_or(derived.0),
        config.id.map(|id| sanitize_id(&id)).unwrap_or(derived.1),
        config.hidden,
    ))
}

/// Preserves other settings in config.toml
pub fn write_project_identity(caliber_path: &Path, name: &str, id: &str) -> io::Result<()> {
    let config_path = caliber_path.join("config.toml");

    let mut doc = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        content
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        toml_edit::DocumentMut::new()
    };

    doc["name"] = toml_edit::value(name);
    doc["id"] = toml_edit::value(id);

    fs::write(&config_path, doc.to_string())
}

/// Creates config.toml only if missing or empty
pub fn ensure_project_config(caliber_path: &Path, name: &str, id: &str) -> io::Result<()> {
    let config_path = caliber_path.join("config.toml");
    let config_missing = !config_path.exists()
        || fs::read_to_string(&config_path)
            .map(|s| s.trim().is_empty())
            .unwrap_or(true);

    if config_missing {
        write_project_identity(caliber_path, name, id)?;
    }
    Ok(())
}

fn derive_identity(root: &Path) -> (String, String) {
    let folder = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let name = capitalize_first(folder);
    let id = sanitize_id(folder);

    (name, id)
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => "Project".to_string(),
    }
}

/// Lowercase, alphanumeric + hyphens, no leading/trailing/consecutive hyphens
fn sanitize_id(s: &str) -> String {
    let id: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    let mut result = String::new();
    let mut prev_hyphen = true;
    for c in id.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    if result.ends_with('-') {
        result.pop();
    }

    if result.is_empty() {
        "project".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_id_handles_special_chars() {
        assert_eq!(sanitize_id("My App"), "my-app");
        assert_eq!(sanitize_id("my_app"), "my-app");
        assert_eq!(sanitize_id("my--app"), "my-app");
        assert_eq!(sanitize_id("--my-app--"), "my-app");
        assert_eq!(sanitize_id("MyApp123"), "myapp123");
        assert_eq!(sanitize_id(""), "project");
        assert_eq!(sanitize_id("---"), "project");
    }

    #[test]
    fn capitalize_first_works() {
        assert_eq!(capitalize_first("myapp"), "Myapp");
        assert_eq!(capitalize_first("my-app"), "My-app");
        assert_eq!(capitalize_first(""), "Project");
    }

    #[test]
    fn id_lookup_is_case_insensitive() {
        let mut registry = ProjectRegistry::default();
        registry.projects.push(ProjectInfo {
            path: PathBuf::from("/test/.caliber"),
            root: PathBuf::from("/test"),
            name: "Test".to_string(),
            id: "myapp".to_string(),
            available: true,
            hidden: false,
        });

        assert!(registry.find_by_id("myapp").is_some());
        assert!(registry.find_by_id("MYAPP").is_some());
        assert!(registry.find_by_id("MyApp").is_some());
        assert!(registry.find_by_id("other").is_none());
    }

    #[test]
    fn generate_unique_id_adds_disambiguator() {
        let mut registry = ProjectRegistry::default();
        registry.projects.push(ProjectInfo {
            path: PathBuf::from("/test/.caliber"),
            root: PathBuf::from("/test"),
            name: "Test".to_string(),
            id: "myapp".to_string(),
            available: true,
            hidden: false,
        });

        assert_eq!(registry.generate_unique_id("other"), "other");
        assert_eq!(registry.generate_unique_id("myapp"), "myapp-2");

        registry.projects.push(ProjectInfo {
            path: PathBuf::from("/test2/.caliber"),
            root: PathBuf::from("/test2"),
            name: "Test 2".to_string(),
            id: "myapp-2".to_string(),
            available: true,
            hidden: false,
        });

        assert_eq!(registry.generate_unique_id("myapp"), "myapp-3");
    }

    #[test]
    fn remove_by_id_is_case_insensitive() {
        let mut registry = ProjectRegistry::default();
        registry.projects.push(ProjectInfo {
            path: PathBuf::from("/test/.caliber"),
            root: PathBuf::from("/test"),
            name: "Test".to_string(),
            id: "myapp".to_string(),
            available: true,
            hidden: false,
        });

        assert!(registry.remove("MYAPP"));
        assert!(registry.projects.is_empty());
    }

    #[test]
    fn find_by_path_accepts_journal_path() {
        let mut registry = ProjectRegistry::default();
        registry.projects.push(ProjectInfo {
            path: PathBuf::from("/test/.caliber"),
            root: PathBuf::from("/test"),
            name: "Test".to_string(),
            id: "myapp".to_string(),
            available: true,
            hidden: false,
        });

        assert!(registry.find_by_path(Path::new("/test/.caliber")).is_some());
        assert!(registry.find_by_path(Path::new("/test/.caliber/journal.md")).is_some());
    }
}
