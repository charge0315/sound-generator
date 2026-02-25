import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [sessions, setSessions] = useState<any[]>([]);
  const [errorMsg, setErrorMsg] = useState("");

  async function fetchSessions() {
    try {
      const result = await invoke("get_audio_sessions");
      setSessions(result as any[]);
      setErrorMsg("");
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  async function setVolume(pid: number, vol: number) {
    try {
      await invoke("set_session_volume", { processId: pid, volume: vol });
      fetchSessions();
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  async function setMute(pid: number, mute: boolean) {
    try {
      await invoke("set_session_mute", { processId: pid, mute });
      fetchSessions();
    } catch (e: any) {
      setErrorMsg(e.toString());
    }
  }

  return (
    <main className="container p-4 flex flex-col items-center">
      <h1 className="text-2xl font-bold mb-4">Sound Generator Dev Test</h1>
      <button
        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4"
        onClick={fetchSessions}>
        Fetch Audio Sessions
      </button>

      {errorMsg && <p className="text-red-500">{errorMsg}</p>}

      <div className="w-full max-w-2xl bg-gray-100 p-4 rounded text-black text-left">
        {sessions.map((session) => (
          <div key={session.process_id} className="border-b border-gray-300 py-2 flex items-center justify-between">
            <span className="font-semibold text-sm w-1/3">
              {session.process_name} (PID: {session.process_id})
            </span>
            <div className="flex items-center gap-2">
              <input
                type="range"
                min="0" max="1" step="0.01"
                value={session.volume}
                onChange={(e) => setVolume(session.process_id, parseFloat(e.target.value))}
              />
              <span className="text-xs w-10">{(session.volume * 100).toFixed(0)}%</span>
            </div>
            <button
              className={`px-3 py-1 rounded text-white text-xs ${session.is_muted ? 'bg-red-500' : 'bg-green-500'}`}
              onClick={() => setMute(session.process_id, !session.is_muted)}>
              {session.is_muted ? 'Unmute' : 'Mute'}
            </button>
          </div>
        ))}
        {sessions.length === 0 && <p className="text-sm text-gray-500">No active audio sessions found or not fetched yet.</p>}
      </div>
    </main>
  );
}

export default App;
