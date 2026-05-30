mod iterator;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct FlagId(usize);

impl FlagId {
    fn assign(&mut self) -> FlagId {
        let flag = FlagId(self.0);
        self.0 += 1;
        flag
    }
}

pub struct FlagSet<'a> {
    help: Option<&'a str>,
    next_flag_id: FlagId,
    short_to_id: std::collections::HashMap<char, FlagId>,
    flag_id_with_arg: std::collections::HashSet<FlagId>,
}

impl<'a> FlagSet<'a> {
    pub fn new() -> FlagSet<'a> {
        return FlagSet {
            help: None,
            next_flag_id: FlagId(0),
            short_to_id: std::collections::HashMap::new(),
            flag_id_with_arg: std::collections::HashSet::new(),
        };
    }

    pub fn bool(&mut self, short: Option<char>, long: Option<&'a str>) -> BoolFlag {
        if short.is_none() && long.is_none() {
            panic!("A flag must have either a short or long argument.")
        }

        if long.is_some() {
            unimplemented!("long options not yet supported")
        }

        let flag = BoolFlag {
            id: self.next_flag_id.assign(),
        };

        if let Some(c) = short {
            if self.short_to_id.insert(c, flag.id).is_some() {
                panic!("flag '{}' is already used.", c)
            }
        }
        return flag;
    }
    pub fn arg<T, U>(&mut self, short: Option<u8>, long: Option<&'a str>) -> Flag<T> {
        if short.is_none() && long.is_none() {
            panic!("A flag must have either a short or long argument.")
        }

        let flag = Flag {
            value: None,
            id: self.next_flag_id.assign(),
        };

        return flag;
    }

    pub fn parse<T>(self, args: T) -> Result<ParsedFlagSet, ()>
    where
        T: std::iter::Iterator<Item = String>,
    {
        let bool_flags = std::collections::HashSet::<BoolFlag>::new();
        for arg in args {
            if arg.starts_with('-') && arg.len() > 1 {
                if arg.starts_with("--") && arg.len() > 2 {
                    unimplemented!()
                } else {
                }
            }
        }
        todo!()
    }
}

pub struct ParsedFlagSet {
    bool_flags: std::collections::HashSet<BoolFlag>,
}

impl ParsedFlagSet {
    pub fn get_bool(&self, flag: BoolFlag) -> bool {
        self.bool_flags.contains(&flag)
    }
    pub fn get<T>(&self, flag: Flag<T>) -> Option<T> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct BoolFlag {
    id: FlagId,
}

pub struct Flag<T> {
    value: Option<T>,
    id: FlagId,
}
