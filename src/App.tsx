import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, Event } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Rustから送られてくるイベントの型定義
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
  // UIとバックエンド（Rust）は疎結合になっており、
  // このStateは常にバックエンドの最新のオーディオセッション状態を反映するための箱として機能する。
  const [sessions, setSessions] = useState<any[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [errorMsg, setErrorMsg] = useState("");
  const appWindow = getCurrentWindow();

  // アプリ起動時の初期化。
  // Rust側から定期的なポーリングではなく、実際に音量や状態が変わった時のみ
  // イベント "audio-session-event" が飛んでくる（Push型アーキテクチャ）ため、低遅延かつ省電力。
  useEffect(() => {
    fetchSessions();
    fetchDevices();

    const unlisten = listen<AudioEventPayload>("audio-session-event", (event: Event<AudioEventPayload>) => {
      console.log("Audio Event Received:", event.payload);

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

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  async function fetchSessions() {
    try {
      const result = await invoke("get_audio_sessions");
      setSessions(result as any[]);
      setErrorMsg("");
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
    // 楽観的UI更新（Optimistic UI Update）:
    // COMを経由したRust側の音量制御はごくわずかな遅延が発生するため、
    // スライダーを操作した瞬間にUI（React）側のStateを先に更新してしまうことで、
    // ユーザーに「もたつき」を感じさせないヌルヌルとした操作感（Fluid UX）を提供する。
    setSessions((prev) =>
      prev.map((s) => (s.process_id === pid ? { ...s, volume: vol } : s))
    );
    try {
      await invoke("set_session_volume", { processId: pid, volume: vol });
    } catch (e: any) {
      setErrorMsg(e.toString());
      fetchSessions(); // リモートの制御に失敗した場合は、実際の状態にロールバックして整合性を保つ
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
      // ルーティング変更後、少し様子を見てリストをリフレッシュする（オプション）
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  // アプリ全体のフレーム。
  // Windows 11 の Mica/Acrylic エフェクトをRust側で有効にするため、
  // Reactのルート要素自体は完全に透過（bg-transparent）にしておく必要がある。
  return (
    <main
      className="flex flex-col h-screen overflow-hidden text-white select-none bg-black/20"
      style={{ background: "transparent" }}
    >
      {/* 
        Windowsのシステム設定（ライト/ダーク）によってAcrylicの透過具合が変わるため、
        文字の視認性を安定させるための薄いオーバーレイ。 
      */}
      <div className="absolute inset-0 bg-black/10 pointer-events-none z-[-1]"></div>

      {/* カスタムタイトルバー */}
      <div
        data-tauri-drag-region
        className="h-10 flex items-center justify-between px-4 hover:bg-white/5 transition-colors"
      >
        <div data-tauri-drag-region className="text-sm font-bold tracking-wide flex items-center gap-2 text-white drop-shadow-md">
          <svg className="w-4 h-4 text-blue-400" fill="currentColor" viewBox="0 0 20 20"><path d="M18 3a1 1 0 00-1.196-.98l-10 2A1 1 0 006 5v9.114A4.369 4.369 0 005 14c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V7.82l8-1.6v5.894A4.37 4.37 0 0015 12c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V3z"></path></svg>
          Sound Generator
        </div>
        <div className="flex z-50">
          {/* 最小化・閉じるボタンのモック（機能はTauri WindowControlsで代替） */}
          <button onClick={() => appWindow.minimize()} className="w-10 h-10 flex items-center justify-center hover:bg-white/10 transition-colors">
            <svg width="10" height="10" viewBox="0 0 10 10"><path fill="currentColor" d="M0,4.5v1h10v-1H0z" /></svg>
          </button>
          <button onClick={() => appWindow.close()} className="w-10 h-10 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors">
            <svg width="10" height="10" viewBox="0 0 10 10"><path fill="currentColor" d="M10,1.4L8.6,0L5,3.6L1.4,0L0,1.4L3.6,5L0,8.6L1.4,10L5,6.4L8.6,10L10,8.6L6.4,5L10,1.4z" /></svg>
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 scroll-smooth">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-xl tracking-tight font-medium">Volume Mixer</h1>
          <button
            className="bg-white/10 hover:bg-white/20 border border-white/10 shadow-sm backdrop-blur-md text-sm text-white py-1.5 px-4 rounded-full transition-all active:scale-95 flex items-center gap-2"
            onClick={fetchSessions}>
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path></svg>
            Refresh
          </button>
        </div>

        {errorMsg && <p className="text-red-400 bg-red-900/20 p-3 rounded-xl border border-red-500/30 mb-4 text-sm">{errorMsg}</p>}

        <div className="space-y-3">
          {sessions.map((session) => (
            <div
              key={session.process_id}
              className="group relative bg-white/5 hover:bg-white/10 border border-white/10 rounded-2xl p-4 transition-all duration-300 ease-out flex items-center gap-4 shadow-lg backdrop-blur-md overflow-hidden"
            >
              {/* 
                Fluent Design特有の「ホバー時に要素全体がうっすらとハイライトされる」表現の模倣。
                マウスオーバー時のみ発動するグラデーションアニメーション。 
              */}
              <div className="absolute inset-0 bg-gradient-to-r from-white/0 via-white/5 to-white/0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none -translate-x-full group-hover:translate-x-full" />

              <div className="w-12 h-12 flex-shrink-0 bg-white/10 rounded-xl flex items-center justify-center border border-white/10 overflow-hidden relative">
                {session.icon_base64 ? (
                  // Rust側でプロセスIDから抽出した「本物の」アプリアイコン。
                  // システムの音量ミキサーと同じ見た目を再現するために必須の要素。
                  <img
                    src={`data:image/png;base64,${session.icon_base64}`}
                    alt={`${session.process_name} icon`}
                    className="w-8 h-8 object-contain drop-shadow"
                  />
                ) : (
                  <span className="text-xs font-bold text-white max-w-[40px] truncate leading-tight">
                    {session.process_name.substring(0, 4).toUpperCase()}
                  </span>
                )}
              </div>

              <div className="flex-1 min-w-0">
                <div className="flex justify-between items-baseline mb-2">
                  <h3 className="font-bold text-sm truncate text-white drop-shadow-md">{session.process_name}</h3>
                  <span className="text-xs text-gray-200 font-mono tracking-tighter ml-2 bg-black/40 px-1 rounded border border-white/5">PID:{session.process_id}</span>
                </div>

                <div className="flex items-center gap-4">
                  {/* Fluent Volume Slider */}
                  <div className="relative flex-1 group/slider flex items-center">
                    <input
                      type="range"
                      min="0" max="1" step="0.01"
                      value={session.volume}
                      onChange={(e) => setVolume(session.process_id, parseFloat(e.target.value))}
                      className="w-full h-1.5 bg-white/10 rounded-full appearance-none outline-none focus:outline-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:bg-blue-400 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-125 focus:[&::-webkit-slider-thumb]:ring-4 focus:[&::-webkit-slider-thumb]:ring-blue-400/30"
                      style={{
                        background: `linear-gradient(to right, #60A5FA ${(session.volume * 100)}%, rgba(255,255,255,0.1) ${(session.volume * 100)}%)`
                      }}
                    />
                  </div>
                  <span className="text-xs text-right w-[42px] text-white font-extrabold tabular-nums drop-shadow-md">
                    {(session.volume * 100).toFixed(0)}%
                  </span>
                </div>
              </div>

              <div className="flex flex-col gap-2">
                <button
                  className={`flex-shrink-0 w-10 h-10 flex items-center justify-center rounded-xl transition-all active:scale-90 shadow-sm border ${session.is_muted
                    ? 'bg-white/20 text-white hover:bg-white/30 border-white/30'
                    : 'bg-blue-500/30 text-blue-100 border-blue-500/40 hover:bg-blue-500/40 hover:text-white'
                    }`}
                  onClick={() => setMute(session.process_id, !session.is_muted)}
                  title={session.is_muted ? 'Unmute' : 'Mute'}
                >
                  {session.is_muted ? (
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM12.293 7.293a1 1 0 011.414 0L15 8.586l1.293-1.293a1 1 0 111.414 1.414L16.414 10l1.293 1.293a1 1 0 01-1.414 1.414L15 11.414l-1.293 1.293a1 1 0 01-1.414-1.414L13.586 10l-1.293-1.293a1 1 0 010-1.414z" clipRule="evenodd"></path></svg>
                  ) : (
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM14.657 2.929a1 1 0 011.414 0A9.972 9.972 0 0119 10a9.972 9.972 0 01-2.929 7.071 1 1 0 01-1.414-1.414A7.971 7.971 0 0017 10c0-2.21-.894-4.208-2.343-5.657a1 1 0 010-1.414zm-2.829 2.828a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z" clipRule="evenodd"></path></svg>
                  )}
                </button>
                <select
                  title="Route Audio"
                  className="w-10 flex-shrink-0 bg-transparent text-white/70 hover:text-white cursor-pointer appearance-none outline-none text-center"
                  onChange={(e) => setAudioRouting(session.process_id, e.target.value)}
                  defaultValue=""
                >
                  <option value="" disabled>⚙️</option>
                  {devices.map(d => (
                    <option key={d.id} value={d.id} className="text-black">{d.name}</option>
                  ))}
                </select>
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
    </main>
  );
}

export default App;
