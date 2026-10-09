use serde::{Deserialize, Serialize};

pub type AppResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Console {
    pub id: i64,
    pub name: String,
    pub hourly_price: i64,
    pub controller_price: i64,
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleInput {
    pub id: Option<i64>,
    pub name: String,
    pub hourly_price: i64,
    pub controller_price: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub console_id: i64,
    pub console_name: String,
    pub start_time: String,
    pub pause_time: Option<String>,
    pub end_time: Option<String>,
    pub paid_at: Option<String>,
    pub controllers: i64,
    pub status: String,
    pub hourly_price: i64,
    pub controller_price: i64,
    pub paused_ms: i64,
    pub checkout_at: Option<String>,
    pub checkout_previous_status: Option<String>,
    pub game_cost: Option<i64>,
    pub shop_cost: Option<i64>,
    pub total_price: Option<i64>,
    pub legacy: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartInput {
    pub console_id: i64,
    pub controllers: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllersInput {
    pub session_id: i64,
    pub controllers: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionItem {
    pub id: i64,
    pub session_id: i64,
    pub product_id: Option<i64>,
    pub name: String,
    pub unit_price: i64,
    pub quantity: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: i64,
    pub stock: i64,
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProductInput {
    pub id: Option<i64>,
    pub name: String,
    pub price: i64,
    pub stock: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProductInput {
    pub session_id: i64,
    pub product_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemQuantityInput {
    pub item_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub session: Session,
    pub items: Vec<SessionItem>,
    pub played_ms: i64,
    pub game_cost: i64,
    pub shop_cost: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleCard {
    pub console: Console,
    pub sessions: Vec<Invoice>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFilter {
    pub from: Option<String>,
    pub to: Option<String>,
    pub search: Option<String>,
    pub console_id: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub console_id: i64,
    pub console_name: String,
    pub start_time: String,
    pub end_time: String,
    pub paid_at: String,
    pub played_ms: i64,
    pub game_cost: i64,
    pub shop_cost: i64,
    pub total: i64,
    pub legacy: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub sessions: i64,
    pub game_revenue: i64,
    pub shop_revenue: i64,
    pub total_revenue: i64,
    pub played_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleReport {
    pub console_id: i64,
    pub name: String,
    pub sessions: i64,
    pub played_ms: i64,
    pub revenue: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub day: String,
    pub sessions: i64,
    pub game_revenue: i64,
    pub shop_revenue: i64,
    pub total_revenue: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub summary: ReportSummary,
    pub consoles: Vec<ConsoleReport>,
    pub days: Vec<DailyReport>,
    pub page: i64,
    pub pages: i64,
}
