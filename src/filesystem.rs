use std::path::{Path, PathBuf};
use std::env;
use std::fs::DirBuilder;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ValidationError {
    UserAbort,
    TargetIsADir,
    EnvError(io::Error),
    BuildDirErr(io::Error),
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        match *self {
            ValidationError::UserAbort => "Abort by user.",
            ValidationError::TargetIsADir => "The selected target is a directory, not a file.",
            ValidationError::EnvError(ref err) => err.description().clone(),
            ValidationError::BuildDirErr(ref err) => err.description().clone(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ERROR] {}", self.description())
    }
}


fn ask_user(question: &str) -> bool {
    let mut answer = String::new();
    loop {
        print!("{} (y/n) ", question);
        io::stdin().read_line(&mut answer).unwrap();
        match answer.trim() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => continue,
        };
    }
}

pub fn validate_target_file(target_path: &str) -> Result<&Path, ValidationError> {
    let mut path = Path::new(target_path);

    // make sure the target directory is absolute
    if path.is_relative() {
        // get the current working directory and adoin the provided relative path onto it
        // potential environment errors are reported back as specialized error type
        let cwd = match env::current_dir() {
            Ok(dir) => dir,
            Err(e) => return Err(ValidationError::EnvError(e)),
        };

        // also, canonicalize the path to generate a clean path w/o relative path descriptions
        // like "../ " and
        path = match cwd.join(path).canonicalize() {
            Ok(dir) => dir.as_path(),
            Err(e) => return Err(ValidationError::EnvError(e)),
        };
    }


    if path.exists() {
        if path.is_file() {
            // The path exists and is a file.
            Ok(path)
        } else {
            Err(ValidationError::TargetIsADir)
        }
    } else {
        // check whether the parent directory exists. If so, create the target directory
        // if not, ask the user if he's sure that he wants to create the directory recursively
        let parent_dir = path.parent().unwrap();
        if parent_dir.exists() {
            Ok(path)
        } else {
            if ask_user("The path to the target directory does not exist. Create it?") {
                let _ = match DirBuilder::new().recursive(true).create(parent_dir) {
                    Ok(b) => b,
                    Err(err) => return Err(ValidationError::BuildDirErr(err)),
                };
                Ok(path)
            } else {
                Err(ValidationError::UserAbort)
            }
        }
    }
}
