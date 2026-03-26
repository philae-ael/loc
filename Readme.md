# LOC

A small util that computes the number of lines in a given dir. 

Output:
```md
┌──────────┬───────┬──────────┬───────┬───────┬────────────┐
│ Language │ Code  │ Comments │ Empty │ Total │ File count │
├──────────┼───────┼──────────┼───────┼───────┼────────────┤
│   Rust   │ 794   │ 8        │ 82    │ 884   │           6│
│   Text   │ 693   │ 0        │ 90    │ 783   │           1│
│ Markdown │ 20    │ 0        │ 5     │ 25    │           3│
│  Shell   │ 15    │ 0        │ 4     │ 19    │           1│
│   TOML   │ 14    │ 1        │ 3     │ 18    │           1│
│  Total   │ 1536  │ 9        │ 184   │ 1729  │          12│
└──────────┴───────┴──────────┴───────┴───────┴────────────┘
```