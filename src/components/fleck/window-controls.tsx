import { useEffect, useState } from "react";
import { Minus, Square, Copy, X } from "lucide-react";
import { appWindow, isTauri } from "@/lib/window";

/**
 * Custom-titlebar window controls (minimize / maximize / close). Only renders in
 * the Tauri desktop shell — in a browser the OS/browser chrome handles this.
 */
export function WindowControls() {
  const [available, setAvailable] = useState(false);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri()) return;
    setAvailable(true);

    let unlisten: (() => void) | undefined;
    appWindow.isMaximized().then(setMaximized);
    appWindow
      .onResized(async () => setMaximized(await appWindow.isMaximized()))
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, []);

  if (!available) return null;

  return (
    <div className="-mr-3 ml-1 flex items-center self-stretch">
      <ControlButton label="Minimize" onClick={() => appWindow.minimize()}>
        <Minus className="size-4" />
      </ControlButton>
      <ControlButton
        label={maximized ? "Restore" : "Maximize"}
        onClick={() => appWindow.toggleMaximize()}
      >
        {maximized ? <Copy className="size-[13px]" /> : <Square className="size-3.5" />}
      </ControlButton>
      <ControlButton label="Close" onClick={() => appWindow.close()} danger>
        <X className="size-4" />
      </ControlButton>
    </div>
  );
}

function ControlButton({
  children,
  label,
  onClick,
  danger = false,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      aria-label={label}
      title={label}
      className={
        "flex h-11 w-[46px] items-center justify-center text-muted-foreground transition-colors " +
        (danger
          ? "hover:bg-destructive hover:text-destructive-foreground"
          : "hover:bg-secondary hover:text-foreground")
      }
    >
      {children}
    </button>
  );
}
