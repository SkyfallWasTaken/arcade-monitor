# \#arcade-monitor

**\#arcade-monitor** is a monitor for the [Hack Club Arcade Shop.](https://hackclub.com/arcade/shop/) It continuously checks for new items, stock changes, and item updates, keeping you in the loop on the stuff you want.

## Environment variables
### Secrets
- `SLACK_WEBHOOK_URL` - Slack webhook to call on updates
- `NTFY_AUTH_TOKEN` - Auth token for ntfy

### Variables
- `ARCADE_SHOP_URL` - Shop URL to fetch
- `NTFY_URL` - URL for ntfy

## Tech Stack
- **Cloudflare Workers** for running the monitor on the edge.
- **Rust** for the monitor's code. I love its type safety, as well as libraries such as `serde`.

_Fun fact: the monitor was originally written in TypeScript, but it turned out to be so buggy I just rewrote it in Rust. The new version took less time to write, has tests (unlike the TypeScript version), and has better formatted messages._

---
