use {
    bip32::{Language, Mnemonic},
    dango_genesis::Contracts,
    dango_types::account_factory::Username,
    grug::{Addr, Coins, Denom, Duration, Udec128, Uint128},
    rand::rngs::OsRng,
    std::{collections::BTreeMap, str::FromStr},
};

#[grug::derive(Serde)]
pub struct Genesis {
    pub accounts: BTreeMap<Username, Account>,
    pub fee_rate: Udec128,
    pub fee_denom: Denom,
    pub fee_denom_creation: Uint128,
    pub contracts: Option<Contracts>,
    pub max_orphan_age: Duration,
}

impl Genesis {
    pub fn owner(&self) -> anyhow::Result<(Username, &Account)> {
        self.account("owner")
    }

    pub fn _fee_recipient(&self) -> anyhow::Result<(Username, &Account)> {
        self.account("fee_recipient")
    }

    pub fn account(&self, name: &str) -> anyhow::Result<(Username, &Account)> {
        let name = Username::from_str(name)?;
        let account = self
            .accounts
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("account not found"))?;
        Ok((name, account))
    }
}

#[grug::derive(Serde)]
pub struct Account {
    pub mnemonic: String,
    pub initial_balance: Coins,
    pub address: Option<Addr>,
}

impl Account {
    pub fn rand() -> Self {
        Self {
            mnemonic: Mnemonic::random(OsRng, Language::English)
                .phrase()
                .to_string(),
            initial_balance: Coins::default(),
            address: None,
        }
    }

    pub fn static_with_coins(mnemonic: &str, coins: Coins) -> Self {
        Self {
            mnemonic: mnemonic.to_string(),
            initial_balance: coins,
            address: None,
        }
    }

    pub fn rand_with_coins(coins: Coins) -> Self {
        let mut val = Self::rand();
        val.initial_balance = coins;
        val
    }
}
