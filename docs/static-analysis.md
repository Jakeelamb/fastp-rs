# Static analysis (upstream `upstream/fastp`)

## Doxygen

- Config: `upstream/fastp/Doxyfile.fastp-rs` (if present) or generate with `doxygen -g` and set `INPUT = src`, `RECURSIVE = YES`, `GENERATE_XML = YES`.
- Output directory convention: `upstream/fastp/doc/doxygen/`.

## Clang

From `upstream/fastp/src`, per-translation-unit parse:

```bash
for f in *.cpp; do
  clang++ -std=c++11 -fsyntax-only -pthread -I. "$f" && echo OK "$f"
done
```

Observed on this machine: **22** `.cpp` files parse **without** Intel ISA-L and without Highway multi-target glue; **7** fail:

| File | Reason |
|------|--------|
| `evaluator.cpp`, `fastqreader.cpp`, `main.cpp`, `peprocessor.cpp`, `seprocessor.cpp`, `unittest.cpp` | `#include <isa-l/igzip_lib.h>` missing (install `libisal` / point `-I` at ISA-L headers) |
| `simd.cpp` | `HWY_TARGET_INCLUDE` expects multi-target build layout |

With ISA-L dev headers and a proper Highway build, regenerate `compile_commands.json` (e.g. `bear -- make`) and use `clang -ast-dump` / `clang-include-graph` on the full graph.

## Joern

Manual one-shot:

```bash
joern-parse "$REPO/upstream/fastp/src" --output "$REPO/upstream/fastp/doc/joern/fastp-cpg"
```

**Repo scripts** (parse and long survey runs in the **background**, logs under `upstream/fastp/doc/joern/logs/`): see [`scripts/joern/README.md`](../scripts/joern/README.md).

Joern’s default C/C++ frontend may label the graph as **NEWC**; treat type/method lists as approximate for C++ OO code. Prefer **Clang** AST + **Doxygen** XML for authoritative class/method names.
