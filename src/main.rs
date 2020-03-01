use emoji_index;
use slack_api as slack;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const BOT_NAME: &str = "emoji_analyzer";

    let token = env::var("SLACK_API_TOKEN").map_err(|_| "SLACK_API_TOKEN env var must be set")?;
    let client = slack::default_client().map_err(|_| "Could not get default_client")?;

    let response = slack::channels::list(&client, &token, &slack::channels::ListRequest::default());
    let channels = response.expect("unable to get channels").channels.unwrap();

    let response = slack::users::list(&client, &token, &slack::users::ListRequest::default());
    let users = response.expect("unable to get users").members.unwrap();

    //TODO possible to do this in a one liner? .find() is only one element
    // let ignored_users = users.iter().take_while(|u| u.is_bot == Some(true)).collect();
    // think ^this stops when any returns false. and the collect isn't quite right

    let mut ignored_users = Vec::new();
    for user in &users {
        if user.is_bot == Some(true) {
            ignored_users.push(user.clone());
        }
    }
    //could do this in previous loop, but passing the user out seems clunky (probably missing something)
    let our_bot_user = ignored_users
        .iter()
        .find(|u| u.real_name == Some(BOT_NAME.to_string()))
        .expect("unable to find our bot user. exiting");

    for channel in &channels {
        emoji_index::join_channel_if_necessary(&client, &token, channel, our_bot_user);
        let users_and_emojis = emoji_index::process_channel_history(&client, &token, channel);
        if !users_and_emojis.is_empty() {
            //TODO change this to combine for all channels and not a new copy for each
            println!("end: {:?}", users_and_emojis);
        }
    }

    Ok(())
}