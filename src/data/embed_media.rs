use crate::enums::command_enums::EmbedType;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub(crate) static ref COMMANDS: HashMap<EmbedType, Vec<&'static str>> = {
        let pat_array = vec![
            "https://media.discordapp.net/attachments/1187355380087537668/1212438556409077831/gQIhfkz.gif?ex=65f1d665&is=65df6165&hm=cb48d221d2ef26bcc1def5122b28b95e31b73ce224dfecc44bfb95fbc927b02e&=",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461602700316673/kanna-kamui-pat.gif?ex=65886b81&is=6575f681&hm=db97fec641c2d019b14696ef63ec2f66f01e56f47645c6200cefbf73b788b43b&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461632718950460/pat.gif?ex=65886b88&is=6575f688&hm=b7b9d42ed41507c586fee897ca362b075f5fb1ea976ed7ce259a69cb24c71f4e&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461661181497364/mai-sakurajima.gif?ex=65886b8f&is=6575f68f&hm=bd9a84bd007425cbf785bd28183c2ed32c2da97a254dc944f13e4e6b0b84bf63&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183493730339139694/hu-tao-hug.gif?ex=6588896d&is=6576146d&hm=f90650df440c5b14a3f965b8448a91f393539739dfdfb98bcceb956b384dd029&",
        ];

        HashMap::from([(EmbedType::Pat, pat_array)])
    };
}
