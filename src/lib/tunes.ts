/** Web Audio API tune synthesizer — no audio files needed */

export const TUNES = [
  { id: "bell",    name: "Bell",    emoji: "🔔" },
  { id: "chime",   name: "Chime",   emoji: "🎵" },
  { id: "ping",    name: "Ping",    emoji: "📣" },
  { id: "gentle",  name: "Gentle",  emoji: "🌙" },
  { id: "alert",   name: "Alert",   emoji: "🚨" },
  { id: "none",    name: "Silent",  emoji: "🔕" },
] as const;

export type TuneId = typeof TUNES[number]["id"];

function ctx(): AudioContext {
  return new (window.AudioContext || (window as any).webkitAudioContext)();
}

export function playTune(id: TuneId): void {
  if (id === "none") return;
  try {
    switch (id) {
      case "bell":   playBell();   break;
      case "chime":  playChime();  break;
      case "ping":   playPing();   break;
      case "gentle": playGentle(); break;
      case "alert":  playAlert();  break;
    }
  } catch {}
}

function tone(ac: AudioContext, freq: number, start: number, dur: number, type: OscillatorType = "sine", gain = 0.4) {
  const osc = ac.createOscillator();
  const g   = ac.createGain();
  osc.type = type;
  osc.frequency.value = freq;
  g.gain.setValueAtTime(0, ac.currentTime + start);
  g.gain.linearRampToValueAtTime(gain, ac.currentTime + start + 0.01);
  g.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + start + dur);
  osc.connect(g);
  g.connect(ac.destination);
  osc.start(ac.currentTime + start);
  osc.stop(ac.currentTime + start + dur + 0.01);
}

function playBell() {
  const ac = ctx();
  tone(ac, 880, 0, 1.8, "sine", 0.5);
  tone(ac, 1760, 0, 0.9, "sine", 0.2);
}

function playChime() {
  const ac = ctx();
  const notes = [523, 659, 784, 1047]; // C5 E5 G5 C6
  notes.forEach((f, i) => tone(ac, f, i * 0.18, 1.0, "sine", 0.35));
}

function playPing() {
  const ac = ctx();
  tone(ac, 1200, 0, 0.35, "sine", 0.55);
  tone(ac, 1600, 0, 0.18, "sine", 0.2);
}

function playGentle() {
  const ac = ctx();
  tone(ac, 440, 0, 2.5, "sine", 0.28);
  tone(ac, 550, 0.05, 2.2, "sine", 0.14);
}

function playAlert() {
  const ac = ctx();
  [0, 0.22, 0.44].forEach(t => {
    tone(ac, 800, t, 0.15, "square", 0.3);
    tone(ac, 600, t + 0.15, 0.1, "square", 0.25);
  });
}
