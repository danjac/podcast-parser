use reqwest::Client;
use rss::Channel;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Duration;
use tokio::task::JoinSet;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for ParseError {}

async fn fetch_podcast(url: &str) -> Result<Channel, Box<dyn Error + Send + Sync>> {
    println!("Fetching URL {}", url);
    let response = Client::new()
        .get(url)
        .timeout(Duration::new(60, 0))
        .send()
        .await?
        .bytes()
        .await?;
    match Channel::read_from(&response[..]) {
        Ok(channel) => Ok(channel),
        Err(err) => Err(Box::new(ParseError(format!("Error parsing XML for URL {}: {}", url, err)))),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut set = JoinSet::new();

    let urls = read_lines("urls.txt")?;

    let mut count = 0;

    for url in urls.flatten() {
        count += 1;
        set.spawn(async move { fetch_podcast(&url).await });
    }

    let mut i = 0;

    while let Some(result) = set.join_next().await {
        match result? {
            Ok(channel) => {
                i += 1;
                println!("Counter: {}/{}", i, count);
                println!("Title: {:?}", channel.title);
                println!("Episodes: {}", channel.items.len());
            }
            Err(err) => println!("Error fetching feed: {}", err),
        }
    }

    Ok(())
}
