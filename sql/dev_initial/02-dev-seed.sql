-- User demo1
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

INSERT INTO "user" 
    (username, pwd, pwd_salt,           token_salt,         cid, ctime, mid, mtime) VALUES 
    ('demo1',  '',  uuid_generate_v4(), uuid_generate_v4(), 0,   now(), 0,   now());
