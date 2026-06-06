const ownership = [
  ["Rust core", "document state"],
  ["React", "interface"],
  ["Skia", "rendering"],
  ["Tauri", "native shell"]
] as const;

function App() {
  return (
    <main className="app-shell">
      <header className="top-bar">
        <strong>Fleck</strong>
        <span>Raster editor workspace</span>
      </header>

      <section className="editor-shell" aria-label="Fleck editor shell">
        <aside className="tool-strip" aria-label="Tool strip">
          <button type="button" aria-label="Move tool">M</button>
          <button type="button" aria-label="Selection tool">S</button>
          <button type="button" aria-label="Export area tool">E</button>
        </aside>

        <section className="canvas-region" aria-label="Workspace canvas">
          <div className="canvas-placeholder">
            <h1>Fleck Workspace</h1>
            <p>Task 1 scaffold: desktop shell, frontend app, and Rust workspace boundaries.</p>
          </div>
        </section>

        <aside className="side-panels" aria-label="Editor panels">
          <Panel title="Inspector" items={["No selection", "Rust owns document state"]} />
          <Panel title="Layers" items={["No layers yet"]} />
          <Panel title="Exports" items={["No export areas yet"]} />
          <Panel title="History" items={["No operations yet"]} />
        </aside>
      </section>

      <footer className="status-bar">
        {ownership.map(([owner, responsibility]) => (
          <span key={owner}>
            {owner}: {responsibility}
          </span>
        ))}
      </footer>
    </main>
  );
}

function Panel({ title, items }: { title: string; items: string[] }) {
  return (
    <section className="panel" aria-label={title}>
      <h2>{title}</h2>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </section>
  );
}

export default App;
