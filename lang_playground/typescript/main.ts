console.log("hello from main.ts!");

let count: number = 0;
let isRunning: boolean = true;

while (isRunning) {

    if (count % 2 === 0) {
        for (let i: number = 0; i < count; i++) {
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
