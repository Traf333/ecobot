use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, ParseMode},
    Bot,
};

use crate::db;

pub struct LocationCommand;

impl LocationCommand {
    pub async fn handle(
        bot: &Bot,
        chat_id: ChatId,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Location received: {} {}", latitude, longitude);
        let bin_locations = db::get_bin_locations(latitude, longitude).await?;

        let mut content = "".to_string();

        if bin_locations.is_empty() {
            content =
                "<b>3- и 4-секционные контейнеры РСО в радиусе 1 км не найдены.</b>".to_string();
            content.push_str("\n👉 Проверить самостоятельно <a href=\"https://new.esoo39.ru/rso-maps/\">на сайте обслуживающей компании ЕСОО</a>");
        } else {
            content = "<b>Ближайшие 3- и 4-секционные контейнеры РСО:</b>".to_string();
            for (distance, bin_location) in bin_locations.into_iter().take(2) {
                let link_url = format!(
                    "https://yandex.ru/maps/?rtext={},{}~{},{}&rtt=pedestrian",
                    latitude, longitude, bin_location.latitude, bin_location.longitude
                );
                let glass_text = if bin_location.preset == "islands#darkgreenIcon" {
                    "со стеклом"
                } else {
                    "без стекла"
                };
                let bin_text = format!(
                    "\n{} м <a href=\"{}\">{}</a> {}",
                    (distance * 1000.0).round(),
                    link_url,
                    bin_location.address,
                    glass_text
                );
                content.push_str(&bin_text);
            }
            content.push_str("\n👉 Проверить самостоятельно <a href=\"https://new.esoo39.ru/rso-maps/\">на сайте обслуживающей компании ЕСОО</a>");
        }

        let main_point = db::main_point();
        let distance_to_main = (main_point.distance(latitude, longitude) * 1000.0).round();
        if distance_to_main < 1000.0 {
            content.push_str(
                &format!("\n\nПлощадка раздельного сбора с самым большим перечнем принимаемых фракций находится на <a href=\"https://yandex.ru/maps/?rtext={},{}~{},{}&amp;rtt=pedestrian\">ул. 5-я Причальная 2а</a> в радиусе {} м.", latitude, longitude, main_point.latitude, main_point.longitude, distance_to_main)
            );
        } else {
            content.push_str(
                &format!("\n\nПлощадка раздельного сбора с самым большим перечнем принимаемых фракций находится на <a href=\"https://yandex.ru/maps/?text={},{}\">ул. 5-я Причальная 2а</a>.", main_point.latitude, main_point.longitude)
            );
        }

        content.push_str(
            "\n\nОтправьте новую геопозицию, если хотите найти другие контейнеры.\nОтправьте «Бот», если хотите вернуться в начало.",
        );

        bot.send_message(chat_id, content)
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}
