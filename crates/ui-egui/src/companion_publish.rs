#![forbid(unsafe_code)]
#[cfg(windows)]
mod windows {
    use std::{
        fs::{self, File, OpenOptions},
        io::{self, Write},
        os::windows::fs::{MetadataExt, OpenOptionsExt},
        path::{Component, Path, PathBuf, Prefix},
    };

    const FILE_SHARE_READ: u32 = 1;
    const FILE_SHARE_WRITE: u32 = 2;
    const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;

    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;

    fn validate_and_lock_parent(target: &Path) -> io::Result<Vec<File>> {
        let raw = target.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "non-Unicode path rejected"))?;
        if raw.contains('/') || raw.split('\\').any(|s| s == "." || s == "..") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "ambiguous path"));
        }
        let mut parts = target.components();
        let prefix = match parts.next() {
            Some(Component::Prefix(p))
                if matches!(p.kind(), Prefix::Disk(_) | Prefix::UNC(_, _) | Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(_, _)) =>
            {
                p
            }
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "only absolute drive and UNC paths are supported")),
        };
        if !matches!(parts.next(), Some(Component::RootDir)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "path must be absolute"));
        }
        let mut all = Vec::new();
        for part in parts {
            let Component::Normal(name) = part else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid path component"));
            };
            let name = name.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid path encoding"))?;
            let base = name.split('.').next().unwrap_or("").to_ascii_uppercase();
            let reserved = matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || ["COM", "LPT"].iter().any(|p| {
                    base.strip_prefix(p).is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
                });
            if name.ends_with(['.', ' ']) || reserved || name.chars().any(|c| c < ' ' || "<>:\"|?*".contains(c)) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid or reserved path component"));
            }
            all.push(name);
        }
        if all.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "missing filename"));
        }
        let mut current = PathBuf::from(prefix.as_os_str());
        current.push("\\");
        let mut locks = Vec::new();
        for step in std::iter::once(None).chain(all.iter().take(all.len() - 1).map(Some)) {
            if let Some(part) = step {
                current.push(part);
            }
            let file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&current)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", current.display())))?;
            let attrs = file.metadata()?.file_attributes();
            if attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 || attrs & FILE_ATTRIBUTE_DIRECTORY == 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "reparse point or non-directory in target path"));
            }
            locks.push(file);
        }
        Ok(locks)
    }

    #[derive(Debug)]
    pub struct PublishError {
        pub stage: &'static str,
        pub cause: io::Error,
        /// A remaining entry, or a metadata check error, is reported but cannot
        /// distinguish a cleanup failure from a concurrent path replacement.
        pub cleanup_warning: Option<String>,
    }

    fn close_and_check(file: File, path: &Path) -> Option<String> {
        drop(file);
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Ok(_) => Some(path.display().to_string()),
            Err(e) => Some(format!("{}: {e}", path.display())),
        }
    }

    /// Caller must provide a completed, immutable source and a stable parent path.
    /// A success may carry a cleanup warning; no partial file is published.
    pub fn publish_step(source: &Path, target: &Path) -> Result<Option<String>, PublishError> {
        publish_with_hook(source, target, |_| Ok(()))
    }

    fn publish_with_hook<F>(source: &Path, target: &Path, hook: F) -> Result<Option<String>, PublishError>
    where
        F: FnOnce(&Path) -> io::Result<()>,
    {
        let error = |stage, cause| PublishError { stage, cause, cleanup_warning: None };
        if target.file_name().is_none() {
            return Err(error("target", io::Error::new(io::ErrorKind::InvalidInput, "target must name a file")));
        }
        let parent = target.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        let _directory_locks = validate_and_lock_parent(target).map_err(|e| error("lock target directories", e))?;
        let mut source_file = OpenOptions::new().read(true).share_mode(FILE_SHARE_READ).open(source).map_err(|e| error("open source", e))?;
        let temp = tempfile::Builder::new()
            .prefix(".solid-step-")
            .rand_bytes(16)
            .disable_cleanup(true)
            .make_in(parent, |p| {
                OpenOptions::new().write(true).create_new(true).share_mode(FILE_SHARE_READ).custom_flags(FILE_FLAG_DELETE_ON_CLOSE).open(p)
            })
            .map_err(|e| error("create temporary file", e))?;
        let (mut file, path) = temp.into_parts();
        let path: PathBuf = path.to_path_buf();
        let result = io::copy(&mut source_file, &mut file)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
            .map_err(|e| ("copy or sync", e))
            .and_then(|_| hook(&path).map_err(|e| ("pre-publication check", e)))
            .and_then(|_| fs::hard_link(&path, target).map_err(|e| ("publish hard link", e)));
        let cleanup_warning = close_and_check(file, &path);
        match result {
            Ok(()) => Ok(cleanup_warning),
            Err((stage, cause)) => Err(PublishError { stage, cause, cleanup_warning }),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn as_io(e: PublishError) -> io::Error {
            io::Error::other(format!("{}: {}", e.stage, e.cause))
        }
        fn suite(base: &Path) -> io::Result<()> {
            let d = tempfile::Builder::new().prefix("solid-publish-suite-").tempdir_in(base)?;
            let inside = d.path().join("inside");
            fs::create_dir(&inside)?;
            let src = d.path().join("kaynak.step");
            let dst = inside.join("çıktı-零.step");
            fs::write(&src, b"complete step")?;
            let w = publish_with_hook(&src, &dst, |tmp| {
                assert!(!dst.exists());
                assert!(OpenOptions::new().write(true).open(&src).is_err());
                assert!(OpenOptions::new().write(true).open(tmp).is_err());
                assert!(fs::remove_file(tmp).is_err());
                assert!(fs::rename(&inside, d.path().join("moved_inside")).is_err());
                assert!(fs::rename(d.path(), d.path().with_extension("moved")).is_err());
                Ok(())
            })
            .map_err(as_io)?;
            assert!(w.is_none());
            assert_eq!(fs::read(&dst)?, b"complete step");
            fs::write(&src, b"replacement")?;
            let e = publish_step(&src, &dst).expect_err("existing target must fail");
            assert!(e.cleanup_warning.is_none());
            assert_eq!(fs::read(&dst)?, b"complete step");
            assert_eq!(fs::read_dir(&inside)?.count(), 1);
            let fresh = inside.join("failure.step");
            let e = publish_with_hook(&src, &fresh, |_| Err(io::Error::other("injected"))).expect_err("must fail");
            assert!(e.cleanup_warning.is_none());
            assert!(!fresh.exists());
            assert_eq!(fs::read_dir(&inside)?.count(), 1);
            let writer = OpenOptions::new().write(true).open(&src)?;
            assert!(publish_step(&src, &fresh).is_err());
            assert!(!fresh.exists());
            drop(writer);
            d.close()?;
            Ok(())
        }
        #[test]
        fn local() -> io::Result<()> {
            suite(&std::env::temp_dir())
        }
        #[test]
        fn long_unicode_verbatim_disk() -> io::Result<()> {
            let d = tempfile::tempdir()?;
            let plain = d.path().to_str().ok_or_else(|| io::Error::other("not unicode"))?;
            let mut nested = PathBuf::from(format!(r"\\?\{plain}"));
            for _ in 0..9 {
                nested.push("uzun dizin Türkçe çığ-零-123456");
            }
            fs::create_dir_all(&nested)?;
            assert!(std::os::windows::ffi::OsStrExt::encode_wide(nested.as_os_str()).count() > 260);
            suite(&nested)?;
            d.close()?;
            Ok(())
        }
        #[test]
        #[ignore = "SMB requires explicit test environment"]
        fn smb() -> io::Result<()> {
            suite(Path::new(&std::env::var("SOLID_SMB_TEST").map_err(io::Error::other)?))
        }
        #[test]
        #[ignore = "requires explicit UNC test environment"]
        fn verbatim_unc() -> io::Result<()> {
            let base = std::env::var("SOLID_SMB_TEST").map_err(io::Error::other)?;
            let tail = base.strip_prefix(r"\\").ok_or_else(|| io::Error::other("UNC required"))?;
            suite(&PathBuf::from(format!(r"\\?\UNC\{tail}")))
        }
        #[test]
        #[ignore = "requires explicit UNC test environment"]
        fn long_verbatim_unc() -> io::Result<()> {
            let base = std::env::var("SOLID_SMB_TEST").map_err(io::Error::other)?;
            let tail = base.strip_prefix(r"\\").ok_or_else(|| io::Error::other("UNC required"))?;
            let unc = PathBuf::from(format!(r"\\?\UNC\{tail}"));
            let d = tempfile::Builder::new().prefix("solid-long-").tempdir_in(&unc)?;
            let mut nested = d.path().to_path_buf();
            for _ in 0..9 {
                nested.push("uzun dizin Türkçe çığ-零-123456");
            }
            fs::create_dir_all(&nested)?;
            assert!(std::os::windows::ffi::OsStrExt::encode_wide(nested.as_os_str()).count() > 260);
            suite(&nested)?;
            d.close()?;
            Ok(())
        }
        #[test]
        #[ignore = "requires explicit existing junction fixture"]
        fn junction_rejected() -> io::Result<()> {
            let p = std::env::var_os("SOLID_JUNCTION_TEST").ok_or_else(|| io::Error::other("missing fixture"))?;
            let path = PathBuf::from(p);
            assert_ne!(fs::symlink_metadata(&path)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT, 0);
            let error = validate_and_lock_parent(&path.join("target.step")).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(error.to_string(), "reparse point or non-directory in target path");
            Ok(())
        }
        #[test]
        fn invalid_paths_fail_closed() {
            for p in [
                r"C:relative.step",
                r"C:\dir\..\a.step",
                r"C:\dir\NUL.step",
                r"C:\dir\x:stream",
                r"\\.\C:\dir\x.step",
                r"\\?\GLOBALROOT\Device\HarddiskVolume1\x",
                r"\\?\Volume{00000000-0000-0000-0000-000000000000}\x",
                r"\\?\C:\dir\..\x",
                r"\\?\UNC\sickserver\Allgemein\a:stream",
            ] {
                assert_eq!(validate_and_lock_parent(Path::new(p)).unwrap_err().kind(), io::ErrorKind::InvalidInput, "{p}");
            }
        }
    }
}
#[cfg(windows)]
pub(crate) use windows::publish_step;
