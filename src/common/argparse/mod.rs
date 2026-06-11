mod arglex;

use arglex::{ArgLexer, ArgToken};
pub struct ArgParser<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    inner: ArgLexer<'a, T>,
    long_arg: Option<&'a str>,
}

impl<'a, T> ArgParser<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    pub fn new(args: T) -> Self {
        ArgParser {
            inner: ArgLexer::new(args),
            long_arg: None,
        }
    }

    pub fn consume_arg(&mut self) -> Result<&'a str, &'static str> {
        if let Some(arg) = self.long_arg.take() {
            Ok(arg)
        } else {
            self.inner.consume_arg().ok_or("expected argument")
        }
    }

    pub fn read(&mut self) -> Result<Option<Arg<'a>>, &'static str> {
        if self.long_arg.is_some() {
            return Err("expected argument");
        }

        let Some(token) = self.inner.next() else {
            return Ok(None);
        };

        match token {
            ArgToken::Short(c) => Ok(Some(Arg::Short(c))),
            ArgToken::Long(s) => Ok(Some(Arg::Long(s))),
            ArgToken::Positional(s) => Ok(Some(Arg::Positional(s))),
            ArgToken::LongArg(param, arg) => {
                self.long_arg = Some(arg);
                Ok(Some(Arg::Long(param)))
            }
        }
    }
}

pub enum Arg<'a> {
    Short(char),
    Long(&'a str),
    Positional(&'a str),
}
