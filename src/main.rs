use reqwest::{Client, ClientBuilder};
use rss::Channel;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;
use std::process::exit;
use std::time::Duration;
use tokio::task::{JoinError, JoinSet};

enum Error {
    Io(io::Error),
    XmlParsing { url: String, err: rss::Error },
    Http(reqwest::Error),
    Task(JoinError),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Self::Task(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {err}"),
            Error::XmlParsing { url, err } => write!(f, "Error parsing XML for URL {url}: {err}"),
            Error::Http(err) => write!(f, "HTTP error: {err}"),
            Error::Task(err) => write!(f, "A task failed: {err}"),
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

async fn fetch_podcast(url: String, client: Client) -> Result<Channel, Error> {
    println!("Fetching URL {}", url);
    let response = client.get(&url).send().await?.bytes().await?;

    match Channel::read_from(&response[..]) {
        Ok(channel) => Ok(channel),
        Err(err) => Err(Error::XmlParsing { url, err }),
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

async fn run() -> Result<(), Error> {
    let mut set = JoinSet::new();
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build the HTTP client: {e}");
            exit(1);
        });

    let urls = read_lines("urls.txt")?;

    let mut count = 0;

    for url in urls {
        let url = url?;
        count += 1;
        let client = client.clone();
        set.spawn(async move { fetch_podcast(url, client).await });
    }

    let mut i = 0;

    while let Some(result) = set.join_next().await {
        match result? {
            Ok(channel) => {
                i += 1;
                println!("Counter: {}/{}", i, count);
                println!("Title: {:?}", channel.title);
                if let Some(pub_date) = parse_pub_date(&channel) {
                    println!("Pub Date: {}", pub_date)
                } else {
                    println!("No pub date found")
                }
                println!("Episodes: {}", channel.items.len());
            }
            Err(err) => println!("Error fetching feed: {}", err),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
    }
}
