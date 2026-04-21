const fs = require('fs');
const path = require('path');

const filePath = path.join(__dirname, '..', 'gadget-interests.json');
const keyword = process.argv[2];

if (!keyword) {
    console.error('Usage: node update_interests.js <keyword>');
    process.exit(1);
}

try {
    const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    
    // すでに学習済みのキーワードか確認
    if (data.learned_keywords[keyword]) {
        data.learned_keywords[keyword].score += 2;
        data.learned_keywords[keyword].last_viewed = new Date().toISOString().split('T')[0];
    } else {
        // 新規追加
        data.learned_keywords[keyword] = {
            score: 2,
            last_viewed: new Date().toISOString().split('T')[0]
        };
    }

    // カテゴリ内のキーワードともマッチするか確認してスコアを微増
    for (const cat in data.categories) {
        if (data.categories[cat].keywords.includes(keyword) || data.categories[cat].brands.includes(keyword)) {
            data.categories[cat].score += 1;
        }
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));
    console.log(`Success: Learned "${keyword}" (Score incremented)`);
} catch (err) {
    console.error('Error updating JSON:', err);
    process.exit(1);
}