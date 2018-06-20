extern crate clap;
extern crate kintampo_server;
extern crate notify;
extern crate zmq;

use std::thread;
use std::time::Duration;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;

use clap::{App, Arg};

fn watch() -> notify::Result<()> {
    //prepare context and publisher
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("tcp://*:5563")
        .expect("failed binding publisher");

    loop {
        publisher
            .send(b"A", zmq::SNDMORE)
            .expect("failed sending first envelope");
        publisher
            .send(b"We don't want to see this", 0)
            .expect("failed sending first message");
        publisher
            .send(b"B", zmq::SNDMORE)
            .expect("failed sending second envelope");
        publisher
            .send(b"We would like to see this", 0)
            .expect("failed sending second message");
        thread::sleep(Duration::from_millis(1));
    }

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = try!(Watcher::new(tx, Duration::from_secs(2)));

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    try!(watcher.watch("/Users/dgregoire/tmp", RecursiveMode::Recursive));

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}


fn main() {
    let matches = App::new("kintampo")
        .version("0.1.0")
        .author("Daniel Gregoire <daniel.l.gregoire@gmail.com>")
        .about("Kintampo service")
        .arg(
            Arg::with_name("example_arg")
                .short("e")
                .long("example")
                .value_name("EXAMPLE_INTEGER_VALUE")
                .required(false)
                .help("An example command-line argument.")
        )
        .get_matches();

    if let Some(example_value) = matches.value_of("example_arg") {
        kintampo_server::example(Some(example_value.parse::<i32>().unwrap()));
    } else {
        kintampo_server::example(None);
    }

    // thread::spawn(|| {
        if let Err(e) = watch() {
            println!("error: {:?}", e)
        }
    // });

    // println!("Starting HelloWorld server...");
    // let context = zmq::Context::new();
    // let responder = context.socket(zmq::REP).unwrap();

    // assert!(responder.bind("tcp://*:5555").is_ok());

    // let mut msg = zmq::Message::new().unwrap();
    // loop {
    //     responder.recv(&mut msg, 0).unwrap();
    //     println!("Received {}", msg.as_str().unwrap());
    //     thread::sleep(Duration::from_millis(1000));
    //     responder.send(b"World", 0).unwrap();
    // }

}
