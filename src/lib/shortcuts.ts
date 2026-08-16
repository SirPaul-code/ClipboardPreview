export const modifierOrder = ['Control', 'Meta', 'Alt', 'Shift'] as const;

const normalizeKey = (key: string) => key.length === 1 ? key.toUpperCase() : key;

export function shortcutFromEvent(event: KeyboardEvent): string | null {
  const mods: string[] = [];
  if (event.ctrlKey) mods.push('Ctrl');
  if (event.metaKey) mods.push('Cmd');
  if (event.altKey) mods.push('Alt');
  if (event.shiftKey) mods.push('Shift');
  if (['Control', 'Meta', 'Alt', 'Shift'].includes(event.key)) return null;
  const key = normalizeKey(event.key === ' ' ? 'Space' : event.key);
  return [...mods, key].join('+');
}

export function shortcutMatchesEvent(value: string, event: KeyboardEvent) {
  return shortcutFromEvent(event)?.toLowerCase() === value.trim().toLowerCase();
}

export const prettyShortcut = (value: string) => value.split('+').join(' + ');
