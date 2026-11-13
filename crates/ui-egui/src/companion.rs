//! Export an independent STEP copy and open it in the matching desktop viewer.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::SolveApp;
use serde_json::json;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Job {
    done: Receiver<Result<Option<String>, String>>,
    viewer: String,
    output: String,
    uid: u64,
    revision: u64,
}

pub(crate) fn viewer_name() -> &'static str {
    match solidcraft_engine::product::PRODUCT.name {
        "HasiSolid" => "Hasi 3D",
        "MSSolid" => "MS3D",
        _ => "3D viewer",
    }
}

fn viewer_exe() -> Option<&'static str> {
    match solidcraft_engine::product::PRODUCT.name {
        "HasiSolid" => Some("hasi-3d.exe"),
        "MSSolid" => Some("ms3d.exe"),
        _ => None,
    }
}

fn valid_viewer(path: &str) -> bool {
    let path = Path::new(path);
    viewer_exe().is_some_and(|expected| path.is_file() && path.file_name().is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(expected)))
}

fn error(app: &mut SolveApp, message: &str) {
    let text = crate::i18n::Locale::for_language(&app.preferences.language).format("family.export_failed", &[("error", message)]);
    app.set_status(text, true);
}

fn unchanged(app: &SolveApp, uid: u64, revision: u64) -> bool {
    app.session.uid == uid && app.session.revision == revision && app.session.pending_moves.is_empty()
}

pub(crate) fn start(app: &mut SolveApp, ctx: egui::Context) {
    if app.companion_job.is_some() || app.services.find_viewer.is_none() || app.services.launch_viewer.is_none() {
        return;
    }
    if !app.session.pending_moves.is_empty() {
        app.set_status(app.tr("family.pending"), true);
        return;
    }
    if !app.session.doc.point_clouds.is_empty() {
        app.set_status(app.tr("family.point_cloud_unsupported"), true);
        return;
    }
    let uid = app.session.uid;
    let revision = app.session.revision;
    let path = app.services.find_viewer.as_ref().and_then(|find| find()).filter(|path| valid_viewer(path));
    if let Some(path) = path {
        choose_output(app, ctx, uid, revision, path);
    } else {
        let title = crate::i18n::Locale::for_language(&app.preferences.language).format("family.choose_viewer", &[("viewer", viewer_name())]);
        app.pick_file(&title, &["exe"], move |app, path| {
            if let Some(path) = path {
                if valid_viewer(&path) {
                    choose_output(app, ctx, uid, revision, path);
                } else {
                    error(app, &format!("{}: {}", viewer_name(), viewer_exe().unwrap_or_default()));
                }
            }
        });
    }
}

fn choose_output(app: &mut SolveApp, ctx: egui::Context, uid: u64, revision: u64, viewer: String) {
    if !unchanged(app, uid, revision) {
        app.set_status(app.tr("family.changed"), true);
        return;
    }
    let name = Path::new(&app.session.doc.name).file_name().and_then(|name| name.to_str()).filter(|name| !name.is_empty()).unwrap_or("model");
    let title = app.tr("family.save_title");
    app.pick_viewer_save(&title, &format!("{name}.step"), move |app, path| {
        let Some(path) = path else { return };
        if !unchanged(app, uid, revision) {
            app.set_status(app.tr("family.changed"), true);
            return;
        }
        let Some(root) = solidcraft_engine::recovery::data_dir().map(|root| root.join("transfer-staging")) else {
            let message = app.tr("family.data_unavailable");
            error(app, message);
            return;
        };
        let output = normalize_output(&path);
        let Some(output) = output else {
            let message = app.tr("family.invalid_step");
            error(app, message);
            return;
        };
        let output_text = output.to_string_lossy().into_owned();
        let mut snapshot = app.session.scratch();
        let locale = crate::i18n::Locale::for_language(&app.preferences.language);
        let (tx, rx) = std::sync::mpsc::channel();
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new().name("viewer-step-export".into()).spawn(move || {
            let result = export_new(&mut snapshot, &output, &root, locale);
            let _ = tx.send(result);
            repaint.request_repaint();
        });
        if let Err(failure) = spawned {
            error(app, &failure.to_string());
            return;
        }
        app.companion_job = Some(Job { done: rx, viewer, output: output_text, uid, revision });
        app.set_status(app.tr("family.exporting"), false);
    });
}

