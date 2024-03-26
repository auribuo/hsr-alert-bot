use std::process::exit;
use std::time::Duration;

use scraper::{Html, Selector};
use tokio::sync::mpsc::Sender;

use anyhow::Result;

async fn retrieve_page(url: &'static str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let page_data = response.text().await?;
    Ok(page_data)
}

fn scrape_codes(page: &str) -> Result<Vec<String>> {
    let html = Html::parse_document(page);
    let code_container_selector = Selector::parse("div.codes").unwrap();

    let code_container = html
        .select(&code_container_selector)
        .next()
        .expect("Page broken: No codes div found");

    let code_count = code_container.child_elements().count();
    let mut codes = Vec::with_capacity(code_count);

    for dv in code_container.child_elements() {
        let code = dv.text().next();
        if let Some(code) = code {
            codes.push(code.to_string());
        }
    }

    Ok(codes)
}

pub async fn run(tx: Sender<Vec<String>>, interval: u64) {
    loop {
        let page: String;
        if let Ok(np) = retrieve_page("https://www.prydwen.gg/star-rail/").await {
            page = np;
        } else {
            error!("Could not fetch page.");
            exit(1);
        }
        match scrape_codes(&page) {
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
