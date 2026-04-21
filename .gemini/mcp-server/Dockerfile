FROM node:20-alpine

WORKDIR /app

# 依存関係のコピーとインストール
COPY package.json ./
RUN npm install

# ソースコードのコピー
COPY index.js scraper.js ./

# ポート3005を公開 (HTTP API用)
EXPOSE 3005

# 実行
CMD ["node", "index.js"]