import type { ChangeEvent } from 'react';
import type { AppSettings } from '../types';
import { switcherCssVariables } from '../lib/switcherStyle';

const textRows: Array<[
  keyof AppSettings['appearance']['switcher'],
  string
]> = [
  ['headerTitle', 'Header title'],
  ['headerSubtitle', 'Header subtitle'],
  ['headerMeta', 'Header count / shortcut'],
  ['itemType', 'Item type label'],
  ['itemContent', 'Item content'],
  ['itemMeta', 'Item date / time'],
  ['detailContent', 'Large preview content'],
  ['detailMeta', 'Large preview metadata'],
  ['footer', 'Footer hint']
];

function ColorInput({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return (
    <label className="sw-color-control">
      <input type="color" value={value} onChange={(event) => onChange(event.target.value)} />
      <code>{value.toUpperCase()}</code>
    </label>
  );
}

function NumberInput({
  value,
  min,
  max,
  suffix,
  onChange
}: {
  value: number;
  min: number;
  max: number;
  suffix: string;
  onChange: (value: number) => void;
}) {
  const change = (event: ChangeEvent<HTMLInputElement>) => {
    const next = Number(event.target.value);
    if (Number.isFinite(next)) onChange(Math.min(max, Math.max(min, next)));
  };
  return (
    <label className="sw-number-control">
      <input type="number" value={value} min={min} max={max} onChange={change} />
      <span>{suffix}</span>
    </label>
  );
}

export function SwitcherAppearanceEditor({
  appearance,
  onChange
}: {
  appearance: AppSettings['appearance'];
  onChange: (value: AppSettings['appearance']) => void;
}) {
  const switcher = appearance.switcher;
  const updateSwitcher = (next: AppSettings['appearance']['switcher']) =>
    onChange({ ...appearance, switcher: next });

  const updateText = (
    key: keyof AppSettings['appearance']['switcher'],
    patch: Partial<{ fontSize: number; color: string }>
  ) => {
    const current = switcher[key];
    if (!current || typeof current !== 'object' || !('fontSize' in current)) return;
    updateSwitcher({ ...switcher, [key]: { ...current, ...patch } });
  };

  return (
    <div className="switcher-customizer">
      <div className="switcher-editor-preview-wrap">
        <div
          className="overlay-shell switcher switcher-split switcher-editor-preview"
          style={switcherCssVariables(appearance)}
        >
          <header className="switcher-header">
            <div>
              <div className="switcher-title">Clipboard</div>
              <div className="switcher-subtitle">Recent text and images</div>
            </div>
            <div className="switcher-header-meta"><span>8</span><kbd>Tab</kbd></div>
          </header>
          <div className="switcher-body">
            <section className="switcher-list">
              <div className="clip-card">
                <div className="clip-icon clip-kind">TXT</div>
                <div className="clip-main">
                  <div className="clip-title-row"><span className="clip-kind">Text</span><span className="clip-meta">12:46</span></div>
                  <div className="clip-text">A useful clipboard item stays one scroll away.</div>
                </div>
              </div>
              <div className="clip-card selected">
                <div className="clip-icon clip-kind">IMG</div>
                <div className="clip-main">
                  <div className="clip-title-row"><span className="clip-kind">Image</span><span className="clip-meta">12:44</span></div>
                  <div className="clip-text">Clipboard image</div>
                </div>
              </div>
              <div className="clip-card">
                <div className="clip-icon clip-kind">URL</div>
                <div className="clip-main">
                  <div className="clip-title-row"><span className="clip-kind">URL</span><span className="clip-meta">Yesterday, 22:08</span></div>
                  <div className="clip-text">https://github.com/SirPaul-code/ClipboardPreview</div>
                </div>
              </div>
            </section>
            <aside className="switcher-detail switcher-editor-detail">
              <div className="editor-image-placeholder" aria-hidden="true"><span>IMAGE PREVIEW</span></div>
              <div className="detail-caption"><strong>12:44</strong><span>Image preview</span></div>
            </aside>
          </div>
          <footer className="switcher-footer">Hold Tab · scroll · release</footer>
        </div>
      </div>

      <div className="switcher-control-group">
        <div className="switcher-control-heading"><strong>Text</strong><span>Size and color update the preview immediately.</span></div>
        {textRows.map(([key, label]) => {
          const current = switcher[key];
          if (!current || typeof current !== 'object' || !('fontSize' in current)) return null;
          const style = current as { fontSize: number; color: string };
          return (
            <div className="switcher-control-row" key={String(key)}>
              <span>{label}</span>
              <div className="switcher-control-inputs">
                <NumberInput value={style.fontSize} min={8} max={28} suffix="px" onChange={(fontSize) => updateText(key, { fontSize })} />
                <ColorInput value={style.color} onChange={(color) => updateText(key, { color })} />
              </div>
            </div>
          );
        })}
      </div>

      <div className="switcher-control-group">
        <div className="switcher-control-heading"><strong>Surface</strong><span>Keep the overlay restrained or make it yours.</span></div>
        {([
          ['panelBackground', 'Panel background'],
          ['rowBackground', 'Row background'],
          ['selectedBackground', 'Selected row'],
          ['borderColor', 'Borders'],
          ['selectedBorderColor', 'Selected border']
        ] as const).map(([key, label]) => (
          <div className="switcher-control-row" key={key}>
            <span>{label}</span>
            <ColorInput value={switcher[key]} onChange={(value) => updateSwitcher({ ...switcher, [key]: value })} />
          </div>
        ))}
        <div className="switcher-control-row">
          <span>Row height</span>
          <NumberInput value={switcher.rowHeight} min={38} max={88} suffix="px" onChange={(rowHeight) => updateSwitcher({ ...switcher, rowHeight })} />
        </div>
        <div className="switcher-control-row">
          <span>Thumbnail size</span>
          <NumberInput value={switcher.thumbnailSize} min={26} max={72} suffix="px" onChange={(thumbnailSize) => updateSwitcher({ ...switcher, thumbnailSize })} />
        </div>
      </div>
    </div>
  );
}
