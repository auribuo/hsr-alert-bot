use std::time::Duration;

use tokio::sync::mpsc::Sender;

use anyhow::Result;
use chrono::Days;
use scraper::{ElementRef, Html, Selector};

static BASE_URL: &'static str = "https://game8.co/games/Honkai-Star-Rail/archives/410296";
static VERSION_INFO_URL: &'static str = "https://honkai-star-rail.fandom.com/wiki/Version";

async fn retrieve_page(url: &'static str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let page_data = response.text().await?;
    Ok(page_data)
}

async fn retrieve_valid_codes() -> Result<Vec<String>> {
    let html = Html::parse_document(retrieve_page(BASE_URL).await?.as_str());

    let lists_selector = Selector::parse("ul.a-list").unwrap();
    let mut lists_result = html.select(&lists_selector);

    let headings_selector = Selector::parse("h2.a-header--2").unwrap();
    let mut headings_result = html.select(&headings_selector);

    let mut version_codes: Vec<String> = extract_codes(lists_result.next().unwrap())?
        .iter()
        .map(|x| x.trim().to_string())
        .collect();
    let current_codes: Vec<String> = extract_codes(lists_result.next().unwrap())?
        .iter()
        .map(|x| x.trim().to_string())
        .collect();

    let game_version = extract_version(
        headings_result
            .next()
            .unwrap()
            .text()
            .into_iter()
            .fold("".to_string(), |acc, e| acc + e),
    );
    let mut valid_codes = current_codes;
    let version_page = retrieve_page(VERSION_INFO_URL).await?;
    if current_version_codes_valid(version_page, game_version)? {
        valid_codes.append(&mut version_codes);
    }
    Ok(valid_codes)
}

fn extract_codes(html: ElementRef) -> Result<Vec<String>> {
    let b_selector = Selector::parse("b").unwrap();
    let b_result = html.select(&b_selector);

    return Ok(b_result
        .into_iter()
        .map(|x| x.text().map(|y| y).fold("".to_string(), |acc, e| acc + e))
        .collect());
}

fn extract_version(heading: String) -> String {
    let rx = regex::Regex::new(r".*[0-9]*\.[0-9]").unwrap();

    let Some(caps) = rx.captures(&heading) else {
        return "".to_string();
    };
    return caps[0].to_string();
}

fn current_version_codes_valid(page: String, game_version: String) -> Result<bool> {
    let html = Html::parse_document(page.as_str());
    let table_selector = Selector::parse("table").unwrap();
    let version_table = html.select(&table_selector).next().unwrap();
    let versions_selector = Selector::parse("tr").unwrap();
    let versions = version_table.select(&versions_selector);
    let mut version_infos: Vec<(String, String)> = vec![];
    let cells_selector = Selector::parse("td").unwrap();
    for version in versions.skip(1) {
        let mut cells = version.select(&cells_selector);
        let version_string = cells
            .next()
            .unwrap()
            .text()
            .fold("".to_string(), |acc, e| acc + e);
        cells.next();
        let version_date_string = cells
            .next()
            .unwrap()
            .text()
            .fold("".to_string(), |acc, e| acc + e);
        version_infos.push((version_string, version_date_string));
    }
    version_infos = version_infos
        .iter_mut()
        .map(|x| (x.0.trim().to_string(), x.1.trim().to_string()))
        .collect();

    let future_version = (
        "".to_string(),
        chrono::Local::now()
            .date_naive()
            .checked_add_days(Days::new(14))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
    );

    let (_, current_version_release) = version_infos
        .iter()
        .find(|&x| x.0 == game_version)
        .or(Some(&future_version))
        .unwrap();
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

pub async fn run(
    tx: Sender<Vec<String>>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    interval: u64,
) {
    loop {
        if shutdown.has_changed().unwrap() && *shutdown.borrow_and_update() {
            break;
        }
        match retrieve_valid_codes().await {
            Ok(data) => {
                tracing::info!("Retrieved {} valid codes. Sending to shards", &data.len());
                tracing::info!("Valid codes are: {:?}", &data);
                tx.send(data).await.unwrap();
            }
            Err(err) => {
                tracing::error!("Error: {}", err);
            }
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
