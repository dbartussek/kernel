#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Config {
    Debug,
    Release,
}

impl Config {
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl Into<&'static str> for Config {
    fn into(self) -> &'static str {
        match self {
            Config::Debug => "debug",
            Config::Release => "release",
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::Debug
    }
}
