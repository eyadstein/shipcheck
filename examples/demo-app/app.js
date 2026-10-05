const query = `SELECT * FROM users WHERE id = ${id}`;
eval(userInput);
el.innerHTML = userInput;
document.cookie = "visited=1";
app.post("/signup", handler);
const mailer = require("nodemailer");
const stripe = require("stripe");
