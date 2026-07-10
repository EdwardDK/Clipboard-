import { create } from 'zustand';
import { clipboardApi } from '../lib/tauri';
import type { ClipboardFilters, ClipboardItem, Group, RetentionSettings } from '../types/clipboard';

interface State {
  items: ClipboardItem[]; groups: Group[]; query: string; filters: ClipboardFilters; selectedId: string | null;
  loading: boolean; error: string | null; paused: boolean; settings: RetentionSettings | null;
  setQuery(query: string): void; setFilters(filters: ClipboardFilters): void; setSelectedId(id: string | null): void;
  refresh(): Promise<void>; loadGroups(): Promise<void>; copy(id: string): Promise<void>; remove(id: string): Promise<void>;
  pin(id: string, pinned: boolean): Promise<void>; assignGroup(id: string, groupId: string | null): Promise<void>;
  createGroup(name: string): Promise<void>; saveSettings(settings: RetentionSettings): Promise<void>; clear(): Promise<void>;
}
export const useClipboardStore = create<State>((set, get) => ({
  items: [], groups: [], query: '', filters: {}, selectedId: null, loading: true, error: null, paused: false, settings: null,
  setQuery: (query) => { set({ query }); void get().refresh(); }, setFilters: (filters) => { set({ filters }); void get().refresh(); }, setSelectedId: (selectedId) => set({ selectedId }),
  refresh: async () => { set({ loading: true }); try { const [items, paused] = await Promise.all([clipboardApi.list({ query: get().query, filters: get().filters, limit: 200 }), clipboardApi.isPaused()]); set({ items, paused, selectedId: items.some((item) => item.id === get().selectedId) ? get().selectedId : items[0]?.id ?? null }); } catch { set({ error: 'Clipboard+ could not reach its local service.' }); } finally { set({ loading: false }); } },
  loadGroups: async () => { try { set({ groups: await clipboardApi.groups() }); } catch { /* history remains usable */ } }, copy: (id) => clipboardApi.copy(id),
  remove: async (id) => { await clipboardApi.delete(id); await get().refresh(); }, pin: async (id, pinned) => { await clipboardApi.pin(id, pinned); await get().refresh(); },
  assignGroup: async (id, groupId) => { await clipboardApi.assignGroup(id, groupId); await Promise.all([get().refresh(), get().loadGroups()]); }, createGroup: async (name) => { await clipboardApi.createGroup(name); await get().loadGroups(); },
  saveSettings: async (settings) => { await clipboardApi.saveSettings(settings); set({ settings }); }, clear: async () => { await clipboardApi.clear(); await get().refresh(); },
}));
