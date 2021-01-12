import { join } from "path";

const CWD = join(__dirname, "../dist");
const m = require(CWD);

m.hello("{}", (output: any) => {
    console.log('I got', output);
})