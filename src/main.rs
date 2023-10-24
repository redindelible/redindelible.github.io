mod series_article;
mod article;
mod link;
mod site;
mod page;
mod index;

use std::path::Path;
use std::fs;

use crate::site::Site;

const SITE_DIR: &'static str = "_site";
const OUTPUT_DIR: &'static str = "docs";

struct Generator {

}

impl Generator {
    fn clear_directory(dir: impl AsRef<Path>) {
        for item in fs::read_dir(dir).unwrap() {
            let p = item.unwrap().path();
            if p.is_dir() {
                fs::remove_dir_all(p).unwrap();
            } else {
                fs::remove_file(p).unwrap();
            }
        }
    }

    fn generate_directory(input: impl AsRef<Path>, output: impl AsRef<Path>) {
        let mut site = Site::new();
        for item in fs::read_dir(input).unwrap() {
            let path = item.unwrap().path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let ext = path.extension().and_then(|e| e.to_str());
                match ext {
                    Some("mdx") => site.add_mdx(path),
                    _ => {
                        let copy_path = Path::new(&output.as_ref()).join(file_name);
                        fs::copy(path, copy_path).unwrap();
                    }
                }
            }
        }

        site.render_pages(Path::new(OUTPUT_DIR));
    }
}



fn generate() {
    Generator::clear_directory(OUTPUT_DIR);

    Generator::generate_directory(SITE_DIR, OUTPUT_DIR);
}

fn main() {
    generate();
}