import {webSocket} from "rxjs/webSocket";
import {ClientMsg} from "../../bs3_core/pkg/bs3_core";
import {buffer, debounceTime, filter, share, switchMap} from "rxjs/operators";
import {Observable} from "rxjs";
import anymatch, {Matcher} from "anymatch";

const wsUri = (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/__bs3/ws/';
const ws = webSocket<ClientMsg>(wsUri);

const fs: Observable<Evt<"FsNotify">> = ws.pipe(
    filter(x => x.kind==="FsNotify"),
);

const sub = fs.pipe(
    filter(x => !x.payload.item.path.match(/.map$/)),
    buffer(fs.pipe(debounceTime(500))),
    share(),
);

const inject: Matcher = [
    /.css$/,
    /.jpg$/,
    /.png$/,
];

const action = sub.pipe(switchMap(events => {
    if (events.every((evt) => anymatch(inject, evt.payload.item.path))) {

    } else {
        console.log("should reload instead");
    }
})).subscribe();


/**
 * Create a 'helper' type for 'extracting' ONE of the union's members
 * based on the 'kind' field
 */
type Evt<K extends ClientMsg["kind"]> = Extract<ClientMsg, { kind: K }>;
