// ─── Page geometry ──────────────────────────────────────────────────────────
#set page(
  paper: "a4",
  margin: (
    left: typos-sizes-side-margin,
    right: typos-sizes-side-margin,
    top: typos-sizes-top-margin,
    bottom: typos-sizes-bottom-margin,
  ),
  header: context {
    grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      gutter: 0.5em,
      [
        #if typos-layout-logo-path != "" {
          image(typos-layout-logo-path, height: typos-sizes-logo-height)
          if typos-layout-header-text != "" {
            h(0.4cm)
            text(fill: typos-colors-header-label, size: 9.5pt)[#typos-layout-header-text]
          }
        } else if typos-layout-header-text != "" {
          text(fill: typos-colors-header-label, size: 9.5pt)[#typos-layout-header-text]
        }
      ],
      text(fill: typos-colors-text, size: 9.5pt)[#typos-identity-author],
    )
    v(-0.4em)
    line(length: 100%, stroke: 0.5pt + typos-colors-rule)
  },
  footer: context {
    line(length: 100%, stroke: 0.5pt + typos-colors-rule)
    v(-0.4em)
    grid(
      columns: (1fr, auto),
      align: (left, right),
      text(fill: typos-colors-text, size: 9pt)[
        #typos-identity-institute
        #if typos-identity-email != "" [
          \ #link("mailto:" + typos-identity-email)[#typos-identity-email]
        ]
      ],
      text(fill: typos-colors-text, size: 9pt)[
        Seite #counter(page).display() von #context counter(page).final().first()
      ],
    )
  },
)

// ─── Typography base ────────────────────────────────────────────────────────
#set text(
  font: typos-fonts-main,
  fill: typos-colors-text,
  size: typos-sizes-body,
  lang: "de",
)
#set par(justify: true, leading: typos-sizes-par-leading, spacing: typos-sizes-par-spacing)

// ─── Headings: uniform semibold weight, size-only hierarchy ─────────────────
// Semibold (not bold) keeps headings readable without shouting.
#show heading: it => block(
  breakable: false,
  above: typos-sizes-heading-above,
  below: typos-sizes-heading-below,
)[#it]
#show heading: set text(fill: typos-colors-heading, weight: "semibold")
#show heading.where(level: 1): set text(size: typos-sizes-h1)
#show heading.where(level: 2): set text(size: typos-sizes-h2)
#show heading.where(level: 3): set text(size: typos-sizes-h3)
#show heading.where(level: 4): set text(size: typos-sizes-h4)
#show heading.where(level: 5): set text(size: typos-sizes-h4, weight: "regular", style: "italic")

// ─── Inline emphasis ────────────────────────────────────────────────────────
// Semibold for `**strong**` mirrors the heading choice — refined, not slab-heavy.
#show strong: set text(weight: "semibold")
#show emph: set text(style: "italic")
#show link: set text(fill: typos-colors-link)

// ─── Code ───────────────────────────────────────────────────────────────────
#show raw.where(block: true): it => block(
  fill: typos-colors-code-fill,
  stroke: 0.5pt + typos-colors-code-border,
  radius: 3pt,
  inset: (x: 10pt, y: 8pt),
  width: 100%,
  breakable: true,
)[#text(font: typos-fonts-mono, size: typos-sizes-code)[#it]]

#show raw.where(block: false): it => box(
  fill: typos-colors-code-inline-fill,
  inset: (x: 3pt, y: 1pt),
  radius: 2pt,
  baseline: 1pt,
)[#text(font: typos-fonts-mono, size: 0.92em)[#it]]

// ─── Block quotes ───────────────────────────────────────────────────────────
#show quote: set block(
  fill: typos-colors-quote-fill,
  stroke: (left: 3pt + typos-colors-quote-border),
  inset: (left: 12pt, top: 6pt, bottom: 6pt, right: 6pt),
  width: 100%,
)

// ─── Lists ──────────────────────────────────────────────────────────────────
#set list(indent: typos-sizes-list-indent, spacing: typos-sizes-list-spacing)
#set enum(indent: typos-sizes-list-indent, spacing: typos-sizes-list-spacing)

// ─── Tables ─────────────────────────────────────────────────────────────────
#set table(
  stroke: 0.5pt + typos-colors-table-stroke,
  inset: 6pt,
  fill: (_, y) => if calc.odd(y) { typos-colors-table-alt-fill } else { white },
)
#show table.cell.where(y: 0): set text(weight: "semibold", fill: typos-colors-text)
