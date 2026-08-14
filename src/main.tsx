import React from 'react';
import ReactDOM from 'react-dom/client';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { SettingsApp } from './apps/SettingsApp';
import { QuickPreview } from './apps/QuickPreview';
import { HistoryOverlay } from './apps/HistoryOverlay';
import './styles.css';

const label = getCurrentWindow().label;
const App = label === 'quick-preview' ? QuickPreview : label === 'history-overlay' ? HistoryOverlay : SettingsApp;

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
