import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import express from 'express';
import cors from 'cors';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { scrapeGadgetNews } from './scraper.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const INTERESTS_PATH = path.join(__dirname, '..', 'gadget-interests.json');

// --- 1. MCP Server Setup ---
const mcpServer = new Server({
    name: "gadget-concierge-mcp",
    version: "1.0.0",
}, {
    capabilities: {
        tools: {},
    },
});

mcpServer.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            {
                name: "get_gadget_dashboard",
                description: "Scrape and return latest gadget news JSON based on user interests.",
                inputSchema: { type: "object", properties: {} }
            },
            {
                name: "add_gadget_interest",
                description: "Add a new category, brand, or keyword to the user's interests.",
                inputSchema: {
                    type: "object",
                    properties: {
                        type: { type: "string", enum: ["category", "keyword", "brand"] },
                        value: { type: "string" },
                        name: { type: "string", description: "Category name for keywords/brands" }
                    },
                    required: ["type", "value"]
                }
            }
        ],
    };
});

mcpServer.setRequestHandler(CallToolRequestSchema, async (request) => {
    const interests = JSON.parse(fs.readFileSync(INTERESTS_PATH, 'utf8'));
    
    if (request.params.name === "get_gadget_dashboard") {
        const data = await scrapeGadgetNews(interests);
        return { content: [{ type: "text", text: JSON.stringify(data, null, 2) }] };
    }
    
    if (request.params.name === "add_gadget_interest") {
        const { type, value, name } = request.params.arguments;
        updateInterestsFile(type, value, name);
        return { content: [{ type: "text", text: `Successfully added ${value} to ${type}.` }] };
    }

    throw new Error("Tool not found");
});

// --- 2. HTTP API Server Setup (for HTML Dashboard) ---
const app = express();
app.use(cors());
app.use(express.json());

app.get('/dashboard', async (req, res) => {
    try {
        const interests = JSON.parse(fs.readFileSync(INTERESTS_PATH, 'utf8'));
        const data = await scrapeGadgetNews(interests);
        res.json(data);
    } catch (e) {
        res.status(500).json({ error: e.message });
    }
});

app.post('/update', (req, res) => {
    try {
        const { type, value, name } = req.body;
        updateInterestsFile(type, value, name);
        res.json({ status: 'success' });
    } catch (e) {
        res.status(500).json({ error: e.message });
    }
});

function updateInterestsFile(type, value, name) {
    const data = JSON.parse(fs.readFileSync(INTERESTS_PATH, 'utf8'));
    if (type === 'category') {
        if (!data.categories[value]) data.categories[value] = { brands: [], keywords: [], score: 5 };
    } else if (type === 'keyword') {
        const target = name || 'ガジェット全般';
        if (!data.categories[target].keywords.includes(value)) data.categories[target].keywords.push(value);
    } else if (type === 'brand') {
        const target = name || 'ガジェット全般';
        if (!data.categories[target].brands.includes(value)) data.categories[target].brands.push(value);
    }
    fs.writeFileSync(INTERESTS_PATH, JSON.stringify(data, null, 2));
}

// --- 3. Execution ---
const transport = new StdioServerTransport();
mcpServer.connect(transport);

const HTTP_PORT = 3005;
app.listen(HTTP_PORT, () => {
    console.error(`HTTP API for Dashboard running on port ${HTTP_PORT}`);
});