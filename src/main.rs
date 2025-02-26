use anyhow::Result;
use std::env;
mod dexclient;
use dexclient::DexClient;
mod dex_collect;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 DEX 客户端
    //let dex_client = DexClient::new("path/to/your/keypair.json")?;
    let home_dir = env::var("HOME").expect("找不到 HOME 环境变量");
    let _path = format!("{}/.config/solana/id.json", home_dir);
    let mut dex_client = DexClient::new();
    println!("Solana DEX 交互程序\n");

    println!("请输入市场地址:HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1");
    let mut market = "HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1".to_string();
    let _account = dex_client.create_market_account(market.trim()).await?;
    dex_client.get_market_info(market.trim()).await?;
    let _price = dex_client.get_latest_price("SOL/USDC").await?;
    let _order_book = dex_client.get_orderbook(market.trim()).await?;

    let _markets = dex_client.get_common_markets();
    // dex_client.monitor_price("SOL/USDC").await?; 
    Ok(())
}
