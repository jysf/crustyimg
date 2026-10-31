# Checking images in GitHub Actions

This guide sets up crustyimg to check every image in a pull request: it flags a photo that would
publish where it was taken, a file over your size budget, a photo stored sideways, or a file that
isn't a readable image. Each problem shows up as a note on the file in the pull request, with the
command that fixes it.

Every step below is running in [`jysf/crustyimg-example`](https://github.com/jysf/crustyimg-example),
a small public site you can copy from. Its
[pull request #1](https://github.com/jysf/crustyimg-example/pull/1) stays open so you can see what
the findings look like.

There are two Actions:

- [`jysf/crustyimg-action`](https://github.com/marketplace/actions/crustyimg) installs crustyimg,
  lints your images, and turns each finding into a pull-request annotation. Start here.
- [`jysf/setup-crustyimg`](https://github.com/marketplace/actions/setup-crustyimg) only installs
  crustyimg (checksum-verified) and adds it to `PATH`, for when you want to run commands yourself.

## 1. Add the lint workflow

Create `.github/workflows/images.yml`:

```yaml
name: images

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: jysf/crustyimg-action@v1
        with:
          paths: assets/**/*
```

Set `paths` to wherever your images live. Separate several with spaces:
`paths: assets/**/* content/**/*`.

**Use a glob, not a bare folder name.** A folder in `paths` covers only the images directly inside
it, so `paths: assets` skips `assets/photos/`. `assets/**/*` includes every subfolder. The action
passes the glob to crustyimg as written, so it doesn't need quotes.

With no config file, this already catches GPS locations, sideways photos and unreadable files.
Files that aren't images, like a `README.md` inside `assets/`, are skipped.

## 2. Set a size budget

The size and width checks only run once you tell crustyimg what "too big" means. Add a
`.crustyimg-lint.toml` to the root of your repo:

```toml
[[budget]]
glob = "assets/**/*"
max_bytes = 500000          # flag any image over 500 KB
max_intended_width = 1600   # flag any image wider than it is ever shown
```

You can have several `[[budget]]` blocks, for example a tighter one for `assets/icons/**`.

The same file can change how strict each rule is, or switch a rule off for some files:

```toml
# error, warn, info or off
[severity]
"orient/orientation-not-baked" = "error"

[[per_file_ignores]]
glob = "assets/press-kit/**"
rules = ["privacy/gps-metadata-leak"]
```

To see what your current images would report, run the same check on your machine:

```sh
crustyimg lint 'assets/**/*'
```

Quote the glob on the command line, so crustyimg expands it rather than your shell. Install
crustyimg with `brew install jysf/tap/crustyimg`, or see the
[README](../README.md#install) for other options.

## 3. Choose what fails the check

By default, an **error** fails the job and a **warning** only adds a note. Change that with
`fail-level`:

| `fail-level` | The job fails on |
|---|---|
| `error` (default) | any error |
| `warn` | any error or warning |
| `never` | nothing; findings are reported but the job always passes |

`never` is a good way to start on a repo that already has a lot of images: you see the findings
without blocking anyone, then switch to `error` once they're cleaned up.

## 4. Make the check required

A failing check only stops a merge if the branch requires it. In your repo's **Settings → Rules**
(or **Branches**), add a rule for `main` that requires the status check named **`lint`**, which is
the job name from step 1.

## 5. Fix what it finds

Each annotation names the rule and suggests a fix. These are the common ones. Each command
rewrites the file in place:

| Rule | What it means | Fix |
|---|---|---|
| `privacy/gps-metadata-leak` | The file holds the location where it was taken. | `crustyimg meta clean --gps <file> -o <file> -y` |
| `size/oversized-bytes` | The file is over its budget. | `crustyimg optimize <file> --profile preserve --max 1600 -o <file> -y` |
| `dims/oversized-dimensions` | The image is wider than it's ever shown. | `crustyimg resize --max 1600 <file> -o <file> -y` |
| `orient/orientation-not-baked` | The pixels are stored sideways, and only a metadata flag turns them upright. | `crustyimg auto-orient <file> -o <file> -y` |
| `size/truncated-or-corrupt` | The file isn't a readable image. | Export it again. |

To remove all metadata, not only the location (camera serial numbers, for example), use
`crustyimg meta strip <file> -o <file> -y`.

## Optional extras

### Findings in the Security tab

crustyimg can write [SARIF](https://sarifweb.azurewebsites.net/), which GitHub shows in the
repository's **Security** tab:

```yaml
jobs:
  sarif:
    # Pull requests from forks can't write to the Security tab, so skip them.
    if: github.event_name == 'push' || github.event.pull_request.head.repo.full_name == github.repository
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    steps:
      - uses: actions/checkout@v4
      - uses: jysf/setup-crustyimg@v1
      - run: crustyimg lint 'assets/**/*' --format sarif > crustyimg.sarif
        continue-on-error: true   # still upload when there are findings
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: crustyimg.sarif
          category: crustyimg
```

### Shrink images when you deploy

`setup-crustyimg` puts the CLI on `PATH`, so any crustyimg command can run in a build. The
simplest is `crustyimg web`, which runs the bundled `web` recipe on each image: it turns the
photo upright, removes its metadata, shrinks it, and writes the smallest modern format (AVIF for
photos, lossless WebP or PNG for graphics):

```yaml
      - uses: jysf/setup-crustyimg@v1
      - run: crustyimg web 'assets/photos/*' --max 1600 --out-dir _site/assets/photos -y
```

`web` picks the format per image, so `photo.jpg` usually comes out as `photo.avif`, and your
pages need to link to the new name. To keep a JPEG for browsers that can't show AVIF, write both
and serve them in a `<picture>`. `--out-dir` puts every output file directly in that folder, so
run it once per folder if you want to keep subfolders.

To keep the original format and file names instead, use
`crustyimg optimize 'assets/photos/*' --profile preserve --max 1600 --out-dir _site/assets/photos -y`.

#### Your own recipes, with `crustyimg build`

For more than one kind of output, write the steps as TOML recipes and list them in a
`crustyimg.build.toml`:

```toml
# crustyimg.build.toml
version = 1

[[target]]                       # the smallest modern format
source = "assets/photos/*.jpg"
recipe = "recipes/portfolio.toml"
out    = "_site/assets/photos"

[[target]]                       # the same recipe, as a JPEG fallback
source = "assets/photos/*.jpg"
recipe = "recipes/portfolio.toml"
out    = "_site/assets/photos"
name   = "{stem}.jpg"
```

Then the workflow step is just `crustyimg build`. A recipe can crop to an exact size, add a text
or logo watermark, and pin its output format. The example repo's
[`recipes/`](https://github.com/jysf/crustyimg-example/tree/main/recipes) folder has three: signed
portfolio photos, plain guest photos, and a 1200×630 JPEG card for link previews. Its
[`pages.yml`](https://github.com/jysf/crustyimg-example/blob/main/.github/workflows/pages.yml)
writes a before-and-after size table to the run summary. The recipe format is in
[`recipes/README.md`](../recipes/README.md).

A target whose `source` matches no files stops the whole build. If a folder can be empty, put its
targets in a second manifest and run that one only when the folder has images.

### Catch problems before they're committed

crustyimg ships a [pre-commit](https://pre-commit.com/) hook:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/jysf/crustyimg
    rev: v0.8.2
    hooks:
      - id: crustyimg-lint
```

### Use the JSON report

After the lint step, the action leaves the full report at `$RUNNER_TEMP/crustyimg-lint.json` in the
`crustyimg.lint/v1` format, for a later step to read. The example repo's
[`canary.yml`](https://github.com/jysf/crustyimg-example/blob/main/.github/workflows/canary.yml)
uses it to check that a known-bad image still fails.

## Pull requests from forks

The lint workflow works on pull requests from forks with no extra setup. It needs only read access,
and its annotations appear on the contributor's files as usual. GitHub waits for a maintainer to
approve the run the first time someone contributes, which you can change under **Settings → Actions
→ General**. The Security-tab upload is the one step that can't run on a fork's pull request, which
is why the example above skips it.

Don't switch the lint workflow to `pull_request_target` to get around that: it runs with write
access while checking out the contributor's files.

## Inputs reference

| Input | Default | What it does |
|---|---|---|
| `paths` | `.` | Files, folders or globs to check, separated by spaces. |
| `fail-level` | `error` | `error`, `warn` or `never`; see step 3. |
| `args` | | Extra flags for `crustyimg lint`, such as `--select privacy` or `--config path/to/config.toml`. |
| `version` | `latest` | The crustyimg release to install, such as `v0.8.0`. |
| `mode` | `lint` | `optimize` re-encodes the images instead of linting them. |
| `install` | `true` | Set to `false` if crustyimg is already on `PATH`. |

`setup-crustyimg` takes only `version`. Both Actions run on Linux, macOS and Windows runners.

**Versions:** `@v1` always points at the newest `v1.x` release, so you get fixes without editing
your workflow. Pin an exact release, such as `@v1.0.3`, if you'd rather upgrade on your own
schedule.
