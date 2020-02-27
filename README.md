# emoji-index-rs

Playing with rust and slack APi. Inspired by alfovo/emoji-index.

The idea is to review slack messages and tally counts of emojis used by each user. Beyond the basics of going through all (or maybe 1000) previous messages, it could be smarter and maybe include the following:

    - collect and process reaction emojis used (requires a additional request for each message)
    - store the last message processed for each channel, as well as the results of user/emoji counts to allow picking back up on subsequent runs (e.g. a nightly job or as slash command)
    - use a threadpool to process messages for channels/users

This is not currently working. It will collect 200 messages for channels and just print the emoji count and user in each.
