use crate::prelude::*;

#[macro_export]
macro_rules! cdn_url {
    ($expr:expr) => {
        concat!("https://cdn.discordapp.com/attachments/", $expr)
    };
}

#[macro_export]
macro_rules! media_url {
    ($expr:expr) => {
        concat!("https://media.discordapp.net/attachments/", $expr)
    };
}


#[cfg(test)]
mod tests {
    use crate::data::command_data::Error;
    use crate::prelude::*;

    /// It is highly encouraged to run this test to check whether or not all your arrays have a
    /// vector with at least 1 link in it. If you don't run this test or if this test gives out an
    /// error, then that means that your program will panic if a user tries to get an embed_type
    /// from the key-value HashMap pair.
    #[test]
    fn test_vecs_not_empty() -> Result<(), Error> {
        for (embed_type, vec) in CONFIG.embeds.iter() {
            assert!(!vec.is_empty(), "{:?} array is empty", embed_type);
        }

        Ok(())
    }

    // NOTE: I don't recommend you run this test more than once or twice due to the chance of you
    // getting rate limited, however, it's still important to assert all the URLs are correct.
    #[cfg(feature = "network_test")]
    #[tokio::test]
    async fn test_bad_request() -> Result<(), Error> {
        use reqwest::{Client, StatusCode};
        let client = Client::new();

        for (_, vec) in CONFIG.embeds.iter() {
            for url in vec.iter() {
                match client.head(*url).send().await {
                    Ok(resp) => match resp.status() {
                        StatusCode::OK => (),
                        StatusCode::BAD_REQUEST => {
                            return Err(format!(
                                "Bad request! ({}) (at URL: {})",
                                resp.status(),
                                url
                            )
                            .into());
                        }
                        code @ StatusCode::NOT_FOUND => {
                            // NOTE: Not guaranteed to be a hard error! This is due to discord's
                            // url query parameters, giving a 404 if the content is expried.
                            tracing::error!("{} | URL: {}", code, url);

                            let response_body = reqwest::get(*url).await?.text().await?;
                            assert_eq!(response_body, "This content is no longer available.");
                        }
                        code => return Err(format!("{} | URL: {}", code, url).into()),
                    },
                    Err(err) => return Err(format!("{} | URL: {}", err, url).into()),
                }
            }
        }

        Ok(())
    }
}
