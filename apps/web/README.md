# Social Graph Visualizer

## Stack

- Svelte Kit (static/SPA) (runes)
- [SvelteFlow](https://svelteflow.dev/)
  - html-to-image for [downloading](https://svelteflow.dev/examples/misc/download-image)
- [AtCute](https://codeberg.org/mary-ext/atcute) for AT Protocol integration
- Dexie.js for caching/storage

## To-Do

- [ ] <https://www.dicebear.com/> integration toggle show avatars
      (from profile data) or

    ```text
    https://api.dicebear.com/9.x/rings/svg?seed={handle}
    ```

  - Rings should be default (because they're cool)
- [ ] An about page
- [ ] Use icons
- [ ] Select a user and shift origin to that user
- [ ] For rendered nodes, we should determine *their* relationships:

    ```text
    Let's say we have an origin @me.

    @me follows @A & @B.
    @B follows @C.
    @C follows @A.

    These should be reflected in the graph.
    ```

- [ ] Paging (we start at 5, and warn the user about potentially lags
      as we go further along the map)
- [ ] [Typeahead](https://typeahead.waow.tech/)/autocomplete search
