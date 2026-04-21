const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 3005;
const JSON_PATH = path.join(__dirname, '..', 'gadget-interests.json');

const server = http.createServer((req, res) => {
    // CORS設定
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'POST, GET, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') {
        res.writeHead(204);
        res.end();
        return;
    }

    if (req.method === 'POST' && req.url === '/update') {
        let body = '';
        req.on('data', chunk => { body += chunk.toString(); });
        req.on('end', () => {
            try {
                const { type, name, value } = JSON.parse(body);
                const data = JSON.parse(fs.readFileSync(JSON_PATH, 'utf8'));

                if (type === 'category') {
                    if (!data.categories[name]) {
                        data.categories[name] = { brands: [], keywords: [], score: 5 };
                    }
                } else if (type === 'keyword') {
                    // 指定されたカテゴリにキーワードを追加、カテゴリ指定がなければ「未分類」へ
                    const targetCat = name || 'ガジェット全般';
                    if (!data.categories[targetCat]) {
                        data.categories[targetCat] = { brands: [], keywords: [], score: 5 };
                    }
                    if (!data.categories[targetCat].keywords.includes(value)) {
                        data.categories[targetCat].keywords.push(value);
                    }
                } else if (type === 'brand') {
                    const targetCat = name || 'ガジェット全般';
                    if (!data.categories[targetCat]) {
                        data.categories[targetCat] = { brands: [], keywords: [], score: 5 };
                    }
                    if (!data.categories[targetCat].brands.includes(value)) {
                        data.categories[targetCat].brands.push(value);
                    }
                }

                fs.writeFileSync(JSON_PATH, JSON.stringify(data, null, 2));
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ status: 'success', message: `Added ${value || name}` }));
            } catch (err) {
                res.writeHead(500);
                res.end(JSON.stringify({ status: 'error', message: err.message }));
            }
        });
    } else {
        res.writeHead(404);
        res.end();
    }
});

server.listen(PORT, () => {
    console.log(`Gadget Dashboard API running on http://localhost:${PORT}`);
});