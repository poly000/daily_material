use log::info;
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult::{self, Article},
        InlineQueryResultArticle, InputMessageContent, InputMessageContentText, ParseMode,
    },
};

use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike, Weekday};
use daily_material::{talent, weapon, SPLITTER};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let bot = Bot::new(unsafe {
        std::str::from_utf8_unchecked(&std::fs::read("bot_token").unwrap_unchecked())
    })
    .auto_send();
    let handler = Update::filter_inline_query().branch(dptree::endpoint(
        |query: InlineQuery, bot: AutoSend<Bot>| async move {
            let result = get_materials();
            let response = bot
                .answer_inline_query(&query.id, result)
                .cache_time(0)
                .send()
                .await;

            if let Err(e) = response {
                log::error!("Error in handler: {:?}", e);
            }

            respond(())
        },
    ));

    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    info!("Bot started successfully");
}

fn talent(time: Weekday) -> Option<&'static [&'static str]> {
    match time {
        Weekday::Mon | Weekday::Thu => Some(&talent::MON_THU),
        Weekday::Tue | Weekday::Fri => Some(&talent::TUE_FRI),
        Weekday::Wed | Weekday::Sat => Some(&talent::WED_SAT),
        _ => None,
    }
}

fn weapon(time: Weekday) -> Option<&'static [&'static str]> {
    match time {
        Weekday::Mon | Weekday::Thu => Some(&weapon::MON_THU),
        Weekday::Tue | Weekday::Fri => Some(&weapon::TUE_FRI),
        Weekday::Wed | Weekday::Sat => Some(&weapon::WED_SAT),
        _ => None,
    }
}

fn utc_plus_4() -> DateTime<FixedOffset> {
    let utc = &chrono::Utc::now().naive_utc();
    FixedOffset::east(4 * 3600).from_utc_datetime(utc)
}

fn rest_of_a_day(time: DateTime<FixedOffset>) -> InlineQueryResult {
    let (rsec, rmin, rhour) = match (time.second(), time.minute(), time.hour()) {
        (0, 0, hour) => (0, 0, 24 - hour),
        (0, min, hour) => (0, 60 - min, 23 - hour),
        (sec, min, hour) => (60 - sec, 59 - min, 23 - hour),
    };

    let info = format!("?????? {rhour:02}:{rmin:02}:{rsec:02} ????????????");

    let content = InputMessageContent::Text(InputMessageContentText::new(&info));
    Article(InlineQueryResultArticle::new("rest", info, content))
}

#[inline(always)]
fn get_materials() -> Vec<InlineQueryResult> {
    let datetime = utc_plus_4();
    let weekday = datetime.weekday();
    let talent_list = talent(weekday);
    let weapon_list = weapon(weekday);

    if let (Some(talent_list), Some(weapon_list)) = (talent_list, weapon_list) {
        let talents = format!("?????????{}", talent_list.join(SPLITTER));
        let weapons = format!("?????????{}", weapon_list.join(SPLITTER));
        let content_talent = InputMessageContent::Text(
            InputMessageContentText::new(
                talents.clone() + "\n?????? [????????????](https://genshin.pub/daily)",
            )
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true),
        );
        let content_weapon = InputMessageContent::Text(
            InputMessageContentText::new(
                weapons.clone() + "\n?????? [????????????](https://genshin.pub/daily)",
            )
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true),
        );

        let talent_text = Article(InlineQueryResultArticle::new(
            "??????",
            talents,
            content_talent,
        ));
        let weapon_text = Article(InlineQueryResultArticle::new(
            "??????",
            weapons,
            content_weapon,
        ));

        vec![talent_text, weapon_text, rest_of_a_day(datetime)]
    } else {
        vec![
            Article(InlineQueryResultArticle::new(
                "_",
                "?????????????????????",
                InputMessageContent::Text(InputMessageContentText::new("????????????????????????????????????")),
            )),
            rest_of_a_day(datetime),
        ]
    }
}
