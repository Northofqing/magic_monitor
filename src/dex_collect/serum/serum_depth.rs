use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{collections::HashMap, str::FromStr, mem::size_of};

/// 深度级别结构
#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub size: f64,
    pub total: f64,
}

/// 市场深度结构
#[derive(Debug, Clone)]
pub struct MarketDepth {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub spread: f64,
    pub total_bid_size: f64,
    pub total_ask_size: f64,
}

/// 市场状态结构 - 不使用 bytemuck，直接解析字段
#[derive(Debug)]
pub struct MarketState {
    pub account_flags: u64,
    pub own_address: Pubkey,
    pub vault_signer_nonce: u64,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub base_deposits_total: u64,
    pub base_fees_accrued: u64,
    pub quote_vault: Pubkey,
    pub quote_deposits_total: u64,
    pub quote_fees_accrued: u64,
    pub quote_dust_threshold: u64,
    pub req_queue: Pubkey,
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub fee_rate_bps: u64,
    pub referrer_rebates_accrued: u64,
}

impl MarketState {
    // 从字节数据解析市场状态
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(anyhow::anyhow!("Data too short"));
        }

        let data = &data[5..]; // 跳过5字节头部
        let mut pos = 0;

        // 辅助函数：读取u64
        let read_u64 = |data: &[u8], pos: &mut usize| -> Result<u64> {
            if *pos + 8 > data.len() {
                return Err(anyhow::anyhow!("Buffer overflow while reading u64"));
            }
            let val = u64::from_le_bytes(data[*pos..*pos + 8].try_into()?);
            *pos += 8;
            Ok(val)
        };

        // 辅助函数：读取Pubkey
        let read_pubkey = |data: &[u8], pos: &mut usize| -> Result<Pubkey> {
            if *pos + 32 > data.len() {
                return Err(anyhow::anyhow!("Buffer overflow while reading Pubkey"));
            }
            let pubkey = Pubkey::new(&data[*pos..*pos + 32]);
            *pos += 32;
            Ok(pubkey)
        };

        Ok(Self {
            account_flags: read_u64(data, &mut pos)?,
            own_address: read_pubkey(data, &mut pos)?,
            vault_signer_nonce: read_u64(data, &mut pos)?,
            base_mint: read_pubkey(data, &mut pos)?,
            quote_mint: read_pubkey(data, &mut pos)?,
            base_vault: read_pubkey(data, &mut pos)?,
            base_deposits_total: read_u64(data, &mut pos)?,
            base_fees_accrued: read_u64(data, &mut pos)?,
            quote_vault: read_pubkey(data, &mut pos)?,
            quote_deposits_total: read_u64(data, &mut pos)?,
            quote_fees_accrued: read_u64(data, &mut pos)?,
            quote_dust_threshold: read_u64(data, &mut pos)?,
            req_queue: read_pubkey(data, &mut pos)?,
            event_queue: read_pubkey(data, &mut pos)?,
            bids: read_pubkey(data, &mut pos)?,
            asks: read_pubkey(data, &mut pos)?,
            base_lot_size: read_u64(data, &mut pos)?,
            quote_lot_size: read_u64(data, &mut pos)?,
            fee_rate_bps: read_u64(data, &mut pos)?,
            referrer_rebates_accrued: read_u64(data, &mut pos)?,
        })
    }
}

pub struct MarketDepthFetcher {
    rpc_client: RpcClient,
    markets: HashMap<String, String>,
}

impl MarketDepthFetcher {
    pub fn new() -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            "https://api.mainnet-beta.solana.com",
            CommitmentConfig::confirmed(),
        );

        let mut markets = HashMap::new();
        markets.insert(
            "SOL/USDC".to_string(),
            "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT".to_string(),
        );

        Self {
            rpc_client,
            markets,
        }
    }

    pub async fn get_market_state(&self, market_address: &str) -> Result<MarketState> {
        let market_pubkey = Pubkey::from_str(market_address)?;
        let account = self.rpc_client.get_account(&market_pubkey)?;
        MarketState::from_bytes(&account.data)
    }

    pub async fn get_depth(&self, market_address: &str, depth_level: usize) -> Result<MarketDepth> {
        let market_state = self.get_market_state(market_address).await?;

        // 获取订单簿账户数据
        let bids_account = self.rpc_client.get_account(&market_state.bids)?;
        let asks_account = self.rpc_client.get_account(&market_state.asks)?;

        // 解析订单簿
        let (mut bids, mut asks) = self.parse_orderbook(&bids_account.data, &asks_account.data)?;

        // 排序
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()); // 买单降序
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()); // 卖单升序

        // 计算累计数量
        self.calculate_totals(&mut bids);
        self.calculate_totals(&mut asks);

        // 截取指定深度
        let bids: Vec<Level> = bids.into_iter().take(depth_level).collect();
        let asks: Vec<Level> = asks.into_iter().take(depth_level).collect();

        // 计算统计数据
        let total_bid_size: f64 = bids.iter().map(|level| level.size).sum();
        let total_ask_size: f64 = asks.iter().map(|level| level.size).sum();
        let spread = if !bids.is_empty() && !asks.is_empty() {
            asks[0].price - bids[0].price
        } else {
            0.0
        };

        Ok(MarketDepth {
            bids,
            asks,
            spread,
            total_bid_size,
            total_ask_size,
        })
    }

    fn parse_orderbook(&self, bids_data: &[u8], asks_data: &[u8]) -> Result<(Vec<Level>, Vec<Level>)> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        self.parse_orders(bids_data, true, &mut bids)?;
        self.parse_orders(asks_data, false, &mut asks)?;

        Ok((bids, asks))
    }

    fn parse_orders(&self, data: &[u8], _is_bids: bool, orders: &mut Vec<Level>) -> Result<()> {
        if data.len() < 5 {
            return Ok(());
        }

        let data = &data[5..]; // 跳过头部

        for chunk in data.chunks(32) {
            if chunk.len() < 16 {
                break;
            }

            if let Ok(price_bytes) = chunk[0..8].try_into() {
                if let Ok(size_bytes) = chunk[8..16].try_into() {
                    let price = u64::from_le_bytes(price_bytes) as f64 / 1_000_000.0;
                    let size = u64::from_le_bytes(size_bytes) as f64 / 1_000_000.0;

                    if price > 0.0 && size > 0.0 {
                        orders.push(Level {
                            price,
                            size,
                            total: 0.0,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn calculate_totals(&self, levels: &mut Vec<Level>) {
        let mut running_total = 0.0;
        for level in levels.iter_mut() {
            running_total += level.size;
            level.total = running_total;
        }
    }

    pub fn print_depth(&self, depth: &MarketDepth) {
        println!("\n市场深度信息:");
        println!("买卖价差: {:.6} USDC", depth.spread);
        println!("买单总量: {:.6}", depth.total_bid_size);
        println!("卖单总量: {:.6}", depth.total_ask_size);

        println!("\n买单深度 (价格降序):");
        println!("{:<15} {:<15} {:<15}", "价格", "数量", "累计数量");
        println!("-----------------------------------------");
        for level in &depth.bids {
            println!("{:<15.6} {:<15.6} {:<15.6}", 
                level.price, level.size, level.total);
        }

        println!("\n卖单深度 (价格升序):");
        println!("{:<15} {:<15} {:<15}", "价格", "数量", "累计数量");
        println!("-----------------------------------------");
        for level in &depth.asks {
            println!("{:<15.6} {:<15.6} {:<15.6}", 
                level.price, level.size, level.total);
        }
    }
}