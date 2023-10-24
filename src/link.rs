use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

#[derive(Clone)]
pub struct Link {
    parts: Vec<String>
}

impl Link {
    pub fn new<I, S>(parts: I) -> Link where I: IntoIterator<Item=S>, S: Into<String> {
        Link { parts: parts.into_iter().map(|item| item.into()).collect() }
    }

    pub fn as_path(&self) -> PathBuf {
        self.clone().into()
    }

    pub fn to_url(&self) -> String {
        self.into()
    }
}

impl From<Link> for PathBuf {
    fn from(value: Link) -> Self {
        PathBuf::from_iter(value.parts.into_iter())
    }
}

impl From<&Link> for String {
    fn from(value: &Link) -> Self {
        value.parts.join("/")
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_url())
    }
}
