import os
from flask import request


def lookup(cursor):
    uid = request.args["id"]
    cursor.execute(f"SELECT * FROM users WHERE id = {uid}")


def ping():
    host = request.args.get("host")
    os.system("ping -c 1 " + host)
