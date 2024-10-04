use {
    crate::{
        ext::{
            cometbft_config_path, cometbft_genesis_path, g_home_dir, read_wasm_file, PathBuffExt,
            Writer,
        },
        types::{Account, Genesis},
    },
    bip32::{Language, Mnemonic, PrivateKey, PublicKey, XPrv},
    clap::Subcommand,
    dango_genesis::{build_genesis, Codes, GenesisUser},
    dango_types::{account_factory::Username, auth::Key},
    grug::{Coins, HashExt, Json, JsonSerExt, NumberConst, Udec128, Uint128},
    k256::ecdsa::SigningKey,
    std::{collections::BTreeMap, path::PathBuf, str::FromStr},
};

const GENESIS_FILE: &str = "genesis.json";
const DEFAULT_COINS: &str = "ugrug:1000";

#[derive(Subcommand)]
pub enum GenesisCommand {
    Build,
    Generate { counter: usize },
    Reset,
}

impl GenesisCommand {
    pub fn run(&self, dir: PathBuf) -> anyhow::Result<()> {
        match self {
            GenesisCommand::Build => build(dir),
            GenesisCommand::Generate { counter } => generate(dir, *counter),
            GenesisCommand::Reset => reset(),
        }
    }
}

fn generate(dir: PathBuf, counter: usize) -> anyhow::Result<()> {
    let mut accounts: BTreeMap<Username, Account> = if counter == 0 {
        vec![(
            Username::from_str("mock_account")?,
            Account {
                menmonic: "replace me!".to_string(),
                initial_balance: Coins::from_str(DEFAULT_COINS)?,
                address: None,
            },
        )]
        .into_iter()
        .collect()
    } else {
        (0..counter)
            .map(|i| {
                Ok((
                    Username::from_str(&format!("account_{}", i))?,
                    Account::rand_with_coins(Coins::from_str(DEFAULT_COINS)?),
                ))
            })
            .collect::<anyhow::Result<_>>()?
    };

    accounts.insert(Username::from_str("owner")?, Account::rand());
    accounts.insert(Username::from_str("fee_recipient")?, Account::rand());

    let genesis = Genesis {
        accounts,
        fee_rate: Udec128::ZERO,
        fee_denom: "ugrug".to_string(),
        fee_denom_creation: Uint128::new(0),
        contracts: BTreeMap::default(),
    };

    let path = dir.join(GENESIS_FILE);

    genesis.write_pretty_json(&path)?;

    println!("Genesis config file generated at: {:?}", path);

    Ok(())
}

fn build(dir: PathBuf) -> anyhow::Result<()> {
    let mut genesis_config: Genesis = dir.join(GENESIS_FILE).read()?;
    let mut genesis_cometbft: Json = match cometbft_genesis_path()?.read() {
        Ok(val) => val,
        Err(_) => {
            println!("cometbft genesis file not found, initializing cometbft...");
            std::process::Command::new("cometbft")
                .arg("init")
                .status()?;
            cometbft_genesis_path()?.read()?
        }
    };

    // Change CORS on cometbft config.toml
    let config: String = cometbft_config_path()?.read_string::<String>()?.replace(
        "cors_allowed_origins = []",
        "cors_allowed_origins = [\"*\"]",
    );

    std::fs::write(cometbft_config_path()?, config)?;

    let codes = {
        let account_factory = read_wasm_file("dango_account_factory.wasm")?;
        let account_spot = read_wasm_file("app_account_spot.wasm")?;
        let account_safe = read_wasm_file("app_account_safe.wasm")?;
        let amm = read_wasm_file("app_amm.wasm")?;
        let bank = read_wasm_file("app_bank.wasm")?;
        let ibc_transfer = read_wasm_file("app_mock_ibc_transfer.wasm")?;
        let taxman = read_wasm_file("app_taxman.wasm")?;
        let token_factory = read_wasm_file("app_token_factory.wasm")?;

        Codes {
            account_factory,
            account_spot,
            account_safe,
            amm,
            bank,
            ibc_transfer,
            taxman,
            token_factory,
        }
    };

    let users: BTreeMap<Username, GenesisUser> = genesis_config
        .accounts
        .iter()
        .map(|(username, account)| {
            let key = Mnemonic::new(account.menmonic.clone(), Language::English)
                .unwrap()
                .to_seed("");
            let sk: SigningKey =
                XPrv::derive_from_path(&key, &"m/44'/60'/0'/0/0".to_string().parse().unwrap())
                    .unwrap()
                    .into();

            let vk = sk.public_key();
            let bytes = vk.to_bytes();
            let key_hash = bytes.hash160();
            let key = Key::Secp256k1(bytes.into());

            (
                username.clone(),
                GenesisUser {
                    key,
                    key_hash,
                    balances: account.initial_balance.clone(),
                },
            )
        })
        .collect();

    let (genesis_state, contracts, addresses) = build_genesis(
        codes,
        users,
        &genesis_config.owner()?.0,
        &genesis_config.fee_recipient()?.0,
        genesis_config.fee_denom.clone(),
        genesis_config.fee_rate,
        genesis_config.fee_denom_creation,
    )?;

    for (username, account) in genesis_config.accounts.iter_mut() {
        if let Some(address) = addresses.get(username) {
            account.address = Some(address.clone())
        };
    }

    genesis_config.contracts = vec![
        ("account_factory".to_string(), contracts.account_factory),
        ("amm".to_string(), contracts.amm),
        ("bank".to_string(), contracts.bank),
        ("ibc_transfer".to_string(), contracts.ibc_transfer),
        ("taxman".to_string(), contracts.taxman),
        ("token_factory".to_string(), contracts.token_factory),
    ]
    .into_iter()
    .collect();

    genesis_cometbft["app_state"] = genesis_state.to_json_value()?;

    genesis_config.write_pretty_json(dir.join(GENESIS_FILE))?;

    genesis_cometbft.write_pretty_json(cometbft_genesis_path()?)?;

    Ok(())
}

fn reset() -> anyhow::Result<()> {
    std::fs::remove_file(g_home_dir()?.join(".rgrug/genesis.json"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/data"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/config"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".grug/data"))?;

    Ok(())
}
