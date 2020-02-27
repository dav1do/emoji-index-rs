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
