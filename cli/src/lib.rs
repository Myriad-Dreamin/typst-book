pub mod error;
pub mod font;
pub mod meta;
mod outline;
pub mod project;
pub mod render;
pub mod theme;
pub mod utils;
pub mod version;
use version::VersionFormat;

use core::fmt;
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[clap(name = "typst-book", version = "0.1.0")]
pub struct Opts {
    /// Print Version
    #[arg(short = 'V', long, group = "version-dump")]
    pub version: bool,

    /// Print Version in format
    #[arg(long = "VV", alias = "version-fmt", group = "version-dump", default_value_t = VersionFormat::None)]
    pub vv: VersionFormat,

    #[clap(subcommand)]
    pub sub: Option<Subcommands>,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "The cli for typst-book.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    #[clap(about = "init book.")]
    Init(InitArgs),
    #[clap(about = "build book.")]
    Build(BuildArgs),
    #[clap(about = "serve book.")]
    Serve(ServeArgs),
}

/// Determine the approach to retrieving metadata of a book project.
#[derive(Default, ValueEnum, Debug, Clone, Eq, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum MetaSource {
    #[default]
    /// Detect mode automatically.
    Auto,
    /// Strictly retrieve the project's meta by label queries.
    /// + retrieve the book meta from `<typst-book-book-meta>`
    /// + retrieve the build meta from `<typst-book-build-meta>`
    Strict,
    /// Infer the project's meta from the outline of main file.
    /// Note: if the main file also contains `<typst-book-book-meta>` or
    /// `<typst-book-build-meta>`, the manual-set meta will be used first.
    Outline,
}

impl fmt::Display for MetaSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_possible_value().unwrap().get_name())
    }
}

#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Root directory for the book
    /// (Defaults to the current directory when omitted)
    #[clap(default_value = "")]
    pub dir: String,

    /// Determine the approach to retrieving metadata of the book project.
    #[clap(long, default_value = "auto")]
    pub meta_source: MetaSource,

    /// Root directory for the typst workspace, which is same as the
    /// `typst-cli`'s root. (Defaults to the root directory for the book
    /// when omitted)
    #[clap(long, short, default_value = "")]
    pub workspace: String,

    /// Output to directory, default in the same directory as the entry file.
    /// Relative paths are interpreted relative to the book's root directory.
    /// If omitted, typst-book uses build.build-dir from book.toml or defaults
    /// to `./dist`.
    #[clap(long, short, default_value = "")]
    pub dest_dir: String,

    /// Reset path to root in html files.
    #[clap(long, default_value = "/")]
    pub path_to_root: String,

    /// Add additional directories to search for fonts
    #[clap(
        long = "font-path",
        env = "TYPST_FONT_PATHS", 
        value_name = "DIR",
        action = ArgAction::Append,
    )]
    pub font_paths: Vec<PathBuf>,
}

#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Init options")]
pub struct InitArgs {
    /// arguments for compile setting.
    #[clap(flatten)]
    pub compile: CompileArgs,
}

#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Build options")]
pub struct BuildArgs {
    /// arguments for compile setting.
    #[clap(flatten)]
    pub compile: CompileArgs,
}

#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct ServeArgs {
    /// arguments for compile setting.
    #[clap(flatten)]
    pub compile: CompileArgs,

    /// Do not build the book before serving.
    #[clap(long)]
    pub no_build: bool,

    /// Listen address.
    #[clap(long, default_value = "127.0.0.1:25520")]
    pub addr: String,
}

pub mod build_info {
    /// The version of the typst-ts-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}
