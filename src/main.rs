use enigo::{Enigo, Keyboard, Settings};
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, stdin, stdout, BufRead, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::{task, time};
use youtube_chat::live_chat::LiveChatClientBuilder;
use roux::{ Reddit, Subreddit };
use reqwest::Client;

async fn share(title: &str, link: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open("config.json").unwrap();
    let config: serde_json::Value = serde_json::from_reader(file).unwrap();

    let client = Reddit::new(
        config["user-agent"].as_str().unwrap(),
        config["client-id"].as_str().unwrap(),
        config["client-secret"].as_str().unwrap(),
    )
    .username(&config["username"].as_str().unwrap())
    .password(&config["password"].as_str().unwrap())
    .login();

    let subreddits = config["subreddits"].as_array().unwrap();
    let me = client.await.unwrap();
    for subreddit in subreddits {
        // let sub = Subreddit::new(subreddit.as_str().unwrap());
        me.submit_link(title, link, subreddit.as_str().unwrap()).await?;
    }

    let client = Client::new();
    let channel_ids = config["channel_ids"].as_array().unwrap();
    dbg!(channel_ids);
    for channel_id in channel_ids {
        dbg!(channel_id);
        let channel_link = format!("https://discord.com/api/v9/channels/{}/messages", channel_id.as_str().unwrap());
        dbg!(&link);
        let _res = client
            .post(&channel_link)
            .header("Authorization", config["authorization"].as_str().unwrap())
            .json(&json!({"content": link}))
            .send()
            .await;
    }
    Ok(())
}

fn database(key: &str, increment_value: i32) -> io::Result<()> {
    let binding = env::current_dir().expect("cant get current directory");
    let cwd = binding
        .to_str()
        .expect("why can't i turn the current directory to a string")
        .to_string();
    let file_path = (cwd.to_owned() + "/data.txt").to_string();
    let mut usr = File::create(cwd.to_owned() + "/usr.txt").expect("cannot open file");
    // println!("{}",&file_path);
    let mut hashmap_vec = vec![];
    // Open the file
    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);

    // Iterate over each line of the file
    for line in reader.lines() {
        let line = line?; // Get the next line, handling errors
        if let Some((key, value)) = line.split_once(": ") {
            hashmap_vec.push((key.to_string(), value.to_string().parse::<i32>().unwrap()));
        }
    }
    // println!("{:?}", &hashmap);

    if let Some(value) = hashmap_vec.iter().position(|r| r.0 == key) {
        hashmap_vec[value].1 += increment_value; // Modify the value in-place
    }

    hashmap_vec.sort_by(|a,b| b.1.cmp(&a.1));

    usr.write_all(
        (key.to_owned()
            + " ("
            +
            &hashmap_vec.iter().find(|x| x.0 == key).expect("cant get score for user").1.to_string()
            + "): ")
            .as_bytes(),
    )
    .expect("write failed");

    let file = File::create(&file_path)?;
    let mut writer = io::BufWriter::new(file);
    for (key, value) in hashmap_vec {
        writeln!(writer, "{}: {}", key, value)?;
    }

    Ok(())
}
// fn print_type_of<T: ?Sized>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }
//pactl list sinks short | grep anime | cut -c1-2
//pactl load-module module-virtual-sink sink_name=anime
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let output: &str = "anime";
    let cwd = env::current_dir()
        .expect("cant get current directory")
        .to_str()
        .expect("cant turn the current directory into a str")
        .to_string();

    let pactl = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .arg("short")
        .stdout(Stdio::piped())
        .spawn()
        .expect("cant run pactl");
    let grep = Command::new("grep")
        .arg(output)
        .stdin(Stdio::from(pactl.stdout.expect("cant pipe into grep")))
        .output()
        .expect("cant find virtual audio cable");
    // let result = std::str::from_utf8(&output.stdout).unwrap();
    // print_type_of(&result);
    // print_type_of(&o);
    // println!("{:?}", &output);
    // println!("{:?}",output.len());
    // println!("{:?}",env::current_dir().unwrap());
    // print_type_of("15");
    // println!("{:?}", result);
    //pactl load-module module-null-sink sink_name=anime sink_properties=device.description=anime
    if grep.stdout.is_empty() {
        println!("virtual audio cable not found, creating virtual audio cable",);
        Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg("sink_name=".to_owned()+output)
            .arg("sink_properties=device.description=".to_owned()+output)
            .spawn()
            .expect("cant make virtual audio cable");
    }

    Command::new("obs").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("cant open obs");
    let mut link = String::new();
    println!("Give link: ");

    let _ = stdout().flush();
    stdin()
        .read_line(&mut link)
        .expect("Did not enter a correct string");

    let _ = share("fish", &link).await;
    // Command::new("python3.11")
    //     .arg("sharing.py")
    //     .arg(link.clone())
    //     .spawn()
    //     .expect("cant share");
    let mut client = LiveChatClientBuilder::new()
        .url(link)
        .expect("cant find live stream")
        .on_chat(move |chat_item| {
            if !chat_item.message.is_empty() {
                match chat_item.message[0] {
                    youtube_chat::item::MessageItem::Text(ref text) => {
                        let _ = Command::new("espeak")
                            .arg("-d")
                            .arg(output)
                            .arg("-v")
                            .arg("en+f2")
                            .arg(text)
                            .spawn()
                            .expect("cant tts");

                        let mut chat =
                            File::create(cwd.to_owned() + "/chat.txt").expect("cannot open file");

                        let mut enigo = Enigo::new(&Settings::default()).unwrap();
                        let _ = database(
                            chat_item.author.name.as_ref().expect("cant get username"),
                            text.len().try_into().expect("cant get message length"),
                        );
                        if text.starts_with('!') {
                            enigo.text(&text.replace('!', "")).expect("uhoh");
                            let _ = Command::new("python3.11")
                            .arg("commands.py")
                            .args(text.split_whitespace())
                            .spawn()
                            .expect("cant tts");
                        } else {
                            chat.write_all(text.as_bytes()).expect("write failed");
                        }

                        println!(
                            "{}:\n {}",
                            chat_item.author.name.expect("cant print username"),
                            text
                        );
                    }
                    _ => {
                        // Handle other types if necessary
                    }
                }
            }
        })
        .on_error(|error| eprintln!("{:?}", error))
        .build();
    client.start().await.unwrap();
    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(300));
        loop {
            interval.tick().await;
            client.execute().await;
        }
    });

    forever.await.unwrap();
    Ok(())
}
