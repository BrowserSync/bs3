import {create} from "./bs";

function init() {
    const bs = create();
    bs.start("{}");
    setTimeout(() => {
        console.log('stopping via node js api');
        bs.stop(() => {
            console.log('stopped via node js api');
        });
    }, 2000);
}

init();