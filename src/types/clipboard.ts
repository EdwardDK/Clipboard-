export type ContentType = 'text' | 'url' | 'code';
export interface ClipboardFilters { pinned?: boolean; groupId?: string; contentType?: ContentType; sourceApplication?: string; recent?: boolean; }
export interface ClipboardItem { id:string; content:string; contentType:ContentType; sourceApplication:string|null; windowTitle:string|null; createdAt:string; lastCopiedAt:string; copyCount:number; pinned:boolean; pinOrder:number|null; label:string|null; groupId:string|null; groupName:string|null; sensitive:boolean; }
export interface Group { id:string; name:string; kind:string; createdAt:string; itemCount:number; }
export interface RetentionSettings { maxHistorySize:number; deleteUnpinnedAfterDays:number; excludedContentTypes:string[]; excludedApplications:string[]; }
export interface ClipboardQuery { query?:string; filters?:ClipboardFilters; limit?:number; }
