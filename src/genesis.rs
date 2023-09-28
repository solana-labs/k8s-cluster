
use {
    crate::{boxed_error, initialize_globals, SOLANA_ROOT},
    log::*,
    std::{
        error::Error,
        path::PathBuf,
        process::{Command, Output},
    },
};

pub const DEFAULT_FAUCET_LAMPORTS: u64 = 500000000000000000;
pub const DEFAULT_MAX_GENESIS_ARCHIVE_UNPACKED_SIZE: u64 = 1073741824;
pub const DEFAULT_INTERNAL_NODE_STAKE_SOL: f64 = 10.0; // 10000000000 lamports
pub const DEFAULT_INTERNAL_NODE_SOL: f64 = 500.0; // 500000000000 lamports
pub const DEFAULT_BOOTSTRAP_NODE_STAKE_LAMPORTS: u64 = 10000000000; // 10 SOL
pub const DEFAULT_BOOTSTRAP_NODE_LAMPORTS: u64 = 500000000000; // 500 SOL

pub struct GenesisFlags {
    pub hashes_per_tick: String,
    pub slots_per_epoch: Option<u64>,
    pub target_lamports_per_signature: Option<u64>,
    pub faucet_lamports: Option<u64>,
    pub enable_warmup_epochs: bool,
    pub max_genesis_archive_unpacked_size: Option<u64>,
    pub cluster_type: String,
    pub bootstrap_validator_lamports: Option<f64>,
    pub bootstrap_validator_stake_lamports: Option<f64>,
}

impl std::fmt::Display for GenesisFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "GenesisFlags {{\n\
             hashes_per_tick: {:?},\n\
             slots_per_epoch: {:?},\n\
             target_lamports_per_signature: {:?},\n\
             faucet_lamports: {:?},\n\
             enable_warmup_epochs: {},\n\
             max_genesis_archive_unpacked_size: {:?},\n\
             bootstrap_validator_sol: {:?},\n\
             bootstrap_validator_stake_sol: {:?},\n\
             }}",
            self.hashes_per_tick,
            self.slots_per_epoch,
            self.target_lamports_per_signature,
            self.faucet_lamports,
            self.enable_warmup_epochs,
            self.max_genesis_archive_unpacked_size,
            self.bootstrap_validator_lamports,
            self.bootstrap_validator_stake_lamports,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SetupConfig<'a> {
    pub namespace: &'a str,
    pub num_validators: i32,
    pub prebuild_genesis: bool,
}

pub struct Genesis {
    pub flags: GenesisFlags,
    pub config_dir: PathBuf,
    pub args: Vec<String>,
}

impl Genesis {
    pub fn new(flags: GenesisFlags) -> Self {
        initialize_globals();
        let config_dir = SOLANA_ROOT.join("config");
        if config_dir.exists() {
            std::fs::remove_dir_all(&config_dir).unwrap();
        }
        std::fs::create_dir_all(&config_dir).unwrap();
        Genesis {
            flags,
            config_dir,
            args: Vec::default(),
        }
    }

    pub fn generate_keypair(
        &self, output_path: PathBuf
    ) -> Output {
        Command::new("solana-keygen")
            .arg("new")
            .arg("--no-bip39-passphrase")
            .arg("--silent")
            .arg("-o")
            .arg(output_path)
            .output() // Capture the output of the script
            .expect("Failed to execute solana-keygen for new keypair")
    }

