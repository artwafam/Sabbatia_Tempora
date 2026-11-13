//! Static descriptive texts, in each supported language.
//!
//! Historical statements are deliberately conservative and limited to what
//! the cited works support. Where a name or association is modern, the text
//! says so. Correspondences are given as commonly used in contemporary Pagan
//! practice, not as ancient fact. Translations make no claim the English
//! text does not make; cited titles stay in their original language.

use super::{Sabbat, Season};
use crate::i18n::Lang;

pub struct SabbatLore {
    pub epithet: &'static str,
    pub history: &'static str,
    pub significance: &'static str,
    pub symbolism: &'static str,
    pub sources: &'static [&'static str],
}

const HUTTON_STATIONS: &str =
    "R. Hutton, The Stations of the Sun: A History of the Ritual Year in Britain (Oxford UP, 1996)";
const HUTTON_TRIUMPH: &str =
    "R. Hutton, The Triumph of the Moon: A History of Modern Pagan Witchcraft (Oxford UP, 1999)";
const DANAHER: &str = "K. Danaher, The Year in Ireland (Mercier Press, 1972)";
const MACNEILL: &str = "M. MacNeill, The Festival of Lughnasa (Oxford UP, 1962)";
const BEDE: &str = "Bede, The Reckoning of Time (De temporum ratione, AD 725), ch. 15, tr. F. Wallis (Liverpool UP, 1999)";
const GRIMM: &str = "J. Grimm, Deutsche Mythologie (1835)";

const SRC_SAMHAIN: &[&str] = &[HUTTON_STATIONS, DANAHER];
const SRC_YULE: &[&str] = &[BEDE, HUTTON_STATIONS];
const SRC_IMBOLC: &[&str] = &[DANAHER, HUTTON_STATIONS];
const SRC_OSTARA: &[&str] = &[BEDE, GRIMM, HUTTON_TRIUMPH];
const SRC_BELTANE: &[&str] = &[HUTTON_STATIONS, DANAHER];
const SRC_LITHA: &[&str] = &[BEDE, HUTTON_STATIONS, HUTTON_TRIUMPH];
const SRC_LUGHNASADH: &[&str] = &[MACNEILL, HUTTON_STATIONS];
const SRC_MABON: &[&str] = &[HUTTON_TRIUMPH, HUTTON_STATIONS];

pub const WHEEL_SOURCE: &str = HUTTON_TRIUMPH;

pub fn sabbat(s: Sabbat, lang: Lang) -> &'static SabbatLore {
    let table: &[SabbatLore; 8] = match lang {
        Lang::En => &EN,
        Lang::PtBr => &PT,
    };
    &table[s.index()]
}

