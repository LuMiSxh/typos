// Page geometry
#set page(
  paper: "a4",
  margin: (left: 2.5cm, right: 2.5cm, top: 3.2cm, bottom: 2.5cm),
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
            text(fill: typos-header-text-color, size: 10pt)[#typos-header-text]
          }
        } else if typos-header-text != "" {
          text(fill: typos-header-text-color, size: 10pt)[#typos-header-text]
        }
      ],
      text(fill: typos-text-color)[#typos-author],
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

// Base typography
#set text(font: typos-main-font, fill: typos-text-color, size: 11pt, lang: "de")
#set par(justify: false, leading: 0.65em)

// Link styling
#show link: set text(fill: typos-primary)

// Heading keep-with-next (widow/orphan prevention)
#show heading: it => block(breakable: false, above: 1.4em, below: 0.7em)[#it]

// Code block styling
#show raw.where(block: true): it => block(
  fill: rgb("#f5f5f5"),
  stroke: 0.5pt + rgb("#dcdcdc"),
  radius: 3pt,
  inset: (x: 8pt, y: 6pt),
  width: 100%,
  breakable: true,
)[#text(font: typos-mono-font, size: 9pt)[#it]]

// Inline code styling
#show raw.where(block: false): it => box(
  fill: rgb("#f0f0f0"),
  inset: (x: 3pt, y: 1pt),
  radius: 2pt,
  baseline: 1pt,
)[#text(font: typos-mono-font)[#it]]

// Table styling — alternating row fill, bold header
#set table(
  stroke: 0.5pt + rgb("#000000"),
  inset: 6pt,
  fill: (_, y) => if calc.odd(y) { rgb("#f5f5f5") } else { white },
)
#show table.cell.where(y: 0): set text(weight: "bold")
