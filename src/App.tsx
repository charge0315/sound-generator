import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, Event } from "@tauri-apps/api/event";

interface AudioSession {
  process_id: number;
  process_name: string;
  volume: number;
  is_muted: boolean;
  peak_level: number;
  icon_base64?: string | null;
  device_id: string;
}

interface AudioDevice {
  id: string;
  name: string;
}

interface AudioEventPayload {
  process_id: number;
  volume: number | null;
  mute: boolean | null;
  state: string | null;
  icon_base64?: string | null;
}

function App() {
  const [sessions, setSessions] = useState<AudioSession[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [peaks, setPeaks] = useState<Record<number, number>>({});
  const [totalPeak, setTotalPeak] = useState(0);
  const [errorMsg, setErrorMsg] = useState("");
  const [isEntering, setIsEntering] = useState(true);

  useEffect(() => {
    refreshData();

    // 1. 高頻度ピークデータ (30fps)
    const unlistenPulse = listen<any[]>("audio-pulse", (event) => {
      let maxP = 0;
      setPeaks((prev) => {
        const next = { ...prev };
        event.payload.forEach((d: any) => {
          next[d.pid] = d.peak;
          if (d.peak > maxP) maxP = d.peak;
        });
        return next;
      });
      setTotalPeak(maxP);
    });

    // 2. 音量・ミュート変更通知
    const unlistenAudio = listen<AudioEventPayload>("audio-session-event", (event: Event<AudioEventPayload>) => {
      setSessions((prev) =>
        prev.map((s) => {
          if (s.process_id === event.payload.process_id) {
            return {
              ...s,
              volume: event.payload.volume !== null ? event.payload.volume : s.volume,
              is_muted: event.payload.mute !== null ? event.payload.mute : s.is_muted,
            };
          }
          return s;
        })
      );
    });

    // 3. デバイス変更 or ウィンドウ表示時の強制リフレッシュ
    const unlistenRefresh = listen("refresh-trigger", refreshData);
    const unlistenVisible = listen("window-visible", () => {
      console.log("PULSE: Window shown, refreshing...");
      refreshData();
    });

    // 4. 定期的な監視（5秒に一度、漏れているセッションを掻き集める）
    const interval = setInterval(refreshData, 5000);

    return () => {
      unlistenPulse.then((f) => f());
      unlistenAudio.then((f) => f());
      unlistenRefresh.then((f) => f());
      unlistenVisible.then((f) => f());
      clearInterval(interval);
    };
  }, []);

  async function refreshData() {
    try {
      const [s, d] = await Promise.all([
        invoke<AudioSession[]>("get_audio_sessions"),
        invoke<AudioDevice[]>("get_audio_devices")
      ]);
      setSessions(s);
      setDevices(d);
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  async function setVolume(pid: number, vol: number) {
    setSessions((prev) => prev.map((s) => (s.process_id === pid ? { ...s, volume: vol } : s)));
    await invoke("set_session_volume", { processId: pid, volume: vol }).catch(setErrorMsg);
  }

  async function setMute(pid: number, mute: boolean) {
    setSessions((prev) => prev.map((s) => (s.process_id === pid ? { ...s, is_muted: mute } : s)));
    await invoke("set_session_mute", { processId: pid, mute }).catch(setErrorMsg);
  }

  async function setRouting(pid: number, deviceId: string) {
    if (pid === 0) return;
    await invoke("set_audio_routing", { processId: pid, deviceId }).catch(setErrorMsg);
    setTimeout(refreshData, 500);
  }

  async function handleClose() {
    setIsEntering(false);
    setTimeout(() => {
      invoke("hide_window");
    }, 200);
  }

  return (
    <main
      className={`flex flex-col h-screen overflow-hidden text-white select-none transition-all duration-300 ${
        isEntering ? "window-enter-active" : "window-enter"
      }`}
      style={{ background: "transparent" }}
    >
      <div 
        className="absolute inset-0 bg-blue-500/10 pointer-events-none z-[-1] transition-opacity duration-150"
        style={{ opacity: totalPeak * 0.6 }}
      ></div>
      <div className="absolute inset-0 bg-[#0a0a0a]/95 border border-white/10 shadow-[0_0_40px_rgba(0,0,0,0.8)] pointer-events-none z-[-2] rounded-2xl"></div>

      <div data-tauri-drag-region className="h-12 flex items-center justify-between px-5 shrink-0">
        <div data-tauri-drag-region className="flex items-center gap-3">
          <div className="relative w-5 h-5 flex items-center justify-center">
             <div className="absolute inset-0 bg-blue-500/20 rounded-full animate-pulse"></div>
             <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#60a5fa" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
          </div>
          <span className="text-[10px] font-bold tracking-widest text-blue-400 uppercase">Antigravity Pulse</span>
        </div>
        <button onClick={handleClose} className="w-8 h-8 flex items-center justify-center rounded-xl hover:bg-red-500/20 hover:text-red-400 transition-all active:scale-90">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto overflow-x-hidden px-5 pb-6 custom-scrollbar space-y-12">
        {errorMsg && (
          <div className="bg-red-500/10 text-red-400 p-3 rounded-xl border border-red-500/20 mb-4 text-[10px] font-mono break-all">
            {errorMsg}
          </div>
        )}

        {devices.map((device) => {
          const deviceSessions = sessions.filter(s => s.device_id === device.id);
          return (
            <section key={device.id} className="animate-in fade-in slide-in-from-bottom-2 duration-500">
              <div className="flex items-center gap-4 mb-6">
                 <div className="w-2 h-7 bg-blue-500 rounded-full shadow-[0_0_15px_rgba(59,130,246,0.6)]"></div>
                 <h2 className="text-[20px] font-black tracking-tighter text-white/95 uppercase truncate leading-none">
                   {device.name}
                 </h2>
                 <div className="flex-1 h-[1px] bg-white/10"></div>
              </div>

              <div className="space-y-6">
                {deviceSessions.map((s) => (
                  <div key={s.process_id} className="group bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 rounded-[28px] p-6 transition-all duration-300">
                    <div className="flex flex-col gap-4">
                      <div className="flex justify-between items-start">
                         <h3 className="font-black text-[17px] text-white/95 break-words leading-tight flex-1 mr-4">
                           {s.process_name}
                         </h3>
                         <div className="text-[9px] font-mono text-white/10 bg-white/5 px-2 py-0.5 rounded-full border border-white/5 uppercase">PID:{s.process_id}</div>
                      </div>

                      {s.process_id !== 0 ? (
                        <div className="relative group/select">
                          <div className="absolute left-4 top-1/2 -translate-y-1/2 text-blue-400/40 group-hover/select:text-blue-400 transition-colors">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                          </div>
                          <select
                            className="appearance-none bg-blue-500/5 border border-white/5 hover:border-blue-500/30 rounded-2xl w-full pl-12 pr-4 py-2.5 text-[11px] font-black text-blue-300/80 hover:text-blue-100 transition-all cursor-pointer outline-none"
                            onChange={(e) => setRouting(s.process_id, e.target.value)}
                            value={s.device_id}
                          >
                            {devices.map(d => (
                              <option key={d.id} value={d.id} className="bg-[#0a0a0a] text-white py-3">
                                SWITCH TO: {d.name}
                              </option>
                            ))}
                          </select>
                          <div className="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-white/10">
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><path d="M6 9l6 6 6-6"/></svg>
                          </div>
                        </div>
                      ) : (
                        <div className="px-4 py-2 bg-white/5 rounded-2xl text-[10px] font-bold text-white/20 italic tracking-widest text-center border border-white/5">
                          SYSTEM CORE ROUTING RESTRICTED
                        </div>
                      )}

                      <div className="flex items-center gap-5 pt-1">
                        <div className="w-14 h-14 flex-shrink-0 bg-black/60 rounded-2xl flex items-center justify-center border border-white/5 relative shadow-2xl overflow-hidden">
                          {s.icon_base64 ? (
                            <img src={`data:image/png;base64,${s.icon_base64}`} className="w-9 h-9 object-contain drop-shadow" alt="" />
                          ) : (
                            <div className="text-[12px] font-black text-white/10 uppercase">{s.process_name.substring(0, 2)}</div>
                          )}
                          <div className="absolute inset-0 bg-gradient-to-tr from-blue-500/10 to-transparent"></div>
                        </div>

                        <div className="flex-1 relative flex flex-col justify-center h-14">
                          <input
                            type="range" min="0" max="1" step="0.01"
                            value={s.volume}
                            onChange={(e) => setVolume(s.process_id, parseFloat(e.target.value))}
                            className="fluent-slider"
                            style={{ 
                              background: `linear-gradient(to right, #3b82f6 ${(s.volume * 100)}%, rgba(255,255,255,0.03) ${(s.volume * 100)}%)` 
                            }}
                          />
                          <div className="absolute left-0 bottom-2.5 w-full h-[4px] bg-white/5 rounded-full overflow-hidden pointer-events-none">
                            <div 
                              className="h-full bg-gradient-to-r from-green-500 via-green-400 to-yellow-300 shadow-[0_0_12px_rgba(74,222,128,0.5)] transition-all duration-75"
                              style={{ width: `${(peaks[s.process_id] || 0) * 100}%` }}
                            ></div>
                          </div>
                        </div>

                        <button
                          onClick={() => setMute(s.process_id, !s.is_muted)}
                          className={`w-12 h-12 flex-shrink-0 flex items-center justify-center rounded-2xl transition-all active:scale-90 border-2 ${
                            s.is_muted ? "bg-red-500/10 border-red-500/40 text-red-500 shadow-[0_0_20px_rgba(239,68,68,0.2)]" : "bg-white/5 border-white/5 text-white/40 hover:text-blue-400 hover:border-blue-400/30"
                          }`}
                        >
                          {s.is_muted ? (
                            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M11 5L6 9H2v6h4l5 4V5zM23 9l-6 6M17 9l6 6"/></svg>
                          ) : (
                            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M11 5L6 9H2v6h4l5 4V5zM19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>
                          )}
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </section>
          );
        })}
      </div>

      <div className="h-10 px-6 flex items-center justify-between border-t border-white/5 bg-white/[0.01]">
        <div className="flex items-center gap-2">
           <div className={`w-1.5 h-1.5 rounded-full ${totalPeak > 0.01 ? 'bg-green-500 animate-pulse' : 'bg-white/10'}`}></div>
           <span className="text-[9px] font-black tracking-widest text-white/30 uppercase italic tracking-widest">Pulse Protocol Stable // Optimized</span>
        </div>
        <div className="text-[8px] font-mono text-white/10 uppercase tracking-[.2em]">AtomMan G7 Pt // X-Pulse Engine</div>
      </div>
    </main>
  );
}

export default App;
