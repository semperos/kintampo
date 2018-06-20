extern crate clap;
extern crate kintampo_server;
#[macro_use] extern crate log;
extern crate notify;
extern crate pretty_env_logger;
extern crate zmq;

use std::fs::create_dir_all;
use std::time::Duration;

use notify::{DebouncedEvent, RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;

use clap::{App, Arg};

fn watch(folder: &str) -> notify::Result<()> {
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("tcp://*:55630")
        .expect("failed binding zmq publisher");

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = try!(Watcher::new(tx, Duration::from_secs(2)));

    try!(watcher.watch(folder, RecursiveMode::Recursive));

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Create(pathbuf) => {
                        trace!("Created new {:?}", pathbuf);
                        publisher
                            .send(b"CREATE", zmq::SNDMORE)
                            .expect("failed sending envelope for new file creation");
                        publisher
                            .send(pathbuf.to_str().unwrap().as_bytes(), 0)
                            .expect("failed sending message for new file creation");
                    }
                    DebouncedEvent::Write(pathbuf) => {
                        trace!("Wrote to existing {:?}", pathbuf);
                        publisher
                            .send(b"WRITE", zmq::SNDMORE)
                            .expect("failed sending envelope for file write");
                        publisher
                            .send(pathbuf.to_str().unwrap().as_bytes(), 0)
                            .expect("failed sending message for file write");
                    }
                    _ => trace!("Sorry, don't handle {:?} yet.", event),
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let matches = App::new("kintampo")
        .version("0.1.0")
        .author("Daniel Gregoire <daniel.l.gregoire@gmail.com>")
        .about("Kintampo service")
        .arg(
            Arg::with_name("root_directory")
                .short("d")
                .long("directory")
                .value_name("ROOT_DIRECTORY_TO_WATCH")
                .required(false)
                .help("The root directory for Kintampo to watch.")
                .default_value("/tmp/kintampo")
        )
        .get_matches();

    let dir = matches.value_of("root_directory").unwrap();
    create_dir_all(dir)?;

    // thread::spawn(|| {
    info!("Watching directory: {:?}", dir);
    if let Err(e) = watch(dir) {
        error!("error: {:?}", e);
    }
    // });

    Ok(())
}
