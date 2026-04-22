import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import App from "../App";
import { invoke } from "@tauri-apps/api/core";

test("App renders Mixer title", async () => {
  // get_audio_sessions が呼ばれた際に空配列を返すように設定
  (invoke as any).mockResolvedValue([]);

  render(<App />);
  const titleElement = screen.getByText(/Mixer/i);
  expect(titleElement).toBeInTheDocument();
});

test("Shows empty state when no sessions found", async () => {
  (invoke as any).mockResolvedValue([]);

  render(<App />);
  const emptyMsg = await screen.findByText(/No active audio streams found/i);
  expect(emptyMsg).toBeInTheDocument();
});
