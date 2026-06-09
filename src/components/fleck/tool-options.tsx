import { useId, type ReactNode } from "react";
import { TOOLS } from "@/lib/fleck-data";
import { cn } from "@/lib/utils";
import { useUIStore, type LassoMode, type MarqueeShape, type ToolColor, type ToolOptions } from "@/store/ui-store";

/**
 * Contextual options bar for the active tool. Sits between the menu bar and the
 * editor body so options are always visible without re-opening a menu, matching
 * the canvas-first layout (REQ-037). Tools without options surface a hint
 * instead of leaving the strip blank.
 */
export function ToolOptionsBar() {
  const activeTool = useUIStore((s) => s.activeTool);
  const tool = TOOLS.find((t) => t.id === activeTool);

  return (
    <div
      className="flex h-9 shrink-0 items-center gap-3 border-b border-border bg-sidebar/70 px-3 text-[11px]"
      aria-label={`${tool?.name ?? "Tool"} options`}
    >
      <span className="flex w-32 shrink-0 items-center gap-1.5 text-muted-foreground">
        <span className="text-foreground">{tool?.name ?? activeTool}</span>
      </span>
      <div className="flex flex-1 items-center gap-3 overflow-x-auto">
        <ToolOptionsFor activeTool={activeTool} />
      </div>
    </div>
  );
}

function ToolOptionsFor({ activeTool }: { activeTool: string }) {
  const marqueeShape = useUIStore((s) => s.marqueeShape);
  const setMarqueeShape = useUIStore((s) => s.setMarqueeShape);
  const lassoMode = useUIStore((s) => s.lassoMode);
  const setLassoMode = useUIStore((s) => s.setLassoMode);
  const options = useUIStore((s) => s.toolOptions);
  const setOption = useUIStore((s) => s.setToolOption);

  switch (activeTool) {
    case "marquee":
      return (
        <Segmented
          label="Shape"
          value={marqueeShape}
          onChange={(v) => setMarqueeShape(v as MarqueeShape)}
          options={[
            { value: "rect", label: "Rect" },
            { value: "ellipse", label: "Ellipse" },
          ]}
        />
      );
    case "lasso":
      return (
        <Segmented
          label="Mode"
          value={lassoMode}
          onChange={(v) => setLassoMode(v as LassoMode)}
          options={[
            { value: "freehand", label: "Freehand" },
            { value: "polygon", label: "Polygon" },
          ]}
        />
      );
    case "wand":
      return (
        <RangeField
          label="Tolerance"
          value={Math.round(options.wandTolerance * 100)}
          min={0}
          max={100}
          suffix="%"
          onChange={(v) => setOption("wandTolerance", v / 100)}
        />
      );
    case "brush":
      return (
        <BrushishOptions
          radius={options.brushRadius}
          opacity={options.brushOpacity}
          color={options.color}
          onRadius={(v) => setOption("brushRadius", v)}
          onOpacity={(v) => setOption("brushOpacity", v)}
          onColor={(v) => setOption("color", v)}
        />
      );
    case "eraser":
      return (
        <>
          <RangeField
            label="Size"
            value={options.eraserRadius}
            min={1}
            max={200}
            suffix="px"
            onChange={(v) => setOption("eraserRadius", v)}
          />
          <RangeField
            label="Opacity"
            value={Math.round(options.eraserOpacity * 100)}
            min={0}
            max={100}
            suffix="%"
            onChange={(v) => setOption("eraserOpacity", v / 100)}
          />
        </>
      );
    case "fill":
      return (
        <>
          <RangeField
            label="Tolerance"
            value={Math.round(options.wandTolerance * 100)}
            min={0}
            max={100}
            suffix="%"
            onChange={(v) => setOption("wandTolerance", v / 100)}
          />
          <ColorSwatch color={options.color} onChange={(v) => setOption("color", v)} />
        </>
      );
    case "picker":
      return (
        <>
          <ColorSwatch color={options.color} readOnly />
          <span className="text-muted-foreground">Click the canvas to sample.</span>
        </>
      );
    case "crop":
      return <span className="text-muted-foreground">Drag to set the crop rectangle.</span>;
    case "area":
      return <span className="text-muted-foreground">Click an empty area to mark a region.</span>;
    case "move":
      return <span className="text-muted-foreground">Drag layers or arrow-nudge the active selection.</span>;
    case "text":
    case "shape":
      return <span className="text-muted-foreground">Not implemented yet.</span>;
    case "pan":
      return <span className="text-muted-foreground">Hold space to pan from any tool.</span>;
    case "zoom":
      return <span className="text-muted-foreground">Click to zoom in, alt-click to zoom out.</span>;
    default:
      return null;
  }
}

