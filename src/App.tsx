function App() {
  return (
    <main className="app-shell">
      <header className="top-bar">
        <strong>Fleck</strong>
        <nav aria-label="Main menu">
          <button type="button">File</button>
          <button type="button">Edit</button>
          <button type="button">View</button>
          <button type="button">Export</button>
        </nav>
      </header>

      <section className="editor-shell" aria-label="Fleck editor shell">
        <aside className="tool-strip" aria-label="Tool strip">
          <button type="button" aria-label="Move tool">M</button>
          <button type="button" aria-label="Selection tool">S</button>
          <button type="button" aria-label="Export area tool">E</button>
        </aside>

        <section className="canvas-region" aria-label="Workspace canvas">
          <div className="canvas-placeholder">
            <div className="empty-workspace">
              <h1>Untitled Workspace</h1>
              <div className="empty-actions" aria-label="Workspace actions">
                <button type="button">Open Image</button>
                <button type="button">New Workspace</button>
                <button type="button">Create Export Area</button>
              </div>
            </div>
          </div>
        </section>

        <aside className="side-panels" aria-label="Editor panels">
          <Panel title="Inspector" items={["No selection"]} />
          <Panel title="Layers" items={["Background"]} />
          <Panel title="Exports" items={["No export areas"]} />
          <Panel title="History" items={["New workspace"]} />
        </aside>
      </section>

      <footer className="status-bar">
        <span>100%</span>
        <span>0 x 0 px</span>
        <span>No selection</span>
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
