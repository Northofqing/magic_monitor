## 基础结构

| Solana 套利系统|  |
| ----------- | ----------- |
| 数据存储层（storage） | PostgreSQL / TimescaleDB       |
| 监控层（monitor） | Prometheus / Grafana         |
| 交易执行引擎（trade） | Solana SDK / Jupiter API         |
| 策略执行层 （strategy） |  跨池套利 / 三角套利 / MEV 规避  |
| 数据分析层 （analysis）|  Python (Pandas, NumPy)          |
| 数据获取层 （collection） |  Solana RPC / Serum SDK / Raydium SDK / Orca SDK |


install rust<br />
`$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`<br />
or<br />
`https://www.rust-lang.org/tools/install`<br />

build project<br />
`cargo build`<br />

run project<br />
`cargo run`<br />
 