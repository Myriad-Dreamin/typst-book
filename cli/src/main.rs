use std::{net::SocketAddr, path::Path, process::exit};

use clap::{Args, Command, FromArgMatches};
use log::error;
use typst_book_cli::{
    error::prelude::*,
    project::Project,
    utils::{async_continue, create_dirs, make_absolute, write_file, UnwrapOrExit},
    version::intercept_version,
    BuildArgs, InitArgs, Opts, ServeArgs, Subcommands,
};
use typst_ts_core::path::{unix_slash, PathClean};
use warp::{http::Method, Filter};

fn get_cli(sub_command_required: bool) -> Command {
    let cli = Command::new("$").disable_version_flag(true);
    Opts::augment_args(cli).subcommand_required(sub_command_required)
}

fn help_sub_command() -> ! {
    Opts::from_arg_matches(&get_cli(true).get_matches()).unwrap_or_exit();
    exit(0);
}

fn main() {
    let opts = Opts::from_arg_matches(&get_cli(false).get_matches()).unwrap_or_exit();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("typst", log::LevelFilter::Warn)
        .filter_module("typst_ts", log::LevelFilter::Info)
        .filter_module("tracing::", log::LevelFilter::Off)
        .init();

    intercept_version(opts.version, opts.vv);

    match opts.sub {
        Some(Subcommands::Init(args)) => {
            async_continue(async { init(args).await.unwrap_or_exit() })
        }
        Some(Subcommands::Build(args)) => build(args).unwrap_or_exit(),
        Some(Subcommands::Serve(args)) => {
            async_continue(async { serve(args).await.unwrap_or_exit() })
        }
        None => help_sub_command(),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

async fn init(args: InitArgs) -> ZResult<()> {
    let dir = make_absolute(Path::new(&args.compile.dir)).clean();

    if dir.exists() {
        clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            format!("the init directory has already existed: {dir:?}"),
        )
        .exit()
    }

    let wd = if args.compile.workspace.is_empty() {
        dir.clone()
    } else {
        make_absolute(Path::new(&args.compile.workspace)).clean()
    };
    let rel = pathdiff::diff_paths(&dir, &wd).unwrap();

    if rel.starts_with("..") {
        clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            format!("bad workspace, which is sub-directory of book.typ's root: {dir:?} / {wd:?} -> {rel:?}"),
        )
        .exit()
    }

    let workspace_to_root = Path::new("/").join(rel);

    let page_template = unix_slash(&workspace_to_root.join("templates/page.typ"));
    let ebook_template = unix_slash(&workspace_to_root.join("templates/ebook.typ"));
    let book_typ = unix_slash(&workspace_to_root.join("book.typ"));

    let build_meta = if args.compile.dest_dir.is_empty() {
        String::default()
    } else {
        format!(
            r##"#build-meta(
  dest-dir: "{}",
)"##,
            args.compile.dest_dir
        )
    };

    create_dirs(&dir)?;
    create_dirs(dir.join("templates"))?;

    write_file(
        dir.join("book.typ"),
        format!(
            r##"
#import "@preview/book:0.2.5": *

#show: book

#book-meta(
  title: "typst-book",
  summary: [
    #prefix-chapter("sample-page.typ")[Hello, typst]
  ]
)

{build_meta}

// re-export page template
#import "{page_template}": project
#let book-page = project
"##
        ),
    )?;
    write_file(
        dir.join("sample-page.typ"),
        format!(
            r##"#import "{book_typ}": book-page

#show: book-page.with(title: "Hello, typst")

= Hello, typst

Sample page
"##
        ),
    )?;
    write_file(
        dir.join("ebook.typ"),
        format!(
            r##"#import "@preview/book:0.2.5": *

#import "{ebook_template}"

#show: ebook.project.with(title: "typst-book", spec: "book.typ")

// set a resolver for inclusion
#ebook.resolve-inclusion(it => include it)
"##
        ),
    )?;
    write_file(
        dir.join("templates/page.typ"),
        include_bytes!("../../contrib/typst/gh-pages.typ"),
    )?;
    write_file(
        dir.join("templates/ebook.typ"),
        std::str::from_utf8(include_bytes!("../../contrib/typst/gh-ebook.typ").as_slice())
            .unwrap()
            .replace("/contrib/typst/gh-pages.typ", &page_template),
    )?;
    write_file(
        dir.join("templates/theme-style.toml"),
        include_bytes!("../../contrib/typst/theme-style.toml"),
    )?;
    write_file(
        dir.join("templates/tokyo-night.tmTheme"),
        include_bytes!("../../contrib/typst/tokyo-night.tmTheme"),
    )?;

    serve(ServeArgs {
        compile: args.compile,
        addr: "127.0.0.1:25520".to_owned(),
        ..Default::default()
    })
    .await
}

fn build(args: BuildArgs) -> ZResult<()> {
    let mut proj = Project::new(args.compile)?;
    proj.build()?;

    exit(0)
}

pub async fn serve(args: ServeArgs) -> ZResult<()> {
    let proj = std::sync::Mutex::new(Project::new(args.compile.clone())?);

    // Build the book if it hasn't been built yet
    if !args.no_build {
        proj.lock().expect("Cannot get lock").build()?;

        // since we don't need the compilation cache anymore, we can evict it
        comemo::evict(0);
    }

    let http_addr: SocketAddr = args
        .addr
        .clone()
        .parse()
        .map_err(map_string_err("ParseServeAddr"))?;

    let server = warp::serve({
        let cors =
            warp::cors().allow_methods(&[Method::GET, Method::POST, Method::DELETE, Method::HEAD]);

        let dev = warp::path("dev").and(warp::fs::dir(""));

        dev.or(warp::fs::dir(
            proj.lock().expect("Cannot get lock").dest_dir.clone(),
        ))
        .with(cors)
        .with(warp::compression::gzip())
    });

    // server.run(http_addr).await;
    tokio::spawn(server.run(http_addr));

    if args.watch && !args.no_build {
        let wx = watchexec::Watchexec::new(move |mut action| {
            // Filter out event that means a new build is needed
            if action.events.iter().any(|event| {
                event.tags.iter().any(|tag| {
                    matches!(tag,
                    watchexec_events::Tag::Path {
                        path,
                        file_type: Some(watchexec_events::FileType::File),
                    } if Some("typ") == path.extension().and_then(|osstr| osstr.to_str()))
                })
            }) {
                proj.lock()
                    .expect("Cannot get lock")
                    .build()
                    .expect("Cannot build the project");
            }

            if action
                .signals()
                .any(|sig| sig == watchexec_signals::Signal::Interrupt)
            {
                action.quit();
            }

            action
        })
        .expect("watch feature is not available");

        wx.config.pathset([args.compile.workspace]);

        match wx.main().await {
            Ok(_) => {}
            Err(err) => {
                error!("watch error: {:?}", err);
            }
        }
    }

    exit(0);
}
