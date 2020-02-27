use regex::Regex;
use slack_api as slack;
use std::collections::HashMap;
use std::env;

#[macro_use]
extern crate lazy_static;

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
    let our_bot_user = &ignored_users
        .iter()
        .find(|u| u.real_name == Some(BOT_NAME.to_string()))
        .expect("unable to find our bot user. exiting");
    //get the member info for current user/app
    //get all channels
    //join channels if we aren't in them
    for channel in &channels {
        //it seems like this should be the array rather than needing to go in another for loop here
        // refactor into join_channels method
        for members in channel.members.iter() {
            //TODO not reference with iter()
            if !members
                .iter()
                .any(|m| *m == our_bot_user.id.clone().unwrap().to_string())
            {
                // TODO update them!
                println!("updating {:?}", &channel.name.clone().unwrap());
                let request = slack::channels::JoinRequest {
                    name: &*channel.name.clone().unwrap(),
                    validate: Some(true),
                };
                let response = slack::channels::join(&client, &token, &request);
                let result = response.expect("unable to join channel"); //TODO map and print error
            }
        }
        //for each channel, get messages (paginating back)
        let request = slack::channels::HistoryRequest {
            channel: &channel.id.clone().unwrap(),
            inclusive: Some(true),
            latest: None,
            oldest: None,
            count: Some(200),
            unreads: Some(true),
        };
        let response = slack::channels::history(&client, &token, &request);
        if let Ok(response) = response {
            // println!("{:?}", response.messages.clone().unwrap());
            process_messages(&response.messages.unwrap());

            if response.has_more == Some(true) {
                // println!("more messages {:?}", &response);
            }
        } else {
            println!("{:?}", response);
        }
        //for each message, find emojis used by member
        //for each response, find emojis used by member
    }
    Ok(())
}
// {user : {emoji : count}}
//

fn process_messages(messages: &[slack::Message]) {
    // let mut hash = HashMap::new();
    // let mut emojimap = HashMap::new();
    // emojimap.insert("grumpycat", 1);
    // hash.insert("user name", emojimap);
    for message in messages {
        match message {
            slack::Message::Standard(m) => println!("{:?}", get_emoji_counts_from_message(m)),
            _ => println!("other message: {:?}", message),
        }
    }
}

// TODO turn this into something that takes a reference to the current HashMap of all users and just adds for ours?
// also, move this stuff to lib.rs once it's working
fn get_emoji_counts_from_message(message: &slack::MessageStandard) -> HashMap<String, HashMap<String, i32>> {
    let m = message.text.clone().unwrap();
    let mut u = HashMap::new();
    let x = extract_emojis(&m);
    u.insert(message.user.clone().unwrap(), x);
    u
}

fn extract_emojis(text: &str) -> HashMap<String, i32> {
    lazy_static! {
        //this finds :: as a valid emoji :/
        static ref EMOJI_REGEX: Regex = Regex::new(r":([a-z1-9-_]*?):").unwrap();
    }
    let mut res = HashMap::new();

    let emojis: Vec<&str> = EMOJI_REGEX
        .find_iter(text)
        .map(|mat| mat.as_str())
        .collect();

    for emoji in emojis {
        let e = emoji.to_string();
        if let Some(count) = res.get_mut(&e) {
            *count += 1;
        } else {
            res.insert(e, 1);
        }
    }

    res
}
