extern crate clap;
extern crate kintampo;
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

fn envelope_from_pathbuf(pb: &str) -> String {
    pb.replace("/", "_")
}

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

    info!("Watching directory: {:?}", dir);
    let context = zmq::Context::new();

    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("inproc://*:55630")
        .expect("failed binding zmq publisher");
    let configurator = context.socket(zmq::REP).unwrap();
    configurator
        .bind("inproc://*:55630")
        .expect("failed binding zmq responder");

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

                    // See http://zguide.zeromq.org/page:all#The-Dynamic-Discovery-Problem
                    let mut backend = context.socket(zmq::XSUB).unwrap();
                    backend
                        .connect("inproc://localhost:55630")
                        .expect("failed connecting dispatcher");

                    // Does the use of XPUB/XSUB let us shuttle both regular messages
                    // and allow clients hitting this frontend to subscribe to the
                    // more granular messages available via the internal publisher?
                    // If so, this is good subordination of detail without hiding.
                    let mut frontend = context.socket(zmq::XPUB).unwrap();
                    frontend
                        .bind("tcp://*:5563")
                        .expect("failed binding zmq dispatch publisher");

                    // I believe we can't use this directly, because
                    // we want to manipulate the granularity of messaging.
                    // let dispatch_proxy = zmq::proxy(&mut frontend, &mut backend);

                    // TODO:
                    // We need to hit the configurator with a request for
                    // "all known subscriptions", which is all the directories
                    // found under the root Kintampo dir, when starting clients.
                    //
                    // When new directories are created, the configurator needs
                    // to gain knowledge of them.
                    //
                    // Clients also need to find out about them and open a
                    // separate socket for each directory.

                    loop {
                        let envelope = backend
                            .recv_string(0)
                            .expect("failed receiving envelope")
                            .unwrap();
                        let message = backend
                            .recv_string(0)
                            .expect("failed receiving message")
                            .unwrap();

                        if envelope == "CREATE" || envelope == "WRITE" {
                            let path_portion = envelope_from_pathbuf(&message);
                            let mut new_envelope = String::with_capacity(path_portion.len() + 4);
                            new_envelope.push_str("NEW/");
                            new_envelope.push_str(&path_portion);
                            frontend
                                .send(new_envelope.as_bytes(), zmq::SNDMORE)
                                .expect("failed sending NEW/path envelope");
                            frontend
                                .send(message.as_bytes(), 0)
                                .expect("failed sending NEW/path message")
                        }
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
}
