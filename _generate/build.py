from __future__ import annotations

import re
import json
import argparse
from datetime import datetime
from pathlib import Path
from typing import Type


def compact_html(s: str) -> str:
    return "".join(line.strip() for line in s.splitlines())


class Generator:
    METADATA_PATTERN = re.compile(r"^%({(?:.|\s)*?})%")

    def __init__(self, source_dir: Path, target_dir: Path):
        self.source_dir: source_dir = source_dir
        self.target_dir: Path = target_dir

        self.site: Site = Site()
        for page in self.parse_dir(self.source_dir):
            self.site.add_page(page)

    def parse_dir(self, from_dir: Path) -> list[Page]:
        pages = []
        for item in from_dir.iterdir():
            if item.is_dir():
                pages.extend(self.parse_dir(item))
            elif item.suffix == ".mdx":
                if page := self.parse_file(item):
                    pages.append(page)
        return pages

    def parse_file(self, path: Path) -> Page | None:
        raw_text = path.read_text()
        if metadata_match := self.METADATA_PATTERN.match(raw_text):
            metadata = json.loads(metadata_match.group(1))
            raw_text = raw_text[metadata_match.end():]
            return Page.create(metadata, raw_text)

    def render(self):
        def remove_tree(dir: Path):
            if dir.is_dir():
                for path in dir.iterdir():
                    remove_tree(path)
                dir.rmdir()
            else:
                dir.unlink()

        def copy_tree(dir: Path, target: Path):
            for it in dir.iterdir():
                new_item = target / it.relative_to(dir)
                if it.is_dir():
                    copy_tree(it, new_item)
                else:
                    if new_item.suffix != ".mdx":
                        new_item.parent.mkdir(parents=True, exist_ok=True)
                        new_item.write_bytes(it.read_bytes())

        for item in self.target_dir.iterdir():
            if item.stem[0] not in "._":
                remove_tree(item)

        copy_tree(self.source_dir, self.target_dir)

        self.site.render(self.target_dir)


class Site:
    def __init__(self):
        self.all_series: dict[str, list[ArticleSeries]] = {}
        self.pages: list[Page] = []

        self.counter: int = 0

    def add_page(self, page: Page):
        if isinstance(page, ArticleSeries):
            self.all_series.setdefault(page.series_name, []).append(page)
        self.pages.append(page)

    def next_number(self) -> int:
        self.counter += 1
        return self.counter

    def render(self, target: Path):
        for page in self.pages:
            page_path = Path(str(target) + str(page.link()))
            page_path.parent.mkdir(parents=True, exist_ok=True)
            page_path.write_text(page.render(self))


class Page:
    page_types: dict[str, Type[Page]] = {}

    def __init_subclass__(cls, *, register_name: str=None, **kwargs):
        if register_name is None:
            raise Exception()
        Page.page_types[register_name] = cls

    @classmethod
    def create(cls, metadata: dict, text: str):
        return cls.page_types[metadata["type"]]._create(metadata, text)

    @classmethod
    def _create(cls, metadata: dict, text: str) -> Page:
        raise NotImplementedError()

    def link(self) -> Path:
        raise NotImplementedError()

    def render(self, site: Site) -> str:
        raise NotImplementedError()


class Index(Page, register_name="index"):
    card_format = compact_html("""
        <div class="article-card-container">
            <div class="article-card">
                <div class="article-card-link">
                    <a href="{link}">{name}</a>
                </div>
                <div class="article-card-info">
                    {date}
                </div>
                <div class="article-card-preview">
                    {preview}...
                </div>
            </div>
        </div>
    """)

    format = compact_html("""
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>redindelible</title>
            <link href="https://fonts.googleapis.com/css2?family=Roboto&family=Shadows+Into+Light&display=swap" rel="stylesheet">
            <link rel='stylesheet' href='/_site/style.css'>
        </head>
        <body>
            <div class="top-bar">
                <a class="top-title" href="/index.html">
                    <div>redindelible</div>
                </a>
            </div>
            
            <div class="content">
                <div class="main-content-container">
                    <div class="main-content">
                        {rendered_cards}
                        <div class="article-card-end"></div>
                    </div>
                </div>
            </div>
        </body>
        </html>
    """)

    def __init__(self):
        pass

    @classmethod
    def _create(cls, metadata: dict, text: str) -> Page:
        return Index()

    def link(self) -> Path:
        return Path("/index.html")

    def render(self, site: Site) -> str:
        cards = []
        for article in sorted((page for page in site.pages if isinstance(page, Article)), key=lambda art: art.date, reverse=True):
            if isinstance(article, ArticleSeries):
                name = f"{article.title} / Part {article.number} &ndash; {article.article_name}"
            else:
                name = article.title
            cards.append(self.card_format.format(link=article.link(), name=name, date=article.date.strftime("%b %d, %Y"), preview=article.preview()))
        return self.format.format(rendered_cards="".join(cards))


