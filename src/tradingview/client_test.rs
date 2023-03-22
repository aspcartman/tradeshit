use anyhow::Result;

use super::client::Client;

#[tokio::test]
async fn it_works() -> Result<()> {
    let (mut client, mut qrx) = Client::new().await?;
    client.subscribe("MOEX:NGH2023").await?;
    while let Some(q) = qrx.recv().await {
        println!("YAY! {:?}", q)
    }
    println!("oh");
    Ok(())
}
