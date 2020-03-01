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

    //would be nice to have conversations API instead of mpim, im, channels, groups
    //TODO: should change process_channel_history to take any history type and parse the messages

    // let response = slack::groups::list(&client, &token, &slack::groups::ListRequest::default());
    // if let Ok(response) = response {
    //     for group in response.groups.unwrap().into_iter() {
    //         let request = slack::groups::HistoryRequest {
    //             channel: &group.id.unwrap()[..],
    //             inclusive: Some(true),
    //             latest: None,
    //             oldest: None,
    //             count: Some(200),
    //             unreads: Some(true),
    //         };
    //         let response = slack::groups::history(&client, &token, &request);
    //         println!("{:?}", response); //this returns a malformed response currently :/
    //     }
    // }

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

    let mut combined_results = Vec::new();
    for channel in &channels {
        if !emoji_index::is_member_of_channel(&our_bot_user, &channel) {
            let response = emoji_index::join_channel(&client, &token, &channel);
            if let Err(response) = response {
                println!("Unable to join channel. Skipping. Response: {:?}", response);
                continue;
            }
        }
        let users_and_emojis = emoji_index::process_channel_history(&client, &token, channel);
        if !users_and_emojis.is_empty() {
            combined_results.push(users_and_emojis);
        }
    }

    emoji_index::display_final_results(combined_results, &client, &token);

    Ok(())
}
