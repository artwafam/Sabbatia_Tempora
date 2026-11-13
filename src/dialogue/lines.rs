//! What the companions say. Each entry has a condition (`When`) and one to
//! three steps; a step is spoken by the companion or, for `ask!`, voiced as
//! the user's own question. Portuguese is written as Brazilian speech, not
//! translated word for word.
//!
//! Placeholders: `{sabbat}` (next festival), `{days}` ("39 days" /
//! "39 dias"), `{season}` (current season name).

use super::{Cue, Entry, Step, When};
use crate::calendar::{Sabbat, Season};

macro_rules! say {
    ($cue:ident, $en:expr, $pt:expr) => {
        Step { you: false, cue: Cue::$cue, en: $en, pt: $pt }
    };
}

macro_rules! ask {
    ($en:expr, $pt:expr) => {
        Step { you: true, cue: Cue::Still, en: $en, pt: $pt }
    };
}

macro_rules! entry {
    ($id:expr, $when:expr, [$($step:expr),+ $(,)?]) => {
        Entry { id: $id, when: $when, steps: &[$($step),+] }
    };
}

use When::*;

/// Wise, reserved, patient, dryly funny. Comfortable with silence.
pub static HERMIT: &[Entry] = &[
    // ── greetings and general ──
    entry!("h.back", Always, [say!(Still, "Ah. You're back.", "Ah. Você voltou.")]),
    entry!("h.moment", Always, [say!(Bow, "Some things deserve a moment of attention.", "Algumas coisas merecem um instante de atenção.")]),
    entry!("h.closer", Always, [
        say!(Raise, "Come closer. There is something I'd like to show you.", "Chegue mais perto. Quero lhe mostrar uma coisa."),
        say!(Still, "The Almanac. It has been waiting patiently.", "O Almanaque. Ele esperou com paciência."),
    ]),
    entry!("h.hurry", Always, [say!(Still, "Do not hurry. The world will keep turning.", "Não se apresse. O mundo vai continuar girando.")]),
    entry!("h.quiet", Always, [say!(Bow, "I was listening to the quiet. It had a lot to say.", "Eu estava ouvindo o silêncio. Ele tinha muito a dizer.")]),
    entry!("h.lantern", Always, [
        say!(Raise, "You may wonder why I carry a lantern in daylight.", "Talvez você se pergunte por que carrego uma lanterna durante o dia."),
        say!(Still, "Habit, mostly.", "Costume, principalmente."),
        say!(Swing, "And because one never knows when it will get dark.", "E porque nunca se sabe quando vai escurecer."),
    ]),
    entry!("h.path", Always, [say!(Still, "The path is long. That is rather the point.", "O caminho é longo. A graça está justamente nisso.")]),
    entry!("h.question", Always, [
        say!(Bow, "You have a question. I can tell.", "Você tem uma pergunta. Dá para perceber."),
        say!(Still, "Keep it a while. Good questions improve with age.", "Guarde-a por um tempo. Boas perguntas melhoram com a idade."),
    ]),
    entry!("h.mountain", Always, [
        say!(Still, "I once climbed a mountain to find an answer.", "Uma vez subi uma montanha para encontrar uma resposta."),
        ask!("And did you find it?", "E encontrou?"),
        say!(Raise, "I found a better view. That was answer enough.", "Encontrei uma vista melhor. Já foi resposta suficiente."),
    ]),
    entry!("h.lesson", Always, [say!(Bow, "Every season has its own lesson.", "Toda estação tem sua própria lição.")]),
    entry!("h.moonwatch", Always, [
        say!(Raise, "I have watched the Moon for longer than I care to admit.", "Observo a Lua há mais tempo do que gosto de admitir."),
        say!(Still, "It has never once been in a hurry.", "Ela nunca teve pressa. Nem uma vez."),
    ]),
    entry!("h.countdown", Always, [say!(Still, "{sabbat} is {days} away. I am in no hurry. Neither should you be.", "Faltam {days} para {sabbat}. Não tenho pressa. Você também não deveria ter.")]),

    // ── humour ──
    entry!("h.tabs", Always, [say!(Still, "Wisdom begins when we learn to close a few tabs.", "A sabedoria começa quando aprendemos a fechar algumas abas.")]),
    entry!("h.progress", Always, [say!(Bow, "My advice? Do not confuse movement with progress.", "Meu conselho? Não confunda movimento com progresso.")]),
    entry!("h.fool", Always, [
        say!(Still, "If you meet a young man about to step off a cliff, do not worry.", "Se encontrar um rapaz prestes a sair andando de um penhasco, não se preocupe."),
        say!(Bow, "He always lands somewhere.", "Ele sempre cai em algum lugar."),
    ]),
    entry!("h.pilgrims", Always, [
        say!(Still, "People climb mountains to ask me things.", "As pessoas sobem montanhas para me perguntar coisas."),
        say!(Bow, "Mostly they ask how to get back down.", "Quase sempre perguntam como se desce."),
    ]),
    entry!("h.beard", Always, [say!(Still, "The staff is for walking. The beard is for thinking.", "O cajado é para caminhar. A barba é para pensar.")]),
    entry!("h.working", LongSession, [say!(Bow, "You've been working for quite some time. Even the wise need rest.", "Você está trabalhando há um bom tempo. Até os sábios precisam descansar.")]),
    entry!("h.window", LongSession, [say!(Still, "Stand up. Look out of a window. I will wait.", "Levante-se. Olhe pela janela. Eu espero.")]),

    // ── time of day ──
    entry!("h.morning", Morning, [say!(Raise, "Morning. The light is still deciding what to be.", "Bom dia. A luz ainda está decidindo o que vai ser.")]),
    entry!("h.evening", Evening, [say!(Raise, "Evening comes. The lantern begins to earn its keep.", "A noite vem chegando. A lanterna começa a ter serventia.")]),
    entry!("h.late", Night, [say!(Bow, "It is late. Even thoughts need sleep.", "É tarde. Até os pensamentos precisam dormir.")]),
    entry!("h.quietnight", Night, [say!(Still, "The night is quiet. Let it stay that way a little longer.", "A noite está quieta. Deixe-a assim mais um pouco.")]),
    entry!("h.awake", Night, [
        say!(Still, "Still up?", "Ainda de pé?"),
        say!(Bow, "So am I. An old habit.", "Eu também. Hábito antigo."),
    ]),

    // ── seasons ──
    entry!("h.winter", InSeason(Season::Winter), [say!(Bow, "Winter is not the end. It is simply another form of rest.", "O inverno não é o fim. É apenas outra forma de descanso.")]),
    entry!("h.longer", Lengthening, [say!(Raise, "The days are growing longer again. Notice the light.", "Os dias estão ficando mais longos de novo. Repare na luz.")]),
    entry!("h.bloom", InSeason(Season::Spring), [say!(Still, "Not everything blooms at the same time.", "Nem tudo floresce ao mesmo tempo.")]),
    entry!("h.summer", InSeason(Season::Summer), [say!(Raise, "Summer. The sun at its strongest, and already turning back.", "Verão. O sol no auge da força, e já começando a voltar.")]),
    entry!("h.autumn", InSeason(Season::Autumn), [say!(Bow, "Autumn teaches the trees to let go. They are better at it than we are.", "O outono ensina as árvores a deixar ir. Elas fazem isso melhor do que nós.")]),
    entry!("h.shorter", Shortening, [say!(Raise, "The days are getting shorter. Light the lamp a little earlier.", "Os dias estão ficando mais curtos. Acenda a lamparina um pouco mais cedo.")]),
    entry!("h.suits", Always, [
        say!(Still, "{season}, then.", "{season}, então."),
        say!(Bow, "Hm. It suits you.", "Hm. Combina com você."),
    ]),
    entry!("h.turning", SeasonTurning, [say!(Bow, "The season is turning. Pay attention; it happens quietly.", "A estação está virando. Preste atenção: isso acontece em silêncio.")]),

    // ── moon ──
    entry!("h.full", FullMoon, [say!(Raise, "The Moon is full tonight. A good occasion to observe the sky.", "A Lua está cheia esta noite. Boa ocasião para observar o céu.")]),
    entry!("h.full2", FullMoon, [say!(Swing, "Full moon. My lantern feels rather unnecessary tonight.", "Lua cheia. Minha lanterna se sente meio dispensável esta noite.")]),
    entry!("h.new", NewMoon, [say!(Bow, "The Moon disappears from our sight, but it does not cease to exist.", "A Lua some da nossa vista, mas não deixa de existir.")]),
    entry!("h.dark", NewMoon, [say!(Still, "There is beauty in darkness, too.", "Também há beleza na escuridão.")]),
    entry!("h.waxing", Waxing, [
        say!(Raise, "The Moon is waxing.", "A Lua está crescendo."),
        ask!("And what does that mean?", "E o que isso quer dizer?"),
        say!(Still, "That its light is returning, little by little. There is something comforting about that.", "Que a luz dela está voltando, aos poucos. Há algo de reconfortante nisso."),
    ]),
    entry!("h.waning", Waning, [say!(Bow, "The Moon is waning. What wanes is not lost. It rests.", "A Lua está minguando. O que mingua não se perde. Descansa.")]),

    // ── sabbats ──
    entry!("h.today", SabbatToday, [say!(Raise, "Today is {sabbat}. Another cycle comes to an end. Another begins.", "Hoje é {sabbat}. Um ciclo se encerra. Outro começa.")]),
    entry!("h.today2", SabbatToday, [say!(Bow, "{sabbat}. Mark the day, quietly if you prefer.", "{sabbat}. Marque o dia, em silêncio se preferir.")]),
    entry!("h.soon", SabbatSoon, [say!(Still, "{sabbat} in {days}. There is time enough.", "{sabbat} em {days}. Há tempo de sobra.")]),
    entry!("h.samhain", Approaching(Sabbat::Samhain), [say!(Raise, "Samhain approaches. A transition worth observing.", "Samhain se aproxima. Uma passagem que merece ser observada.")]),
    entry!("h.beltane", Approaching(Sabbat::Beltane), [say!(Raise, "Beltane draws near. Life finds new ways to bloom.", "Beltane se aproxima. A vida encontra novas maneiras de florescer.")]),
    entry!("h.yule", Approaching(Sabbat::Yule), [say!(Raise, "Yule approaches. The longest night, and then the light returns. It always has.", "Yule se aproxima. A noite mais longa, e depois a luz volta. Sempre voltou.")]),
    entry!("h.litha", Approaching(Sabbat::Litha), [say!(Raise, "Litha is near. The longest day. Enjoy it; it is also where the decline begins.", "Litha está perto. O dia mais longo. Aproveite: é também onde começa o declínio.")]),
    entry!("h.imbolc", Approaching(Sabbat::Imbolc), [say!(Bow, "Imbolc soon. The first stirrings beneath the frost.", "Imbolc vem aí. Os primeiros sinais de vida sob a geada.")]),
    entry!("h.ostara", Approaching(Sabbat::Ostara), [say!(Still, "Ostara approaches. Day and night, briefly in agreement.", "Ostara se aproxima. Dia e noite, por um instante, de acordo.")]),
    entry!("h.lughnasadh", Approaching(Sabbat::Lughnasadh), [
        say!(Raise, "Lughnasadh comes. The first harvest.", "Lughnasadh vem aí. A primeira colheita."),
        say!(Still, "What have you gathered this year?", "O que você colheu este ano?"),
    ]),
    entry!("h.mabon", Approaching(Sabbat::Mabon), [say!(Bow, "Mabon is near. Balance once more, and then the long walk into the dark.", "Mabon está perto. Equilíbrio mais uma vez, e depois a longa caminhada rumo ao escuro.")]),

    // ── inactivity: almost never ──
    entry!("h.still_here", Inactive, [say!(Still, "I am still here. Take your time.", "Continuo aqui. Sem pressa.")]),
];

