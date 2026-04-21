import axios from 'axios';
import * as cheerio from 'cheerio';
import Parser from 'rss-parser';

const parser = new Parser();

export async function scrapeGadgetNews(interests) {
    const news = {
        "AI・開発関連": [],
        "PC関連": [],
        "音楽関連": [],
        "ガジェット全般": []
    };

    const targets = [
        { url: 'https://pc.watch.impress.co.jp/data/rss/pcw/index.rdf', cat: 'PC関連' },
        { url: 'https://av.watch.impress.co.jp/data/rss/avw/index.rdf', cat: '音楽関連' },
        { url: 'https://rss.itmedia.co.jp/rss/2.0/pcuser.xml', cat: 'PC関連' },
        { url: 'https://www.gizmodo.jp/index.xml', cat: 'ガジェット全般' }
    ];

    for (const target of targets) {
        try {
            const feed = await parser.parseURL(target.url);
            for (const item of feed.items) {
                let score = 5;
                const text = (item.title + item.contentSnippet).toLowerCase();
                
                // 興味度スコアの計算
                for (const catName in interests.categories) {
                    const cat = interests.categories[catName];
                    cat.brands.forEach(b => { if (text.includes(b.toLowerCase())) score += 5; });
                    cat.keywords.forEach(k => { if (text.includes(k.toLowerCase())) score += 3; });
                }
                for (const k in interests.learned_keywords) {
                    if (text.includes(k.toLowerCase())) score += interests.learned_keywords[k].score;
                }

                const article = {
                    title: item.title,
                    link: item.link,
                    desc: item.contentSnippet?.slice(0, 100) + '...',
                    brand: extractBrand(item.title, interests),
                    score: score,
                    img: extractImage(item) || "https://images.unsplash.com/photo-1550745165-9bc0b252726f?w=400"
                };

                // カテゴリ分け（簡易的）
                if (text.includes('ai') || text.includes('llm') || text.includes('gpt')) {
                    news["AI・開発関連"].push(article);
                } else if (news[target.cat]) {
                    news[target.cat].push(article);
                } else {
                    news["ガジェット全般"].push(article);
                }
            }
        } catch (e) {
            console.error(`Error scraping ${target.url}:`, e.message);
        }
    }

    // スコア順にソートして各10件に制限
    for (const cat in news) {
        news[cat] = news[cat].sort((a, b) => b.score - a.score).slice(0, 10);
    }

    return news;
}

function extractBrand(title, interests) {
    for (const catName in interests.categories) {
        for (const brand of interests.categories[catName].brands) {
            if (title.toLowerCase().includes(brand.toLowerCase())) return brand;
        }
    }
    return "News";
}

function extractImage(item) {
    // RSSから画像を抽出する簡易的なロジック（メディアによって異なる）
    if (item.enclosure && item.enclosure.url) return item.enclosure.url;
    if (item.content) {
        const $ = cheerio.load(item.content);
        return $('img').attr('src');
    }
    return null;
}