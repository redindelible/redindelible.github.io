use std::fmt::{Debug, Display};
use askama::Template;
use regex::{Captures, Regex};

pub trait Element : Debug {
    fn pattern() -> Regex where Self: Sized;
    fn create(captures: Captures) -> Self where Self: Sized;

    fn render(&self) -> String;
}


#[derive(Debug)]
pub struct MDXFile {
    elements: Vec<Box<dyn Element>>
}

impl MDXFile {
    pub fn render(&self) -> String {
        let mut s = String::new();
        for elem in &self.elements {
            s.push_str(&elem.render());
        }
        s
    }
}


#[derive(Debug, Template)]
#[template(path="template_code_block.html")]
pub struct CodeBlock {
    source: Option<String>,
    code: String
}

impl Element for CodeBlock {
    fn pattern() -> Regex {
        Regex::new(r"^```((?:\\`|.|\n)*)```").unwrap()
    }

    fn create(captures: Captures) -> Self {
        let (_, [code]) = captures.extract();
        CodeBlock { source: None, code: code.to_string() }
    }

    fn render(&self) -> String {
        Template::render(self).unwrap()
    }
}

#[derive(Debug,Template)]
#[template(path="template_heading.html")]
pub struct Heading {
    level: usize,
    id: String,
    heading: String
}

impl Element for Heading {
    fn pattern() -> Regex where Self: Sized {
        Regex::new(r"^(#+)\s*(.*)").unwrap()
    }

    fn create(captures: Captures) -> Self where Self: Sized {
        let (_, [level, heading]) = captures.extract();
        let id: String = heading.chars().filter_map(|c| {
            match c {
                c if c.is_alphanumeric() => Some(c),
                c if c.is_whitespace() => Some('-'),
                _ => None
            }
        }).collect();
        Heading { level: level.len(), heading: heading.to_string(), id }
    }

    fn render(&self) -> String {
        Template::render(self).unwrap()
    }
}

#[derive(Debug, Template)]
#[template(path="template_paragraph.html")]
pub struct Paragraph {
    text: String
}

impl Element for Paragraph {
    fn pattern() -> Regex where Self: Sized {
        Regex::new(r"^.+(?:\n.+)*").unwrap()
    }

    fn create(captures: Captures) -> Self where Self: Sized {
        let (text, []) = captures.extract();
        Paragraph { text: text.replace("\n", " ") }
    }

    fn render(&self) -> String {
        Template::render(self).unwrap()
    }
}


pub struct Parser {
    element_types: Vec<(Regex, fn(Captures) -> Box<dyn Element>)>
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            element_types: Vec::new()
        }
    }

    pub fn add_element<E: Element + 'static>(&mut self) -> &mut Self {
        self.element_types.push((E::pattern(), |s| Box::new(E::create(s))));
        self
    }

    pub fn parse(&self, text: &str) -> MDXFile {
        let text = text.replace("\r\n", "\n");
        let mut text: &str = &text;
        let mut elements: Vec<Box<dyn Element>> = Vec::new();
        while !text.is_empty() {
            for (regex, create) in &self.element_types {
                if let Some(captures) = regex.captures_at(text, 0) {
                    text = &text[captures.get(0).unwrap().len()..];
                    elements.push(create(captures));
                    continue;
                }
            }
            let mut c = text.chars();
            c.next();
            text = c.as_str();
        }
        MDXFile { elements }
    }
}
