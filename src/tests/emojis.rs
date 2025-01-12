use crate::prelude::*;

#[tokio::test]
async fn test_valid_emoji_urls() -> Result<(), Error> {
    use reqwest::Client;

    let client = Client::new();

    let make_discord_emoji_url =
        |emoji_id: &str| format!("https://cdn.discordapp.com/emojis/{}", emoji_id);

    for variant in Emojis::variants().iter() {
        let response = client
            .head(make_discord_emoji_url(variant.get_id()))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
    }

    Ok(())
}
