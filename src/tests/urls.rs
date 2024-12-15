use reqwest::StatusCode;

use crate::commands::cmd_utils::HU_BOOM_URL;
use crate::data::command_data::Error;

#[tokio::test]
async fn test_valid_asset_url() -> Result<(), Error> {
    use reqwest::Client;

    let client = Client::new();

    let response = client.head(HU_BOOM_URL).send().await?;

    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
