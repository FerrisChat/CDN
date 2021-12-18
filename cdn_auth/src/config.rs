use std::env;

lazy_static! {
    pub static ref AUTH_TOKEN: String =
        env::var("FC_CDN_AUTH_TOKEN").expect("FC_CDN_AUTH_TOKEN env var is not set.");
}
