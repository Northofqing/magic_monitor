use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use solana_sdk::account::Account;
use std::str::FromStr;

use crate::dex_collect::serum::serum_client::SerumPriceFetcher;
 

/// DEX 交互结构体
pub struct DexClient {
    price_fetcher: SerumPriceFetcher,
}

impl DexClient {
    /// 创建 DEX 客户端
    pub fn new() -> Self {
        Self {
            price_fetcher: SerumPriceFetcher::new(),
        }
    }
    /// 创建市场账户
    pub async fn create_market_account(&self, market_address: &str) -> Result<Account> {
        // 获取市场信息
        let account = self.price_fetcher.get_account(market_address).await?;
        println!("市场账户大小: {} bytes", account.data.len());
        Ok(account)
    }
    // 获取市场信息
    pub async fn get_market_info(&self, market_address: &str) -> Result<()> {
        match self.price_fetcher.get_account(market_address).await {
            Ok(account) => {
                println!("市场信息:");
                println!("地址: {}", market_address);
                println!("数据大小: {} bytes", account.data.len());
                println!("所有者程序: {}", account.owner);
                println!("lamports余额: {}", account.lamports);
            }
            Err(e) => println!("获取市场信息失败: {}", e),
        }

        Ok(())
    }

    /// 下限价单
    pub async fn place_limit_order(
        &self,
        market_address: &str,
        side: OrderSide,
        price: f64,
        size: f64,
    ) -> Result<()> {
        let _market_pubkey = Pubkey::from_str(market_address)?;

        println!("准备下单:");
        println!("市场: {}", market_address);
        println!("方向: {:?}", side);
        println!("价格: {}", price);
        println!("数量: {}", size);

        // 这里添加实际的下单逻辑
        // 需要构建相应的指令和交易

        Ok(())
    }

    /// 取消订单
    pub async fn cancel_order(&self, market_address: &str, order_id: &str) -> Result<()> {
        let _market_pubkey = Pubkey::from_str(market_address)?;
        println!("取消订单: {}", order_id);

        // 这里添加取消订单的逻辑

        Ok(())
    }

    /// 获取市场深度
    pub async fn get_orderbook(&mut self, market_address: &str) -> Result<()> {
        let _price = self.price_fetcher.get_orderbook(market_address).await?;

        // 这里添加获取订单簿的逻辑

        Ok(())
    }

    /// 获取最新成交价格
    pub async fn get_latest_price(&mut self, market_pair: &str) -> Result<()> {
        let _price = self.price_fetcher.get_price_details(market_pair).await?;

        // 这里添加获取价格的逻辑

        Ok(())
    }

    pub async fn monitor_price(&mut self, market_pair: &str)-> Result<()> {
        // 这里添加价格监控逻辑
        self.price_fetcher.monitor_price(market_pair).await?;
        Ok(())
    }
    // 获取常见市场地址
    pub fn get_common_markets(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            // Raydium 市场
            ("SOL/USDC", "HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1"),
            ("RAY/USDC", "2xiv8A5xrJ7RnGdxXB42uFEkYHJjszEhaJyKKt4WaLep"),
            ("SRM/USDC", "ByRys5tuUWDgL73G8JBAEfkdFf8JWBzPBDHsBVQ5vbQA"),
            // Orca 市场
            (
                "SOL/USDC (Orca)",
                "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6",
            ),
            ("ORCA/USDC", "8N1KkhaCYDpj3awD58d85n973EwkpeYnRp84y1kdZpMX"),
            // Serum 市场
            (
                "SOL/USDC (Serum)",
                "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
            ),
            ("BTC/USDC", "A8YFbxQYFVqKZaoYJLLUVcQiWP7G2MeEgW5wsAQgMvFw"),
            ("ETH/USDC", "4tSvZvnbyzHXLMTiFonMyxZoHmFqau1XArcRCVHLZ5gX"),
        ]
    }
}

/// 订单方向
#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 市场信息结构
#[derive(Debug)]
pub struct MarketInfo {
    address: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    bid_accounts: Vec<Pubkey>,
    ask_accounts: Vec<Pubkey>,
}
