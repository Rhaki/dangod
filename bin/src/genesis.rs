use {
    crate::{
        ext::{cometbft_config_path, cometbft_genesis_path, g_home_dir, PathBuffExt, Writer},
        types::{Account, Genesis},
    },
    bip32::{Language, Mnemonic, PrivateKey, PublicKey, XPrv},
    dango_genesis::{build_genesis, GenesisConfig, GenesisUser},
    dango_types::{
        account_factory::Username,
        auth::Key,
        constants::{GUARDIAN_SETS, PYTH_PRICE_SOURCES},
        taxman,
    },
    grug::{
        btree_map, Coins, Denom, Duration, HashExt, Inner, Json, JsonSerExt, NumberConst, Udec128,
        Uint128,
    },
    k256::ecdsa::SigningKey,
    std::{collections::BTreeMap, path::PathBuf, str::FromStr, sync::LazyLock},
};

pub const STATIC_OWNER_KEY: &str = "junior fault athlete legal inject duty board school anger mesh humor file desk element ticket shop engine paper question love castle ghost bring discover";
pub const STATIC_FEE_RECIPIENT_KEY: &str = "believe share juice host giraffe photo silent equip drift upset seed abstract border stage funny fabric rate boring power village tower north sniff potato";
pub const STATIC_KEY_1: &str = "impulse youth electric wink tomorrow fruit squirrel practice effort mimic leave year visual calm surge system census tower involve wild symbol coral purchase uniform";
pub const STATIC_KEY_2: &str = "visit spend fatigue fork acid junk prize monitor bonus gym frog educate blouse mountain beyond loop nominee argue car shield mixed chunk current force";

const GENESIS_FILE: &str = "genesis.json";
const DEFAULT_COINS: LazyLock<Coins> = LazyLock::new(|| {
    Coins::try_from(btree_map! {
        "udng" => 100_000_000,
    })
    .unwrap()
});
const FEE_DENOM: &str = "udng";
const FEE_RATE: Udec128 = Udec128::ZERO;
const DENOM_FEE_CREATION: Uint128 = Uint128::new(1);
const DEFAULT_MAX_ORPHAN_AGE: Duration = Duration::from_seconds(7 * 24 * 60 * 60);

pub fn generate(dir: PathBuf, accounts: BTreeMap<Username, Account>) -> anyhow::Result<()> {
    let genesis = Genesis {
        accounts,
        fee_rate: FEE_RATE,
        fee_denom: Denom::from_str(FEE_DENOM)?,
        fee_denom_creation: DENOM_FEE_CREATION,
        contracts: None,
        max_orphan_age: DEFAULT_MAX_ORPHAN_AGE,
    };

    let path = dir.join(GENESIS_FILE);

    genesis.write_pretty_json(&path)?;

    println!("Genesis config file generated at: {:?}", path);

    Ok(())
}

pub fn generate_random(dir: PathBuf, counter: usize) -> anyhow::Result<()> {
    let mut accounts: BTreeMap<Username, Account> = if counter == 0 {
        vec![(
            Username::from_str("mock_account")?,
            Account {
                mnemonic: "replace me!".to_string(),
                initial_balance: DEFAULT_COINS.clone(),
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
                    Account::rand_with_coins(DEFAULT_COINS.clone()),
                ))
            })
            .collect::<anyhow::Result<_>>()?
    };

    accounts.insert(Username::from_str("owner")?, Account::rand());
    accounts.insert(Username::from_str("fee_recipient")?, Account::rand());

    generate(dir, accounts)
}

pub fn generate_static(dir: PathBuf) -> anyhow::Result<()> {
    let mut accounts: BTreeMap<Username, Account> =
        [(STATIC_KEY_1, "user_1"), (STATIC_KEY_2, "user_2")]
            .into_iter()
            .map(|(key, username)| {
                let account = Account::static_with_coins(key, DEFAULT_COINS.clone());
                let username = Username::from_str(username)?;
                Ok((username, account))
            })
            .collect::<anyhow::Result<_>>()?;

    accounts.insert(
        Username::from_str("owner")?,
        Account::static_with_coins(STATIC_OWNER_KEY, DEFAULT_COINS.clone()),
    );
    accounts.insert(
        Username::from_str("fee_recipient")?,
        Account::static_with_coins(STATIC_FEE_RECIPIENT_KEY, DEFAULT_COINS.clone()),
    );

    generate(dir, accounts)
}

pub fn build(dir: PathBuf) -> anyhow::Result<()> {
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

    let codes = dango_genesis::build_rust_codes();

    let users: BTreeMap<Username, GenesisUser> = genesis_config
        .accounts
        .iter()
        .map(|(username, account)| {
            let seed = Mnemonic::new(account.mnemonic.clone(), Language::English)?.to_seed("");
            let pk_bytes = SigningKey::from(XPrv::derive_from_path(
                &seed,
                &"m/44'/60'/0'/0/0".to_string().parse()?,
            )?)
            .public_key()
            .to_bytes();

            Ok((
                username.clone(),
                GenesisUser {
                    key: Key::Secp256k1(pk_bytes.into()),
                    key_hash: pk_bytes.hash256(),
                    balances: account.initial_balance.clone(),
                },
            ))
        })
        .collect::<anyhow::Result<_>>()?;

    let (genesis_state, contracts, addresses) = build_genesis(GenesisConfig {
        codes,
        users,
        owner: genesis_config.owner()?.0,
        fee_cfg: taxman::Config {
            fee_denom: genesis_config.fee_denom.clone(),
            fee_rate: genesis_config.fee_rate,
        },
        max_orphan_age: genesis_config.max_orphan_age,
        metadatas: btree_map! {},
        pairs: vec![],
        markets: btree_map! {},
        price_sources: PYTH_PRICE_SOURCES.clone(),
        unlocking_cliff: Duration::from_weeks(4 * 9),
        unlocking_period: Duration::from_weeks(4 * 27),
        wormhole_guardian_sets: GUARDIAN_SETS.clone(),
        hyperlane_local_domain: 88888888,
        hyperlane_ism_validator_sets: btree_map! {},
        warp_routes: btree_map! {},
        account_factory_minimum_deposit: Coins::default(),
    })?;

    for (username, account) in genesis_config.accounts.iter_mut() {
        if let Some(address) = addresses.get(username) {
            account.address = Some(*address)
        };
    }

    genesis_config.contracts = Some(contracts);

    genesis_cometbft["app_state"] = genesis_state.to_json_value()?.into_inner();

    genesis_config.write_pretty_json(dir.join(GENESIS_FILE))?;

    genesis_cometbft.write_pretty_json(cometbft_genesis_path()?)?;

    Ok(())
}

pub fn reset() -> anyhow::Result<()> {
    std::fs::remove_file(g_home_dir()?.join(".dagnod/genesis.json"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/data"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".cometbft/config"))?;
    std::fs::remove_dir_all(g_home_dir()?.join(".dango/data"))?;

    Ok(())
}
