use {
    lazy_static::lazy_static,
    log::*,
    std::{env, path::PathBuf},
};

lazy_static! {
    #[derive(Debug)]
    static ref SOLANA_ROOT: PathBuf = get_solana_root();
}

pub fn initialize_globals() {
    let _ = *SOLANA_ROOT; // Force initialization of lazy_static
}

pub mod genesis;
pub mod kubernetes;
// pub mod release;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidatorType {
    Bootstrap,
    Standard,
}

pub fn get_solana_root() -> PathBuf {
    let solana_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR")).to_path_buf();
    info!("solana root: {:?}", solana_root);
    solana_root
}

#[macro_export]
macro_rules! boxed_error {
    ($message:expr) => {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, $message)) as Box<dyn Error>
    };
}

pub fn load_env_variable_by_name(name: &str) -> Result<String, env::VarError> {
    env::var(name)
}
