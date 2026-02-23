  // The following code is a translation of
  // https://github.com/kovidgoyal/kitty/blob/master/kittens/ssh/main.go#L122
  // in particular how the runtime dir is obtained for use in control socket path.
  use dirs;
  use std::env;
  use std::fs;
  use std::io;
  use std::os::unix::fs::symlink;
  use std::path::{Path, PathBuf};

  const SSH_CONTROL_MASTER_TEMPLATE: &str = "pctlssh-{pid}-{ssh_placeholder}";

  fn runtime_dir() -> io::Result<PathBuf> {

      if cfg!(target_os = "macos")
      {
          if let Some(dir) = macos_user_cache_dir() {
              let path = PathBuf::from(dir);
              fs::create_dir_all(&path)?;
              set_permissions_0700(&path)?;
              return Ok(path);
          }
      }

      // Use dirs crate for XDG_RUNTIME_DIR
      if let Some(dir) = dirs::runtime_dir() {
          fs::create_dir_all(&dir)?;
          set_permissions_0700(&dir)?;
          return Ok(dir);
      }

      // Fallback to /run/user/{uid} with permission checks
      let uid = unsafe { libc::geteuid() };
      let run_user_dir = format!("/run/user/{}", uid);
      if let Ok(metadata) = fs::metadata(&run_user_dir) {
          if metadata.is_dir() && access_check(&run_user_dir) {
              return Ok(PathBuf::from(run_user_dir));
          }
      }

      // Final fallback: {cache_dir}/run
      let cache = cache_dir()?;
      let runtime_path = cache.join("run");
      fs::create_dir_all(&runtime_path)?;
      set_permissions_0700(&runtime_path)?;
      Ok(runtime_path)
  }

  /// Get cache directory using dirs crate where possible
  fn cache_dir() -> io::Result<PathBuf> {
      if cfg!(target_os = "macos")
      {
          // dirs::cache_dir() returns ~/Library/Caches, we add "promptcmd"
          if let Some(cache) = dirs::cache_dir() {
              let path = cache.join("promptcmd");
              fs::create_dir_all(&path)?;
              return Ok(path);
          }
          // Fallback if dirs fails
          let path = expanduser("~/Library/Caches/promptcmd");
          fs::create_dir_all(&path)?;
          return Ok(path);
      }

      #[cfg(not(target_os = "macos"))]
      {
          // dirs::cache_dir() handles XDG_CACHE_HOME and ~/.cache
          if let Some(cache) = dirs::cache_dir() {
              let path = cache.join("promptcmd");
              fs::create_dir_all(&path)?;
              return Ok(path);
          }
          // Fallback if dirs fails
          let path = expanduser("~/.cache").join("promptcmd");
          fs::create_dir_all(&path)?;
          Ok(path)
      }
  }

  fn expanduser(path: &str) -> PathBuf {
      if path.starts_with('~') {
          if let Some(home) = dirs::home_dir() {
              if path == "~" {
                  return home;
              } else if path.starts_with("~/") {
                  return home.join(&path[2..]);
              }
          }
      }
      PathBuf::from(path)
  }

  fn set_permissions_0700(path: &Path) -> io::Result<()> {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = fs::metadata(path)?.permissions();
      if perms.mode() & 0o777 != 0o700 {
          perms.set_mode(0o700);
          fs::set_permissions(path, perms)?;
      }
      Ok(())
  }

  fn access_check(path: &str) -> bool {
      use std::ffi::CString;
      if let Ok(c_path) = CString::new(path) {
          unsafe {
              libc::access(c_path.as_ptr(), libc::R_OK | libc::W_OK | libc::X_OK) == 0
          }
      } else {
          false
      }
  }

  fn macos_user_cache_dir() -> Option<String> {
      use std::process::Command;

      // Try TMPDIR hack first
      if let Ok(tmpdir) = env::var("TMPDIR") {
          let tmpdir = tmpdir.trim_end_matches('/');
          let path = Path::new(tmpdir);
          if path.file_name().and_then(|n| n.to_str()) == Some("T") {
              if let Some(parent) = path.parent() {
                  let candidate = parent.join("C");
                  if is_valid_macos_cache_dir(&candidate) {
                      return Some(candidate.to_string_lossy().into_owned());
                  }
              }
          }
      }

      // Try glob pattern
      if let Ok(entries) = glob::glob("/private/var/folders/*/*/C") {
          for entry in entries.flatten() {
              if is_valid_macos_cache_dir(&entry) {
                  return Some(entry.to_string_lossy().into_owned());
              }
          }
      }

      // Fallback to getconf
      if let Ok(output) = Command::new("/usr/bin/getconf")
          .arg("DARWIN_USER_CACHE_DIR")
          .output()
      {
          if output.status.success() {
              let dir = String::from_utf8_lossy(&output.stdout);
              return Some(dir.trim().trim_end_matches('/').to_string());
          }
      }

      None
  }

  fn is_valid_macos_cache_dir(path: &Path) -> bool {
      use std::os::unix::fs::MetadataExt;

      if let Ok(metadata) = fs::metadata(path) {
          let uid = unsafe { libc::geteuid() };
          metadata.is_dir()
              && metadata.uid() == uid
              && metadata.mode() & 0o777 == 0o700
              && access_check(&path.to_string_lossy())
      } else {
          false
      }
  }

  fn atomic_create_symlink(oldname: &Path, newname: &Path) -> io::Result<()> {
      match symlink(oldname, newname) {
          Ok(()) => return Ok(()),
          Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
          _ => {}
      }

      if let Ok(existing_target) = fs::read_link(newname) {
          if existing_target == oldname {
              return Ok(());
          }
      }

      loop {
          let random_suffix: String = (0..8)
              .map(|_| format!("{:x}", rand::random::<u8>()))
              .collect();
          let tempname = PathBuf::from(format!(
              "{}{}",
              newname.to_string_lossy(),
              random_suffix
          ));

          match symlink(oldname, &tempname) {
              Ok(()) => {
                  match fs::rename(&tempname, newname) {
                      Ok(()) => return Ok(()),
                      Err(e) => {
                          let _ = fs::remove_file(&tempname);
                          return Err(e);
                      }
                  }
              }
              Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
              Err(e) => return Err(e),
          }
      }
  }


  pub fn control_path(pid: u32) -> io::Result<String> {
      let rd = runtime_dir()?;
      let rd_str = rd.to_string_lossy().into_owned();

      // Bloody OpenSSH generates a 40 char hash and in creating the socket appends a 27 char temp
      // suffix to it. Socket max path length is approx ~104 chars. And on idiotic Apple the path
      // length to the runtime dir (technically the cache dir since Apple has no runtime dir and
      // thinks it's a great idea to delete files in /tmp) is ~48 chars.
      let rd_to_use = if rd_str.len() > 35 {
          let uid = unsafe { libc::geteuid() };
          let idiotic_design = format!("/tmp/kssh-rdir-{}", uid);
          atomic_create_symlink(&rd, Path::new(&idiotic_design))?;
          idiotic_design
      } else {
          rd_str
      };

      let cp = SSH_CONTROL_MASTER_TEMPLATE
          .replace("{pid}", &pid.to_string())
          .replace("{ssh_placeholder}", "%C");

      let control_path = Path::new(&rd_to_use).join(cp);
      Ok(control_path.to_string_lossy().into_owned())
  }


