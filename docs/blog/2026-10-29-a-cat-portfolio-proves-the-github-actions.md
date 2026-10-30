# A cat's portfolio proves the GitHub Actions work

*2026-10-29 · launch · the GitHub Actions*

## Green checks in the wrong repo

crustyimg has two GitHub Actions, and this week they went up on the Marketplace:
[`crustyimg-action`](https://github.com/marketplace/actions/crustyimg) lints the images in a pull
request and annotates each problem on the file, and
[`setup-crustyimg`](https://github.com/marketplace/actions/setup-crustyimg) just installs the CLI.

Both had self-tests from the start. Both self-tests were green. Getting them ready to list, by
re-testing against crustyimg 0.8.0, turned up three bugs anyway:

- **The default version didn't install.** `version: latest` looked up the newest release by piping
  GitHub's release JSON through `grep`. Once the 0.8.0 release notes made that JSON big enough,
  `grep` stopped reading early, `curl` failed on the closed pipe, and every runner failed. The
  self-test pinned an exact version, so it never took that path.
- **Windows annotations pointed nowhere.** On Windows, crustyimg reports paths like
  `\\?\D:\a\repo\repo\assets\photo.jpg`. GitHub only attaches an annotation to a pull-request file
  when the path is relative to the repo, so every finding landed in the run log instead of on the
  file.
- **Recursive globs quietly lost depth.** The action passed `paths` through bash unquoted, so
  `assets/**/*` was expanded by bash, which treats `**` like `*`. Images in subfolders were never
  checked, and nothing reported that they'd been skipped.

All three are fixed as of `v1.0.3`. The lesson I took: a self-test only proves what it exercises. It
couldn't fail the way a real repo with real paths fails. So I built a real repo.

## Meet Mr. Whiskers

[`jysf/crustyimg-example`](https://github.com/jysf/crustyimg-example) is the portfolio of a
professional cat, live at
[jysf.github.io/crustyimg-example](https://jysf.github.io/crustyimg-example/). Services include
napping, keyboard QA and gravity audits. The photos are my own cats, and everything around them is
an ordinary small site that happens to use every way crustyimg plugs into GitHub:

| Workflow | What it does |
|---|---|
| `lint.yml` | Runs `crustyimg-action` on every pull request. Findings become annotations, and errors fail the required check. |
| `code-scanning.yml` | Uploads the same findings to the Security tab as SARIF. |
| `pages.yml` | Resizes and re-encodes every photo with `crustyimg optimize`, then deploys to GitHub Pages. |
| `canary.yml` | Runs weekly on Linux, macOS and Windows against the newest crustyimg. |

## The rejected shots

A portfolio needs a "did not make the cut" section. The site has one, and it links to
[pull request #1](https://github.com/jysf/crustyimg-example/pull/1), which tries to add four new
photos and will never be merged:

| Photo | What crustyimg said | The artist's statement |
|---|---|---|
| `headshot.jpg` | GPS location in a public image | It revealed my secret nap spot. I have moved. |
| `hero-loaf.jpg` | 4 MB, over the 1 MB budget | I contain multitudes. |
| `sunbeam.jpg` | Stored sideways | It is accurate. That is how I lie. |
| `hairball.jpg` | Not a readable image | … |

Open its **Files changed** tab and each finding sits on its file, with the command that fixes it.
The `lint` check is red, and `main` requires it, so the merge button stays blocked. That pull
request is the whole demo, and it stays open so anyone can look.

The GPS coordinates in `headshot.jpg` are fake. They point at the Hemingway House in Key West, home
of the famous six-toed cats.

## The real photos had problems too

The real photos were where the demo started paying for itself. They came straight off a Nikon
point-and-shoot, and crustyimg flagged every one of them for GPS.

They didn't contain a location. The camera writes an empty GPS block into every JPEG, with a
version number and no coordinates. Flagging that is cautious rather than wrong, but the message
claims a location is there, and it isn't. Looking closer, every photo also carried the camera's
serial number, which is the kind of thing you don't think about until it's on a public website. So
every photo on the site went through `crustyimg meta strip`, which removes all metadata, and was
checked with exiftool afterwards: zero tags left.

Writing the contributor guide turned up a second problem. The fix the linter suggests for a GPS
finding, `crustyimg meta clean --gps photo.jpg`, writes the cleaned image to standard output rather
than back to the file. Run it in a terminal and you get a screen full of binary. The form that
works is `crustyimg meta clean --gps photo.jpg -o photo.jpg -y`. Both issues are being fixed in
crustyimg itself. Neither would have shown up in a test suite that only checks the rule fires.

## A canary that has to fail

A weekly job that passes every week doesn't tell you much: it would look the same if the linter
had stopped checking anything. So the canary first runs the action on an image with a GPS location
in it and checks that the job **fails**, with exactly that finding on exactly that file. Only then
does it check that the real site passes.

If a crustyimg release ever stops catching GPS, or an Action tag breaks on one operating system,
that job goes red within a week.

## Send in your cat

The site has a **Guest stars** section, and it's empty. Fork the repo, add a photo of your cat to
`assets/photos/guests/`, add a line to `photos.json` and open a pull request. It goes through the
same checks as everything else. If your phone saved where the photo was taken, you'll get a red
check and a note on your file saying how to remove it. That's the point of the demo.

[CONTRIBUTING.md](https://github.com/jysf/crustyimg-example/blob/main/CONTRIBUTING.md) has the
details. To set the same checks up on your own repo, start with the
[GitHub Actions guide](../github-actions.md).

---

*crustyimg is a fast, scriptable, single-binary image CLI written in Rust: resize, convert,
optimize, strip metadata and lint, with zero system dependencies by default.*
