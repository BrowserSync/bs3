/* tslint:disable */
/* eslint-disable */
export type ServedFile = { path: string; web_path: string; referer: string | null };

export type ClientMsg = 
 | { kind: "Connect" } 
 | { kind: "Disconnect" } 
 | { kind: "Scroll"; fields: ScrollMsg } 
 | { kind: "FsNotify"; fields: FsNotify };

export type FsNotify = { item: ServedFile };

export type ScrollMsg = { x: number; y: number };

