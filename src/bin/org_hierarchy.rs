use std::{collections::HashMap, io::{Write, stdout}};
use anyhow::{Context, Result};
use itertools::Itertools;

struct Org {
    name: String,
    children: Vec<String>,
}

fn main() -> Result<()> {
    let org_file = std::env::args().nth(1).context("expected org file as argument")?;
    let mut reader = csv::Reader::from_path(&org_file)?;
    let mut orgs = HashMap::new();
    let mut top_level_id: Option<String> = None;

    for record in reader.records() {
        let record = record?;
        orgs.insert(
            record[0].to_owned(),
            Org {
                name: record[1].to_owned(),
                children: Vec::new(),
            },
        );
        if top_level_id.is_none() {
            top_level_id = Some(record[0].to_owned());
        }
        if let Some(parent) = orgs.get_mut(&record[2]) {
            parent.children.push(record[0].to_owned());
        }
    }

    let mut writer = csv::Writer::from_writer(stdout());
    writer.write_record([
        "Top Level",
        "Level 1",
        "Level 2",
        "Level 3",
        "Level 4",
        "Level 5",
    ])?;
    build_hierarchy(&mut writer, &orgs, &orgs[&top_level_id.unwrap()], &mut Vec::new())
}

fn build_hierarchy(writer: &mut csv::Writer<impl Write>, all_orgs: &HashMap<String, Org>, org: &Org, state: &mut Vec<String>) -> Result<()> {
    static EMPTY: String = String::new();
    state.push(org.name.clone());
    writer.write_record(state.iter().pad_using(6, |_| &EMPTY))?;
    for child in &org.children {
        let child_org = &all_orgs[child];
        build_hierarchy(writer, all_orgs, child_org, state)?;
    }
    state.pop();    
    Ok(())
}