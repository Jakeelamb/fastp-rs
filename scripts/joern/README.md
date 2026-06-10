# Joern (upstream fastp)

Paths are relative to the **repository root** (`fastp-rs`).

## One-time CPG build

```bash
chmod +x scripts/joern/*.sh   # if needed
./scripts/joern/parse.sh
```

Output: `upstream/fastp/doc/joern/fastp-cpg/`

Re-parse from scratch:

```bash
./scripts/joern/parse.sh --force
```

## Long-running parse (background)

```bash
./scripts/joern/parse-background.sh
# or with force:
./scripts/joern/parse-background.sh --force
```

Logs and PIDs: `upstream/fastp/doc/joern/logs/parse-*.log`, `parse-*.pid`.

## Survey queries (background)

After the CPG exists:

```bash
./scripts/joern/survey-background.sh
```

Writes `upstream/fastp/doc/joern/logs/survey-*.log`. Inspect with `tail -f` on that file.

The Scala script is `survey.sc` (method/type decl counts and name samples). Edit it and re-run the background script as needed.

## Foreground (debug)

```bash
joern --script scripts/joern/survey.sc --param "cpgPath=$(pwd)/upstream/fastp/doc/joern/fastp-cpg" --nocolors
```
