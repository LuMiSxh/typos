# Configuration & Custom Templates

Full reference for `typos.toml`, document front-matter, variable references, and the Typst template API.

---

## At a glance

```toml
[[profiles]]
name = "luca"

[profiles.identity]
display_name = "Luca Schmidt"
author       = "Luca Michael Schmidt"
institute    = "HS Fulda"
email        = "luca@example.com"

[profiles.colors]
primary = "#ED1B24"
text    = "#000000"
heading = "$colors.text"     # variable reference — track text color
link    = "$colors.primary"

[profiles.sizes]
top_margin = "3cm"
body       = "11pt"

[profiles.layout]
logo = "assets/logo.png"
```

Every field is **optional**. Anything you don't set uses the built-in default. Use `$section.field` anywhere in a string value to reference another resolved field within the same profile.

---

## Schema

typos walks up the directory tree from your current working directory to find `typos.toml`, the same way Cargo finds `Cargo.toml`. Place it at your project root.

### Top-level

```toml
[defaults]      # applies to every profile unless overridden
[[profiles]]    # one entry per profile
name    = "..."
extends = "..." # optional inheritance
```

The `[defaults]` table and each `[[profiles]]` entry share the same section structure described below.

### `[profiles.identity]`

| Key | Type | Default | Description |
|---|---|---|---|
| `display_name` | string | `name` | Shown in the interactive picker |
| `author` | string | `""` | Header (right) and footer |
| `institute` | string | `""` | Footer (left) |
| `email` | string | `""` | Footer below `institute`, rendered as `mailto:` |

### `[profiles.colors]`

All values are hex strings (`"#RRGGBB"`).

| Key | Default | Effect |
|---|---|---|
| `primary` | `"#000000"` | Rules, links by default |
| `text` | `"#000000"` | Body text, footer text |
| `heading` | `$colors.text` | Heading color (any level) |
| `link` | `$colors.primary` | Hyperlink color |
| `rule` | `$colors.primary` | Header/footer rule line color |
| `header_label` | `$colors.text` | Color of `layout.header_text` in the page header |
| `code_fill` | `"#f6f6f6"` | Code block background |
| `code_border` | `"#dcdcdc"` | Code block border |
| `code_inline_fill` | `"#f0f0f0"` | Inline `` `code` `` background |
| `quote_fill` | `"#fafafa"` | Blockquote background |
| `quote_border` | `$colors.primary` | Blockquote left bar |
| `table_stroke` | `"#cccccc"` | Table cell borders |
| `table_alt_fill` | `"#fafafa"` | Alternating row fill |

### `[profiles.sizes]`

All values are Typst length literals as strings — supported units: `pt`, `em`, `cm`, `mm`, `in`.

| Key | Default | Effect |
|---|---|---|
| `body` | `"11pt"` | Base body text size |
| `code` | `"9.5pt"` | Code block text size |
| `top_margin` | `"3cm"` | Top page margin |
| `side_margin` | `"2.5cm"` | Left + right page margin |
| `bottom_margin` | `"2.5cm"` | Bottom page margin |
| `head_height` | `"1.3cm"` | Reserved header height |
| `logo_height` | `"1cm"` | Logo height in header |
| `par_leading` | `"0.65em"` | Line height inside paragraphs |
| `par_spacing` | `"0.9em"` | Space between paragraphs |
| `list_indent` | `"1em"` | List indent |
| `list_spacing` | `"0.8em"` | Space between list items |
| `heading_above` | `"1.4em"` | Space above headings |
| `heading_below` | `"0.6em"` | Space below headings |
| `h1` | `"17pt"` | H1 size |
| `h2` | `"14pt"` | H2 size |
| `h3` | `"12pt"` | H3 size |
| `h4` | `"11pt"` | H4 size |

### `[profiles.fonts]`

