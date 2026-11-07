# AGENTS.md

## Schema versions

thaliak, thaliak-timeline-linter and thaliak-diagram-editor cache this tool's JSON output and refetch a cached row only
if its `schemaVersion` is missing or older than the version of the ironworks-cli they run. A row with a *newer* version
is accepted, because several consumers on different ironworks-cli versions can share one cache directory.

- Every sheet in `SHEET_COLUMNS` (`src/data/sheets.rs`) has a `schema_version`. It **must** be incremented whenever the
  JSON output of that sheet can change for any row: its columns, its links, derived fields (such as
  `AnimationEndDuration` in `src/data/sheet_extractor.rs`) or how its values are written.
- A change to output logic shared by all sheets (`KeyValues::write`, key casing, ordering)
  increments the `schema_version` of **every** sheet.
- At most one increment per commit. Changes that no consumer ever saw don't need their own.
- A change may only add fields or change values. Removing or renaming a field also needs every consumer updated,
  since an older consumer accepts the newer row and won't find the field.
- A new sheet starts at 1.

### The guard test

`cargo test` runs `schema_versions_track_output` (`src/data/sheets.rs`). It compares each sheet's columns and links with
`schema-versions.json` as of the last commit (`git show HEAD:schema-versions.json`):

- columns or links changed: the version must be exactly one above the committed one;
- nothing changed: the version may stay or go up by one (for value changes);
- never more than one above the committed version, never lower.

It then rewrites `schema-versions.json` in the working tree. **Commit that file with the change.**

What it can't see: derived fields and changed values. Those need the increment by hand. Also, if you install an
uncommitted build (`cargo install --path .`), let a consumer cache its output, and then change the output again before
committing, those cached rows stay stale: increment again (in a separate commit) or clear the consumer's cache.
