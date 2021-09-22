import subprocess
from subprocess import PIPE
from datetime import datetime

process = subprocess.run(
    ["sqlx", "migrate", "info"], stdout=PIPE)
process.check_returncode()
if not process.stdout.decode().__contains__("pending"):
    print("No migrations pending. Keeping current database.")
    exit(0)

print("There is at least one pending migration. Backing up the current database and running the migration(s).")
process = subprocess.run(
    ["cp", "app.db", f"../boombot_db_backups/{datetime.now()}.db"])
process.check_returncode()
process = subprocess.run(["sqlx", "database", "setup"])
process.check_returncode()
print("New database setup.")
