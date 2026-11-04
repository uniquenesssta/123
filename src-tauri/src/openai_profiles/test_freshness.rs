use super::*;

fn fixture() -> (tempfile::TempDir, OpenAiProfileStore, Uuid, DateTime<Utc>) {
    let directory = tempfile::tempdir().expect("task temporary directory");
    let store = OpenAiProfileStore::new(directory.path().join("profiles.json"));
    let (id, revision) = {
        let state = store.lock().expect("store state");
        let profile = state.profiles.first().expect("default profile");
        (profile.id, profile.updated_at)
    };
    (directory, store, id, revision)
}

#[test]
fn acknowledged_test_commits_only_its_original_snapshot() {
    let (_directory, store, id, revision) = fixture();
    let tested_at = revision + chrono::Duration::seconds(1);
    store
        .update_test_state(
            id,
            revision,
            "success",
            Some("minimal request passed"),
            tested_at,
        )
        .expect("matching snapshot");
    let state = store.lock().expect("read state");
    assert_eq!(state.profiles[0].last_test_status, "success");
    assert_eq!(state.profiles[0].last_tested_at, Some(tested_at));
}

#[test]
fn changed_settings_or_key_cannot_receive_old_test_success() {
    let (_directory, store, id, revision) = fixture();
    {
        let mut state = store.lock().expect("edit metadata");
        state.profiles[0].research_model = "replacement-model".to_string();
        state.profiles[0].updated_at = revision + chrono::Duration::seconds(1);
    }
    assert!(store
        .update_test_state(id, revision, "success", Some("old response"), Utc::now())
        .is_err());
    assert_eq!(
        store.lock().expect("unchanged result").profiles[0].last_test_status,
        "untested"
    );
}

#[test]
fn late_failed_test_cannot_overwrite_newer_committed_result() {
    let (_directory, store, id, revision) = fixture();
    store
        .update_test_state(
            id,
            revision,
            "success",
            None,
            revision + chrono::Duration::seconds(1),
        )
        .expect("first result");
    assert!(store
        .update_test_state(id, revision, "failed", Some("late failure"), Utc::now())
        .is_err());
    assert_eq!(
        store.lock().expect("result remains").profiles[0].last_test_status,
        "success"
    );
}

#[test]
fn removed_profile_is_never_recreated_by_test_completion() {
    let (_directory, store, id, revision) = fixture();
    store.lock().expect("remove original").profiles.clear();
    assert!(store
        .update_test_state(id, revision, "success", None, Utc::now())
        .is_err());
    assert!(store.lock().expect("no resurrection").profiles.is_empty());
}
