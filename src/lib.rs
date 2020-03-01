use regex::Regex;
use slack_api as slack;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

pub fn join_channel_if_necessary<R>(
    slack_client: &R,
    slack_api_token: &str,
    channel: &slack::Channel,
    user: &slack::User,
)
//-> Result<slack::channels::JoinResponse, slack::channels::JoinError<R::Error>>
// TODO how do we encapsulate both the idea "should we join" and "did we join"?
//if that's overloaded, should it be 2 functions or just in line?
where
    R: slack::requests::SlackWebRequestSender,
{
    if !channel
        .members
        .as_ref()
        .unwrap()
        .iter()
        .any(|m| m == user.id.as_ref().unwrap())
    {
        // println!("joining {}", &channel_name);
        let request = slack::channels::JoinRequest {
            name: &channel.name.clone().unwrap(),
            validate: Some(true),
        };
        let response = slack::channels::join(slack_client, slack_api_token, &request);
        let result = response.expect("unable to join channel"); //TODO map and print error
    } else {
        // println!("already in {}", &channel_name);
        //this doesn't error if we try to join again so maybe we just always join?
    }
}

pub fn process_channel_history<R>(
    client: &R,
    api_token: &str,
    channel: &slack::Channel,
) -> std::vec::Vec<MessageEmojiData>
where
    R: slack::requests::SlackWebRequestSender,
{
    let mut users_and_emojis: Vec<MessageEmojiData> = Vec::new();

    //TODO: while there are more messages
    loop {
        let request = slack::channels::HistoryRequest {
            channel: &channel.id.clone().unwrap(),
            inclusive: Some(true),
            latest: None,
            oldest: None,
            count: Some(200),
            unreads: Some(true),
        };
        let response = slack::channels::history(client, &api_token, &request);

        if let Ok(response) = response {
            for message in &response.messages.unwrap() {
                process_message(message, &mut users_and_emojis);
            }
            if let Some(false) = response.has_more {
                break;
            }
        } else {
            println!("unable to get messages {:?}", response);
            //return an error or break
            break;
        }
    }

    users_and_emojis
}

fn process_message(message: &slack::Message, users_and_emojis: &mut Vec<MessageEmojiData>) {
    //we only care about standard messages where we find emojis. otherwise we just keep going
    if let slack::Message::Standard(m) = message {
        if let Some(message_emoji_data) = get_emoji_counts_from_message(m) {
            // check if we already have data for our user and update the counts or add a new entry
            if let Some(existing_user_data) = users_and_emojis
                .iter_mut()
                .find(|md| md.user == message_emoji_data.user)
            {
                for emoji_cnt in message_emoji_data.emojis_and_counts {
                    existing_user_data.update_emoji_cnt(emoji_cnt.0, emoji_cnt.1);
                }
            } else {
                users_and_emojis.push(message_emoji_data);
            }
        }
    }
}

fn get_emoji_counts_from_message(message: &slack::MessageStandard) -> Option<MessageEmojiData> {
    let user = message.user.clone().unwrap();
    let message = message.text.clone().unwrap();

    let emojis_and_counts = extract_emojis(&message);
    if emojis_and_counts.is_empty() {
        return None;
    }
    Some(MessageEmojiData {
        user,
        emojis_and_counts,
    })
}

fn extract_emojis(text: &str) -> HashMap<String, u32> {
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

#[derive(Clone, Debug)]
pub struct MessageEmojiData {
    user: String,
    //emoji name: count of occurences for user
    emojis_and_counts: HashMap<String, u32>,
}

impl MessageEmojiData {
    fn update_emoji_cnt(&mut self, emoji: String, count: u32) {
        if let Some(cnt) = self.emojis_and_counts.get_mut(&emoji) {
            *cnt += count;
        } else {
            self.emojis_and_counts.insert(emoji, count);
        }
    }
}

// use std::sync::mpsc;
// use std::thread;
// use std::time::Duration;
// use std::sync::{Mutex, Arc};

// fn main2() {
//     let counter = Arc::new(Mutex::new(0));
//     let mut handles = vec![];

//     for _ in 0..10 {
//         let counter = Arc::clone(&counter);
//         let handle = thread::spawn(move || {
//             let mut num = counter.lock().unwrap();

//             *num += 1;
//         });
//         handles.push(handle);
//     }

//     for handle in handles {
//         handle.join().unwrap();
//     }

//     println!("Result: {}", *counter.lock().unwrap());

//     let (tx, rx) = mpsc::channel();

//     let tx1 = mpsc::Sender::clone(&tx);
//     thread::spawn(move || {
//         let vals = vec![
//             String::from("hi"),
//             String::from("from"),
//             String::from("the"),
//             String::from("thread"),
//         ];

//         for val in vals {
//             tx1.send(val).unwrap();
//             thread::sleep(Duration::from_secs(1));
//         }
//     });

//     thread::spawn(move || {
//         let vals = vec![
//             String::from("more"),
//             String::from("messages"),
//             String::from("for"),
//             String::from("you"),
//         ];

//         for val in vals {
//             tx.send(val).unwrap();
//             thread::sleep(Duration::from_secs(1));
//         }
//     });

//     for received in rx {
//         println!("Got: {}", received);
//     }
//     println!("huh?");
// }

// fn get_channel_ids(response: Option<slack_api::channels::ListResponse>) -> Vec<String> {

//     let mut channels_result = Vec::new();
//     // let mut channels_result: Vec<String> = Vec::new();
//     if let Ok(response) = response {
//         if let Some(channels) = response.channels.iter() {
//             // channels_result.append(&mut channels.map(|c| c.as_ref().id).collect());
//             channels_result.append(&channels.iter().map(|c| c.id.as_ref().take().unwrap().to_string()).collect());
//             return channels_result;
//         }
//     }
//     channels_result
// }

// use std::sync::mpsc;
// use std::thread;
// fn async() {

// let (tx, rx) = mpsc::channel();
// // let tx1 = mpsc::Sender::clone(&tx);
// thread::spawn(move || {
//     let response =
//         slack::channels::list(&client, &token, &slack::channels::ListRequest::default());
//     if let Ok(response) = response {
//         if let Some(channels) = response.channels {
//             for channel in &channels {
//                 // prefer clone() to as_ref(). profile to remove clones
//                 tx.send(channel.id.clone().unwrap().to_string()).unwrap();
//             }
//         }
//     }
// });
//     for received in rx {
//     println!("Got: {}", received);
// }
// }
