//! Configuration migration system
//!
//! Handles versioned config migrations to ensure backward compatibility.

use crate::PackError;
use serde_json::Value;
use std::collections::HashMap;

/// Current config version
pub const CURRENT_VERSION: u32 = 2;

/// Config migration result
pub type MigrationResult<T> = Result<T, PackError>;

/// Config migrator
pub struct ConfigMigrator {
    migrations: HashMap<u32, Box<dyn Fn(Value) -> MigrationResult<Value> + Send + Sync>>,
}

impl ConfigMigrator {
    /// Create migrator with all registered migrations
    pub fn new() -> Self {
        let mut migrator = Self {
            migrations: HashMap::new(),
        };
        migrator.register(1, migrate_v1_to_v2);
        migrator
    }

    /// Register a migration
    pub fn register<F>(&mut self, from_version: u32, migration: F)
    where
        F: Fn(Value) -> MigrationResult<Value> + Send + Sync + 'static,
    {
        self.migrations.insert(from_version, Box::new(migration));
    }

    /// Migrate config to current version
    pub fn migrate(&self, mut config: Value) -> MigrationResult<Value> {
        let version = config.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

        if version > CURRENT_VERSION {
            return Err(PackError::Config(format!(
                "Config version {} is newer than supported version {}",
                version, CURRENT_VERSION
            )));
        }

        let mut current = version;
        while current < CURRENT_VERSION {
            if let Some(migration) = self.migrations.get(&current) {
                tracing::info!("Migrating config from v{} to v{}", current, current + 1);
                config = migration(config)?;
            }
            current += 1;
        }

        // Update version
        if let Some(obj) = config.as_object_mut() {
            obj.insert("version".into(), CURRENT_VERSION.into());
        }

        Ok(config)
    }

    /// Check if migration is needed
    pub fn needs_migration(&self, config: &Value) -> bool {
        let version = config.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        version < CURRENT_VERSION
    }
}

impl Default for ConfigMigrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Migrate v1 to v2: Restructure frontend/backend config
fn migrate_v1_to_v2(mut config: Value) -> MigrationResult<Value> {
    let obj = config
        .as_object_mut()
        .ok_or_else(|| PackError::Config("Config must be an object".into()))?;

    // Migrate [package] -> [app]
    if let Some(pkg) = obj.remove("package") {
        let mut app = serde_json::Map::new();
        if let Some(name) = pkg.get("name") {
            app.insert("name".into(), name.clone());
        }
        if let Some(ver) = pkg.get("version") {
            app.insert("version".into(), ver.clone());
        }
        if let Some(title) = pkg.get("title") {
            app.insert("name".into(), title.clone());
        }
        if let Some(desc) = pkg.get("description") {
            app.insert("description".into(), desc.clone());
        }
        if let Some(author) = pkg.get("author") {
            app.insert("author".into(), author.clone());
        }
        obj.insert("app".into(), Value::Object(app));
    }

    // Migrate [frontend] with url/path fields
    if let Some(fe) = obj.get("frontend").cloned() {
        if fe.get("url").is_some() || fe.get("path").is_some() {
            // Already new format
        } else if let Some(url) = fe.as_str() {
            obj.insert("frontend".into(), serde_json::json!({ "url": url }));
        }
    }

    // Migrate [backend.python] -> [backend] with type
    if let Some(be) = obj.remove("backend") {
        if let Some(py) = be.get("python") {
            let mut new_be = py.clone();
            if let Some(obj) = new_be.as_object_mut() {
                obj.insert("type".into(), "python".into());
            }
            obj.insert("backend".into(), new_be);
        }
    }

    // Migrate [bundle] -> [target] + platform fields
    if let Some(bundle) = obj.remove("bundle") {
        if let Some(icon) = bundle.get("icon") {
            if let Some(app) = obj.get_mut("app").and_then(|a| a.as_object_mut()) {
                app.insert("icon".into(), icon.clone());
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1 = serde_json::json!({
            "version": 1,
            "package": { "name": "test", "version": "1.0.0" },
            "frontend": { "path": "./dist" }
        });

        let migrator = ConfigMigrator::new();
        let v2 = migrator.migrate(v1).unwrap();

        assert_eq!(v2["version"], CURRENT_VERSION);
        assert!(v2.get("app").is_some());
    }

    #[test]
    fn test_no_migration_needed() {
        let current = serde_json::json!({ "version": CURRENT_VERSION });
        let migrator = ConfigMigrator::new();
        assert!(!migrator.needs_migration(&current));
    }
}
