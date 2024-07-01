# \#arcade-monitor

**\#arcade-monitor** is a monitor for the [Hack Club Arcade Shop.](https://hackclub.com/arcade/shop/) It continuously checks for new items, stock changes, and item updates, keeping you in the loop on the stuff you want.

## Tech Stack
- **Cloudflare Workers** for running the monitor on the edge.
- **Rust** for the monitor's code. I love its type safety, as well as libraries such as `serde`.

_Fun fact: the monitor was originally written in TypeScript, but it turned out to be so buggy I just rewrote it in Rust. The new version took less time to write, has tests (unlike the TypeScript version), and has better formatted messages._

---
