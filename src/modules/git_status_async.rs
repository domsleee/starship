use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::Local;

use crate::{
    configs::git_status::GitStatusConfig,
    context::{self, Context},
};
use std::io::Write;

use super::git_status::RepoStatus;

pub struct GitStatusAsync {
    async_paths: AsyncPaths,
    log_file: File,
    enabled: bool,
}

impl GitStatusAsync {
    pub fn new(
        context: &Context,
        repo: &context::Repo,
        config: &GitStatusConfig,
    ) -> GitStatusAsync {
        let async_paths = get_async_paths(context, repo);
        let log_file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&async_paths.log_path)
            .expect("able to open file");

        let enabled = config
            .async_paths
            .as_ref()
            .map(|x| x.contains(&repo.path.parent().unwrap().to_path_buf()))
            .unwrap_or(false);

        return Self {
            async_paths,
            log_file,
            enabled,
        };
    }

    pub fn get_git_status_and_run_worker(
        &mut self,
        context: &Context,
        repo: &context::Repo,
    ) -> Option<RepoStatus> {
        if !self.enabled {
            return None;
        }

        log::debug!("retrieving git_status from async, {:?}", self.async_paths);

        if context.properties.is_async_worker {
            File::create(&self.async_paths.lock_path).expect("can be created");
            self.log_info("async worker created a lock!");
            return None;
        }

        if !self.async_paths.lock_path.exists() {
            let repo_path = repo.path.parent()?.to_string_lossy();
            launch_async_worker(&repo_path.to_string());
        } else {
            log::debug!(
                "did not launch git_status worker since lock file exists ({:?})",
                self.async_paths.lock_path
            );
        }

        if !self.async_paths.data_path.exists() {
            return None;
        }

        log::debug!("reading git_status from {:?}", self.async_paths.data_path);
        let file = File::open(&self.async_paths.data_path).ok()?;
        serde_json::from_reader(file).ok()
    }

    pub fn store_result(
        &mut self,
        context: &Context,
        repo_status: &RepoStatus,
    ) -> Result<(), std::io::Error> {
        if !context.properties.is_async_worker || !self.enabled {
            return Ok(());
        }

        log::debug!("setting git_status from async");
        let file = File::create(&self.async_paths.data_path)?;
        serde_json::to_writer(file, &repo_status)?;

        log::debug!("removing lock");
        self.log_info("removing_lock");
        match fs::canonicalize(&self.async_paths.lock_path) {
            Ok(path) => {
                fs::remove_file(&path)?;
                log::debug!("lock removed");
                self.log_info("lock removed");
            }
            Err(_) => {
                log::debug!("lock {:?} did not exist?", self.async_paths.lock_path);
                self.log_info(&format!(
                    "lock {:?} did not exist?",
                    self.async_paths.lock_path
                ));
            }
        }
        Ok(())
    }

    fn log_info(&mut self, s: &str) {
        let current_time = Local::now();
        let formatted_time = current_time.format("%H:%M:%S%.3f");
        writeln!(self.log_file, "[{}] {}", formatted_time, s).expect("able to append");
    }
}

#[derive(Debug)]
struct AsyncPaths {
    lock_path: PathBuf,
    data_path: PathBuf,
    log_path: PathBuf,
}

fn get_async_paths(context: &Context, repo: &context::Repo) -> AsyncPaths {
    let repo_path = repo.path.to_string_lossy();
    let home_path = context.get_home().expect("must have home path");
    let async_dir = Path::new(&home_path)
        .join(".config")
        .join("git_status_async");

    if !async_dir.exists() {
        fs::create_dir_all(&async_dir).expect("should be able to create directory");
    }

    let sanitized_repo_path = repo_path.replace(&['/', '\\', ':'][..], "_");
    log::debug!("async_dir: {async_dir:?}, sanitized_repo_path: {sanitized_repo_path}");
    return AsyncPaths {
        lock_path: async_dir.join(format!("{sanitized_repo_path}.lock")),
        data_path: async_dir.join(format!("{sanitized_repo_path}.json")),
        log_path: async_dir.join(format!("{sanitized_repo_path}.log")),
    };
}

fn launch_async_worker(repo_path: &str) {
    log::debug!("launching async git_status worker");
    #[cfg(windows)]
    {
        let args = format!("module --is-async-worker git_status --path \"{repo_path}\"");
        crate::win_fast_spawn::fast_background_spawn(r"starship.exe", &args);
    }

    // todo: below works on windows but is noticeably slower on powershell 🤔🤔🤔
    #[cfg(not(windows))]
    {
        let child = std::process::Command::new("starship")
            .args(&[
                "module",
                "--is-async-worker",
                "git_status",
                "--path",
                repo_path,
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Can spawn");
        std::mem::forget(child);
    }
}
