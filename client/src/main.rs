extern crate clap;
extern crate kintampo_client;
#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate zmq;

use clap::{App, Arg};

fn main() {
    pretty_env_logger::init();

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

    let context = zmq::Context::new();
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
}