class Article(Page, register_name="article"):
    format = compact_html("""
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>redindelible</title>
            <link href="https://fonts.googleapis.com/css2?family=Roboto&family=Shadows+Into+Light&display=swap" rel="stylesheet">
            <link rel='stylesheet' href='/style.css'>
            <link rel="icon" href="/favicon.png">
        </head>
        <body>
            <div class="top-bar">
                <a class="top-title" href="/index.html">
                    <div>redindelible</div>
                </a>
            </div>
            <div class="content">
                <div class="main-content-container">
                    <div class="main-content">
                        <div class="article-title">
                            {title}
                        </div>
                        <div class="article-date">
                            Published {date}
                        </div>
                        {rendered_content}
                    </div>
                </div>
            </div>
        </body>
        </html>
    """)

    def __init__(self, title: str, date: datetime, elements: list[Element]):
        self.title: str = title
        self.date: datetime = date
        self.elements = elements

    def link(self):
        return Path(f"/articles/{self.title[:20].strip().lower().replace(' ', '-')}-{self.date.strftime('%m-%d-%y')}.html")

    @classmethod
    def _create(cls, metadata: dict, raw_text: str) -> Article:
        title = metadata["title"]
        date = datetime.strptime(metadata["date"], "%b %d, %Y")
        elements = cls.parse_text(raw_text)
        return Article(title, date, elements)

    @staticmethod
    def parse_text(text: str) -> list[Element]:
        text = text.strip()
        i = 0
        elements: list[Element] = []
        while i < len(text):
            if text.startswith("```", i):
                if match := CodeBlock.code_block_pattern.match(text, i):
                    elements.append(CodeBlock(match.group(1), match.group(2).strip()))
                    i = match.end()
                    while i < len(text) and text[i] in " \t\n\r":
                        i += 1
                else:
                    raise Exception()
            else:
                start = i
                while True:
                    i = text.find("\n", i)
                    if i < 0:
                        paragraph_text = text[start:]
                        i = len(text)
                        break
                    elif match := Paragraph.paragraph_break_pattern.match(text, i):
                        paragraph_text = text[start:match.start()]
                        i = match.end()
                        break
                    else:
                        i += 1
                elements.append(Paragraph(paragraph_text.replace("\n", " ")))
        return elements

    def preview(self) -> str:
        for element in self.elements:
            if isinstance(element, Paragraph):
                return element.raw_text[:300].strip()
        else:
            return ""

    def render(self, site: Site) -> str:
        return self.format.format(title=self.title, date=self.date.strftime("%b %d, %Y"),
                           rendered_content="".join(element.render(site) for element in self.elements))


