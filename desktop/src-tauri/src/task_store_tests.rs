use crate::task_store::{CreateCommandTaskRequest, TaskStore};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use uuid::Uuid;

fn temp_runtime() -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("sam-command-center-test-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let db = root.join("runtime.sqlite3");
    let shared = root.join(".sam-runtime/shared_state.json");
    (root, db, shared)
}

fn request(title: &str, dependencies: Vec<String>) -> CreateCommandTaskRequest {
    CreateCommandTaskRequest {
        title: title.into(),
        description: format!("Test task {title}"),
        owner: "travis".into(),
        execution_mode: "AGENT".into(),
        milestone: Some("TEST".into()),
        priority: 3,
        dependency_ids: dependencies,
    }
}

#[test]
fn completing_dependency_auto_unblocks_next_task_and_projects_shared_state() {
    let (root, db, shared) = temp_runtime();
    let store = TaskStore::new_with_shared_state(&db, &shared).unwrap();

    let first = store.create(&request("First", vec![])).unwrap();
    let second = store.create(&request("Second", vec![first.task_id.clone()])).unwrap();
    assert_eq!(second.status, "BLOCKED");
    assert!(second.blocked_reason.as_deref().unwrap_or("").contains(&first.task_id));

    store.begin_preparation(&first.task_id, "sam").unwrap();
    store.finalize_preparation(
        &first.task_id,
        "# Prepared test task",
        &json!({"can_start": true, "prompt_markdown": "# Prepared test task"}),
        "orchestrator",
    ).unwrap();
    store.start(&first.task_id, "sam").unwrap();
    store.send_to_review(&first.task_id, "validator").unwrap();
    store.complete(&first.task_id, "sam").unwrap();

    assert_eq!(store.get(&first.task_id).unwrap().status, "DONE");
    assert_eq!(store.get(&second.task_id).unwrap().status, "TODO");

    let projection: Value = serde_json::from_slice(&fs::read(&shared).unwrap()).unwrap();
    assert_eq!(projection.get("source_of_truth").and_then(Value::as_str), Some("SQLITE"));
    let tasks = projection.get("tasks").and_then(Value::as_array).unwrap();
    let projected_first = tasks.iter().find(|row| row.get("taskId").and_then(Value::as_str) == Some(&first.task_id)).unwrap();
    let projected_second = tasks.iter().find(|row| row.get("taskId").and_then(Value::as_str) == Some(&second.task_id)).unwrap();
    assert_eq!(projected_first.get("status").and_then(Value::as_str), Some("DONE"));
    assert_eq!(projected_second.get("status").and_then(Value::as_str), Some("TODO"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn task_cannot_start_before_prepare_finishes() {
    let (root, db, shared) = temp_runtime();
    let store = TaskStore::new_with_shared_state(&db, &shared).unwrap();
    let task = store.create(&request("Needs prepare", vec![])).unwrap();

    let error = store.start(&task.task_id, "sam").unwrap_err();
    assert!(error.contains("Prepare the task before starting it"));
    assert_eq!(store.get(&task.task_id).unwrap().status, "TODO");

    fs::remove_dir_all(root).unwrap();
}
