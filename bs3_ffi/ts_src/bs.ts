import { join } from "path";
const CWD = join(__dirname, "../dist");
const m = require(CWD);

export function create(name: string = "default") {
    // m.hello("{}", (output: any) => {
    //     console.log('I got', output);
    // })
    return {
        start(config: string = "{}") {
            const asJson = JSON.stringify(config);
            m.start("{}", (output: any) => {
                console.log('graceful stop message from nodejs', output);
            });
        },
        stop(cb: () => void) {
            m.stop("http://localhost:8090", cb)
        }
    }
}

