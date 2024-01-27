use reqwest::{Client, ClientBuilder};
use rss::Channel;
use std::fmt;
use std::fs::File;
use std::io::{self, stdout, BufRead, BufReader, Lines, Write};
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
            Error::Http(err) => write!(f, "HTTP client error: {err}"),
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
    println!("Fetching URL: {url}");
    let response = client.get(&url).send().await?.bytes().await?;

    Channel::read_from(&response[..]).map_err(|err| Error::XmlParsing { url, err })
}

fn parse_pub_date(channel: &Channel) -> Option<&str> {
    if let Some(pub_date) = &channel.pub_date {
        Some(pub_date)
    } else if let Some(item) = channel.items.first() {
        item.pub_date.as_deref()
    } else {
        None
    }
}

async fn run() -> Result<(), Error> {
    let mut set = JoinSet::new();
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()?;

    let urls = read_lines("urls.txt")?;

    let mut count = 0;

    for url in urls {
        let url = url?;
        count += 1;
        let client = client.clone();
        set.spawn(async move { fetch_podcast(url, client).await });
    }

    let mut i = 0;
    let stdout = stdout();

    while let Some(result) = set.join_next().await {
        match result? {
            Ok(channel) => {
                i += 1;

                let mut stdout = stdout.lock();
                stdout.write_fmt(format_args!("Title: {}\n", channel.title))?;

                if let Some(pub_date) = parse_pub_date(&channel) {
                    stdout.write_fmt(format_args!("Pub Date: {pub_date}\n"))?;
                } else {
                    stdout.write_all(b"No pub date found\n")?;
                }

                stdout.write_fmt(format_args!("Episodes: {}\n\n", channel.items.len()))?;

                stdout.write_fmt(format_args!("Counter: {i}/{count}\r"))?;
                stdout.flush()?;
            }
            Err(err) => eprintln!("Error fetching feed: {err}"),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        exit(1);
    }
}
