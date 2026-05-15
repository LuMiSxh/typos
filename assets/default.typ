// ─── Page geometry ──────────────────────────────────────────────────────────
#set page(
  paper: "a4",
  margin: (left: 2.5cm, right: 2.5cm, top: typos-top-margin, bottom: 2.5cm),
  header: context {
    grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      gutter: 0.5em,
      [
        #if typos-logo-path != "" {
          image(typos-logo-path, height: typos-logo-height)
          if typos-header-text != "" {
            h(0.4cm)
            text(fill: typos-header-text-color, size: 9.5pt)[#typos-header-text]
          }
        } else if typos-header-text != "" {
          text(fill: typos-header-text-color, size: 9.5pt)[#typos-header-text]
        }
      ],
      text(fill: typos-text-color, size: 9.5pt)[#typos-author],
    )
    v(-0.4em)
    line(length: 100%, stroke: 0.5pt + typos-primary)
  },
  footer: context {
    line(length: 100%, stroke: 0.5pt + typos-primary)
    v(-0.4em)
    grid(
      columns: (1fr, auto),
      align: (left, right),
      text(fill: typos-text-color, size: 9pt)[
        #typos-institute
        #if typos-email != "" [\ #link("mailto:" + typos-email)[#typos-email]]
      ],
      text(fill: typos-text-color, size: 9pt)[
        Seite #counter(page).display() von #context counter(page).final().first()
      ],
    )
  },
)

// ─── Typography base ────────────────────────────────────────────────────────
#set text(font: typos-main-font, fill: typos-text-color, size: 11pt, lang: "de")
#set par(justify: true, leading: 0.65em, spacing: 0.9em)

// ─── Headings: uniform weight, size-only hierarchy, primary-tinted ──────────
#show heading: it => block(breakable: false, above: 1.4em, below: 0.6em)[#it]
#show heading: set text(fill: typos-primary, weight: "bold")
#show heading.where(level: 1): set text(size: 17pt)
#show heading.where(level: 2): set text(size: 14pt)
#show heading.where(level: 3): set text(size: 12pt)
#show heading.where(level: 4): set text(size: 11pt)
#show heading.where(level: 5): set text(size: 11pt, weight: "regular", style: "italic")

// ─── Inline emphasis ────────────────────────────────────────────────────────
#show strong: set text(weight: "bold")
#show emph: set text(style: "italic")
#show link: set text(fill: typos-primary)

// ─── Code ───────────────────────────────────────────────────────────────────
#show raw.where(block: true): it => block(
  fill: rgb("#f6f6f6"),
  stroke: 0.5pt + rgb("#dcdcdc"),
  radius: 3pt,
  inset: (x: 10pt, y: 8pt),
  width: 100%,
  breakable: true,
)[#text(font: typos-mono-font, size: 9.5pt)[#it]]

#show raw.where(block: false): it => box(
  fill: rgb("#f0f0f0"),
  inset: (x: 3pt, y: 1pt),
  radius: 2pt,
  baseline: 1pt,
)[#text(font: typos-mono-font, size: 0.92em)[#it]]

// ─── Block quotes ───────────────────────────────────────────────────────────
#show quote: set block(
  fill: rgb("#fafafa"),
  stroke: (left: 3pt + typos-primary),
  inset: (left: 12pt, top: 6pt, bottom: 6pt, right: 6pt),
  width: 100%,
)

// ─── Lists ──────────────────────────────────────────────────────────────────
#set list(indent: 1em, spacing: 0.8em)
#set enum(indent: 1em, spacing: 0.8em)

// ─── Tables ─────────────────────────────────────────────────────────────────
#set table(
  stroke: 0.5pt + rgb("#cccccc"),
  inset: 6pt,
  fill: (_, y) => if calc.odd(y) { rgb("#fafafa") } else { white },
)
#show table.cell.where(y: 0): set text(weight: "bold", fill: typos-text-color)
