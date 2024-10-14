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
            categories: item.get_categories(),
            desc: item.get_desc(),
            create_date: item.get_create_date(Utc::now()),
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

    #[test]
    fn test_i_can_deserialize_rss_with_categories() {
        let rss: Rss = from_str(TEST_2).unwrap();
        assert_eq!(
            vec![
                "Biotech & Health",
                "Neurostimulation",
                "Neurotech",
                "neurotech medtech"
            ],
            rss.channel.item.first().unwrap().category.clone().unwrap(),
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

    const TEST_2: &str = r#"
    <rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"
    xmlns:wfw="http://wellformedweb.org/CommentAPI/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:atom="http://www.w3.org/2005/Atom"
	xmlns:sy="http://purl.org/rss/1.0/modules/syndication/"
	xmlns:slash="http://purl.org/rss/1.0/modules/slash/"
	xmlns:georss="http://www.georss.org/georss"
	xmlns:geo="http://www.w3.org/2003/01/geo/wgs84_pos#">
    <channel>
        <title>TechCrunch</title>
        <atom:link href="https://techcrunch.com/feed/" rel="self" type="application/rss+xml" />
        <link>https://techcrunch.com/</link>
        <description>Startup and Technology News</description>
        <lastBuildDate>Sat, 12 Oct 2024 07:12:32 +0000</lastBuildDate>
        <language>en-US</language>
        <sy:updatePeriod>hourly</sy:updatePeriod>
        <sy:updateFrequency>1</sy:updateFrequency>
        <generator>https://wordpress.org/?v=6.6.2</generator>
        <image>
            <url>https://techcrunch.com/wp-content/uploads/2015/02/cropped-cropped-favicon-gradient.png?w=32</url>
            <title>TechCrunch</title>
            <link>https://techcrunch.com/</link>
            <width>32</width>
            <height>32</height>
        </image>
        <item>
            <title>What is wearable neurotech and why might we need it?</title>
            <link>https://techcrunch.com/2024/10/12/what-is-wearable-neurotech-and-why-might-we-need-it/</link>
            <dc:creator><![CDATA[Natasha Lomas]]></dc:creator>
            <pubDate>Sat, 12 Oct 2024 12:00:00 +0000</pubDate>
                    <category><![CDATA[Biotech & Health]]></category>
            <category><![CDATA[Neurostimulation]]></category>
            <category><![CDATA[Neurotech]]></category>
            <category><![CDATA[neurotech medtech]]></category>
            <guid isPermaLink="false">https://techcrunch.com/?p=2846025</guid>
            <description><![CDATA[<p>The wearables category already contains multitudes, from exercise-focused smart watches and sleep tracking smart rings to smart femtech and semi-invasive blood glucose monitors &#8212; to name a few of the gizmos we&#8217;ve tracked over roughly a decade of novel personal hardware launches. But the space is set to get even more active, with a new [&#8230;]</p>
            <p>© 2024 TechCrunch. All rights reserved. For personal use only.</p>
    ]]></description>
        </item>
	</channel>
</rss>"#;
}
