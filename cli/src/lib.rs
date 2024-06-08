pub mod error;
pub mod meta;
pub mod project;
pub mod render;
pub mod theme;
pub mod utils;
pub mod version;
use version::VersionFormat;

use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

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

#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Root directory for the book
    /// (Defaults to the current directory when omitted)
    #[clap(default_value = "")]
    pub dir: String,

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

    /// Watch for changes and rebuild the book automatically.
    ///
    /// - If false, no watch.
    /// - If true, `workspace` is watched.
    ///
    /// This option will do nothing if `no_build` is true.
    #[clap(long)]
    pub watch: bool,
}

pub mod build_info {
    /// The version of the typst-ts-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}
