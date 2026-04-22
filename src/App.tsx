import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, Event } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface AudioEventPayload {
  process_id: number;
  volume: number | null;
  mute: boolean | null;
  state: string | null;
  icon_base64?: string | null;
}

interface AudioDevice {
  id: string;
  name: string;
}

function App() {
  const [sessions, setSessions] = useState<any[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [errorMsg, setErrorMsg] = useState("");
  const [isEntering, setIsEntering] = useState(false);
  const appWindow = getCurrentWindow();
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchSessions();
    fetchDevices();

    const unlistenTray = listen<any>("tray_click_left", async (event) => {
      const isVisible = await appWindow.isVisible();
      if (isVisible) {
        setIsEntering(false);
        // アニメーションが終わるのを少し待ってから非表示にする
        setTimeout(() => appWindow.hide(), 200);
      } else {
        const width = 360;
        const height = 500;
        
        // --- 高度な配置ロジック ---
        // クリックされたモニターの情報を取得
        // モニターの有効領域 (WorkArea) を考慮して、タスクバーの位置を推測する
        const monitor = await appWindow.currentMonitor();
        if (monitor) {
          const { x: clickX, y: clickY } = event.payload;
          const { size: mSize, position: mPos } = monitor;
          
          let targetX = clickX - (width / 2);
          let targetY = clickY - height - 8; // デフォルトは上（タスクバーが下にある場合）

          // 画面端の境界チェック（はみ出し防止）
          if (targetX < mPos.x) targetX = mPos.x + 8;
          if (targetX + width > mPos.x + mSize.width) targetX = mPos.x + mSize.width - width - 8;

          // タスクバーが上にある場合の判定（単純化のため y 座標で判断）
          if (clickY < mPos.y + 100) {
            targetY = clickY + 24; // 下に表示
          }

          await invoke("set_window_position", { x: Math.round(targetX), y: Math.round(targetY) });
        }

        setIsEntering(true);
        await appWindow.show();
        await appWindow.setFocus();
      }
    });

    const unlistenAudio = listen<AudioEventPayload>("audio-session-event", (event: Event<AudioEventPayload>) => {
      setSessions((prevSessions) => {
        return prevSessions.map((session) => {
          if (session.process_id === event.payload.process_id) {
            return {
              ...session,
              volume: event.payload.volume !== null ? event.payload.volume : session.volume,
              is_muted: event.payload.mute !== null ? event.payload.mute : session.is_muted,
            };
          }
          return session;
        });
      });
    });

    // ウィンドウがフォーカスを失ったら自動的に閉じる（Windows 11 標準フライアウト挙動）
    const unlistenBlur = appWindow.onFocusedChanged(({ focused }) => {
      if (!focused) {
        setIsEntering(false);
        setTimeout(() => appWindow.hide(), 200);
      }
    });

    return () => {
      unlistenTray.then(f => f());
      unlistenAudio.then(f => f());
      unlistenBlur.then(f => f());
    };
  }, []);

  async function fetchSessions() {
    try {
      const result = await invoke("get_audio_sessions");
      setSessions(result as any[]);
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  async function fetchDevices() {
    try {
      const result = await invoke("get_audio_devices");
      setDevices(result as AudioDevice[]);
    } catch (e: any) {
      console.error("Failed to fetch devices:", e);
    }
  }

  async function setVolume(pid: number, vol: number) {
    setSessions((prev) =>
      prev.map((s) => (s.process_id === pid ? { ...s, volume: vol } : s))
    );
    try {
      await invoke("set_session_volume", { processId: pid, volume: vol });
    } catch (e: any) {
      setErrorMsg(e.toString());
      fetchSessions();
    }
  }

  async function setMute(pid: number, mute: boolean) {
    setSessions((prev) =>
      prev.map((s) => (s.process_id === pid ? { ...s, is_muted: mute } : s))
    );
    try {
      await invoke("set_session_mute", { processId: pid, mute });
    } catch (e: any) {
      setErrorMsg(e.toString());
      fetchSessions();
    }
  }

  async function setAudioRouting(pid: number, deviceId: string) {
    if (!deviceId) return;
    try {
      await invoke("set_audio_routing", { processId: pid, deviceId });
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  return (
    <main
      ref={containerRef}
      className={`flex flex-col h-screen overflow-hidden text-white select-none transition-all duration-300 ${isEntering ? 'window-enter-active' : 'window-enter'}`}
      style={{ background: "transparent" }}
    >
      {/* 背景オーバーレイ */}
      <div className="absolute inset-0 bg-black/20 pointer-events-none z-[-1]"></div>

      {/* カスタムタイトルバー（ドラッグ可能領域） */}
      <div
        data-tauri-drag-region
        className="h-12 flex items-center justify-between px-4"
      >
        <div data-tauri-drag-region className="text-xs font-semibold tracking-widest flex items-center gap-3 text-white/90 uppercase">
          <div className="w-1 h-4 bg-blue-500 rounded-full"></div>
          Sound Generator
        </div>
        <div className="flex z-50 gap-1">
          <button onClick={() => appWindow.minimize()} className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-white/10 transition-colors">
            <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M1,6v1h10V6H1z" /></svg>
          </button>
          <button onClick={() => { setIsEntering(false); setTimeout(() => appWindow.hide(), 200); }} className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-red-500/80 transition-colors">
            <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M11,2.1L9.9,1L6,4.9L2.1,1L1,2.1L4.9,6L1,9.9L2.1,11L6,7.1L9.9,11L11,9.9L7.1,6L11,2.1z" /></svg>
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-5 pb-6 scroll-smooth">
        <div className="flex justify-between items-center mt-2 mb-6">
          <h1 className="text-2xl font-bold tracking-tight">Mixer</h1>
          <button
            className="w-8 h-8 flex items-center justify-center rounded-full bg-white/5 hover:bg-white/15 border border-white/5 transition-all active:scale-90"
            onClick={fetchSessions}
            title="Refresh"
          >
            <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path></svg>
          </button>
        </div>

        {errorMsg && (
          <div className="animate-in fade-in slide-in-from-top-2 duration-300 bg-red-500/10 text-red-400 p-3 rounded-xl border border-red-500/20 mb-4 text-xs">
            {errorMsg}
          </div>
        )}

        <div className="space-y-4">
          {sessions.map((session) => (
            <div
              key={session.process_id}
              className="group relative bg-white/[0.03] hover:bg-white/[0.08] border border-white/5 rounded-2xl p-4 transition-all duration-300 ease-out"
            >
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 flex-shrink-0 bg-black/20 rounded-xl flex items-center justify-center border border-white/5 shadow-inner overflow-hidden">
                  {session.icon_base64 ? (
                    <img
                      src={`data:image/png;base64,${session.icon_base64}`}
                      alt={session.process_name}
                      className="w-8 h-8 object-contain"
                    />
                  ) : (
                    <div className="text-[10px] font-bold text-white/40">{session.process_name.substring(0, 2).toUpperCase()}</div>
                  )}
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex justify-between items-center mb-1">
                    <h3 className="font-bold text-sm truncate text-white/90">{session.process_name}</h3>
                    <div className="flex items-center gap-2">
                      <select
                        className="bg-transparent text-[10px] text-white/40 hover:text-white/80 transition-colors cursor-pointer appearance-none outline-none border-none pr-1"
                        onChange={(e) => setAudioRouting(session.process_id, e.target.value)}
                        defaultValue=""
                      >
                        <option value="" disabled>Routing</option>
                        {devices.map(d => (
                          <option key={d.id} value={d.id} className="bg-neutral-900">{d.name}</option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div className="flex items-center gap-4 relative">
                    <div className="relative flex-1 flex items-center group/slider">
                      {/* 
                          Fluent UI Slider (Custom CSS)
                          背景の青いトラック（進捗）を表現するためにインラインスタイルで動的グラデーションを使用 
                      */}
                      <input
                        type="range"
                        min="0" max="1" step="0.01"
                        value={session.volume}
                        onChange={(e) => setVolume(session.process_id, parseFloat(e.target.value))}
                        className="fluent-slider"
                        style={{
                          background: `linear-gradient(to right, #60A5FA ${(session.volume * 100)}%, rgba(255,255,255,0.1) ${(session.volume * 100)}%)`
                        }}
                      />
                    </div>
                    <span className="text-[11px] text-right w-8 text-white/60 font-mono font-bold">
                      {Math.round(session.volume * 100)}
                    </span>
                  </div>
                </div>

                <button
                  className={`flex-shrink-0 w-10 h-10 flex items-center justify-center rounded-xl transition-all active:scale-90 border ${session.is_muted
                    ? 'bg-red-500/20 text-red-400 border-red-500/20 hover:bg-red-500/30'
                    : 'bg-white/5 text-white/60 border-white/10 hover:bg-white/15 hover:text-white'
                    }`}
                  onClick={() => setMute(session.process_id, !session.is_muted)}
                >
                  {session.is_muted ? (
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" /></svg>
                  ) : (
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" /></svg>
                  )}
                </button>
              </div>
            </div>
          ))}
          {sessions.length === 0 && (
            <div className="text-center py-12 text-white/40 border border-white/5 bg-white/5 rounded-2xl flex flex-col items-center gap-2">
              <svg className="w-8 h-8 opacity-50 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"></path></svg>
              <p className="text-sm font-medium tracking-wide">No active audio streams found</p>
              <p className="text-xs opacity-70">Play some audio and hit Refresh.</p>
            </div>
          )}
        </div>
      </div>
      
      {/* 
          フッター：EarTrumpetらしい、トレイアイコンのような親しみやすさ。 
      */}
      <div className="h-10 px-5 flex items-center justify-between border-t border-white/5 bg-white/[0.02]">
        <div className="text-[10px] text-white/30 font-medium">ANTIGRAVITY PROTOCOL v3.1</div>
        <div className="flex gap-4">
           <div className="w-1.5 h-1.5 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"></div>
        </div>
      </div>
    </main>
  );
}

export default App;
