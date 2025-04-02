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

pub trait ErrorStackExt {
    fn to_string_no_bt(&self) -> String;
}

impl<E> ErrorStackExt for error_stack::Report<E> {
    fn to_string_no_bt(&self) -> String {
        let string = format!("{:?}", self);
        let split = string
            .split("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            .collect::<Vec<&str>>();
        split.first().unwrap().to_string()
    }
}
