use anyhow::{Context, Result};
use governor::Quota;
use itertools::{join, Itertools};
use lazy_static::lazy_static;
use std::{collections::HashMap, env::args, fmt::Display, io::stdout, num::NonZeroU32, path::Path};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let (org_file, res_file) = args().skip(1).collect_tuple().context("expected org file and res file as arguments")?;
    let orgs = build_org_lookup(&org_file)?;
    let mut reader = csv::Reader::from_path(&res_file)?;
    let mut writer = csv::Writer::from_writer(stdout());
    writer.write_record(["Author", "Email", "Department", "DOI"])?;
    for batch in &reader.records().flatten().filter(|r| !r[4].is_empty()).chunks(BATCH_SIZE) {
        let records = batch.collect_vec();
        let dois = doi_batch_lookup(records.iter().map(|r| &r[4])).await?;
        for record in records {
            if let Some(doi) = dois.get(&record[4]) {
                writer.write_record([
                    &format!("{}, {}", &record[2], &record[1]),
                    &record[6],
                    &orgs[&record[3]],
                    &doi,
                ])?;
                writer.flush()?;
            }
        }
    }
    Ok(())
}

fn build_org_lookup(org_file: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    let mut reader = csv::Reader::from_path(org_file)?;
    let mut map = HashMap::new();
    for record in reader.records() {
        let record = record?;
        map.insert(record[0].to_owned(), record[1].to_owned());
    }
    Ok(map)
}

type RateLimiter = governor::RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::QuantaClock,
    governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
>;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
    static ref APIKEY: String = std::env::var("WOS_APIKEY").expect("missing web of science starter api key");
    static ref RATE_LIMITER: RateLimiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
}

const BATCH_SIZE: usize = 50;

async fn doi_batch_lookup(uids: impl IntoIterator<Item = impl Display>) -> Result<HashMap<String, String>> {
    let query = format!("UT=({})", join(uids, " "));
    RATE_LIMITER.until_ready().await;
    let response = CLIENT
        .get("https://api.clarivate.com/apis/wos-starter/v1/documents")
        .query(&[("limit", BATCH_SIZE.to_string()), ("q", query)])
        .header("X-ApiKey", &*APIKEY)
        .send()
        .await?;
    let json = json::parse(&response.text().await?)?;
    let mut map = HashMap::new();
    for hit in json["hits"].members() {
        if let Some(doi) = hit["identifiers"]["doi"].as_str() {
            map.insert(hit["uid"].as_str().unwrap().to_owned(), doi.to_owned());
        }
    }
    Ok(map)
}
