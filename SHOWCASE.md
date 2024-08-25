# Arcade Monitor

Arcade Monitor is a **monitor for the Hack Club Arcade Shop.** It continuously checks for new items, stock changes, and item updates, keeping you in the loop on the stuff you want. It chatters away in the [#arcade-monitor channel on Slack](https://hackclub.slack.com/archives/C079RG9HJ81) to over 350 members across Slack and ntfy.

![image](https://github.com/user-attachments/assets/3fe6199a-7d82-4620-ad40-9ab2c10044da)

## The first version

The first version of Arcade Monitor was a Cloudflare Worker written in TypeScript. In hindsight, a lot of the code was needlessly complicated, and the bot was _severely_ buggy. Take a look at this:

![image](https://github.com/user-attachments/assets/22320356-c8a9-418e-b799-623e1a7b9e9f)

Yeah, that wasn't the greatest first impression... 😅

It also created random "updates" that were completely unclear - what had actually happened was that the _description_ had updated, but it showed a price update.

![image](https://github.com/user-attachments/assets/0c0633a7-ed35-4894-bf66-39cd3f9ef76f)

What was worse was that Hack Club had created a new channel called [#arcade-bulletin](https://google.com) that seemingly made the bot obsolete.

![image](https://github.com/user-attachments/assets/b09b07bf-a824-4a51-a739-16931a88eea1)

I did quickly realise, however, that the bot could track things like stock updates that were infeasible for Hack Club staff to do every time someone bought a YubiKey.

> (Also, fun fact: the YubiKeys are technically limited, but there's over 1,000 of them. The more you know 🤷‍♂️)
