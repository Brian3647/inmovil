use colored::Colorize;
use snowboard::response;
use snowboard::Server;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
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

fn read_file(path: impl Into<PathBuf>) -> io::Result<String> {
    let path: PathBuf = path.into();
    let contents = fs::read_to_string(path)?;

    Ok(contents)
}

fn read_paths(paths: Vec<PathBuf>) -> HashMap<String, String> {
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

fn load_dir(path: impl Into<PathBuf>) -> HashMap<String, String> {
    let path = path.into();
    let mut paths = get_paths(path, vec![]);
    paths.sort();

    read_paths(paths)
}

#[derive(Clone)]
struct Data {
    contents: HashMap<String, String>,
    port: u16,
    dir: String,
}

fn main() {
    let args = env::args();
    let (argc, mut argv) = (args.len(), args);

    if argc < 2 {
        error!("Usage: <dir> [port]");
        return;
    }

    argv.next();

    let dir = argv.next().unwrap();
    info!("Loading files from `{}`...", dir);
    let contents = load_dir(&dir);
    success!("Loaded {} files.", contents.len());

    let port = if argc > 2 {
        match argv.next().unwrap().parse::<u16>() {
            Ok(port) => port,
            Err(e) => {
                error!("Failed to parse port: {:?}", e);
                return;
            }
        }
    } else {
        3000
    };

    let data = Data {
        contents,
        port,
        dir,
    };

    Server::new(&format!("localhost:{}", port), data)
        .on_load(|data| {
            success!("Server started at http://localhost:{}", data.port);
        })
        .on_request(|request, data| {
            let contents = &data.contents;
            let dir = &data.dir;
            info!("{} {}", request.method, &request.url);
            let path = Path::new(&dir).join(request.url.trim_start_matches('/'));

            if let Some(contents) = contents.get(&path.display().to_string()) {
                response!(ok, contents)
            } else {
                response!(not_found)
            }
        })
        .run();
}
