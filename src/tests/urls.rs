use crate::prelude::*;

#[tokio::test]
async fn test_valid_asset_url() -> Result<(), Error> {
    use reqwest::Client;

    let client = Client::new();

    for asset in Assets::iter_variants() {
        let response = client.head(asset.to_string()).send().await?;
        assert_eq!(response.status(), StatusCode::OK);
    }

    Ok(())
}
