pub struct ArgLexer<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    args: std::iter::Peekable<T>,
    remaining: Option<std::str::Chars<'a>>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ArgToken<'a> {
    Short(char),
    Long(&'a str),
    LongArg(&'a str, &'a str),
    Positional(&'a str),
}

impl<'a, T> ArgLexer<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    pub fn new(args: T) -> Self {
        return ArgLexer {
            args: args.peekable(),
            remaining: None,
        };
    }
    pub fn consume_arg(&mut self) -> Option<&'a str> {
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

impl<'a, T> std::iter::Iterator for ArgLexer<'a, T>
where
    T: std::iter::Iterator<Item = &'a str>,
{
    type Item = ArgToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut remaining) = self.remaining.take() {
            if let Some(c) = remaining.next() {
                self.remaining = Some(remaining);
                return Some(ArgToken::Short(c));
            }
        }
        Some(match self.args.next()?.as_bytes() {
            b"" => ArgToken::Positional(""),
            s if s == b"-" || s == b"--" => {
                ArgToken::Positional(unsafe { std::str::from_utf8_unchecked(s) })
            }
            [b'-', b'-', rest @ ..] => {
                let s = unsafe { std::str::from_utf8_unchecked(rest) };

                if let Some((param, arg)) = s.split_once('=') {
                    ArgToken::LongArg(param, arg)
                } else {
                    ArgToken::Long(s)
                }
            }
            [b'-', rest @ ..] => {
                let mut chars = unsafe { std::str::from_utf8_unchecked(rest) }
                    .chars()
                    .into_iter();

                let c = chars.next().expect("already checked");
                self.remaining = Some(chars);
                ArgToken::Short(c)
            }

            s => ArgToken::Positional(unsafe { std::str::from_utf8_unchecked(s) }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let args = vec![];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positional() {
        let args = vec!["hello, my darling"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(
            ArgToken::Positional("hello, my darling"),
            iter.next().unwrap()
        );
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positionals() {
        let args = vec!["hello", "my", "darling"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Positional("hello"), iter.next().unwrap());
        assert_eq!(ArgToken::Positional("my"), iter.next().unwrap());
        assert_eq!(ArgToken::Positional("darling"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn single_dash_is_positional() {
        let args = vec!["-"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Positional("-"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn double_dash_is_positional() {
        let args = vec!["--"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Positional("--"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn short() {
        let args = vec!["-a"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Short('a'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn shorts_together() {
        let args = vec!["-abcd"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Short('a'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('b'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('c'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('d'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn shorts_separate() {
        let args = vec!["-a", "-b", "-c", "-d"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Short('a'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('b'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('c'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('d'), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long() {
        let args = vec!["--long-boy"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Long("long-boy"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn longs() {
        let args = vec!["--long-boy", "--long boy", "--loooonger-boy"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::Long("long-boy"), iter.next().unwrap());
        assert_eq!(ArgToken::Long("long boy"), iter.next().unwrap());
        assert_eq!(ArgToken::Long("loooonger-boy"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long_arg_together() {
        let args = vec!["--long-boy=Waluigi"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(
            ArgToken::LongArg("long-boy", "Waluigi"),
            iter.next().unwrap()
        );
        assert_eq!(None, iter.next())
    }

    #[test]
    fn long_args_together() {
        let args = vec!["--fat-boy=Wario", "--attack=mario"];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(ArgToken::LongArg("fat-boy", "Wario"), iter.next().unwrap());
        assert_eq!(ArgToken::LongArg("attack", "mario"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn all_one_word_types() {
        let args = vec!["-ab", "pos", "--fat-boy", "--attack=mario"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!(ArgToken::Short('a'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('b'), iter.next().unwrap());
        assert_eq!(ArgToken::Positional("pos"), iter.next().unwrap());
        assert_eq!(ArgToken::Long("fat-boy"), iter.next().unwrap());
        assert_eq!(ArgToken::LongArg("attack", "mario"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_arg_on_empty_stays_none() {
        let args = vec![];
        let mut iter = ArgLexer::new(args.into_iter());
        assert_eq!(None, iter.consume_arg());
        assert_eq!(None, iter.consume_arg());
        assert_eq!(iter.consume_arg(), None)
    }

    #[test]
    fn consume_single_arg() {
        let args = vec!["hamburger"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!("hamburger", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_intra_word_arg_after_one_short() {
        let args = vec!["-ibackup"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!(ArgToken::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_intra_word_arg_after_many_shorts() {
        let args = vec!["-vabibackup"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!(ArgToken::Short('v'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('a'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('b'), iter.next().unwrap());
        assert_eq!(ArgToken::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn positional_after_consumed() {
        let args = vec!["-ibackup", "file", "--argname", "arg", "file2"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!(ArgToken::Short('i'), iter.next().unwrap());
        assert_eq!("backup", iter.consume_arg().unwrap());
        assert_eq!(ArgToken::Positional("file"), iter.next().unwrap());
        assert_eq!(ArgToken::Long("argname"), iter.next().unwrap());
        assert_eq!("arg", iter.consume_arg().unwrap());
        assert_eq!(ArgToken::Positional("file2"), iter.next().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn consume_arg_starting_with_dash_or_dashes() {
        let args = vec!["-backup", "--arg"];
        let mut iter = ArgLexer::new(args.into_iter());

        assert_eq!("-backup", iter.consume_arg().unwrap());
        assert_eq!("--arg", iter.consume_arg().unwrap());
        assert_eq!(None, iter.next())
    }

    #[test]
    fn example() {
        let args = vec![
            "-an8",
            "--color",
            "blue",
            "--context=3",
            "-b",
            "s/a/b/g",
            "file",
        ];
        let mut iter = ArgLexer::new(args.into_iter());
        let mut a = false;
        let mut b = false;
        let mut c = false;
        let mut n = "";
        let mut color = "";
        let mut context = "";
        let mut pos = vec![];
        while let Some(arg) = iter.next() {
            use ArgToken::*;
            match arg {
                Short('a') => a = true,
                Short('b') => b = true,
                Short('c') => c = true,
                Short('n') => n = iter.consume_arg().unwrap(),

                Long("color") => color = iter.consume_arg().unwrap(),
                LongArg("context", arg) => context = arg,
                Positional(p) => pos.push(p),
                _ => unimplemented!(),
            }
        }
        assert_eq!(true, a);
        assert_eq!(true, b);
        assert_eq!(false, c);
        assert_eq!("8", n);
        assert_eq!("blue", color);
        assert_eq!("3", context);
        let want: Vec<_> = vec!["s/a/b/g", "file"].iter().map(|x| *x).collect();
        let got: Vec<_> = pos.iter().map(|x| *x).collect();
        assert_eq!(want, got);
    }
}
