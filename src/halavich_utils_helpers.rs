use error_stack::Report;
/// HalavicH's utils & helpers (HUH)
/// Module to collect cross-project helper functions
use std::sync::Arc;
use tokio::sync::Mutex;

/// Asynchronously mutually exclusive shared object (wrapper for the Arc<tokio::sync::Mutex>>)
pub type AMShared<T> = Arc<Mutex<T>>;
pub trait IntoAMShared {
    fn into_shared(self) -> AMShared<Self>;
}

// Blanked implementation
impl<T> IntoAMShared for T {
    fn into_shared(self) -> AMShared<Self> {
        Arc::new(Mutex::new(self))
    }
}

pub trait ArcEd {
    fn into_arc(self) -> Arc<Self>;
}

impl<T> ArcEd for T {
    fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

pub trait Boxed {
    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T> Boxed for T {} // Blanket implementation

///// Error Stack Ext /////
pub trait IntoReport<T, E> {
    fn into_report(self) -> error_stack::Result<T, E>;
}

impl<T, E> IntoReport<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_report(self) -> error_stack::Result<T, E> {
        self.map_err(Report::new)
    }
}

// type ErasedError = impl std::error::Error + Send + Sync + 'static;

pub trait ErrorStackExt {
    fn to_string_no_bt(&self) -> String;
    fn into_error_and_log(self) -> impl std::error::Error + Send + Sync + 'static;
}

impl<E: 'static> ErrorStackExt for error_stack::Report<E> {
    fn to_string_no_bt(&self) -> String {
        let string = format!("{:?}", self);
        let split = string
            .split("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            .collect::<Vec<&str>>();
        split.first().unwrap().to_string()
    }

    fn into_error_and_log(self) -> impl std::error::Error + Send + Sync + 'static {
        log::error!("Can't find users in range: {}", self.to_string_no_bt());
        self.into_error()
    }
}

pub trait ErrorStackResultExt<T> {
    fn into_error_and_log(
        self,
    ) -> error_stack::Result<T, impl std::error::Error + Send + Sync + 'static>;
    fn into_error_no_bt(
        self,
    ) -> error_stack::Result<T, impl std::error::Error + Send + Sync + 'static>;
}

impl<T, E: 'static> ErrorStackResultExt<T> for error_stack::Result<T, E> {
    fn into_error_and_log(
        self,
    ) -> error_stack::Result<T, impl std::error::Error + Send + Sync + 'static> {
        Ok(self.map_err(|e| e.into_error_and_log())?)
    }

    fn into_error_no_bt(
        self,
    ) -> error_stack::Result<T, impl std::error::Error + Send + Sync + 'static> {
        Ok(self.map_err(|e| e.into_error())?)
    }
}

#[cfg(test)]
mod error_stack_ext_tests {
    use super::*;
    use derive_more::Display;
    use error_stack::bail;
    use thiserror::Error;

    #[derive(Debug, Error, Display)]
    pub enum SomeError {
        Foo,
        Bar,
    }

    fn failing_function() -> error_stack::Result<(), SomeError> {
        bail!(SomeError::Bar)
    }

    #[test]
    fn given_typed_error__when_into_error_and_log__then_ok() {
        let result = failing_function();
        println!("Original error: {:#?}", result);
        let erased: error_stack::Result<(), _> = result.into_error_and_log();
        println!("Erazed error: {:#?}", erased);
    }
}
