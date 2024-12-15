use crate::enums::command_enums::EmbedType;
use lazy_static::lazy_static;
use std::collections::HashMap;

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

lazy_static! {
    pub(crate) static ref COMMANDS: HashMap<EmbedType, Vec<&'static str>> = {
        let tieup_array = vec![
            cdn_url!("614790390020833280/1183349571468918814/tied-up-aiura.gif"),
            cdn_url!("1180115044218978425/1183694079847059517/ezgif.com-video-to-gif.gif"),
            cdn_url!("614790390020833280/1183694247724077056/sasha-blouse.gif"),
            cdn_url!("614790390020833280/1192391499376230511/8mb.video-Rjm-kx7W5rXN1.gif"),
        ];
        let pat_array = vec![
            media_url!("1187355380087537668/1212438556409077831/gQIhfkz.gif"),
            cdn_url!("614790390020833280/1183461602700316673/kanna-kamui-pat.gif"),
            cdn_url!("614790390020833280/1183461632718950460/pat.gif"),
            cdn_url!("614790390020833280/1183461661181497364/mai-sakurajima.gif"),
            cdn_url!("614790390020833280/1183493730339139694/hu-tao-hug.gif"),
        ];
        let hug_array = vec![
            cdn_url!("614790390020833280/1183462503364186112/hug.gif"),
            cdn_url!("614790390020833280/1183462503011844096/anime-hug-anime-hugging.gif"),
            cdn_url!("614790390020833280/1183462502630174740/hug-surprise-chuunibyou.gif"),
        ];
        let kiss_array = vec![
            media_url!("614790390020833280/1184153815767855234/hutao-kiss.gif"),
            media_url!("614790390020833280/1184153816187277462/kiss.gif"),
            media_url!("614790390020833280/1184153816644468766/cute-kawai.gif"),
        ];
        let slap_array = vec![
            media_url!("614790390020833280/1184154726238007349/genshin-impact-venti.gif"),
            media_url!("614790390020833280/1184154726670028882/slap.gif"),
            media_url!("614790390020833280/1184154727286579210/anime-slap-mad.gif"),
        ];
        let punch_array = vec![
            media_url!("614790390020833280/1184154350172508222/one-punch.gif"),
            media_url!("614790390020833280/1184154350575169568/anime-fight.gif"),
            media_url!("614790390020833280/1184154351049113761/anime-smash.gif"),
        ];
        let bonk_array = vec![
            media_url!("614790390020833280/1184200805738348696/powerful-head-slap.gif"),
            media_url!("614790390020833280/1184200806245879828/atonnic-bonk.gif"),
            media_url!("614790390020833280/1184200806673686608/shinji-shinji-broom.gif"),
        ];
        let ryan_gosling_drive_array = vec![
            cdn_url!("1180115044218978425/1185222721546756216/giphy.gif"),
            cdn_url!("1180115044218978425/1185222722037481573/ryan-gosling-car.gif"),
            cdn_url!("1180115044218978425/1185222722545000488/ryan-gosling.gif"),
            cdn_url!("1180115044218978425/1185222722926674013/ryan-gosling-ryan-gosling-drive.gif"),
            cdn_url!("1180115044218978425/1185222728068911134/ryan-gosling-drive.gif"),
            cdn_url!("1180115044218978425/1185222728568021042/driving-ryan-gosling.gif"),
        ];
        let nom_array = vec![
            cdn_url!("614790390020833280/1185289189097476216/vsauce-michael-stevens.gif"),
            cdn_url!("614790390020833280/1185289189697278162/eatin-anima.gif"),
            cdn_url!("614790390020833280/1185289190070550688/paimon-genshin.gif"),
        ];
        let kill_array = vec![
            cdn_url!("614790390020833280/1185293538485870724/dead.gif"),
            cdn_url!("614790390020833280/1185293538875936899/die-kill.gif"),
            cdn_url!("614790390020833280/1185293539232460820/ira-gamagoori-attack.gif"),
            cdn_url!("904591166580879400/1185318839177728020/wasted-wastedmidi.gif"),
        ];
        let kick_array = vec![
            cdn_url!("614790390020833280/1185566729104019486/falling-from-window-anime-death.gif"),
            cdn_url!("614790390020833280/1185566728541966458/mad-angry.gif"),
            cdn_url!("614790390020833280/1185566727845720195/kick-funny.gif"),
        ];
        let bury_array = vec![
            cdn_url!("614790390020833280/1185635484412694549/mark-cooper-jones-jay-foreman.gif"),
            cdn_url!("614790390020833280/1185635484945354862/nohemy-noh.gif"),
            cdn_url!("614790390020833280/1185635485545144331/grave-rip.gif"),
        ];
        let self_bury_array = vec![
            cdn_url!("614790390020833280/1185635416989253652/spongebob-squarepants-spongebob.gif"),
            cdn_url!("614790390020833280/1185635416594993172/dead-bury.gif"),
        ];
        let chair_array = vec![
            cdn_url!("614790390020833280/1186285033779122207/20231218_143252.gif"),
            cdn_url!("614790390020833280/1186290567190171658/vergil-chair.gif"),
        ];
        let peek_array = vec![
            media_url!("614790390020833280/1203304453512372235/Hh4nIiw.gif"),
            media_url!("614790390020833280/1203304454074671155/wkPTm8l.gif"),
            media_url!("614790390020833280/1203304454582173696/aI1RZsy.gif"),
            media_url!("614790390020833280/1203304455043420200/4XviQL7.gif"),
            media_url!("614790390020833280/1203304455554994226/wH7kSo2.gif"),
            media_url!("614790390020833280/1203304456007974942/1SMUFuk.gif"),
        ];

        HashMap::from([
            (EmbedType::TieUp, tieup_array),
            (EmbedType::Pat, pat_array),
            (EmbedType::Hug, hug_array),
            (EmbedType::Kiss, kiss_array),
            (EmbedType::Slap, slap_array),
            (EmbedType::Punch, punch_array),
            (EmbedType::Bonk, bonk_array),
            (EmbedType::RyanGoslingDrive, ryan_gosling_drive_array),
            (EmbedType::Nom, nom_array),
            (EmbedType::Kill, kill_array),
            (EmbedType::Kick, kick_array),
            (EmbedType::Bury, bury_array),
            (EmbedType::SelfBury, self_bury_array),
            (EmbedType::Chair, chair_array),
            (EmbedType::Peek, peek_array),
        ])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::command_data::Error;

    /// It is highly encouraged to run this test to check whether or not all your arrays have a
    /// vector with at least 1 link in it. If you don't run this test or if this test gives out an
    /// error, then that means that your program will panic if a user tries to get an embed_type
    /// from the key-value HashMap pair.
    #[test]
    fn test_vecs_not_empty() -> Result<(), Error> {
        for (embed_type, vec) in COMMANDS.iter() {
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

        for (_, vec) in COMMANDS.iter() {
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
                            .into())
                        }
                        code @ StatusCode::NOT_FOUND => {
                            // NOTE: Not guaranteed to be a hard error! This is due to discord's
                            // url query parameters, giving a 404 if the content is expried.
                            eprintln!("{} | URL: {}", code, url);

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
