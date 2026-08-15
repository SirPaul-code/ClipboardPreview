import type { CSSProperties } from 'react';
import type { AppSettings } from '../types';

export type SwitcherCssVariables = CSSProperties & Record<`--sw-${string}`, string | number>;

export function switcherCssVariables(appearance: AppSettings['appearance']): SwitcherCssVariables {
  const style = appearance.switcher;
  return {
    '--overlay-opacity': appearance.overlayOpacity,
    '--overlay-radius': `${appearance.cornerRadius}px`,
    '--sw-panel-bg': style.panelBackground,
    '--sw-row-bg': style.rowBackground,
    '--sw-selected-bg': style.selectedBackground,
    '--sw-border': style.borderColor,
    '--sw-selected-border': style.selectedBorderColor,
    '--sw-title-size': `${style.headerTitle.fontSize}px`,
    '--sw-title-color': style.headerTitle.color,
    '--sw-subtitle-size': `${style.headerSubtitle.fontSize}px`,
    '--sw-subtitle-color': style.headerSubtitle.color,
    '--sw-header-meta-size': `${style.headerMeta.fontSize}px`,
    '--sw-header-meta-color': style.headerMeta.color,
    '--sw-item-type-size': `${style.itemType.fontSize}px`,
    '--sw-item-type-color': style.itemType.color,
    '--sw-item-content-size': `${style.itemContent.fontSize}px`,
    '--sw-item-content-color': style.itemContent.color,
    '--sw-item-meta-size': `${style.itemMeta.fontSize}px`,
    '--sw-item-meta-color': style.itemMeta.color,
    '--sw-detail-content-size': `${style.detailContent.fontSize}px`,
    '--sw-detail-content-color': style.detailContent.color,
    '--sw-detail-meta-size': `${style.detailMeta.fontSize}px`,
    '--sw-detail-meta-color': style.detailMeta.color,
    '--sw-footer-size': `${style.footer.fontSize}px`,
    '--sw-footer-color': style.footer.color,
    '--sw-row-height': `${style.rowHeight}px`,
    '--sw-thumb-size': `${style.thumbnailSize}px`
  };
}
