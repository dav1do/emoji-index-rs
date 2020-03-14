use regex::Regex;
use slack_api as slack;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref EMOJI_REGEX: Regex = Regex::new(r":([a-z1-9-_]+?):").unwrap();
}

pub fn display_final_results<R>(multiple_results: Vec<Vec<UserEmojiData>>, client: &R, token: &str)
where
    R: slack::requests::SlackWebRequestSender,
{
    let result = get_final_emoji_counts(multiple_results);
    for data in &result {
        let request = slack::users::InfoRequest {
            user: &data.user[..],
        };
        let response = slack::users::info(client, &token, &request).expect("unable to get user info");
        let user_name = response
            .user
            .unwrap()
            .profile
            .and_then(|p| p.real_name_normalized.or(p.display_name_normalized))
            .unwrap_or(data.user.clone());

        display_results_for_user(&user_name, &data.emojis_and_counts, Some(5));
    }
}

fn display_results_for_user(
    user_name: &str,
    emojis_and_cnts: &HashMap<String, u32>,
    result_limit: Option<usize>,
) {
    let mut count_vec: Vec<(&String, &u32)> = emojis_and_cnts.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    count_vec.truncate(result_limit.unwrap_or(10));
    println!("\n{}", user_name);
    for emoji_cnt in count_vec {
        println!("\t{} used {} times", emoji_cnt.0, emoji_cnt.1);
    }
}

pub fn is_member_of_channel(user: &slack::User, channel: &slack::Channel) -> bool {
    if channel
        .members
        .as_ref()
        .unwrap()
        .iter()
        .any(|m| m == user.id.as_ref().unwrap())
    {
        return true;
    }
    false
}

pub fn join_channel<R>(
    slack_client: &R,
    slack_api_token: &str,
    channel: &slack::Channel,
) -> Result<slack::channels::JoinResponse, slack::channels::JoinError<R::Error>>
where
    R: slack::requests::SlackWebRequestSender,
{
    // println!("joining {}", &channel_name);
    let request = slack::channels::JoinRequest {
        name: &channel.name.clone().unwrap(),
        validate: Some(true),
    };
    slack::channels::join(slack_client, slack_api_token, &request)
}

