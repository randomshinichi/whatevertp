use clap::{App, Arg, ArgMatches, SubCommand};
use iron::Iron;
use mount::Mount;
use staticfile::Static;
use std::net::SocketAddr;
use std::path::PathBuf;

fn get_ip_port_as_string(m: &ArgMatches) -> String {
    let ip = m.value_of("ip").unwrap();
    let port = m.value_of("port").unwrap();
    return format!("{}:{}", ip, port);
}

fn main() {
    let matches = App::new("whatevertp")
        .version("1.0")
        .about("Convenient HTTP/TFTP server in any directory, without configuration")
        .arg(
            Arg::with_name("ip")
                .help("IPv4/IPv6 address to listen on")
                .default_value("0.0.0.0")
                .long("ip"),
        )
        .arg(
            Arg::with_name("port")
                .help("TCP/UDP port to listen on")
                .default_value("3333")
                .long("port"),
        )
        .subcommand(
            SubCommand::with_name("http")
                .about("starts a http server")
                .arg(Arg::with_name("HTTP_DIR").required(true)),
        )
        .subcommand(
            SubCommand::with_name("tftp")
                .about("starts a tftp server")
                .arg(Arg::with_name("TFTP_DIR").required(true)),
        )
        .get_matches();

    let ip_port = get_ip_port_as_string(&matches);

    if let Some(ref tftp_subcommand) = matches.subcommand_matches("tftp") {
        let socket_addr = ip_port.parse::<SocketAddr>().ok();
        let path = tftp_subcommand.value_of("TFTP_DIR").unwrap();

        tftp(path, socket_addr);
    } else if let Some(ref matches) = matches.subcommand_matches("http") {
        http(matches.value_of("HTTP_DIR").unwrap(), ip_port);
    } else {
        match matches.subcommand_name() {
            None => println!("No subcommand was used"),
            _ => println!("Some other subcommand was used"),
        }
    }
}

fn tftp(path: &str, ip_port: Option<SocketAddr>) {
    let path_b1 = PathBuf::from(path).canonicalize().unwrap();

    let path_b2 = path_b1.clone();
    let path_b2_str = path_b2.to_str();
    let mut server = tftp_server::server::TftpServerBuilder::new()
        .addr_opt(ip_port)
        .serve_dir(path_b1)
        .build()
        .expect("Error creating server");
    println!(
        "Listening at {:?}, {:?}",
        server.local_addr().unwrap().to_string(),
        path_b2_str
    );
    print!("{:?}, {:?}", path_b2, path_b2_str);
    match server.run() {
        Ok(_) => std::process::exit(0),
        Err(e) => println!("tftp_server error: {:?}", e),
    }
}

fn http(path: &str, ip_port: String) {
    let mut mount = Mount::new();
    mount.mount("/", Static::new(PathBuf::from(path)));
    println!("Listening at {:?}, {:?}", ip_port, path);
    Iron::new(mount).http(ip_port.as_str()).unwrap();
}
