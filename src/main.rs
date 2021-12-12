use clap::{crate_authors, crate_description, crate_name, crate_version, Arg};

fn main() {
    let matches = clap::app_from_crate!()
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .help("The mode that FerrisChat CDN will run on, can be either storage or server")
                .takes_value(true)
                .required(true)
                .validator(|arg| match arg.as_str() {
                    "storage" => Ok(()),
                    "server" => Ok(()),
                    _ => Err(String::from(
                        "Invalid mode, must be either storage or server",
                    )),
                }),
        )
        .arg(
            Arg::with_name("node_id")
                .short("n")
                .long("node-id")
                .help("The node id that FerrisChat CDN storage server will run on, only required if mode is storage")
                .takes_value(true)
                .validator(|arg| match arg.parse::<u64>() {
                    Ok(_) => Ok(()),
                    Err(_) => Err(String::from(
                        "Invalid node id, must be a valid number",
                    )),
                })
        )
        .get_matches();

    let mode = matches.value_of("mode").unwrap(); // Is safe to unwrap since arg mode is required

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if mode == "storage" {
        use cdn_storage::entrypoint;

        let node_id: u64 = match matches.value_of("node_id") {
            Some(node_id) => node_id.parse::<u64>().unwrap(), // Validator already checked if it can be parsed to u64, so is safe to call unwrap
            None => panic!("Node id is required for storage mode"),
        };

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(entrypoint(node_id));
    } else {
        use cdn_server::entrypoint;

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(entrypoint());
    }
}