static EN: [SabbatLore; 8] = [
    SabbatLore {
        epithet: "Beginning of winter · the ancestors",
        history: "Samhain is the Gaelic festival that opened winter and the dark half of the year. \
Medieval Irish literature presents it as a time of royal assemblies and as a night when the \
Otherworld lay open. Later folk custom in Ireland and Scotland included bonfires, divination, \
guising and carved turnip lanterns.",
        significance: "In modern Paganism Samhain honours the dead and the ancestors, and many \
traditions treat it as the close or the start of the ritual year.",
        symbolism: "Ancestors, thresholds, the last of the harvest, apples, lanterns, the dark half of the year.",
        sources: SRC_SAMHAIN,
    },
    SabbatLore {
        epithet: "Winter solstice · return of the sun",
        history: "Yule (Old English geola, Old Norse jól) was a Germanic midwinter season. Bede \
records the Anglo-Saxon months Giuli around the solstice and a festival called Modranecht \
(\"mothers' night\"). Many of its customs merged into Christmas; the Yule log and evergreen \
decoration are documented in early modern Britain.",
        significance: "The longest night: modern Pagans celebrate the rebirth of the sun as the \
days begin to lengthen again.",
        symbolism: "Evergreens, holly and ivy, candles and fire, the returning sun.",
        sources: SRC_YULE,
    },
    SabbatLore {
        epithet: "First stirrings of spring · Brigid",
        history: "Imbolc is the Gaelic festival of the beginning of spring, sharing its date with \
the feast of St Brigid. The name's origin is uncertain; a medieval gloss links it to ewes' \
milk. Irish folk custom included weaving Brigid's crosses from rushes and welcoming Brigid \
into the home.",
        significance: "A festival of returning light, purification and new beginnings as the \
lambing season starts.",
        symbolism: "Brigid, candles and hearth fire, milk, rushes, early flowers.",
        sources: SRC_IMBOLC,
    },
    SabbatLore {
        epithet: "Spring equinox · balance and growth",
        history: "The name comes from Jacob Grimm's reconstruction (1835) of a goddess from \
Bede's note that the Anglo-Saxon month Eosturmonath was named after Eostre. Bede is the only \
early source for Eostre, and scholars still debate her. The name was attached to the spring \
equinox in the modern Wheel of the Year in the 1970s.",
        significance: "Day and night stand equal; modern Pagans celebrate balance, fertility and \
the quickening of growth.",
        symbolism: "Balance of light and dark, eggs, hares, seeds and spring flowers.",
        sources: SRC_OSTARA,
    },
    SabbatLore {
        epithet: "Beginning of summer · fire and blossom",
        history: "Beltane is the Gaelic May Day festival that opened summer. The early medieval \
Irish glossary attributed to Cormac describes two fires between which cattle were driven for \
protection. Folk custom in Ireland and Scotland included bonfires, May boughs and protective \
charms. Modern celebration often also borrows English May Day customs such as the maypole.",
        significance: "A festival of fertility, vitality and the full flowering of the land.",
        symbolism: "Fire, blossoms, hawthorn, the maypole, green branches, union.",
        sources: SRC_BELTANE,
    },
    SabbatLore {
        epithet: "Summer solstice · the sun at its height",
        history: "The name comes from Bede, who gives Litha as the Anglo-Saxon name for the months \
around midsummer. It was adopted for the solstice festival in the modern Wheel of the Year in \
the 1970s. Midsummer, especially St John's Eve, has been marked with bonfires across Europe \
since the Middle Ages.",
        significance: "The longest day: the sun at the peak of its strength, after which the light \
begins to wane.",
        symbolism: "The sun, bonfires, oak, St John's wort and other summer herbs.",
        sources: SRC_LITHA,
    },
    SabbatLore {
        epithet: "First harvest · Lugh and the grain",
        history: "Lughnasadh is the Gaelic festival of the start of the harvest, named after the \
god Lugh. Medieval Irish tradition says he founded it in memory of his foster-mother Tailtiu. \
Máire MacNeill documented its survival in Irish hilltop gatherings and first-fruits customs. \
In England the same day was Lammas (\"loaf-mass\"), when bread from the new wheat was blessed.",
        significance: "Thanksgiving for the first fruits of the harvest, and an acknowledgement of \
the sacrifice that harvest implies.",
        symbolism: "Grain and bread, first fruits, bilberries, games and gatherings.",
        sources: SRC_LUGHNASADH,
    },
    SabbatLore {
        epithet: "Autumn equinox · the second harvest",
        history: "There is no historical evidence of a festival called Mabon. Aidan Kelly gave \
the autumn equinox this name in the 1970s, after Mabon ap Modron, a figure in medieval Welsh \
literature. Traditional British harvest-home feasts followed the end of the harvest rather \
than the equinox, and Michaelmas fell on 29 September.",
        significance: "Day and night stand equal once more; modern Pagans give thanks for the \
harvest and prepare for the dark half of the year.",
        symbolism: "Balance, fruit and nuts, wine, the gathered harvest.",
        sources: SRC_MABON,
    },
];

