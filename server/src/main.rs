extern crate clap;
extern crate kintampo_server;
#[macro_use] extern crate log;
extern crate notify;
extern crate pretty_env_logger;
extern crate zmq;

use std::fs::create_dir_all;
use std::thread;
use std::time::Duration;

use notify::{DebouncedEvent, RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;

use clap::{App, Arg};

fn main() -> Result<(),std::io::Error> {
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
    let context = zmq::Context::new();

    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("tcp://*:55630")
        .expect("failed binding zmq publisher");

    let (tx, rx) = channel();

    let mut watcher: Result<RecommendedWatcher, notify::Error> = Watcher::new(tx, Duration::from_secs(1));
    match watcher {
        Ok(ref mut watcher) => {
            match watcher.watch(dir, RecursiveMode::Recursive) {
                Ok(_) => {
                    thread::spawn(move || {
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
                    });

                    let subscriber = context.socket(zmq::SUB).unwrap();
                    info!("Subscribing to CREATE and WRITE messages from Kintampo server...");
                    subscriber
                        .connect("tcp://localhost:55630")
                        .expect("failed connecting subscriber");
                    subscriber
                        .set_subscribe(b"CREATE")
                        .expect("failed subscribing to CREATE");
                    subscriber
                        .set_subscribe(b"WRITE")
                        .expect("failed subscribing to WRITE");

                    loop {
                        let envelope = subscriber
                            .recv_string(0)
                            .expect("failed receiving envelope")
                            .unwrap();
                        let message = subscriber
                            .recv_string(0)
                            .expect("failed receiving message")
                            .unwrap();
                        info!("[{}] {}", envelope, message);
                    }
                },
                Err(e) => {
                    error!("Unable to start filesystem watcher on directory {}: {:?}", dir, e);
                    panic!();
                }
            }
        },
        Err(e) => {
            error!("Unable to construct filesystem watcher: {:?}", e);
            panic!();
        }
    }
    // Ok(())
}
