import { FolderOpen, FilePlus2, ImageIcon, Save, SaveAll, Clock } from "lucide-react";
import { useRecentFiles } from "@/lib/queries";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from "@/components/ui/dropdown-menu";

export function FileMenu() {
  const newWorkspace = useWorkspaceFilesStore((s) => s.newWorkspace);
  const openWorkspace = useWorkspaceFilesStore((s) => s.openWorkspace);
  const openWorkspacePath = useWorkspaceFilesStore((s) => s.openWorkspacePath);
  const openImage = useWorkspaceFilesStore((s) => s.openImage);
  const save = useWorkspaceFilesStore((s) => s.save);
  const saveAs = useWorkspaceFilesStore((s) => s.saveAs);
  const { data: recent = [] } = useRecentFiles();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="rounded-md px-2.5 py-1 text-[13px] text-muted-foreground transition-colors outline-none hover:bg-secondary hover:text-foreground focus-visible:bg-secondary focus-visible:text-foreground data-[state=open]:bg-secondary data-[state=open]:text-foreground">
        File
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <DropdownMenuItem onSelect={() => newWorkspace()}>
          <FilePlus2 />
          New workspace
          <DropdownMenuShortcut>⌘N</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => openWorkspace()}>
          <FolderOpen />
          Open workspace…
          <DropdownMenuShortcut>⌘O</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => openImage()}>
          <ImageIcon />
          Open image…
        </DropdownMenuItem>

        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <Clock />
            Open recent
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent>
            {recent.length === 0 ? (
              <DropdownMenuItem disabled>No recent files</DropdownMenuItem>
            ) : (
              recent.map((file) => (
                <DropdownMenuItem key={file.path} onSelect={() => openWorkspacePath(file.path)}>
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate font-mono text-[12px] text-foreground">{file.name}</span>
                    <span className="truncate text-[10px] text-muted-foreground">{file.openedAt}</span>
                  </span>
                </DropdownMenuItem>
              ))
            )}
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSeparator />

        <DropdownMenuItem onSelect={() => save()}>
          <Save />
          Save workspace
          <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => saveAs()}>
          <SaveAll />
          Save as…
          <DropdownMenuShortcut>⌘⇧S</DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
