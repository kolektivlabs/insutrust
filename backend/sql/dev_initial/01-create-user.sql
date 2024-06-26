-- User
CREATE TABLE IF NOT EXISTS "user" (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  username varchar(128) NOT NULL UNIQUE,
  user_typ varchar(32) NOT NULL,

  -- Auth
  pwd varchar(256),
  pwd_salt uuid NOT NULL DEFAULT gen_random_uuid(),
  token_salt uuid NOT NULL DEFAULT gen_random_uuid(),

  -- Timestamps
  cid bigint NOT NULL,
  ctime timestamp with time zone NOT NULL,
  mid bigint NOT NULL,
  mtime timestamp with time zone NOT NULL  
);