pub fn process_channel_history<R>(
    client: &R,
    api_token: &str,
    channel: &slack::Channel,
) -> Vec<UserEmojiData>
where
    R: slack::requests::SlackWebRequestSender,
{
    let mut users_and_emojis: Vec<UserEmojiData> = Vec::new();

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
                if let Some(message_emoji_data) = get_emoji_counts_from_message(message) {
                    add_message_data_to_result(&mut users_and_emojis, message_emoji_data);
                }
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

fn add_message_data_to_result(
    users_and_emojis_result: &mut Vec<UserEmojiData>,
    message_emoji_data: UserEmojiData,
) {
    if let Some(existing_user_data) = users_and_emojis_result
        .iter_mut()
        .find(|md| md.user == message_emoji_data.user)
    {
        for emoji_cnt in message_emoji_data.emojis_and_counts {
            existing_user_data.update_emoji_cnt(emoji_cnt.0, emoji_cnt.1);
        }
    } else {
        users_and_emojis_result.push(message_emoji_data);
    }
}

fn get_final_emoji_counts(multiple_results: Vec<Vec<UserEmojiData>>) -> Vec<UserEmojiData> {
    let mut final_results = Vec::new();

    for (idx, res) in multiple_results.into_iter().enumerate() {
        if idx == 0 {
            final_results = res; //just add everything from the first entry. could be smart and find the biggest instead
        } else {
            for message in res {
                add_message_data_to_result(&mut final_results, message);
            }
        }
    }

    final_results
}

fn get_emoji_counts_from_message(message: &slack::Message) -> Option<UserEmojiData> {
    match message {
        slack::Message::Standard(m) => {
            let user = m.user.clone().unwrap();
            let message = m.text.clone().unwrap();
            if let Some(thread) = &m.thread_ts {
                //look in the thread for info (conversations.replies)
                //need to consider how to separate each user reply
            }

            let emojis_and_counts = extract_emojis(&message);
            if emojis_and_counts.is_empty() {
                return None;
            }
            Some(UserEmojiData {
                user,
                emojis_and_counts,
            })
        }
        //should check thread for all message types?
        _ => None,
    }
}

fn extract_emojis(text: &str) -> HashMap<String, u32> {
    let mut res = HashMap::new();

    let emojis: Vec<&str> = EMOJI_REGEX
        .find_iter(text)
        .map(|mat| mat.as_str())
        .collect();

    for emoji in emojis {
        //remove the : from start and end.
        //going to assume the byte works since it's not grapheme clusters or anything
        let e = emoji[1..(emoji.len() - 1)].to_string();
        if e.contains("skin-tone") {
            continue; //don't add the skin tone stuff e.g. :wave::skin-tone-3:
        }
        if let Some(count) = res.get_mut(&e) {
            *count += 1;
        } else {
            res.insert(e, 1);
        }
    }

    res
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserEmojiData {
    user: String,
    //emoji name: count of occurences for user
    emojis_and_counts: HashMap<String, u32>,
}

impl UserEmojiData {
    fn update_emoji_cnt(&mut self, emoji: String, count: u32) {
        if let Some(cnt) = self.emojis_and_counts.get_mut(&emoji) {
            *cnt += count;
        } else {
            self.emojis_and_counts.insert(emoji, count);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use slack_api as slack;
    use std::collections::HashMap;

    fn get_standard_message(
        user_id: Option<String>,
        message_text: Option<String>,
    ) -> slack::Message {
        slack::Message::Standard(slack::MessageStandard {
            attachments: None,
            bot_id: None,
            channel: None,
            edited: None,
            event_ts: None,
            reply_broadcast: None,
            source_team: None,
            thread_ts: None,
            team: Some("T22D7LDBK".to_string()),
            text: message_text,
            ts: Some("1582758607.000300".to_string()),
            ty: Some("message".to_string()),
            user: user_id,
        })
    }

    #[test]
    fn stardard_message_without_emojis_is_none() {
        let user_id = Some("U22CTJP7W".to_string());
        let text = Some("this is :a message :.".to_string());
        let message = get_standard_message(user_id, text);
        let res = get_emoji_counts_from_message(&message);
        assert_eq!(None, res);
    }

    #[test]
    fn stardard_message_with_emojis() {
        let user_id = Some("U22CTJP7W".to_string());
        let text = Some(":rage: :rage: :smile: words: a sentence :grumpycat:".to_string());
        let message = get_standard_message(user_id.clone(), text);
        let res = get_emoji_counts_from_message(&message);
        let mut vals = HashMap::new();
        vals.insert("rage".to_string(), 2);
        vals.insert("smile".to_string(), 1);
        vals.insert("grumpycat".to_string(), 1);
        let expected = UserEmojiData {
            user: user_id.unwrap(),
            emojis_and_counts: vals,
        };
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn skin_tone_emojis_ignored() {
        let user_id = Some("U22CTJP7W".to_string());
        let text = Some(":rage: :rage: :wave::skin-tone-3: :wave:".to_string());
        let message = get_standard_message(user_id.clone(), text);
        let res = get_emoji_counts_from_message(&message);
        let mut vals = HashMap::new();
        vals.insert("rage".to_string(), 2);
        vals.insert("wave".to_string(), 2);
        let expected = UserEmojiData {
            user: user_id.unwrap(),
            emojis_and_counts: vals,
        };
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn stardard_message_with_almost_emojis() {
        let user_id = Some("U22CTJP7W".to_string());
        let text = Some("junk :! this !::".to_string());
        let message = get_standard_message(user_id, text);
        let res = get_emoji_counts_from_message(&message);
        assert_eq!(None, res);
    }
    #[test]
    fn parse_channel_join_message_is_none() {
        let message = slack::Message::ChannelJoin(slack::MessageChannelJoin {
            subtype: Some("channel_join".to_string()),
            text: Some("<@UTD5RCL2E> has joined the channel".to_string()),
            ts: Some("1582080741.000200".to_string()),
            ty: Some("message".to_string()),
            user: Some("UTD5RCL2E".to_string()),
        });
        let res = get_emoji_counts_from_message(&message);
        assert_eq!(None, res);
    }
}
