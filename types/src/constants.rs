use {
    dango_types::constants::DANGO_DENOM,
    grug::{btree_map, Coins, NumberConst, Udec128, Uint128},
    std::sync::LazyLock,
};

pub const DANGOD_APP_DIR: &str = ".dangod";

pub const STATIC_OWNER_KEY: &str = "junior fault athlete legal inject duty board school anger mesh humor file desk element ticket shop engine paper question love castle ghost bring discover";
pub const STATIC_FEE_RECIPIENT_KEY: &str = "believe share juice host giraffe photo silent equip drift upset seed abstract border stage funny fabric rate boring power village tower north sniff potato";
pub const STATIC_KEY_1: &str = "impulse youth electric wink tomorrow fruit squirrel practice effort mimic leave year visual calm surge system census tower involve wild symbol coral purchase uniform";
pub const STATIC_KEY_2: &str = "visit spend fatigue fork acid junk prize monitor bonus gym frog educate blouse mountain beyond loop nominee argue car shield mixed chunk current force";

pub const GENESIS_FILE: &str = "genesis.json";
pub const DEFAULT_COINS: LazyLock<Coins> = LazyLock::new(|| {
    Coins::try_from(btree_map! {
        DANGO_DENOM.clone() => 100_000_000,
    })
    .unwrap()
});
pub const FEE_RATE: Udec128 = Udec128::ZERO;
pub const DENOM_FEE_CREATION: Uint128 = Uint128::new(1);

pub use grug::DEFAULT_MAX_ORPHAN_AGE;
