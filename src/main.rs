use clap::{crate_authors, crate_description, crate_name, crate_version, Arg};

#[tokio::main]
async fn main() {
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
        .get_matches();

    let mode = matches.value_of("mode").unwrap(); // Is safe to unwrap since arg mode is required

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if mode == "storage" {
        use cdn_storage::entrypoint;

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(entrypoint());
    } else {
        use cdn_server::entrypoint;

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(entrypoint());
    }
}
