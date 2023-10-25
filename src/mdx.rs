use std::fmt::{Debug, Display};
use regex::{Captures, Regex};

pub trait Element : Debug {
    fn pattern() -> Regex where Self: Sized;
    fn create(captures: Captures) -> Self where Self: Sized;
}


#[derive(Debug)]
pub struct MDXFile {
    elements: Vec<Box<dyn Element>>
}


#[derive(Debug)]
pub struct CodeBlock {
    code: String
}

impl Element for CodeBlock {
    fn pattern() -> Regex {
        Regex::new(r"^```((?:\\`|.|\n)*)```").unwrap()
    }

    fn create(captures: Captures) -> Self {
        let (_, [code]) = captures.extract();
        CodeBlock { code: code.to_string() }
    }
}

#[derive(Debug)]
pub struct Heading {
    heading_level: usize,
    heading: String
}

impl Element for Heading {
    fn pattern() -> Regex where Self: Sized {
        Regex::new(r"^(#+)\s*(.*)").unwrap()
    }

    fn create(captures: Captures) -> Self where Self: Sized {
        let (_, [level, heading]) = captures.extract();
        Heading { heading_level: level.len(), heading: heading.to_string() }
    }
}

#[derive(Debug)]
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
