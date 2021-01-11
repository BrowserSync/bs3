import { BrowserSyncOutputMsg } from "../../bs3_core/pkg/bs3_core";
import { spawn } from 'child_process';
import { join } from 'path';

const CWD = process.cwd();
const exec = join(CWD, "..", "target/debug/bs3");
const ls = spawn(exec, []);

ls.stdout.on('data', (data) => {
    console.log(`stdout: ${data}`);
    try {
        const parsed = toMsg(data);
        if (parsed) {
            handleMessage(parsed);
        }
    } catch (e) {
        console.error(`stdout parsed error: ${e}`);
    }
});

ls.stderr.on('data', (data) => {
    console.error(`stderr: ${data}`);
});

ls.on('close', (code) => {
    console.log(`child process exited with code ${code}`);
});


function toMsg(msg: any): BrowserSyncOutputMsg | undefined {
    try {
        return JSON.parse(msg);
    } catch (e) {
        console.error("could not parse a msg = %s", e);
        return undefined
    }
}

function handleMessage(msg: BrowserSyncOutputMsg) {
    switch (msg.kind) {
        case "Listening": {
            console.log("Listening on %O", `http://` + msg.payload.bind_address);
            break;
        }
        default: {
            console.log("got a message...%O", msg)
        }
    }
}