class ArticleSeries(Article, register_name="article-series"):
    left_link_format = compact_html("""
        <div class="left-bar-item">
            <a class="left-bar-item-link" href="{link}">{number}. {article_name}</a>
        </div>
    """)

    format = compact_html("""
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>redindelible</title>
            <link href="https://fonts.googleapis.com/css2?family=Roboto&family=Shadows+Into+Light&display=swap" rel="stylesheet">
            <link rel='stylesheet' href='/style.css'>
            <link rel="icon" href="/favicon.png">
        </head>
        <body>
            <div class="top-bar">
                <a class="top-title" href="/index.html">
                    <div>redindelible</div>
                </a>
            </div>
            
            <div class="content">
                <div class="left-bar">
                    <div class="left-bar-title">
                        {series_name}
                    </div>
                    {rendered_left_links}
                </div>
                <div class="main-content-container">
                    <div class="main-content">
                        <div class="article-title">
                            {title}
                        </div>
                        <div class="article-subtitle">
                            Part {number} &ndash; {article_name}
                        </div>
                        <div class="article-date">
                            Published {date}
                        </div>
                        {rendered_content}
                    </div>
                </div>
            </div>
        </body>
        </html>
    """)

    def __init__(self, title: str, date: datetime, series_name: str, article_name: str, number: int, elements: list[Element]):
        super().__init__(title, date, elements)
        self.series_name: str = series_name
        self.article_name: str = article_name
        self.number = number

    @classmethod
    def _create(cls, metadata: dict, raw_text: str) -> Article:
        title = metadata["title"]
        date = datetime.strptime(metadata["date"], "%b %d, %Y")
        elements = cls.parse_text(raw_text)
        series_name = metadata["series"]["series_name"]
        article_name = metadata["series"]["article_name"]
        number = metadata["series"]["number"]
        return ArticleSeries(title, date, series_name, article_name, number, elements)

    def link(self):
        return Path(f"/articles/{self.title[:20].strip().lower().replace(' ', '-')}-{self.number}-{self.date.strftime('%m-%d-%y')}.html")

    def render(self, site: Site) -> str:
        left_links = []
        for article in sorted(site.all_series[self.series_name], key=lambda art: art.number):
            left_link = self.left_link_format.format(number=article.number, article_name=article.article_name, link=article.link())
            left_links.append(left_link)
        return self.format.format(title=self.title, date=self.date.strftime("%b %d, %Y"), number=self.number,
                                  article_name=self.article_name, series_name=self.series_name, rendered_left_links="".join(left_links),
                                  rendered_content="".join(element.render(site) for element in self.elements))


class Element:
    def render(self, site: Site) -> str:
        raise NotImplementedError()


class Paragraph(Element):
    paragraph_break_pattern = re.compile(r"\n\s*\n")
    note_pattern = re.compile(r"(?<!\\)`@note (.*?)(?<!\\)`")
    inline_code_pattern = re.compile(r"(?<!\\)`(.*?)(?<!\\)`")

    format = compact_html("""
        <div class="article-paragraph">{text}</div>
    """)
    note_format = compact_html(r"""
        <input type="checkbox" class="article-note-input" id="article-note-{number}">
        <span class="article-note">
            <label class="article-note-label" for="article-note-{number}">
                <span class="article-note-button">Note</span>
            </label>
            <span class="article-note-text-container">
                <span class="article-note-text">{note}</span>
            </span>
        </span>
    """)
    inline_code_format = compact_html(r"""
        <span class="article-inline-code">{code}</span>
    """)

    def __init__(self, raw_text: str):
        self.raw_text = raw_text

    def __repr__(self):
        return f"Paragraph({self.raw_text!r})"

    def render(self, site: Site) -> str:
        text = self.raw_text
        while match := self.note_pattern.search(text):
            text = text[:match.start()] + self.note_format.format(number=site.next_number(), note=match.group(1)) + text[match.end():]
        while match := self.inline_code_pattern.search(text):
            text = text[:match.start()] + self.inline_code_format.format(code=match.group(1)) + text[match.end():]
        return self.format.format(text=text)


class CodeBlock(Element):
    code_block_pattern = re.compile(r"```(?:@(\S*))?((?:.|\s)*?)(?<!\\)```")
    with_file_format = compact_html("""
        <div class="article-codeblock">
            <div class="article-codeblock-source">in {source}</div>
            <div class="article-codeblock-code">{code}</div>
        </div>""")

    def __init__(self, file: str | None, raw_code: str):
        self.file = file
        self.raw_code = raw_code

    def __repr__(self):
        return f"CodeBlock({self.raw_code!r})"

    def render(self, site: Site) -> str:
        if self.file is None:
            raise Exception()
        else:
            return self.with_file_format.format(source=self.file, code=self.raw_code)


def main():
    parser = argparse.ArgumentParser(description="Generate a website from source markdown files")
    parser.add_argument("source", type=Path, help="directory to copy and generate from")
    parser.add_argument("dest", type=Path, help="directory to copy and generate to")
    args = parser.parse_args()
    generator = Generator(args.source, args.dest)
    generator.render()



if __name__ == '__main__':
    main()
