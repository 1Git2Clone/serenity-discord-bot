use std::{collections::HashMap, sync::LazyLock, time::Duration};

use moka::future::Cache;
use poise::serenity_prelude::model::id::{GuildId, UserId};
use regex::Regex;
use serde::Deserialize;

#[cfg(feature = "ai")]
use crate::data::ai::AiChannelCache;
use crate::enums::command_enums::EmbedType;

#[derive(Deserialize)]
struct BotConfigRaw {
    prefixes: Vec<String>,
    mention_patterns: Vec<String>,
}

pub struct BotConfig {
    pub token: String,
    pub start_time: std::time::Instant,
    pub prefixes: Vec<String>,
    pub mention_patterns: Vec<String>,
    pub emoji_regex: Regex,
}

#[derive(Deserialize)]
struct XpConfigRaw {
    default_xp: i32,
    default_level: i32,
    min_xp: i32,
    max_xp: i32,
    cooldown_secs: i64,
}

pub struct XpConfig {
    pub default_xp: i32,
    pub default_level: i32,
    pub min_xp: i32,
    pub max_xp: i32,
    pub cooldown_secs: i64,
    pub cooldowns: Cache<(UserId, GuildId), ()>,
}

#[cfg(feature = "ai")]
#[derive(Deserialize)]
struct AiConfigRaw {
    rate_limit_secs: u64,
    default_stream: bool,
    num_predict: u32,
    temperature: f64,
}

#[cfg(feature = "ai")]
pub struct AiConfig {
    pub chat_endpoint: String,
    pub model: String,
    pub rate_limit_secs: u64,
    pub default_stream: bool,
    pub num_predict: u32,
    pub temperature: f64,
    pub rate_limit: Cache<UserId, ()>,
    pub channel_cache: AiChannelCache,
}

#[derive(Deserialize)]
pub struct LevelConfig {
    pub steps: Vec<f64>,
}

pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize)]
struct ConfigJson {
    bot: BotConfigRaw,
    xp: XpConfigRaw,
    #[cfg(feature = "ai")]
    ai: AiConfigRaw,
    level: LevelConfig,
}

pub struct Config {
    pub bot: BotConfig,
    pub xp: XpConfig,
    #[cfg(feature = "ai")]
    pub ai: AiConfig,
    pub level: LevelConfig,
    pub database: DatabaseConfig,
    pub embeds: HashMap<EmbedType, Vec<&'static str>>,
}

