use std::error::Error;
use rss::Channel;


use tokio::task::JoinSet;

async fn fetch_podcast(url: &str) -> Channel {
    println!("running url: {}", url);
    let response = reqwest::get(url).await.expect("could not fetch").bytes().await.expect("ok");
    println!("done: {}", url);
    Channel::read_from(&response[..]).expect("cannot parse XML")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut set = JoinSet::new();

    let urls = vec![
        "https://pod.link/1171270672.rss",
        "http://feeds.feedburner.com/thememorypalace",
        "https://feeds.acast.com/public/shows/cfd8aed5-6d63-4b2f-b325-46e4b5665583",
    ];

    for url in urls {
        set.spawn(async move { 
            fetch_podcast(url).await
        });
    }

    while let Some(result) = set.join_next().await {
        let channel = result?;
        println!("Title: {:?}", channel.title);
        //println!("Description: {:?}", channel.description);
        //println!("Image: {:?}", channel.image);
        println!("Episodes: {}", channel.items.len());

    //     println!("Title: {:?}", channel.title);
    //     println!("Description: {:?}", channel.description);
    //     println!("Image: {:?}", channel.image);
    //     println!("Episodes: {}", channel.items.len());

    //     if let Some(item) = channel.items.first() {
    //         println!("LATEST EPISODE");
    //         println!("Title: {:?}", item.title);
    //         println!("GUID: {:?}", item.guid);
    //         println!("Description: {:?}", item.description);
    //         println!("Pub date: {:?}", item.pub_date);
    //         println!("Itunes: {:?}", item.itunes_ext);
    //         println!("Enclosure: {:?}", item.enclosure);
    //     }
     }

    Ok(())
}
