use std::path::PathBuf;
use fs2::FileExt;
use std::fs;
use std::io::{Error,ErrorKind};
use std::process::Command;
use std::ffi::OsStr;

pub struct Lock {
    lock: Option<fs::File>
}

impl Lock {
    pub fn unlock(&mut self) {
        self.lock = None
    }
}

pub trait GitClonable {
    fn clone_from(&self) -> String;
    fn clone_to(&self) -> PathBuf;
    fn extra_clone_args(&self) -> Vec<&OsStr>;

    fn lock_path(&self) -> PathBuf;

    fn lock(&self) -> Result<Lock, Error> {
        println!("Locking {:?}", self.lock_path());
        let lock = fs::File::create(self.lock_path())?;
        lock.lock_exclusive()?;
        return Ok(Lock{
            lock: Some(lock)
        })
    }

    fn clone_repo(&self) -> Result<(), Error> {
        let mut lock = self.lock()?;

        if self.clone_to().is_dir() {
            return Ok(())
        }

        let result = Command::new("git")
            .arg("clone")
            .args(self.extra_clone_args())
            .arg(&self.clone_from())
            .arg(&self.clone_to())
            .status()?;

        lock.unlock();

        if result.success() {
            return Ok(())
        } else {
            return Err(Error::new(ErrorKind::Other, "Failed to clone"));
        }
    }

    fn fetch_repo(&self) -> Result<(), Error> {
        let mut lock = self.lock()?;

        let result = Command::new("git")
            .arg("fetch")
            .arg("origin")
            .current_dir(self.clone_to())
            .status()?;

        lock.unlock();

        if result.success() {
            return Ok(())
        } else {
            return Err(Error::new(ErrorKind::Other, "Failed to fetch"));
        }
    }

    fn clean(&self) -> Result<(), Error> {
        let mut lock = self.lock()?;

        Command::new("git")
            .arg("am")
            .arg("--abort")
            .current_dir(self.clone_to())
            .status()?;

        Command::new("git")
            .arg("merge")
            .arg("--abort")
            .current_dir(self.clone_to())
            .status()?;

        Command::new("git")
            .arg("reset")
            .arg("--hard")
            .current_dir(self.clone_to())
            .status()?;

        lock.unlock();

        return Ok(())
    }

    fn checkout(&self, git_ref: &OsStr) -> Result<(), Error> {
        let mut lock = self.lock()?;

        let result = Command::new("git")
            .arg("checkout")
            .arg(git_ref)
            .current_dir(self.clone_to())
            .status()?;

        lock.unlock();

        if result.success() {
            return Ok(())
        } else {
            return Err(Error::new(ErrorKind::Other, "Failed to checkout"));
        }
    }
}
