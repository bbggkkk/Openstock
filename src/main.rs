use clap::{Parser, Subcommand};
use commands::account::AccountCommands;
use commands::api::ApiCommands;
use commands::cache::CacheCommands;
use commands::dart::DartCommands;
use commands::market::MarketCommand;
use commands::order::OrderCommands;
use commands::universe::UniverseCommands;

mod apis;
mod commands;
mod core;
mod providers;

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    about = "CLI로 사용하는 증권 API",
    version = env!("CARGO_PKG_VERSION"),
    help_template = "{name}:: {about}\n 사용방법:: {usage}\n\n{all-args}"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 애플리케이션 버젼 표시
    Version,

    /// 증권사 API 리스트 조회 및 API 설정
    Api {
        #[command(subcommand)]
        sub: ApiCommands,
    },

    /// OpenDART 공시정보 조회
    Dart {
        #[command(subcommand)]
        sub: DartCommands,
    },

    /// 계좌 조회 및 관리
    Account {
        #[command(subcommand)]
        sub: AccountCommands,
    },

    /// 로컬 캐시 상태 조회 및 용량 정리
    Cache {
        #[command(subcommand)]
        sub: CacheCommands,
    },

    /// 주문 실행 및 조회
    Order {
        #[command(subcommand)]
        sub: OrderCommands,
    },

    /// 종목 universe 캐시 구축 및 조회
    Universe {
        #[command(subcommand)]
        sub: UniverseCommands,
    },

    /// 종목 검색
    Search {
        /// 종목명 또는 종목코드
        query: String,
    },

    /// 종목 정보 및 기업정보 조회
    Market(MarketCommand),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Version) => commands::handle_version(),
        Some(Commands::Api { sub }) => commands::handle_api(sub),
        Some(Commands::Dart { sub }) => commands::handle_dart(sub),
        Some(Commands::Account { sub }) => commands::handle_account(sub),
        Some(Commands::Cache { sub }) => commands::handle_cache(sub),
        Some(Commands::Order { sub }) => commands::handle_order(sub),
        Some(Commands::Universe { sub }) => commands::handle_universe(sub),
        Some(Commands::Search { query }) => commands::handle_search(query),
        Some(Commands::Market(command)) => commands::handle_market(command),
        None => {
            println!(
                "{}",
                core::output::explained(
                    "help",
                    "명령이 지정되지 않았을 때의 사용 안내",
                    vec![
                        core::output::field(
                            "program",
                            "실행한 CLI 프로그램 이름",
                            serde_json::json!(env!("CARGO_PKG_NAME")),
                        ),
                        core::output::field(
                            "usage",
                            "도움말을 확인하는 명령",
                            serde_json::json!(format!("{} --help", env!("CARGO_PKG_NAME"))),
                        ),
                    ],
                )
            );
        }
    }
}
