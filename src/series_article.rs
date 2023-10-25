use serde_json::Value as JSONValue;
use chrono::NaiveDate;
use askama::Template;

use crate::link::Link;
use crate::mdx::{CodeBlock, Heading, MDXFile, Paragraph, Parser};
use crate::page::{IsArticle, IsSeriesArticle};
use crate::page::Page;
use crate::site::Site;

#[derive(Template)]
#[template(path="template_series_article.html", escape="none")]
struct SeriesArticleTemplate<'a> {
    article: &'a SeriesArticle,
    series: Vec<&'a dyn IsSeriesArticle>
}

pub struct SeriesArticle {
    series_name: String,
    number: u32,

    title: String,
    date: NaiveDate,
    preview: String,

    elements: MDXFile
}

fn first_n(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

impl Page for SeriesArticle {
    fn from_metadata(metadata: JSONValue, text: &str) -> Option<Self> where Self: Sized {
        let title = metadata.get("title")?.as_str()?.to_string();
        let date = NaiveDate::parse_from_str(metadata.get("date")?.as_str()?, "%b %d, %Y").unwrap();
        let data = metadata.get("series")?;
        let series_name = data.get("series_name")?.as_str()?.to_string();
        let number = data.get("number")?.as_u64()? as u32;

        let elements= Parser::new()
            .add_element::<CodeBlock>()
            .add_element::<Heading>()
            .add_element::<Paragraph>()
            .parse(text);
        dbg!(&elements);
        Some(SeriesArticle { series_name, number, title, date, preview: "".to_string(), elements })
    }

    fn add_to_site(self: Box<Self>, site: &mut Site) {
        site.add_series_article(*self);
    }

    fn path(&self) -> Link {
        Link::new(["articles".to_owned(), first_n(&self.title, 20).trim().to_lowercase().replace(' ', "-") + ".html"])
    }

    fn render(&self, site: &Site) -> String {
        let series: Vec<&dyn IsSeriesArticle> = site.series(&self.series_name).map(|p| p.as_ref()).collect();
        SeriesArticleTemplate {
            article: self,
            series
        }.render().unwrap()
    }
}

impl IsArticle for SeriesArticle {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn date(&self) -> NaiveDate {
        self.date
    }

    fn preview(&self) -> String {
        "".to_owned()
    }
}

impl IsSeriesArticle for SeriesArticle {
    fn series(&self) -> String {
        self.series_name.clone()
    }

    fn number(&self) -> u32 {
        self.number
    }
}
