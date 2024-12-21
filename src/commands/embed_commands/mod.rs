/// dependencies for the commands.
use crate::commands::{cmd_utils, cmd_utils::get_replied_user, cmd_utils::make_full_response};
use crate::data::bot_data::START_TIME;
use crate::data::command_data::{Context, Error};
use crate::enums::command_enums::EmbedType;
use ::serenity::all::Mentionable;
use poise::serenity_prelude as serenity;
use std::sync::Arc;

// #region User interaction commands

pub mod tieup;
pub use tieup::tieup;

pub mod pat;
pub use pat::pat;

pub mod hug;
pub use hug::hug;

pub mod kiss;
pub use kiss::kiss;

pub mod slap;
pub use slap::slap;

pub mod punch;
pub use punch::punch;

pub mod bonk;
pub use bonk::bonk;

pub mod nom;
pub use nom::nom;

pub mod kill;
pub use kill::kill;

pub mod kick;
pub use kick::kick;

pub mod bury;
pub use bury::bury;

pub mod selfbury;
pub use selfbury::selfbury;

pub mod peek;
pub use peek::peek;

// #endregion

pub mod avatar;
pub use avatar::avatar;

pub mod drive;
pub use drive::drive;

pub mod chair;
pub use chair::chair;

pub mod boom;
pub use boom::boom;

pub mod quote;
pub use quote::quote;

pub mod uptime;
pub use uptime::uptime;
