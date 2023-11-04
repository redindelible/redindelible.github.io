use serde_json::Value as JSONValue;
use askama::Template;
use crate::link::Link;
use crate::page::{IsArticle, Page};
use crate::site::Site;


#[derive(Template)]
#[template(path="template_index.html")]
struct IndexTemplate<'a> {
    articles: Vec<&'a dyn IsArticle>
}

pub struct Index;

impl Page for Index {
    fn from_metadata(_metadata: JSONValue, _text: &str) -> Option<Self> where Self: Sized {
        Some(Index)
    }

    fn add_to_site(self: Box<Self>, site: &mut Site) {
        site.add_page(*self);
    }

    fn path(&self) -> Link {
        Link::new(["index.html"])
    }

    fn render(&self, site: &Site) -> String {
        let mut articles: Vec<&dyn IsArticle> = site.articles().map(|p| p.as_ref()).collect();
        articles.sort_unstable_by_key(|article| std::cmp::Reverse(article.date()));
        IndexTemplate { articles }.render().unwrap()
    }
}
