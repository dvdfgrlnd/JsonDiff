import init, { find_diff } from "../pkg/jsondiff.js";


let r = `
            {
            "f1": "v2",
            "f8": "v0",
            "f9": false,
            "f2": {
                "f3": 456,
                "f4": {
                    "f0":123
                }
            },
            "f3": {
                "f5": "v6"
            }
        }`;
let r2 = `
            {
            "f1": "v2",
            "f8": "v1",
            "f9": true,
            "f2": {
                "f3": 456,
                "f4": {
                    "f0":122
                }
            },
            "f3": "v6"
        }`;
init()
    .then(() => {
        document.getElementById("computediff").addEventListener("click", function () {
            let r1 = document.getElementById("text1").value;
            let r2 = document.getElementById("text2").value;
            if (r1.length > 0 && r2.length > 0) {
                let output = find_diff(r1, r2);
                console.log("output", output);
                document.getElementById("jsonresult").innerHTML = output;
            }
        });
    });