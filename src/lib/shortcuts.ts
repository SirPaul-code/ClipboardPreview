export const modifierOrder = ['Control', 'Meta', 'Alt', 'Shift'] as const;

const normalizeKey = (key: string) => key.length === 1 ? key.toUpperCase() : key;

function physicalAlphaNumericKey(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  return null;
}

function keyFromEvent(event: KeyboardEvent): string {
  if (event.key === ' ') return 'Space';

  // Option/Alt can transform the printable character reported by WebKit (for
  // example Alt+O -> Œ on some layouts). For shortcut identity, keep the stable
  // physical alphanumeric key while preserving the modifier. The native macOS
  // event tap also sees the hardware-independent virtual key code, so recording
  // and runtime matching stay consistent across layout-generated characters.
  if (event.altKey && event.key.length === 1) {
    const physical = physicalAlphaNumericKey(event.code);
    if (physical) return physical;
  }

  return normalizeKey(event.key);
}

export function shortcutFromEvent(event: KeyboardEvent): string | null {
  const mods: string[] = [];
  if (event.ctrlKey) mods.push('Ctrl');
  if (event.metaKey) mods.push('Cmd');
  if (event.altKey) mods.push('Alt');
  if (event.shiftKey) mods.push('Shift');
  if (['Control', 'Meta', 'Alt', 'Shift'].includes(event.key)) return null;
  const key = keyFromEvent(event);
  return [...mods, key].join('+');
}

export function shortcutMatchesEvent(value: string, event: KeyboardEvent) {
  return shortcutFromEvent(event)?.toLowerCase() === value.trim().toLowerCase();
}

export const prettyShortcut = (value: string) => value.split('+').join(' + ');
