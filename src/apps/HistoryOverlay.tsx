import { useEffect, useMemo, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Image as ImageIcon } from 'lucide-react';
import { backend } from '../lib/tauri';
import { formatClipboardTimestamp } from '../lib/clipboardFormat';
import { shortcutMatchesEvent } from '../lib/shortcuts';
import { switcherCssVariables } from '../lib/switcherStyle';
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
      if (shortcutMatchesEvent(payload.nextShortcut, event)) {
        event.preventDefault();
        event.stopPropagation();
        void backend.navigate(1);
      } else if (shortcutMatchesEvent(payload.previousShortcut, event)) {
        event.preventDefault();
        event.stopPropagation();
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
  const previousShortcut = payload?.previousShortcut ?? 'ArrowUp';
  const nextShortcut = payload?.nextShortcut ?? 'ArrowDown';
  const holdMode = payload?.interactionMode === 'hold_release';
  const largePreviewPanel = payload?.largePreviewPanel ?? true;
  const compactImagePopover =
    !largePreviewPanel && selected?.type === 'image' && (imageLoading || imagePreview);
  const selectedTimestamp = selected ? formatClipboardTimestamp(selected.createdAt) : '';

  return (
    <main
      className={`overlay-shell switcher ${largePreviewPanel ? 'switcher-split' : 'switcher-compact'}`}
      style={payload ? switcherCssVariables(payload.appearance) : undefined}
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

      <div
        className="switcher-body"
        style={
          largePreviewPanel
            ? { gridTemplateColumns: 'minmax(0, .84fr) minmax(0, 1.16fr)' }
            : undefined
        }
      >
        <section className="switcher-list" style={{ justifyContent: 'flex-start' }}>
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
                  <strong>{selectedTimestamp}</strong>
                  <span>
                    {imagePreview
                      ? 'Image preview'
                      : imageLoading
                        ? 'Loading…'
                        : 'Pause to preview'}
                  </span>
                </div>
              </div>
            ) : (
              <div className="text-detail">
                <div className="detail-kind">{selected.type} · {selectedTimestamp}</div>
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
            style={{
              top: '8px',
              width: 'min(280px, calc(100% - 16px))',
              maxHeight: 'calc(100% - 16px)'
            }}
            aria-hidden="true"
          >
            <div
              className="image-preview-popover-stage"
              style={{ height: 'clamp(64px, calc(100vh - 120px), 220px)' }}
            >
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
              <strong>{selectedTimestamp}</strong>
              <span>{imagePreview ? 'Image preview' : 'Loading…'}</span>
            </div>
          </aside>
        ) : null}
      </div>

      <footer className="switcher-footer">
        {holdMode
          ? `Hold ${shortcut} · wheel / ${previousShortcut} ${nextShortcut} · release`
          : `${previousShortcut} ${nextShortcut} / scroll · Enter select · Esc cancel`}
      </footer>
    </main>
  );
}
