#[derive(Debug)]
pub struct ErrorWithContext {
    inner: Box<dyn std::error::Error>,
    context: String,
}

impl std::error::Error for ErrorWithContext {}

impl std::fmt::Display for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.context, self.inner)
    }
}

pub trait ErrorContextAdd {
    fn add_context(self, context: String) -> ErrorWithContext;
}

impl<E: std::error::Error + 'static> ErrorContextAdd for E {
    fn add_context(self, context: String) -> ErrorWithContext {
        return add_context(self, context);
    }
}

pub fn add_context<E>(inner: E, context: String) -> ErrorWithContext
where
    E: std::error::Error + 'static,
{
    ErrorWithContext {
        inner: Box::new(inner),
        context,
    }
}
