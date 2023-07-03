# InCites to Altmetric Publications Tool

These two programs take the CSVs outputted by InCites and convert them to a format suitable for upload into Altmetric. Both send the output csv to stdout.

# Usage

## Organization Hierarchy

`cargo run --bin org_hierarchy OrgHierarchy.csv`

## Publications

`cargo run --bin publications OrgHierarchy.csv PeopleDocument.csv`