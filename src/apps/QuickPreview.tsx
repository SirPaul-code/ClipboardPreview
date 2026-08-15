import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { CSSProperties } from 'react';
import { formatClipboardTimestamp } from '../lib/clipboardFormat';
import type { QuickPreviewPayload } from '../types';

export function QuickPreview() {
  const [payload, setPayload] = useState<QuickPreviewPayload | null>(null);

  useEffect(() => {
    let off: undefined | (() => void);
    listen<QuickPreviewPayload>('clipboard://quick-preview', (event) => setPayload(event.payload)).then(
      (listener) => (off = listener)
    );
    return () => off?.();
  }, []);

  const item = payload?.item;
  const isImage = item?.type === 'image';
  const timestamp = item ? formatClipboardTimestamp(item.createdAt) : '';

  return (
    <main
      className={`overlay-shell quick ${isImage ? 'quick-image' : ''}`}
      style={
        payload
          ? ({
              '--overlay-opacity': payload.appearance.overlayOpacity,
              '--overlay-radius': `${payload.appearance.cornerRadius}px`,
              '--overlay-font': `${payload.settings.fontSize}px`
            } as CSSProperties)
          : undefined
      }
    >
      {isImage ? (
        <div className="quick-image-stage">
          {item?.thumbnailDataUrl ? <img src={item.thumbnailDataUrl} alt="Clipboard preview" /> : null}
          <div className="quick-image-meta">
            <strong>Image</strong>
            <time dateTime={item?.createdAt}>{timestamp}</time>
          </div>
        </div>
      ) : (
        <div className="preview-content">
          {payload?.settings.showContentType ? (
            <div className="overlay-kicker">{item ? item.type.toUpperCase() : 'CLIPBOARD'}</div>
          ) : null}
          <div
            className={`preview-text ${item?.type === 'code' ? 'mono' : ''}`}
            style={{
              whiteSpace: payload?.settings.textWrapping ? 'pre-wrap' : 'nowrap',
              WebkitLineClamp: payload?.settings.maxLines
            }}
          >
            {item?.content?.slice(0, payload?.settings.maxCharacters) || 'Clipboard is empty'}
          </div>
          {timestamp ? <time className="preview-count" dateTime={item?.createdAt}>{timestamp}</time> : null}
        </div>
      )}
    </main>
  );
}
