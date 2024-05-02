use anyhow::{anyhow, Context, Result};
use governor::Quota;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{collections::HashMap, env::args, io::stdout, num::NonZeroU32, path::Path, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let (org_file, res_file) = args().skip(1).collect_tuple().context("expected org file and res file as arguments")?;
    let orgs = build_org_lookup(&org_file)?;
    let mut reader = csv::Reader::from_path(&res_file)?;
    let mut writer = csv::Writer::from_writer(stdout());
    writer.write_record(["Author", "Email", "Department", "DOI"])?;
    for record in reader.records() {
        let record = record?;
        let doi = match doi_lookup(&record[4]).await {
            Ok(doi) => doi,
            Err(err) => {
                eprintln!("{err}");
                eprintln!("{record:?}");
                continue;
            }
        };
        writer.write_record([
            &format!("{}, {}", &record[2], &record[1]),
            &record[6],
            &orgs[&record[3]],
            &doi,
        ])?;
        writer.flush()?;
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

async fn doi_lookup(uid: &str) -> Result<String> {
    lazy_static! {
        static ref CLIENT: reqwest::Client = reqwest::Client::new();
        static ref APIKEY: String = std::env::var("WOS_APIKEY").expect("missing web of science starter api key");
        static ref RATE_LIMITER: RateLimiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
    }
    if uid.is_empty() {
        return Err(anyhow!("record missing UID"));
    }
    loop {
        RATE_LIMITER.until_ready().await;
        let response = CLIENT
            .get(format!("https://api.clarivate.com/apis/wos-starter/v1/documents/{uid}"))
            .header("X-ApiKey", &*APIKEY)
            .send()
            .await?
            .text()
            .await?;
        let json = json::parse(&response)?;
        if json["message"] == "API rate limit exceeded" {
            eprintln!("daily rate limit reached. see you tomorrow");
            sleep(Duration::from_secs(86400)).await;
        } else {
            return Ok(json["identifiers"]["doi"].as_str().context("response missing DOI")?.to_owned());
        }
    }
}
