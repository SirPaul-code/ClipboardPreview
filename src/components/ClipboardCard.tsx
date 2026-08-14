import type { ClipboardItem } from '../types';

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

function formatBytes(bytes: number) {
  if (!bytes) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function ClipboardCard({
  item,
  selected = false,
  onClick
}: {
  item: ClipboardItem;
  selected?: boolean;
  onClick?: () => void;
}) {
  const imageMeta =
    item.type === 'image' && item.metadata.width && item.metadata.height
      ? `${item.metadata.width}×${item.metadata.height}`
      : null;
  const meta =
    imageMeta ??
    (item.metadata.characterCount
      ? `${item.metadata.characterCount} ch`
      : formatBytes(item.metadata.byteSize));

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
          {meta ? <span className="clip-meta">{meta}</span> : null}
        </div>
        <div className={`clip-text ${item.type === 'code' ? 'mono' : ''}`}>
          {item.preview || (item.type === 'image' ? 'Clipboard image' : 'Empty text')}
        </div>
      </div>
    </button>
  );
}
