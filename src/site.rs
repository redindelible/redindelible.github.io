use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;
use std::fs;

use crate::page::{generate_mdx, Page};
use crate::page::IsArticle;
use crate::page::IsSeriesArticle;


pub struct Site {
    pages_collection: Vec<Rc<dyn Page>>,
    articles: Vec<Rc<dyn IsArticle>>,
    series_articles: HashMap<String, Vec<Rc<dyn IsSeriesArticle>>>
}

impl Site {
    pub fn new() -> Site {
        Site {
            pages_collection: Vec::new(),
            articles: Vec::new(),
            series_articles: HashMap::new()
        }
    }

    pub fn add_mdx(&mut self, path: impl AsRef<Path>) {
        generate_mdx(path).add_to_site(self)
    }

    pub fn add_page<P: Page + 'static>(&mut self, page: P) {
        self.pages_collection.push(Rc::new(page))
    }

    pub fn add_article<A: IsArticle + 'static>(&mut self, article: A) {
        let boxed_a: Rc<A> = Rc::new(article);
        let boxed_article: Rc<dyn IsArticle> = boxed_a.clone();
        let boxed_page: Rc<dyn Page> = boxed_a.clone();
        self.pages_collection.push(boxed_page);
        self.articles.push(boxed_article);
    }

    pub fn add_series_article<A: IsSeriesArticle + 'static>(&mut self, article: A) {
        let series_name = article.series();

        let boxed_a: Rc<A> = Rc::new(article);
        self.pages_collection.push(boxed_a.clone());
        self.articles.push(boxed_a.clone());

        if let Some(series) = self.series_articles.get_mut(&series_name) {
            series.push(boxed_a.clone());
        } else {
            self.series_articles.insert(series_name, vec![boxed_a.clone()]);
        }
    }

    pub fn pages(&self) -> impl Iterator<Item=&Rc<dyn Page>> {
        self.pages_collection.iter()
    }

    pub fn articles(&self) -> impl Iterator<Item=&Rc<dyn IsArticle>> {
        self.articles.iter()
    }

    pub fn series(&self, series: &str) -> impl Iterator<Item=&Rc<dyn IsSeriesArticle>> {
        self.series_articles[series].iter()
    }

    pub fn render_pages(&self, target_dir: &Path) {
        for page in self.pages() {
            let text = page.render(self);
            let to_path = target_dir.join(page.path().as_path());
            fs::create_dir_all(to_path.parent().unwrap()).unwrap();

            fs::write(to_path, text).unwrap();
        }
    }
}
