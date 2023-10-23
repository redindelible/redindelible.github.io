mod site;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use askama::Template;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value as JSONValue;
use chrono::NaiveDate;
use slotmap::{new_key_type, SlotMap};

const SITE_DIR: &'static str = "_site";
const OUTPUT_DIR: &'static str = "docs";


const HEADER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)^%\{.*}%").unwrap()
});

#[derive(Clone)]
struct Link {
    parts: Vec<String>
}

impl Link {
    fn new<I, S>(parts: I) -> Link where I: IntoIterator<Item=S>, S: Into<String> {
        Link { parts: parts.into_iter().map(|item| item.into()).collect() }
    }

    fn as_path(&self) -> PathBuf {
        self.clone().into()
    }

    fn to_url(&self) -> String {
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

struct ArticleInfo {
    title: String,
    date: NaiveDate,
    path: String,
    preview: String
}

struct SeriesArticleInfo {
    article_info: ArticleInfo,
    series: String,
    number: u64
}

trait Page {
    fn from_metadata(metadata: JSONValue) -> Option<Self> where Self: Sized;

    fn article_info(&self) -> Option<ArticleInfo> {
        None
    }

    fn series_article_info(&self) -> Option<SeriesArticleInfo> {
        None
    }

    fn path(&self) -> Link;
    fn render(&self, site: &Site) -> String;
}

new_key_type! {
    struct PageKey;
}

struct Site {
    pages: Vec<Box<dyn Page>>
}

impl Site {
    fn new() -> Site {
        Site {
            pages: Vec::new()
        }
    }

    fn add_page(&mut self, page: Box<dyn Page>) {
        self.pages.push(page)
    }

    fn iter_articles(&self) -> impl Iterator<Item=ArticleInfo> + '_ {
        self.pages.iter().filter_map(|page| page.article_info())
    }

    fn iter_series_articles<'a>(&'a self, series: &'a str) -> impl Iterator<Item=SeriesArticleInfo> + 'a {
        self.pages.iter().filter_map(|page| page.series_article_info()).filter(move |info| info.series == series)
    }

    fn render_pages(&self) {
        for page in &self.pages {
            let text = page.render(self);
            let to_path = Path::new(OUTPUT_DIR).join(page.path().as_path());
            fs::create_dir_all(to_path.parent().unwrap()).unwrap();

            fs::write(to_path, text).unwrap();
        }
    }
}

struct SeriesArticle {
    series_name: String,
    number: u64,

    title: String,
    date: NaiveDate,
    preview: String,
}

fn first_n(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

impl Page for SeriesArticle {
    fn from_metadata(metadata: JSONValue) -> Option<Self> where Self: Sized {
        let title = metadata.get("title")?.as_str()?.to_string();
        let date = NaiveDate::parse_from_str(metadata.get("date")?.as_str()?, "%b %d, %Y").unwrap();
        let data = metadata.get("series")?;
        let series_name = data.get("series_name")?.as_str()?.to_string();
        let number = data.get("number")?.as_u64()?;
        Some(SeriesArticle { series_name, number, title, date, preview: "".to_string() })
    }

    fn article_info(&self) -> Option<ArticleInfo> {
        Some(ArticleInfo {
            title: self.title.clone(),
            date: self.date,
            path: self.path().to_url(),
            preview: self.preview.clone()
        })
    }

    fn series_article_info(&self) -> Option<SeriesArticleInfo> {
        Some(SeriesArticleInfo {
            article_info: self.article_info().unwrap(),
            series: self.series_name.to_owned(),
            number: self.number
        })
    }

    fn path(&self) -> Link {
        Link::new(["articles".to_owned(), first_n(&self.title, 20).trim().to_lowercase().replace(' ', "-") + ".html"])
    }

    fn render(&self, site: &Site) -> String {
        let series: Vec<SeriesArticleInfo> = site.iter_series_articles(&self.series_name).collect();
        SeriesArticleTemplate {
            article: self,
            series
        }.render().unwrap()
    }
}


struct Index;
impl Page for Index {
    fn from_metadata(metadata: JSONValue) -> Option<Self> where Self: Sized {
        Some(Index)
    }

    fn path(&self) -> Link {
        Link::new(["index.html"])
    }

    fn render(&self, site: &Site) -> String {
        let mut articles: Vec<ArticleInfo> = site.iter_articles().collect();
        articles.sort_by_key(|article| article.date);
        IndexTemplate { articles }.render().unwrap()
    }
}

#[derive(Template)]
#[template(path="template_index.html")]
struct IndexTemplate {
    articles: Vec<ArticleInfo>
}

#[derive(Template)]
#[template(path="template_series_article.html")]
struct SeriesArticleTemplate<'a> {
    article: &'a SeriesArticle,
    series: Vec<SeriesArticleInfo>
}

fn generate_mdx(path: impl AsRef<Path>) -> Box<dyn Page> {
    let text = fs::read_to_string(path).unwrap();

    let mat = HEADER_PATTERN.find(&text).unwrap();

    let json_contents = &mat.as_str()[1..mat.len()-1];
    let json: JSONValue = serde_json::from_str(json_contents).unwrap();
    let rest = &text[mat.end()..];

    let typ = json.get("type").unwrap().as_str().unwrap();
    match typ {
        "index" => {
            Box::new(Index::from_metadata(json).unwrap())
        }
        "article" => {
            Box::new(SeriesArticle::from_metadata(json).unwrap())
        }
        _ => { panic!("{}", typ); }
    }
}

fn generate() {
    for item in fs::read_dir(OUTPUT_DIR).unwrap() {
        let p = item.unwrap().path();
        if p.is_dir() {
            fs::remove_dir(p).unwrap();
        } else {
            fs::remove_file(p).unwrap();
        }
    }

    let mut site = Site::new();
    for item in fs::read_dir(SITE_DIR).unwrap() {
        let path = item.unwrap().path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let ext = path.extension().and_then(|e| e.to_str());
            match ext {
                Some("mdx") => {
                    site.add_page(generate_mdx(path))
                },
                _ => {
                    let copy_path = Path::new(OUTPUT_DIR).join(file_name);
                    fs::copy(path, copy_path).unwrap();
                }
            }
        }
    }

    site.render_pages()
}

fn main() {
    generate();
}