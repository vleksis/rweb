# rweb

`rweb` is a Rust implementation of a small web browser, built while following [Web Browser Engineering](https://browser.engineering/) by Pavel Panchekha and Chris Harrelson.

The goal of this project is to learn how browsers work by building the core pieces step by step.

## Status

This repository is in an early stage. The project structure and implementation will grow as chapters from Web Browser Engineering are completed.

Completed:

- [x] [Chapter 1. Downloading Web Pages](https://browser.engineering/http.html)
  - [x] 1-1 HTTP/1.1
  - [x] 1-2 File URLs (does not default to a file when no URL is provided)
  - [x] 1-3 data
  - [ ] 1-4 Entities
  - [ ] 1-5 view-source
  - [x] 1-6 Keep-alive
  - [x] 1-7 Redirects
  - [ ] 1-8 Caching
  - [ ] 1-9 Compression
- [x] [Chapter 2. Drawing to the Screen](https://browser.engineering/graphics.html)
  - [x] 2-1 Line breaks
  - [x] 2-2 Mouse wheel
  - [x] 2-3 Resizing
  - [x] 2-4 Scrollbar
  - [ ] 2-5 Emoji
  - [ ] 2-6 about:blank
  - [ ] 2-7 Alternate text direction
- [x] [Chapter 3. Formatting Text](https://browser.engineering/text.html)
  - [ ] 3-1 Centered text
  - [ ] 3-2 Superscripts
  - [ ] 3-3 Soft hyphens
  - [ ] 3-4 Small caps
  - [ ] 3-5 Preformatted text
- [ ] [Chapter 4. Constructing an HTML Tree](https://browser.engineering/html.html)
  - [ ] 4-1 Comments
  - [ ] 4-2 Paragraphs
  - [ ] 4-3 Scripts
  - [ ] 4-4 Quoted attributes
  - [ ] 4-5 Syntax highlighting
  - [ ] 4-6 Mis-nested formatting tags
- [ ] [Chapter 5. Laying Out Pages](https://browser.engineering/layout.html)
  - [ ] 5-1 Links bar
  - [ ] 5-2 Hidden head
  - [ ] 5-3 Bullets
  - [ ] 5-4 Table of contents
  - [ ] 5-5 Anonymous block boxes
  - [ ] 5-6 Run-ins
- [ ] [Chapter 6. Applying Author Styles](https://browser.engineering/styles.html)
  - [ ] 6-1 Fonts
  - [ ] 6-2 Width/height
  - [ ] 6-3 Class selectors
  - [ ] 6-4 display
  - [ ] 6-5 Shorthand properties
  - [ ] 6-6 Inline style sheets
  - [ ] 6-7 Fast descendant selectors
  - [ ] 6-8 Selector sequences
  - [ ] 6-9 !important
  - [ ] 6-10 :has selectors
- [ ] [Chapter 7. Handling Buttons and Links](https://browser.engineering/chrome.html)
  - [ ] 7-1 Backspace
  - [ ] 7-2 Middle-click
  - [ ] 7-3 Window title
  - [ ] 7-4 Forward
  - [ ] 7-5 Fragments
  - [ ] 7-6 Search
  - [ ] 7-7 Visited links
  - [ ] 7-8 Bookmarks
  - [ ] 7-9 Cursor
  - [ ] 7-10 Multiple windows
  - [ ] 7-11 Clicks via the display list

## Reference

- [Web Browser Engineering](https://browser.engineering/) by Pavel Panchekha and Chris Harrelson

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
