
use solana_client::rpc_client::RpcClient;
pub struct RaydiumPriceFetcher{
    rpc_client: RpcClient,
}
impl RaydiumPriceFetcher {
    fn new(url: &str) -> RaydiumPriceFetcher {
        Self{
            rpc_client: RpcClient::new(url),
        }
    }
}