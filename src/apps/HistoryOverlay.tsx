import { useEffect, useMemo, useState } from 'react';
import type { CSSProperties } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Image as ImageIcon } from 'lucide-react';
import { backend } from '../lib/tauri';
import type { HistoryPayload, ImagePreviewPayload } from '../types';
import { ClipboardCard } from '../components/ClipboardCard';

export function HistoryOverlay() {
  const [payload, setPayload] = useState<HistoryPayload | null>(null);
  const [imagePreview, setImagePreview] = useState<ImagePreviewPayload | null>(null);
  const [imageLoading, setImageLoading] = useState(false);

  useEffect(() => {
    const offs: Array<() => void> = [];
    Promise.all(
      ['clipboard://history-show', 'clipboard://history-selection'].map((name) =>
        listen<HistoryPayload>(name, (event) => setPayload(event.payload))
      )
    ).then((listeners) => offs.push(...listeners));
    return () => offs.forEach((off) => off());
  }, []);

  const selected = useMemo(
    () => payload?.items[payload.selectedIndex] ?? null,
    [payload]
  );

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;
    setImagePreview(null);
    setImageLoading(false);

    if (selected?.type === 'image') {
      timer = window.setTimeout(async () => {
        setImageLoading(true);
        try {
          const preview = await backend.imagePreview(selected.id);
          if (!cancelled) setImagePreview(preview);
        } catch {
          if (!cancelled) setImagePreview(null);
        } finally {
          if (!cancelled) setImageLoading(false);
        }
      }, payload?.imagePreviewDelayMs ?? 650);
    }

    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [payload?.imagePreviewDelayMs, selected?.id, selected?.type]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!payload || payload.interactionMode !== 'sticky') return;
      if (['ArrowDown', 'j', 'J'].includes(event.key)) {
        event.preventDefault();
        void backend.navigate(1);
      } else if (['ArrowUp', 'k', 'K'].includes(event.key)) {
        event.preventDefault();
        void backend.navigate(-1);
      } else if (event.key === 'Enter') {
        event.preventDefault();
        void backend.accept();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        void backend.cancel();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [payload]);

  useEffect(() => {
    const onWheel = (event: WheelEvent) => {
      if (!payload || payload.interactionMode !== 'sticky') return;
      event.preventDefault();
      void backend.navigate(event.deltaY > 0 ? 1 : -1);
    };
    window.addEventListener('wheel', onWheel, { passive: false });
    return () => window.removeEventListener('wheel', onWheel);
  }, [payload]);

  const shortcut = payload?.shortcut ?? 'Tab';
  const holdMode = payload?.interactionMode === 'hold_release';
  const largePreviewPanel = payload?.largePreviewPanel ?? true;
  const compactImagePopover =
    !largePreviewPanel && selected?.type === 'image' && (imageLoading || imagePreview);

  const compactPreviewTop = useMemo(() => {
    if (!payload) return 8;
    const bodyHeight = Math.max(150, payload.items.length * 48 + 6);
    const desired = 8 + payload.selectedIndex * 48 - 42;
    return Math.min(Math.max(desired, 8), Math.max(8, bodyHeight - 196));
  }, [payload]);

  return (
    <main
      className={`overlay-shell switcher ${largePreviewPanel ? 'switcher-split' : 'switcher-compact'}`}
      style={
        payload
          ? ({
              '--overlay-opacity': payload.appearance.overlayOpacity,
              '--overlay-radius': `${payload.appearance.cornerRadius}px`
            } as CSSProperties)
          : undefined
      }
    >
      <header className="switcher-header">
        <div>
          <div className="switcher-title">Clipboard</div>
          <div className="switcher-subtitle">Recent text and images</div>
        </div>
        <div className="switcher-header-meta">
          <span>{payload?.totalItems ?? 0}</span>
          <kbd>{shortcut}</kbd>
        </div>
      </header>

      <div className="switcher-body">
        <section className="switcher-list">
          {payload?.items.length ? (
            payload.items.map((item, index) => (
              <ClipboardCard
                key={item.id}
                item={item}
                selected={index === payload.selectedIndex}
                onClick={() => void backend.selectItem(item.id)}
              />
            ))
          ) : (
            <div className="empty-state">
              <strong>Clipboard history is empty</strong>
              <span>Copy text or an image and it will appear here.</span>
            </div>
          )}
        </section>

        {largePreviewPanel ? (
          <aside className="switcher-detail">
            {!selected ? (
              <div className="detail-empty">Nothing selected</div>
            ) : selected.type === 'image' ? (
              <div className="image-detail">
                <div className={`image-stage ${imagePreview ? 'ready' : ''}`}>
                  {selected.thumbnailDataUrl ? (
                    <img
                      src={imagePreview?.dataUrl ?? selected.thumbnailDataUrl}
                      alt="Selected clipboard item"
                    />
                  ) : (
                    <ImageIcon size={32} />
                  )}
                </div>
                <div className="detail-caption">
                  <strong>
                    {selected.metadata.width}×{selected.metadata.height}
                  </strong>
                  <span>
                    {imagePreview
                      ? 'Preview'
                      : imageLoading
                        ? 'Loading…'
                        : 'Pause to preview'}
                  </span>
                </div>
              </div>
            ) : (
              <div className="text-detail">
                <div className="detail-kind">{selected.type}</div>
                <div className={`detail-text ${selected.type === 'code' ? 'mono' : ''}`}>
                  {selected.content || selected.preview || 'Empty text'}
                </div>
              </div>
            )}
          </aside>
        ) : null}

        {compactImagePopover && selected ? (
          <aside
            className={`image-preview-popover ${imagePreview ? 'ready' : ''}`}
            style={{ top: `${compactPreviewTop}px` }}
            aria-hidden="true"
          >
            <div className="image-preview-popover-stage">
              {selected.thumbnailDataUrl ? (
                <img
                  src={imagePreview?.dataUrl ?? selected.thumbnailDataUrl}
                  alt=""
                />
              ) : (
                <ImageIcon size={28} />
              )}
            </div>
            <div className="image-preview-popover-meta">
              <strong>{selected.metadata.width}×{selected.metadata.height}</strong>
              <span>{imagePreview ? 'Image preview' : 'Loading…'}</span>
            </div>
          </aside>
        ) : null}
      </div>

      <footer className="switcher-footer">
        {holdMode
          ? `Hold ${shortcut} · scroll · release`
          : '↑ ↓ / scroll · Enter select · Esc cancel'}
      </footer>
    </main>
  );
}