function BrushishOptions({
  radius,
  opacity,
  color,
  onRadius,
  onOpacity,
  onColor,
}: {
  radius: number;
  opacity: number;
  color: ToolColor;
  onRadius: (v: number) => void;
  onOpacity: (v: number) => void;
  onColor: (v: ToolColor) => void;
}) {
  return (
    <>
      <RangeField label="Size" value={radius} min={1} max={200} suffix="px" onChange={onRadius} />
      <RangeField
        label="Opacity"
        value={Math.round(opacity * 100)}
        min={0}
        max={100}
        suffix="%"
        onChange={(v) => onOpacity(v / 100)}
      />
      <ColorSwatch color={color} onChange={onColor} />
    </>
  );
}

function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <Field label={label}>
      <div className="flex items-center gap-0.5 rounded-md border border-border bg-background p-0.5">
        {options.map((opt) => (
          <button
            key={opt.value}
            onClick={() => onChange(opt.value)}
            aria-pressed={value === opt.value}
            className={cn(
              "rounded px-2 py-0.5 text-[11px] transition-colors",
              value === opt.value
                ? "bg-primary/15 text-primary"
                : "text-muted-foreground hover:bg-secondary hover:text-foreground",
            )}
          >
            {opt.label}
          </button>
        ))}
      </div>
    </Field>
  );
}

function RangeField({
  label,
  value,
  min,
  max,
  suffix,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  suffix: string;
  onChange: (value: number) => void;
}) {
  return (
    <Field label={label}>
      <div className="flex items-center gap-1.5">
        <input
          type="range"
          min={min}
          max={max}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
          className="h-1 w-24 cursor-pointer accent-primary"
          aria-label={label}
        />
        <span className="w-9 text-right font-mono text-foreground">
          {value}
          {suffix}
        </span>
      </div>
    </Field>
  );
}

function ColorSwatch({
  color,
  onChange,
  readOnly,
}: {
  color: ToolColor;
  onChange?: (value: ToolColor) => void;
  readOnly?: boolean;
}) {
  const id = useId();
  const hex = rgbToHex(color);
  return (
    <Field label="Color">
      <label
        htmlFor={id}
        className={cn(
          "flex items-center gap-1.5 rounded-md border border-border bg-background px-1.5 py-0.5",
          readOnly && "opacity-80",
        )}
      >
        <span
          aria-hidden="true"
          className="size-4 rounded border border-border"
          style={{ backgroundColor: hex }}
        />
        <span className="font-mono text-foreground">{hex.toUpperCase()}</span>
        {!readOnly && onChange && (
          <input
            id={id}
            type="color"
            value={hex}
            onChange={(e) => onChange({ ...color, ...hexToRgb(e.target.value) })}
            className="sr-only"
          />
        )}
      </label>
    </Field>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-muted-foreground">{label}</span>
      {children}
    </div>
  );
}

function rgbToHex({ r, g, b }: ToolColor): string {
  const c = (v: number) => v.toString(16).padStart(2, "0");
  return `#${c(r)}${c(g)}${c(b)}`;
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const v = hex.replace("#", "");
  return {
    r: parseInt(v.slice(0, 2), 16),
    g: parseInt(v.slice(2, 4), 16),
    b: parseInt(v.slice(4, 6), 16),
  };
}

export type { ToolOptions };
