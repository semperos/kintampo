extern crate clap;
#[macro_use]
extern crate log;
extern crate notify;
extern crate pretty_env_logger;
extern crate zmq;

extern crate kintampo;

use std::fs::{create_dir_all};
use std::path::Path;
use std::thread;
use std::time::Duration;

use notify::{DebouncedEvent, RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;

use clap::{App, Arg};

fn ask_configurator_for_directories(context: &zmq::Context, configurator_port: &str) -> Vec<String> {
    let config_client = context.socket(zmq::REQ).unwrap();
    let mut msg_buffer = zmq::Message::new().unwrap();
    config_client
        .connect(&format!("tcp://localhost:{}", configurator_port))
        .expect("failed connecting client to frontend");
    config_client.send(b"topology", 0).unwrap();
    config_client.recv(&mut msg_buffer, 0).unwrap();
    let msg = msg_buffer.as_str().unwrap();
    info!("Config client received: {}", msg);
    kintampo::parse_edn_vector(msg)
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
                .default_value("kintampo_root")
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .required(false)
                .help("The main port to use for publishing data events.")
                .default_value("5563")
        )
        .get_matches();

    let dir = matches.value_of("root_directory").unwrap();
    // Note: depending on platform (?), this may occur
    // after all the ZeroMQ machinery is already running,
    // and so it will trigger events.
    create_dir_all(dir)?;
    let abs_dir = std::fs::canonicalize(dir).unwrap();

    let base_port = matches.value_of("port").unwrap();
    let configurator_port = format!("{}0", base_port);
    let publisher_port = format!("{}1", base_port);

    info!("Watching directory: {:?}", abs_dir);
    let context = zmq::Context::new();

    let configurator = context.socket(zmq::REP).unwrap();
    configurator
        .bind(&format!("tcp://*:{}", configurator_port))
        .expect("failed binding zmq configurator");

    let mut configurator_msg = zmq::Message::new().unwrap();
    let walk_target = abs_dir.to_owned();
    thread::spawn(move || {
        loop {
            configurator.recv(&mut configurator_msg, 0).unwrap();
            let msg = configurator_msg.as_str().unwrap();
            if msg == "topology" {
                // LET IT BE KNOWN
                // ZeroMQ subscriptions to a _prefix_ match,
                // which in our case (since we're establishing
                // subscriptions based on path hierarchies)
                // means we only need to set a ZeroMQ subscriber
                // on the root directory.
                //
                // Will leave this in place, as there may be a
                // good use-case for having multiple separate
                // Kintampo roots, so keeping the initial
                // subscription process flexible on this front
                // is good for now.
                let paths = kintampo::all_dirs(&walk_target);
                configurator.send(format!("[{}]",paths.join(",")).as_bytes(), 0).unwrap();
            } else {
                configurator.send(b"Unsupported operation", 0).unwrap();
            }
        }
    });

    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind(&format!("tcp://*:{}", publisher_port))
        .expect("failed binding zmq publisher");

    let (tx, rx) = channel();

    let mut watcher: Result<RecommendedWatcher, notify::Error> = Watcher::new(tx, Duration::from_secs(1));
    let root_watch_dir = abs_dir.to_owned();
    match watcher {
        Ok(ref mut watcher) => {
            match watcher.watch(root_watch_dir, RecursiveMode::Recursive) {
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
                                        _ => warn!("Sorry, don't handle {:?} yet.", event),
                                    }
                                }
                                Err(e) => error!("watch error: {:?}", e),
                            }
                        }
                    });

                    // See http://zguide.zeromq.org/page:all#The-Dynamic-Discovery-Problem
                    let mut backend = context.socket(zmq::SUB).unwrap();
                    backend
                        .connect(&format!("tcp://localhost:{}", publisher_port))
                        .expect("failed connecting dispatcher");
                    backend
                        .set_subscribe(b"CREATE")
                        .expect("failed to subscribe to CREATE events");
                    backend
                        .set_subscribe(b"WRITE")
                        .expect("failed to subscribe to WRITE events");

                    // Does the use of XPUB/XSUB let us shuttle both regular messages
                    // and allow clients hitting this frontend to subscribe to the
                    // more granular messages available via the internal publisher?
                    // If so, this is good subordination of detail without hiding.
                    info!("Kintampo publishing events on port {}", base_port);
                    let mut frontend = context.socket(zmq::PUB).unwrap();
                    frontend
                        .bind(&format!("tcp://*:{}", base_port))
                        .expect("failed binding zmq dispatch publisher");

                    // I believe we can't use this directly, because
                    // we want to manipulate the granularity of messaging.
                    // let dispatch_proxy = zmq::proxy(&mut frontend, &mut backend);

                    thread::spawn(move || {
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
                                let new_envelope = kintampo::new_path_envelope(&message);
                                frontend
                                    .send(new_envelope.as_bytes(), zmq::SNDMORE)
                                    .expect("failed sending NEW/path envelope");
                                frontend
                                    .send(message.as_bytes(), 0)
                                    .expect("failed sending NEW/path message")
                            }
                        }
                    });

                    // Client that connects to the `frontend`
                    let mut client = context.socket(zmq::SUB).unwrap();
                    client
                        .connect(&format!("tcp://localhost:{}", base_port))
                        .expect("failed connecting client to frontend");
                    let directories = ask_configurator_for_directories(&context, &configurator_port);
                    let envelopes: Vec<String> = directories
                        .into_iter()
                        .map(|d| kintampo::new_path_envelope(&d))
                        .collect();
                    for envelope in envelopes {
                        trace!("Client subscribing to {}", envelope);
                        client
                            .set_subscribe(envelope.as_bytes())
                            .expect(&format!("client failed to subscribe to {}", envelope));
                    }
                    loop {
                        let envelope = client
                            .recv_string(0)
                            .expect("failed receiving client envelope")
                            .unwrap();
                        let message = client
                            .recv_string(0)
                            .expect("failed receiving client message")
                            .unwrap();

                        let (op, path) = kintampo::parse_envelope(&envelope);
                        match op.as_ref() {
                            "NEW" => {
                                if path.is_dir() {
                                    // ZeroMQ subscriptions are prefix-based,
                                    // so this client will already handle
                                    // this new sub-directory.
                                    info!("Client now watching directory: {}",path.to_str().unwrap());
                                } else {
                                    info!("Client processing NEW file: {}", path.to_str().unwrap());
                                    let run_file = path.parent().unwrap().join("run.sh");
                                    if run_file.exists() {
                                        let output = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(run_file)
                                            .output()
                                            .expect("failed to run");
                                        println!("OUTPUT: {:?}", output.stdout);
                                    }
                                }
                            },
                            _ => trace!("Client doesn't handled {:?} yet.", op)
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