fn normalize_output(path: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(path);
    match path.extension().and_then(|s| s.to_str()) {
        None => {
            path.set_extension("step");
        }
        Some(extension) if extension.eq_ignore_ascii_case("step") || extension.eq_ignore_ascii_case("stp") => {}
        _ => return None,
    }
    Some(path)
}

/// Write a new user-selected file without ever replacing an existing file.
fn export_new(session: &mut solidcraft_engine::Session, output: &Path, root: &Path, locale: crate::i18n::Locale) -> Result<Option<String>, String> {
    if !session.doc.point_clouds.is_empty() {
        return Err(locale.text("family.point_cloud_unsupported").to_string());
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let work = root.join(format!("{}-{time}-{sequence}", std::process::id()));
    std::fs::create_dir(&work).map_err(|e| e.to_string())?;
    let temporary = work.join("model.step");
    let result = (|| {
        session.execute("file.export", &json!({"path": temporary.to_string_lossy(), "format": "step"})).map_err(|e| {
            let detail = e.to_string();
            if detail.starts_with(solidcraft_engine::io::STEP_ASSEMBLY_INVALID_PREFIX) {
                locale.text("family.invalid_assembly").to_string()
            } else {
                detail
            }
        })?;
        #[cfg(windows)]
        {
            match crate::companion_publish::publish_step(&temporary, output) {
                Ok(warning) => Ok(warning),
                Err(failure) => {
                    let key = match failure.stage {
                        "target" | "lock target directories" => "family.stage_target",
                        "open source" => "family.stage_source",
                        "create temporary file" => "family.stage_temp",
                        "publish hard link" => "family.stage_publish",
                        _ => "family.stage_copy",
                    };
                    let message = match failure.cause.raw_os_error() {
                        Some(_) => format!("{}: {}", locale.text(key), failure.cause),
                        None => locale.text(key).to_string(),
                    };
                    match failure.cleanup_warning {
                        Some(warning) => Err(format!("{message}; {}", locale.format("family.cleanup_failed", &[("error", &warning)]))),
                        None => Err(message),
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            Err("Viewer transfer is unavailable on this platform".to_string())
        }
    })();
    let cleanup_file = match std::fs::remove_file(&temporary) {
        Ok(()) => None,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(format!("{}: {e}", temporary.display())),
    };
    let cleanup_dir = std::fs::remove_dir(&work).err().map(|e| format!("{}: {e}", work.display()));
    let cleanup = [cleanup_file, cleanup_dir].into_iter().flatten().collect::<Vec<_>>().join("; ");
    match (result, cleanup.is_empty()) {
        (result, true) => result,
        (Ok(warning), false) => Ok(Some(match warning {
            Some(previous) => format!("{previous}; {cleanup}"),
            None => cleanup,
        })),
        (Err(error), false) => Err(format!("{error}; {}", locale.format("family.cleanup_failed", &[("error", &cleanup)]))),
    }
}

pub(crate) fn tick(app: &mut SolveApp, ctx: &egui::Context) {
    let Some(job) = app.companion_job.as_ref() else { return };
    match job.done.try_recv() {
        Err(TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(100)),
        Err(TryRecvError::Disconnected) => {
            app.companion_job = None;
            let message = app.tr("family.worker_stopped");
            error(app, message);
        }
        Ok(result) => {
            let viewer = job.viewer.clone();
            let path = job.output.clone();
            let current = unchanged(app, job.uid, job.revision);
            app.companion_job = None;
            let warning = match result {
                Ok(warning) => warning,
                Err(failure) => {
                    error(app, &failure);
                    return;
                }
            };
            let locale = crate::i18n::Locale::for_language(&app.preferences.language);
            let append_warning = |text: String| {
                if let Some(warning) = &warning {
                    let detail = locale.format("family.cleanup_failed", &[("error", warning)]);
                    format!("{text}; {detail}")
                } else {
                    text
                }
            };
            if !current {
                app.set_status(append_warning(format!("{} ({path})", app.tr("family.changed"))), true);
            } else if let Some(launch) = &app.services.launch_viewer {
                match launch(&viewer, &path) {
                    Ok(()) => {
                        let text = crate::i18n::Locale::for_language(&app.preferences.language)
                            .format("family.opened", &[("viewer", viewer_name()), ("path", &path)]);
                        app.set_status(append_warning(text), warning.is_some());
                    }
                    Err(failure) => {
                        let text = crate::i18n::Locale::for_language(&app.preferences.language)
                            .format("family.launch_failed", &[("viewer", viewer_name()), ("path", &path), ("error", &failure)]);
                        app.set_status(append_warning(text), true);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_extension_is_limited_to_step() {
        assert_eq!(normalize_output("part"), Some(PathBuf::from("part.step")));
        assert_eq!(normalize_output("Baugruppe ü.STP"), Some(PathBuf::from("Baugruppe ü.STP")));
        assert_eq!(normalize_output("part.dxf"), None);
    }

    #[cfg(windows)]
    #[test]
    fn export_never_overwrites_source_and_new_copy_reopens() {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("solid-viewer-transfer-{}-{id}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let original = dir.join("existing.step");
        let output = dir.join("Baugruppe ü.step");
        let staging = dir.join("staging");
        std::fs::write(&original, b"existing user file").unwrap();
        let mut session = solidcraft_engine::Session::default();
        session.execute("solid.box", &json!({"length":10,"width":20,"height":30})).unwrap();
        session.path = Some(original.to_string_lossy().into_owned());
        let revision = session.revision;
        let undo = session.undo.len();
        let locale = crate::i18n::Locale::for_language("en");
        assert!(export_new(&mut session.scratch(), &original, &staging, locale).is_err());
        assert_eq!(std::fs::read(&original).unwrap(), b"existing user file");
        export_new(&mut session.scratch(), &output, &staging, locale).unwrap();
        assert_eq!(session.revision, revision);
        assert_eq!(session.undo.len(), undo);
        assert_eq!(session.path.as_deref(), original.to_str());
        let mut reopened = solidcraft_engine::Session::default();
        reopened.execute("doc.open", &json!({"path":output})).unwrap();
        let measured = reopened.execute("inspect.measure", &json!({})).unwrap();
        assert!((measured["bodies"][0]["volume_mm3"].as_f64().unwrap() - 6000.0).abs() < 1e-6);
        std::fs::remove_file(original).unwrap();
        std::fs::remove_file(output).unwrap();
        std::fs::remove_dir(staging).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn picker_result_is_rejected_if_active_design_changes() {
        let mut app = SolveApp::new(Default::default(), Default::default());
        let (uid, revision) = (app.session.uid, app.session.revision);
        assert!(unchanged(&app, uid, revision));
        app.session.execute("solid.box", &json!({"length":1,"width":1,"height":1})).unwrap();
        assert!(!unchanged(&app, uid, revision));
    }

    #[test]
    fn point_cloud_with_solid_stops_transfer_before_viewer_or_output() {
        use solidcraft_engine::doc::point_cloud::{ColorBy, Colors, PointCloud, Points};
        use std::cell::Cell;
        use std::rc::Rc;

        let mut app = SolveApp::new(Default::default(), Default::default());
        app.session.execute("solid.box", &json!({"length":1,"width":1,"height":1})).unwrap();
        app.session.doc_mut().point_clouds.push(PointCloud {
            id: 1,
            name: "Scan".into(),
            file: String::new(),
            origin: solidcraft_engine::geom::Vec3::new(0.0, 0.0, 0.0),
            points: Points::new(vec![0.0, 0.0, 0.0].into()),
            colors: Colors::default(),
            source_points: 1,
            units: "mm".into(),
            transform: solidcraft_engine::doc::IDENTITY,
            point_size: 2.0,
            color_by: ColorBy::File,
            color: None,
            height_range: None,
            visible: true,
            complex: Default::default(),
            color_property: None,
            filters: Vec::new(),
        });
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        app.services.find_viewer = Some(Box::new(move || {
            count.set(count.get() + 1);
            None
        }));
        app.services.launch_viewer = Some(Box::new(|_, _| Ok(())));
        start(&mut app, egui::Context::default());
        assert_eq!(calls.get(), 0);
        assert!(app.companion_job.is_none());
        assert!(app.status.as_ref().is_some_and(|status| status.2 && status.0 == app.tr("family.point_cloud_unsupported")));
        let dir = std::env::temp_dir().join(format!("solid-cloud-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let output = dir.join("copy.step");
        let locale = crate::i18n::Locale::for_language("en");
        assert_eq!(export_new(&mut app.session.scratch(), &output, &dir, locale).unwrap_err(), locale.text("family.point_cloud_unsupported"));
        assert!(!dir.exists());
    }

    #[test]
    fn unreachable_assembly_body_reports_localized_error_without_step() {
        let mut app = SolveApp::new(Default::default(), Default::default());
        app.session.execute("solid.box", &json!({"length":1,"width":1,"height":1})).unwrap();
        let body = app.session.model.state().bodies[0].name.clone();
        let doc = app.session.doc_mut();
        doc.components.push(solidcraft_engine::doc::Component { id: 7, name: "Orphan".into(), ..Default::default() });
        doc.body_components.insert(body, 7);
        let dir = std::env::temp_dir().join(format!("solid-orphan-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let output = dir.join("copy.step");
        let locale = crate::i18n::Locale::for_language("de");
        assert_eq!(export_new(&mut app.session.scratch(), &output, &dir, locale).unwrap_err(), locale.text("family.invalid_assembly"));
        assert!(!output.exists());
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn root_body_with_unused_component_exports_and_reopens() {
        let mut session = solidcraft_engine::Session::default();
        session.execute("solid.box", &json!({"length":10,"width":20,"height":30})).unwrap();
        session.doc_mut().components.push(solidcraft_engine::doc::Component { id: 7, name: "Unused".into(), ..Default::default() });
        let dir = std::env::temp_dir().join(format!("solid-root-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&dir).unwrap();
        let output = dir.join("root.step");
        session.execute("file.export", &json!({"path":output,"format":"step"})).unwrap();
        let mut reopened = solidcraft_engine::Session::default();
        reopened.execute("doc.open", &json!({"path":output})).unwrap();
        let measured = reopened.execute("inspect.measure", &json!({})).unwrap();
        assert!((measured["bodies"][0]["volume_mm3"].as_f64().unwrap() - 6000.0).abs() < 1e-6);
        std::fs::remove_file(output).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn completed_export_preserves_cleanup_warning_and_launch_result() {
        use std::cell::Cell;
        use std::rc::Rc;

        for (changed, launch_fails) in [(false, false), (false, true), (true, false)] {
            let mut app = SolveApp::new(Default::default(), Default::default());
            let launches = Rc::new(Cell::new(0));
            let called = launches.clone();
            app.services.launch_viewer = Some(Box::new(move |_, _| {
                called.set(called.get() + 1);
                if launch_fails { Err("launch-failure".to_string()) } else { Ok(()) }
            }));
            let (sender, receiver) = std::sync::mpsc::channel();
            sender.send(Ok(Some("cleanup-detail".to_string()))).unwrap();
            app.companion_job = Some(Job {
                done: receiver,
                viewer: "viewer.exe".to_string(),
                output: "copy.step".to_string(),
                uid: app.session.uid,
                revision: app.session.revision,
            });
            if changed {
                app.session.execute("solid.box", &json!({"length":1,"width":1,"height":1})).unwrap();
            }
            tick(&mut app, &egui::Context::default());
            assert_eq!(launches.get(), usize::from(!changed));
            let status = &app.status.as_ref().unwrap().0;
            assert!(status.contains("cleanup-detail"));
            assert!(status.contains("copy.step"));
            if launch_fails {
                assert!(status.contains("launch-failure"));
            }
        }
    }
}
