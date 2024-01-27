use reqwest::{Client, ClientBuilder};
use rss::Channel;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;
use std::process::exit;
use std::time::Duration;
use tokio::task::JoinSet;

fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for ParseError {}

async fn fetch_podcast(url: &str, client: Client) -> Result<Channel, Box<dyn Error + Send + Sync>> {
    println!("Fetching URL {url}");
    let response = client.get(url).send().await?.bytes().await?;
    match Channel::read_from(&response[..]) {
        Ok(channel) => Ok(channel),
        Err(err) => Err(Box::new(ParseError(format!(
            "Error parsing XML for URL {url}: {err}"
        )))),
    }
}

fn parse_pub_date(channel: &Channel) -> Option<String> {
    if let Some(pub_date) = &channel.pub_date {
        Some(pub_date.clone())
    } else if let Some(item) = channel.items.first() {
        item.pub_date.clone()
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut set = JoinSet::new();
    let timeout = Duration::from_secs(30);

    let client = ClientBuilder::new()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build HTTP client: {e}");
            exit(1);
        });

    let urls = read_lines("urls.txt")?;

    let mut count = 0;

    for url in urls {
        count += 1;
        let url = url?;
        let client = client.clone();
        set.spawn(async move { fetch_podcast(&url, client).await });
    }

    let mut i = 0;

    while let Some(result) = set.join_next().await {
        match result? {
            Ok(channel) => {
                i += 1;
                println!("Counter: {i}/{count}");
                println!("Title: {:?}", channel.title);
                if let Some(pub_date) = parse_pub_date(&channel) {
                    println!("Pub Date: {pub_date}")
                } else {
                    println!("No pub date found")
                }
                println!("Episodes: {}", channel.items.len());
            }
            Err(err) => println!("Error fetching feed: {err}"),
        }
    }

    Ok(())
}
