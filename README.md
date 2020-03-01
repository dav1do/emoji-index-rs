# emoji-index-rs

Playing with rust and slack APi. Inspired by alfovo/emoji-index.

The idea is to review slack messages and tally counts of emojis used by each user. Beyond the basics of going through all previous messages like it currently tries, it could be smarter and maybe include the following:

- collect and process reaction emojis used
- review emojis used in threads
- store the last message processed for each channel, as well as the results of user/emoji counts to allow picking back up on subsequent runs (e.g. a nightly job or as slash command)
- use a threadpool to process messages for channels/users concurrently

To run, you need to add a Slack app with name "emoji_analyzer" (this is hardcoded currently to add itself to channels).  It needs permission to join channels and read messages, you can grant DMs as well to potentially review this. For example:

- View messages and other content in public channels that emoji analyzer has been added to
- View basic information about public channels in the workspace
- View custom emoji in the workspace
- View messages and other content in direct messages that emoji analyzer has been added to
- View basic information about direct messages that emoji analyzer has been added to
- View people in the workspace
- View profile details about people in the workspace
- Send messages as @emoji_analyzer
- View emoji reactions and their associated content in channels and conversations that emoji analyzer has been added to
- View messages and other content in private channels that emoji analyzer has been added to
- View messages and other content in group direct messages that emoji analyzer has been added to
- Join public channels in the workspace

Then you need to add the app token as an environment variable named `SLACK_API_TOKEN`. At this point, you can `cargo run` and it should process the data and print some results (though it may panic because I haven't tested it for any cases with lots of results.
