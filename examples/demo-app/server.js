const express = require("express");
const { exec } = require("child_process");
const fs = require("fs");
const app = express();

app.get("/user", (req, res) => {
  const id = req.params.id;
  db.query(`SELECT * FROM users WHERE id = ${id}`);
});

app.get("/ping", (req, res) => {
  exec("ping -c 1 " + req.query.host);
});

app.get("/file", (req, res) => {
  const name = req.query.name;
  fs.readFile("./uploads/" + name, (err, data) => res.send(data));
});

app.get("/safe", (req, res) => {
  const id = parseInt(req.params.id, 10);
  db.query("SELECT * FROM users WHERE id = ?", [id]);
});