static PT: [SabbatLore; 8] = [
    SabbatLore {
        epithet: "Início do inverno · os ancestrais",
        history: "Samhain é o festival gaélico que abria o inverno e a metade escura do ano. A \
literatura irlandesa medieval o apresenta como época de assembleias reais e como uma noite em \
que o Outro Mundo ficava aberto. Mais tarde, o costume popular na Irlanda e na Escócia incluía \
fogueiras, adivinhações, disfarces e lanternas esculpidas em nabos.",
        significance: "No paganismo moderno, Samhain honra os mortos e os ancestrais, e muitas \
tradições o tratam como o fim ou o começo do ano ritual.",
        symbolism: "Ancestrais, limiares, o fim da colheita, maçãs, lanternas, a metade escura do ano.",
        sources: SRC_SAMHAIN,
    },
    SabbatLore {
        epithet: "Solstício de inverno · o retorno do sol",
        history: "Yule (inglês antigo geola, nórdico antigo jól) era uma temporada germânica de \
meio do inverno. Beda registra os meses anglo-saxões Giuli em torno do solstício e um festival \
chamado Modranecht (\"noite das mães\"). Muitos de seus costumes se fundiram ao Natal; o tronco \
de Yule e a decoração com plantas perenes estão documentados na Grã-Bretanha do início da era \
moderna.",
        significance: "A noite mais longa: os pagãos modernos celebram o renascimento do sol, \
quando os dias voltam a crescer.",
        symbolism: "Plantas perenes, azevinho e hera, velas e fogo, o sol que retorna.",
        sources: SRC_YULE,
    },
    SabbatLore {
        epithet: "Primeiros sinais da primavera · Brígida",
        history: "Imbolc é o festival gaélico do início da primavera e coincide com a festa de \
Santa Brígida. A origem do nome é incerta; uma glosa medieval o associa ao leite das ovelhas. \
O costume popular irlandês incluía tecer cruzes de Brígida com juncos e receber Brígida em casa.",
        significance: "Um festival da luz que retorna, da purificação e de novos começos, quando \
começa a época do nascimento dos cordeiros.",
        symbolism: "Brígida, velas e o fogo do lar, leite, juncos, as primeiras flores.",
        sources: SRC_IMBOLC,
    },
    SabbatLore {
        epithet: "Equinócio de primavera · equilíbrio e crescimento",
        history: "O nome vem da reconstrução que Jacob Grimm fez (1835) de uma deusa, a partir da \
nota de Beda de que o mês anglo-saxão Eosturmonath recebeu o nome de Eostre. Beda é a única fonte \
antiga sobre Eostre, e os estudiosos ainda a debatem. O nome foi associado ao equinócio de \
primavera na Roda do Ano moderna nos anos 1970.",
        significance: "Dia e noite se igualam; os pagãos modernos celebram o equilíbrio, a \
fertilidade e o despertar do crescimento.",
        symbolism: "Equilíbrio entre luz e escuridão, ovos, lebres, sementes e flores de primavera.",
        sources: SRC_OSTARA,
    },
    SabbatLore {
        epithet: "Início do verão · fogo e flores",
        history: "Beltane é o festival gaélico do primeiro de maio, que abria o verão. O glossário \
irlandês do início da Idade Média atribuído a Cormac descreve duas fogueiras entre as quais o \
gado era conduzido para proteção. O costume popular na Irlanda e na Escócia incluía fogueiras, \
ramos de maio e amuletos de proteção. A celebração moderna muitas vezes também adota costumes \
ingleses do Dia de Maio, como o mastro de fitas.",
        significance: "Um festival de fertilidade, vitalidade e do pleno florescer da terra.",
        symbolism: "Fogo, flores, espinheiro, o mastro de fitas, ramos verdes, união.",
        sources: SRC_BELTANE,
    },
    SabbatLore {
        epithet: "Solstício de verão · o sol no auge",
        history: "O nome vem de Beda, que dá Litha como o nome anglo-saxão dos meses em torno do \
meio do verão. Ele foi adotado para o festival do solstício na Roda do Ano moderna nos anos \
1970. O meio do verão, especialmente a véspera de São João, é marcado com fogueiras por toda a \
Europa desde a Idade Média.",
        significance: "O dia mais longo: o sol no auge de sua força, depois do qual a luz começa \
a minguar.",
        symbolism: "O sol, fogueiras, carvalho, erva-de-são-joão e outras ervas de verão.",
        sources: SRC_LITHA,
    },
    SabbatLore {
        epithet: "Primeira colheita · Lugh e os grãos",
        history: "Lughnasadh é o festival gaélico do início da colheita e leva o nome do deus \
Lugh. A tradição irlandesa medieval diz que ele o fundou em memória de sua mãe adotiva, Tailtiu. \
Máire MacNeill documentou sua permanência em reuniões no alto de colinas e em costumes de \
primeiros frutos na Irlanda. Na Inglaterra, o mesmo dia era Lammas (\"missa do pão\"), quando \
se abençoava o pão feito com o trigo novo.",
        significance: "Agradecimento pelos primeiros frutos da colheita e reconhecimento do \
sacrifício que toda colheita implica.",
        symbolism: "Grãos e pão, primeiros frutos, mirtilos, jogos e reuniões.",
        sources: SRC_LUGHNASADH,
    },
    SabbatLore {
        epithet: "Equinócio de outono · a segunda colheita",
        history: "Não há evidência histórica de um festival chamado Mabon. Aidan Kelly deu esse \
nome ao equinócio de outono nos anos 1970, em referência a Mabon ap Modron, personagem da \
literatura galesa medieval. As festas tradicionais britânicas de fim de colheita aconteciam ao \
término da colheita, e não no equinócio; o dia de São Miguel caía em 29 de setembro.",
        significance: "Dia e noite se igualam mais uma vez; os pagãos modernos agradecem pela \
colheita e se preparam para a metade escura do ano.",
        symbolism: "Equilíbrio, frutas e nozes, vinho, a colheita recolhida.",
        sources: SRC_MABON,
    },
];

