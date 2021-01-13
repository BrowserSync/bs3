import { join } from "path";

const CWD = join(__dirname, "../dist");
const m = require(CWD);

(async () => {
    console.log("js->before");
    const r = await m.hello("{}", (str: string) => {
        console.log("js->cb---after..", str)
    });
    console.log("js->after");
})()
