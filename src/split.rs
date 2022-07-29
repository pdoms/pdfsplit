use lopdf::Document;
use std::io;
use std::path::Path;

use crate::ErrMessages;

pub struct PDF<'a> {
    orig_doc: Document,
    splits: Vec<Split<'a>>,
    target_path: String,
}

#[derive(Debug)]
pub struct Split<'a>(pub u32, pub u32, pub &'a str);

impl PDF<'_> {
    pub fn new<'a>(path: &'a str, splits: Vec<Split<'a>>) -> Result<PDF<'a>, lopdf::Error> {
        let target_path = get_root(path.clone());
        let path = Path::new(path);
        if path.is_file() {
            let doc = match Document::load(path) {
                Ok(d) => d,
                Err(err) => return Err(err),
            };

            let pdf = PDF {
                orig_doc: doc,
                splits,
                target_path,
            };
            Ok(pdf)
        } else {
            Err(lopdf::Error::IO(io::Error::new(
                io::ErrorKind::NotFound,
                "file not found",
            )))
        }
    }

    pub fn split(&mut self) -> Result<(), ErrMessages> {
        let all_pages = self.orig_doc.get_pages();
        let max_pages = *all_pages.iter().last().unwrap().0;
        for split in self.splits.iter() {
            let mut doc = self.orig_doc.clone();
            let mut del = (1..=split.0 - 1).into_iter().collect::<Vec<_>>();
            let mut del_post = (split.1 + 1..=max_pages).into_iter().collect::<Vec<_>>();
            del.append(&mut del_post);
            doc.delete_pages(&mut del);
            let path = self.target_path.to_owned() + split.2;
            match doc.save(path) {
                Ok(_) => (),
                Err(_) => return Err(ErrMessages::FileSave),
            }
        }
        Ok(())
    }
}

impl Split<'_> {
    pub fn new<'a>() -> Split<'a> {
        Split(0, 0, "")
    }
}

fn get_root(fullpath: &str) -> String {
    let mut root = String::from(fullpath);
    let mut i = root.len();

    let idx = match root.rfind("/") {
        Some(v) => v,
        None => i,
    };
    loop {
        if i == idx + 1 {
            break;
        } else {
            root.pop();
            i = i - 1;
        }
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pdf() {
        let splits: Vec<Split> = vec![Split(1, 4, "file1.pdf"), Split(5, 8, "file2.pdf")];
        let path = "/home/paulo/Projects/Rust/pdf/src/guide.pdf";
        let pdf: PDF = PDF::new(path, splits).unwrap();
        assert_eq!(pdf.splits[0].0, 1);
    }

    #[test]
    fn test_split() {
        let splits: Vec<Split> = vec![Split(1, 4, "file1.pdf"), Split(5, 8, "file2.pdf")];
        let path = "/home/paulo/Projects/Rust/pdf/src/guide.pdf";
        let mut pdf: PDF = PDF::new(path, splits).unwrap();
        pdf.split();
    }

    #[test]
    fn test_get_root() {
        let path = "/home/paulo/Projects/Rust/pdf/src/guide.pdf";
        let res = get_root(path);
        assert_eq!(res, "/home/paulo/Projects/Rust/pdf/src/".to_string());
    }
}
