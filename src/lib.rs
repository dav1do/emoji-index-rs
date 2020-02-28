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
    let channel_name = &channel.name.clone().unwrap();
    if !channel
        .members
        .as_ref()
        .unwrap()
        .iter()
        .any(|m| m == user.id.as_ref().unwrap())
    {
        // println!("joining {}", &channel_name);
        let request = slack::channels::JoinRequest {
            name: &channel_name,
            validate: Some(true),
        };
        let response = slack::channels::join(slack_client, slack_api_token, &request);
        let result = response.expect("unable to join channel"); //TODO map and print error
    } else {
        // println!("already in {}", &channel_name);
        //this doesn't error if we try to join again so maybe we just always join?
    }
}

pub fn process_channel_history<R>(client: &R, api_token: &str, channel: &slack::Channel)
where
    R: slack::requests::SlackWebRequestSender,
{
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
        // println!("{:?}", response.messages.clone().unwrap());
        process_messages(&response.messages.unwrap());
    } else {
        println!("{:?}", response);
    }
}

fn process_messages(messages: &[slack::Message]) {
    //need a mutable hash of user: [<emoji : count>] that we can add to efficiently
    for message in messages {
        //TODO if let here, but how exactly?
        match message {
            slack::Message::Standard(m) => {
                let parsed_messages = get_emoji_counts_from_message(m);
                println!("{:?}", parsed_messages);
                //TODO add a 
            }
            _ => (),
        }
    }
}
// TODO turn this into something that takes a reference to the current HashMap of all users and just adds for ours?
// also, move this stuff to lib.rs once it's working
fn get_emoji_counts_from_message(
    message: &slack::MessageStandard,
) -> HashMap<String, HashMap<String, u32>> {
    let m = message.text.clone().unwrap();
    let mut u = HashMap::new();
    let x = extract_emojis(&m);
    u.insert(message.user.clone().unwrap(), x);
    u
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
