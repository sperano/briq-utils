# briq-utils

Utility for preparing the public data from [Rebrickable](https://rebrickable.com/downloads/) to be used with the [BRIQ](https://github.com/sperano/BRIQ) application.

## Preparation

Go to [Rebrickable](https://rebrickable.com/downloads/) and download the following CSV files in a work directory

- colors.csv
- inventories.csv
- inventory-minifigs.csv
- inventory-parts.csv
- minifigs.csv
- part-categories.csv
- parts.csv
- sets.csv
- themes.csv

## Building the utility 

```bash
cargo build --release
```

## Analyzing the data 

Run the command to analyze the data:

```bash
briq-utils analyze -w ~/my-workdir/
```

Example output:
```
Reading all CSV data...
Validating data...
78513pr0001: part does not exist
84720pr0001: part does not exist
Themes tree has a max depth of 3
There are 7 unique parts materials.
Converting data to BRIQ model...
Set Version 1: Ignoring part 84720pr0001: does not exist
Set Version 1: Ignoring part 78513pr0001: does not exist
Analyzing data...
1189 sets has more than 1 version (5.2% of sets). 204 sets has more than 2 versions (1.3%).
8612 sets (33.9%) has a parts_count mismatch.
```

## Generating data

To generate the initial datasets as well as the Swift code:

```bash
briq-utils generate -w ~/my-workdir/
```

Example output:
```bash
Reading all CSV data...
Generating Swift code...
Converting data to BRIQ model...
Set Version 1: Ignoring part 84720pr0001: does not exist
Set Version 1: Ignoring part 78513pr0001: does not exist
Generating JSON...
```

This will also have generated those files in the work directory:

- init.json
- PartCategories.swift
- PartColors.swift
- Themes.swift

## Mirror 

To have a local copy of all the images:

```bash
briq-utils mirror -w ~/my-workdir -c ~/Downloads/my-cache
```
