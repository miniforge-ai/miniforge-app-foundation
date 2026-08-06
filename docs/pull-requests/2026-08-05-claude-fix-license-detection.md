<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# fix: restore the Apache-2.0 appendix so GitHub detects the licence

## The symptom

With the repository public, `gh repo view --json licenseInfo` reports **`not detected`**. GitHub shows no licence badge,
and the licence is invisible to the dependency and compliance scanners that read that field.

This repository is the seam other people build against — it exists to be depended on from outside the organisation. An
undetectable licence on it is worse than on an application, because a consumer evaluating whether they may pin this
crate is exactly the reader that field is for.

## The cause

`LICENSE` was copied from Minibench, which copied it from the public `miniforge` repository, whose copy had the
appendix's placeholder line replaced with a filled-in project notice. Each copy retitled the block rather than removing
it.

The appendix is part of the licence *template*: it tells a reader how to apply the licence to their own work, and
`Copyright [yyyy] [name of copyright owner]` is a placeholder meant to stay a placeholder. Substituting real values
leaves four lines of foreign content in a body that GitHub's `licensee` normalises and hashes to identify the licence,
so the match fails.

Removed:

```
   Title: Miniforge App Foundation
   Subtitle: the public seam — app envelope, data-plane router, workbench contract
   Author: Christopher Lester
   Copyright 2025-2026 Christopher Lester (christopher@miniforge.ai)
```

Restored:

```
   Copyright [yyyy] [name of copyright owner]
```

Nothing is lost. The real attribution is unchanged in the Apache-2.0 header on every source file, the
`license = "Apache-2.0"` field in each crate manifest, and the README.

## Verification

The appendix now matches a canonical Apache-2.0 text verbatim, modulo the leading indentation the rest of this file
uses (diffed with whitespace normalised against `fnv-1.0.7/LICENSE-APACHE`). The terms body was already unmodified.

Detection is a GitHub-side computation on the default branch, so it can only be confirmed after merge.

## Same defect elsewhere

Minibench carries the identical block and is fixed alongside this. The public `miniforge` repository is the origin of the
pattern and is also reported as `not detected`; it is outside this repository's scope and is flagged for a separate fix.
