//! Save caches survive a filename-driven rename. Display caches are never warmed for a save
//! nor certified for placements that do not match the document being saved; the model cache
//! (the timeline's bodies, before placement) needs no warm display.
use serde_json::json;
use solidcraft_doc::Document;
use solidcraft_engine::Session;

fn assembly() -> Session {
    let mut session = Session::new(Document::new("Untitled"));
    session.execute("component.create", &json!({"name":"PlacedPart"})).unwrap();
    session.execute("solid.box", &json!({"length":2,"width":3,"height":4})).unwrap();
    session
}

fn warm(session: &Session) {
    for body in &session.world_state().bodies {
        body.mesh();
    }
}

fn save(session: &mut Session, label: &str) -> Vec<u8> {
    let path = std::env::temp_dir().join(format!("solidcraft-save-as-{label}-{}.solidcraft", std::process::id()));
    session.execute("file.save_as", &json!({"path":path,"cache":true,"preview":true})).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(solidcraft_io::read_design(&bytes).unwrap().name, path.file_stem().unwrap().to_string_lossy());
    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_file(format!("{}.bak", path.display()));
    bytes
}

/// Whether the file holds saved display meshes (a picture is also drawn from the model cache).
fn saved_display(bytes: &[u8]) -> bool {
    bytes.windows(b"cache/preview.bin".len()).any(|entry| entry == b"cache/preview.bin")
}

#[test]
fn a_filename_rename_preserves_opted_in_warm_assembly_caches() {
    let mut session = assembly();
    warm(&session);
    for label in ["first-warm", "renamed-warm"] {
        let bytes = save(&mut session, label);
        let geometry = solidcraft_io::read_design_geometry(&bytes).expect("warm geometry survives the rename");
        assert_eq!(geometry.bodies.len(), 1);
        assert!((solidcraft_kernel::measure(&geometry.bodies[0].body).unwrap().volume - 24.0).abs() < 1e-9);
        assert!(solidcraft_io::read_design_preview(&bytes).is_some(), "warm display meshes survive the rename");
        warm(&session);
    }
}

#[test]
fn naming_a_design_does_not_warm_cold_or_revalidate_stale_placements() {
    let original = assembly();
    let mut cold = Session::new((*original.doc).clone());
    let bytes = save(&mut cold, "cold");
    assert!(solidcraft_io::read_design_geometry(&bytes).is_some(), "the model needs no warm display");
    assert!(!saved_display(&bytes));

    let mut stale = assembly();
    warm(&stale);
    stale.doc_mut().occurrences.first_mut().unwrap().transform[3][0] = 25.0;
    let bytes = save(&mut stale, "stale");
    assert!(solidcraft_io::read_design_geometry(&bytes).is_some(), "a placement is not part of the timeline's bodies");
    assert!(!saved_display(&bytes), "old placements must not be certified for a changed document");

    // A document edited without an evaluation since is not saved with the old one's bodies.
    let mut edited = assembly();
    let feature = edited.doc.features.last().unwrap().id;
    if let solidcraft_doc::FeatureKind::Box { length, .. } = &mut edited.doc_mut().features.iter_mut().find(|f| f.id == feature).unwrap().kind {
        *length = "20".into();
    }
    let bytes = save(&mut edited, "edited");
    assert!(solidcraft_io::read_design_geometry(&bytes).is_none(), "bodies of another document must not be saved as this one's");
}

#[cfg(any(feature = "brand-hasisolid", feature = "brand-mssolid"))]
#[test]
fn branded_sheet_rules_preserve_legacy_files_and_roundtrip() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/legacy-027-sheet");
    for (name, thickness) in [("implicit-027.sc", "2.5 mm"), ("custom-027.sc", "4 mm")] {
        let mut session = Session::default();
        session.execute("doc.open", &json!({"path":fixtures.join(name)})).unwrap();
        assert_eq!(session.doc.sheet_rule(None).thickness, thickness, "{name}");
        assert_eq!(session.world_state().bodies.len(), 1, "{name}");
    }
    let path = std::env::temp_dir().join(format!("hasisolid-sheet-roundtrip-{}.sc", std::process::id()));
    assert!(!path.exists(), "test target already exists");
    let mut session = Session::default();
    assert_eq!(session.doc.sheet_rule(None).thickness, "2 mm");
    assert!(session.execute("sheet.manage_rules", &json!({"name":"Steel (mm)","delete":true})).is_err());
    assert_eq!(session.doc.sheet_rule(None).thickness, "2 mm", "deleting the last rule must not silently restore 2.5 mm");
    session.execute("sheet.manage_rules", &json!({"name":"Steel (mm)","thickness":"3 mm","active":true})).unwrap();
    session.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(session.doc.sheet_rule(None).thickness, "2 mm");
    session.execute("file.save_as", &json!({"path":path})).unwrap();
    let mut reopened = Session::default();
    reopened.execute("doc.open", &json!({"path":path})).unwrap();
    assert_eq!(reopened.doc.sheet_rule(None).thickness, "2 mm");
    drop(reopened);
    drop(session);
    std::fs::remove_file(path).unwrap();
}
