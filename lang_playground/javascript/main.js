console.log("hello from main.js!");

let count = 0;
let isRunning = true;

while (isRunning) {
    if (count % 2 === 0) {
        for (let i = 0; i < count; i++) {
            console.log(`${count}: 偶数`);
        }
    } else {
        console.log(`${count}: 奇数`);
    }

    switch (count) {
        case 3:
            console.log("end");
            isRunning = false;
            break;
    }

    count++;
}
