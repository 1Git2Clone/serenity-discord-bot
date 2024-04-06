use poise::serenity_prelude as serenity;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Data {
    pub poise_mentions: AtomicU32,
    pub bot_user: Arc<serenity::CurrentUser>,
    // I'm not sure why clippy gives a warning, this works just fine...
    // #[cfg_attr(feature = "serde", serde(skip))]
}

/// This is a useful way to test if your structs can be syncronized.
/// Take for example using Rc<T> instead of Arc<T>
/// It'll give an error on compile time telling you that you can't synchronize the data safely.
///
/// NOTE - The Arc<T> vs Rc<T> example won't work with this exact code sample because both of
/// them can't be serialized. (serde::Serialize)
/// In order to use data that can't be serialized or deserlialized you need to do the following:
///
/// ```rust
/// pub struct Data {
///     // Existing data...
///     #[cfg_attr(feature = "serde", serde(skip))]
///     pub some_unserializable_data: std::sync::Arc<i32>,
/// }
/// ```
///
/// Tutorial vid for the topic:
/// https://www.youtube.com/watch?v=Nzclc6MswaI
fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
#[test]
fn normal_types() {
    _is_normal::<Data>();
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
pub type Context<'a> = poise::Context<'a, Data, Error>;
