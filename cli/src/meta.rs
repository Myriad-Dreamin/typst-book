use serde::{Deserialize, Serialize};

/// Typst content kind embedded in metadata nodes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum BookMetaContent {
    #[serde(rename = "raw")]
    Raw { content: serde_json::Value },
    #[serde(rename = "plain-text")]
    PlainText { content: String },
}

/// Content summary kind in summary.typ
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum BookMetaElem {
    #[serde(rename = "part")]
    Part { title: BookMetaContent, level: i32 },
    #[serde(rename = "chapter")]
    Chapter {
        title: BookMetaContent,
        link: Option<String>,
        #[serde(default)]
        sub: Vec<BookMetaElem>,
        section: Option<String>,
    },
    #[serde(rename = "separator")]
    Separator {},
}

/// General information about your book.
/// Book metadata in summary.typ
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BookMeta {
    /// The title of the book
    pub title: String,
    /// The author(s) of the book
    pub authors: Vec<String>,
    /// A description for the book, which is added as meta information in the
    /// html `<head>` of each page
    pub description: String,
    /// The github repository for the book
    pub repository: String,
    /// The github repository editing template for the book
    /// example: `https://github.com/Me/Book/edit/main/path/to/book/{path}`
    pub repository_edit: String,
    /// The main language of the book, which is used as a language attribute
    /// `<html lang="en">` for example.
    pub language: String,
    /// Content summary of the book
    pub summary: Vec<BookMetaElem>,
}

/// Build metadata in summary.typ
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BuildMeta {
    /// The directory to put the rendered book in. By default this is book/ in
    /// the book's root directory. This can overridden with the --dest-dir CLI
    /// option.
    #[serde(rename = "dest-dir")]
    pub dest_dir: String,
}
