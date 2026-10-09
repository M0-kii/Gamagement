use crate::{
    database::{err, invoice_at, parse_time},
    models::*,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::BTreeMap;

const PAGE_SIZE: usize = 25;

pub fn history_entries(conn: &Connection, filter: &HistoryFilter) -> AppResult<Vec<HistoryEntry>> {
    let from = filter.from.as_deref().map(parse_time).transpose()?;
    let to = filter.to.as_deref().map(parse_time).transpose()?;
    if let (Some(a), Some(b)) = (from, to) {
        if a >= b {
            return Err("بازه تاریخ نامعتبر است".into());
        }
    }
    let search = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
    if search.len() > 300 {
        return Err("عبارت جست‌وجو طولانی است".into());
    }
    let mut q=conn.prepare("SELECT id FROM sessions WHERE status='ended' AND (? IS NULL OR julianday(paidAt)>=julianday(?)) AND (? IS NULL OR julianday(paidAt)<julianday(?)) AND (? IS NULL OR consoleId=?) ORDER BY paidAt DESC,id DESC").map_err(err)?;
    let rows = q
        .query_map(
            params![
                filter.from,
                filter.from,
                filter.to,
                filter.to,
                filter.console_id,
                filter.console_id
            ],
            |r| r.get::<_, i64>(0),
        )
        .map_err(err)?;
    let ids = rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)?;
    ids.into_iter()
        .filter_map(|id| {
            let invoice = invoice_at(conn, id, Utc::now());
            match invoice {
                Ok(i) => {
                    if !search.is_empty()
                        && !i.session.console_name.to_lowercase().contains(&search)
                        && !i.session.id.to_string().contains(&search)
                    {
                        return None;
                    }
                    Some(Ok(HistoryEntry {
                        id: i.session.id,
                        console_id: i.session.console_id,
                        console_name: i.session.console_name,
                        start_time: i.session.start_time,
                        end_time: i.session.end_time.unwrap_or_default(),
                        paid_at: i.session.paid_at.unwrap_or_default(),
                        played_ms: i.played_ms,
                        game_cost: i.game_cost,
                        shop_cost: i.shop_cost,
                        total: i.total,
                        legacy: i.session.legacy,
                    }))
                }
                Err(e) => Some(Err(e)),
            }
        })
        .collect()
}
pub fn get_history(conn: &Connection, filter: HistoryFilter) -> AppResult<HistoryPage> {
    let all = history_entries(conn, &filter)?;
    let mut summary = ReportSummary {
        sessions: all.len() as i64,
        game_revenue: 0,
        shop_revenue: 0,
        total_revenue: 0,
        played_ms: 0,
    };
    let mut consoles: BTreeMap<i64, ConsoleReport> = BTreeMap::new();
    let mut days: BTreeMap<String, DailyReport> = BTreeMap::new();
    for e in &all {
        summary.game_revenue = summary
            .game_revenue
            .checked_add(e.game_cost)
            .ok_or("مبلغ گزارش بیش از حد مجاز است")?;
        summary.shop_revenue = summary
            .shop_revenue
            .checked_add(e.shop_cost)
            .ok_or("مبلغ گزارش بیش از حد مجاز است")?;
        summary.total_revenue = summary
            .total_revenue
            .checked_add(e.total)
            .ok_or("مبلغ گزارش بیش از حد مجاز است")?;
        summary.played_ms = summary
            .played_ms
            .checked_add(e.played_ms)
            .ok_or("مدت گزارش بیش از حد مجاز است")?;
        let c = consoles.entry(e.console_id).or_insert(ConsoleReport {
            console_id: e.console_id,
            name: e.console_name.clone(),
            sessions: 0,
            played_ms: 0,
            revenue: 0,
        });
        c.sessions += 1;
        c.played_ms += e.played_ms;
        c.revenue += e.total;
        let day = (parse_time(&e.paid_at)? + chrono::Duration::minutes(210))
            .format("%Y-%m-%d")
            .to_string();
        let d = days.entry(day.clone()).or_insert(DailyReport {
            day,
            sessions: 0,
            game_revenue: 0,
            shop_revenue: 0,
            total_revenue: 0,
        });
        d.sessions += 1;
        d.game_revenue += e.game_cost;
        d.shop_revenue += e.shop_cost;
        d.total_revenue += e.total;
    }
    let pages = all.len().div_ceil(PAGE_SIZE).max(1) as i64;
    let page = filter.page.unwrap_or(1).clamp(1, pages);
    let entries = all
        .into_iter()
        .skip((page as usize - 1) * PAGE_SIZE)
        .take(PAGE_SIZE)
        .collect();
    Ok(HistoryPage {
        entries,
        summary,
        consoles: consoles.into_values().collect(),
        days: days.into_values().rev().collect(),
        page,
        pages,
    })
}
fn cell(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    // Spreadsheet applications interpret these prefixes as formulas even in CSV.
    let prefix = if value
        .trim_start()
        .starts_with(['=', '+', '-', '@', '\t', '\r'])
    {
        "'"
    } else {
        ""
    };
    format!("\"{prefix}{escaped}\"")
}
pub fn export_history(conn: &Connection, filter: HistoryFilter) -> AppResult<String> {
    let entries = history_entries(conn, &filter)?;
    let mut csv=String::from("\u{feff}Session,Console,Started (UTC),Stopped (UTC),Paid (UTC),Played minutes,Game (toman),Shop (toman),Total (toman)\r\n");
    for e in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{:.2},{},{},{}\r\n",
            e.id,
            cell(&e.console_name),
            cell(&e.start_time),
            cell(&e.end_time),
            cell(&e.paid_at),
            e.played_ms as f64 / 60_000.0,
            e.game_cost,
            e.shop_cost,
            e.total
        ));
    }
    Ok(csv)
}
