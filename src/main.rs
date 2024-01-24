use std::error::Error;
use rss::Channel;


use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut set = JoinSet::new();

    let urls = vec![
        "http://feeds.feedburner.com/thememorypalace",
        "https://feeds.acast.com/public/shows/cfd8aed5-6d63-4b2f-b325-46e4b5665583",
        "https://feeds.acast.com/public/shows/d41a80b2-1fe3-45dc-9966-79caeb36e911",
        "https://feeds.simplecast.com/BqbsxVfO",
        "https://pod.link/1171270672.rss",
        "https://rss.art19.com/hello-from-magic-tavern",
        "https://feeds.megaphone.fm/YMRT7068253588",
        "https://feed.podbean.com/folkinscotland/feed.xml",
        "https://feeds.acast.com/public/shows/e9da3d2c-d264-5c31-83b5-259aad10fee5",
        "https://thegrognardfiles.com/category/podcast/feed/",
        "https://softwareengineeringdaily.com/feed/podcast/",
        "https://anchor.fm/s/b2eb61a0/podcast/rss",
        "https://feeds.megaphone.fm/GLT5697813216",
        "https://audioboom.com/channels/4322549.rss",
    ];

    for url in urls {
        set.spawn(async move { 
            println!("Fetching URL {}", url);
            reqwest::get(url).await?.bytes().await
        });
    }

    while let Some(result) = set.join_next().await {
        let response = result??;
        let channel = Channel::read_from(&response[..])?;
        println!("Title: {:?}", channel.title);
        println!("Episodes: {}", channel.items.len());
     }

    Ok(())
}
