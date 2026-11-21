//! A single Exile Market at once (specifications, Error handling; mock-up section "Already open
//! (22m)"): a lock on a file of the cache directory, taken before anything else, even the log,
//! whose rotation would rename the log of the Exile Market already open.

use std::fs::{self, File, TryLockError};
use std::io;
use std::path::Path;

use crate::i18n::Translator;

/// The lock file, in the cache directory.
const FILE_NAME: &str = "exile-market.lock";

/// What the lock says of this launch.
#[derive(Debug)]
pub enum Instance {
    /// No other Exile Market runs: this one holds the lock while it lives.
    Alone(Lock),
    /// Another Exile Market holds the lock.
    AlreadyOpen,
    /// The lock couldn't be taken (no home directory, a folder that can't be written): this
    /// launch runs as usual, two Exile Market being worth more than none.
    Unlocked(io::Error),
}

/// The lock of this Exile Market, held while it lives: the system releases it when the process
/// ends, even after a crash, so that no lock is ever left behind.
#[derive(Debug)]
pub struct Lock {
    _file: File,
}

/// Takes the lock in the cache directory of the user.
#[must_use]
pub fn take() -> Instance {
    match crate::files::cache_directory() {
        Some(directory) => take_in(&directory),
        None => Instance::Unlocked(io::Error::new(
            io::ErrorKind::NotFound,
            "no home directory for the lock file",
        )),
    }
}

/// Takes the lock in `directory`, made when missing.
fn take_in(directory: &Path) -> Instance {
    let file = fs::create_dir_all(directory).and_then(|()| {
        File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(directory.join(FILE_NAME))
    });
    let file = match file {
        Ok(file) => file,
        Err(error) => return Instance::Unlocked(error),
    };
    match file.try_lock() {
        Ok(()) => Instance::Alone(Lock { _file: file }),
        Err(TryLockError::WouldBlock) => Instance::AlreadyOpen,
        Err(TryLockError::Error(error)) => Instance::Unlocked(error),
    }
}

/// The message of a launch while Exile Market already runs: that it is open, then where its
/// window may be, in paragraphs.
#[must_use]
pub fn message(translator: &Translator) -> String {
    translator.already_open().join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::TestDirectory;
    use crate::language::Language;

    #[test]
    fn a_second_launch_finds_the_lock_taken_until_the_first_one_ends() {
        let directory = TestDirectory::new("single-instance");
        let first = take_in(&directory.0);
        assert!(matches!(first, Instance::Alone(_)), "{first:?}");
        let second = take_in(&directory.0);
        assert!(matches!(second, Instance::AlreadyOpen), "{second:?}");
        drop(first);
        let after = take_in(&directory.0);
        assert!(matches!(after, Instance::Alone(_)), "{after:?}");
    }

    #[test]
    fn a_lock_that_cant_be_taken_lets_the_launch_go_on() {
        let directory = TestDirectory::new("single-instance-blocked");
        // A file where the folder should be.
        fs::write(&directory.0, "").expect("the file");
        let instance = take_in(&directory.0);
        assert!(matches!(instance, Instance::Unlocked(_)), "{instance:?}");
        fs::remove_file(&directory.0).expect("removed");
    }

    #[test]
    fn the_message_says_it_is_open_then_where_the_window_may_be() {
        assert_eq!(
            message(&Translator::new(Language::English)),
            "Exile Market is already open.\n\n\
             Its window may be minimized, or behind the game: look for it in the taskbar."
        );
        assert_eq!(
            message(&Translator::new(Language::French)),
            "Exile Market est déjà ouvert.\n\n\
             Sa fenêtre est peut-être réduite, ou derrière le jeu\u{a0}: cherchez-la dans la \
             barre des tâches."
        );
    }
}
