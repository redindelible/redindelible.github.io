from __future__ import annotations

import re
import json
import argparse
from datetime import datetime
from pathlib import Path
from typing import Type

import jinja2


class Templates:
    def __init__(self, dir: Path):
        self.dir = dir
        self.env = jinja2.Environment(
            loader=jinja2.FileSystemLoader(dir),
            trim_blocks=True,
            lstrip_blocks=True,
        )

    def load(self, name: str) -> jinja2.Template:
        return self.env.get_template(f"{name}.html")


class Generator:
    METADATA_PATTERN = re.compile(r"^%({(?:.|\s)*?})%")

    def __init__(self, source_dir: Path, templates_dir: Path, target_dir: Path):
        self.source_dir: Path = source_dir
        self.target_dir: Path = target_dir

        self.templates: Templates = Templates(templates_dir)
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

        self.site.render(self.templates, self.target_dir)


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

    def render(self, templates: Templates, target: Path):
        for page in self.pages:
            page_path = Path(str(target) + str(page.link))
            page_path.parent.mkdir(parents=True, exist_ok=True)
            page_path.write_text(page.render(templates, self))


class Page:
    page_types: dict[str, Type[Page]] = {}

    def __init_subclass__(cls, *, register_name: str=None, **kwargs):
        if register_name is None:
            raise Exception()
        Page.page_types[register_name] = cls
        cls._page_type = register_name

    def __init__(self):
        self.type = self._page_type

    @classmethod
    def create(cls, metadata: dict, text: str):
        return cls.page_types[metadata["type"]]._create(metadata, text)

    @classmethod
    def _create(cls, metadata: dict, text: str) -> Page:
        raise NotImplementedError()

    @property
    def link(self) -> Path:
        raise NotImplementedError()

    def render(self, templates: Templates, site: Site) -> str:
        raise NotImplementedError()


class Index(Page, register_name="index"):
    def __init__(self):
        super().__init__()

    @classmethod
    def _create(cls, metadata: dict, text: str) -> Page:
        return Index()

    @property
    def link(self) -> Path:
        return Path("/index.html")

    def render(self, templates: Templates, site: Site) -> str:
        articles = sorted((page for page in site.pages if isinstance(page, Article)), key=lambda art: art.date, reverse=True)
        return templates.load("template_index").render(articles=articles)


class Article(Page, register_name="article"):
    def __init__(self, title: str, date: datetime, elements: list[Element]):
        super().__init__()
        self.title: str = title
        self.date: datetime = date
        self.elements = elements

    @property
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
            elif text.startswith("#", i):
                if match := Heading.heading_pattern.match(text, i):
                    elements.append(Heading(len(match.group(1)), match.group(2).strip()))
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

    @property
    def preview(self) -> str:
        for element in self.elements:
            if isinstance(element, Paragraph):
                return element.raw_text[:300].strip()
        else:
            return ""

    def render(self, templates: Templates, site: Site) -> str:
        return templates.load("template_article").render(article=self, templates=templates, site=site)


class ArticleSeries(Article, register_name="article-series"):
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

    @property
    def link(self):
        return Path(f"/articles/{self.title[:20].strip().lower().replace(' ', '-')}-{self.number}-{self.date.strftime('%m-%d-%y')}.html")

    def render(self, templates: Templates, site: Site) -> str:
        series = sorted(site.all_series[self.series_name], key=lambda art: art.number)
        return templates.load("template_article_series").render(article=self, series=series, templates=templates, site=site)


class Element:
    def render(self, templates: Templates, site: Site) -> str:
        raise NotImplementedError()


class Heading(Element):
    heading_pattern = re.compile(r"(#)([^#\n]*)")

    def __init__(self, level: int, heading: str):
        self.level = level
        self.heading = heading

    def render(self, templates: Templates, site: Site) -> str:
        return templates.load("template_heading").render(level=self.level, heading=self.heading)


class Paragraph(Element):
    paragraph_break_pattern = re.compile(r"\n\s*\n")
    note_pattern = re.compile(r"(?<!\\)`@note (.*?)(?<!\\)`")
    inline_code_pattern = re.compile(r"(?<!\\)`(.*?)(?<!\\)`")

    def __init__(self, raw_text: str):
        self.raw_text = raw_text

    def __repr__(self):
        return f"Paragraph({self.raw_text!r})"

    def render(self, templates: Templates, site: Site) -> str:
        text = self.raw_text
        while match := self.note_pattern.search(text):
            note = templates.load("template_note").render(number=site.next_number(), note=match.group(1))
            text = text[:match.start()] + note + text[match.end():]
        while match := self.inline_code_pattern.search(text):
            code = templates.load("template_inline_code").render(code=match.group(1))
            text = text[:match.start()] + code + text[match.end():]
        return templates.load("template_paragraph").render(text=text)


class CodeBlock(Element):
    code_block_pattern = re.compile(r"```(?:@(\S*))?((?:.|\s)*?)(?<!\\)```")

    def __init__(self, file: str | None, raw_code: str):
        self.file = file
        self.raw_code = raw_code

    def __repr__(self):
        return f"CodeBlock({self.raw_code!r})"

    def render(self, templates: Templates, site: Site) -> str:
        return templates.load("template_code_block").render(source=self.file, code=self.raw_code)


def main():
    parser = argparse.ArgumentParser(description="Generate a website from source markdown files")
    parser.add_argument("source", type=Path, help="directory to copy and generate from")
    parser.add_argument("templates", type=Path, help="directory with template files")
    parser.add_argument("dest", type=Path, help="directory to copy and generate to")
    args = parser.parse_args()
    generator = Generator(args.source, args.templates, args.dest)
    generator.render()



if __name__ == '__main__':
    main()
