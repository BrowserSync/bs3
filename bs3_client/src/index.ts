import {webSocket} from "rxjs/webSocket";
import {ClientMsg} from "../../bs3_core/pkg/bs3_core";
import {buffer, debounceTime, filter, share, switchMap} from "rxjs/operators";
import {EMPTY, Observable, of} from "rxjs";

const wsUri = (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/__bs3/ws/';
const ws = webSocket<ClientMsg>(wsUri);

const fs = ws.pipe(
    filter(x => x.kind === "FsNotify"),
) as Observable<Evt<"FsNotify">>;

const fsSub = fs.pipe(
    filter(x => !x.payload.item.path.match(/.map$/)),
    buffer(fs.pipe(debounceTime(500))),
    share(),
);

const inject = [
    /.css$/,
    /.jpg$/,
    /.png$/,
];

type Effects =
    | {
        kind: 'Reload'
    };

const actions = fsSub.pipe(switchMap((events): Observable<Effects> => {
    if (events.every((evt) => inject.some(regex => evt.payload.item.path.match(regex)))) {
        console.log('all were injectable');
        return EMPTY;
    } else {
        return of({kind: "Reload"});
    }
})) as Observable<Effects>;

const sub = actions.subscribe((action) => {
    switch (action.kind) {
        case "Reload": {
            window.location.reload()
        }
    }
});


/**
 * Create a 'helper' type for 'extracting' ONE of the union's members
 * based on the 'kind' field
 */
type Evt<K extends ClientMsg["kind"]> = Extract<ClientMsg, { kind: K }>;
