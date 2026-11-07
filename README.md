# ironworks-cli

A minimal CLI used by thaliak.com to fetch various FFXIV game data from FFXIV's own Excel sheets
via EXDSchema.

**Please note that this CLI tool only supports a small number of FFXIV's Excel sheets,
and even then only retrieves the columns needed by thaliak.com.
If you need more comprehensive access to FFXIV's game data, check out xivapi.com or Lumina instead.**

## Installation

This tool is deliberately not published on Cargo, for the reason outlined above.
If you'd like to install it anyway, run:

```bash
git clone git@github.com:thaliakcom/ironworks-cli.git
cd ironworks-cli
cargo install --path ./
```

## Schema versions

Every row's JSON starts with `schemaVersion`, the version of that sheet's output it was written with.
`ironworks-cli --schema-version` prints the current version of every sheet, e.g. `{"Action":8,"Status":3,...}`,
without needing a game path. Consumers that cache rows refetch those whose `schemaVersion` is missing or older
than the current one.
