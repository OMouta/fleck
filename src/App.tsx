import { useEffect } from "react";
import { MenuBar } from "@/components/fleck/menu-bar";
import { ToolStrip } from "@/components/fleck/tool-strip";
import { Canvas } from "@/components/fleck/canvas";
import { SidePanel } from "@/components/fleck/side-panel";
import { StatusBar } from "@/components/fleck/status-bar";
import { CommandPalette } from "@/components/fleck/command-palette";
import { WorkspaceDialogs } from "@/components/fleck/workspace-dialogs";
import { TOOLS } from "@/lib/fleck-data";
import { pasteImageFlow } from "@/lib/image-import";
import { useUIStore } from "@/store/ui-store";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";
import { useCommandStore } from "@/store/command-store";

function App() {
  const paletteOpen = useUIStore((s) => s.paletteOpen);
  const togglePalette = useUIStore((s) => s.togglePalette);
  const setActiveTool = useUIStore((s) => s.setActiveTool);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;

      if (mod && e.key.toLowerCase() === "k") {
        e.preventDefault();
        togglePalette();
        return;
      }

      // Modifier shortcuts: file ops + the command engine (undo/redo/repeat).
      if (mod) {
        const files = useWorkspaceFilesStore.getState();
        const commands = useCommandStore.getState();
        const key = e.key.toLowerCase();
        if (key === "o") {
          e.preventDefault();
          files.openWorkspace();
          return;
        }
        if (key === "s") {
          e.preventDefault();
          if (e.shiftKey) files.saveAs();
          else files.save();
          return;
        }
        if (key === "n" && !e.shiftKey) {
          e.preventDefault();
          files.newWorkspace();
          return;
        }
        if (key === "z") {
          e.preventDefault();
          if (e.shiftKey) commands.redo();
          else commands.undo();
          return;
        }
        if (key === "y") {
          e.preventDefault();
          commands.redo();
          return;
        }
        if (key === "v" && !e.shiftKey) {
          // Paste an image from the clipboard, unless the user is editing text.
          const el = e.target as HTMLElement;
          if (el.tagName !== "INPUT" && el.tagName !== "TEXTAREA" && !el.isContentEditable) {
            e.preventDefault();
            pasteImageFlow();
          }
          return;
        }
        if (key === ".") {
          e.preventDefault();
          commands.repeatLast();
          return;
        }
        return; // leave other modifier combos alone
      }

      // Tool shortcuts only when not typing and no modifier held
      if (paletteOpen || e.altKey) return;
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
      <WorkspaceDialogs />
    </main>
  );
}

export default App;