| Key | Type | Default | Description |
|---|---|---|---|
| `main` | [FontSpec](#font-specification) | `"Libertinus Serif"` | Body font |
| `mono` | [FontSpec](#font-specification) | `"DejaVu Sans Mono"` | Monospace font |

### `[profiles.layout]`

| Key | Type | Default | Description |
|---|---|---|---|
| `logo` | path | _(none)_ | Path to a logo image (PNG/JPG/PDF), relative to `typos.toml` |
| `header_text` | string | `""` | Optional text in the page header next to the logo |
| `template` | path | built-in | Path to a `.typ` file replacing the built-in template |
| `output_dir` | path | _(next to source)_ | Where to write PDFs |

### `[profiles.vars]`

Free-form table of custom variables. Each entry is exposed to the Typst template as `typos-<key>`:

```toml
[profiles.vars]
course = "Sensor Fusion"
year   = 2026
tags   = ["draft", "internal"]
```

In the template: `#typos-course`, `#typos-year`, `#typos-tags`. Names must start with an ASCII letter and contain only letters, digits, `-`, or `_`.

---

## Variable references

Any string value across all sections may use `$section.field` to reference another resolved field within the same profile:

```toml
[profiles.colors]
primary = "#ED1B24"
heading = "$colors.text"      # heading tracks text color
link    = "$colors.primary"   # link tracks primary
rule    = "$colors.primary"
```

References are resolved **after** the extends chain and defaults are applied. Chains of references resolve up to 16 levels deep; cycles short-circuit and leave the literal `$...` in place.

You can also reference sizes and identity fields:

```toml
[profiles.sizes]
h2 = "$sizes.h1"   # H2 inherits H1's size
```

---

## Inheritance via `extends`

A profile can declare `extends = "<other_profile_name>"` to inherit every field from another profile. Inheritance walks leaf → parent → grandparent at field granularity — only fields that are unset in the child fall through to the parent. Cycles are detected and broken safely.

```toml
[[profiles]]
name = "company-base"
[profiles.colors]
primary = "#ED1B24"
text    = "#000000"
[profiles.layout]
logo = "assets/logo.png"

[[profiles]]
name    = "luca"
extends = "company-base"
[profiles.identity]
author = "Luca Schmidt"
email  = "luca@example.com"

[[profiles]]
name    = "team-lead"
extends = "company-base"
[profiles.identity]
author = "Team Lead"
[profiles.colors]
heading = "$colors.primary"   # branded headings just for this profile
```

---

## Font specification

A font is either a bare string (family name — system or bundled) or an inline table with `path`:

```toml
[profiles.fonts]
main = "Libertinus Serif"               # bundled, always works
mono = "DejaVu Sans Mono"               # bundled, always works
# or:
main = { path = "fonts/MyFont.ttf" }    # user file, path relative to typos.toml
```

**Bundled fonts** (no install required):
- `Libertinus Serif` — body (Regular, Bold, Italic, BoldItalic, Semibold, SemiboldItalic)
- `DejaVu Sans Mono` — monospace (Regular, Bold, Oblique, BoldOblique)
- `New Computer Modern Math` — automatic for `$...$` math

Supported file formats: `.ttf`, `.otf`, `.ttc`, `.otc`. For `.woff2`, convert to TTF first:

```bash
pip install fonttools brotli
python -m fontTools.ttLib.woff2 decompress font.woff2
```

---

## Document front-matter

Any `.md` or `.typ` file can start with a TOML front-matter block. The shape is **identical** to a `[[profiles]]` entry minus the `name` and `extends` keys:

```markdown
+++
[identity]
author = "Co-Author"

[colors]
heading = "$colors.primary"
+++

# My Report

…
```

`---` is accepted as an alternative fence. Dotted keys work too:

```markdown
+++
colors.heading = "$colors.primary"
identity.author = "Co-Author"
+++
```

Front-matter overrides apply on top of the resolved profile, and `$var` references are re-resolved after the merge, so a front-matter `primary` change cascades to everything that references `$colors.primary`.

---

## Custom templates

The built-in template handles A4 page geometry, header/footer, body styling, code blocks, blockquotes, lists, and tables. To replace it entirely, set `layout.template` (per profile or in `[defaults.layout]`) to a `.typ` file.

### How it works

Before your template source is compiled, typos prepends a block of `#let` bindings — one per resolved field. Names follow the pattern `typos-<section>-<field>` (dots become dashes for Typst identifier compatibility):

```typst
#let typos-identity-author    = "Luca Schmidt"
#let typos-identity-email     = "luca@example.com"
#let typos-colors-primary     = rgb("ED1B24")
#let typos-colors-text        = rgb("000000")
#let typos-colors-heading     = rgb("000000")
#let typos-colors-link        = rgb("ED1B24")
#let typos-colors-code-fill   = rgb("f6f6f6")
#let typos-sizes-body         = 11pt
#let typos-sizes-top-margin   = 3cm
#let typos-sizes-h1           = 17pt
#let typos-fonts-main         = "Libertinus Serif"
#let typos-fonts-mono         = "DejaVu Sans Mono"
#let typos-layout-logo-path   = "/abs/path/logo.png"  // empty string = no logo
#let typos-layout-header-text = "Internal"
// + one #let per custom var, named typos-<key>
```

The document body (converted Markdown for `.md`, or raw source for `.typ`) is appended **after** your template.

### Minimal template

```typst
#set page(
  paper: "a4",
  margin: (
    top: typos-sizes-top-margin,
    bottom: typos-sizes-bottom-margin,
    left: typos-sizes-side-margin,
    right: typos-sizes-side-margin,
  ),
)

#set text(font: typos-fonts-main, fill: typos-colors-text, size: typos-sizes-body)
#show link: set text(fill: typos-colors-link)
#show raw:  set text(font: typos-fonts-mono)
```

### Tips

- **Colors arrive as `rgb()` values** — use directly in `fill:` / `stroke:`.
- **Lengths arrive as Typst `length` literals** — use directly in margin/spacing.
- **Logo path** is empty (`""`) when not configured — always guard `if typos-layout-logo-path != ""`.
- **`.typ` files use the same template** — your file body is appended after typos' `#set` rules, so additional `#set`/`#show` rules in your document layer cleanly on top.
- **Compile errors include hints** from Typst pointing to the offending line in the assembled source.
