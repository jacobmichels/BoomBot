import subprocess
import os
from subprocess import PIPE
from datetime import datetime

# Ensure sqlx is installed
process = subprocess.run(["sqlx", "-V"])
if not process.check_returncode():
    print("sqlx cli tool not installed. Install it with cargo install sqlx-cli.")
    exit(-1)

# If database does not exist, create it and apply migrations
if not os.path.isfile("app.db"):
    print("Database does not exist. Creating it and applying migrations.")
    subprocess.run(["sqlx", "database", "setup"])
    exit(0)

# Get pending migrations
process = subprocess.run(
    ["sqlx", "migrate", "info"], stdout=PIPE)
process.check_returncode()

# Exit if there are no pending migrations
if not process.stdout.decode().__contains__("pending"):
    print("No migrations pending. Keeping current database.")
    exit(0)

print("There is at least one pending migration. Backing up the current database and running the migration(s).")

# Create backup folder and copy current database into it.
process = subprocess.run(["mkdir", "-p", "db_backups/"])
process.check_returncode()

process = subprocess.run(
    ["cp", "app.db", f"db_backups/{datetime.now()}.db"])
process.check_returncode()

# Recreate the database with the migrations applied
process = subprocess.run(["sqlx", "database", "reset", "-y"])
process.check_returncode()

print("New database setup.")
