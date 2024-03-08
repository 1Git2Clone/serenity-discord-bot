use crate::enums::command_enums::EmbedType;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub(crate) static ref COMMANDS: HashMap<EmbedType, Vec<&'static str>> = {
        let tieup_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1183349571468918814/tied-up-aiura.gif?ex=6588032b&is=65758e2b&hm=b92c39af90ca21bbbc2965487a418f89e5ff9fef02f2ad5722cbfb0bf0cbb3c1&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1183694079847059517/ezgif.com-video-to-gif.gif?ex=65894404&is=6576cf04&hm=a5f8a526218dc1d13a2957d6e3405f90cd6de1cc6d3d7e7b27d02108898db347&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183694247724077056/sasha-blouse.gif?ex=6589442c&is=6576cf2c&hm=63dc677cf5f449483f9f94372bd4d7e241f7aec2b92636e4e906ae22081bf04c&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1192391499376230511/8mb.video-Rjm-kx7W5rXN1.gif?ex=65a8e81f&is=6596731f&hm=e62bbc642a618f336583386090f3778e32db59ad2e6c95381f86c6aaeb2c12ac&",
        ];
        let pat_array = vec![
            "https://media.discordapp.net/attachments/1187355380087537668/1212438556409077831/gQIhfkz.gif?ex=65f1d665&is=65df6165&hm=cb48d221d2ef26bcc1def5122b28b95e31b73ce224dfecc44bfb95fbc927b02e&=",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461602700316673/kanna-kamui-pat.gif?ex=65886b81&is=6575f681&hm=db97fec641c2d019b14696ef63ec2f66f01e56f47645c6200cefbf73b788b43b&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461632718950460/pat.gif?ex=65886b88&is=6575f688&hm=b7b9d42ed41507c586fee897ca362b075f5fb1ea976ed7ce259a69cb24c71f4e&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183461661181497364/mai-sakurajima.gif?ex=65886b8f&is=6575f68f&hm=bd9a84bd007425cbf785bd28183c2ed32c2da97a254dc944f13e4e6b0b84bf63&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183493730339139694/hu-tao-hug.gif?ex=6588896d&is=6576146d&hm=f90650df440c5b14a3f965b8448a91f393539739dfdfb98bcceb956b384dd029&",
        ];
        let hug_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1183462503364186112/hug.gif?ex=65886c58&is=6575f758&hm=275d61937ad2b529274870ad670319c4f978513f9442e19e74e698ca7fb88448&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183462503011844096/anime-hug-anime-hugging.gif?ex=65886c58&is=6575f758&hm=a4138a57846d2ceeb75110e622353ce7506bf7f624d6a5f061e54dcefceb647b&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1183462502630174740/hug-surprise-chuunibyou.gif?ex=65886c58&is=6575f758&hm=311ad33b7ca80fe4a352a725b49ba29968ea17713d83b4292b2af0b64ee789f7&"
        ];
        let kiss_array = vec![
            "https://media.discordapp.net/attachments/614790390020833280/1184153815767855234/hutao-kiss.gif?ex=658af02e&is=65787b2e&hm=83a389fd93d6bd7b34a4ae7db2d9e7cfe3f29d819c503cc1e24c05018a070453&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184153816187277462/kiss.gif?ex=658af02e&is=65787b2e&hm=a43bc326b2d410a7bd797f806b7961ffc62ea32d7453f9a8be40fc3d3220ea3f&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184153816644468766/cute-kawai.gif?ex=658af02e&is=65787b2e&hm=685e92a6efc33064088f8862042a1591128722c9ac61d167cefd334ddbcd6ce4&=",
        ];
        let slap_array = vec![
            "https://media.discordapp.net/attachments/614790390020833280/1184154726238007349/genshin-impact-venti.gif?ex=658af107&is=65787c07&hm=610dce3dc84a7149573449cd51000b044b1daf2ba8b64b7547809f619e4cd83b&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184154726670028882/slap.gif?ex=658af107&is=65787c07&hm=a432223a617d6296ef0ea9627a0522d822f5f300d471a2208b38551f70e11395&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184154727286579210/anime-slap-mad.gif?ex=658af107&is=65787c07&hm=a59ed74001222dc70c1bbb40be3311e3bed4405d6469e9416ec3a7af8c344ac0&=",
        ];
        let punch_array = vec![
            "https://media.discordapp.net/attachments/614790390020833280/1184154350172508222/one-punch.gif?ex=658af0ad&is=65787bad&hm=b50f9fd6ac75cb0095182914f6d324c71543d8c6edf4943c71100366d0ca30c7&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184154350575169568/anime-fight.gif?ex=658af0ad&is=65787bad&hm=f3682a7426a04be86716517f519952bcfcd7ce8de3b08e9e3f2b57abc8b7c024&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184154351049113761/anime-smash.gif?ex=658af0ad&is=65787bad&hm=8a7c0dd6a847e099805224c55d42a4b742d8ca7e74731b28d3256d30afd3761a&=",
        ];
        let bonk_array = vec![
            "https://media.discordapp.net/attachments/614790390020833280/1184200805738348696/powerful-head-slap.gif?ex=658b1bf1&is=6578a6f1&hm=4690e1ecdfe9cc1a23d77bcc174ec1cf31c32ce9e007d02a024bbe960270f6db&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184200806245879828/atonnic-bonk.gif?ex=658b1bf1&is=6578a6f1&hm=007abbc5c5b7ec6140d752ebe6a1337a6ff461fa09a607539b1226ae984b7c97&=",
            "https://media.discordapp.net/attachments/614790390020833280/1184200806673686608/shinji-shinji-broom.gif?ex=658b1bf1&is=6578a6f1&hm=6d0d271fb33ad7d3e42a70365d7e10460bb219608a20c1093b8c9a9c3bb18ef8&=",
        ];
        let ryan_gosling_drive_array = vec![
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222721546756216/giphy.gif?ex=658ed3ad&is=657c5ead&hm=cea95c4af9bacd8149dae0a5be2b346b93895cd50dbacb2963758cd5cc6bcb92&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222722037481573/ryan-gosling-car.gif?ex=658ed3ad&is=657c5ead&hm=7c2236466b86584a766c3ac9dd09d4535dcf014f85738d1aa346fcf7aaf540b4&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222722545000488/ryan-gosling.gif?ex=658ed3ad&is=657c5ead&hm=8632f3e6c89fe7acf9491e760e2802185ec3bee4732add59709f2ff8e5346c0b&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222722926674013/ryan-gosling-ryan-gosling-drive.gif?ex=658ed3ad&is=657c5ead&hm=a2be391959c019bce6143fcfc5c6c0aab7c913ee649015d2628f287eacab5433&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222728068911134/ryan-gosling-drive.gif?ex=658ed3ae&is=657c5eae&hm=dd108c6cefac19939b032bec79cd89aae1a8450cb62e49296a35af78413fc0a1&",
            "https://cdn.discordapp.com/attachments/1180115044218978425/1185222728568021042/driving-ryan-gosling.gif?ex=658ed3ae&is=657c5eae&hm=ad2b217dac9dce31932ce40edb369ade1870195f9c4b7cfa34754cacf7e75f27&",
        ];
        let nom_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1185289189097476216/vsauce-michael-stevens.gif?ex=658f1194&is=657c9c94&hm=06d6794b1d8ede7f2e7b88a40db46e7e251e4c29fbc2a86338ae2b0358b58dbc&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185289189697278162/eatin-anima.gif?ex=658f1194&is=657c9c94&hm=72d465b5ab3ecc74dcc88e70becf6b0a4bb2e436c3d42ab42a85f298a93f8534&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185289190070550688/paimon-genshin.gif?ex=658f1194&is=657c9c94&hm=3856e3ecd75cb20477ac97544dd3632c4a3c964e094719950856cb7dfe4194e4&",
        ];
        let kill_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1185293538485870724/dead.gif?ex=658f15a1&is=657ca0a1&hm=bb00931507db300a86087fd31590916da5fb20e4bf68459ec337725d0d74a6ae&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185293538875936899/die-kill.gif?ex=658f15a1&is=657ca0a1&hm=d93d571e253152085d6ef1b7c36ea140a20181f22d90f03ea2e31b4460b23c4b&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185293539232460820/ira-gamagoori-attack.gif?ex=658f15a1&is=657ca0a1&hm=cc342930e5cf319f374e974e1ad421a292f131a5f5d8aff1d345901b535a9daf&",
            "https://cdn.discordapp.com/attachments/904591166580879400/1185318839177728020/wasted-wastedmidi.gif?ex=658f2d31&is=657cb831&hm=0bfccf35f557f0ba356a374d70976a8b7a7197a0d402fd837c92111cf7100326&",
        ];
        let kick_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1185566729104019486/falling-from-window-anime-death.gif",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185566728541966458/mad-angry.gif",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185566727845720195/kick-funny.gif",
        ];
        let bury_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1185635484412694549/mark-cooper-jones-jay-foreman.gif",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185635484945354862/nohemy-noh.gif",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185635485545144331/grave-rip.gif",
        ];
        let self_bury_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1185635416989253652/spongebob-squarepants-spongebob.gif",
            "https://cdn.discordapp.com/attachments/614790390020833280/1185635416594993172/dead-bury.gif"
        ];
        let chair_array = vec![
            "https://cdn.discordapp.com/attachments/614790390020833280/1186285033779122207/20231218_143252.gif?ex=6592b108&is=65803c08&hm=d0e4ae0b1733395d42537429b93d2fb41043a7666b3d1ecc27f2ac6372441300&",
            "https://cdn.discordapp.com/attachments/614790390020833280/1186290567190171658/vergil-chair.gif?ex=6592b62f&is=6580412f&hm=9befa3c6fcdba7688dfb6561602406df9059dbfbfd675af40dd6ffea1f2aedbb&",
        ];
        let peek_array = vec![
            "https://media.discordapp.net/attachments/614790390020833280/1203304453512372235/Hh4nIiw.gif?ex=65d09b9a&is=65be269a&hm=2289a079f5db64d138664742b71d8a7cf18bbd16a3fc55b60ef6b97195947ee7&=",
            "https://media.discordapp.net/attachments/614790390020833280/1203304454074671155/wkPTm8l.gif?ex=65d09b9a&is=65be269a&hm=b31872121ee68676b0a30c5288ec63f3cc17bf9a7ce908e177d4e93a61faf329&=",
            "https://media.discordapp.net/attachments/614790390020833280/1203304454582173696/aI1RZsy.gif?ex=65d09b9a&is=65be269a&hm=c33610f5882bbe4bb508c361e7605287e3d64487597ca2627fc2a53f558307f5&=",
            "https://media.discordapp.net/attachments/614790390020833280/1203304455043420200/4XviQL7.gif?ex=65d09b9a&is=65be269a&hm=0c3b7b2a2fcc8a018d376d72ac570b1c6e6acc4e06a06b340ba60ae2fe14df67&=",
            "https://media.discordapp.net/attachments/614790390020833280/1203304455554994226/wH7kSo2.gif?ex=65d09b9a&is=65be269a&hm=20c28587bcff491e8245c04e39645f8e1603aa964fb24657992eef8b60d51b9b&=",
            "https://media.discordapp.net/attachments/614790390020833280/1203304456007974942/1SMUFuk.gif?ex=65d09b9a&is=65be269a&hm=705ada33b0c670b3d50ab87cf40dab883a2b5d97a5cb287298ae9a8090e7d577&=",
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
