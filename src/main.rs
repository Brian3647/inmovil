use colored::Colorize;
use snowboard::response;
use snowboard::Server;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use std::{env, fs, io};

macro_rules! info {
    ($($arg:tt)*) => {
        println!("{} {}", "info ~>".bright_cyan(), format!($($arg)*));
    };
}

macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{} {}", "warn ~>".bright_yellow(), format!($($arg)*));
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "error ~>".bright_red(), format!($($arg)*));
    };
}

macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "success ~>".bright_green(), format!($($arg)*));
    };
}

fn get_paths(path: impl Into<PathBuf>, existing_files: Vec<PathBuf>) -> Vec<PathBuf> {
    let path = path.into();
    let mut files = existing_files;

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            files = get_paths(path, files);
        } else {
            files.push(path);
        }
    }

    files
}

fn read_file(path: impl Into<PathBuf>) -> io::Result<Vec<u8>> {
    let path: PathBuf = path.into();
    let contents = fs::read(path)?;

    Ok(contents)
}

fn read_paths(paths: Vec<PathBuf>) -> HashMap<String, Vec<u8>> {
    let mut contents = HashMap::new();

    for path in paths {
        let name = path.clone();
        let file_contents = read_file(path);

        if let Err(e) = file_contents {
            warn!("Failed to read file {:?}. Ignoring it...", name);
            warn!("{:?}", e);
            continue;
        }

        contents.insert(name.display().to_string(), file_contents.unwrap());
    }

    contents
}

fn load_dir(path: impl Into<PathBuf>) -> HashMap<String, Vec<u8>> {
    let path = path.into();
    let mut paths = get_paths(path, vec![]);
    paths.sort();

    read_paths(paths)
}

struct Data {
    contents: HashMap<String, Vec<u8>>,
    dir: String,
}

fn main() -> snowboard::Result {
    info!("Loading files...");
    let start_time = Instant::now();
    let mut quiet = false;

    let args = env::args();
    let (argc, mut argv) = (args.len(), args);

    if argc < 2 {
        error!("Usage: <dir> [port]");
        return Ok(());
    }

    argv.next();

    let dir = argv.next().unwrap();
    let contents = load_dir(&dir);

    let mut port = 3000;

    for arg in argv.take(2) {
        if arg == "--no-logs" {
            quiet = true;
        } else {
            port = match arg.parse::<u16>() {
                Ok(x) => x,
                Err(e) => {
                    error!("Failed to parse port `{}`: {:?}", arg, e);
                    return Ok(());
                }
            };
        }
    }

    let mut mime_types = HashMap::new();

    for path in contents.keys() {
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .essence_str()
            .to_owned();
        mime_types.insert(path.clone(), mime_type);
    }

    let time_taken = start_time.elapsed().as_secs_f32();

    success!("Loaded {} files in {:.2}s.", contents.len(), time_taken);

    let data = Data { contents, dir };

    let addr = format!("localhost:{}", port);

    if !quiet {
        println!();
        info!("-- Server logs --");
    }

    Server::new(addr)?.run(move |request| {
        let contents = &data.contents;
        let dir = &data.dir;

        if !quiet {
            info!("{} {}", request.method, &request.url);
        }

        let path = Path::new(&dir)
            .join(request.url.trim_start_matches('/'))
            .display()
            .to_string();

        if let Some(contents) = contents.get(&path) {
            let mut res = response!(ok);
            res.set_bytes(contents);

            if let Some(mime_type) = mime_types.get(&path) {
                res.content_type(mime_type.into());
            }

            res
        } else {
            response!(not_found)
        }
    })
}
