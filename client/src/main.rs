extern crate clap;
extern crate kintampo_client;
extern crate zmq;

use clap::{App, Arg};

fn main() {
    let matches = App::new("kintampo-client")
        .version("0.1.0")
        .author("Daniel Gregoire <daniel.l.gregoire@gmail.com>")
        .about("Kintampo client")
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
        kintampo_client::example(Some(example_value.parse::<i32>().unwrap()));
    } else {
        kintampo_client::example(None);
    }

    // println!("Connecting to hello world server...\n");

    // let context = zmq::Context::new();
    // let requester = context.socket(zmq::REQ).unwrap();

    // assert!(requester.connect("tcp://localhost:5555").is_ok());

    // let mut msg = zmq::Message::new().unwrap();

    // for request_nbr in 0..10 {
    //     println!("Sending Hello {}...", request_nbr);
    //     requester.send(b"Hello", 0).unwrap();

    //     requester.recv(&mut msg, 0).unwrap();
    //     println!("Received World {}: {}", msg.as_str().unwrap(), request_nbr);
    // }

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    subscriber
        .connect("tcp://localhost:5563")
        .expect("failed connecting subscriber");
    subscriber
        .set_subscribe(b"B")
        .expect("failed subscribing");

    loop {
        let envelope = subscriber
            .recv_string(0)
            .expect("failed receiving envelope")
            .unwrap();
        let message = subscriber
            .recv_string(0)
            .expect("failed receiving message")
            .unwrap();
        println!("[{}] {}", envelope, message);
    }
}
