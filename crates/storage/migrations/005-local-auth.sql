-- SPDX-License-Identifier: MPL-2.0
-- Machine-local; excluded from project/evidence/portable JSON.
CREATE TABLE local_environment_settings (
 project TEXT PRIMARY KEY REFERENCES projects(id), environment TEXT NOT NULL,
 settings TEXT NOT NULL CHECK(length(CAST(settings AS BLOB))<=65536)
);
CREATE TABLE local_sessions (
 project TEXT NOT NULL REFERENCES projects(id), environment TEXT NOT NULL,
 session TEXT NOT NULL, ephemeral INTEGER NOT NULL CHECK(ephemeral IN (0,1)),
 PRIMARY KEY(project,environment)
);
