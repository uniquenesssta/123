use super::*;

fn draft() -> IssueLogDraft {
    IssueLogDraft {
        severity: "error".into(),
        source: "frontend".into(),
        operation: "open profile".into(),
        user_message: "request failed".into(),
        technical_message: "same failure".into(),
        occurrence_key: None,
    }
}

#[test]
fn corrupted_history_is_not_an_empty_success() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("issues.json");
    fs::write(&path, b"{damaged").unwrap();
    let store = IssueLogStore::new(path.clone());
    assert!(store.list(500).is_err());
    assert!(store.record(draft()).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"{damaged");
}

#[test]
fn failed_clear_keeps_memory_and_persisted_history() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().join("nested");
    let moved = directory.path().join("preserved");
    assert!(parent.starts_with(directory.path()) && moved.starts_with(directory.path()));
    let store = IssueLogStore::new(parent.join("issues.json"));
    store.record(draft()).unwrap();
    fs::rename(&parent, &moved).unwrap();
    fs::write(&parent, b"block writes").unwrap();
    assert!(store.clear().is_err());
    assert_eq!(store.list(500).unwrap().len(), 1);
    let persisted: Vec<IssueLogEntry> =
        serde_json::from_slice(&fs::read(moved.join("issues.json")).unwrap()).unwrap();
    assert_eq!(persisted.len(), 1);
}

#[test]
fn issue_strings_use_credential_and_url_redaction() {
    let directory = tempfile::tempdir().unwrap();
    let store = IssueLogStore::new(directory.path().join("issues.json"));
    let mut input = draft();
    input.technical_message = "Bearer TEST-PRIVATE-KEY password=TEST-PASSWORD https://example.invalid/api?api_key=TEST-QUERY".into();
    let result = store.record(input).unwrap();
    assert!(!result.technical_message.contains("TEST-PRIVATE-KEY"));
    assert!(!result.technical_message.contains("TEST-PASSWORD"));
    assert!(!result.technical_message.contains("TEST-QUERY"));
}

#[test]
fn changed_collection_is_rejected_inside_the_storage_lock() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("issues.json");
    let store = IssueLogStore::new(path.clone());
    store.record(draft()).unwrap();
    let confirmed = store.list(500).unwrap();
    store.record(draft()).unwrap();
    assert!(store.clear_snapshot(Some(&confirmed)).is_err());
    assert!(store
        .export_snapshot(&directory.path().join("report.txt"), Some(&confirmed))
        .is_err());
    assert!(!directory.path().join("report.txt").exists());
    assert_eq!(store.list(500).unwrap()[0].occurrence_count, 2);
    let current = store.list(500).unwrap();
    store.clear_snapshot(Some(&current)).unwrap();
    assert!(store.list(500).unwrap().is_empty());
    assert!(IssueLogStore::new(path).list(500).unwrap().is_empty());
}

#[test]
fn corrupt_history_retries_only_after_the_file_is_repaired() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("issues.json");
    fs::write(&path, b"broken").unwrap();
    let store = IssueLogStore::new(path.clone());
    assert!(store.list(500).is_err());
    assert!(store.clear().is_err());
    fs::write(&path, b"[]").unwrap();
    assert!(store.list(500).unwrap().is_empty());
    store.record(draft()).unwrap();
    assert_eq!(IssueLogStore::new(path).list(500).unwrap().len(), 1);
}

#[test]
fn failed_record_never_commits_a_phantom_occurrence() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().join("nested");
    let moved = directory.path().join("retained");
    assert!(parent.starts_with(directory.path()) && moved.starts_with(directory.path()));
    let store = IssueLogStore::new(parent.join("issues.json"));
    store.record(draft()).unwrap();
    fs::rename(&parent, &moved).unwrap();
    fs::write(&parent, b"block").unwrap();
    assert!(store.record(draft()).is_err());
    assert_eq!(store.list(500).unwrap()[0].occurrence_count, 1);
}

#[test]
fn legacy_fields_are_redacted_at_read_and_export_boundaries() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("issues.json");
    let store = IssueLogStore::new(path.clone());
    let mut row = store.record(draft()).unwrap();
    row.operations = vec!["password=QA-OLD-OP".into()];
    row.user_message = "api_key=QA-OLD-KEY".into();
    row.technical_message = "https://example.invalid/api?token=QA-OLD-QUERY".into();
    row.source = "Bearer QA-OLD-SOURCE".into();
    fs::write(&path, serde_json::to_vec(&vec![row]).unwrap()).unwrap();
    let restored = IssueLogStore::new(path);
    let rows = restored.list(500).unwrap();
    let text = serde_json::to_string(&rows).unwrap();
    assert!(!text.contains("QA-OLD"));
    let output = directory.path().join("report.txt");
    restored.export_snapshot(&output, Some(&rows)).unwrap();
    assert!(!fs::read_to_string(output).unwrap().contains("QA-OLD"));
}
