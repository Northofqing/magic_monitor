use anyhow::Result;
use chrono::NaiveDateTime;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
};
use crate::dex_collect::serum::serum_depth::MarketDepthFetcher;
#[derive(Debug)]
pub struct SerumMarketState {
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

impl SerumMarketState {
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
/// 价格详情结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceDetails {
    pub base_symbol: String,      // 基础代币符号
    pub quote_symbol: String,     // 计价代币符号
    pub price: f64,               // 当前价格
    pub high_24h: f64,            // 24小时最高价
    pub low_24h: f64,             // 24小时最低价
    pub volume_24h: f64,          // 24小时交易量
    pub bid: f64,                 // 最佳买价
    pub ask: f64,                 // 最佳卖价
    pub spread: f64,              // 买卖价差
    pub timestamp: DateTime<Utc>, // 时间戳
}

/// 市场深度结构
#[derive(Debug, Clone)]
pub struct MarketDepth {
    pub bids: Vec<(f64, f64)>,    // 买单 (价格, 数量)
    pub asks: Vec<(f64, f64)>,    // 卖单 (价格, 数量)
    pub timestamp: DateTime<Utc>, // 时间戳
}
/// 深度级别结构
#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub size: f64,
    pub total: f64,
}
/// 价格追踪器
#[derive(Clone)]
struct PriceTracker {
    prices: VecDeque<(f64, DateTime<Utc>)>, // 价格历史
    max_size: usize,                        // 最大历史记录数
}

impl PriceTracker {
    fn new(max_size: usize) -> Self {
        Self {
            prices: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add_price(&mut self, price: f64, timestamp: DateTime<Utc>) {
        if self.prices.len() >= self.max_size {
            self.prices.pop_front();
        }
        self.prices.push_back((price, timestamp));
    }

    fn get_high_low_24h(&self) -> (f64, f64) {
        let day_ago = Utc::now() - chrono::Duration::days(1);
        let recent_prices: Vec<f64> = self
            .prices
            .iter()
            .filter(|(_, ts)| *ts > day_ago)
            .map(|(price, _)| *price)
            .collect();

        if recent_prices.is_empty() {
            return (0.0, 0.0);
        }

        let high = recent_prices
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let low = recent_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        (high, low)
    }
}

pub struct SerumPriceFetcher {
    rpc_client: RpcClient,
    markets: HashMap<String, String>,
    price_trackers: HashMap<String, PriceTracker>,
}

impl SerumPriceFetcher {
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
        markets.insert(
            "BTC/USDC".to_string(),
            "A8YFbxQYFVqKZaoYJLLUVcQiWP7G2MeEgW5wsAQgMvFw".to_string(),
        );
        markets.insert(
            "ETH/USDC".to_string(),
            "4tSvZvnbyzHXLMTiFonMyxZoHmFqau1XArcRCVHLZ5gX".to_string(),
        );

        let price_trackers = markets
            .keys()
            .map(|k| (k.clone(), PriceTracker::new(1440))) // 存储24小时的分钟数据
            .collect();

        Self {
            rpc_client,
            markets,
            price_trackers,
        }
    }
    pub async fn get_account(&self, market_address: &str) -> Result<solana_sdk::account::Account> {
        let market_pubkey = Pubkey::from_str(market_address)?;
        let account = self.rpc_client.get_account(&market_pubkey)?;
        Ok(account)
    }
   
    /// 获取实时价格详情
    pub async fn get_price_details(&mut self, market_pair: &str) -> Result<PriceDetails> {
        let market_address = self
            .markets
            .get(market_pair)
            .ok_or_else(|| anyhow::anyhow!("Unsupported market pair"))?;

        let (bids, asks) = self.get_orderbook(market_address).await?;

        let bid = bids.first().map(|(price, _)| *price).unwrap_or(0.0);
        let ask = asks.first().map(|(price, _)| *price).unwrap_or(0.0);
        let price = (bid + ask) / 2.0;
        let spread = ask - bid;

        let timestamp = Utc::now();

        // 更新价格追踪器
        if let Some(tracker) = self.price_trackers.get_mut(market_pair) {
            tracker.add_price(price, timestamp);
        }

        // 获取24小时高低价
        let (high_24h, low_24h) = self
            .price_trackers
            .get(market_pair)
            .map(|t| t.get_high_low_24h())
            .unwrap_or((0.0, 0.0));

        // 计算24小时交易量
        let volume_24h = self.calculate_volume_24h(market_address).await?;

        let (base, quote) = market_pair
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("Invalid market pair format"))?;

        Ok(PriceDetails {
            base_symbol: base.to_string(),
            quote_symbol: quote.to_string(),
            price,
            high_24h,
            low_24h,
            volume_24h,
            bid,
            ask,
            spread,
            timestamp,
        })
    }

