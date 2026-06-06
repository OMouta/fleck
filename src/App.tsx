import { useEffect } from "react";
import { MenuBar } from "@/components/fleck/menu-bar";
import { ToolStrip } from "@/components/fleck/tool-strip";
import { Canvas } from "@/components/fleck/canvas";
import { SidePanel } from "@/components/fleck/side-panel";
import { StatusBar } from "@/components/fleck/status-bar";
import { CommandPalette } from "@/components/fleck/command-palette";
import { TOOLS } from "@/lib/fleck-data";
import { useUIStore } from "@/store/ui-store";

function App() {
  const paletteOpen = useUIStore((s) => s.paletteOpen);
  const togglePalette = useUIStore((s) => s.togglePalette);
  const setActiveTool = useUIStore((s) => s.setActiveTool);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        togglePalette();
        return;
      }
      // Tool shortcuts only when not typing and no modifier held
      if (paletteOpen || e.metaKey || e.ctrlKey || e.altKey) return;
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;
      const match = TOOLS.find((t) => t.shortcut.toLowerCase() === e.key.toLowerCase());
      if (match) setActiveTool(match.id);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [paletteOpen, togglePalette, setActiveTool]);

  return (
    <main className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <MenuBar />
      <div className="flex flex-1 overflow-hidden">
        <ToolStrip />
        <Canvas />
        <SidePanel />
      </div>
      <StatusBar />
      <CommandPalette />
    </main>
  );
}

export default App;