/// Curious, spontaneous, playful, easily distracted. His dog joins in.
pub static FOOL: &[Entry] = &[
    // ── greetings and general ──
    entry!("f.there", Always, [say!(Hop, "Oh! There you are!", "Ah! Aí está você!")]),
    entry!("f.discover", Always, [say!(Raise, "Shall we discover something interesting?", "Vamos descobrir alguma coisa interessante?")]),
    entry!("f.twenty", Always, [
        say!(LookUp, "I have a question...", "Tenho uma pergunta..."),
        say!(Step, "Actually, I have about twenty.", "Na verdade, tenho umas vinte."),
        say!(LookUp, "Let's start with the easy one: why is the sky up there?", "Começando pela mais fácil: por que o céu fica lá em cima?"),
    ]),
    entry!("f.sky", Always, [say!(LookUp, "Have you looked at the sky today?", "Você já olhou para o céu hoje?")]),
    entry!("f.anywhere", Always, [say!(Step, "Where are we going today? Anywhere is fine!", "Para onde vamos hoje? Qualquer lugar serve!")]),
    entry!("f.dog_hi", Always, [say!(Hop, "The dog says hello. Well, he barked. I'm translating.", "O cachorro mandou um oi. Bom, ele latiu. Eu estou traduzindo.")]),
    entry!("f.bag", Always, [
        say!(Swing, "Want to know what's in my bag?", "Quer saber o que tem na minha trouxa?"),
        say!(Step, "Me too! I packed it in a hurry.", "Eu também! Arrumei com pressa."),
    ]),
    entry!("f.rose", Always, [say!(Raise, "I found this rose this morning. Or maybe it found me.", "Achei esta rosa hoje cedo. Ou foi ela que me achou.")]),
    entry!("f.cliff", Always, [
        say!(LookUp, "Is there a cliff in front of me?", "Tem um penhasco na minha frente?"),
        ask!("Yes. A big one.", "Tem. Bem grande."),
        say!(Raise, "Oh. Well, the view is lovely!", "Ah. Bom, a vista é linda!"),
    ]),
    entry!("f.countdown", Always, [say!(Step, "{days} until {sabbat}. That's practically tomorrow! Well, not really.", "Faltam {days} para {sabbat}. É praticamente amanhã! Bom, quase.")]),

    // ── humour ──
    entry!("f.idea", Always, [say!(LookUp, "I had a brilliant idea... has anyone seen where it went?", "Tive uma ideia brilhante... alguém viu para onde ela foi?")]),
    entry!("f.moon_dog", Always, [say!(Hop, "The dog is convinced the Moon belongs to him.", "O cachorro tem certeza de que a Lua é dele.")]),
    entry!("f.where", Always, [say!(Step, "I don't know where we're going, but it looks interesting!", "Não sei para onde estamos indo, mas parece interessante!")]),
    entry!("f.dream", Always, [say!(Ponder, "Do you think someone can get lost inside a dream?", "Você acha que dá para se perder dentro de um sonho?")]),
    entry!("f.screen", Always, [
        say!(LookUp, "Do you know what's on the other side of the screen?", "Você sabe o que tem do outro lado da tela?"),
        say!(Hop, "The dog thinks it's more desktop.", "O cachorro acha que é mais área de trabalho."),
        say!(Raise, "The dog is usually right.", "O cachorro costuma ter razão."),
    ]),
    entry!("f.map", Always, [say!(Swing, "My map has no roads. I drew it myself.", "Meu mapa não tem estradas. Fui eu que desenhei.")]),
    entry!("f.hermit", Always, [
        say!(Step, "There's an old man with a lantern who keeps telling me to watch my step.", "Tem um velho com uma lanterna que vive me dizendo para olhar onde piso."),
        say!(LookUp, "Watch it where, though?", "Olhar onde, né?"),
    ]),
    entry!("f.staring", LongSession, [say!(Step, "You've been staring at that screen for ages. Let's go for a walk!", "Você está olhando para essa tela há séculos. Vamos dar uma volta!")]),
    entry!("f.play", LongSession, [say!(Hop, "The dog wants to play. Honestly, so do I.", "O cachorro quer brincar. Para ser sincero, eu também.")]),

    // ── time of day ──
    entry!("f.morning", Morning, [say!(Hop, "Good morning! Today could be anything!", "Bom dia! Hoje pode ser qualquer coisa!")]),
    entry!("f.sunset", Evening, [say!(LookUp, "The sun is setting. Do you think it goes somewhere nice?", "O sol está se pondo. Será que ele vai para algum lugar legal?")]),
    entry!("f.snoring", Night, [say!(Ponder, "It's late... the dog is already snoring.", "Já é tarde... o cachorro já está roncando.")]),
    entry!("f.miss", Night, [say!(Ponder, "I should sleep. But what if I miss something?", "Eu devia dormir. Mas e se eu perder alguma coisa?")]),
    entry!("f.psst_night", Night, [
        say!(LookUp, "Psst. Still up?", "Psiu. Ainda de pé?"),
        say!(Ponder, "Me too. Shh, don't wake the dog.", "Eu também. Shh, não acorda o cachorro."),
    ]),

    // ── seasons ──
    entry!("f.flowers", InSeason(Season::Spring), [say!(Raise, "Look! The flowers are coming back!", "Olha! As flores estão voltando!")]),
    entry!("f.plans", InSeason(Season::Spring), [say!(Hop, "Summer is coming! I have plans. Lots of plans!", "O verão está chegando! Tenho planos. Muitos planos!")]),
    entry!("f.trees", InSeason(Season::Spring), [say!(LookUp, "Do you think trees know when it's spring?", "Você acha que as árvores sabem quando é primavera?")]),
    entry!("f.summer", InSeason(Season::Summer), [say!(Step, "Summer! Long days, more time to wander!", "Verão! Dias longos, mais tempo para andar por aí!")]),
    entry!("f.snow", InSeason(Season::Autumn), [say!(Ponder, "It's getting colder. I wonder if the dog likes snow.", "Está ficando mais frio. Será que o cachorro gosta de neve?")]),
    entry!("f.winter", InSeason(Season::Winter), [say!(Ponder, "Winter! Everything is so quiet. Suspiciously quiet.", "Inverno! Está tudo tão quieto. Quieto demais.")]),
    entry!("f.longer", Lengthening, [say!(Hop, "The days are getting longer! More day for the same price!", "Os dias estão ficando mais longos! Mais dia pelo mesmo preço!")]),
    entry!("f.shorter", Shortening, [say!(Ponder, "The days are getting shorter. Who's taking them?", "Os dias estão ficando mais curtos. Quem está levando eles embora?")]),
    entry!("f.turning", SeasonTurning, [say!(LookUp, "Wait, the season's changing? Already? When did that happen?", "Espera, a estação está mudando? Já? Quando foi isso?")]),

    // ── moon ──
    entry!("f.enormous", FullMoon, [say!(Raise, "Look! The Moon is enormous!", "Olha! A Lua está enorme!")]),
    entry!("f.howl", FullMoon, [
        say!(Raise, "Full moon tonight!", "Lua cheia esta noite!"),
        say!(Hop, "The dog is going to howl. I might join him.", "O cachorro vai uivar. Talvez eu uive junto."),
    ]),
    entry!("f.blank", NewMoon, [say!(Raise, "A new Moon! It looks like a blank page.", "Lua nova! Parece uma página em branco.")]),
    entry!("f.where_moon", NewMoon, [
        say!(LookUp, "Where does the Moon go when it's new?", "Para onde a Lua vai quando é nova?"),
        say!(Hop, "I asked the dog.", "Perguntei para o cachorro."),
        say!(Raise, "He didn't know either, but he looked very confident.", "Ele também não sabia, mas fez cara de quem sabia."),
    ]),
    entry!("f.hide", Waning, [say!(LookUp, "It's disappearing again. Is it playing hide-and-seek?", "Ela está sumindo de novo. Será que está brincando de esconde-esconde?")]),
    entry!("f.growing", Waxing, [say!(Hop, "The Moon is growing! Good for her.", "A Lua está crescendo! Que bom para ela.")]),

    // ── sabbats ──
    entry!("f.today", SabbatToday, [say!(Hop, "It's {sabbat}! Today! Right now!", "É {sabbat}! Hoje! Agora mesmo!")]),
    entry!("f.happy", SabbatToday, [
        say!(Raise, "Happy {sabbat}!", "Feliz {sabbat}!"),
        say!(Ponder, "I'm not exactly sure how to celebrate it.", "Não sei bem como se comemora."),
        say!(Step, "So I'm doing a little bit of everything.", "Então estou fazendo um pouquinho de tudo."),
    ]),
    entry!("f.soon", SabbatSoon, [say!(Hop, "{sabbat} in {days}! I'm counting. The dog lost count.", "{sabbat} em {days}! Estou contando. O cachorro perdeu a conta.")]),
    entry!("f.lughnasadh", Approaching(Sabbat::Lughnasadh), [say!(LookUp, "Lughnasadh is coming! Does that mean the year is already halfway over?", "Lughnasadh está chegando! Quer dizer que o ano já está na metade?")]),
    entry!("f.beltane", Approaching(Sabbat::Beltane), [say!(Raise, "Beltane! I like this time of year. Everything seems to be waking up!", "Beltane! Gosto desta época. Parece que tudo está acordando!")]),
    entry!("f.samhain", Approaching(Sabbat::Samhain), [say!(Ponder, "Samhain... I like the longer nights. I think the dog does, too.", "Samhain... Gosto das noites mais longas. Acho que o cachorro também.")]),
    entry!("f.yule", Approaching(Sabbat::Yule), [say!(LookUp, "Yule is coming! The sun comes back after that, right? Right?", "Yule está chegando! O sol volta depois disso, né? Né?")]),
    entry!("f.imbolc", Approaching(Sabbat::Imbolc), [say!(Step, "Imbolc soon! Something is about to sprout. I can feel it in my boots.", "Imbolc vem aí! Alguma coisa está para brotar. Sinto nas botas.")]),
    entry!("f.ostara", Approaching(Sabbat::Ostara), [say!(LookUp, "Ostara! Day and night are tied. Who wins next?", "Ostara! Dia e noite empatados. Quem será que ganha agora?")]),
    entry!("f.litha", Approaching(Sabbat::Litha), [say!(Hop, "Litha! The longest day! I'm going to use every minute of it.", "Litha! O dia mais longo! Vou aproveitar cada minuto.")]),
    entry!("f.mabon", Approaching(Sabbat::Mabon), [say!(Swing, "Mabon is near! The second harvest. I didn't plant anything, but I can help eat.", "Mabon está perto! A segunda colheita. Não plantei nada, mas posso ajudar a comer.")]),

    // ── calling out after a long quiet ──
    entry!("f.psst", Inactive, [say!(Hop, "Psst! Over here! ...Just checking you're still there.", "Psiu! Aqui! ...Só conferindo se você ainda está aí.")]),
    entry!("f.stick", Inactive, [say!(Hop, "The dog found a stick. Want to see?", "O cachorro achou um graveto. Quer ver?")]),
    entry!("f.waiting", Inactive, [say!(Step, "I've been standing here a while. Not complaining! Just saying.", "Estou aqui parado faz um tempo. Não é reclamação! Só comentando.")]),
];
