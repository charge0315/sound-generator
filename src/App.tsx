import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AudioSession {
  process_id: number;
  process_name: string;
  volume: number;
  is_muted: boolean;
  peak_level: number;
  icon_base64: string | null;
  device_id: string;
}

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

interface PeakData {
  pid: number;
  peak: number;
}

function App() {
  const [sessions, setSessions] = useState<AudioSession[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [draggedPid, setDraggedPid] = useState<number | null>(null);
  const [tacticalMode, setTacticalMode] = useState(false);
  
  const canvasRefs = useRef<Record<number, HTMLCanvasElement | null>>({});

  useEffect(() => {
    refreshData();

    const unlistenPulse = listen<PeakData[]>("audio-pulse", (event) => {
      event.payload.forEach((p) => drawPeak(p.pid, p.peak));
    });

    const unlistenVolume = listen<any>("volume-change", () => refreshData());
    const unlistenRefresh = listen("refresh-trigger", () => refreshData());
    const unlistenAutoRefresh = listen<AudioSession[]>("refresh-sessions", (event) => {
      setSessions(event.payload);
    });

    return () => {
      unlistenPulse.then((f) => f());
      unlistenVolume.then((f) => f());
      unlistenRefresh.then((f) => f());
      unlistenAutoRefresh.then((f) => f());
    };
  }, []);

  const refreshData = async () => {
    try {
      const [sessionData, deviceData] = await Promise.all([
        invoke<AudioSession[]>("get_audio_sessions"),
        invoke<AudioDevice[]>("get_audio_devices")
      ]);
      setSessions(sessionData);
      setDevices(deviceData);
    } catch (e) {
      console.error("Failed to fetch data", e);
    }
  };

  const drawPeak = (pid: number, peak: number) => {
    const canvas = canvasRefs.current[pid];
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;
    ctx.clearRect(0, 0, width, height);

    const gradient = ctx.createLinearGradient(0, 0, width, 0);
    gradient.addColorStop(0, "rgba(0, 242, 255, 0.1)");
    gradient.addColorStop(0.5, "rgba(0, 242, 255, 0.8)");
    gradient.addColorStop(1, "rgba(0, 242, 255, 0.1)");

    ctx.fillStyle = gradient;
    ctx.shadowBlur = 12;
    ctx.shadowColor = "#00f2ff";
    ctx.fillRect(0, 0, width * peak, height);
  };

  const updateVolume = async (pid: number, volume: number) => {
    await invoke("set_session_volume", { pid, volume });
    setSessions(prev => prev.map(s => s.process_id === pid ? { ...s, volume } : s));
  };

  const handleRoute = async (deviceId: string) => {
    if (draggedPid !== null) {
      try {
        await invoke("set_audio_routing", { pid: draggedPid, deviceId: deviceId });
        setDraggedPid(null);
        setTimeout(refreshData, 200);
      } catch (e) {
        console.error("Routing failed", e);
      }
    }
  };

  const toggleTactical = async (enabled: boolean) => {
    setTacticalMode(enabled);
    await invoke("set_tactical_mode", { enabled });
  };

  return (
    <main 
      onContextMenu={(e) => e.preventDefault()}
      className={`flex flex-col h-screen overflow-hidden p-4 space-y-4 select-none transition-all duration-500 ${tacticalMode ? 'bg-black/40' : 'bg-transparent'}`}
    >
      {/* Header */}
      <header className="flex justify-between items-end border-b border-pulse-neon/20 pb-3">
        <div className="flex items-center space-x-4">
          <div>
            <h1 className="text-lg font-black tracking-tighter text-pulse-neon leading-none">ANTIGRAVITY <span className="opacity-50">PULSE</span></h1>
            <p className="text-[9px] font-mono opacity-40 mt-1 uppercase tracking-widest">Master Instrument / Tactical Audio</p>
          </div>
          
          <label className="flex items-center space-x-2 cursor-pointer group pt-1">
            <div className="relative">
              <input 
                type="checkbox" 
                className="sr-only" 
                checked={tacticalMode} 
                onChange={(e) => toggleTactical(e.target.checked)} 
              />
              <div className={`w-8 h-4 rounded-full transition-colors ${tacticalMode ? 'bg-pulse-neon' : 'bg-white/10'}`} />
              <div className={`absolute left-0.5 top-0.5 w-3 h-3 rounded-full bg-white transition-transform ${tacticalMode ? 'translate-x-4' : 'translate-x-0'}`} />
            </div>
            <span className={`text-[10px] font-bold tracking-widest uppercase transition-colors ${tacticalMode ? 'text-pulse-neon' : 'text-white/30 group-hover:text-white/60'}`}>Tactical</span>
          </label>
        </div>
        <div className="flex items-center space-x-2 pb-0.5">
          <div className="w-1.5 h-1.5 rounded-full bg-pulse-neon shadow-[0_0_8px_#00f2ff] animate-pulse" />
          <span className="text-[10px] font-bold opacity-60 font-mono">LINKED</span>
        </div>
      </header>

      {/* Devices Grid */}
      <div className="space-y-2">
        <h2 className="text-[10px] font-bold opacity-30 uppercase tracking-[0.2em] px-1">Active Endpoints</h2>
        <div className="flex gap-2 overflow-x-auto pb-2 custom-scrollbar">
          {devices.map(device => (
            <div 
              key={device.id}
              onDragOver={(e) => e.preventDefault()}
              onDrop={() => handleRoute(device.id)}
              className={`flex-shrink-0 w-36 p-3 rounded-lg border transition-all duration-300 ${device.is_default ? 'border-pulse-neon/50 bg-pulse-neon/10' : 'border-white/10 bg-white/5'} hover:bg-white/10 hover:border-white/20`}
            >
              <div className="text-[10px] font-black truncate text-white/90 mb-1">{device.name}</div>
              <div className="flex justify-between items-center">
                <span className={`text-[8px] px-1.5 py-0.5 rounded ${device.is_default ? 'bg-pulse-neon text-black' : 'bg-white/10 text-white/40'}`}>
                  {device.is_default ? 'PRIMARY' : 'ACTIVE'}
                </span>
                <div className="w-8 h-1 bg-white/10 rounded-full overflow-hidden">
                  <div className={`h-full ${device.is_default ? 'bg-pulse-neon' : 'bg-white/30'} w-2/3 shadow-[0_0_5px_currentColor]`} />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Sessions List */}
      <div className="flex-1 flex flex-col space-y-2 overflow-hidden">
        <h2 className="text-[10px] font-bold opacity-30 uppercase tracking-[0.2em] px-1">Signal Sessions</h2>
        <section className="flex-1 overflow-y-auto space-y-2 pr-2 custom-scrollbar">
          {sessions.map((session) => (
            <div 
              key={session.process_id}
              draggable
              onDragStart={() => setDraggedPid(session.process_id)}
              onDragEnd={() => setDraggedPid(null)}
              className={`group bg-gradient-to-r from-white/5 to-transparent border border-white/10 rounded-xl p-4 transition-all duration-200 cursor-grab active:cursor-grabbing ${draggedPid === session.process_id ? 'opacity-30 scale-95 blur-sm' : 'hover:border-pulse-neon/30 hover:from-white/10'}`}
            >
              <div className="flex items-center space-x-4 mb-4">
                <div className="relative w-12 h-12 flex-shrink-0">
                  <div className="absolute inset-0 bg-pulse-neon/5 rounded-lg border border-white/5 group-hover:border-pulse-neon/20 transition-colors" />
                  <div className="absolute inset-0 flex items-center justify-center overflow-hidden p-2">
                    {session.icon_base64 ? (
                      <img src={`data:image/png;base64,${session.icon_base64}`} className="w-full h-full object-contain filter drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                    ) : (
                      <div className="w-6 h-6 bg-pulse-neon/20 rounded-md border border-pulse-neon/40 animate-pulse" />
                    )}
                  </div>
                </div>
                
                <div className="flex-1 min-w-0">
                  <div className="flex justify-between items-start">
                    <div className="text-[13px] font-black truncate text-white/80 group-hover:text-white transition-colors uppercase tracking-tight">
                      {session.process_name}
                    </div>
                    <button 
                      onClick={(e) => { e.stopPropagation(); invoke("set_session_mute", { pid: session.process_id, mute: !session.is_muted }); }}
                      className={`p-1.5 rounded-lg border transition-all ${session.is_muted ? 'bg-red-500/20 border-red-500/40 text-red-400' : 'bg-white/5 border-white/10 text-white/40 hover:text-pulse-neon hover:border-pulse-neon/40'}`}
                    >
                      <MuteIcon isMuted={session.is_muted} />
                    </button>
                  </div>
                  <div className="text-[9px] font-mono opacity-30 mt-1 flex items-center space-x-2">
                    <span>PID:{session.process_id}</span>
                    <span className="opacity-20">•</span>
                    <span className="truncate">{devices.find(d => d.id === session.device_id)?.name || "SYSTEM DEFAULT"}</span>
                  </div>
                </div>
              </div>

              <div className="space-y-3">
                <div className="relative h-2 bg-black/40 rounded-full overflow-hidden border border-white/5">
                  <canvas 
                    ref={(el) => { canvasRefs.current[session.process_id] = el; }}
                    width={340}
                    height={8}
                    className="absolute inset-0 w-full h-full"
                  />
                </div>
                <div className="flex items-center space-x-3">
                   <input 
                    type="range" 
                    min="0" max="1" step="0.01"
                    value={session.volume}
                    onChange={(e) => updateVolume(session.process_id, parseFloat(e.target.value))}
                    onDragStart={(e) => { e.preventDefault(); e.stopPropagation(); }}
                    onMouseDown={(e) => e.stopPropagation()}
                    onTouchStart={(e) => e.stopPropagation()}
                    className="flex-1 h-1.5"
                  />
                  <span className="text-[10px] font-mono opacity-50 w-8 text-right">{(session.volume * 100).toFixed(0)}%</span>
                </div>
              </div>
            </div>
          ))}
        </section>
      </div>

      <footer className="pt-3 border-t border-white/5 flex justify-between items-center text-[8px] font-mono opacity-20 uppercase tracking-[0.3em]">
        <span>Build v4.0.0 Stable</span>
        <span>Antigravity Engine // Pulse v2</span>
      </footer>
    </main>
  );
}

const MuteIcon = ({ isMuted }: { isMuted: boolean }) => (
  <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    {isMuted ? (
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
    ) : (
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
    )}
  </svg>
);

export default App;
