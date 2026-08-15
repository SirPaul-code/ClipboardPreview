import type { ClipboardItem } from '../types';
import { formatClipboardTimestamp } from '../lib/clipboardFormat';

const labels = {
  text: 'Text',
  url: 'URL',
  code: 'Code',
  multiline: 'Text',
  image: 'Image'
} as const;

const markers = {
  text: 'TXT',
  url: 'URL',
  code: 'COD',
  multiline: 'TXT',
  image: 'IMG'
} as const;

export function ClipboardCard({
  item,
  selected = false,
  onClick
}: {
  item: ClipboardItem;
  selected?: boolean;
  onClick?: () => void;
}) {
  const timestamp = formatClipboardTimestamp(item.createdAt);

  return (
    <button
      type="button"
      className={`clip-card ${selected ? 'selected' : ''}`}
      onClick={onClick}
    >
      {item.type === 'image' && item.thumbnailDataUrl ? (
        <div className="clip-thumb">
          <img src={item.thumbnailDataUrl} alt="Clipboard thumbnail" />
        </div>
      ) : (
        <div className="clip-icon clip-kind" aria-hidden="true">
          {markers[item.type]}
        </div>
      )}
      <div className="clip-main">
        <div className="clip-title-row">
          <span className="clip-kind">{labels[item.type]}</span>
          {timestamp ? <time className="clip-meta" dateTime={item.createdAt}>{timestamp}</time> : null}
        </div>
        <div className={`clip-text ${item.type === 'code' ? 'mono' : ''}`}>
          {item.preview || (item.type === 'image' ? 'Clipboard image' : 'Empty text')}
        </div>
      </div>
    </button>
  );
}
