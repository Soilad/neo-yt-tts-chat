use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, stdin, stdout, BufRead, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::{task, time};
use youtube_chat::live_chat::LiveChatClientBuilder;

fn dadabase(key: &str, increment_value: i32) -> io::Result<()> {
    let binding = env::current_dir().expect("cant get current directory");
    let cwd = binding
        .to_str()
        .expect("why tf cant i turn the current directory to a string")
        .to_string();
    let file_path = (cwd.to_owned() + "/dada.txt").to_string();
    let mut usr = File::create(cwd.to_owned() + "/usr.txt").expect("cannot open file");
    // println!("{}",&file_path);
    let mut hashmap = HashMap::new();
    hashmap.entry(key.to_string()).or_insert(increment_value);
    // Open the file
    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);

    // Iterate over each line of the file
    for line in reader.lines() {
        let line = line?; // Get the next line, handling errors
        if let Some((key, value)) = line.split_once(": ") {
            // Insert the key-value pair into the HashMap
            hashmap.insert(
                key.to_string(),
                value
                    .to_string()
                    .parse::<i32>()
                    .expect("ok the score isnt scoring wtf"),
            );
        }
    }
    // println!("{:?}", &hashmap);

    if let Some(value) = hashmap.get_mut(key) {
        *value += increment_value; // Modify the value in-place
    }

    usr.write_all(
        (key.to_owned()
            + " ("
            + &hashmap
                .get(key)
                .expect("cant get the value with the key")
                .to_string()
            + "): ")
            .as_bytes(),
    )
    .expect("write failed");

    let file = File::create(&file_path)?;
    let mut writer = io::BufWriter::new(file);
    for (key, value) in &hashmap {
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
async fn main() {
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
        .arg("anime")
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
            .arg("sink_name=anime")
            .arg("sink_properties=device.description=anime")
            .spawn()
            .expect("cant make virtual audio cable");
    }

    let output = "anime";
    let _ = Command::new("obs").spawn().expect("cant open obs");
    let mut s = String::new();
    println!("Gib link: ");

    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");

    let mut client = LiveChatClientBuilder::new()
        .url(s)
        .expect("cant find live stream")
        .on_chat(move |chat_item| {
            if !chat_item.message.is_empty() {
                match chat_item.message[0] {
                    youtube_chat::item::MessageItem::Text(ref text) => {
                        let _ = Command::new("espeak")
                            .arg("-d")
                            .arg(output)
                            .arg("-v")
                            .arg("en+f5")
                            .arg(text)
                            .spawn()
                            .expect("cant tts");

                        let mut chat =
                            File::create(cwd.to_owned() + "/chat.txt").expect("cannot open file");

                        let _ = dadabase(
                            chat_item.author.name.as_ref().expect("cant get username"),
                            text.len().try_into().expect("cant get message length"),
                        );

                        chat.write_all(text.as_bytes()).expect("write failed");

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
}