/// Note about the Wheel of the Year itself.
pub fn wheel_note(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "The eightfold Wheel of the Year is a modern synthesis. It took shape in the \
1950s in British Wicca and Druidry and combines the four Gaelic seasonal festivals with the \
solstices and equinoxes. The names Ostara, Litha and Mabon were popularised in the 1970s.",
        Lang::PtBr => "A Roda do Ano com oito festivais é uma síntese moderna. Ela tomou forma nos \
anos 1950, na Wicca e no Druidismo britânicos, e combina os quatro festivais sazonais gaélicos \
com os solstícios e equinócios. Os nomes Ostara, Litha e Mabon se popularizaram nos anos 1970.",
    }
}

/// Note on fixed versus astronomical cross-quarter dates.
pub fn cross_quarter_note(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "The Gaelic festivals are traditionally held on fixed days, celebrated from \
the eve before. Some practitioners use the astronomical midpoint instead, when the Sun is \
halfway between a solstice and an equinox; this usually falls several days later.",
        Lang::PtBr => "Os festivais gaélicos são tradicionalmente celebrados em dias fixos, a \
partir da véspera. Alguns praticantes usam o ponto médio astronômico, quando o Sol está a meio \
caminho entre um solstício e um equinócio; ele costuma cair alguns dias depois.",
    }
}

/// Short label for the period following each festival. The Wheel follows
/// the Gaelic convention in which the cross-quarter days open the seasons.
pub fn period_title(s: Sabbat, lang: Lang) -> &'static str {
    const EN: [&str; 8] = [
        "Early winter", "Late winter", "Early spring", "Late spring", "Early summer", "Late summer",
        "Early autumn", "Late autumn",
    ];
    const PT: [&str; 8] = [
        "Início do inverno", "Fim do inverno", "Início da primavera", "Fim da primavera", "Início do verão",
        "Fim do verão", "Início do outono", "Fim do outono",
    ];
    match lang {
        Lang::En => EN[s.index()],
        Lang::PtBr => PT[s.index()],
    }
}

