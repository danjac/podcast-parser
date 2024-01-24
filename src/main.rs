use std::error::Error;
use rss::Channel;

async fn fetch_podcast(url: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?.bytes().await?;

    let channel = Channel::read_from(&response[..])?;
    println!("Title: {:?}", channel.title);
    println!("Description: {:?}", channel.description);
    println!("Image: {:?}", channel.image);
    println!("Episodes: {}", channel.items.len());

    if let Some(item) = channel.items.first() {
        println!("LATEST EPISODE");
        println!("Title: {:?}", item.title);
        println!("GUID: {:?}", item.guid);
        println!("Description: {:?}", item.description);
        println!("Pub date: {:?}", item.pub_date);
        println!("Itunes: {:?}", item.itunes_ext);
        println!("Enclosure: {:?}", item.enclosure);
    }

    Ok(())

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    fetch_podcast("https://pod.link/1171270672.rss").await
}
