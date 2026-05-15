# Configuration & Custom Templates

Full reference for `typos.toml`, document front-matter, and the Typst template variable API.

---

## typos.toml

typos walks up the directory tree from your current working directory to find `typos.toml`, the same way Cargo finds `Cargo.toml`. Place it at your project root.

### `[defaults]`

Values in `[defaults]` apply to every profile unless the profile overrides them.

| Key | Type | Default | Description |
|---|---|---|---|
| `output_dir` | path | _(next to source)_ | Directory where PDFs are written, relative to `typos.toml`. When omitted, each PDF is placed next to its source file. |
| `main_font` | [FontSpec](#font-specification) | `"Libertinus Serif"` | Body text font. Defaults to the bundled Libertinus Serif. |
| `mono_font` | [FontSpec](#font-specification) | `"DejaVu Sans Mono"` | Monospace font for code blocks and inline code. Defaults to the bundled DejaVu Sans Mono. |
| `template` | path | built-in | Path to a `.typ` file, relative to `typos.toml` |
| `top_margin` | length | `"3cm"` | Top page margin (passed to template as `typos-top-margin`) |
| `head_height` | length | `"1.3cm"` | Reserved header height (passed to template as `typos-head-height`) |
| `vars` | table | _(empty)_ | Custom variables exposed to your Typst template as `typos-<key>`. See [Custom variables](#custom-variables). |

### `[[profiles]]`

Each `[[profiles]]` block defines one named profile. The `name` field is required; everything else is optional and falls back to `extends → [defaults] → built-in default`.

| Key | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✓ | Identifier used with `--profile`. No spaces (use `_` or `-`). |
| `extends` | string | | Name of another profile to inherit from. See [Inheritance](#inheritance). |
| `display_name` | string | | Human-readable name shown in the interactive picker. Falls back to `name`. |
| `primary_color` | hex color | | Accent color for lines, links, and headings. Default: `#000000` |
| `text_color` | hex color | | Body text and footer text color. Default: `#000000` |
| `author` | string | | Shown in the header (right side) and footer. |
| `institute` | string | | Shown in the footer (left side). |
| `email` | string | | Shown in the footer below `institute`. Rendered as a `mailto:` link. |
| `logo` | path | | Path to a logo image (PNG, JPG, PDF), relative to `typos.toml`. |
| `logo_height` | length | | Height of the logo in the header. Default: `"1cm"` |
| `header_text` | string | | Optional text displayed next to the logo in the header. |
| `header_text_color` | hex color | | Color of `header_text`. Default: `#000000` |
| `main_font` | [FontSpec](#font-specification) | | Overrides `[defaults].main_font` for this profile. |
| `mono_font` | [FontSpec](#font-specification) | | Overrides `[defaults].mono_font` for this profile. |
| `template` | path | | Overrides `[defaults].template` for this profile. |
| `output_dir` | path | | Overrides `[defaults].output_dir` for this profile. Omit to place PDF next to the source file. |
| `top_margin` | length | | Overrides `[defaults].top_margin` for this profile. |
| `head_height` | length | | Overrides `[defaults].head_height` for this profile. |
| `vars` | table | | Custom variables (see below). Profile vars merge over default vars. |

### Inheritance

A profile can use `extends = "<other_profile_name>"` to inherit every field from another profile. Inheritance walks the chain leaf → parent → grandparent and stops on a cycle. Each `Option` field is filled by the first profile in the chain that defines it; `vars` tables merge with later entries overriding earlier ones.

```toml
[[profiles]]
name         = "acme"
primary_color = "#E63946"
author        = "Default ACME Author"
institute     = "ACME Corporation"
logo          = "assets/acme-logo.png"

[[profiles]]
name    = "acme-jdoe"
extends = "acme"
author  = "John Doe"        # only this differs from "acme"
email   = "john@acme.com"
```

### Custom variables

The `vars` table lets you expose arbitrary values to a custom Typst template without changing the typos source. Each key becomes a Typst binding named `typos-<key>`:

```toml
[defaults.vars]
course = "Robotics"
year   = 2026

[[profiles]]
name = "luca"
[profiles.vars]
course = "Sensor Fusion"   # overrides the default
```

In your template:

```typst
= #typos-course (#typos-year)
```

Supported value types: strings, integers, floats, booleans, arrays, and inline tables. Variable names must start with an ASCII letter and contain only letters, digits, `-`, or `_`.

### Font specification

A font can be specified as a bare string (system or bundled font name) or as an inline table with a `path` key (font file):

```toml
# Bundled font (no install needed — these always work)
main_font = "Libertinus Serif"
mono_font = "DejaVu Sans Mono"

# Any system font by family name
main_font = "Helvetica Neue"

# File font — TTF or OTF, path relative to typos.toml
main_font = { path = "fonts/MyFont.ttf" }
```

**Bundled fonts** (always available, no install required):
- `Libertinus Serif` — body text (Regular, Bold, Italic, BoldItalic, Semibold, SemiboldItalic)
- `DejaVu Sans Mono` — monospace (Regular, Bold, Oblique, BoldOblique)
- `New Computer Modern Math` — automatic for `$...$` math

Supported file formats: `.ttf`, `.otf`, `.ttc`, `.otc`. For `.woff2` fonts, convert to TTF first:

```bash
pip install fonttools brotli
python -m fontTools.ttLib.woff2 decompress font.woff2
```

### Length values

Any field that accepts a length (margins, heights) takes a Typst length literal as a string:

```toml
top_margin  = "3cm"
logo_height = "1.5cm"
head_height = "1.3cm"
```

Supported units: `cm`, `mm`, `pt`, `em`, `in`.

### Full example

```toml
[defaults]
output_dir = "output"
top_margin = "3cm"

[defaults.vars]
year = 2026

[[profiles]]
name         = "acme"
display_name = "ACME Corp"
primary_color     = "#E63946"
text_color        = "#1D3557"
author            = "Jane Smith"
institute         = "ACME Corporation"
email             = "jane@acme.com"
logo              = "assets/acme-logo.png"
logo_height       = "1cm"
header_text       = "Internal"
header_text_color = "#E63946"

[[profiles]]
name    = "acme-internal"
extends = "acme"
header_text = "Confidential"

[[profiles]]
name      = "personal"
primary_color = "#2196F3"
author        = "Jane Smith"
main_font     = { path = "fonts/MyFont.ttf" }
output_dir    = "dist"
```

---

## Document front-matter

Any `.md` or `.typ` source file can begin with a TOML front-matter block. The values override profile fields just for that document:

```markdown
+++
author = "Co-Author Name"
header_text = "Draft — Do Not Distribute"
primary_color = "#FF6B00"
+++

# My Report

Body content starts here.
```

Both `+++` and `---` are accepted as fences. The block must be the first thing in the file (a UTF-8 BOM is allowed before it).

Recognised keys are: `display_name`, `primary_color`, `text_color`, `author`, `institute`, `email`, `logo`, `logo_height`, `header_text`, `header_text_color`, `top_margin`, `head_height`.

Any unrecognised key is exposed to the template as `typos-<key>`, so per-document custom variables work the same way as profile-level ones.

```markdown
+++
course = "Sensor Fusion"
chapter = 4
+++

# Lecture #typos-chapter — #typos-course
```

---

## Custom Templates

By default typos uses a built-in A4 template with a header (logo + author), footer (institute + email + page count), and styling for code blocks, tables, headings, and math.

You can replace it entirely — globally via `[defaults].template` or per-profile via `template` — with any Typst (`.typ`) file.

### How it works

Before your template source is compiled, typos prepends a block of `#let` variable declarations. These bindings carry all profile values into the Typst namespace. Your template can use any of them:

```typst
#let typos-primary          = rgb("E63946")   // color
#let typos-text-color       = rgb("1D3557")   // color
#let typos-author           = "Jane Smith"    // str
#let typos-institute        = "ACME Corp"     // str
#let typos-email            = "jane@acme.com" // str
#let typos-logo-path        = "/abs/path/logo.png" // str, empty when no logo
#let typos-logo-height      = 1cm             // length
#let typos-header-text      = "Internal"      // str, empty when not set
#let typos-header-text-color = rgb("E63946")  // color
#let typos-main-font        = "Libertinus Serif" // str
#let typos-mono-font        = "DejaVu Sans Mono" // str
#let typos-top-margin       = 3cm             // length
#let typos-head-height      = 1.3cm           // length
// + one #let per custom var, named typos-<key>
```

The document content (converted Markdown for `.md`, or the file body for `.typ`) is appended **after** your template, so a minimal template only needs to configure page geometry and text styles — it doesn't need to `#include` anything.

### Variable reference

| Variable | Type | Notes |
|---|---|---|
| `typos-primary` | `color` | Use for accents: rules, links, headings |
| `typos-text-color` | `color` | Use for body text and subdued UI elements |
| `typos-author` | `str` | May be empty string — check before using |
| `typos-institute` | `str` | May be empty string |
| `typos-email` | `str` | May be empty string |
| `typos-logo-path` | `str` | Absolute path. Empty string `""` means no logo was configured — always guard with `if typos-logo-path != ""` |
| `typos-logo-height` | `length` | Ready to use directly: `image(..., height: typos-logo-height)` |
| `typos-header-text` | `str` | May be empty string |
| `typos-header-text-color` | `color` | Falls back to `#000000` when not set |
| `typos-main-font` | `str` | Pass to `#set text(font: typos-main-font)` |
| `typos-mono-font` | `str` | Pass to `#show raw: set text(font: typos-mono-font)` |
| `typos-top-margin` | `length` | Use in `#set page(margin: (top: typos-top-margin, ...))` |
| `typos-head-height` | `length` | Use in `#set page(header-ascent: ...)` or header sizing |
| `typos-<custom-key>` | varies | One per `vars` entry or front-matter key; type matches the TOML literal |

### Minimal template

```typst
#set page(
  paper: "a4",
  margin: (top: typos-top-margin, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
)

#set text(font: typos-main-font, fill: typos-text-color, size: 11pt)
#show link: set text(fill: typos-primary)
#show raw:  set text(font: typos-mono-font)
```

That's enough to get a clean, branded page. Add header/footer, rule styling, and table theming as needed.

### Header and footer pattern

```typst
#set page(
  header: context {
    stack(dir: ltr, spacing: 0.4cm,
      if typos-logo-path != "" {
        image(typos-logo-path, height: typos-logo-height)
      },
      if typos-header-text != "" {
        text(fill: typos-header-text-color)[#typos-header-text]
      },
    )
    place(right + horizon, text(fill: typos-text-color)[#typos-author])
    v(-0.4em)
    line(length: 100%, stroke: 0.5pt + typos-primary)
  },
  footer: context {
    line(length: 100%, stroke: 0.5pt + typos-primary)
    v(-0.4em)
    text(fill: typos-text-color, size: 9pt)[
      #typos-institute
      #if typos-email != "" [\ #link("mailto:" + typos-email)[#typos-email]]
    ]
    place(right, text(fill: typos-text-color, size: 9pt)[
      Seite #counter(page).display() von #context counter(page).final().first()
    ])
  },
)
```

### Tips

- **Colors are ready-to-use Typst `color` values** — pass them directly to `fill:`, `stroke:`, etc. No conversion needed.
- **Lengths are ready-to-use Typst `length` values** — use them directly in margin/spacing expressions.
- **Strings may be empty** — always guard `typos-logo-path`, `typos-header-text`, `typos-institute`, and `typos-email` before rendering them.
- **`.typ` files use the same template** — the file body is appended after the template, so define `#set` rules normally and they'll layer cleanly on top of typos' defaults.
- **Test with `typos convert sample.md --profile yourprofile`** — Typst compile errors are reported with line numbers pointing into the assembled source (variable block + template + content).
