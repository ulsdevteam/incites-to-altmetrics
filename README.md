# InCites to Altmetric Publications Tool

These two programs take the CSVs outputted by InCites and convert them to a format suitable for upload into Altmetric. Both send the output csv to stdout.

# Usage

The environment variable `WOS_APIKEY` is required to be set to the API key for the WOS Starter API.

## Organization Hierarchy

`incites_to_altmetrics_org_hierarchy.exe OrgHierarchy.csv`
or
`cargo run --bin org_hierarchy OrgHierarchy.csv`

## Publications

`incites_to_altmetrics_publications.exe OrgHierarchy.csv PeopleDocument.csv`
or
`cargo run --bin publications OrgHierarchy.csv PeopleDocument.csv`