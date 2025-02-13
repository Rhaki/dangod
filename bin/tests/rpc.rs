use {
    dango_client::SingleSigner,
    dangod::{STATIC_KEY_1, STATIC_KEY_2},
    grug::{
        Addr, Base64Encoder, Client, Coins, Defined, Encoder, Hash256, JsonDeExt, Message,
        SigningClient, TxEvents,
    },
    std::str::FromStr,
};

pub const LOCALHOST_RPC: &str = "http://localhost:26657";

type Account = SingleSigner<Defined<u32>>;

#[tokio::test]
async fn works() {
    let client = client().await.unwrap();
    let block = client.query_block_result(None).await.unwrap();
    println!("{:?}", block);
}

#[tokio::test]
async fn transfer() {
    let client = client().await.unwrap();

    let mut account_1 = create_account(
        &client,
        STATIC_KEY_1,
        "0x76e21577e7df18de93bbe82779bf3a16b2bacfd9",
        "user_1",
    )
    .await
    .unwrap();

    let account_2 = create_account(
        &client,
        STATIC_KEY_2,
        "0xe23490cec98ba421f6506d598f1d61087d299863",
        "user_2",
    )
    .await
    .unwrap();

    let response = client
        .send_message(
            &mut account_1,
            Message::transfer(account_2.address, Coins::one("udng", 10).unwrap()).unwrap(),
            grug::GasOption::Simulate {
                scale: 1.2,
                flat_increase: 1_000_000,
            },
        )
        .await
        .unwrap();

    println!("response: {:?}", response);
    println!("hash: {}", response.hash);
}

#[tokio::test]
async fn search_tx() {
    let tx_hash = "CD4D60019594667946768384AAB81ED2EF99416582A3C5E4099D816986119CD0";

    let client = client().await.unwrap();

    let tx = client
        .query_tx(Hash256::from_str(tx_hash).unwrap())
        .await
        .unwrap();

    let decoded_bytes = Base64Encoder::ENCODING.decode(&tx.tx_result.data).unwrap();
    let events = decoded_bytes.deserialize_json::<TxEvents>().unwrap();

    println!("e: {:?}", events);
}

async fn chain_id() -> anyhow::Result<String> {
    let client = Client::connect(LOCALHOST_RPC)?;
    Ok(client.query_status().await?.node_info.network.to_string())
}

async fn client() -> anyhow::Result<SigningClient> {
    let chain_id = chain_id().await?;
    Ok(SigningClient::connect(chain_id, LOCALHOST_RPC)?)
}

async fn create_account(
    client: &Client,
    mnemonic: &str,
    addr: &str,
    username: &str,
) -> anyhow::Result<Account> {
    let account = SingleSigner::from_mnemonic(username, Addr::from_str(addr)?, mnemonic, 60)?
        .query_nonce(client)
        .await?;
    Ok(account)
}
