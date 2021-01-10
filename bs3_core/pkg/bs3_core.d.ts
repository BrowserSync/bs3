/* tslint:disable */
/* eslint-disable */
export type BrowserSyncOutputMsg = 
 | { kind: "Listening"; payload: { bind_address: string } };

export type ServedFile = { path: string; web_path: string; referer: string | null };

export type ClientMsg = 
 | { kind: "Connect" } 
 | { kind: "Disconnect" } 
 | { kind: "Scroll"; payload: ScrollMsg } 
 | { kind: "FsNotify"; payload: FsNotify };

export type FsNotify = { item: ServedFile };

export type ScrollMsg = { x: number; y: number };

