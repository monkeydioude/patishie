use crate::entities::{potential_articles::PotentialArticle, rss::Rss};
use chrono::Utc;
use serde_xml_rs::from_str;
use uuid::Uuid;

pub async fn get_cookies_from_rss(
    channel_url: &str,
    channel_id: i32,
    uuid: Uuid,
) -> Option<Vec<PotentialArticle>> {
    let url = channel_url;
    let response = reqwest::get(url)
        .await
        .map_err(|err| eprintln!("[{}] ({}) {}", uuid, Utc::now(), err))
        .ok()?;
    let raw_data = response
        .text()
        .await
        .map_err(|err| eprintln!("[{}] ({}) {}", uuid, Utc::now(), err))
        .ok()?;
    let rss: Rss = from_str(&raw_data)
        .map_err(|err| eprintln!("[{}] ({}) {}", uuid, Utc::now(), err))
        .ok()?;
    let mut res: Vec<PotentialArticle> = vec![];
    rss.channel.item.iter().for_each(|item| {
        res.push(PotentialArticle {
            link: item.link.clone().unwrap_or_default(),
            img: item.get_img(),
            title: item.get_title(),
            desc: item.get_desc(),
            create_date: item.get_create_date(),
            channel_name: Some(rss.channel.get_channel_name(url)),
            channel_id: Some(channel_id),
        })
    });
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_can_deserialize_rss() {
        let rss: Rss = from_str(TEST_1).unwrap();
        assert_eq!(
            rss.channel.item.first().unwrap().content.as_ref().unwrap().url,
            "https://img.lemde.fr/2024/07/17/441/0/4000/2000/644/322/60/0/8e5c70e_1721206405128-000-1wb54n.jpg"
        );
    }
    const TEST_1: &str = r#"
    <rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:media="http://search.yahoo.com/mrss/" xmlns:content="http://purl.org/rss/1.0/modules/content/" version="2.0">
        <channel>
            <title>Sciences : Toute l’actualité sur Le Monde.fr.</title>
            <description>Sciences - Découvrez gratuitement tous les articles, les vidéos et les infographies de la rubrique Sciences sur Le Monde.fr.</description>
            <copyright>Le Monde - L’utilisation des flux RSS du Monde.fr est réservée à un usage strictement personnel, non professionnel et non collectif. Toute autre exploitation doit faire l’objet d’une autorisation et donner lieu au versement d’une rémunération. Contact : syndication@lemonde.fr</copyright>
            <link>https://www.lemonde.fr/sciences/rss_full.xml</link>
            <pubDate>Sat, 20 Jul 2024 14:26:03 +0000</pubDate>
            <language>fr</language>
            <atom:link href="https://www.lemonde.fr/sciences/rss_full.xml" rel="self" type="application/rss+xml"/>
            <item>
                <title>
                    <![CDATA[ Découverte d’une hormone qui contribue à la solidité des os, même pendant l’allaitement ]]>
                </title>
                <pubDate>Thu, 18 Jul 2024 05:45:05 +0200</pubDate>
                <updated>Thu, 18 Jul 2024 16:57:53 +0200</updated>
                <description>
                    <![CDATA[ Une équipe américaine a débusqué une protéine cérébrale qui permet aux femelles souris de récupérer le tissu osseux qu’elles ont perdu après la gestation et l’allaitement, au profit de la minéralisation du squelette du fœtus et de la production de lait. ]]>
                </description>
                <guid isPermaLink="true">https://www.lemonde.fr/sciences/article/2024/07/18/decouverte-d-une-hormone-qui-contribue-a-la-solidite-des-os-meme-pendant-l-allaitement_6251930_1650684.html</guid>
                <link>https://www.lemonde.fr/sciences/article/2024/07/18/decouverte-d-une-hormone-qui-contribue-a-la-solidite-des-os-meme-pendant-l-allaitement_6251930_1650684.html</link>
                <media:content width="644" height="322" url="https://img.lemde.fr/2024/07/17/441/0/4000/2000/644/322/60/0/8e5c70e_1721206405128-000-1wb54n.jpg">
                    <media:description type="plain">En Chine, en août 2020.</media:description>
                    <media:credit scheme="urn:ebu">AFP</media:credit>
                </media:content>
            </item>
        </channel>
    </rss>"#;
}
