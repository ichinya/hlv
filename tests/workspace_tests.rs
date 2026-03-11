use std::path::Path;
use tempfile::TempDir;

/// Create a minimal HLV project at `root` so workspace validation passes.
fn setup_project(root: &Path) {
    hlv::cmd::init::run_with_milestone(
        root.to_str().unwrap(),
        Some("test-proj"),
        Some("team"),
        Some("claude"),
        Some("ws-test"),
        Some("minimal"),
    )
    .unwrap();
}

/// Read workspace YAML back as WorkspaceConfig.
fn load_ws(path: &Path) -> hlv::mcp::workspace::WorkspaceConfig {
    hlv::mcp::workspace::WorkspaceConfig::load_lenient(path).unwrap()
}

#[test]
fn init_creates_file() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();

    assert!(ws.exists());
    let config = load_ws(&ws);
    assert!(config.projects.is_empty());
}

#[test]
fn init_idempotent() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();
    // Second call should not fail (warns "already exists")
    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();

    let config = load_ws(&ws);
    assert!(config.projects.is_empty());
}

#[test]
fn add_with_explicit_id_and_root() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    let proj = dir.path().join("myproj");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    hlv::cmd::workspace::run_add(
        Some("myproj"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    let config = load_ws(&ws);
    assert_eq!(config.projects.len(), 1);
    assert_eq!(config.projects[0].id, "myproj");
}

#[test]
fn add_derives_id_from_dir_name() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    let proj = dir.path().join("cool-project");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    hlv::cmd::workspace::run_add(
        None, // no explicit ID
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    let config = load_ws(&ws);
    assert_eq!(config.projects[0].id, "cool-project");
}

#[test]
fn add_auto_creates_workspace_file() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("sub").join("workspace.yaml");
    let proj = dir.path().join("proj");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    // No init — add should auto-create
    hlv::cmd::workspace::run_add(
        Some("proj"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    assert!(ws.exists());
    let config = load_ws(&ws);
    assert_eq!(config.projects.len(), 1);
}

#[test]
fn add_rejects_missing_project_yaml() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    let proj = dir.path().join("empty");
    std::fs::create_dir(&proj).unwrap();
    // No setup_project — no project.yaml

    let err = hlv::cmd::workspace::run_add(
        Some("empty"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    );

    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("project.yaml"));
}

#[test]
fn add_rejects_duplicate_id() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    let proj1 = dir.path().join("proj1");
    std::fs::create_dir(&proj1).unwrap();
    setup_project(&proj1);

    let proj2 = dir.path().join("proj2");
    std::fs::create_dir(&proj2).unwrap();
    setup_project(&proj2);

    hlv::cmd::workspace::run_add(
        Some("same-id"),
        Some(proj1.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    let err = hlv::cmd::workspace::run_add(
        Some("same-id"),
        Some(proj2.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    );

    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("already in workspace"));
}

#[test]
fn add_rejects_duplicate_root() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    let proj = dir.path().join("proj");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    hlv::cmd::workspace::run_add(
        Some("id1"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    let err = hlv::cmd::workspace::run_add(
        Some("id2"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    );

    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("already in workspace"));
}

#[test]
fn add_multiple_projects() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    for name in &["alpha", "beta", "gamma"] {
        let proj = dir.path().join(name);
        std::fs::create_dir(&proj).unwrap();
        setup_project(&proj);
        hlv::cmd::workspace::run_add(
            Some(name),
            Some(proj.to_str().unwrap()),
            Some(ws.to_str().unwrap()),
        )
        .unwrap();
    }

    let config = load_ws(&ws);
    assert_eq!(config.projects.len(), 3);
    assert!(config.find("alpha").is_some());
    assert!(config.find("beta").is_some());
    assert!(config.find("gamma").is_some());
}

#[test]
fn remove_existing_project() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    let proj = dir.path().join("proj");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    hlv::cmd::workspace::run_add(
        Some("proj"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    hlv::cmd::workspace::run_remove("proj", Some(ws.to_str().unwrap())).unwrap();

    let config = load_ws(&ws);
    assert!(config.projects.is_empty());
}

#[test]
fn remove_nonexistent_project_fails() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();

    let err = hlv::cmd::workspace::run_remove("ghost", Some(ws.to_str().unwrap()));
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("not found"));
}

#[test]
fn remove_leaves_other_projects() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    for name in &["a", "b", "c"] {
        let proj = dir.path().join(name);
        std::fs::create_dir(&proj).unwrap();
        setup_project(&proj);
        hlv::cmd::workspace::run_add(
            Some(name),
            Some(proj.to_str().unwrap()),
            Some(ws.to_str().unwrap()),
        )
        .unwrap();
    }

    hlv::cmd::workspace::run_remove("b", Some(ws.to_str().unwrap())).unwrap();

    let config = load_ws(&ws);
    assert_eq!(config.projects.len(), 2);
    assert!(config.find("a").is_some());
    assert!(config.find("b").is_none());
    assert!(config.find("c").is_some());
}

#[test]
fn list_no_workspace_file() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("nonexistent.yaml");

    // Should not fail, just warn
    hlv::cmd::workspace::run_list(Some(ws.to_str().unwrap())).unwrap();
}

#[test]
fn list_empty_workspace() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");
    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();

    // Should not fail
    hlv::cmd::workspace::run_list(Some(ws.to_str().unwrap())).unwrap();
}

#[test]
fn list_with_projects() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    let proj = dir.path().join("proj");
    std::fs::create_dir(&proj).unwrap();
    setup_project(&proj);

    hlv::cmd::workspace::run_add(
        Some("proj"),
        Some(proj.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();

    // Should not fail
    hlv::cmd::workspace::run_list(Some(ws.to_str().unwrap())).unwrap();
}

#[test]
fn full_lifecycle() {
    let dir = TempDir::new().unwrap();
    let ws = dir.path().join("workspace.yaml");

    // init
    hlv::cmd::workspace::run_init(Some(ws.to_str().unwrap())).unwrap();
    assert_eq!(load_ws(&ws).projects.len(), 0);

    // add two projects
    let p1 = dir.path().join("p1");
    let p2 = dir.path().join("p2");
    std::fs::create_dir(&p1).unwrap();
    std::fs::create_dir(&p2).unwrap();
    setup_project(&p1);
    setup_project(&p2);

    hlv::cmd::workspace::run_add(
        Some("p1"),
        Some(p1.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();
    hlv::cmd::workspace::run_add(
        Some("p2"),
        Some(p2.to_str().unwrap()),
        Some(ws.to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(load_ws(&ws).projects.len(), 2);

    // remove one
    hlv::cmd::workspace::run_remove("p1", Some(ws.to_str().unwrap())).unwrap();
    let config = load_ws(&ws);
    assert_eq!(config.projects.len(), 1);
    assert_eq!(config.projects[0].id, "p2");

    // list (should not fail)
    hlv::cmd::workspace::run_list(Some(ws.to_str().unwrap())).unwrap();
}