    /// 获取市场状态
    pub async fn get_market_state(&self, market_address: &str) -> Result<SerumMarketState> {
        let market_pubkey = Pubkey::from_str(market_address)?;
        let account = self.rpc_client.get_account(&market_pubkey)?;
        SerumMarketState::from_bytes(&account.data)
    }
    /// 获取市场深度
    pub  async fn get_orderbook(
        &self,
        market_address: &str,
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        // 原有的订单簿获取逻辑
        let marker = MarketDepthFetcher::new();
        let depth = marker
            .get_depth(market_address, 1)
            .await?;
        marker.print_depth(&depth);

        let mut bids = vec![];
        for level in &depth.bids {
            bids.push((level.price, level.size));
            println!("{:<15.6} {:<15.6} {:<15.6}", 
                level.price, level.size, level.total);
        }
        let mut asks = vec![];
        for level in &depth.asks {
            asks.push((level.price, level.size));
            println!("{:<15.6} {:<15.6} {:<15.6}", 
                level.price, level.size, level.total);
        }
        Ok((bids, asks)) // 临时返回，需要实现实际逻辑
    }
    fn parse_orderbook(
        &self,
        bids_data: &[u8],
        asks_data: &[u8],
    ) -> Result<(Vec<Level>, Vec<Level>)> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        self.parse_orders(bids_data, true, &mut bids)?;
        self.parse_orders(asks_data, false, &mut asks)?;

        Ok((bids, asks))
    }
    fn parse_orders(&self, data: &[u8], is_bids: bool, orders: &mut Vec<Level>) -> Result<()> {
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

    /// 计算24小时交易量
    async fn calculate_volume_24h(&self, market_address: &str) -> Result<f64> {
        // ... 实现交易量计算逻辑 ...
        Ok(0.0) // 临时返回，需要实现实际逻辑
    }

    /// 获取所有支持的市场对
    pub fn get_supported_markets(&self) -> Vec<String> {
        self.markets.keys().cloned().collect()
    }

    /// 监控价格变化
    pub async fn monitor_price(&mut self, market_pair: &str) -> Result<()> {
        println!("开始监控 {:?} 价格变化...", market_pair);

        let mut last_price = 0.0;
        loop {
            match self.get_price_details(market_pair).await {
                Ok(details) => {
                    //if (details.price - last_price).abs() > 0.01 {
                        println!("\n价格更新 - {:?}", market_pair);
                        println!("时间: {:?}", details.timestamp);
                        println!("当前价格: {:?} {}", details.price, details.quote_symbol);
                        println!("买价: {} {}", details.bid, details.quote_symbol);
                        println!("卖价: {} {}", details.ask, details.quote_symbol);
                        println!("价差: {} {}", details.spread, details.quote_symbol);
                        println!("24h高: {} {}", details.high_24h, details.quote_symbol);
                        println!("24h低: {} {}", details.low_24h, details.quote_symbol);
                        println!("24h成交量: {} {}", details.volume_24h, details.base_symbol);

                        last_price = details.price;
                    //}
                }
                Err(e) => println!("获取价格失败: {}", e),
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
