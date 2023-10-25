use serde_json::Value as JSONValue;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use std::fs;
use chrono::NaiveDate;

use crate::index::Index;
use crate::link::Link;
use crate::series_article::SeriesArticle;
use crate::site::Site;

pub trait Page {
    fn from_metadata(metadata: JSONValue, text: &str) -> Option<Self> where Self: Sized;
    fn add_to_site(self: Box<Self>, site: &mut Site);

    fn path(&self) -> Link;
    fn render(&self, site: &Site) -> String;
}

pub trait IsArticle: Page {
    fn title(&self) -> String;
    fn date(&self) -> NaiveDate;
    fn preview(&self) -> String;
}

pub trait IsSeriesArticle: IsArticle {
    fn series(&self) -> String;
    fn number(&self) -> u32;
}

const HEADER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)^%\{.*}%").unwrap()
});

pub fn generate_mdx(path: impl AsRef<Path>) -> Box<dyn Page> {
    let text = fs::read_to_string(path).unwrap();

    let mat = HEADER_PATTERN.find(&text).unwrap();

    let json_contents = &mat.as_str()[1..mat.len()-1];
    let json: JSONValue = serde_json::from_str(json_contents).unwrap();
    let rest = &text[mat.end()..];

    let typ = json.get("type").unwrap().as_str().unwrap();
    match typ {
        "index" => {
            Box::new(Index::from_metadata(json, rest).unwrap())
        }
        "series-article" => {
            Box::new(SeriesArticle::from_metadata(json, rest).unwrap())
        }
        _ => { panic!("{}", typ); }
    }
}
