use miette::Diagnostic;

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    GitDiscoveryError(#[from] gix::discover::Error),
    #[error(transparent)]
    GitStatusError(#[from] gix::status::Error),
    #[error(transparent)]
    GitStatusIntoIterError(#[from] gix::status::into_iter::Error),
    #[error(transparent)]
    GitStatusIterError(#[from] gix::status::iter::Error),
    #[error(transparent)]
    GitPeelIntoIdError(#[from] gix::head::peel::into_id::Error),
    #[error(transparent)]
    GitError(#[from] gix::reference::find::existing::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