    pub fn generate_faucet(&mut self) -> Result<(), Box<dyn Error>> {
        let outfile = self.config_dir.join("faucet.json");
        let output = self.generate_keypair(outfile);
        if !output.status.success() {
            return Err(boxed_error!(format!(
                "Failed to create new faucet keypair. err: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    pub fn generate_accounts(
        &mut self,
        validator_type: &str,
        number_of_accounts: i32,
    ) -> Result<(), Box<dyn Error>> {
        let mut filename_prefix = "validator".to_string();
        if validator_type == "bootstrap" {
            filename_prefix = format!("{}-{}", validator_type, filename_prefix);
        } else if validator_type == "validator" {
            filename_prefix = "validator".to_string();
        } else {
            return Err(boxed_error!(format!(
                "Invalid validator type: {}",
                validator_type
            )));
        }

        for i in 0..number_of_accounts {
            self.generate_account(validator_type, filename_prefix.as_str(), i)?;
        }

        Ok(())
    }

    // Create identity, stake, and vote accounts
    fn generate_account(
        &mut self,
        validator_type: &str,
        filename_prefix: &str,
        i: i32,
    ) -> Result<(), Box<dyn Error>> {
        let account_types = vec!["identity", "vote-account", "stake-account"];

        for account in account_types {
            let filename: String;
            if validator_type == "bootstrap" {
                filename = format!("{}/{}.json", filename_prefix, account);
            } else if validator_type == "validator" {
                filename = format!("{}-{}-{}.json", filename_prefix, account, i);
            } else {
                return Err(boxed_error!(format!(
                    "Invalid validator type: {}",
                    validator_type
                )));
            }

            let outfile = self.config_dir.join(filename);
            trace!("outfile: {:?}", outfile);

            let output = self.generate_keypair(outfile);
            if !output.status.success() {
                return Err(boxed_error!(format!(
                    "Failed to create new account keypair. account: {}, err: {}",
                    account,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }
        Ok(())

    }

    fn setup_genesis_flags(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        args.push("--bootstrap-validator-stake-lamports".to_string());
        match self.flags.bootstrap_validator_stake_lamports {
            Some(lamports) => args.push(lamports.to_string()),
            None => args.push(DEFAULT_BOOTSTRAP_NODE_STAKE_LAMPORTS.to_string()),   
        }
        args.push("--bootstrap-validator-lamports".to_string());
        match self.flags.bootstrap_validator_lamports {
            Some(lamports) => args.push(lamports.to_string()),
            None => args.push(DEFAULT_BOOTSTRAP_NODE_LAMPORTS.to_string()),   
        }

        args.extend(vec!["--hashes-per-tick".to_string(), self.flags.hashes_per_tick.clone()]);

        args.push("--max-genesis-archive-unpacked-size".to_string());
        match self.flags.max_genesis_archive_unpacked_size {
            Some(size) => args.push(size.to_string()),
            None => args.push(DEFAULT_MAX_GENESIS_ARCHIVE_UNPACKED_SIZE.to_string()),
        }

        if self.flags.enable_warmup_epochs {
            args.push("--enable-warmup-epochs".to_string());
        }

        args.push("--faucet-lamports".to_string());
        match self.flags.faucet_lamports {
            Some(lamports) => args.push(lamports.to_string()),
            None => args.push(DEFAULT_FAUCET_LAMPORTS.to_string()),
        }

        args.extend(vec!["--faucet-pubkey".to_string(), self.config_dir.join("faucet.json").to_string_lossy().to_string()]);
        args.extend(vec!["--cluster-type".to_string(), self.flags.cluster_type.clone()]);
        args.extend(vec!["--ledger".to_string(), self.config_dir.join("bootstrap-validator").to_string_lossy().to_string()]);

        // Order of accounts matters here!!
        args.extend(
            vec![
                "--bootstrap-validator".to_string(),
                self.config_dir.join("bootstrap-validator/identity.json").to_string_lossy().to_string(),
                self.config_dir.join("bootstrap-validator/vote-account.json").to_string_lossy().to_string(), 
                self.config_dir.join("bootstrap-validator/stake-account.json").to_string_lossy().to_string(),
            ]
        );

        if let Some(slots_per_epoch) = self.flags.slots_per_epoch {
            args.extend(vec!["--slots-per-epoch".to_string(), slots_per_epoch.to_string()]);
        }

        if let Some(lamports_per_signature) = self.flags.target_lamports_per_signature {
            args.extend(vec!["--target-lamports-per-signature".to_string(), lamports_per_signature.to_string()]);
        }

        args

        //TODO see multinode-demo.sh. we need spl-genesis-args.sh
        
    }

    pub fn generate(&mut self) -> Result<(), Box<dyn Error>> {
        let args = self.setup_genesis_flags();
        debug!("genesis args: ");
        for arg in &args {
            debug!("{}", arg);
        }
        
        let output = Command::new("solana-genesis")
            .args(&args)
            .output() // Capture the output of the script
            .expect("Failed to execute solana-genesis");

        if !output.status.success() {
            return Err(boxed_error!(format!(
                "Failed to create genesis. err: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }
}