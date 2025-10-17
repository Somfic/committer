use miette::Diagnostic;

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    GitError(#[from] git2::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
