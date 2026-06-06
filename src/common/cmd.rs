pub struct MainError(Box<dyn std::error::Error>);

impl<E> From<E> for MainError
where
    E: Into<Box<dyn std::error::Error>>,
{
    fn from(value: E) -> Self {
        MainError(value.into())
    }
}

impl std::fmt::Debug for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type MainResult = Result<(), MainError>;
