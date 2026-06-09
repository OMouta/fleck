import {
  Maximize2,
  Minimize2,
  Feather,
  FlipVertical2,
  Copy,
  Trash2,
  Layers,
  Frame,
  FileDown,
  SquareDashed,
} from "lucide-react";
import { useUIStore } from "@/store/ui-store";
import { useCommandStore } from "@/store/command-store";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
} from "@/components/ui/dropdown-menu";

/**
 * Menu-bar entry for selection operations. Routes through the command engine
 * (same `selection.*` IDs as the palette and HUD) so menu actions are undoable
 * and visible in history.
 */
export function SelectMenu() {
  const activeSelectionId = useUIStore((s) => s.activeSelectionId);
  const execute = useCommandStore((s) => s.execute);
  const has = activeSelectionId !== null;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="rounded-md px-2.5 py-1 text-[13px] text-muted-foreground transition-colors outline-none hover:bg-secondary hover:text-foreground focus-visible:bg-secondary focus-visible:text-foreground data-[state=open]:bg-secondary data-[state=open]:text-foreground">
        Select
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.expand", { amount: 1 })}>
          <Maximize2 />
          Expand
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.contract", { amount: 1 })}>
          <Minimize2 />
          Contract
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.feather", { radius: 2 })}>
          <Feather />
          Feather
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.invert")}>
          <FlipVertical2 />
          Invert
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.copy")}>
          <Copy />
          Copy
          <DropdownMenuShortcut>⌘C</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.layer_from_selection")}>
          <Layers />
          Layer from selection
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.area_from_selection")}>
          <Frame />
          Area from selection
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.direct_export")}>
          <FileDown />
          Export selection
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem disabled={!has} onSelect={() => execute("selection.delete")}>
          <Trash2 />
          Deselect
          <DropdownMenuShortcut>Del</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem disabled={!has} onSelect={() => useUIStore.getState().setActiveSelectionId(null)}>
          <SquareDashed />
          Clear focus
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
