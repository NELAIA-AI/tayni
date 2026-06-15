# NELAIA GitHub Linguist Submission

This folder contains everything needed to submit NELAIA to GitHub Linguist
for official language recognition.

## Requirements Status

| Requirement | Status | Notes |
|-------------|--------|-------|
| TextMate Grammar | ✅ Ready | `grammars/nelaia.tmLanguage.json` |
| MIT License | ✅ Ready | Repository is MIT licensed |
| Real-world Samples | ✅ Ready | `samples/NELAIA/` (3 files) |
| languages.yml Entry | ✅ Ready | `languages.yml.snippet` |
| 200 Unique Repos | ❌ Pending | Need community adoption |

## When to Submit

Submit the PR to `github-linguist/linguist` when:
1. At least 200 unique `user/repo` combinations have `.nela` files
2. Check with: `https://github.com/search?q=extension%3Anela&type=code`

## Submission Steps

1. Fork `github-linguist/linguist`
2. Add grammar:
   ```bash
   script/add-grammar https://github.com/NELAIA-AI/nelaia-grammar
   ```
3. Copy samples to `samples/NELAIA/`
4. Add entry to `lib/linguist/languages.yml` (use `languages.yml.snippet`)
5. Run `script/update-ids` to generate language_id
6. Open PR with template filled out

## Files

```
linguist-prep/
├── README.md                    # This file
├── languages.yml.snippet        # Entry for languages.yml
├── grammars/
│   └── nelaia.tmLanguage.json   # TextMate grammar
└── samples/
    └── NELAIA/
        ├── compiler.nela        # Self-compiler (real code)
        ├── parser.nela          # Parser (real code)
        └── stdlib.nela          # Standard library (real code)
```

## Grammar Repository

For the `script/add-grammar` command, we need a separate repository
containing just the grammar. Create `NELAIA-AI/nelaia-grammar` with:
- `nelaia.tmLanguage.json`
- `LICENSE` (MIT)
- `README.md`

## Color

NELAIA's color in GitHub: `#4A90D9` (blue, representing AI/technology)

## Notes

- Samples are real production code, not tutorials
- Grammar covers all NELAIA operators and syntax
- Extension `.nela` is unique (no conflicts in languages.yml)
