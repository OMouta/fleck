use crate::model::{
    Asset, AssetSource, ObjectId, ValidationError, Workspace, CURRENT_WORKSPACE_FORMAT_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const CURRENT_FLECK_FILE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspacePackage {
    pub file_format_version: u32,
    pub workspace: Workspace,
    pub embedded_assets: Vec<EmbeddedAssetBlob>,
}

impl WorkspacePackage {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            file_format_version: CURRENT_FLECK_FILE_FORMAT_VERSION,
            workspace,
            embedded_assets: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), PackageValidationError> {
        self.workspace
            .validate()
            .map_err(PackageValidationError::Workspace)?;

        let asset_ids = self
            .workspace
            .assets
            .iter()
            .map(|asset| &asset.id)
            .collect::<HashSet<_>>();
        let embedded_asset_ids = self
            .workspace
            .assets
            .iter()
            .filter_map(|asset| match asset.source {
                AssetSource::Embedded { .. } => Some(&asset.id),
                AssetSource::Linked { .. } => None,
            })
            .collect::<HashSet<_>>();

        let mut seen_blobs = HashSet::new();
        let mut issues = Vec::new();

        for blob in &self.embedded_assets {
            if !seen_blobs.insert(&blob.asset_id) {
                issues.push(PackageValidationIssue::DuplicateEmbeddedAssetBlob {
                    asset_id: blob.asset_id.clone(),
                });
            }
            if !asset_ids.contains(&blob.asset_id) {
                issues.push(PackageValidationIssue::EmbeddedAssetBlobWithoutAsset {
                    asset_id: blob.asset_id.clone(),
                });
            }
            if !embedded_asset_ids.contains(&blob.asset_id) {
                issues.push(
                    PackageValidationIssue::EmbeddedAssetBlobForNonEmbeddedAsset {
                        asset_id: blob.asset_id.clone(),
                    },
                );
            }
        }

        let blob_ids = self
            .embedded_assets
            .iter()
            .map(|blob| &blob.asset_id)
            .collect::<HashSet<_>>();

        for asset_id in embedded_asset_ids {
            if !blob_ids.contains(asset_id) {
                issues.push(PackageValidationIssue::EmbeddedAssetWithoutBlob {
                    asset_id: asset_id.clone(),
                });
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(PackageValidationError::Package { issues })
        }
    }

    pub fn missing_linked_assets(&self, workspace_dir: impl AsRef<Path>) -> Vec<LinkedAssetReport> {
        self.workspace
            .assets
            .iter()
            .filter_map(|asset| linked_asset_report(asset, workspace_dir.as_ref()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedAssetBlob {
    pub asset_id: ObjectId,
    pub digest: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadOutcome {
    pub package: WorkspacePackage,
    pub warnings: Vec<LoadWarning>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadWarning {
    MigratedFileFormat { from: u32, to: u32 },
    NewerFileFormat { found: u32, supported: u32 },
    NewerWorkspaceFormat { found: u32, supported: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkedAssetReport {
    pub asset_id: ObjectId,
    pub name: String,
    pub path: PathBuf,
    pub resolved_path: PathBuf,
}

pub fn save_package_to_writer(
    package: &WorkspacePackage,
    mut writer: impl Write,
) -> Result<(), SaveError> {
    package.validate()?;
    serde_json::to_writer_pretty(&mut writer, package)?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn save_package_to_path(
    package: &WorkspacePackage,
    path: impl AsRef<Path>,
) -> Result<(), SaveError> {
    let file = fs::File::create(path)?;
    save_package_to_writer(package, file)
}

pub fn load_package_from_reader(mut reader: impl Read) -> Result<LoadOutcome, LoadError> {
    let mut source = String::new();
    reader.read_to_string(&mut source)?;
    load_package_from_str(&source)
}

pub fn load_package_from_path(path: impl AsRef<Path>) -> Result<LoadOutcome, LoadError> {
    let file = fs::File::open(path)?;
    load_package_from_reader(file)
}

pub fn load_package_from_str(source: &str) -> Result<LoadOutcome, LoadError> {
    let value = serde_json::from_str::<Value>(source)?;
    let (value, mut warnings) = migrate_package_value(value)?;
    let package = serde_json::from_value::<WorkspacePackage>(value)?;

    if package.file_format_version > CURRENT_FLECK_FILE_FORMAT_VERSION {
        warnings.push(LoadWarning::NewerFileFormat {
            found: package.file_format_version,
            supported: CURRENT_FLECK_FILE_FORMAT_VERSION,
        });
    }

    if package.workspace.format_version > CURRENT_WORKSPACE_FORMAT_VERSION {
        warnings.push(LoadWarning::NewerWorkspaceFormat {
            found: package.workspace.format_version,
            supported: CURRENT_WORKSPACE_FORMAT_VERSION,
        });
    }

    package.validate()?;
    Ok(LoadOutcome { package, warnings })
}

fn migrate_package_value(mut value: Value) -> Result<(Value, Vec<LoadWarning>), LoadError> {
    let Some(object) = value.as_object_mut() else {
        return Err(LoadError::InvalidPackage(
            "expected .fleck package to be a JSON object",
        ));
    };

    let version = object
        .get("file_format_version")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;

    if version == 0 {
        object.insert(
            "file_format_version".to_owned(),
            Value::from(CURRENT_FLECK_FILE_FORMAT_VERSION),
        );
        object
            .entry("embedded_assets".to_owned())
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Some(workspace) = object.get_mut("workspace").and_then(Value::as_object_mut) {
            workspace.insert(
                "format_version".to_owned(),
                Value::from(CURRENT_WORKSPACE_FORMAT_VERSION),
            );
        }

        return Ok((
            value,
            vec![LoadWarning::MigratedFileFormat {
                from: 0,
                to: CURRENT_FLECK_FILE_FORMAT_VERSION,
            }],
        ));
    }

    Ok((value, Vec::new()))
}

fn linked_asset_report(asset: &Asset, workspace_dir: &Path) -> Option<LinkedAssetReport> {
    let AssetSource::Linked { path } = &asset.source else {
        return None;
    };

    let path = PathBuf::from(path);
    let resolved_path = if path.is_absolute() {
        path.clone()
    } else {
        workspace_dir.join(&path)
    };

    if resolved_path.exists() {
        None
    } else {
        Some(LinkedAssetReport {
            asset_id: asset.id.clone(),
            name: asset.name.clone(),
            path,
            resolved_path,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("package validation failed")]
    Validation(#[from] PackageValidationError),
    #[error("failed to serialize .fleck package")]
    Json(#[from] serde_json::Error),
    #[error("failed to write .fleck package")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to read .fleck package")]
    Io(#[from] std::io::Error),
    #[error("failed to parse .fleck package JSON")]
    Json(#[from] serde_json::Error),
    #[error("invalid .fleck package: {0}")]
    InvalidPackage(&'static str),
    #[error("package validation failed")]
    Validation(#[from] PackageValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum PackageValidationError {
    #[error("workspace validation failed")]
    Workspace(#[from] ValidationError),
    #[error("package validation failed with {issue_count} issue(s)", issue_count = .issues.len())]
    Package { issues: Vec<PackageValidationIssue> },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PackageValidationIssue {
    #[error("embedded asset `{asset_id}` is missing stored bytes")]
    EmbeddedAssetWithoutBlob { asset_id: ObjectId },
    #[error("embedded asset blob `{asset_id}` has no matching asset")]
    EmbeddedAssetBlobWithoutAsset { asset_id: ObjectId },
    #[error("embedded asset blob `{asset_id}` points to a non-embedded asset")]
    EmbeddedAssetBlobForNonEmbeddedAsset { asset_id: ObjectId },
    #[error("embedded asset blob `{asset_id}` is duplicated")]
    DuplicateEmbeddedAssetBlob { asset_id: ObjectId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AssetSource, CanvasSettings, DocumentSettings, HistoryState, WorkspaceMetadata,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn saves_and_loads_current_package() {
        let package = package_with_embedded_asset();
        let mut buffer = Vec::new();

        save_package_to_writer(&package, &mut buffer).expect("save package");
        let loaded = load_package_from_reader(buffer.as_slice()).expect("load package");

        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.package, package);
    }

    #[test]
    fn embedded_assets_must_have_blob_storage() {
        let package = WorkspacePackage::new(workspace_with_asset(AssetSource::Embedded {
            digest: Some("sha256:logo".to_owned()),
        }));

        let error = package
            .validate()
            .expect_err("embedded asset without blob should fail");

        assert!(matches!(
            error,
            PackageValidationError::Package { issues }
                if issues.iter().any(|issue| matches!(
                    issue,
                    PackageValidationIssue::EmbeddedAssetWithoutBlob { .. }
                ))
        ));
    }

    #[test]
    fn legacy_v0_package_migrates_to_current_version() {
        let workspace = Workspace::empty(id("workspace"));
        let legacy = serde_json::json!({
            "workspace": workspace
        });

        let loaded = load_package_from_str(&legacy.to_string()).expect("load legacy package");

        assert_eq!(
            loaded.package.file_format_version,
            CURRENT_FLECK_FILE_FORMAT_VERSION
        );
        assert_eq!(
            loaded.package.workspace.format_version,
            CURRENT_WORKSPACE_FORMAT_VERSION
        );
        assert_eq!(
            loaded.warnings,
            vec![LoadWarning::MigratedFileFormat {
                from: 0,
                to: CURRENT_FLECK_FILE_FORMAT_VERSION
            }]
        );
    }

    #[test]
    fn newer_versions_load_with_warning_when_shape_is_known() {
        let mut package = WorkspacePackage::new(Workspace::empty(id("workspace")));
        package.file_format_version = CURRENT_FLECK_FILE_FORMAT_VERSION + 1;
        package.workspace.format_version = CURRENT_WORKSPACE_FORMAT_VERSION + 1;

        let json = serde_json::to_string(&package).expect("serialize package");
        let loaded = load_package_from_str(&json).expect("load newer package");

        assert!(loaded.warnings.contains(&LoadWarning::NewerFileFormat {
            found: CURRENT_FLECK_FILE_FORMAT_VERSION + 1,
            supported: CURRENT_FLECK_FILE_FORMAT_VERSION
        }));
        assert!(loaded
            .warnings
            .contains(&LoadWarning::NewerWorkspaceFormat {
                found: CURRENT_WORKSPACE_FORMAT_VERSION + 1,
                supported: CURRENT_WORKSPACE_FORMAT_VERSION
            }));
    }

    #[test]
    fn missing_linked_assets_report_relink_metadata() {
        let package = WorkspacePackage::new(workspace_with_asset(AssetSource::Linked {
            path: "missing/logo.png".to_owned(),
        }));
        let dir = unique_temp_dir();

        let missing = package.missing_linked_assets(&dir);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].asset_id, id("asset-logo"));
        assert_eq!(missing[0].name, "logo.png");
        assert!(missing[0].resolved_path.ends_with("missing/logo.png"));
    }

    #[test]
    fn save_and_load_package_from_path() {
        let package = package_with_embedded_asset();
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("workspace.fleck");

        save_package_to_path(&package, &path).expect("save package to path");
        let loaded = load_package_from_path(&path).expect("load package from path");

        assert_eq!(loaded.package, package);
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(dir);
    }

    fn package_with_embedded_asset() -> WorkspacePackage {
        let mut package = WorkspacePackage::new(workspace_with_asset(AssetSource::Embedded {
            digest: Some("sha256:logo".to_owned()),
        }));
        package.embedded_assets.push(EmbeddedAssetBlob {
            asset_id: id("asset-logo"),
            digest: Some("sha256:logo".to_owned()),
            bytes: vec![137, 80, 78, 71],
        });
        package
    }

    fn workspace_with_asset(source: AssetSource) -> Workspace {
        Workspace {
            format_version: CURRENT_WORKSPACE_FORMAT_VERSION,
            id: id("workspace"),
            metadata: WorkspaceMetadata::default(),
            canvas: CanvasSettings::default(),
            layers: Vec::new(),
            image_objects: Vec::new(),
            selections: Vec::new(),
            guides: Vec::new(),
            export_areas: Vec::new(),
            outputs: Vec::new(),
            recipes: Vec::new(),
            assets: vec![Asset {
                id: id("asset-logo"),
                name: "logo.png".to_owned(),
                source,
                media_type: Some("image/png".to_owned()),
                color_profile: None,
                image_metadata: None,
            }],
            object_groups: Vec::new(),
            history: HistoryState::default(),
            document_settings: DocumentSettings::default(),
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_millis();
        std::env::temp_dir().join(format!("fleck-test-{millis}"))
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
