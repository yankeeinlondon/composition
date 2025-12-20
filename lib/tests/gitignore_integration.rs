//! Integration tests for gitignore filtering
//!
//! These tests verify that sensitive files are properly blocked
//! from transclusion, including security-critical scenarios.

use lib::error::CompositionError;
use lib::graph::utils::load_resource;
use lib::types::Resource;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test git project with .gitignore
fn create_git_project() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    // Create .git directory
    fs::create_dir(project_root.join(".git")).unwrap();

    // Create .gitignore with security-sensitive patterns
    let gitignore_content = r#"
# Secrets and credentials
.env
.env.*
*.secret
*.key
credentials.json
config/secrets.yml

# Dependencies
node_modules/
vendor/
target/

# Build output
dist/
build/
*.log

# OS files
.DS_Store
Thumbs.db
"#;
    fs::write(project_root.join(".gitignore"), gitignore_content).unwrap();

    (temp_dir, project_root)
}

#[tokio::test]
async fn test_env_file_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create .env file with sensitive content
    let env_file = root.join(".env");
    fs::write(&env_file, "DATABASE_PASSWORD=super_secret_123").unwrap();

    // Attempt to load .env file
    let resource = Resource::local(env_file.clone());
    let result = load_resource(&resource).await;

    // Should be rejected with FileIgnored error
    assert!(result.is_err());
    match result.unwrap_err() {
        CompositionError::Parse(parse_err) => {
            let err_msg = parse_err.to_string();
            assert!(
                err_msg.contains("ignored") || err_msg.contains(".env"),
                "Expected FileIgnored error, got: {}",
                err_msg
            );
        }
        other => panic!("Expected Parse error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_node_modules_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create node_modules directory and file
    let node_modules = root.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    let package_json = node_modules.join("package.json");
    fs::write(&package_json, r#"{"name": "malicious-package"}"#).unwrap();

    // Attempt to load file from node_modules
    let resource = Resource::local(package_json);
    let result = load_resource(&resource).await;

    // Should be rejected
    assert!(result.is_err());
}

#[tokio::test]
async fn test_credentials_json_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create credentials.json with API keys
    let creds_file = root.join("credentials.json");
    fs::write(
        &creds_file,
        r#"{"api_key": "sk-1234567890", "secret": "abc123"}"#,
    )
    .unwrap();

    // Attempt to load credentials
    let resource = Resource::local(creds_file);
    let result = load_resource(&resource).await;

    // Should be rejected
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wildcard_secret_files_are_blocked() {
    let (_temp, root) = create_git_project();

    // Create various .secret files
    let files = vec!["api.secret", "database.secret", "oauth.secret"];

    for filename in files {
        let secret_file = root.join(filename);
        fs::write(&secret_file, "secret_content").unwrap();

        let resource = Resource::local(secret_file);
        let result = load_resource(&resource).await;

        assert!(
            result.is_err(),
            "File {} should be blocked",
            filename
        );
    }
}

#[tokio::test]
async fn test_normal_markdown_is_allowed() {
    let (_temp, root) = create_git_project();

    // Create normal markdown file
    let readme = root.join("README.md");
    fs::write(&readme, "# My Project\n\nThis is a normal file.").unwrap();

    // Should be allowed
    let resource = Resource::local(readme);
    let result = load_resource(&resource).await;

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("My Project"));
}

#[tokio::test]
async fn test_nested_gitignore_patterns() {
    let (_temp, root) = create_git_project();

    // Create config directory at root level (to match gitignore pattern)
    let config_dir = root.join("config");
    fs::create_dir(&config_dir).unwrap();

    // Create secrets.yml in config/
    let secrets_file = config_dir.join("secrets.yml");
    fs::write(&secrets_file, "password: secret123").unwrap();

    // Should be blocked (matches config/secrets.yml pattern)
    let resource = Resource::local(secrets_file);
    let result = load_resource(&resource).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_log_files_are_blocked() {
    let (_temp, root) = create_git_project();

    // Create various log files
    let log_files = vec!["app.log", "debug.log", "error.log"];

    for filename in log_files {
        let log_file = root.join(filename);
        fs::write(&log_file, "Log content here").unwrap();

        let resource = Resource::local(log_file);
        let result = load_resource(&resource).await;

        assert!(result.is_err(), "Log file {} should be blocked", filename);
    }
}

#[tokio::test]
async fn test_dist_directory_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create dist directory
    let dist_dir = root.join("dist");
    fs::create_dir(&dist_dir).unwrap();

    let bundle_file = dist_dir.join("bundle.js");
    fs::write(&bundle_file, "compiled code").unwrap();

    // Should be blocked
    let resource = Resource::local(bundle_file);
    let result = load_resource(&resource).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_target_directory_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create target directory (Rust build output)
    let target_dir = root.join("target");
    fs::create_dir(&target_dir).unwrap();

    let debug_dir = target_dir.join("debug");
    fs::create_dir(&debug_dir).unwrap();

    let binary = debug_dir.join("myapp");
    fs::write(&binary, "binary content").unwrap();

    // Should be blocked
    let resource = Resource::local(binary);
    let result = load_resource(&resource).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_env_variant_files_are_blocked() {
    let (_temp, root) = create_git_project();

    // Test various .env.* patterns
    let env_variants = vec![".env.local", ".env.development", ".env.production"];

    for filename in env_variants {
        let env_file = root.join(filename);
        fs::write(&env_file, "SECRET=value").unwrap();

        let resource = Resource::local(env_file);
        let result = load_resource(&resource).await;

        assert!(
            result.is_err(),
            "Env variant {} should be blocked",
            filename
        );
    }
}

#[tokio::test]
async fn test_without_git_directory() {
    // Create temp directory WITHOUT .git
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create .gitignore anyway
    fs::write(root.join(".gitignore"), ".env\n").unwrap();

    // Create .env file
    let env_file = root.join(".env");
    fs::write(&env_file, "SECRET=value").unwrap();

    // Without .git directory, gitignore filtering is not applied
    // File should be readable (no project root found)
    let resource = Resource::local(env_file);
    let result = load_resource(&resource).await;

    // Should succeed (no .git means no project root, so no filtering)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ds_store_is_blocked() {
    let (_temp, root) = create_git_project();

    // Create .DS_Store file (macOS metadata)
    let ds_store = root.join(".DS_Store");
    fs::write(&ds_store, "binary metadata").unwrap();

    // Should be blocked
    let resource = Resource::local(ds_store);
    let result = load_resource(&resource).await;

    assert!(result.is_err());
}
