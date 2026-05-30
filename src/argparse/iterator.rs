pub struct ArgIterator<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    args: std::iter::Peekable<T>,
    remaining: Option<std::str::Chars<'a>>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Arg<'a> {
    Short(char),
    Long(&'a str),
    LongArg(&'a str, &'a str),
    Positional(&'a str),
}

impl<'a, T> ArgIterator<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    pub fn new(args: T) -> Self {
        return ArgIterator {
            args: args.peekable(),
            remaining: None,
        };
    }
    pub fn consume_arg(&mut self) -> Option<&str> {
        if let Some(arg_chars) = self.remaining.take() {
            let arg = arg_chars.as_str();
            if arg != "" {
                return Some(arg);
            }
        }

        if self.args.peek().is_none() {
            None
        } else {
            self.args.next()
        }
    }
}

impl<'a, T> std::iter::Iterator for ArgIterator<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    type Item = Arg<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(remaining) = &mut self.remaining {
            if let Some(c) = remaining.next() {
                return Some(Arg::Short(c));
            }
        }
        Some(match self.args.next()?.as_bytes() {
            b"" => Arg::Positional(""),
            s if s == b"-" || s == b"--" => {
                Arg::Positional(unsafe { std::str::from_utf8_unchecked(s) })
            }
            [b'-', b'-', rest @ ..] => {
                let s = unsafe { std::str::from_utf8_unchecked(rest) };

                if let Some((param, arg)) = s.split_once('=') {
                    Arg::LongArg(param, arg)
                } else {
                    Arg::Long(s)
                }
            }
            [b'-', rest @ ..] => {
                let mut chars = unsafe { std::str::from_utf8_unchecked(rest) }
                    .chars()
                    .into_iter();

                let c = chars.next().expect("already checked");
                self.remaining = Some(chars);
                Arg::Short(c)
            }

            s => Arg::Positional(unsafe { std::str::from_utf8_unchecked(s) }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let args = vec![];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positional() {
        let args = vec!["hello, my darling"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Positional("hello, my darling"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positionals() {
        let args = vec!["hello", "my", "darling"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Positional("hello"), iter.next().unwrap());
        assert_eq!(Arg::Positional("my"), iter.next().unwrap());
        assert_eq!(Arg::Positional("darling"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn single_dash_is_positional() {
        let args = vec!["-"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Positional("-"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn double_dash_is_positional() {
        let args = vec!["--"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Positional("--"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn short() {
        let args = vec!["-a"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Short('a'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn shorts_together() {
        let args = vec!["-abcd"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Short('a'), iter.next().unwrap());
        assert_eq!(Arg::Short('b'), iter.next().unwrap());
        assert_eq!(Arg::Short('c'), iter.next().unwrap());
        assert_eq!(Arg::Short('d'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn shorts_separate() {
        let args = vec!["-a", "-b", "-c", "-d"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Short('a'), iter.next().unwrap());
        assert_eq!(Arg::Short('b'), iter.next().unwrap());
        assert_eq!(Arg::Short('c'), iter.next().unwrap());
        assert_eq!(Arg::Short('d'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long() {
        let args = vec!["--long-boy"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Long("long-boy"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn longs() {
        let args = vec!["--long-boy", "--long boy", "--loooonger-boy"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::Long("long-boy"), iter.next().unwrap());
        assert_eq!(Arg::Long("long boy"), iter.next().unwrap());
        assert_eq!(Arg::Long("loooonger-boy"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long_arg_together() {
        let args = vec!["--long-boy=Waluigi"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::LongArg("long-boy", "Waluigi"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long_args_together() {
        let args = vec!["--fat-boy=Wario", "--attack=mario"];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(Arg::LongArg("fat-boy", "Wario"), iter.next().unwrap());
        assert_eq!(Arg::LongArg("attack", "mario"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn all_one_word_types() {
        let args = vec!["-ab", "pos", "--fat-boy", "--attack=mario"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!(Arg::Short('a'), iter.next().unwrap());
        assert_eq!(Arg::Short('b'), iter.next().unwrap());
        assert_eq!(Arg::Positional("pos"), iter.next().unwrap());
        assert_eq!(Arg::Long("fat-boy"), iter.next().unwrap());
        assert_eq!(Arg::LongArg("attack", "mario"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_arg_on_empty_stays_none() {
        let args = vec![];
        let mut iter = ArgIterator::new(args.into_iter());
        assert_eq!(None, iter.consume_arg());
        assert_eq!(None, iter.consume_arg());
        assert_eq!(iter.consume_arg(), None)
    }

    #[test]
    fn consume_single_arg() {
        let args = vec!["hamburger"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!("hamburger", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_intra_word_arg_after_one_short() {
        let args = vec!["-ibackup"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!(Arg::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_intra_word_arg_after_many_shorts() {
        let args = vec!["-vabibackup"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!(Arg::Short('v'), iter.next().unwrap());
        assert_eq!(Arg::Short('a'), iter.next().unwrap());
        assert_eq!(Arg::Short('b'), iter.next().unwrap());
        assert_eq!(Arg::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positional_after_consumed() {
        let args = vec!["-ibackup", "file", "--argname", "arg", "file2"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!(Arg::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(Arg::Positional("file"), iter.next().unwrap());
        assert_eq!(Arg::Long("argname"), iter.next().unwrap());
        assert_eq!("arg", iter.consume_arg().unwrap());
        assert_eq!(Arg::Positional("file2"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_arg_starting_with_dash_or_dashes() {
        let args = vec!["-backup", "--arg"];
        let mut iter = ArgIterator::new(args.into_iter());

        assert_eq!("-backup", iter.consume_arg().unwrap());
        assert_eq!("--arg", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }
}