/// Description of the interval that follows each festival.
pub fn period_after(s: Sabbat, lang: Lang) -> &'static str {
    const EN: [&str; 8] = [
        "The dark half of the year deepens. Nights lengthen toward midwinter and the land rests \
after the harvest.",
        "The light returns. Days lengthen slowly while the cold often deepens, and the land waits \
for the first signs of spring.",
        "Early spring. Life quickens beneath the surface and the days lengthen quickly toward the \
equinox.",
        "Light outweighs dark. Growth speeds up through the second half of spring, toward the \
opening of summer.",
        "The bright half of the year. Late spring ripens into summer as the days keep lengthening \
toward the solstice.",
        "The light begins to wane after its peak, while the heat of summer often still builds and \
the crops ripen.",
        "Harvest time. The grain is gathered and the days shorten steadily toward the autumn \
equinox.",
        "Dark outweighs light. Autumn deepens, the last fruits are gathered and the year turns \
toward its dark half.",
    ];
    const PT: [&str; 8] = [
        "A metade escura do ano se aprofunda. As noites crescem rumo ao meio do inverno e a terra \
descansa depois da colheita.",
        "A luz retorna. Os dias crescem devagar enquanto o frio muitas vezes se intensifica, e a \
terra aguarda os primeiros sinais da primavera.",
        "Início da primavera. A vida desperta sob a superfície e os dias crescem depressa rumo ao \
equinócio.",
        "A luz supera a escuridão. O crescimento se acelera na segunda metade da primavera, rumo \
ao início do verão.",
        "A metade clara do ano. O fim da primavera amadurece em verão enquanto os dias seguem \
crescendo rumo ao solstício.",
        "A luz começa a minguar depois do auge, enquanto o calor do verão muitas vezes ainda \
aumenta e as plantações amadurecem.",
        "Tempo de colheita. Os grãos são recolhidos e os dias encurtam aos poucos rumo ao \
equinócio de outono.",
        "A escuridão supera a luz. O outono se aprofunda, os últimos frutos são colhidos e o ano \
se volta para sua metade escura.",
    ];
    match lang {
        Lang::En => EN[s.index()],
        Lang::PtBr => PT[s.index()],
    }
}

pub fn season_description(s: Season, lang: Lang) -> &'static str {
    match (lang, s) {
        (Lang::En, Season::Spring) => "Astronomical spring runs from the equinox, when day and \
night are nearly equal, to the summer solstice. Day length grows fastest around the equinox.",
        (Lang::En, Season::Summer) => "Astronomical summer runs from the solstice, the longest day, \
to the autumn equinox. Days shorten through the season, but because of seasonal lag the warmest \
weeks usually come after the solstice.",
        (Lang::En, Season::Autumn) => "Astronomical autumn runs from the equinox to the winter \
solstice. Nights lengthen fastest around the equinox and the sun's noon altitude falls week by \
week.",
        (Lang::En, Season::Winter) => "Astronomical winter runs from the solstice, the longest \
night, to the spring equinox. Days lengthen again from the start, though the coldest weeks \
usually come later.",
        (Lang::PtBr, Season::Spring) => "A primavera astronômica vai do equinócio, quando dia e \
noite têm quase a mesma duração, até o solstício de verão. A duração do dia cresce mais rápido \
perto do equinócio.",
        (Lang::PtBr, Season::Summer) => "O verão astronômico vai do solstício, o dia mais longo, \
até o equinócio de outono. Os dias encurtam ao longo da estação, mas, por causa do atraso \
sazonal, as semanas mais quentes costumam vir depois do solstício.",
        (Lang::PtBr, Season::Autumn) => "O outono astronômico vai do equinócio até o solstício de \
inverno. As noites crescem mais rápido perto do equinócio e a altura do sol ao meio-dia diminui \
semana a semana.",
        (Lang::PtBr, Season::Winter) => "O inverno astronômico vai do solstício, a noite mais \
longa, até o equinócio de primavera. Os dias voltam a crescer desde o início, mas as semanas \
mais frias costumam vir depois.",
    }
}

/// Meteorological seasons use whole calendar months instead.
pub fn season_note(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Meteorological seasons use whole calendar months instead; the dates shown \
here are the astronomical ones.",
        Lang::PtBr => "As estações meteorológicas usam meses inteiros; as datas mostradas aqui são \
as astronômicas.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_every_text() {
        for lang in [Lang::En, Lang::PtBr] {
            for s in Sabbat::ALL {
                let l = sabbat(s, lang);
                for t in [l.epithet, l.history, l.significance, l.symbolism, period_after(s, lang), period_title(s, lang)] {
                    assert!(!t.trim().is_empty(), "{lang:?} {s:?}");
                }
                assert!(!l.sources.is_empty());
                // Translations cite exactly the same works.
                assert_eq!(l.sources, sabbat(s, Lang::En).sources);
            }
        }
    }

    #[test]
    fn portuguese_is_not_english() {
        for s in Sabbat::ALL {
            assert_ne!(sabbat(s, Lang::PtBr).history, sabbat(s, Lang::En).history);
        }
    }
}
