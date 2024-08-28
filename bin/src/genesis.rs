use {
    crate::ext::{cometbft_genesis_path, g_home_dir, read_wasm_file, PathBuffExt, Writer},
    app_types::{
        account_factory::{self, GenesisUser},
        auth::{AccountId, AccountType, Key, Username},
        bank, taxman,
    },
    bip32::{Language, Mnemonic},
    clap::Subcommand,
    grug::{
        btree_map, Addr, Coins, Hash256, HashExt, Json, JsonSerExt, NumberConst, Udec256,
        GENESIS_SENDER,
    },
    grug_client::{AdminOption, GenesisBuilder, SigningKey},
    rand::rngs::OsRng,
    std::{collections::BTreeMap, path::PathBuf, str::FromStr},
};

const GENESIS_FILE: &str = "genesis.json";

#[grug::derive(Serde)]
struct Genesis {
    pub owner: Account,
    pub fee_recipient: Account,
    pub accounts: Vec<Account>,
    pub fee_rate: Udec256,
    pub fee_denom: String,
}

#[derive(Default)]
#[grug::derive(Serde)]
struct Account {
    pub name: String,
    pub menmonic: String,
    pub initial_balance: Coins,
    pub address: Option<Addr>,
}

impl Account {
    pub fn rand(name: &str) -> Self {
        Self {
            name: name.to_string(),
            menmonic: Mnemonic::random(OsRng, Language::English)
                .phrase()
                .to_string(),
            ..Default::default()
        }
    }
}

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
    let accounts = if counter == 0 {
        vec![Account {
            name: "replace me!".to_string(),
            menmonic: "replace me!".to_string(),
            ..Default::default()
        }]
    } else {
        (0..counter)
            // .iter()
            .map(|i| Account::rand(&format!("account_{}", i)))
            .collect()
    };

    let genesis = Genesis {
        owner: Account::rand("owner"),
        fee_recipient: Account::rand("fee_recipient"),
        accounts,
        fee_rate: Udec256::ZERO,
        fee_denom: "ugrug".to_string(),
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

    let mut builder = GenesisBuilder::new();

    let hash_taxman = builder.upload(read_wasm_file("app_taxman.wasm")?)?;
    let hash_app_bank = builder.upload(read_wasm_file("app_bank.wasm")?)?;
    let hash_account_spot = builder.upload(read_wasm_file("app_account_spot.wasm")?)?;
    let hash_account_factory = builder.upload(read_wasm_file("app_account_factory.wasm")?)?;

    let genesis_users = &mut BTreeMap::new();

    let initial_balances = &mut BTreeMap::new();

    let account_factory = Addr::compute(GENESIS_SENDER, hash_account_factory, b"salt");

    println!("account_factory: {account_factory}");

    for account in genesis_config.accounts.iter_mut() {
        add_user(
            account,
            genesis_users,
            initial_balances,
            hash_account_spot,
            account_factory,
        )?;
    }

    add_user(
        &mut genesis_config.owner,
        genesis_users,
        initial_balances,
        hash_account_spot,
        account_factory,
    )?;
    add_user(
        &mut genesis_config.fee_recipient,
        genesis_users,
        initial_balances,
        hash_account_spot,
        account_factory,
    )?;

    let account_factory = builder.instantiate(
        hash_account_factory,
        &account_factory::InstantiateMsg {
            code_hashes: btree_map!(AccountType::Spot => hash_account_spot, ),
            genesis_users: genesis_users.clone(),
        },
        *b"salt",
        Coins::default(),
        AdminOption::SetToNone,
    )?;

    let taxman = builder.instantiate(
        hash_taxman,
        &app_types::taxman::InstantiateMsg {
            config: taxman::Config {
                fee_recipient: genesis_config.fee_recipient.address.unwrap(),
                fee_denom: genesis_config.fee_denom.clone(),
                fee_rate: genesis_config.fee_rate,
            },
        },
        *b"salt",
        Coins::default(),
        AdminOption::SetToNone,
    )?;

    let bank = builder.instantiate(
        hash_app_bank,
        &bank::InstantiateMsg {
            initial_balances: initial_balances.clone(),
        },
        *b"salt",
        Coins::default(),
        AdminOption::SetToNone,
    )?;

    let finalize_genesis_state = builder
        .set_bank(bank)
        .set_taxman(taxman)
        .set_owner(account_factory)
        .add_app_config("account_factory", &account_factory)?
        .set_instantiate_permission(grug::Permission::Everybody)
        .set_upload_permission(grug::Permission::Everybody)
        .build();

    genesis_cometbft["app_state"] = finalize_genesis_state.to_json_value()?;

    genesis_config.write_pretty_json(dir.join(GENESIS_FILE))?;

    genesis_cometbft.write_pretty_json(cometbft_genesis_path()?)?;

    println!("hash account spot: {:?}", hash_account_spot);
    println!("hash account factory: {:?}", hash_account_factory);
    println!("hash taxman: {:?}", hash_taxman);
    println!("hash app bank: {:?}", hash_app_bank);

    Ok(())
}

fn reset() -> anyhow::Result<()> {
    std::fs::remove_file(g_home_dir()?.join(".rgrug/genesis.json"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/data"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/config"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".grug/data"))?;

    Ok(())
}

fn add_user(
    account: &mut Account,
    genesis_users: &mut BTreeMap<Username, GenesisUser>,
    initial_balances: &mut BTreeMap<Addr, Coins>,
    hash_account_spot: Hash256,
    account_factory: Addr,
) -> anyhow::Result<()> {
    let username = Username::from_str(&account.name)?;

    let key = SigningKey::from_mnemonic(
        &Mnemonic::new(account.menmonic.clone(), Language::English)?,
        60,
    )?
    .public_key();

    let key_hash = key.hash160();

    let key = Key::Secp256k1(key.into());

    let user = GenesisUser {
        key: key.clone(),
        key_hash,
    };

    let salt = AccountId {
        username: username.clone(),
        index: 1,
    }
    .to_string()
    .into_bytes();

    let user_addr = Addr::compute(account_factory, hash_account_spot, &salt);

    account.address = Some(user_addr);

    genesis_users.insert(username, user);

    initial_balances.insert(user_addr, account.initial_balance.clone());

    Ok(())
}
