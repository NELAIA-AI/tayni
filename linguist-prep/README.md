# TAYNI GitHub Linguist Submission

This folder contains everything needed to submit TAYNI to GitHub Linguist
for official language recognition.

## Requirements Status

| Requirement | Status | Notes |
|-------------|--------|-------|
| TextMate Grammar | ✅ Ready | `grammars/TAYNI.tmLanguage.json` |
| MIT License | ✅ Ready | Repository is MIT licensed |
| Real-world Samples | ✅ Ready | `samples/TAYNI/` (3 files) |
| languages.yml Entry | ✅ Ready | `languages.yml.snippet` |
| 200 Unique Repos | ❌ Pending | Need community adoption |

## When to Submit

Submit the PR to `github-linguist/linguist` when:
1. At least 200 unique `user/repo` combinations have `.tayni` files
2. Check with: `https://github.com/search?q=extension%3Anela&type=code`

## Submission Steps

1. Fork `github-linguist/linguist`
2. Add grammar:
   ```bash
   script/add-grammar https://github.com/TAYNI/TAYNI-grammar
   ```
3. Copy samples to `samples/TAYNI/`
4. Add entry to `lib/linguist/languages.yml` (use `languages.yml.snippet`)
5. Run `script/update-ids` to generate language_id
6. Open PR with template filled out

## Files

```
linguist-prep/
├── README.md                    # This file
├── languages.yml.snippet        # Entry for languages.yml
├── grammars/
│   └── TAYNI.tmLanguage.json   # TextMate grammar
└── samples/
    └── TAYNI/
        ├── compiler.tayni        # Self-compiler (real code)
        ├── parser.tayni          # Parser (real code)
        └── stdlib.tayni          # Standard library (real code)
```

## Grammar Repository

For the `script/add-grammar` command, we need a separate repository
containing just the grammar. Create `TAYNI/TAYNI-grammar` with:
- `TAYNI.tmLanguage.json`
- `LICENSE` (MIT)
- `README.md`

## Color

TAYNI's color in GitHub: `#4A90D9` (blue, representing AI/technology)

## Notes

- Samples are real production code, not tutorials
- Grammar covers all TAYNI operators and syntax
- Extension `.tayni` is unique (no conflicts in languages.yml)
