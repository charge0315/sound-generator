import "@testing-library/jest-dom";
import { vi } from "vitest";

// Tauri の API をモック化します（テスト環境では実行できないため）
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    isVisible: vi.fn(() => Promise.resolve(false)),
    show: vi.fn(),
    hide: vi.fn(),
    setFocus: vi.fn(),
    currentMonitor: vi.fn(() => Promise.resolve(null)),
    onFocusedChanged: vi.fn(() => Promise.resolve(() => {})),
  }),
}));
