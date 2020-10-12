const http = require("http");

for (let i = 0; i < 12; i++) {
    console.log(`Starting request ${i}`);
    http.get("http://127.0.0.1:8080", msg => {
        console.log(`Request ${i} ${msg.statusMessage}`);
    });
}