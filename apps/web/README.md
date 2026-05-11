# Social Graph Visualizer

## Stack

- Svelte Kit (static/SPA) (runes)
- [SvelteFlow](https://svelteflow.dev/)
  - html-to-image for [downloading](https://svelteflow.dev/examples/misc/download-image)
- [AtCute](https://codeberg.org/mary-ext/atcute) for AT Protocol integration
- Dexie.js for caching/storage

## To-Do

- [ ] An about page
- [ ] Use icons
- [ ] Select a user and shift origin to that user
- [ ] For rendered nodes, we should determine _their_ relationships:

  ```text
  Let's say we have an origin @me.

  @me follows @A & @B.
  @B follows @C.
  @C follows @A.

  These should be reflected in the graph.
  ```

- [ ] Paging (we start at 5, and warn the user about potentially lags
      as we go further along the map)
      - [ ] Load More/Expand to fetch more people
- [ ] [Typeahead](https://typeahead.waow.tech/)/autocomplete search
- [ ] OG-Image

  ```text
  Svelte component
      ↓ render()
  HTML string
      ↓ satori-html
  Satori node
      ↓ satori
  SVG string
      ↓ resvg
  PNG response
      ↓ SvelteKit prerender
  static /og.png
  ```
