//! File system operations implementation

use super::types::{DirEntry, FileStat};
use crate::{PathScope, PluginError, PluginResult};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Read a file as text
pub fn read_file(path: &str, encoding: Option<&str>, scope: &PathScope) -> PluginResult<String> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if !canonical.exists() {
        return Err(PluginError::file_not_found(path));
    }

    let content = fs::read_to_string(&canonical).map_err(PluginError::io_error)?;

    // Handle encoding (currently only UTF-8 is supported)
    if let Some(enc) = encoding {
        if enc.to_lowercase() != "utf-8" && enc.to_lowercase() != "utf8" {
            tracing::warn!("Encoding '{}' not supported, using UTF-8", enc);
        }
    }

    Ok(content)
}

/// Read a file as binary (returns base64-encoded string)
pub fn read_file_binary(path: &str, scope: &PathScope) -> PluginResult<String> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if !canonical.exists() {
        return Err(PluginError::file_not_found(path));
    }

    let mut file = fs::File::open(&canonical).map_err(PluginError::io_error)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(PluginError::io_error)?;

    // Return as base64
    use base64::{engine::general_purpose::STANDARD, Engine};
    Ok(STANDARD.encode(&buffer))
}

/// Write text to a file
pub fn write_file(path: &str, contents: &str, append: bool, scope: &PathScope) -> PluginResult<()> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    // Ensure parent directory exists
    if let Some(parent) = canonical.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(PluginError::io_error)?;
        }
    }

    let mut file = if append {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&canonical)
            .map_err(PluginError::io_error)?
    } else {
        fs::File::create(&canonical).map_err(PluginError::io_error)?
    };

    file.write_all(contents.as_bytes())
        .map_err(PluginError::io_error)?;

    Ok(())
}

/// Write binary data to a file
pub fn write_file_binary(
    path: &str,
    contents: &[u8],
    append: bool,
    scope: &PathScope,
) -> PluginResult<()> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    // Ensure parent directory exists
    if let Some(parent) = canonical.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(PluginError::io_error)?;
        }
    }

    let mut file = if append {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&canonical)
            .map_err(PluginError::io_error)?
    } else {
        fs::File::create(&canonical).map_err(PluginError::io_error)?
    };

    file.write_all(contents).map_err(PluginError::io_error)?;

    Ok(())
}

/// Read directory contents
pub fn read_dir(path: &str, recursive: bool, scope: &PathScope) -> PluginResult<Vec<DirEntry>> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if !canonical.exists() {
        return Err(PluginError::file_not_found(path));
    }

    let mut entries = Vec::new();
    read_dir_impl(&canonical, recursive, scope, &mut entries)?;

    Ok(entries)
}

fn read_dir_impl(
    path: &Path,
    recursive: bool,
    scope: &PathScope,
    entries: &mut Vec<DirEntry>,
) -> PluginResult<()> {
    let dir = fs::read_dir(path).map_err(PluginError::io_error)?;

    for entry in dir {
        let entry = entry.map_err(PluginError::io_error)?;
        let entry_path = entry.path();
        let metadata = entry.metadata().map_err(PluginError::io_error)?;

        let dir_entry = DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry_path.to_string_lossy().to_string(),
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.file_type().is_symlink(),
        };

        entries.push(dir_entry);

        if recursive && metadata.is_dir() {
            // Check scope for subdirectory
            if scope.is_allowed(&entry_path).is_ok() {
                read_dir_impl(&entry_path, recursive, scope, entries)?;
            }
        }
    }

    Ok(())
}

/// Create a directory
pub fn create_dir(path: &str, recursive: bool, scope: &PathScope) -> PluginResult<()> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if recursive {
        fs::create_dir_all(&canonical).map_err(PluginError::io_error)?;
    } else {
        fs::create_dir(&canonical).map_err(PluginError::io_error)?;
    }

    Ok(())
}

/// Remove a file or directory
pub fn remove(path: &str, recursive: bool, scope: &PathScope) -> PluginResult<()> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if !canonical.exists() {
        return Err(PluginError::file_not_found(path));
    }

    if canonical.is_dir() {
        if recursive {
            fs::remove_dir_all(&canonical).map_err(PluginError::io_error)?;
        } else {
            fs::remove_dir(&canonical).map_err(PluginError::io_error)?;
        }
    } else {
        fs::remove_file(&canonical).map_err(PluginError::io_error)?;
    }

    Ok(())
}

/// Copy a file or directory
pub fn copy(from: &str, to: &str, scope: &PathScope) -> PluginResult<()> {
    let from_canonical = scope
        .is_allowed(from)
        .map_err(|_| PluginError::scope_violation(from))?;
    let to_canonical = scope
        .is_allowed(to)
        .map_err(|_| PluginError::scope_violation(to))?;

    if !from_canonical.exists() {
        return Err(PluginError::file_not_found(from));
    }

    if from_canonical.is_dir() {
        copy_dir_all(&from_canonical, &to_canonical)?;
    } else {
        // Ensure parent directory exists
        if let Some(parent) = to_canonical.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(PluginError::io_error)?;
            }
        }
        fs::copy(&from_canonical, &to_canonical).map_err(PluginError::io_error)?;
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> PluginResult<()> {
    fs::create_dir_all(dst).map_err(PluginError::io_error)?;

    for entry in fs::read_dir(src).map_err(PluginError::io_error)? {
        let entry = entry.map_err(PluginError::io_error)?;
        let file_type = entry.file_type().map_err(PluginError::io_error)?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path).map_err(PluginError::io_error)?;
        }
    }

    Ok(())
}

/// Rename/move a file or directory
pub fn rename(from: &str, to: &str, scope: &PathScope) -> PluginResult<()> {
    let from_canonical = scope
        .is_allowed(from)
        .map_err(|_| PluginError::scope_violation(from))?;
    let to_canonical = scope
        .is_allowed(to)
        .map_err(|_| PluginError::scope_violation(to))?;

    if !from_canonical.exists() {
        return Err(PluginError::file_not_found(from));
    }

    // Ensure parent directory exists
    if let Some(parent) = to_canonical.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(PluginError::io_error)?;
        }
    }

    fs::rename(&from_canonical, &to_canonical).map_err(PluginError::io_error)?;

    Ok(())
}

/// Check if a path exists
pub fn exists(path: &str, scope: &PathScope) -> PluginResult<bool> {
    let result = scope.is_allowed(path);
    match result {
        Ok(canonical) => Ok(canonical.exists()),
        Err(_) => Ok(false), // Path not in scope, treat as non-existent
    }
}

/// Get file/directory statistics
pub fn stat(path: &str, scope: &PathScope) -> PluginResult<FileStat> {
    let canonical = scope
        .is_allowed(path)
        .map_err(|_| PluginError::scope_violation(path))?;

    if !canonical.exists() {
        return Err(PluginError::file_not_found(path));
    }

    let metadata = fs::metadata(&canonical).map_err(PluginError::io_error)?;

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    let created_at = metadata
        .created()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    let accessed_at = metadata
        .accessed()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    Ok(FileStat {
        is_directory: metadata.is_dir(),
        is_file: metadata.is_file(),
        is_symlink: metadata.file_type().is_symlink(),
        size: metadata.len(),
        modified_at,
        created_at,
        accessed_at,
        readonly: metadata.permissions().readonly(),
    })
}
