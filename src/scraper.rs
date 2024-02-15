use std::process::exit;
use std::time::Duration;

use tokio::sync::mpsc::Sender;

use anyhow::Result;
use chrono::Days;
use soup::{NodeExt, QueryBuilderExt};

static BASE_URL: &'static str = "https://game8.co/games/Honkai-Star-Rail/archives/410296";
static VERSION_INFO_URL: &'static str = "https://honkai-star-rail.fandom.com/wiki/Version";

async fn retrieve_page(url: &'static str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let page_data = response.text().await?;
    Ok(page_data)
}

fn retrieve_valid_codes(page: &str, version_page: &str) -> Result<Vec<(String, bool)>> {
    let soup = soup::Soup::new(page);
    let lists = soup
        .class("a-list")
        .find_all()
        .take(2)
        .map(|node| {
            let bold = node
                .tag("b")
                .find_all()
                .map(|inner| inner.text())
                .collect::<Vec<_>>();
            if bold.is_empty() {
                node.tag("li")
                    .find_all()
                    .map(|line| line.tag("a").find().expect("Page differs from expected"))
                    .map(|a| a.text())
                    .collect::<Vec<_>>()
            } else {
                bold
            }
        })
        .map(|text| {
            text.iter()
                .map(|t| t.trim().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let current_codes = lists[0]
        .iter()
        .map(|code| (code.clone(), false))
        .collect::<Vec<_>>();
    let mut version_codes = lists[1]
        .iter()
        .map(|code| (code.clone(), true))
        .collect::<Vec<_>>();

    let mut valid_codes = current_codes;
    if current_version_codes_valid(version_page)? {
        valid_codes.append(&mut version_codes);
    }

    Ok(valid_codes)
}

fn current_version_codes_valid(page: &str) -> Result<bool> {
    let soup = soup::Soup::new(page);
    let latest_version_table = soup
        .tag("table")
        .find_all()
        .take(1)
        .find(|_| true)
        .expect("Page differs from expected");
    let current_version_release = &latest_version_table
        .tag("tr")
        .find_all()
        .skip(1)
        .map(|row| {
            row.tag("td")
                .find_all()
                .take(3)
                .map(|cell| cell.text().trim().to_string())
                .collect::<Vec<_>>()
        })
        .skip_while(|cells| cells[2] == "Unknown")
        .find(|_| true)
        .map_or_else(
            || {
                chrono::Local::now()
                    .date_naive()
                    .checked_add_days(Days::new(14))
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string()
            },
            |ok| ok[2].clone(),
        );

    let current_version_release_date =
        chrono::NaiveDate::parse_from_str(&current_version_release, "%Y-%m-%d").unwrap();
    let current_version_livestream = current_version_release_date
        .checked_sub_days(Days::new(12))
        .unwrap();
    return Ok(chrono::Local::now()
        .date_naive()
        .signed_duration_since(current_version_livestream)
        < chrono::Duration::days(1));
}

pub async fn run(tx: Sender<Vec<(String, bool)>>, interval: u64) {
    loop {
        let page: String;
        let version_page: String;
        if let Ok(p) = retrieve_page(BASE_URL).await {
            page = p;
        } else {
            error!("Could not fetch page.");
            exit(1);
        }
        if let Ok(vp) = retrieve_page(VERSION_INFO_URL).await {
            version_page = vp;
        } else {
            error!("Could not fetch page.");
            exit(1);
        }
        match retrieve_valid_codes(&page, &version_page) {
            Ok(data) => {
                info!(
                    amount = &data.len(),
                    "Retrieved valid codes. Sending to shards"
                );
                info!(codes=?&data, "Valid codes");
                tx.send(data).await.unwrap();
            }
            Err(err) => {
                error!(reason = err.to_string(), "Could not retrieve valid codes");
            }
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
