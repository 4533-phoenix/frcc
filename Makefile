.POSIX:

DATABASE_URL := "sqlite:./test.db?mode=rwc"

fresh:
	DATABASE_URL=${DATABASE_URL} sea-orm-cli migrate fresh

migrations:
	DATABASE_URL=${DATABASE_URL} sea-orm-cli migrate up

entities:
	DATABASE_URL=${DATABASE_URL} sea-orm-cli generate entity -l -o entity/src

all: migrations entities

dev: fresh entities

.PHONY: fresh_migrations migrations entities all dev

