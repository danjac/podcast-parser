use std::error::Error;
use rss::Channel;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use tokio::task::JoinSet;
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut set = JoinSet::new();

    let urls = read_lines("urls.txt")?;

    for url in urls.flatten() {
        set.spawn(async move { 
            println!("Fetching URL {}", url);
            reqwest::get(url).await?.bytes().await
        });
    }

    while let Some(result) = set.join_next().await {
        if let Ok(response) = result? {
            if let Ok(channel) = Channel::read_from(&response[..]) {
                println!("Title: {:?}", channel.title);
                println!("Episodes: {}", channel.items.len());
            }
        }
     }

    Ok(())
}