impl Config {
    fn load() -> Self {
        #[allow(
            clippy::expect_used,
            reason = "Fail fast on missing or malformed config."
        )]
        let raw = std::fs::read_to_string("config.json").unwrap_or_else(|_| {
            tracing::warn!("config.json not found, falling back to config.example.json");
            std::fs::read_to_string("config.example.json")
                .expect("neither config.json nor config.example.json found at repo root")
        });
        #[allow(
            clippy::expect_used,
            reason = "Fail fast on missing or malformed config."
        )]
        let json: ConfigJson =
            serde_json::from_str(&raw).expect("failed to parse config.json");

        #[allow(clippy::unwrap_used, reason = "Regex is a hardcoded valid pattern.")]
        let emoji_regex = Regex::new(concat!(
            "(?<emoji>",
            r":[^:\s]*:",
            ")|(?<embed_emoji>",
            r"\[[^\[\]]*\]\([^()]*\)",
            ")",
        ))
        .unwrap();

        Self {
            bot: BotConfig {
                token: std::env::var("BOT_TOKEN")
                    .expect("Expected a token in the dotenv file."),
                start_time: std::time::Instant::now(),
                emoji_regex,
                prefixes: json.bot.prefixes,
                mention_patterns: json.bot.mention_patterns,
            },
            xp: XpConfig {
                cooldowns: Cache::builder()
                    .time_to_live(Duration::from_secs(json.xp.cooldown_secs as u64))
                    .build(),
                default_xp: json.xp.default_xp,
                default_level: json.xp.default_level,
                min_xp: json.xp.min_xp,
                max_xp: json.xp.max_xp,
                cooldown_secs: json.xp.cooldown_secs,
            },
            #[cfg(feature = "ai")]
            ai: AiConfig {
                chat_endpoint: std::env::var("AI_CHAT_ENDPOINT")
                    .expect("Set the `AI_CHAT_ENDPOINT` environment variable."),
                model: std::env::var("AI_MODEL").expect("Set the `AI_MODEL` variable."),
                rate_limit: Cache::builder()
                    .time_to_live(Duration::from_secs(json.ai.rate_limit_secs))
                    .build(),
                channel_cache: AiChannelCache::new(),
                rate_limit_secs: json.ai.rate_limit_secs,
                default_stream: json.ai.default_stream,
                num_predict: json.ai.num_predict,
                temperature: json.ai.temperature,
            },
            level: json.level,
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .expect("Failed to get `DATABASE_URL` from the environment."),
            },
            embeds: {
                HashMap::from([
                    (EmbedType::TieUp, vec![
                        cdn_url!("614790390020833280/1183349571468918814/tied-up-aiura.gif"),
                        cdn_url!("1180115044218978425/1183694079847059517/ezgif.com-video-to-gif.gif"),
                        cdn_url!("614790390020833280/1183694247724077056/sasha-blouse.gif"),
                        cdn_url!("614790390020833280/1192391499376230511/8mb.video-Rjm-kx7W5rXN1.gif"),
                    ]),
                    (EmbedType::Pat, vec![
                        media_url!("1187355380087537668/1212438556409077831/gQIhfkz.gif"),
                        cdn_url!("614790390020833280/1183461602700316673/kanna-kamui-pat.gif"),
                        cdn_url!("614790390020833280/1183461632718950460/pat.gif"),
                        cdn_url!("614790390020833280/1183461661181497364/mai-sakurajima.gif"),
                        cdn_url!("614790390020833280/1183493730339139694/hu-tao-hug.gif"),
                    ]),
                    (EmbedType::Hug, vec![
                        cdn_url!("614790390020833280/1183462503364186112/hug.gif"),
                        cdn_url!("614790390020833280/1183462503011844096/anime-hug-anime-hugging.gif"),
                        cdn_url!("614790390020833280/1183462502630174740/hug-surprise-chuunibyou.gif"),
                    ]),
                    (EmbedType::Kiss, vec![
                        media_url!("614790390020833280/1184153815767855234/hutao-kiss.gif"),
                        media_url!("614790390020833280/1184153816187277462/kiss.gif"),
                        media_url!("614790390020833280/1184153816644468766/cute-kawai.gif"),
                    ]),
                    (EmbedType::Slap, vec![
                        media_url!("614790390020833280/1184154726238007349/genshin-impact-venti.gif"),
                        media_url!("614790390020833280/1184154726670028882/slap.gif"),
                        media_url!("614790390020833280/1184154727286579210/anime-slap-mad.gif"),
                    ]),
                    (EmbedType::Punch, vec![
                        media_url!("614790390020833280/1184154350172508222/one-punch.gif"),
                        media_url!("614790390020833280/1184154350575169568/anime-fight.gif"),
                        media_url!("614790390020833280/1184154351049113761/anime-smash.gif"),
                    ]),
                    (EmbedType::Bonk, vec![
                        media_url!("614790390020833280/1184200805738348696/powerful-head-slap.gif"),
                        media_url!("614790390020833280/1184200806245879828/atonnic-bonk.gif"),
                        media_url!("614790390020833280/1184200806673686608/shinji-shinji-broom.gif"),
                    ]),
                    (EmbedType::RyanGoslingDrive, vec![
                        cdn_url!("1180115044218978425/1185222721546756216/giphy.gif"),
                        cdn_url!("1180115044218978425/1185222722037481573/ryan-gosling-car.gif"),
                        cdn_url!("1180115044218978425/1185222722545000488/ryan-gosling.gif"),
                        cdn_url!("1180115044218978425/1185222722926674013/ryan-gosling-ryan-gosling-drive.gif"),
                        cdn_url!("1180115044218978425/1185222728068911134/ryan-gosling-drive.gif"),
                        cdn_url!("1180115044218978425/1185222728568021042/driving-ryan-gosling.gif"),
                    ]),
                    (EmbedType::Nom, vec![
                        cdn_url!("614790390020833280/1185289189097476216/vsauce-michael-stevens.gif"),
                        cdn_url!("614790390020833280/1185289189697278162/eatin-anima.gif"),
                        cdn_url!("614790390020833280/1185289190070550688/paimon-genshin.gif"),
                    ]),
                    (EmbedType::Kill, vec![
                        cdn_url!("614790390020833280/1185293538485870724/dead.gif"),
                        cdn_url!("614790390020833280/1185293538875936899/die-kill.gif"),
                        cdn_url!("614790390020833280/1185293539232460820/ira-gamagoori-attack.gif"),
                        cdn_url!("904591166580879400/1185318839177728020/wasted-wastedmidi.gif"),
                    ]),
                    (EmbedType::Kick, vec![
                        cdn_url!("614790390020833280/1185566729104019486/falling-from-window-anime-death.gif"),
                        cdn_url!("614790390020833280/1185566728541966458/mad-angry.gif"),
                        cdn_url!("614790390020833280/1185566727845720195/kick-funny.gif"),
                    ]),
                    (EmbedType::Bury, vec![
                        cdn_url!("614790390020833280/1185635484412694549/mark-cooper-jones-jay-foreman.gif"),
                        cdn_url!("614790390020833280/1185635484945354862/nohemy-noh.gif"),
                        cdn_url!("614790390020833280/1185635485545144331/grave-rip.gif"),
                    ]),
                    (EmbedType::SelfBury, vec![
                        cdn_url!("614790390020833280/1185635416989253652/spongebob-squarepants-spongebob.gif"),
                        cdn_url!("614790390020833280/1185635416594993172/dead-bury.gif"),
                    ]),
                    (EmbedType::Chair, vec![
                        cdn_url!("614790390020833280/1186285033779122207/20231218_143252.gif"),
                        cdn_url!("614790390020833280/1186290567190171658/vergil-chair.gif"),
                    ]),
                    (EmbedType::Peek, vec![
                        media_url!("614790390020833280/1203304453512372235/Hh4nIiw.gif"),
                        media_url!("614790390020833280/1203304454074671155/wkPTm8l.gif"),
                        media_url!("614790390020833280/1203304454582173696/aI1RZsy.gif"),
                        media_url!("614790390020833280/1203304455043420200/4XviQL7.gif"),
                        media_url!("614790390020833280/1203304455554994226/wH7kSo2.gif"),
                        media_url!("614790390020833280/1203304456007974942/1SMUFuk.gif"),
                    ]),
                ])
            },
        }
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);
